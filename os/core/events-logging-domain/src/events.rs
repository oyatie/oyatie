//! Runtime machine events.
//!
//! Mirrors `internal/app/machined/pkg/runtime/event.go` and the
//! `machine.Event` protobuf in Talos: each event has a monotonically assigned
//! id, an actor (the service or controller that produced it), and a typed
//! payload describing what happened (a sequence transition, a service state
//! change, a config validation result, etc.).

use std::fmt::Write as _;

use os_kernel::error::{Error, Result};

/// The category of a machine event payload.
///
/// In Talos the event payload is a oneof of concrete proto messages. We model
/// the common ones the runtime emits onto the event stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKind {
    /// A boot/upgrade/reset/shutdown sequence changed phase.
    /// (`machine.SequenceEvent`)
    Sequence {
        /// Sequence name, e.g. `"boot"`, `"upgrade"`, `"reset"`.
        sequence: String,
        /// Action within the sequence: noop/start/stop.
        action: SequenceAction,
    },
    /// A supervised service changed lifecycle state.
    /// (`machine.ServiceStateEvent`)
    ServiceState {
        /// Service id, e.g. `"kubelet"`.
        service: String,
        /// New action/state of the service.
        action: ServiceAction,
        /// Optional human-readable message.
        message: String,
    },
    /// A config was (re)loaded or validated. (`machine.ConfigLoadErrorEvent` /
    /// `machine.ConfigValidationErrorEvent` collapsed into one with a flag.)
    Config {
        /// Whether load/validation succeeded.
        ok: bool,
        /// Diagnostic message (empty when `ok`).
        message: String,
    },
    /// An installation/upgrade task reported progress.
    /// (`machine.TaskEvent`)
    Task {
        /// Task name.
        task: String,
        /// Whether the task is starting (`true`) or stopping (`false`).
        starting: bool,
    },
    /// A free-form message (`machine.MessageEvent`).
    Message(String),
}

impl EventKind {
    /// Short stable discriminant string, used for filtering on the stream and
    /// for the `type` field of the API representation.
    pub fn type_str(&self) -> &'static str {
        match self {
            EventKind::Sequence { .. } => "SequenceEvent",
            EventKind::ServiceState { .. } => "ServiceStateEvent",
            EventKind::Config { .. } => "ConfigEvent",
            EventKind::Task { .. } => "TaskEvent",
            EventKind::Message(_) => "MessageEvent",
        }
    }

    /// Whether this event represents a failure condition.
    pub fn is_error(&self) -> bool {
        match self {
            EventKind::Config { ok, .. } => !ok,
            EventKind::ServiceState { action, .. } => *action == ServiceAction::Failed,
            _ => false,
        }
    }
}

/// Action carried by a [`EventKind::Sequence`] event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceAction {
    /// No state change, just an informational tick.
    Noop,
    /// The sequence started.
    Start,
    /// The sequence stopped.
    Stop,
}

impl SequenceAction {
    /// The canonical uppercase wire string.
    pub fn as_str(self) -> &'static str {
        match self {
            SequenceAction::Noop => "NOOP",
            SequenceAction::Start => "START",
            SequenceAction::Stop => "STOP",
        }
    }

    /// Parse from the Talos wire string.
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_uppercase().as_str() {
            "NOOP" => Ok(SequenceAction::Noop),
            "START" => Ok(SequenceAction::Start),
            "STOP" => Ok(SequenceAction::Stop),
            other => Err(Error::parse(format!("unknown sequence action '{other}'"))),
        }
    }
}

/// Lifecycle action carried by a [`EventKind::ServiceState`] event. Mirrors
/// `machine.ServiceStateEvent_Action`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceAction {
    /// Service object created/initialized.
    Initialized,
    /// Service is being prepared (pulling, writing config).
    Preparing,
    /// Service is waiting on a dependency/condition.
    Waiting,
    /// Service is running.
    Running,
    /// Service is being stopped.
    Stopping,
    /// Service finished cleanly.
    Finished,
    /// Service failed.
    Failed,
    /// Service was skipped.
    Skipped,
}

impl ServiceAction {
    /// The canonical uppercase wire string (mirrors
    /// `ServiceStateEvent_Action` proto enum names).
    pub fn as_str(self) -> &'static str {
        match self {
            ServiceAction::Initialized => "INITIALIZED",
            ServiceAction::Preparing => "PREPARING",
            ServiceAction::Waiting => "WAITING",
            ServiceAction::Running => "RUNNING",
            ServiceAction::Stopping => "STOPPING",
            ServiceAction::Finished => "FINISHED",
            ServiceAction::Failed => "FAILED",
            ServiceAction::Skipped => "SKIPPED",
        }
    }

    /// Whether the service is in a healthy/terminal-ok condition.
    pub fn is_up(self) -> bool {
        matches!(self, ServiceAction::Running)
    }

    /// Whether this is a terminal condition.
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            ServiceAction::Finished | ServiceAction::Failed | ServiceAction::Skipped
        )
    }

    /// Parse from the Talos wire string.
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_uppercase().as_str() {
            "INITIALIZED" => Ok(ServiceAction::Initialized),
            "PREPARING" => Ok(ServiceAction::Preparing),
            "WAITING" => Ok(ServiceAction::Waiting),
            "RUNNING" => Ok(ServiceAction::Running),
            "STOPPING" => Ok(ServiceAction::Stopping),
            "FINISHED" => Ok(ServiceAction::Finished),
            "FAILED" => Ok(ServiceAction::Failed),
            "SKIPPED" => Ok(ServiceAction::Skipped),
            other => Err(Error::parse(format!("unknown service action '{other}'"))),
        }
    }
}

/// A single runtime event as carried on the event stream.
///
/// Mirrors `runtime.EventInfo`: an id, the typed payload and the actor id that
/// produced the event. `id` is assigned by the [`crate::event_stream::EventStream`]
/// and is monotonically increasing; `0` means "unassigned".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// Monotonic id assigned by the stream (0 = unassigned).
    pub id: u64,
    /// The id of the producer (service/controller name).
    pub actor_id: String,
    /// The typed payload.
    pub kind: EventKind,
}

impl Event {
    /// Construct an unassigned event with the given actor and payload.
    pub fn new(actor_id: impl Into<String>, kind: EventKind) -> Self {
        Event {
            id: 0,
            actor_id: actor_id.into(),
            kind,
        }
    }

    /// Whether this event has been published (i.e. assigned a non-zero id).
    pub fn is_published(&self) -> bool {
        self.id != 0
    }

    /// Convenience constructor for a service-state event.
    pub fn service(
        actor: impl Into<String>,
        service: impl Into<String>,
        action: ServiceAction,
    ) -> Self {
        let service = service.into();
        Event::new(
            actor,
            EventKind::ServiceState {
                service,
                action,
                message: String::new(),
            },
        )
    }

    /// Whether the event represents an error condition.
    pub fn is_error(&self) -> bool {
        self.kind.is_error()
    }

    /// Render the event as a single-line JSON object, mirroring the shape the
    /// Talos API marshals (`id`, `actor_id`, `type`, plus the payload fields).
    /// Uses no external crate.
    pub fn to_json(&self) -> String {
        let mut body = format!(
            "{{\"id\":{},\"actor_id\":{},\"type\":{}",
            self.id,
            json_str(&self.actor_id),
            json_str(self.kind.type_str()),
        );
        match &self.kind {
            EventKind::Sequence { sequence, action } => {
                let _ = write!(
                    body,
                    ",\"sequence\":{},\"action\":{}",
                    json_str(sequence),
                    json_str(action.as_str()),
                );
            }
            EventKind::ServiceState {
                service,
                action,
                message,
            } => {
                let _ = write!(
                    body,
                    ",\"service\":{},\"action\":{},\"message\":{}",
                    json_str(service),
                    json_str(action.as_str()),
                    json_str(message),
                );
            }
            EventKind::Config { ok, message } => {
                let _ = write!(body, ",\"ok\":{},\"message\":{}", ok, json_str(message));
            }
            EventKind::Task { task, starting } => {
                let _ = write!(
                    body,
                    ",\"task\":{},\"starting\":{}",
                    json_str(task),
                    starting
                );
            }
            EventKind::Message(m) => {
                let _ = write!(body, ",\"message\":{}", json_str(m));
            }
        }
        body.push('}');
        body
    }
}

/// Minimal JSON string quoting/escaping (no external crate).
fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_strings_match_kind() {
        let e = Event::new(
            "sequencer",
            EventKind::Sequence {
                sequence: "boot".to_string(),
                action: SequenceAction::Start,
            },
        );
        assert_eq!(e.kind.type_str(), "SequenceEvent");
        assert!(!e.is_published());
        assert!(!e.is_error());
    }

    #[test]
    fn service_action_classification() {
        assert!(ServiceAction::Running.is_up());
        assert!(!ServiceAction::Failed.is_up());
        assert!(ServiceAction::Finished.is_terminal());
        assert!(ServiceAction::Failed.is_terminal());
        assert!(!ServiceAction::Running.is_terminal());
    }

    #[test]
    fn error_detection() {
        let bad = Event::new(
            "config",
            EventKind::Config {
                ok: false,
                message: "x".into(),
            },
        );
        assert!(bad.is_error());
        let good = Event::new(
            "config",
            EventKind::Config {
                ok: true,
                message: String::new(),
            },
        );
        assert!(!good.is_error());
        let failed = Event::service("svc", "etcd", ServiceAction::Failed);
        assert!(failed.is_error());
    }

    #[test]
    fn parse_actions() {
        assert_eq!(
            SequenceAction::parse("start").unwrap(),
            SequenceAction::Start
        );
        assert!(SequenceAction::parse("bogus").is_err());
        assert_eq!(
            ServiceAction::parse("RUNNING").unwrap(),
            ServiceAction::Running
        );
        assert!(ServiceAction::parse("nope").is_err());
    }

    #[test]
    fn action_as_str_roundtrips() {
        for a in [
            ServiceAction::Initialized,
            ServiceAction::Preparing,
            ServiceAction::Waiting,
            ServiceAction::Running,
            ServiceAction::Stopping,
            ServiceAction::Finished,
            ServiceAction::Failed,
            ServiceAction::Skipped,
        ] {
            assert_eq!(ServiceAction::parse(a.as_str()).unwrap(), a);
        }
        for a in [
            SequenceAction::Noop,
            SequenceAction::Start,
            SequenceAction::Stop,
        ] {
            assert_eq!(SequenceAction::parse(a.as_str()).unwrap(), a);
        }
    }

    #[test]
    fn json_service_state_event() {
        let mut e = Event::service("system", "etcd", ServiceAction::Running);
        e.id = 7;
        let j = e.to_json();
        assert_eq!(
            j,
            "{\"id\":7,\"actor_id\":\"system\",\"type\":\"ServiceStateEvent\",\"service\":\"etcd\",\"action\":\"RUNNING\",\"message\":\"\"}"
        );
    }

    #[test]
    fn json_sequence_and_config_and_task_and_message() {
        let seq = Event::new(
            "sequencer",
            EventKind::Sequence {
                sequence: "boot".into(),
                action: SequenceAction::Start,
            },
        );
        assert!(seq.to_json().contains("\"sequence\":\"boot\""));
        assert!(seq.to_json().contains("\"action\":\"START\""));

        let cfg = Event::new(
            "config",
            EventKind::Config {
                ok: false,
                message: "bad".into(),
            },
        );
        assert!(cfg.to_json().contains("\"ok\":false"));
        assert!(cfg.to_json().contains("\"message\":\"bad\""));

        let task = Event::new(
            "installer",
            EventKind::Task {
                task: "install".into(),
                starting: true,
            },
        );
        assert!(task.to_json().contains("\"task\":\"install\""));
        assert!(task.to_json().contains("\"starting\":true"));

        let msg = Event::new("kmsg", EventKind::Message("hi \"there\"".into()));
        assert!(msg.to_json().contains("\"message\":\"hi \\\"there\\\"\""));
    }
}
