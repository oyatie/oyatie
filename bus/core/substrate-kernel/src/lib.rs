//! # messaging-substrate-kernel
//!
//! The ONE owned messaging substrate interface (ADR-0536 D-13; G009 lane).
//!
//! ## Posture
//! ADR-0536 D-13 rules messaging as a queue/stream/bus trichotomy of
//! single-concern surfaces (ADR-0132) over ONE substrate, consumed only
//! through a thin owned Rust interface; Apache Pulsar is the validated
//! transitional implementation behind this port (ADR-0510), never the
//! terminal shape. This crate is that port: pure types + sync traits,
//! dependency-free at kernel scope (ADR-0083 kernel-tier invariant),
//! shaped by the OWNED destination substrate — segmented storage with
//! subscription fan-out, competing consumers within a subscription, and
//! seekable cursors (precedent: Apache Pulsar subscription model; AWS SQS
//! at-least-once doctrine; Google Pub/Sub seek/replay; Meta FOQS
//! disaggregated queues). W5 cutover review: every trait here models
//! destination semantics, not broker wire details — an owned broker swap
//! changes adapters only.
//!
//! ## Delivery contract (normative)
//! - **At-least-once transport.** There is deliberately NO exactly-once
//!   API surface; effectively-once is composed at the producer edge via
//!   the transactional outbox (`shared-transactional-outbox-kernel`)
//!   and consumer-side idempotency (ADR-0536 D-13 rejected: global
//!   "exactly-once" delivery promises).
//! - **Per-key ordering only.** Messages sharing a [`MessageKey`] are
//!   delivered in publish order within a subscription; no cross-key or
//!   cross-topic ordering is promised.
//! - **Two loss classes** (ADR-0537 step 8): [`LossClass::NeverLose`]
//!   (metering/audit grade — durable, replayable) and
//!   [`LossClass::Expendable`] (telemetry grade — load-sheddable).
//!
//! ## Surface mapping
//! The three single-concern boundary kernels compose this port:
//! queue = one competing subscription; bus = fan-out across independent
//! subscriptions; stream = a seekable cursor subscription.
//!
//! # Naming justification
//! `messaging-substrate-kernel` = `<capability:messaging>-<leaf:substrate-kernel>`,
//! the de-branded name for path `messaging/core/substrate-kernel` (ADR-0562
//! capability-first; ADR-0532/0533 de-brand drops the vendor prefix), mirroring
//! the sibling leaf `messaging/core/domain` (`messaging-domain`) — owned
//! substrate interface crates carry the owned-surface name, not a vendor name.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;
use std::num::NonZeroU32;
use std::time::Duration;

pub mod conformance;
pub mod reference;

/// Maximum accepted topic-name length.
pub const MAX_TOPIC_NAME_LEN: usize = 255;
/// Maximum accepted subscription-name length.
pub const MAX_SUBSCRIPTION_NAME_LEN: usize = 255;
/// Maximum accepted message-key length in bytes.
pub const MAX_MESSAGE_KEY_LEN: usize = 1024;
/// Maximum accepted payload size in bytes (aligned with the 1 MiB ceiling
/// shared by SQS/Pub/Sub-class substrates; large payloads belong in the
/// object store with a claim-check envelope).
pub const MAX_PAYLOAD_BYTES: usize = 1024 * 1024;

// =====================================================================
// Errors
// =====================================================================

/// Substrate contract errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessagingError {
    /// Topic name violates the canonical slug shape.
    InvalidTopicName { value: String },
    /// Subscription name violates the canonical slug shape.
    InvalidSubscriptionName { value: String },
    /// Message key is empty or oversized.
    InvalidMessageKey,
    /// Payload exceeds [`MAX_PAYLOAD_BYTES`].
    PayloadTooLarge { size: usize },
    /// The topic has not been ensured.
    TopicNotFound { topic: String },
    /// The subscription has not been ensured on the topic.
    SubscriptionNotFound { topic: String, subscription: String },
    /// `ensure_topic` was replayed with a conflicting spec; topic specs are
    /// immutable once created (a spec change is a new topic, mirroring the
    /// versioned-immutable doctrine of ADR-0536 D-14).
    TopicSpecMismatch { topic: String },
    /// The ack token is unknown or already settled.
    UnknownAckToken,
    /// The seek position is beyond the topic head.
    InvalidSeekPosition { position: u64 },
    /// Transient backend failure; the caller retries (at-least-once makes
    /// retry safe by contract).
    BackendUnavailable { detail: String },
}

impl fmt::Display for MessagingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTopicName { value } => write!(f, "invalid topic name: {value:?}"),
            Self::InvalidSubscriptionName { value } => {
                write!(f, "invalid subscription name: {value:?}")
            }
            Self::InvalidMessageKey => write!(f, "invalid message key"),
            Self::PayloadTooLarge { size } => {
                write!(f, "payload of {size} bytes exceeds {MAX_PAYLOAD_BYTES}")
            }
            Self::TopicNotFound { topic } => write!(f, "topic not found: {topic}"),
            Self::SubscriptionNotFound {
                topic,
                subscription,
            } => write!(f, "subscription {subscription} not found on topic {topic}"),
            Self::TopicSpecMismatch { topic } => {
                write!(f, "topic {topic} re-ensured with a conflicting spec")
            }
            Self::UnknownAckToken => write!(f, "unknown or settled ack token"),
            Self::InvalidSeekPosition { position } => {
                write!(f, "seek position {position} is beyond the topic head")
            }
            Self::BackendUnavailable { detail } => write!(f, "backend unavailable: {detail}"),
        }
    }
}

impl std::error::Error for MessagingError {}

// =====================================================================
// Names and specs
// =====================================================================

fn valid_slug(value: &str, max_len: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_len
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.')
        && !value.starts_with(['-', '.'])
        && !value.ends_with(['-', '.'])
}

/// Canonical topic name: dot-namespaced lowercase slug
/// (e.g. `metering.usage-events`).
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct TopicName(String);

impl TopicName {
    /// Parses and validates a topic name.
    ///
    /// # Errors
    /// Returns [`MessagingError::InvalidTopicName`] when the value is not a
    /// canonical slug.
    pub fn parse(value: &str) -> Result<Self, MessagingError> {
        if valid_slug(value, MAX_TOPIC_NAME_LEN) {
            Ok(Self(value.to_owned()))
        } else {
            Err(MessagingError::InvalidTopicName {
                value: value.to_owned(),
            })
        }
    }

    /// The canonical string form.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TopicName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Canonical subscription name (same slug shape as topics).
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct SubscriptionName(String);

impl SubscriptionName {
    /// Parses and validates a subscription name.
    ///
    /// # Errors
    /// Returns [`MessagingError::InvalidSubscriptionName`] when the value is
    /// not a canonical slug.
    pub fn parse(value: &str) -> Result<Self, MessagingError> {
        if valid_slug(value, MAX_SUBSCRIPTION_NAME_LEN) {
            Ok(Self(value.to_owned()))
        } else {
            Err(MessagingError::InvalidSubscriptionName {
                value: value.to_owned(),
            })
        }
    }

    /// The canonical string form.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SubscriptionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The two loss classes of ADR-0537 step 8.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum LossClass {
    /// Durable, replayable, never load-shed: metering, audit, outbox relay.
    NeverLose,
    /// Load-sheddable under pressure: telemetry, presence, hints.
    Expendable,
}

/// Immutable topic specification, fixed at `ensure_topic` time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopicSpec {
    /// Which loss class the topic carries.
    pub loss_class: LossClass,
    /// Retention horizon for seek/replay; `None` = retain until acked by
    /// every subscription (queue-grade), `Some` = time-bounded replay
    /// window (stream-grade).
    pub retention: Option<Duration>,
}

// =====================================================================
// Messages
// =====================================================================

/// Per-key ordering unit. Messages sharing a key are delivered in publish
/// order within a subscription; that is the ONLY ordering promise.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct MessageKey(String);

impl MessageKey {
    /// Parses and validates a message key.
    ///
    /// # Errors
    /// Returns [`MessagingError::InvalidMessageKey`] when empty or oversized.
    pub fn parse(value: &str) -> Result<Self, MessagingError> {
        if value.is_empty() || value.len() > MAX_MESSAGE_KEY_LEN {
            Err(MessagingError::InvalidMessageKey)
        } else {
            Ok(Self(value.to_owned()))
        }
    }

    /// The canonical string form.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// What producers publish: payload plus routing/ordering metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageEnvelope {
    /// Ordering/partition key; `None` opts out of any ordering promise.
    pub key: Option<MessageKey>,
    /// Transport headers (tenant id, trace context, schema version).
    /// String-typed by design: headers are routing metadata, never payload.
    pub headers: BTreeMap<String, String>,
    /// Opaque payload bytes (schema ownership belongs to the surface
    /// kernels and their consumers, never to the substrate).
    pub payload: Vec<u8>,
}

impl MessageEnvelope {
    /// Validates envelope invariants (payload ceiling).
    ///
    /// # Errors
    /// Returns [`MessagingError::PayloadTooLarge`] when the payload exceeds
    /// [`MAX_PAYLOAD_BYTES`].
    pub fn validate(&self) -> Result<(), MessagingError> {
        if self.payload.len() > MAX_PAYLOAD_BYTES {
            return Err(MessagingError::PayloadTooLarge {
                size: self.payload.len(),
            });
        }
        Ok(())
    }
}

/// Broker-assigned message identity, unique within a topic.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct MessageId(String);

impl MessageId {
    /// Wraps a broker-assigned identity.
    #[must_use]
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// The canonical string form.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Totally-ordered position of a message within a topic; the seek/replay
/// coordinate for stream semantics.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct StreamPosition(pub u64);

impl StreamPosition {
    /// The earliest retained position.
    pub const EARLIEST: Self = Self(0);
}

/// Opaque per-delivery settlement token. Each (re)delivery mints a fresh
/// token; settling a token twice is a contract error (so consumer bugs
/// surface instead of silently double-settling).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AckToken(String);

impl AckToken {
    /// Wraps a broker-assigned settlement token.
    #[must_use]
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// The canonical string form.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One delivered message: envelope plus delivery bookkeeping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Delivery {
    /// Broker identity of the message.
    pub message_id: MessageId,
    /// Position of the message within the topic.
    pub position: StreamPosition,
    /// The published envelope.
    pub envelope: MessageEnvelope,
    /// 1 on first delivery; increments on each redelivery. Consumers use
    /// this to route poison messages to their dead-letter policy.
    pub delivery_count: u32,
    /// Settlement token for THIS delivery.
    pub ack_token: AckToken,
}

// =====================================================================
// The ONE substrate port
// =====================================================================

/// Topic administration. Topic specs are immutable; `ensure_topic` is
/// idempotent for an identical spec and rejects a conflicting replay.
pub trait MessagingAdmin {
    /// Creates the topic if absent; verifies the spec if present.
    ///
    /// # Errors
    /// Returns [`MessagingError::TopicSpecMismatch`] when the topic exists
    /// with a different spec; [`MessagingError::BackendUnavailable`] on
    /// transient failure.
    fn ensure_topic(&self, topic: &TopicName, spec: &TopicSpec) -> Result<(), MessagingError>;

    /// Creates the subscription on the topic if absent (idempotent). New
    /// subscriptions begin at the topic head (they see messages published
    /// after creation), matching Pulsar/Pub/Sub subscription semantics.
    ///
    /// # Errors
    /// Returns [`MessagingError::TopicNotFound`] when the topic has not
    /// been ensured; [`MessagingError::BackendUnavailable`] on transient
    /// failure.
    fn ensure_subscription(
        &self,
        topic: &TopicName,
        subscription: &SubscriptionName,
    ) -> Result<(), MessagingError>;
}

/// Producer edge. At-least-once: a timeout after the broker persisted the
/// write means a retry can duplicate — consumers dedup, the API never
/// promises exactly-once.
pub trait MessageProducer {
    /// Publishes one envelope; returns the broker identity.
    ///
    /// # Errors
    /// Returns [`MessagingError::TopicNotFound`] for un-ensured topics,
    /// [`MessagingError::PayloadTooLarge`] for oversized payloads, and
    /// [`MessagingError::BackendUnavailable`] on transient failure.
    fn publish(
        &self,
        topic: &TopicName,
        envelope: MessageEnvelope,
    ) -> Result<MessageId, MessagingError>;
}

/// Consumer edge. Within one subscription, deliveries compete (each
/// outstanding message is held by at most one in-flight delivery); across
/// subscriptions every subscription independently observes every message.
pub trait MessageConsumer {
    /// Pulls up to `max` available deliveries for the subscription.
    ///
    /// # Errors
    /// Returns [`MessagingError::SubscriptionNotFound`] for un-ensured
    /// subscriptions and [`MessagingError::BackendUnavailable`] on
    /// transient failure.
    fn receive(
        &self,
        topic: &TopicName,
        subscription: &SubscriptionName,
        max: NonZeroU32,
    ) -> Result<Vec<Delivery>, MessagingError>;

    /// Settles a delivery as processed; the message is never redelivered
    /// to this subscription.
    ///
    /// # Errors
    /// Returns [`MessagingError::UnknownAckToken`] for unknown or
    /// already-settled tokens.
    fn ack(&self, token: &AckToken) -> Result<(), MessagingError>;

    /// Settles a delivery as failed; the message returns to the
    /// subscription backlog with an incremented delivery count.
    ///
    /// # Errors
    /// Returns [`MessagingError::UnknownAckToken`] for unknown or
    /// already-settled tokens.
    fn nack(&self, token: &AckToken) -> Result<(), MessagingError>;

    /// Rewinds the subscription cursor: every retained message at or after
    /// `position` becomes deliverable again (stream replay). In-flight
    /// deliveries for the subscription are invalidated.
    ///
    /// # Errors
    /// Returns [`MessagingError::SubscriptionNotFound`] for un-ensured
    /// subscriptions and [`MessagingError::InvalidSeekPosition`] when
    /// `position` is beyond the topic head.
    fn seek(
        &self,
        topic: &TopicName,
        subscription: &SubscriptionName,
        position: StreamPosition,
    ) -> Result<(), MessagingError>;
}

/// The ONE messaging substrate (ADR-0536 D-13): admin + producer +
/// consumer edges over a single broker estate. The queue, stream, and bus
/// boundary kernels each compose THIS trait — services never speak a raw
/// broker protocol.
pub trait MessagingSubstrate: MessagingAdmin + MessageProducer + MessageConsumer {}

impl<T: MessagingAdmin + MessageProducer + MessageConsumer> MessagingSubstrate for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_name_accepts_canonical_slugs() {
        for ok in ["metering.usage-events", "a", "a.b-c.d0"] {
            assert_eq!(TopicName::parse(ok).unwrap().as_str(), ok);
        }
    }

    #[test]
    fn topic_name_rejects_malformed_slugs() {
        for bad in ["", "UPPER", "-lead", "trail-", ".lead", "trail.", "sp ace"] {
            assert!(TopicName::parse(bad).is_err(), "{bad:?} must be rejected");
        }
        let oversized = "a".repeat(MAX_TOPIC_NAME_LEN + 1);
        assert!(TopicName::parse(&oversized).is_err());
    }

    #[test]
    fn message_key_bounds_are_enforced() {
        assert!(MessageKey::parse("").is_err());
        assert!(MessageKey::parse(&"k".repeat(MAX_MESSAGE_KEY_LEN + 1)).is_err());
        assert!(MessageKey::parse("tenant-0001").is_ok());
    }

    #[test]
    fn envelope_payload_ceiling_is_enforced() {
        let envelope = MessageEnvelope {
            key: None,
            headers: BTreeMap::new(),
            payload: vec![0_u8; MAX_PAYLOAD_BYTES + 1],
        };
        assert!(matches!(
            envelope.validate(),
            Err(MessagingError::PayloadTooLarge { .. })
        ));
    }

    #[test]
    fn errors_render_diagnostics() {
        let rendered = MessagingError::TopicSpecMismatch { topic: "t".into() }.to_string();
        assert!(rendered.contains("conflicting spec"));
    }
}
