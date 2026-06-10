//! The machined event stream, mirroring Talos
//! `internal/app/machined/pkg/runtime` events (`Watcher`/`EventStream`).
//!
//! As machined sequences the machine and supervises services it publishes
//! structured events: sequence start/finish, phase boundaries, task progress,
//! and service state transitions. The apid surfaces these over the
//! `EventsRequest` streaming RPC and they back `talosctl dmesg`/`events`.
//!
//! Here the stream is an in-memory ring buffer with monotonic sequence numbers,
//! exactly the shape the real `runtime.EventStream` exposes (a bounded circular
//! buffer plus tailing watchers). Watchers subscribe with an optional
//! tail/offset and drain new events.

use crate::sequence::Sequence;
use crate::service::ServiceState;
use crate::state_machine::MachineState;

/// The kind of thing an [`Event`] reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKind {
    /// A sequence began.
    SequenceStart {
        /// The sequence that started.
        sequence: Sequence,
    },
    /// A sequence finished (successfully or because a reboot was requested).
    SequenceFinish {
        /// The sequence that finished.
        sequence: Sequence,
        /// Whether it ended early due to a requested reboot.
        rebooted: bool,
    },
    /// A phase within a sequence began.
    PhaseStart {
        /// The phase name.
        phase: String,
    },
    /// A phase within a sequence finished.
    PhaseFinish {
        /// The phase name.
        phase: String,
    },
    /// A task reported progress (informational).
    TaskProgress {
        /// The task name.
        task: String,
        /// A short human-readable message.
        message: String,
    },
    /// A service changed lifecycle state.
    ServiceStateChange {
        /// The service id.
        service: String,
        /// The state it moved into.
        state: ServiceState,
    },
    /// The coarse machine lifecycle state changed.
    MachineStateChange {
        /// The new machine state.
        state: MachineState,
    },
    /// A free-form informational message (mirrors `runtime.MessageEvent`).
    Message {
        /// The message body.
        body: String,
    },
}

impl EventKind {
    /// A stable lowercase type label for the event kind.
    pub fn type_label(&self) -> &'static str {
        match self {
            EventKind::SequenceStart { .. } => "sequence.start",
            EventKind::SequenceFinish { .. } => "sequence.finish",
            EventKind::PhaseStart { .. } => "phase.start",
            EventKind::PhaseFinish { .. } => "phase.finish",
            EventKind::TaskProgress { .. } => "task.progress",
            EventKind::ServiceStateChange { .. } => "service.state",
            EventKind::MachineStateChange { .. } => "machine.state",
            EventKind::Message { .. } => "message",
        }
    }
}

/// A single published event with a monotonic id.
///
/// Mirrors `runtime.Event`: an opaque, ordered record with a unique id (the
/// real implementation uses a ULID; here a `u64` is sufficient and ordered).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// Monotonic sequence id (1-based; never reused within a stream).
    pub id: u64,
    /// The payload.
    pub kind: EventKind,
}

/// A bounded, in-memory event stream with monotonic ids and tailing watchers.
///
/// Mirrors `runtime.EventStream`: a fixed-capacity circular buffer. When the
/// buffer is full the oldest event is dropped, but ids keep increasing so a
/// watcher can detect a gap (it fell behind).
#[derive(Debug)]
pub struct EventStream {
    capacity: usize,
    buffer: std::collections::VecDeque<Event>,
    next_id: u64,
}

impl EventStream {
    /// Create a stream holding at most `capacity` events (must be > 0).
    pub fn new(capacity: usize) -> Self {
        EventStream {
            capacity: capacity.max(1),
            buffer: std::collections::VecDeque::new(),
            next_id: 1,
        }
    }

    /// Publish an event, returning its assigned id. Drops the oldest event if
    /// the buffer is at capacity.
    pub fn publish(&mut self, kind: EventKind) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        if self.buffer.len() == self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(Event { id, kind });
        id
    }

    /// The id that will be assigned to the next published event.
    pub fn next_id(&self) -> u64 {
        self.next_id
    }

    /// Number of events currently buffered.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// The maximum number of events retained.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// All currently-buffered events in publication order.
    pub fn events(&self) -> impl Iterator<Item = &Event> {
        self.buffer.iter()
    }

    /// The most recent `n` events (the tail), oldest-first.
    ///
    /// Mirrors the `tail_events` field of the Talos `EventsRequest`.
    pub fn tail(&self, n: usize) -> Vec<&Event> {
        let len = self.buffer.len();
        let start = len.saturating_sub(n);
        self.buffer.iter().skip(start).collect()
    }

    /// Every event with an id strictly greater than `after`. Used by a watcher
    /// resuming from the last id it saw.
    pub fn since(&self, after: u64) -> Vec<&Event> {
        self.buffer.iter().filter(|e| e.id > after).collect()
    }

    /// All events whose payload matches `type_label`.
    pub fn of_type(&self, type_label: &str) -> Vec<&Event> {
        self.buffer
            .iter()
            .filter(|e| e.kind.type_label() == type_label)
            .collect()
    }

    /// Whether the watcher resuming `after` has missed events (fell out of the
    /// retained window). True if `after` is older than the oldest buffered id.
    pub fn has_gap(&self, after: u64) -> bool {
        match self.buffer.front() {
            // A watcher at id N expects to next see N+1. If the oldest buffered
            // id is greater than N+1, events between were dropped.
            Some(front) => after + 1 < front.id,
            None => false,
        }
    }
}

impl Default for EventStream {
    fn default() -> Self {
        // 1000 mirrors the default Talos event stream capacity.
        EventStream::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publishes_with_monotonic_ids() {
        let mut s = EventStream::new(10);
        let a = s.publish(EventKind::Message { body: "hi".into() });
        let b = s.publish(EventKind::Message {
            body: "there".into(),
        });
        assert_eq!(a, 1);
        assert_eq!(b, 2);
        assert_eq!(s.len(), 2);
        assert_eq!(s.next_id(), 3);
    }

    #[test]
    fn ring_buffer_drops_oldest_but_keeps_ids() {
        let mut s = EventStream::new(2);
        s.publish(EventKind::Message { body: "1".into() });
        s.publish(EventKind::Message { body: "2".into() });
        s.publish(EventKind::Message { body: "3".into() });
        assert_eq!(s.len(), 2);
        let ids: Vec<u64> = s.events().map(|e| e.id).collect();
        assert_eq!(ids, vec![2, 3]);
    }

    #[test]
    fn tail_returns_last_n() {
        let mut s = EventStream::new(10);
        for i in 0..5 {
            s.publish(EventKind::Message {
                body: format!("{i}"),
            });
        }
        let t = s.tail(2);
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].id, 4);
        assert_eq!(t[1].id, 5);
        // Asking for more than exist returns all.
        assert_eq!(s.tail(100).len(), 5);
    }

    #[test]
    fn since_resumes_after_id() {
        let mut s = EventStream::new(10);
        for _ in 0..4 {
            s.publish(EventKind::Message { body: "x".into() });
        }
        let after = s.since(2);
        assert_eq!(after.iter().map(|e| e.id).collect::<Vec<_>>(), vec![3, 4]);
        assert!(s.since(4).is_empty());
    }

    #[test]
    fn of_type_filters() {
        let mut s = EventStream::new(10);
        s.publish(EventKind::SequenceStart {
            sequence: Sequence::Boot,
        });
        s.publish(EventKind::Message { body: "x".into() });
        s.publish(EventKind::SequenceFinish {
            sequence: Sequence::Boot,
            rebooted: false,
        });
        assert_eq!(s.of_type("sequence.start").len(), 1);
        assert_eq!(s.of_type("message").len(), 1);
    }

    #[test]
    fn detects_gap_when_watcher_fell_behind() {
        let mut s = EventStream::new(2);
        for _ in 0..5 {
            s.publish(EventKind::Message { body: "x".into() });
        }
        // Oldest buffered id is 4; a watcher last at id 1 expects id 2 next.
        assert!(s.has_gap(1));
        // A watcher at id 3 expects id 4, which is present: no gap.
        assert!(!s.has_gap(3));
    }

    #[test]
    fn type_labels_are_stable() {
        assert_eq!(
            EventKind::ServiceStateChange {
                service: "etcd".into(),
                state: ServiceState::Healthy,
            }
            .type_label(),
            "service.state"
        );
        assert_eq!(
            EventKind::MachineStateChange {
                state: MachineState::Running
            }
            .type_label(),
            "machine.state"
        );
    }
}
