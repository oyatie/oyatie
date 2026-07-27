//! The runtime event stream: a bounded ring of published events with id-based
//! tail/follow semantics.
//!
//! Mirrors `internal/app/machined/pkg/runtime/v1alpha1/event/event.go`. The
//! Talos stream is a fixed-capacity circular buffer. Each `Publish` assigns the
//! next monotonic id and overwrites the oldest entry when full. Subscribers can
//! ask for the last `N` events (tail) and then follow new ones; consumers that
//! fall too far behind observe a gap.

use crate::events::{Event, EventKind};
use os_kernel::error::{Error, Result};

/// Default capacity of the runtime event stream (Talos uses 1000).
pub const DEFAULT_CAPACITY: usize = 1000;

/// A bounded, monotonically-id'd stream of [`Event`]s.
#[derive(Debug, Clone)]
pub struct EventStream {
    /// Backing ring; `buf[i]` holds an event once written.
    buf: Vec<Event>,
    /// Fixed capacity (> 0).
    capacity: usize,
    /// Id that will be assigned to the next published event (starts at 1).
    next_id: u64,
}

impl EventStream {
    /// Create a stream with the given capacity. Capacity is clamped to at least
    /// 1.
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        EventStream {
            buf: Vec::with_capacity(capacity),
            capacity,
            next_id: 1,
        }
    }

    /// Create a stream with [`DEFAULT_CAPACITY`].
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    /// Capacity of the stream.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Number of events currently retained.
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Whether no events are retained.
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// The id of the most recently published event, or 0 if none.
    pub fn last_id(&self) -> u64 {
        self.next_id.saturating_sub(1)
    }

    /// The id of the oldest retained event, or 0 if empty.
    pub fn oldest_id(&self) -> u64 {
        self.buf.first().map_or(0, |e| e.id)
    }

    /// Publish an event, assigning it the next monotonic id and returning that
    /// id. When the ring is full the oldest event is evicted.
    pub fn publish(&mut self, mut event: Event) -> u64 {
        let id = self.next_id;
        event.id = id;
        self.next_id += 1;
        if self.buf.len() == self.capacity {
            self.buf.remove(0);
        }
        self.buf.push(event);
        id
    }

    /// Return up to `n` of the most recently published events, oldest-first.
    /// `n == 0` returns an empty slice clone.
    pub fn tail(&self, n: usize) -> Vec<Event> {
        if n == 0 {
            return Vec::new();
        }
        let start = self.buf.len().saturating_sub(n);
        self.buf[start..].to_vec()
    }

    /// Return all retained events with id strictly greater than `after`,
    /// oldest-first. Used by following subscribers.
    ///
    /// Returns an error if `after` is older than the oldest retained id and not
    /// zero — i.e. the consumer fell behind and there is an unrecoverable gap.
    pub fn since(&self, after: u64) -> Result<Vec<Event>> {
        if after != 0 && !self.buf.is_empty() && after < self.oldest_id().saturating_sub(1) {
            return Err(Error::invalid_state("subscriber fell behind: event gap"));
        }
        Ok(self.buf.iter().filter(|e| e.id > after).cloned().collect())
    }

    /// Return retained events of a given [`EventKind`] discriminant
    /// (`type_str`), oldest-first.
    pub fn filter_type(&self, type_str: &str) -> Vec<Event> {
        self.buf
            .iter()
            .filter(|e| e.kind.type_str() == type_str)
            .cloned()
            .collect()
    }

    /// Count retained events that represent errors.
    pub fn error_count(&self) -> usize {
        self.buf.iter().filter(|e| e.is_error()).count()
    }

    /// Look up a retained event by id.
    pub fn get(&self, id: u64) -> Option<&Event> {
        self.buf.iter().find(|e| e.id == id)
    }

    /// Convenience: publish a free-form message event.
    pub fn publish_message(&mut self, actor: &str, msg: &str) -> u64 {
        self.publish(Event::new(actor, EventKind::Message(msg.into())))
    }
}

impl Default for EventStream {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::ServiceAction;

    fn svc(name: &str, action: ServiceAction) -> Event {
        Event::service("system", name, action)
    }

    #[test]
    fn publish_assigns_monotonic_ids() {
        let mut s = EventStream::with_capacity(8);
        let a = s.publish(svc("etcd", ServiceAction::Running));
        let b = s.publish(svc("kubelet", ServiceAction::Running));
        assert_eq!((a, b), (1, 2));
        assert_eq!(s.last_id(), 2);
        assert_eq!(s.len(), 2);
        assert!(s.get(1).unwrap().is_published());
    }

    #[test]
    fn ring_evicts_oldest_when_full() {
        let mut s = EventStream::with_capacity(3);
        for i in 0..5 {
            s.publish(svc("svc", ServiceAction::Running));
            let _ = i;
        }
        assert_eq!(s.len(), 3);
        assert_eq!(s.oldest_id(), 3);
        assert_eq!(s.last_id(), 5);
        assert!(s.get(1).is_none());
        assert!(s.get(5).is_some());
    }

    #[test]
    fn tail_returns_newest_n_oldest_first() {
        let mut s = EventStream::with_capacity(10);
        for _ in 0..4 {
            s.publish(svc("svc", ServiceAction::Running));
        }
        let t = s.tail(2);
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].id, 3);
        assert_eq!(t[1].id, 4);
        assert!(s.tail(0).is_empty());
    }

    #[test]
    fn since_detects_gap() {
        let mut s = EventStream::with_capacity(2);
        for _ in 0..4 {
            s.publish(svc("svc", ServiceAction::Running));
        }
        // retained ids: 3,4. asking after=1 is a gap.
        assert!(s.since(1).is_err());
        // after=3 returns just id 4.
        let got = s.since(3).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].id, 4);
        // after=0 returns everything retained.
        assert_eq!(s.since(0).unwrap().len(), 2);
    }

    #[test]
    fn filter_and_error_count() {
        let mut s = EventStream::new();
        s.publish(svc("etcd", ServiceAction::Running));
        s.publish(svc("etcd", ServiceAction::Failed));
        s.publish_message("test", "hello");
        assert_eq!(s.filter_type("ServiceStateEvent").len(), 2);
        assert_eq!(s.filter_type("MessageEvent").len(), 1);
        assert_eq!(s.error_count(), 1);
    }
}
