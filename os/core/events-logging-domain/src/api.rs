//! The machined API surface for the events & logging subsystem.
//!
//! Mirrors the three relevant RPCs of the Talos `machine.MachineService`
//! (`api/machine/machine.proto`):
//!
//! * `Events`  — stream runtime events (tail + follow).
//! * `Dmesg`   — stream the kernel ring buffer.
//! * `Logs`    — stream a service's captured log output.
//!
//! Each call is gated by the caller's [`RoleSet`]: these are all read-only
//! observability endpoints, so they require *read* access (`os`, `admin`, or
//! `reader`). We bundle the [`EventStream`], a kmsg source and the
//! [`LogRegistry`] behind a single facade and model the request options
//! (`tail_events`, `tail_lines`, `id`/`actor` filters) the proto exposes.

use crate::event_stream::EventStream;
use crate::events::Event;
use crate::kmsg::{KmsgReader, KmsgRecord, KmsgSource};
use crate::log_sink::{LogRegistry, LogSink, MemorySink};
use os_kernel::error::{Error, Result};
use os_kernel::role::RoleSet;

/// Options for an `Events` request. Mirrors `machine.EventsRequest`.
#[derive(Debug, Clone, Default)]
pub struct EventsRequest {
    /// Return at most this many of the most recent events (0 = none retained,
    /// only follow). Negative "all" semantics are modeled by `tail_all`.
    pub tail_events: usize,
    /// Return all retained events regardless of `tail_events`.
    pub tail_all: bool,
    /// Only return events whose id is greater than this (follow cursor).
    pub tail_id: u64,
    /// If set, only return events produced by this actor.
    pub with_actor: Option<String>,
}

impl EventsRequest {
    /// Request the last `n` events.
    pub fn tail(n: usize) -> Self {
        EventsRequest {
            tail_events: n,
            ..Default::default()
        }
    }

    /// Request all retained events.
    pub fn all() -> Self {
        EventsRequest {
            tail_all: true,
            ..Default::default()
        }
    }

    /// Follow events strictly after `id`.
    pub fn since(id: u64) -> Self {
        EventsRequest {
            tail_id: id,
            ..Default::default()
        }
    }

    /// Restrict to a single actor.
    pub fn actor(mut self, actor: impl Into<String>) -> Self {
        self.with_actor = Some(actor.into());
        self
    }
}

/// Options for a `Logs` request. Mirrors `machine.LogsRequest`.
#[derive(Debug, Clone)]
pub struct LogsRequest {
    /// Service / log id, e.g. `"kubelet"`.
    pub service: String,
    /// Return at most this many trailing lines (0 = all retained).
    pub tail_lines: usize,
}

impl LogsRequest {
    /// All retained lines for a service.
    pub fn all(service: impl Into<String>) -> Self {
        LogsRequest {
            service: service.into(),
            tail_lines: 0,
        }
    }

    /// The last `n` lines for a service.
    pub fn tail(service: impl Into<String>, n: usize) -> Self {
        LogsRequest {
            service: service.into(),
            tail_lines: n,
        }
    }
}

/// The events & logging service facade backing the three RPCs.
pub struct EventsLoggingService<K: KmsgSource, S: LogSink = MemorySink> {
    events: EventStream,
    kmsg: K,
    logs: LogRegistry<S>,
}

impl<K: KmsgSource, S: LogSink> EventsLoggingService<K, S> {
    /// Assemble the service from its component subsystems.
    pub fn new(events: EventStream, kmsg: K, logs: LogRegistry<S>) -> Self {
        EventsLoggingService { events, kmsg, logs }
    }

    /// Mutable access to the underlying event stream (for the runtime to
    /// publish onto).
    pub fn events_mut(&mut self) -> &mut EventStream {
        &mut self.events
    }

    /// Mutable access to the log registry (for services to append output).
    pub fn logs_mut(&mut self) -> &mut LogRegistry<S> {
        &mut self.logs
    }

    /// Serve an `Events` request after checking read access.
    pub fn events(&self, roles: &RoleSet, req: &EventsRequest) -> Result<Vec<Event>> {
        authorize_read(roles)?;
        let mut out = if req.tail_id != 0 {
            self.events.since(req.tail_id)?
        } else if req.tail_all {
            self.events.tail(self.events.len())
        } else {
            self.events.tail(req.tail_events)
        };
        if let Some(actor) = &req.with_actor {
            out.retain(|e| &e.actor_id == actor);
        }
        Ok(out)
    }

    /// Serve a `Dmesg` request after checking read access: drains the kernel
    /// ring buffer into parsed records.
    pub fn dmesg(&mut self, roles: &RoleSet) -> Result<Vec<KmsgRecord>> {
        authorize_read(roles)?;
        let mut reader = KmsgReader::new(&mut self.kmsg);
        Ok(reader.drain())
    }

    /// Serve an `Events` request and render each event as a JSON line. Used by
    /// the streaming wire path that marshals events to JSON.
    pub fn events_json(&self, roles: &RoleSet, req: &EventsRequest) -> Result<Vec<String>> {
        Ok(self
            .events(roles, req)?
            .iter()
            .map(Event::to_json)
            .collect())
    }

    /// Serve a `Dmesg` request, returning only records at or above a minimum
    /// severity (e.g. errors-and-worse). The kernel ring is consumed.
    pub fn dmesg_filtered(
        &mut self,
        roles: &RoleSet,
        max: crate::kmsg::Severity,
    ) -> Result<Vec<KmsgRecord>> {
        authorize_read(roles)?;
        let mut reader = KmsgReader::new(&mut self.kmsg);
        Ok(reader.drain_at_least(max))
    }

    /// Serve a `Logs` request after checking read access.
    pub fn logs(&self, roles: &RoleSet, req: &LogsRequest) -> Result<Vec<String>> {
        authorize_read(roles)?;
        if req.tail_lines == 0 {
            self.logs.read(&req.service)
        } else {
            self.logs.tail(&req.service, req.tail_lines)
        }
    }
}

/// Reject callers without read access to the observability APIs.
fn authorize_read(roles: &RoleSet) -> Result<()> {
    if roles.can_read() {
        Ok(())
    } else {
        Err(Error::permission_denied(
            "events/logs API requires os, admin, or reader role",
        ))
    }
}

// Allow `KmsgReader::new(&mut self.kmsg)` to treat a mutable reference to a
// source as a source itself.
impl<T: KmsgSource + ?Sized> KmsgSource for &mut T {
    fn next_line(&mut self) -> Option<String> {
        (**self).next_line()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::ServiceAction;
    use crate::kmsg::MemoryKmsg;
    use os_kernel::role::Role;

    fn admin() -> RoleSet {
        RoleSet::from_roles([Role::Admin])
    }

    fn reader() -> RoleSet {
        RoleSet::from_roles([Role::Reader])
    }

    fn build() -> EventsLoggingService<MemoryKmsg, MemorySink> {
        let mut events = EventStream::with_capacity(16);
        events.publish(Event::service("system", "etcd", ServiceAction::Running));
        events.publish(Event::service("system", "kubelet", ServiceAction::Running));
        events.publish_message("config", "loaded");

        let kmsg = MemoryKmsg::from_lines(["6,1,100,-;booting", "3,2,200,-;disk error"]);

        let mut logs = LogRegistry::new().with_sink(MemorySink::new());
        logs.append_line("etcd", "etcd line 1").unwrap();
        logs.append_line("etcd", "etcd line 2").unwrap();

        EventsLoggingService::new(events, kmsg, logs)
    }

    #[test]
    fn events_tail_and_actor_filter() {
        let svc = build();
        let all = svc.events(&reader(), &EventsRequest::all()).unwrap();
        assert_eq!(all.len(), 3);

        let last2 = svc.events(&reader(), &EventsRequest::tail(2)).unwrap();
        assert_eq!(last2.len(), 2);
        assert_eq!(last2[0].id, 2);

        let by_actor = svc
            .events(&reader(), &EventsRequest::all().actor("config"))
            .unwrap();
        assert_eq!(by_actor.len(), 1);
        assert_eq!(by_actor[0].actor_id, "config");
    }

    #[test]
    fn events_since_cursor() {
        let svc = build();
        let after1 = svc.events(&admin(), &EventsRequest::since(1)).unwrap();
        assert_eq!(after1.len(), 2);
        assert_eq!(after1[0].id, 2);
    }

    #[test]
    fn dmesg_drains_records() {
        let mut svc = build();
        let recs = svc.dmesg(&admin()).unwrap();
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[1].message, "disk error");
        assert!(recs[1].severity().is_error());
        // Second drain is empty; the ring was consumed.
        assert!(svc.dmesg(&admin()).unwrap().is_empty());
    }

    #[test]
    fn logs_read_and_tail() {
        let svc = build();
        let all = svc.logs(&reader(), &LogsRequest::all("etcd")).unwrap();
        assert_eq!(all, ["etcd line 1", "etcd line 2"]);
        let last = svc.logs(&reader(), &LogsRequest::tail("etcd", 1)).unwrap();
        assert_eq!(last, ["etcd line 2"]);
    }

    #[test]
    fn unauthorized_is_rejected() {
        let mut svc = build();
        let none = RoleSet::new();
        assert_eq!(
            svc.events(&none, &EventsRequest::all()).unwrap_err().kind(),
            "permission_denied"
        );
        assert_eq!(
            svc.logs(&none, &LogsRequest::all("etcd"))
                .unwrap_err()
                .kind(),
            "permission_denied"
        );
        assert_eq!(svc.dmesg(&none).unwrap_err().kind(), "permission_denied");
    }

    #[test]
    fn impersonator_alone_cannot_read() {
        let svc = build();
        let imp = RoleSet::from_roles([Role::Impersonator]);
        assert!(svc.events(&imp, &EventsRequest::all()).is_err());
    }

    #[test]
    fn events_json_marshals_each_event() {
        let svc = build();
        let json = svc.events_json(&reader(), &EventsRequest::all()).unwrap();
        assert_eq!(json.len(), 3);
        assert!(json[0].contains("\"type\":\"ServiceStateEvent\""));
        assert!(json[0].contains("\"service\":\"etcd\""));
        assert!(json[2].contains("\"type\":\"MessageEvent\""));
        // json path is also access-gated.
        assert!(
            svc.events_json(&RoleSet::new(), &EventsRequest::all())
                .is_err()
        );
    }

    #[test]
    fn dmesg_filtered_by_severity() {
        let mut svc = build();
        // ring has info(6) "booting" and err(3) "disk error"; keep err-and-worse.
        let recs = svc
            .dmesg_filtered(&admin(), crate::kmsg::Severity::Error)
            .unwrap();
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].message, "disk error");
    }
}
