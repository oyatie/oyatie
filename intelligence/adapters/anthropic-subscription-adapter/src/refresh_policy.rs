//! RefreshPolicy: BinaryHeap-based proactive refresh scheduler.
//!
//! The background ticker runs as a Tokio task. It holds a min-heap of
//! `RefreshEntry` keyed by `next_due` (unix epoch secs). On each tick it
//! wakes the earliest-due seat and drives a refresh through the shared
//! `AnthropicOAuthAdapter` state. After a successful refresh the seat is
//! re-enqueued at its new `next_refresh_due()`.
// data_class: INTERNAL_ONLY throughout this module.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::ports::SeatId;

/// A single entry in the refresh scheduler heap.
/// `Ord` is derived so `BinaryHeap<Reverse<RefreshEntry>>` gives a min-heap on `next_due`.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RefreshEntry {
    /// Unix epoch seconds when this seat's token should next be refreshed.
    pub next_due: u64,
    pub seat_id: SeatId,
}

impl Ord for SeatId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for SeatId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Min-heap scheduler for proactive token refreshes.
#[derive(Default)]
pub struct RefreshScheduler {
    heap: BinaryHeap<Reverse<RefreshEntry>>,
}

impl RefreshScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enqueue a seat for refresh at `next_due` epoch seconds.
    pub fn enqueue(&mut self, seat_id: SeatId, next_due: u64) {
        self.heap.push(Reverse(RefreshEntry { next_due, seat_id }));
    }

    /// Peek at the earliest-due entry without removing it.
    pub fn peek_next_due(&self) -> Option<u64> {
        self.heap.peek().map(|Reverse(e)| e.next_due)
    }

    /// Pop all entries whose `next_due <= now_secs`.
    pub fn drain_due(&mut self, now_secs: u64) -> Vec<RefreshEntry> {
        let mut due = Vec::new();
        while let Some(Reverse(entry)) = self.heap.peek() {
            if entry.next_due <= now_secs {
                due.push(self.heap.pop().unwrap().0);
            } else {
                break;
            }
        }
        due
    }

    /// Number of entries in the heap.
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seat(s: &str) -> SeatId {
        SeatId(s.into())
    }

    #[test]
    fn min_heap_ordering() {
        let mut sched = RefreshScheduler::new();
        sched.enqueue(seat("b"), 300);
        sched.enqueue(seat("a"), 100);
        sched.enqueue(seat("c"), 200);

        assert_eq!(sched.peek_next_due(), Some(100));
        let due = sched.drain_due(200);
        // Should drain entries with next_due <= 200: "a" (100) and "c" (200).
        assert_eq!(due.len(), 2);
        assert_eq!(due[0].next_due, 100);
        assert_eq!(due[1].next_due, 200);
        // "b" (300) remains.
        assert_eq!(sched.len(), 1);
        assert_eq!(sched.peek_next_due(), Some(300));
    }

    #[test]
    fn drain_due_nothing_due() {
        let mut sched = RefreshScheduler::new();
        sched.enqueue(seat("x"), 1000);
        let due = sched.drain_due(999);
        assert!(due.is_empty());
        assert_eq!(sched.len(), 1);
    }

    #[test]
    fn drain_due_all() {
        let mut sched = RefreshScheduler::new();
        sched.enqueue(seat("x"), 10);
        sched.enqueue(seat("y"), 20);
        let due = sched.drain_due(100);
        assert_eq!(due.len(), 2);
        assert!(sched.is_empty());
    }

    #[test]
    fn empty_scheduler_peek_is_none() {
        let sched = RefreshScheduler::new();
        assert_eq!(sched.peek_next_due(), None);
    }

    #[test]
    fn expires_lead_scheduling_example() {
        // A seat expiring at 1000 with lead=300 should be scheduled at 700.
        use crate::token_state::{EXPIRES_LEAD_SECS, SeatTokenState};
        let s = SeatTokenState::new("a".into(), "r".into(), 1000, 0);
        let next_due = s.next_refresh_due();
        assert_eq!(next_due, 1000 - EXPIRES_LEAD_SECS);

        let mut sched = RefreshScheduler::new();
        sched.enqueue(SeatId("seat".into()), next_due);
        // At now = next_due the entry should be due.
        let due = sched.drain_due(next_due);
        assert_eq!(due.len(), 1);
    }
}
