//! The generic conformance checks every messaging substrate adapter must
//! pass (the messaging analogue of the resource-provider contract harness).
//!
//! Each check is a pure generic fn over a [`SubstrateFixture`]; it builds a
//! FRESH substrate, drives it through the D-13 contract scenario, and
//! returns the first divergence as a typed [`ConformanceViolation`] (never
//! panicking — assertion style belongs to the caller's test harness, the
//! diagnosis belongs here). The transitional Pulsar adapter runs the SAME
//! checks against a real broker in its integration rung (AMENDMENT 7 test
//! ladder), so reference and production substrates are held to one
//! specification.

use std::collections::BTreeMap;
use std::fmt;
use std::num::NonZeroU32;

use crate::{
    Delivery, MessageConsumer, MessageEnvelope, MessageKey, MessageProducer, MessagingAdmin,
    MessagingError, MessagingSubstrate, StreamPosition, SubscriptionName, TopicName, TopicSpec,
};

/// Receive-loop budget so a non-conformant substrate cannot hang a check.
pub const MAX_RECEIVE_ROUNDS: u32 = 64;

/// A single conformance divergence: which check failed and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceViolation {
    pub check: &'static str, // data_class: INTERNAL_ONLY
    pub detail: String,      // data_class: INTERNAL_ONLY
}

impl fmt::Display for ConformanceViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.check, self.detail)
    }
}

impl std::error::Error for ConformanceViolation {}

fn violation(check: &'static str, detail: impl Into<String>) -> ConformanceViolation {
    ConformanceViolation {
        check,
        detail: detail.into(),
    }
}

/// What an adapter supplies to run the harness: a fresh substrate per
/// check plus a topic spec for each loss class.
pub trait SubstrateFixture {
    /// The substrate under test.
    type Substrate: MessagingSubstrate;

    /// A FRESH substrate with no topics (checks never share state).
    fn fresh_substrate(&self) -> Self::Substrate;

    /// The spec the harness uses for its scratch topics.
    fn topic_spec(&self) -> TopicSpec;
}

fn batch_size(value: u32) -> Result<NonZeroU32, ConformanceViolation> {
    NonZeroU32::new(value).ok_or_else(|| violation("fixture", "zero batch size"))
}

fn scratch_topic(check: &'static str) -> Result<TopicName, ConformanceViolation> {
    TopicName::parse("conformance.scratch").map_err(|e| violation(check, e.to_string()))
}

fn scratch_subscription(
    check: &'static str,
    name: &str,
) -> Result<SubscriptionName, ConformanceViolation> {
    SubscriptionName::parse(name).map_err(|e| violation(check, e.to_string()))
}

fn envelope(
    check: &'static str,
    key: Option<&str>,
    ordinal: u32,
) -> Result<MessageEnvelope, ConformanceViolation> {
    let key = match key {
        Some(value) => Some(MessageKey::parse(value).map_err(|e| violation(check, e.to_string()))?),
        None => None,
    };
    Ok(MessageEnvelope {
        key,
        headers: BTreeMap::new(),
        payload: format!("payload-{ordinal:04}").into_bytes(),
    })
}

fn drain<S: MessagingSubstrate>(
    check: &'static str,
    substrate: &S,
    topic: &TopicName,
    subscription: &SubscriptionName,
    expected: usize,
) -> Result<Vec<Delivery>, ConformanceViolation> {
    let max = batch_size(16)?;
    let mut collected = Vec::new();
    for _ in 0..MAX_RECEIVE_ROUNDS {
        let mut round = substrate
            .receive(topic, subscription, max)
            .map_err(|e| violation(check, format!("receive failed: {e}")))?;
        if round.is_empty() {
            break;
        }
        collected.append(&mut round);
        if collected.len() >= expected {
            break;
        }
    }
    Ok(collected)
}

/// Publish-then-receive: an ensured subscription observes every message
/// published after it, exactly once while unsettled deliveries are held.
pub fn check_publish_then_receive<F: SubstrateFixture>(
    fixture: &F,
) -> Result<(), ConformanceViolation> {
    const CHECK: &str = "publish_then_receive";
    let substrate = fixture.fresh_substrate();
    let topic = scratch_topic(CHECK)?;
    let subscription = scratch_subscription(CHECK, "main")?;
    substrate
        .ensure_topic(&topic, &fixture.topic_spec())
        .map_err(|e| violation(CHECK, format!("ensure_topic failed: {e}")))?;
    substrate
        .ensure_subscription(&topic, &subscription)
        .map_err(|e| violation(CHECK, format!("ensure_subscription failed: {e}")))?;
    for ordinal in 0..3_u32 {
        substrate
            .publish(&topic, envelope(CHECK, None, ordinal)?)
            .map_err(|e| violation(CHECK, format!("publish failed: {e}")))?;
    }
    let deliveries = drain(CHECK, &substrate, &topic, &subscription, 3)?;
    if deliveries.len() != 3 {
        return Err(violation(
            CHECK,
            format!("expected 3 deliveries, got {}", deliveries.len()),
        ));
    }
    // While all three are in flight, nothing further is deliverable.
    let extra = substrate
        .receive(&topic, &subscription, batch_size(16)?)
        .map_err(|e| violation(CHECK, format!("receive failed: {e}")))?;
    if !extra.is_empty() {
        return Err(violation(
            CHECK,
            format!("{} in-flight deliveries were handed out twice", extra.len()),
        ));
    }
    Ok(())
}

/// Idempotent ensure: replaying `ensure_topic` with the identical spec is a
/// no-op; replaying with a conflicting spec is rejected (immutable specs).
pub fn check_topic_spec_immutability<F: SubstrateFixture>(
    fixture: &F,
) -> Result<(), ConformanceViolation> {
    const CHECK: &str = "topic_spec_immutability";
    let substrate = fixture.fresh_substrate();
    let topic = scratch_topic(CHECK)?;
    let spec = fixture.topic_spec();
    substrate
        .ensure_topic(&topic, &spec)
        .map_err(|e| violation(CHECK, format!("ensure_topic failed: {e}")))?;
    substrate
        .ensure_topic(&topic, &spec)
        .map_err(|e| violation(CHECK, format!("identical re-ensure must be a no-op: {e}")))?;
    let conflicting = TopicSpec {
        loss_class: match spec.loss_class {
            crate::LossClass::NeverLose => crate::LossClass::Expendable,
            crate::LossClass::Expendable => crate::LossClass::NeverLose,
        },
        retention: spec.retention,
    };
    match substrate.ensure_topic(&topic, &conflicting) {
        Err(MessagingError::TopicSpecMismatch { .. }) => Ok(()),
        Err(other) => Err(violation(
            CHECK,
            format!("conflicting re-ensure raised the wrong error: {other}"),
        )),
        Ok(()) => Err(violation(
            CHECK,
            "conflicting re-ensure was accepted; topic specs must be immutable",
        )),
    }
}

/// At-least-once: a nacked delivery is redelivered with an incremented
/// delivery count; an acked delivery is never seen again.
pub fn check_redelivery_after_nack<F: SubstrateFixture>(
    fixture: &F,
) -> Result<(), ConformanceViolation> {
    const CHECK: &str = "redelivery_after_nack";
    let substrate = fixture.fresh_substrate();
    let topic = scratch_topic(CHECK)?;
    let subscription = scratch_subscription(CHECK, "main")?;
    substrate
        .ensure_topic(&topic, &fixture.topic_spec())
        .map_err(|e| violation(CHECK, format!("ensure_topic failed: {e}")))?;
    substrate
        .ensure_subscription(&topic, &subscription)
        .map_err(|e| violation(CHECK, format!("ensure_subscription failed: {e}")))?;
    substrate
        .publish(&topic, envelope(CHECK, None, 0)?)
        .map_err(|e| violation(CHECK, format!("publish failed: {e}")))?;

    let first = drain(CHECK, &substrate, &topic, &subscription, 1)?;
    let [first] = first.as_slice() else {
        return Err(violation(CHECK, "expected exactly one delivery"));
    };
    if first.delivery_count != 1 {
        return Err(violation(
            CHECK,
            format!("first delivery_count was {}", first.delivery_count),
        ));
    }
    substrate
        .nack(&first.ack_token)
        .map_err(|e| violation(CHECK, format!("nack failed: {e}")))?;

    let second = drain(CHECK, &substrate, &topic, &subscription, 1)?;
    let [second] = second.as_slice() else {
        return Err(violation(CHECK, "nacked message was not redelivered"));
    };
    if second.message_id != first.message_id {
        return Err(violation(CHECK, "redelivery returned a different message"));
    }
    if second.delivery_count != 2 {
        return Err(violation(
            CHECK,
            format!("redelivery count was {}", second.delivery_count),
        ));
    }
    substrate
        .ack(&second.ack_token)
        .map_err(|e| violation(CHECK, format!("ack failed: {e}")))?;
    let after_ack = drain(CHECK, &substrate, &topic, &subscription, 1)?;
    if !after_ack.is_empty() {
        return Err(violation(CHECK, "acked message was redelivered"));
    }
    Ok(())
}

/// Settlement tokens are single-use: a second settle of the same token is
/// rejected, and a stale token cannot resurrect a settled message.
pub fn check_settlement_token_single_use<F: SubstrateFixture>(
    fixture: &F,
) -> Result<(), ConformanceViolation> {
    const CHECK: &str = "settlement_token_single_use";
    let substrate = fixture.fresh_substrate();
    let topic = scratch_topic(CHECK)?;
    let subscription = scratch_subscription(CHECK, "main")?;
    substrate
        .ensure_topic(&topic, &fixture.topic_spec())
        .map_err(|e| violation(CHECK, format!("ensure_topic failed: {e}")))?;
    substrate
        .ensure_subscription(&topic, &subscription)
        .map_err(|e| violation(CHECK, format!("ensure_subscription failed: {e}")))?;
    substrate
        .publish(&topic, envelope(CHECK, None, 0)?)
        .map_err(|e| violation(CHECK, format!("publish failed: {e}")))?;
    let deliveries = drain(CHECK, &substrate, &topic, &subscription, 1)?;
    let [delivery] = deliveries.as_slice() else {
        return Err(violation(CHECK, "expected exactly one delivery"));
    };
    substrate
        .ack(&delivery.ack_token)
        .map_err(|e| violation(CHECK, format!("ack failed: {e}")))?;
    match substrate.nack(&delivery.ack_token) {
        Err(MessagingError::UnknownAckToken) => Ok(()),
        Err(other) => Err(violation(
            CHECK,
            format!("double settle raised the wrong error: {other}"),
        )),
        Ok(()) => Err(violation(
            CHECK,
            "settled token was accepted again; tokens must be single-use",
        )),
    }
}

/// Per-key ordering: messages sharing a key arrive in publish order even
/// across a nack/redelivery cycle; the key's later messages are never
/// delivered ahead of its earlier in-flight message.
pub fn check_per_key_ordering<F: SubstrateFixture>(
    fixture: &F,
) -> Result<(), ConformanceViolation> {
    const CHECK: &str = "per_key_ordering";
    let substrate = fixture.fresh_substrate();
    let topic = scratch_topic(CHECK)?;
    let subscription = scratch_subscription(CHECK, "main")?;
    substrate
        .ensure_topic(&topic, &fixture.topic_spec())
        .map_err(|e| violation(CHECK, format!("ensure_topic failed: {e}")))?;
    substrate
        .ensure_subscription(&topic, &subscription)
        .map_err(|e| violation(CHECK, format!("ensure_subscription failed: {e}")))?;
    for ordinal in 0..3_u32 {
        substrate
            .publish(&topic, envelope(CHECK, Some("tenant-a"), ordinal)?)
            .map_err(|e| violation(CHECK, format!("publish failed: {e}")))?;
    }

    // One at a time: while message 0 is in flight, the key is blocked.
    let one = batch_size(1)?;
    let first = substrate
        .receive(&topic, &subscription, one)
        .map_err(|e| violation(CHECK, format!("receive failed: {e}")))?;
    let [first] = first.as_slice() else {
        return Err(violation(CHECK, "expected one delivery for the key head"));
    };
    let blocked = substrate
        .receive(&topic, &subscription, one)
        .map_err(|e| violation(CHECK, format!("receive failed: {e}")))?;
    if !blocked.is_empty() {
        return Err(violation(
            CHECK,
            "a later message of an in-flight key was delivered out of order",
        ));
    }
    // Nack and confirm the SAME head is redelivered before its successors.
    substrate
        .nack(&first.ack_token)
        .map_err(|e| violation(CHECK, format!("nack failed: {e}")))?;
    let mut seen = Vec::new();
    for _ in 0..MAX_RECEIVE_ROUNDS {
        let round = substrate
            .receive(&topic, &subscription, one)
            .map_err(|e| violation(CHECK, format!("receive failed: {e}")))?;
        let [delivery] = round.as_slice() else { break };
        substrate
            .ack(&delivery.ack_token)
            .map_err(|e| violation(CHECK, format!("ack failed: {e}")))?;
        seen.push(delivery.position);
        if seen.len() == 3 {
            break;
        }
    }
    let mut sorted = seen.clone();
    sorted.sort_unstable();
    if seen.len() != 3 || seen != sorted {
        return Err(violation(
            CHECK,
            format!("key deliveries out of publish order: {seen:?}"),
        ));
    }
    Ok(())
}

/// Fan-out: two subscriptions on one topic each independently observe and
/// settle every message (bus semantics over the one substrate).
pub fn check_subscription_fanout<F: SubstrateFixture>(
    fixture: &F,
) -> Result<(), ConformanceViolation> {
    const CHECK: &str = "subscription_fanout";
    let substrate = fixture.fresh_substrate();
    let topic = scratch_topic(CHECK)?;
    let sub_a = scratch_subscription(CHECK, "consumer-a")?;
    let sub_b = scratch_subscription(CHECK, "consumer-b")?;
    substrate
        .ensure_topic(&topic, &fixture.topic_spec())
        .map_err(|e| violation(CHECK, format!("ensure_topic failed: {e}")))?;
    for subscription in [&sub_a, &sub_b] {
        substrate
            .ensure_subscription(&topic, subscription)
            .map_err(|e| violation(CHECK, format!("ensure_subscription failed: {e}")))?;
    }
    for ordinal in 0..2_u32 {
        substrate
            .publish(&topic, envelope(CHECK, None, ordinal)?)
            .map_err(|e| violation(CHECK, format!("publish failed: {e}")))?;
    }
    for subscription in [&sub_a, &sub_b] {
        let deliveries = drain(CHECK, &substrate, &topic, subscription, 2)?;
        if deliveries.len() != 2 {
            return Err(violation(
                CHECK,
                format!(
                    "subscription {} saw {} of 2 messages",
                    subscription.as_str(),
                    deliveries.len()
                ),
            ));
        }
        for delivery in &deliveries {
            substrate
                .ack(&delivery.ack_token)
                .map_err(|e| violation(CHECK, format!("ack failed: {e}")))?;
        }
    }
    Ok(())
}

/// Seek/replay: after acking everything, a seek to EARLIEST makes every
/// retained message deliverable again (stream semantics).
pub fn check_seek_replay<F: SubstrateFixture>(fixture: &F) -> Result<(), ConformanceViolation> {
    const CHECK: &str = "seek_replay";
    let substrate = fixture.fresh_substrate();
    let topic = scratch_topic(CHECK)?;
    let subscription = scratch_subscription(CHECK, "reader")?;
    substrate
        .ensure_topic(&topic, &fixture.topic_spec())
        .map_err(|e| violation(CHECK, format!("ensure_topic failed: {e}")))?;
    substrate
        .ensure_subscription(&topic, &subscription)
        .map_err(|e| violation(CHECK, format!("ensure_subscription failed: {e}")))?;
    for ordinal in 0..3_u32 {
        substrate
            .publish(&topic, envelope(CHECK, None, ordinal)?)
            .map_err(|e| violation(CHECK, format!("publish failed: {e}")))?;
    }
    let first_pass = drain(CHECK, &substrate, &topic, &subscription, 3)?;
    if first_pass.len() != 3 {
        return Err(violation(
            CHECK,
            format!("first pass saw {} of 3 messages", first_pass.len()),
        ));
    }
    for delivery in &first_pass {
        substrate
            .ack(&delivery.ack_token)
            .map_err(|e| violation(CHECK, format!("ack failed: {e}")))?;
    }
    substrate
        .seek(&topic, &subscription, StreamPosition::EARLIEST)
        .map_err(|e| violation(CHECK, format!("seek failed: {e}")))?;
    let replay = drain(CHECK, &substrate, &topic, &subscription, 3)?;
    if replay.len() != 3 {
        return Err(violation(
            CHECK,
            format!("replay after seek saw {} of 3 messages", replay.len()),
        ));
    }
    let first_ids: Vec<_> = first_pass.iter().map(|d| &d.message_id).collect();
    let replay_ids: Vec<_> = replay.iter().map(|d| &d.message_id).collect();
    if first_ids != replay_ids {
        return Err(violation(CHECK, "replay order diverged from log order"));
    }
    Ok(())
}

/// One conformance check as run by [`run_all`].
pub type Check<F> = fn(&F) -> Result<(), ConformanceViolation>;

/// Runs every check, collecting all violations (so an adapter sees its
/// full divergence surface in one run, not one failure at a time).
pub fn run_all<F: SubstrateFixture>(fixture: &F) -> Vec<ConformanceViolation> {
    let checks: [Check<F>; 7] = [
        check_publish_then_receive,
        check_topic_spec_immutability,
        check_redelivery_after_nack,
        check_settlement_token_single_use,
        check_per_key_ordering,
        check_subscription_fanout,
        check_seek_replay,
    ];
    checks
        .iter()
        .filter_map(|check| check(fixture).err())
        .collect()
}
