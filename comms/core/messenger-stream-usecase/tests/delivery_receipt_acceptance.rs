//! Integration-level acceptance tests for the delivery-receipt state machine.
//!
//! Covers all three subtask acceptance criteria from the slice spec:
//!   ST-1: DeliveryStatus enum ordering, totality, terminal-forward constraint
//!   ST-2: acknowledge_delivery — happy path, idempotent replay, regression rejection,
//!         context-validation → Api error mapping
//!   ST-3: aggregate_channel_delivery — min-status semantics, all-Read, empty slice

use comms_messenger_stream_api::{
    AuthorizedMessengerContext, MessengerApiContext, MessengerApiError,
};
use comms_messenger_stream_usecase::{
    ChannelDeliveryAggregate, DeliveryStatus, MessengerUsecaseError, RecipientDeliveryState,
    acknowledge_delivery, aggregate_channel_delivery,
};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn work_ctx(principal: &str) -> AuthorizedMessengerContext {
    AuthorizedMessengerContext {
        context: MessengerApiContext::Work,
        scope_ref: "tenant:t".into(),
        principal_ref: principal.into(),
        idempotency_key: "ctx-idem".into(),
        policy_decision_ref: "cedar:allow:delivery-ack".into(),
        audit_correlation_id: "audit-x".into(),
    }
}

fn initial_state(recipient: &str) -> RecipientDeliveryState {
    RecipientDeliveryState {
        recipient_ref: recipient.into(),
        status: DeliveryStatus::Sent,
        ordinal: 0,
        last_idempotency_key: "init".into(),
    }
}

fn read_state(recipient: &str) -> RecipientDeliveryState {
    RecipientDeliveryState {
        recipient_ref: recipient.into(),
        status: DeliveryStatus::Read,
        ordinal: 2,
        last_idempotency_key: "k-read".into(),
    }
}

// ---------------------------------------------------------------------------
// ST-1: DeliveryStatus enum — total order + terminal-forward guarantee
// ---------------------------------------------------------------------------

#[test]
fn delivery_status_total_order_sent_lt_delivered_lt_read() {
    assert!(DeliveryStatus::Sent < DeliveryStatus::Delivered);
    assert!(DeliveryStatus::Delivered < DeliveryStatus::Read);
    assert!(DeliveryStatus::Sent < DeliveryStatus::Read);
}

#[test]
fn delivery_status_equality_is_reflexive() {
    assert_eq!(DeliveryStatus::Sent, DeliveryStatus::Sent);
    assert_eq!(DeliveryStatus::Delivered, DeliveryStatus::Delivered);
    assert_eq!(DeliveryStatus::Read, DeliveryStatus::Read);
}

#[test]
fn read_is_the_maximum_status() {
    // Read must be >= all other variants — i.e. terminal-forward.
    assert!(DeliveryStatus::Read >= DeliveryStatus::Delivered);
    assert!(DeliveryStatus::Read >= DeliveryStatus::Sent);
}

#[test]
fn sent_is_the_minimum_status() {
    assert!(DeliveryStatus::Sent <= DeliveryStatus::Delivered);
    assert!(DeliveryStatus::Sent <= DeliveryStatus::Read);
}

// ---------------------------------------------------------------------------
// ST-2: acknowledge_delivery — happy path Sent -> Delivered -> Read
// ---------------------------------------------------------------------------

#[test]
fn happy_path_sent_to_delivered_increments_ordinal_and_records_key() {
    let ctx = work_ctx("user:alice");
    let s0 = initial_state("user:alice");

    let s1 = acknowledge_delivery(&ctx, &s0, "user:alice", DeliveryStatus::Delivered, "k1")
        .expect("forward transition must succeed");

    assert_eq!(s1.status, DeliveryStatus::Delivered);
    assert_eq!(s1.ordinal, 1, "ordinal must increment once");
    assert_eq!(s1.last_idempotency_key, "k1");
    assert_eq!(s1.recipient_ref, "user:alice");
}

#[test]
fn happy_path_delivered_to_read_increments_ordinal_again() {
    let ctx = work_ctx("user:alice");
    let s1 = RecipientDeliveryState {
        recipient_ref: "user:alice".into(),
        status: DeliveryStatus::Delivered,
        ordinal: 1,
        last_idempotency_key: "k1".into(),
    };

    let s2 = acknowledge_delivery(&ctx, &s1, "user:alice", DeliveryStatus::Read, "k2")
        .expect("Delivered -> Read must succeed");

    assert_eq!(s2.status, DeliveryStatus::Read);
    assert_eq!(s2.ordinal, 2);
    assert_eq!(s2.last_idempotency_key, "k2");
}

#[test]
fn full_progression_sent_delivered_read_produces_ordinal_two() {
    let ctx = work_ctx("user:bob");
    let s0 = initial_state("user:bob");

    let s1 = acknowledge_delivery(&ctx, &s0, "user:bob", DeliveryStatus::Delivered, "k1").unwrap();
    let s2 = acknowledge_delivery(&ctx, &s1, "user:bob", DeliveryStatus::Read, "k2").unwrap();

    assert_eq!(s2.status, DeliveryStatus::Read);
    assert_eq!(s2.ordinal, 2);
}

// ---------------------------------------------------------------------------
// ST-2: acknowledge_delivery — idempotent replay (same key -> unchanged Ok)
// ---------------------------------------------------------------------------

#[test]
fn idempotent_replay_with_same_key_returns_unchanged_state() {
    let ctx = work_ctx("user:alice");
    let s0 = initial_state("user:alice");

    let s1 =
        acknowledge_delivery(&ctx, &s0, "user:alice", DeliveryStatus::Delivered, "k1").unwrap();
    // Replay: same idempotency key — must return exactly s1, no ordinal bump.
    let s1_replay =
        acknowledge_delivery(&ctx, &s1, "user:alice", DeliveryStatus::Delivered, "k1").unwrap();

    assert_eq!(s1_replay, s1, "replay must return the unchanged state");
    assert_eq!(s1_replay.ordinal, 1, "ordinal must not advance on replay");
}

#[test]
fn idempotent_replay_does_not_regress_status() {
    let ctx = work_ctx("user:alice");
    let s2 = read_state("user:alice");

    // Replay with the same key that produced the Read state — must return unchanged.
    let replayed =
        acknowledge_delivery(&ctx, &s2, "user:alice", DeliveryStatus::Read, "k-read").unwrap();

    assert_eq!(replayed.status, DeliveryStatus::Read);
    assert_eq!(replayed.ordinal, 2);
}

// ---------------------------------------------------------------------------
// ST-2: acknowledge_delivery — regression rejection
// ---------------------------------------------------------------------------

#[test]
fn regression_read_to_delivered_returns_illegal_transition_error() {
    let ctx = work_ctx("user:alice");
    let s = read_state("user:alice");

    let err = acknowledge_delivery(&ctx, &s, "user:alice", DeliveryStatus::Delivered, "k-new")
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
fn regression_read_to_sent_returns_illegal_transition_error() {
    let ctx = work_ctx("user:alice");
    let s = read_state("user:alice");

    let err =
        acknowledge_delivery(&ctx, &s, "user:alice", DeliveryStatus::Sent, "k-new").unwrap_err();

    assert_eq!(
        err,
        MessengerUsecaseError::IllegalDeliveryTransition {
            from: DeliveryStatus::Read,
            to: DeliveryStatus::Sent,
        }
    );
}

#[test]
fn same_status_with_new_key_is_non_forward_transition_and_rejected() {
    let ctx = work_ctx("user:alice");
    let s = RecipientDeliveryState {
        recipient_ref: "user:alice".into(),
        status: DeliveryStatus::Delivered,
        ordinal: 1,
        last_idempotency_key: "k1".into(),
    };

    let err = acknowledge_delivery(
        &ctx,
        &s,
        "user:alice",
        DeliveryStatus::Delivered,
        "k-different",
    )
    .unwrap_err();

    assert_eq!(
        err,
        MessengerUsecaseError::IllegalDeliveryTransition {
            from: DeliveryStatus::Delivered,
            to: DeliveryStatus::Delivered,
        }
    );
}

// ---------------------------------------------------------------------------
// ST-2: acknowledge_delivery — context validation maps to Api error
// ---------------------------------------------------------------------------

#[test]
fn missing_idempotency_key_in_context_maps_to_api_error() {
    let bad_ctx = AuthorizedMessengerContext {
        context: MessengerApiContext::Work,
        scope_ref: "tenant:t".into(),
        principal_ref: "user:alice".into(),
        idempotency_key: "".into(), // triggers MissingIdempotencyKey
        policy_decision_ref: "cedar:allow".into(),
        audit_correlation_id: "audit".into(),
    };
    let s0 = initial_state("user:alice");

    let err = acknowledge_delivery(&bad_ctx, &s0, "user:alice", DeliveryStatus::Delivered, "k1")
        .unwrap_err();

    assert_eq!(
        err,
        MessengerUsecaseError::Api(MessengerApiError::MissingIdempotencyKey)
    );
}

#[test]
fn missing_policy_decision_in_context_maps_to_api_error() {
    let bad_ctx = AuthorizedMessengerContext {
        context: MessengerApiContext::Work,
        scope_ref: "tenant:t".into(),
        principal_ref: "user:alice".into(),
        idempotency_key: "k".into(),
        policy_decision_ref: "".into(), // triggers MissingPolicyDecision
        audit_correlation_id: "audit".into(),
    };
    let s0 = initial_state("user:alice");

    let err = acknowledge_delivery(&bad_ctx, &s0, "user:alice", DeliveryStatus::Delivered, "k1")
        .unwrap_err();

    assert_eq!(
        err,
        MessengerUsecaseError::Api(MessengerApiError::MissingPolicyDecision)
    );
}

#[test]
fn wrong_scope_prefix_for_context_kind_maps_to_api_error() {
    let bad_ctx = AuthorizedMessengerContext {
        context: MessengerApiContext::Work,
        scope_ref: "person:u".into(), // Work requires tenant: prefix
        principal_ref: "user:alice".into(),
        idempotency_key: "k".into(),
        policy_decision_ref: "cedar:allow".into(),
        audit_correlation_id: "audit".into(),
    };
    let s0 = initial_state("user:alice");

    let err = acknowledge_delivery(&bad_ctx, &s0, "user:alice", DeliveryStatus::Delivered, "k1")
        .unwrap_err();

    assert_eq!(
        err,
        MessengerUsecaseError::Api(MessengerApiError::MissingTenantScope)
    );
}

#[test]
fn principal_mismatch_between_context_and_recipient_is_rejected() {
    let ctx = work_ctx("user:alice");
    let s0 = initial_state("user:bob");

    let err =
        acknowledge_delivery(&ctx, &s0, "user:bob", DeliveryStatus::Delivered, "k1").unwrap_err();

    assert_eq!(err, MessengerUsecaseError::PrincipalMismatch);
}

// ---------------------------------------------------------------------------
// ST-3: aggregate_channel_delivery — min-status semantics
// ---------------------------------------------------------------------------

#[test]
fn aggregate_returns_sent_when_any_recipient_is_sent() {
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
    assert_eq!(
        agg.aggregate_status,
        DeliveryStatus::Sent,
        "one Sent recipient must pull aggregate down to Sent"
    );
}

#[test]
fn aggregate_returns_delivered_when_mix_of_delivered_and_read() {
    let recipients = vec![
        RecipientDeliveryState {
            recipient_ref: "user:a".into(),
            status: DeliveryStatus::Delivered,
            ordinal: 1,
            last_idempotency_key: "k1".into(),
        },
        RecipientDeliveryState {
            recipient_ref: "user:b".into(),
            status: DeliveryStatus::Read,
            ordinal: 2,
            last_idempotency_key: "k2".into(),
        },
    ];

    let agg = aggregate_channel_delivery("ch:2", &recipients);

    assert_eq!(agg.aggregate_status, DeliveryStatus::Delivered);
}

#[test]
fn aggregate_returns_read_when_all_recipients_are_read() {
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
        RecipientDeliveryState {
            recipient_ref: "user:c".into(),
            status: DeliveryStatus::Read,
            ordinal: 2,
            last_idempotency_key: "k2".into(),
        },
    ];

    let agg = aggregate_channel_delivery("ch:3", &recipients);

    assert_eq!(
        agg.aggregate_status,
        DeliveryStatus::Read,
        "all Read recipients must yield Read aggregate"
    );
}

#[test]
fn aggregate_empty_slice_defaults_to_sent() {
    let agg = aggregate_channel_delivery("ch:empty", &[]);

    assert_eq!(
        agg.aggregate_status,
        DeliveryStatus::Sent,
        "empty slice must default to Sent"
    );
    assert_eq!(agg.channel_id, "ch:empty");
}

#[test]
fn aggregate_single_recipient_sent_yields_sent() {
    let recipients = vec![RecipientDeliveryState {
        recipient_ref: "user:solo".into(),
        status: DeliveryStatus::Sent,
        ordinal: 0,
        last_idempotency_key: "init".into(),
    }];

    let agg = aggregate_channel_delivery("ch:solo", &recipients);

    assert_eq!(agg.aggregate_status, DeliveryStatus::Sent);
}

#[test]
fn aggregate_preserves_channel_id_from_caller() {
    let agg = aggregate_channel_delivery("channel:explicit-id", &[]);

    assert_eq!(agg.channel_id, "channel:explicit-id");
}

#[test]
fn aggregate_result_type_is_channel_delivery_aggregate() {
    // Type-level assertion: the return type must be ChannelDeliveryAggregate.
    let result: ChannelDeliveryAggregate = aggregate_channel_delivery("ch:type-check", &[]);
    let _ = result;
}
