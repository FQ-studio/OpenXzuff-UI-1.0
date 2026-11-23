//! Single-hart PLIC interrupt dispatch for the Gravity RV64 kernel.
use crate::plic::{PlicError, PlicLayout, PlicMmio};
use core::sync::atomic::{AtomicBool, Ordering};

pub const MAX_PLIC_SOURCES: usize = 1023;
pub type PlicIrqHandler = fn(u32) -> bool;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IrqError { AlreadyInstalled, NotInstalled, Plic(PlicError) }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlicDispatch { Idle, Handled(u32), Masked(u32) }

/// Single-hart controller. Registration is boot-time only; SMP needs an IRQ-save
/// lock around the handler table and PLIC enable-word read/modify/write.
pub struct PlicInterruptController<'a> {
    mmio: PlicMmio,
    handlers: &'a mut [Option<PlicIrqHandler>; MAX_PLIC_SOURCES + 1],
}

impl<'a> PlicInterruptController<'a> {
    /// # Safety
    /// The region must remain identity-mapped supervisor RW/NX with device PMA
    /// semantics for the controller lifetime.
    pub unsafe fn new(
        layout: PlicLayout,
        context: u32,
        handlers: &'a mut [Option<PlicIrqHandler>; MAX_PLIC_SOURCES + 1],
    ) -> Result<Self, PlicError> {
        let mmio = PlicMmio::new(layout, context)?;
        mmio.set_threshold(0)?;
        if mmio.threshold()? != 0 { return Err(PlicError::ReadbackMismatch); }
        Ok(Self { mmio, handlers })
    }

    /// Handler returns true only after the device has deasserted its IRQ.
    /// False or missing handlers are masked before the PLIC claim is completed.
    pub fn register_handler(&mut self, source: u32, priority: u32, handler: PlicIrqHandler)
        -> Result<(), PlicError>
    {
        if source == 0 || source as usize > MAX_PLIC_SOURCES {
            return Err(PlicError::InvalidSourceId);
        }
        self.mmio.set_priority(source, priority)?;
        self.handlers[source as usize] = Some(handler);
        if let Err(error) = self.mmio.set_source_enabled(source, true) {
            self.handlers[source as usize] = None;
            return Err(error);
        }
        Ok(())
    }

    /// Claim and dispatch one source; additional pending sources retrap after sret.
    pub fn dispatch_one(&mut self) -> Result<PlicDispatch, PlicError> {
        let source = self.mmio.claim()?;
        if source == 0 { return Ok(PlicDispatch::Idle); }
        let handled = self.handlers[source as usize]
            .map(|callback| callback(source)).unwrap_or(false);
        let mask_result = if handled { Ok(()) } else { self.mmio.set_source_enabled(source, false) };
        let complete_result = self.mmio.complete(source);
        mask_result?;
        complete_result?;
        Ok(if handled { PlicDispatch::Handled(source) } else { PlicDispatch::Masked(source) })
    }
}

static mut GLOBAL_HANDLERS: [Option<PlicIrqHandler>; MAX_PLIC_SOURCES + 1] = [None; MAX_PLIC_SOURCES + 1];
static mut GLOBAL_CONTROLLER: Option<PlicInterruptController<'static>> = None;
static HANDLER_REGISTERED: AtomicBool = AtomicBool::new(false);

/// Install the selected boot-hart S-mode context after its Sv39 mapping is active.
///
/// # Safety
/// Caller must uphold PlicInterruptController::new mapping and single-hart rules.
pub unsafe fn install(layout: PlicLayout, context: u32) -> Result<(), IrqError> {
    let slot = core::ptr::addr_of_mut!(GLOBAL_CONTROLLER);
    if (*slot).is_some() { return Err(IrqError::AlreadyInstalled); }
    // Keep the 1024-entry table in BSS, not in this 16 KiB boot stack frame.
    let handlers: &'static mut [Option<PlicIrqHandler>; MAX_PLIC_SOURCES + 1] =
        &mut *core::ptr::addr_of_mut!(GLOBAL_HANDLERS);
    *slot = Some(
        PlicInterruptController::new(layout, context, handlers).map_err(IrqError::Plic)?
    );
    Ok(())
}

/// Register a device handler before runtime::start enables supervisor interrupts.
pub fn register_handler(source: u32, priority: u32, handler: PlicIrqHandler) -> Result<(), IrqError> {
    let slot = unsafe { core::ptr::addr_of_mut!(GLOBAL_CONTROLLER) };
    let controller = unsafe { (*slot).as_mut().ok_or(IrqError::NotInstalled)? };
    controller.register_handler(source, priority, handler).map_err(IrqError::Plic)?;
    HANDLER_REGISTERED.store(true, Ordering::Release);
    Ok(())
}

pub fn has_registered_handlers() -> bool {
    HANDLER_REGISTERED.load(Ordering::Acquire)
}

/// Called from S-origin and U-origin supervisor external-interrupt traps.
/// SIE is cleared on trap entry; nested dispatch and SMP are not supported.
pub fn dispatch_external() -> Result<PlicDispatch, IrqError> {
    let slot = unsafe { core::ptr::addr_of_mut!(GLOBAL_CONTROLLER) };
    let controller = unsafe { (*slot).as_mut().ok_or(IrqError::NotInstalled)? };
    controller.dispatch_one().map_err(IrqError::Plic)
}

#[cfg(test)]
mod tests {
    use super::{PlicDispatch, PlicInterruptController};
    use crate::{mmio::MmioRegion, plic::PlicLayout};
    use core::sync::atomic::{AtomicUsize, Ordering};

    static LAST_SOURCE: AtomicUsize = AtomicUsize::new(0);
    fn record_source(source: u32) -> bool {
        LAST_SOURCE.store(source as usize, Ordering::Release);
        true
    }

    fn controller_memory() -> (std::vec::Vec<u64>, PlicLayout) {
        let mut memory = std::vec![0u64; 0x0020_2000 / 8];
        let base = memory.as_mut_ptr() as usize as u64;
        let region = MmioRegion::new(base, 0x0020_2000, 8, 4).unwrap();
        let layout = PlicLayout::new(region, 95, 2).unwrap();
        (memory, layout)
    }

    #[test]
    fn dispatch_calls_registered_handler_and_completes_source() {
        LAST_SOURCE.store(0, Ordering::Release);
        let (memory, layout) = controller_memory();
        let mut handlers = [None; super::MAX_PLIC_SOURCES + 1];
        let mut plic = unsafe { PlicInterruptController::new(layout, 1, &mut handlers).unwrap() };
        let base = memory.as_ptr() as usize as u64;
        plic.register_handler(10, 1, record_source).unwrap();
        unsafe { core::ptr::write_volatile((base + 0x0020_1004) as *mut u32, 10); }
        assert_eq!(plic.dispatch_one(), Ok(PlicDispatch::Handled(10)));
        assert_eq!(LAST_SOURCE.load(Ordering::Acquire), 10);
        assert_eq!(unsafe { core::ptr::read_volatile((base + 0x0020_1004) as *const u32) }, 10);
        assert_eq!(unsafe { core::ptr::read_volatile((base + 0x28) as *const u32) }, 1);
        assert_eq!(unsafe { core::ptr::read_volatile((base + 0x2080) as *const u32) } & (1 << 10), 1 << 10);
    }

    #[test]
    fn dispatch_masks_unregistered_source_and_preserves_other_enable_bits() {
        let (memory, layout) = controller_memory();
        let mut handlers = [None; super::MAX_PLIC_SOURCES + 1];
        let mut plic = unsafe { PlicInterruptController::new(layout, 1, &mut handlers).unwrap() };
        let base = memory.as_ptr() as usize as u64;
        unsafe {
            core::ptr::write_volatile((base + 0x2080) as *mut u32, (1 << 10) | (1 << 31));
            core::ptr::write_volatile((base + 0x0020_1004) as *mut u32, 10);
        }
        assert_eq!(plic.dispatch_one(), Ok(PlicDispatch::Masked(10)));
        assert_eq!(unsafe { core::ptr::read_volatile((base + 0x2080) as *const u32) }, 1 << 31);
        assert_eq!(unsafe { core::ptr::read_volatile((base + 0x0020_1004) as *const u32) }, 10);
    }

    #[test]
    fn controller_keeps_large_handler_table_out_of_its_stack_footprint() {
        assert!(core::mem::size_of::<PlicInterruptController<'_>>() < 128);
    }

    #[test]
    fn dispatch_returns_idle_when_claim_register_is_zero() {
        let (_memory, layout) = controller_memory();
        let mut handlers = [None; super::MAX_PLIC_SOURCES + 1];
        let mut plic = unsafe { PlicInterruptController::new(layout, 1, &mut handlers).unwrap() };
        assert_eq!(plic.dispatch_one(), Ok(PlicDispatch::Idle));
    }
}
