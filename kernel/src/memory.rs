use core::mem::size_of;

pub const MAX_REGIONS: usize = 32;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Region {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryMap {
    pub regions: [Region; MAX_REGIONS],
    pub count: usize,
}

impl MemoryMap {
    pub const fn new() -> Self {
        Self { regions: [Region { start: 0, end: 0 }; MAX_REGIONS], count: 0 }
    }

    pub fn add(&mut self, start: usize, end: usize) -> Result<(), ()> {
        if start == 0 || end <= start || self.count >= MAX_REGIONS { return Err(()); }
        for r in self.regions[..self.count].iter() {
            if start < r.end && end > r.start { return Err(()); }
        }
        let mut i = self.count;
        while i > 0 && self.regions[i - 1].start > start {
            self.regions[i] = self.regions[i - 1];
            i -= 1;
        }
        self.regions[i] = Region { start, end };
        self.count += 1;
        Ok(())
    }

    pub fn reserve(&mut self, start: usize, end: usize) -> Result<(), ()> {
        if start == 0 || end <= start { return Err(()); }
        // Mutate a copy first so capacity failures never partially change the map.
        let mut candidate = *self;
        candidate.reserve_inner(start, end)?;
        *self = candidate;
        Ok(())
    }

    fn reserve_inner(&mut self, start: usize, end: usize) -> Result<(), ()> {
        let mut i = 0;
        while i < self.count {
            let r = self.regions[i];
            if end <= r.start || start >= r.end { i += 1; continue; }
            let cut_start = start.max(r.start);
            let cut_end = end.min(r.end);
            if cut_start == r.start && cut_end == r.end {
                for j in i + 1..self.count { self.regions[j - 1] = self.regions[j]; }
                self.count -= 1;
            } else if cut_start == r.start {
                self.regions[i].start = cut_end;
                i += 1;
            } else if cut_end == r.end {
                self.regions[i].end = cut_start;
                i += 1;
            } else {
                if self.count >= MAX_REGIONS { return Err(()); }
                for j in (i + 1..=self.count).rev() { self.regions[j] = self.regions[j - 1]; }
                self.regions[i].end = cut_start;
                self.regions[i + 1] = Region { start: cut_end, end: r.end };
                self.count += 1;
                i += 2;
            }
        }
        Ok(())
    }

    pub fn usable_bytes(&self) -> usize {
        self.regions[..self.count].iter().fold(0usize, |sum, r| sum.saturating_add(r.end - r.start))
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BootAllocator {
    pub next: usize,
    pub limit: usize,
    pub allocations: usize,
}

impl BootAllocator {
    pub fn init(&mut self, start: usize, end: usize) -> Result<(), ()> {
        if start == 0 || end <= start { return Err(()); }
        self.next = start;
        self.limit = end;
        self.allocations = 0;
        Ok(())
    }

    pub fn allocate(&mut self, size: usize, alignment: usize) -> Option<usize> {
        if size == 0 || alignment == 0 || !alignment.is_power_of_two() || self.next > self.limit { return None; }
        let aligned = self.next.checked_add(alignment - 1)? & !(alignment - 1);
        let end = aligned.checked_add(size)?;
        if end > self.limit { return None; }
        self.next = end;
        self.allocations += 1;
        Some(aligned)
    }

    pub fn remaining(&self) -> usize { self.limit.saturating_sub(self.next) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocator_alignment_bounds_and_overflow() {
        let mut a = BootAllocator::default();
        assert!(a.init(0x1003, 0x1100).is_ok());
        assert_eq!(a.allocate(16, 16), Some(0x1010));
        assert_eq!(a.allocate(0x100, 1), None);
        assert_eq!(a.allocate(1, 3), None);
        assert_eq!(a.remaining(), 0xE0);
        assert_eq!(a.allocate(usize::MAX, 8), None);
    }

    #[test]
    fn map_add_sort_and_reserve_middle() {
        let mut m = MemoryMap::new();
        assert!(m.add(0x3000, 0x5000).is_ok());
        assert!(m.add(0x1000, 0x2000).is_ok());
        assert_eq!(m.regions[0], Region { start: 0x1000, end: 0x2000 });
        assert!(m.reserve(0x3800, 0x4000).is_ok());
        assert_eq!(m.regions[1], Region { start: 0x3000, end: 0x3800 });
        assert_eq!(m.regions[2], Region { start: 0x4000, end: 0x5000 });
    }

    #[test]
    fn failed_split_does_not_partially_mutate_full_map() {
        let mut m = MemoryMap::new();
        for i in 0..MAX_REGIONS {
            let start = 0x1000 + i * 0x2000;
            assert!(m.add(start, start + 0x1000).is_ok());
        }
        let before = m;
        assert!(m.reserve(0x1800, 0x1900).is_err());
        assert_eq!(m.count, before.count);
        assert_eq!(m.regions[0], before.regions[0]);
    }

    #[test]
    fn map_rejects_overlap_and_bad_ranges() {
        let mut m = MemoryMap::new();
        assert!(m.add(0x1000, 0x2000).is_ok());
        assert!(m.add(0x1800, 0x2800).is_err());
        assert!(m.add(0, 0x2000).is_err());
        assert_eq!(size_of::<Region>(), size_of::<usize>() * 2);
    }
}
