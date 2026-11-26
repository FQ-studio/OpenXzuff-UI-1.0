//! Minimal polling 16550 UART HAL for the QEMU virt ns16550a device.
//!
//! This driver deliberately accepts an already translated and supervisor-mapped
//! MMIO window. It does not discover FDT resources or configure PLIC interrupts.
use crate::mmio::{MmioError, MmioRegion};
#[cfg(not(target_arch = "riscv64"))]
use core::sync::atomic::{compiler_fence, Ordering};

const RBR_THR_DLL: u64 = 0;
const IER_DLM: u64 = 1;
const FCR: u64 = 2;
const IIR: u64 = 2;
const LCR: u64 = 3;
const MCR: u64 = 4;
const LSR: u64 = 5;

const LCR_DLAB: u8 = 1 << 7;
const LCR_8N1: u8 = 0x03;
const MCR_LOOPBACK: u8 = 1 << 4;
const LSR_DATA_READY: u8 = 1 << 0;
const LSR_OVERRUN_ERROR: u8 = 1 << 1;
const LSR_PARITY_ERROR: u8 = 1 << 2;
const LSR_FRAMING_ERROR: u8 = 1 << 3;
const LSR_BREAK_INTERRUPT: u8 = 1 << 4;
const LSR_FIFO_ERROR: u8 = 1 << 7;
const LSR_ERROR_MASK: u8 = LSR_OVERRUN_ERROR | LSR_PARITY_ERROR
    | LSR_FRAMING_ERROR | LSR_BREAK_INTERRUPT | LSR_FIFO_ERROR;
const IER_RX_DATA_AVAILABLE: u8 = 1 << 0;
const LSR_THR_EMPTY: u8 = 1 << 5;
const LSR_TRANSMITTER_EMPTY: u8 = 1 << 6;
const POLL_LIMIT: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UartError {
    Mmio(MmioError),
    InvalidDivisor,
    DivisorLatchActive,
    Timeout,
    ReadbackMismatch,
}

/// Narrow polling-console contract shared by supported UART IP drivers.
pub trait UartConsole {
    fn write_byte(&mut self, byte: u8) -> Result<(), UartError>;
    fn read_byte(&mut self) -> Result<Option<u8>, UartError>;
    fn flush(&mut self) -> Result<(), UartError>;
}

/// Bounded result of servicing one 16550 receive interrupt.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UartRxBatch {
    pub bytes: usize,
    pub line_status: u8,
    pub more_pending: bool,
    pub interrupt_handled: bool,
}

/// Drain bytes from one receive interrupt. The output buffer bounds ISR work;
/// if it fills, more_pending tells the caller that the device may retrap.
fn drain_rx_fifo(
    identification: u8,
    output: &mut [u8],
    mut read_lsr: impl FnMut() -> Result<u8, UartError>,
    mut read_rbr: impl FnMut() -> Result<u8, UartError>,
) -> Result<UartRxBatch, UartError> {
    let mut batch = UartRxBatch::default();
    if identification & 1 != 0 {
        return Ok(batch);
    }
    let cause = (identification >> 1) & 0x07;
    if !matches!(cause, 0x02 | 0x03 | 0x06) {
        return Ok(batch);
    }

    while batch.bytes < output.len() {
        let status = read_lsr()?;
        batch.line_status |= status & LSR_ERROR_MASK;
        if status & LSR_DATA_READY == 0 {
            break;
        }
        output[batch.bytes] = read_rbr()?;
        batch.bytes += 1;
    }

    if batch.bytes == output.len() {
        let status = read_lsr()?;
        batch.line_status |= status & LSR_ERROR_MASK;
        batch.more_pending = status & LSR_DATA_READY != 0;
    }

    batch.interrupt_handled = batch.bytes != 0
        || batch.line_status != 0
        || batch.more_pending;
    Ok(batch)
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

    /// Enable only the receiver-data-available interrupt, preserving other IER bits.
    pub fn enable_rx_data_available_interrupt(&mut self) -> Result<(), UartError> {
        if self.read_register(LCR)? & LCR_DLAB != 0 {
            return Err(UartError::DivisorLatchActive);
        }
        let enabled = self.read_register(IER_DLM)? | IER_RX_DATA_AVAILABLE;
        self.write_register(IER_DLM, enabled)
    }

    /// Drain a bounded batch for receiver-data, character-timeout or line-status
    /// interrupts. The caller must service another interrupt if more_pending.
    pub fn drain_rx_interrupt(&mut self, output: &mut [u8]) -> Result<UartRxBatch, UartError> {
        let identification = self.read_register(IIR)?;
        drain_rx_fifo(
            identification,
            output,
            || self.read_register(LSR),
            || self.read_register(RBR_THR_DLL),
        )
    }

    /// Compatibility helper for callers that intentionally consume one byte.
    pub fn service_rx_interrupt(&mut self) -> Result<Option<u8>, UartError> {
        let mut byte = [0u8; 1];
        let batch = self.drain_rx_interrupt(&mut byte)?;
        Ok(if batch.bytes == 0 { None } else { Some(byte[0]) })
    }

    /// QEMU virt smoke probe: use the 16550 internal loopback, then restore
    /// normal transmit mode. This performs real volatile IO against the UART.
    pub fn loopback_probe(&mut self, byte: u8) -> Result<u8, UartError> {
        let saved_lcr = self.read_register(LCR)?;
        if saved_lcr & LCR_DLAB != 0 { return Err(UartError::DivisorLatchActive); }
        let saved_ier = self.read_register(IER_DLM)?;
        let saved_mcr = self.read_register(MCR)?;
        self.write_register(IER_DLM, 0)?;
        let probe = (|| {
            self.write_register(MCR, saved_mcr | MCR_LOOPBACK)?;
            self.wait_status(LSR_THR_EMPTY)?;
            self.write_register(RBR_THR_DLL, byte)?;
            self.wait_status(LSR_DATA_READY)?;
            let received = self.read_register(RBR_THR_DLL)?;
            self.flush()?;
            Ok(received)
        })();
        let restore = self.write_register(MCR, saved_mcr)
            .and_then(|_| self.write_register(IER_DLM, saved_ier));
        restore?;
        let received = probe?;
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
    use super::{
        drain_rx_fifo, Uart16550, UartError, UartRxBatch, IIR, LCR_8N1, LSR,
        LSR_DATA_READY, LSR_OVERRUN_ERROR, RBR_THR_DLL,
    };
    use crate::mmio::{MmioError, MmioRegion};
    use std::{cell::{Cell, RefCell}, collections::VecDeque};

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

    #[test]
    fn uart_rx_interrupt_enable_preserves_existing_ier_bits() {
        let mut memory = [0u8; 8];
        memory[1] = 0x04;
        memory[3] = LCR_8N1;
        let region = MmioRegion::new(memory.as_mut_ptr() as usize as u64, 8, 1, 1).unwrap();
        let mut uart = unsafe { Uart16550::new(region).unwrap() };
        uart.enable_rx_data_available_interrupt().unwrap();
        assert_eq!(memory[1], 0x05);
    }

    #[test]
    fn uart_rx_interrupt_service_reads_rbr_only_for_receive_causes() {
        let mut memory = [0u8; 8];
        memory[IIR as usize] = 0x04; // Receiver data available.
        memory[LSR as usize] = LSR_DATA_READY;
        memory[RBR_THR_DLL as usize] = 0x5a;
        let region = MmioRegion::new(memory.as_mut_ptr() as usize as u64, 8, 1, 1).unwrap();
        let mut uart = unsafe { Uart16550::new(region).unwrap() };
        assert_eq!(uart.service_rx_interrupt(), Ok(Some(0x5a)));

        memory[IIR as usize] = 0x01; // No interrupt pending.
        assert_eq!(uart.service_rx_interrupt(), Ok(None));
    }

    #[test]
    fn uart_rx_interrupt_drains_all_available_fifo_bytes_in_order() {
        let fifo = RefCell::new(VecDeque::from([0x5a, 0x6b, 0x7c, 0x8d]));
        let mut output = [0u8; 8];
        let batch = drain_rx_fifo(
            0x04,
            &mut output,
            || Ok(if fifo.borrow().is_empty() { 0 } else { LSR_DATA_READY }),
            || Ok(fifo.borrow_mut().pop_front().unwrap()),
        ).unwrap();

        assert_eq!(batch, UartRxBatch {
            bytes: 4,
            line_status: 0,
            more_pending: false,
            interrupt_handled: true,
        });
        assert_eq!(&output[..batch.bytes], &[0x5a, 0x6b, 0x7c, 0x8d]);
        assert!(fifo.borrow().is_empty());
    }

    #[test]
    fn uart_rx_interrupt_reports_pending_data_when_output_buffer_fills() {
        let fifo = RefCell::new(VecDeque::from([0x11, 0x22, 0x33]));
        let mut output = [0u8; 2];
        let batch = drain_rx_fifo(
            0x0c, // Character-timeout receive interrupt.
            &mut output,
            || Ok(if fifo.borrow().is_empty() { 0 } else { LSR_DATA_READY }),
            || Ok(fifo.borrow_mut().pop_front().unwrap()),
        ).unwrap();

        assert_eq!(batch.bytes, 2);
        assert!(batch.more_pending);
        assert!(batch.interrupt_handled);
        assert_eq!(&output, &[0x11, 0x22]);
        assert_eq!(fifo.borrow().front(), Some(&0x33));
    }

    #[test]
    fn uart_line_status_interrupt_preserves_error_flags_and_drains_data() {
        let fifo = RefCell::new(VecDeque::from([0x42]));
        let status_reads = Cell::new(0usize);
        let mut output = [0u8; 2];
        let batch = drain_rx_fifo(
            0x06, // Receiver line-status interrupt.
            &mut output,
            || {
                let read = status_reads.get();
                status_reads.set(read + 1);
                if read == 0 {
                    Ok(LSR_DATA_READY | LSR_OVERRUN_ERROR)
                } else if fifo.borrow().is_empty() {
                    Ok(0)
                } else {
                    Ok(LSR_DATA_READY)
                }
            },
            || Ok(fifo.borrow_mut().pop_front().unwrap()),
        ).unwrap();

        assert_eq!(batch.bytes, 1);
        assert_eq!(batch.line_status, LSR_OVERRUN_ERROR);
        assert!(batch.interrupt_handled);
        assert_eq!(output[0], 0x42);
    }

    #[test]
    fn uart_loopback_refuses_to_touch_registers_when_dlab_is_active() {
        let mut memory = [0u8; 8];
        memory[1] = 0x05;
        memory[3] = 0x80;
        memory[4] = 0x02;
        let region = MmioRegion::new(memory.as_mut_ptr() as usize as u64, 8, 1, 1).unwrap();
        let mut uart = unsafe { Uart16550::new(region).unwrap() };
        assert_eq!(uart.loopback_probe(0x5a), Err(UartError::DivisorLatchActive));
        assert_eq!(memory[1], 0x05);
        assert_eq!(memory[3], 0x80);
        assert_eq!(memory[4], 0x02);
    }
}
