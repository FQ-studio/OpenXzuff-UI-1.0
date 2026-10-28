//! RV64 callee-saved context switch ABI.
//! This primitive is not yet wired to user-mode tasks or the trap return path.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SavedContext {
    pub ra: usize,
    pub sp: usize,
    pub s0: usize,
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
}

extern "C" {
    /// Saves the current kernel callee-saved context into old and restores next.
    /// Both pointers must point to valid, aligned SavedContext values.
    pub fn gravity_context_switch(old: *mut SavedContext, next: *const SavedContext);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn context_layout_matches_assembly_offsets() {
        assert_eq!(core::mem::size_of::<SavedContext>(), 14 * core::mem::size_of::<usize>());
        assert_eq!(core::mem::offset_of!(SavedContext, ra), 0);
        assert_eq!(core::mem::offset_of!(SavedContext, sp), 8);
        assert_eq!(core::mem::offset_of!(SavedContext, s0), 16);
        assert_eq!(core::mem::offset_of!(SavedContext, s11), 104);
    }
}
