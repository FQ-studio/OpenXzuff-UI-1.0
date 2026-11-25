//! Single-producer/single-consumer byte queue for interrupt-to-task handoff.
//!
//! The intended RV64 use is one UART interrupt producer and one task consumer
//! on a single hart. This is not an SMP queue and must not have multiple
//! producers or consumers without an external lock.
use core::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ByteQueueError {
    Full,
    ZeroCapacity,
}

pub struct SpscByteQueue<const N: usize> {
    slots: [AtomicUsize; N],
    head: AtomicUsize,
    tail: AtomicUsize,
    dropped: AtomicUsize,
}

impl<const N: usize> SpscByteQueue<N> {
    pub const fn new() -> Self {
        Self {
            slots: [const { AtomicUsize::new(0) }; N],
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            dropped: AtomicUsize::new(0),
        }
    }

    /// Publish one byte from the interrupt producer.
    pub fn push(&self, byte: u8) -> Result<(), ByteQueueError> {
        if N == 0 {
            return Err(ByteQueueError::ZeroCapacity);
        }
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        if head.wrapping_sub(tail) >= N {
            self.dropped.fetch_add(1, Ordering::Relaxed);
            return Err(ByteQueueError::Full);
        }
        self.slots[head % N].store(byte as usize, Ordering::Relaxed);
        self.head.store(head.wrapping_add(1), Ordering::Release);
        Ok(())
    }

    /// Consume one byte from the task context.
    pub fn pop(&self) -> Option<u8> {
        if N == 0 {
            return None;
        }
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        if tail == head {
            return None;
        }
        let byte = self.slots[tail % N].load(Ordering::Relaxed) as u8;
        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        Some(byte)
    }

    pub fn len(&self) -> usize {
        self.head.load(Ordering::Acquire)
            .wrapping_sub(self.tail.load(Ordering::Acquire))
            .min(N)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn dropped(&self) -> usize {
        self.dropped.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::{ByteQueueError, SpscByteQueue};

    #[test]
    fn queue_preserves_fifo_order_and_exact_capacity() {
        let queue = SpscByteQueue::<3>::new();
        assert!(queue.is_empty());
        queue.push(0x5a).unwrap();
        queue.push(0x6b).unwrap();
        queue.push(0x7c).unwrap();
        assert_eq!(queue.len(), 3);
        assert_eq!(queue.push(0x8d), Err(ByteQueueError::Full));
        assert_eq!(queue.dropped(), 1);
        assert_eq!(queue.pop(), Some(0x5a));
        assert_eq!(queue.pop(), Some(0x6b));
        assert_eq!(queue.pop(), Some(0x7c));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn queue_wraps_indices_without_reordering_bytes() {
        let queue = SpscByteQueue::<2>::new();
        queue.push(1).unwrap();
        queue.push(2).unwrap();
        assert_eq!(queue.pop(), Some(1));
        queue.push(3).unwrap();
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), Some(3));
        assert!(queue.is_empty());
    }

    #[test]
    fn zero_capacity_queue_rejects_push_without_indexing() {
        let queue = SpscByteQueue::<0>::new();
        assert_eq!(queue.push(1), Err(ByteQueueError::ZeroCapacity));
        assert_eq!(queue.pop(), None);
        assert_eq!(queue.dropped(), 0);
    }
}
