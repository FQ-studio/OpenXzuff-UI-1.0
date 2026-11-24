//! Minimal polling 16550 UART HAL for the QEMU virt ns16550a device.
//!
//! This driver deliberately accepts an already translated and supervisor-mapped
//! MMIO window. It does not discover FDT resources or configure PLIC interrupts.
use crate::mmio::{MmioError, MmioRegion};
use core::sync::atomic::{compiler_fence, Ordering};

const RBR_THR_DLL: u64 = 0;
const IER_DLM: u64 = 1;
const FCR: u64 = 2;
const LCR: u64 = 3;
const MCR: u64 = 4;
const LSR: u64 = 5;

const LCR_DLAB: u8 = 1 << 7;
const LCR_8N1: u8 = 0x03;
const MCR_LOOPBACK: u8 = 1 << 4;
const LSR_DATA_READY: u8 = 1 << 0;
const LSR_THR_EMPTY: u8 = 1 << 5;
const LSR_TRANSMITTER_EMPTY: u8 = 1 << 6;
const POLL_LIMIT: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UartError {
    Mmio(MmioError),
    InvalidDivisor,
    Timeout,
    ReadbackMismatch,
}

/// Narrow polling-console contract shared by supported UART IP drivers.
pub trait UartConsole {
    fn write_byte(&mut self, byte: u8) -> Result<(), UartError>;
    fn read_byte(&mut self) -> Result<Option<u8>, UartError>;
    fn flush(&mut self) -> Result<(), UartError>;
}

/// 16550-compatible UART using byte-wide registers at offsets 0..=7.
pub struct Uart16550 {
    region: MmioRegion,
}

impl Uart16550 {
    /// Construct after the translated UART window is mapped supervisor RW/NX.
    ///
    /// # Safety
    /// The complete MMIO window must remain mapped and classified as device IO
    /// for the lifetime of this driver.
    pub unsafe fn new(region: MmioRegion) -> Result<Self, UartError> {
        region.register_address(7, 1).map_err(UartError::Mmio)?;
        Ok(Self { region })
    }

    fn address(&self, offset: u64) -> Result<usize, UartError> {
        let address = self.region.register_address(offset, 1).map_err(UartError::Mmio)?;
        if address > usize::MAX as u64 {
            return Err(UartError::Mmio(MmioError::AddressOverflow));
        }
        Ok(address as usize)
    }

    #[inline]
    fn io_fence() {
        #[cfg(target_arch = "riscv64")]
        unsafe {
            core::arch::asm!("fence iorw, iorw", options(nostack));
        }
        #[cfg(not(target_arch = "riscv64"))]
        compiler_fence(Ordering::SeqCst);
    }

    fn read_register(&self, offset: u64) -> Result<u8, UartError> {
        let pointer = self.address(offset)? as *const u8;
        Self::io_fence();
        let value = unsafe { core::ptr::read_volatile(pointer) };
        Self::io_fence();
        Ok(value)
    }

    fn write_register(&self, offset: u64, value: u8) -> Result<(), UartError> {
        let pointer = self.address(offset)? as *mut u8;
        Self::io_fence();
        unsafe { core::ptr::write_volatile(pointer, value); }
        Self::io_fence();
        Ok(())
    }

    fn wait_status(&self, mask: u8) -> Result<(), UartError> {
        for _ in 0..POLL_LIMIT {
            if self.read_register(LSR)? & mask == mask {
                return Ok(());
            }
            core::hint::spin_loop();
        }
        Err(UartError::Timeout)
    }

    /// Configure 8-N-1 and a caller-supplied divisor; no board clock is guessed.
    pub fn configure(&mut self, divisor: u16) -> Result<(), UartError> {
        if divisor == 0 { return Err(UartError::InvalidDivisor); }
        self.write_register(IER_DLM, 0)?;
        self.write_register(LCR, LCR_DLAB)?;
        self.write_register(RBR_THR_DLL, divisor as u8)?;
        self.write_register(IER_DLM, (divisor >> 8) as u8)?;
        self.write_register(LCR, LCR_8N1)?;
        self.write_register(FCR, 0x07)?;
        self.write_register(MCR, 0)?;
        Ok(())
    }

    /// QEMU virt smoke probe: use the 16550 internal loopback, then restore
    /// normal transmit mode. This performs real volatile IO against the UART.
    pub fn loopback_probe(&mut self, byte: u8) -> Result<u8, UartError> {
        self.configure(1)?;
        self.write_register(MCR, MCR_LOOPBACK)?;
        self.wait_status(LSR_THR_EMPTY)?;
        self.write_register(RBR_THR_DLL, byte)?;
        self.wait_status(LSR_DATA_READY)?;
        let received = self.read_register(RBR_THR_DLL)?;
        self.write_register(MCR, 0)?;
        self.write_register(FCR, 0x07)?;
        self.flush()?;
        if received != byte { return Err(UartError::ReadbackMismatch); }
        Ok(received)
    }
}

impl UartConsole for Uart16550 {
    fn write_byte(&mut self, byte: u8) -> Result<(), UartError> {
        self.wait_status(LSR_THR_EMPTY)?;
        self.write_register(RBR_THR_DLL, byte)
    }

    fn read_byte(&mut self) -> Result<Option<u8>, UartError> {
        if self.read_register(LSR)? & LSR_DATA_READY == 0 {
            return Ok(None);
        }
        self.read_register(RBR_THR_DLL).map(Some)
    }

    fn flush(&mut self) -> Result<(), UartError> {
        self.wait_status(LSR_TRANSMITTER_EMPTY)
    }
}

#[cfg(test)]
mod tests {
    use super::{Uart16550, UartError};
    use crate::mmio::{MmioError, MmioRegion};

    #[test]
    fn uart_requires_the_complete_eight_register_window() {
        let short = MmioRegion::new(0x1000, 7, 1, 1).unwrap();
        assert!(matches!(
            unsafe { Uart16550::new(short) },
            Err(UartError::Mmio(MmioError::OutOfBounds))
        ));
    }

    #[test]
    fn uart_accepts_byte_registers_and_rejects_offset_eight() {
        let region = MmioRegion::new(0x1000, 8, 1, 1).unwrap();
        let uart = unsafe { Uart16550::new(region).unwrap() };
        assert_eq!(uart.address(0), Ok(0x1000));
        assert_eq!(uart.address(7), Ok(0x1007));
        assert_eq!(uart.address(8), Err(UartError::Mmio(MmioError::OutOfBounds)));
    }

    #[test]
    fn uart_rejects_zero_baud_divisor_before_mmio_access() {
        let mut memory = [0u8; 8];
        let region = MmioRegion::new(memory.as_mut_ptr() as usize as u64, 8, 1, 1).unwrap();
        let mut uart = unsafe { Uart16550::new(region).unwrap() };
        assert_eq!(uart.configure(0), Err(UartError::InvalidDivisor));
        assert_eq!(memory, [0; 8]);
    }
}
