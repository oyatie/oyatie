use comms_messenger_stream_api::AuthorizedMessengerContext;

use crate::MessengerUsecaseError;

// ---------------------------------------------------------------------------
// Subtask 1: Core types
// ---------------------------------------------------------------------------

/// Per-recipient delivery progression. Total order: Sent < Delivered < Read.
/// `Read` is the terminal-forward state; no regression is permitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DeliveryStatus {
    Sent,
    Delivered,
    Read,
}

/// Per-recipient delivery state held by the usecase layer.
/// `ordinal` is monotonically incremented on each accepted forward transition.
/// `last_idempotency_key` enables idempotent replay detection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipientDeliveryState {
    pub recipient_ref: String,
    pub status: DeliveryStatus,
    pub ordinal: u64,
    pub last_idempotency_key: String,
}

// ---------------------------------------------------------------------------
// Subtask 2: acknowledge_delivery — idempotent forward-only transition
// ---------------------------------------------------------------------------

/// Advance the delivery status of a single recipient.
///
/// Steps:
/// 1. Validate `ctx` — map `MessengerApiError` → `MessengerUsecaseError::Api`
/// 2. Principal check: `ctx.principal_ref` must equal `recipient`
/// 3. Idempotent replay: same key already on state → return unchanged `Ok`
/// 4. Regression: `target <= state.status` → `IllegalDeliveryTransition`
/// 5. Advance: new state with incremented ordinal
pub fn acknowledge_delivery(
    ctx: &AuthorizedMessengerContext,
    state: &RecipientDeliveryState,
    recipient: &str,
    target: DeliveryStatus,
    idempotency_key: &str,
) -> Result<RecipientDeliveryState, MessengerUsecaseError> {
    ctx.validate().map_err(MessengerUsecaseError::Api)?;
    if ctx.principal_ref != recipient {
        return Err(MessengerUsecaseError::PrincipalMismatch);
    }
    // Idempotent replay: same key means this request was already applied.
    if idempotency_key == state.last_idempotency_key {
        return Ok(state.clone());
    }
    // Monotonic non-regression.
    if target <= state.status {
        return Err(MessengerUsecaseError::IllegalDeliveryTransition {
            from: state.status,
            to: target,
        });
    }
    Ok(RecipientDeliveryState {
        recipient_ref: state.recipient_ref.clone(),
        status: target,
        ordinal: state.ordinal + 1,
        last_idempotency_key: idempotency_key.to_owned(),
    })
}

// ---------------------------------------------------------------------------
// Subtask 3: Channel-level aggregate
// ---------------------------------------------------------------------------

/// Aggregate delivery state across all recipients in a channel.
/// `aggregate_status` = min(recipient statuses); empty slice defaults to `Sent`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChannelDeliveryAggregate {
    pub channel_id: String,
    pub aggregate_status: DeliveryStatus,
}

/// Derive the channel-level delivery state from a slice of recipient states.
/// Reuses `MessageReceipt::channel_id` semantics — the caller supplies the id.
pub fn aggregate_channel_delivery(
    channel_id: &str,
    recipients: &[RecipientDeliveryState],
) -> ChannelDeliveryAggregate {
    let aggregate_status = recipients
        .iter()
        .map(|r| r.status)
        .min()
        .unwrap_or(DeliveryStatus::Sent);
    ChannelDeliveryAggregate {
        channel_id: channel_id.to_owned(),
        aggregate_status,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use comms_messenger_stream_api::{MessengerApiContext, MessengerApiError};

    fn work_ctx(principal: &str) -> AuthorizedMessengerContext {
        AuthorizedMessengerContext {
            context: MessengerApiContext::Work,
            scope_ref: "tenant:t".into(),
            principal_ref: principal.into(),
            idempotency_key: "idem-ctx".into(),
            policy_decision_ref: "cedar:allow:delivery-ack".into(),
            audit_correlation_id: "audit-1".into(),
        }
    }

    fn sent_state(recipient: &str) -> RecipientDeliveryState {
        RecipientDeliveryState {
            recipient_ref: recipient.into(),
            status: DeliveryStatus::Sent,
            ordinal: 0,
            last_idempotency_key: "init".into(),
        }
    }

    // Subtask 2 tests -------------------------------------------------------

    #[test]
    fn happy_path_sent_delivered_read() {
        let ctx = work_ctx("user:alice");
        let s0 = sent_state("user:alice");

        let s1 =
            acknowledge_delivery(&ctx, &s0, "user:alice", DeliveryStatus::Delivered, "k1").unwrap();
        assert_eq!(s1.status, DeliveryStatus::Delivered);
        assert_eq!(s1.ordinal, 1);
        assert_eq!(s1.last_idempotency_key, "k1");

        let s2 = acknowledge_delivery(&ctx, &s1, "user:alice", DeliveryStatus::Read, "k2").unwrap();
        assert_eq!(s2.status, DeliveryStatus::Read);
        assert_eq!(s2.ordinal, 2);
        assert_eq!(s2.last_idempotency_key, "k2");
    }

    #[test]
    fn idempotent_replay_returns_unchanged() {
        let ctx = work_ctx("user:alice");
        let s0 = sent_state("user:alice");

        let s1 =
            acknowledge_delivery(&ctx, &s0, "user:alice", DeliveryStatus::Delivered, "k1").unwrap();
        // Replay with same key — must return the already-advanced state unchanged.
        let s1_replay =
            acknowledge_delivery(&ctx, &s1, "user:alice", DeliveryStatus::Delivered, "k1").unwrap();
        assert_eq!(s1_replay, s1);
        assert_eq!(s1_replay.ordinal, 1);
    }

    #[test]
    fn regression_read_to_delivered_rejected() {
        let ctx = work_ctx("user:alice");
        let read_state = RecipientDeliveryState {
            recipient_ref: "user:alice".into(),
            status: DeliveryStatus::Read,
            ordinal: 2,
            last_idempotency_key: "k2".into(),
        };
        let err = acknowledge_delivery(
            &ctx,
            &read_state,
            "user:alice",
            DeliveryStatus::Delivered,
            "k3",
        )
        .unwrap_err();
        assert_eq!(
            err,
            MessengerUsecaseError::IllegalDeliveryTransition {
                from: DeliveryStatus::Read,
                to: DeliveryStatus::Delivered,
            }
        );
    }

    #[test]
    fn same_status_replay_without_key_match_is_regression() {
        let ctx = work_ctx("user:alice");
        let s = RecipientDeliveryState {
            recipient_ref: "user:alice".into(),
            status: DeliveryStatus::Delivered,
            ordinal: 1,
            last_idempotency_key: "k1".into(),
        };
        // Attempting Delivered -> Delivered with a new key is a non-forward transition.
        let err = acknowledge_delivery(&ctx, &s, "user:alice", DeliveryStatus::Delivered, "k2-new")
            .unwrap_err();
        assert_eq!(
            err,
            MessengerUsecaseError::IllegalDeliveryTransition {
                from: DeliveryStatus::Delivered,
                to: DeliveryStatus::Delivered,
            }
        );
    }

    #[test]
    fn context_validation_failure_maps_to_api_error() {
        let bad_ctx = AuthorizedMessengerContext {
            context: MessengerApiContext::Work,
            scope_ref: "tenant:t".into(),
            principal_ref: "user:alice".into(),
            idempotency_key: "".into(), // missing — triggers MissingIdempotencyKey
            policy_decision_ref: "cedar:allow".into(),
            audit_correlation_id: "audit".into(),
        };
        let s0 = sent_state("user:alice");
        let err =
            acknowledge_delivery(&bad_ctx, &s0, "user:alice", DeliveryStatus::Delivered, "k1")
                .unwrap_err();
        assert_eq!(
            err,
            MessengerUsecaseError::Api(MessengerApiError::MissingIdempotencyKey)
        );
    }

    #[test]
    fn principal_mismatch_rejected() {
        let ctx = work_ctx("user:alice");
        let s0 = sent_state("user:bob");
        let err = acknowledge_delivery(&ctx, &s0, "user:bob", DeliveryStatus::Delivered, "k1")
            .unwrap_err();
        assert_eq!(err, MessengerUsecaseError::PrincipalMismatch);
    }

    // Subtask 3 tests -------------------------------------------------------

    #[test]
    fn aggregate_min_status_mixed_recipients() {
        let recipients = vec![
            RecipientDeliveryState {
                recipient_ref: "user:a".into(),
                status: DeliveryStatus::Sent,
                ordinal: 0,
                last_idempotency_key: "init".into(),
            },
            RecipientDeliveryState {
                recipient_ref: "user:b".into(),
                status: DeliveryStatus::Read,
                ordinal: 2,
                last_idempotency_key: "k2".into(),
            },
            RecipientDeliveryState {
                recipient_ref: "user:c".into(),
                status: DeliveryStatus::Read,
                ordinal: 2,
                last_idempotency_key: "k2".into(),
            },
        ];
        let agg = aggregate_channel_delivery("ch:1", &recipients);
        assert_eq!(agg.channel_id, "ch:1");
        assert_eq!(agg.aggregate_status, DeliveryStatus::Sent);
    }

    #[test]
    fn aggregate_all_read() {
        let recipients = vec![
            RecipientDeliveryState {
                recipient_ref: "user:a".into(),
                status: DeliveryStatus::Read,
                ordinal: 2,
                last_idempotency_key: "k2".into(),
            },
            RecipientDeliveryState {
                recipient_ref: "user:b".into(),
                status: DeliveryStatus::Read,
                ordinal: 2,
                last_idempotency_key: "k2".into(),
            },
        ];
        let agg = aggregate_channel_delivery("ch:2", &recipients);
        assert_eq!(agg.aggregate_status, DeliveryStatus::Read);
    }

    #[test]
    fn aggregate_empty_slice_defaults_to_sent() {
        let agg = aggregate_channel_delivery("ch:3", &[]);
        assert_eq!(agg.aggregate_status, DeliveryStatus::Sent);
    }
}
