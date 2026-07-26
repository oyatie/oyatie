//! Watch/event stream primitives. Mirrors COSI's `state.Event` /
//! `state.EventType` and the bootstrap protocol used when a controller first
//! subscribes to a kind.

use crate::reduced::ReducedResource;
use crate::resource::AnyResource;
use core::fmt;
use std::collections::VecDeque;

/// The kind of change a watch event describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// A resource appeared in the store.
    Created,
    /// An existing resource changed (spec, labels, finalizers, or phase).
    Updated,
    /// A resource was removed from the store.
    Destroyed,
    /// The initial snapshot for a watch has been fully delivered; subsequent
    /// events are live. Mirrors COSI's `Bootstrapped` sentinel.
    Bootstrapped,
}

impl EventKind {
    /// Stable lowercase name.
    pub fn as_str(&self) -> &'static str {
        match self {
            EventKind::Created => "created",
            EventKind::Updated => "updated",
            EventKind::Destroyed => "destroyed",
            EventKind::Bootstrapped => "bootstrapped",
        }
    }

    /// Whether this event carries a resource payload (all except
    /// [`EventKind::Bootstrapped`]).
    pub fn has_payload(&self) -> bool {
        !matches!(self, EventKind::Bootstrapped)
    }
}

impl fmt::Display for EventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A single watch event.
///
/// For [`EventKind::Created`]/[`EventKind::Updated`] `resource` holds the new
/// state. For [`EventKind::Updated`] `old` may hold the previous state. For
/// [`EventKind::Destroyed`] `resource` holds the last-known state. For
/// [`EventKind::Bootstrapped`] both are `None`.
#[derive(Debug, Clone)]
pub struct Event {
    kind: EventKind,
    resource: Option<AnyResource>,
    old: Option<AnyResource>,
}

impl Event {
    /// A created event.
    pub fn created(resource: AnyResource) -> Self {
        Event {
            kind: EventKind::Created,
            resource: Some(resource),
            old: None,
        }
    }

    /// An updated event with optional previous state.
    pub fn updated(resource: AnyResource, old: Option<AnyResource>) -> Self {
        Event {
            kind: EventKind::Updated,
            resource: Some(resource),
            old,
        }
    }

    /// A destroyed event carrying the last-known state.
    pub fn destroyed(resource: AnyResource) -> Self {
        Event {
            kind: EventKind::Destroyed,
            resource: Some(resource),
            old: None,
        }
    }

    /// The bootstrap sentinel.
    pub fn bootstrapped() -> Self {
        Event {
            kind: EventKind::Bootstrapped,
            resource: None,
            old: None,
        }
    }

    /// The event kind.
    pub fn kind(&self) -> EventKind {
        self.kind
    }

    /// The resource payload, if any.
    pub fn resource(&self) -> Option<&AnyResource> {
        self.resource.as_ref()
    }

    /// The previous state, if any (only for updates).
    pub fn old(&self) -> Option<&AnyResource> {
        self.old.as_ref()
    }

    /// A reduced view of the payload, if present.
    pub fn reduced(&self) -> Option<ReducedResource> {
        self.resource
            .as_ref()
            .map(|r| ReducedResource::from_metadata(r.metadata()))
    }
}

/// An in-memory, bounded watch channel.
///
/// In real COSI watch streams are backed by Go channels; here we model the
/// boundary as a simple queue that controllers drain. When the buffer overflows
/// the watch is marked errored (mirroring COSI's "buffer overrun" failure).
#[derive(Debug)]
pub struct WatchChannel {
    buffer: VecDeque<Event>,
    capacity: usize,
    overran: bool,
    bootstrapped: bool,
}

impl WatchChannel {
    /// Create a watch channel with the given buffer capacity.
    pub fn new(capacity: usize) -> Self {
        WatchChannel {
            buffer: VecDeque::new(),
            capacity: capacity.max(1),
            overran: false,
            bootstrapped: false,
        }
    }

    /// Push an event. If pushing would exceed capacity the channel is marked
    /// as overrun and the event is dropped. Returns `false` on overrun.
    pub fn push(&mut self, event: Event) -> bool {
        if self.overran {
            return false;
        }
        if event.kind() == EventKind::Bootstrapped {
            self.bootstrapped = true;
        }
        if self.buffer.len() >= self.capacity {
            self.overran = true;
            return false;
        }
        self.buffer.push_back(event);
        true
    }

    /// Pop the next event in FIFO order.
    pub fn pop(&mut self) -> Option<Event> {
        self.buffer.pop_front()
    }

    /// Drain all currently buffered events.
    pub fn drain(&mut self) -> Vec<Event> {
        self.buffer.drain(..).collect()
    }

    /// Whether the channel has overrun its buffer and is no longer usable.
    pub fn is_overran(&self) -> bool {
        self.overran
    }

    /// Whether the bootstrap sentinel has been observed.
    pub fn is_bootstrapped(&self) -> bool {
        self.bootstrapped
    }

    /// Number of buffered events.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::Metadata;
    use crate::resource::Resource;
    use crate::resource::ResourceKind;
    use os_kernel::ResourceId;

    #[derive(Debug, Clone)]
    struct R {
        meta: Metadata,
        v: u32,
    }
    impl R {
        fn new(id: &str, v: u32) -> Self {
            R {
                meta: Metadata::new("default", "R", ResourceId::new(id).unwrap()),
                v,
            }
        }
    }
    impl Resource for R {
        fn metadata(&self) -> &Metadata {
            &self.meta
        }
        fn metadata_mut(&mut self) -> &mut Metadata {
            &mut self.meta
        }
        fn spec_fingerprint(&self) -> String {
            format!("v={}", self.v)
        }
        fn clone_box(&self) -> Box<dyn Resource> {
            Box::new(self.clone())
        }
    }

    #[test]
    fn event_kinds_and_payload() {
        assert!(EventKind::Created.has_payload());
        assert!(!EventKind::Bootstrapped.has_payload());
        let e = Event::created(Box::new(R::new("a", 1)));
        assert_eq!(e.kind(), EventKind::Created);
        assert_eq!(e.reduced().unwrap().key(), "default/R/a");
        let _ = ResourceKind::new("default", "R");
    }

    #[test]
    fn channel_fifo_and_bootstrap() {
        let mut ch = WatchChannel::new(8);
        ch.push(Event::bootstrapped());
        ch.push(Event::created(Box::new(R::new("a", 1))));
        assert!(ch.is_bootstrapped());
        assert_eq!(ch.pop().unwrap().kind(), EventKind::Bootstrapped);
        assert_eq!(ch.pop().unwrap().kind(), EventKind::Created);
        assert!(ch.is_empty());
    }

    #[test]
    fn channel_overrun_marks_errored() {
        let mut ch = WatchChannel::new(2);
        assert!(ch.push(Event::created(Box::new(R::new("a", 1)))));
        assert!(ch.push(Event::created(Box::new(R::new("b", 1)))));
        assert!(!ch.push(Event::created(Box::new(R::new("c", 1)))));
        assert!(ch.is_overran());
        assert!(!ch.push(Event::bootstrapped()));
    }
}
