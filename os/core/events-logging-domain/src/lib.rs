//! # talos-events-logging
//!
//! The events and logging subsystem for the operating-system Talos migration.
//!
//! This crate models the observability machinery of `siderolabs/talos`:
//!
//! * [`events`] — the typed runtime [`Event`](events::Event) payloads
//!   (sequence transitions, service-state changes, config load/validation
//!   results, task progress, free-form messages).
//! * [`event_stream`] — the bounded, monotonically-id'd
//!   [`EventStream`](event_stream::EventStream) with tail/follow and gap
//!   detection, mirroring the machined runtime event ring.
//! * [`event_sink`] — the
//!   [`EventSinkController`](event_sink::EventSinkController) that forwards
//!   runtime events to a remote collector with a delivery cursor and
//!   exponential backoff, mirroring the machined `EventSinkConfig` path.
//! * [`circular_buffer`] — the byte-oriented
//!   [`CircularBuffer`](circular_buffer::CircularBuffer) used to retain recent
//!   service log output.
//! * [`kmsg`] — a `/dev/kmsg` record parser and reader powering the `Dmesg`
//!   API, with the kernel boundary modeled as the
//!   [`KmsgSource`](kmsg::KmsgSource) trait.
//! * [`log_sink`] — per-service log capture ([`LogRegistry`](log_sink::LogRegistry))
//!   plus the [`LogSink`](log_sink::LogSink) forwarding boundary.
//! * [`api`] — the role-gated `Events` / `Dmesg` / `Logs` API facade
//!   ([`EventsLoggingService`](api::EventsLoggingService)).
//!
//! The crate uses only the standard library plus the internal `talos-core`
//! crate; it pulls in no external dependencies so the workspace builds fully
//! offline.

// Pedantic lints that demand pervasive attribute/doc boilerplate without
// changing behavior or improving the idiom of this crate's APIs:
//   * `must_use_candidate` / `return_self_not_must_use` would sprinkle
//     `#[must_use]` across nearly every getter and builder method.
//   * `missing_errors_doc` / `missing_panics_doc` would require `# Errors` /
//     `# Panics` sections on the many small `Result`-returning helpers whose
//     failure modes are already evident from their (internal) signatures.
#![allow(
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]

pub mod api;
pub mod circular_buffer;
pub mod event_sink;
pub mod event_stream;
pub mod events;
pub mod kmsg;
pub mod log_sink;

pub use api::{EventsLoggingService, EventsRequest, LogsRequest};
pub use circular_buffer::CircularBuffer;
pub use event_sink::{
    Backoff, DeliveryOutcome, EventEndpoint, EventSinkConfig, EventSinkController, MemoryEndpoint,
};
pub use event_stream::{DEFAULT_CAPACITY, EventStream};
pub use events::{Event, EventKind, SequenceAction, ServiceAction};
pub use kmsg::{KmsgReader, KmsgRecord, KmsgSource, MemoryKmsg, Severity, write_kmsg};
pub use log_sink::{
    ByteWriter, DEFAULT_SERVICE_LOG_BYTES, FormattingSink, LogDestination, LogFormat, LogRegistry,
    LogScheme, LogSink, MemorySink, MemoryWriter, MultiSink,
};
