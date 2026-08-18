//! TDD RED tests — thread-lifecycle state-machine kernel.
//!
//! These tests define the *full* expected behaviour of `ThreadState`,
//! `ThreadLifecycle`, `ThreadSubscription`, and `ThreadSubscriptionMode` as
//! specified in the slice acceptance criteria (subtasks 1–3).  They are written
//! as integration tests exercising only the crate's public API surface so that
//! any gap in the public re-export or implementation causes a compile or
//! assertion failure here.
//!
//! ADR-0083 Tier 3: integration tests are permitted to call `.unwrap()` /
//! `.expect()` to assert invariants.
//!
//! ADR-0029 / ADR-0208: chat governance + InternalAuditOnly invariants —
//! thread-state fields carry `InternalOnly` data-class; subscription
//! `participant_ref` is `PII_IDENTIFYING`, mirroring the `MessageGovernance`
//! tagging pattern.
//!
//! The `OwnershipPillar` pillar-isolation invariant mirrors
//! `PresenceState::CrossPillarPresenceDenied` in `governance.rs`.

use comms_messenger_domain::{
    ChatError, OwnershipPillar, ThreadLifecycle, ThreadLifecycleCreate, ThreadState,
    ThreadSubscription, ThreadSubscriptionCreate, ThreadSubscriptionMode,
};
use oya_data_boundary_kernel::{DataClass, DataClassification};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn open_lifecycle() -> ThreadLifecycle {
    ThreadLifecycle::new(ThreadLifecycleCreate {
        thread_id: "thread-red-1".into(),
        tenant_id: "tenant-red-1".into(),
        initial_state: ThreadState::Open,
    })
    .unwrap()
}

/// Advance an `Open` thread to `state` using the minimal legal path.
fn lifecycle_in(state: ThreadState) -> ThreadLifecycle {
    let t = open_lifecycle();
    if state == ThreadState::Open {
        return t;
    }
    t.transition(state).unwrap()
}

fn work_subscription() -> ThreadSubscription {
    ThreadSubscription::new(ThreadSubscriptionCreate {
        thread_id: "thread-red-1".into(),
        tenant_id: "tenant-red-1".into(),
        participant_ref: "user:alice@work.example.com".into(),
        participant_pillar: OwnershipPillar::Work,
        thread_pillar: OwnershipPillar::Work,
    })
    .unwrap()
}

fn personal_subscription() -> ThreadSubscription {
    ThreadSubscription::new(ThreadSubscriptionCreate {
        thread_id: "thread-red-2".into(),
        tenant_id: "tenant-red-2".into(),
        participant_ref: "user:bob@personal.example.com".into(),
        participant_pillar: OwnershipPillar::Personal,
        thread_pillar: OwnershipPillar::Personal,
    })
    .unwrap()
}

// ---------------------------------------------------------------------------
// Subtask 1 — ThreadState enum + ThreadLifecycle constructor invariants
// ---------------------------------------------------------------------------

/// Happy-path: a valid `Open` `ThreadLifecycle` is created without error.
#[test]
fn new_lifecycle_open_state_returns_ok() {
    assert!(
        ThreadLifecycle::new(ThreadLifecycleCreate {
            thread_id: "t-1".into(),
            tenant_id: "ten-1".into(),
            initial_state: ThreadState::Open,
        })
        .is_ok()
    );
}

/// All four `ThreadState` variants are constructable as initial state.
#[test]
fn all_thread_states_are_constructable_as_initial_state() {
    for state in [
        ThreadState::Open,
        ThreadState::Locked,
        ThreadState::Resolved,
        ThreadState::Archived,
    ] {
        assert!(
            ThreadLifecycle::new(ThreadLifecycleCreate {
                thread_id: "t-any".into(),
                tenant_id: "ten-any".into(),
                initial_state: state,
            })
            .is_ok(),
            "should accept initial state {state:?}"
        );
    }
}

/// `thread_id` is tagged `InternalOnly`.
#[test]
fn thread_id_field_is_tagged_internal_only() {
    let t = open_lifecycle();
    assert_eq!(
        t.thread_id.data_class,
        DataClass::InternalOnly.into(),
        "thread_id must be INTERNAL_ONLY"
    );
}

/// `tenant_id` is tagged `InternalOnly`.
#[test]
fn tenant_id_field_is_tagged_internal_only() {
    let t = open_lifecycle();
    assert_eq!(
        t.tenant_id.data_class,
        DataClass::InternalOnly.into(),
        "tenant_id must be INTERNAL_ONLY"
    );
}

/// `state` field is tagged `InternalOnly`.
#[test]
fn state_field_is_tagged_internal_only() {
    let t = open_lifecycle();
    assert_eq!(
        t.state.data_class,
        DataClass::InternalOnly.into(),
        "state must be INTERNAL_ONLY"
    );
}

/// Empty `thread_id` is rejected with `InvalidThreadId`.
#[test]
fn empty_thread_id_returns_invalid_thread_id() {
    assert_eq!(
        ThreadLifecycle::new(ThreadLifecycleCreate {
            thread_id: String::new(),
            tenant_id: "ten-1".into(),
            initial_state: ThreadState::Open,
        }),
        Err(ChatError::InvalidThreadId)
    );
}

/// Whitespace-only `thread_id` is rejected with `InvalidThreadId`.
#[test]
fn whitespace_only_thread_id_returns_invalid_thread_id() {
    assert_eq!(
        ThreadLifecycle::new(ThreadLifecycleCreate {
            thread_id: "   ".into(),
            tenant_id: "ten-1".into(),
            initial_state: ThreadState::Open,
        }),
        Err(ChatError::InvalidThreadId)
    );
}

/// Empty `tenant_id` is rejected with `InvalidTenantId`.
#[test]
fn empty_tenant_id_returns_invalid_tenant_id() {
    assert_eq!(
        ThreadLifecycle::new(ThreadLifecycleCreate {
            thread_id: "t-1".into(),
            tenant_id: String::new(),
            initial_state: ThreadState::Open,
        }),
        Err(ChatError::InvalidTenantId)
    );
}

/// Whitespace-only `tenant_id` is rejected with `InvalidTenantId`.
#[test]
fn whitespace_only_tenant_id_returns_invalid_tenant_id() {
    assert_eq!(
        ThreadLifecycle::new(ThreadLifecycleCreate {
            thread_id: "t-1".into(),
            tenant_id: "\t".into(),
            initial_state: ThreadState::Open,
        }),
        Err(ChatError::InvalidTenantId)
    );
}

/// Constructor stores the supplied `thread_id` value unchanged.
#[test]
fn constructor_preserves_thread_id_value() {
    let t = open_lifecycle();
    assert_eq!(t.thread_id.value, "thread-red-1");
}

/// Constructor stores the supplied `tenant_id` value unchanged.
#[test]
fn constructor_preserves_tenant_id_value() {
    let t = open_lifecycle();
    assert_eq!(t.tenant_id.value, "tenant-red-1");
}

/// Constructor stores the supplied `initial_state`.
#[test]
fn constructor_preserves_initial_state() {
    let t = open_lifecycle();
    assert_eq!(t.state.value, ThreadState::Open);
}

// ---------------------------------------------------------------------------
// Subtask 2 — transition() state-machine: legal paths
// ---------------------------------------------------------------------------

/// `Open -> Locked` is a legal transition.
#[test]
fn transition_open_to_locked_is_legal() {
    let t = open_lifecycle().transition(ThreadState::Locked).unwrap();
    assert_eq!(t.state.value, ThreadState::Locked);
}

/// `Open -> Resolved` is a legal transition.
#[test]
fn transition_open_to_resolved_is_legal() {
    let t = open_lifecycle().transition(ThreadState::Resolved).unwrap();
    assert_eq!(t.state.value, ThreadState::Resolved);
}

/// `Open -> Archived` is a legal transition.
#[test]
fn transition_open_to_archived_is_legal() {
    let t = open_lifecycle().transition(ThreadState::Archived).unwrap();
    assert_eq!(t.state.value, ThreadState::Archived);
}

/// `Locked -> Open` is a legal transition.
#[test]
fn transition_locked_to_open_is_legal() {
    let t = lifecycle_in(ThreadState::Locked)
        .transition(ThreadState::Open)
        .unwrap();
    assert_eq!(t.state.value, ThreadState::Open);
}

/// `Locked -> Resolved` is a legal transition.
#[test]
fn transition_locked_to_resolved_is_legal() {
    let t = lifecycle_in(ThreadState::Locked)
        .transition(ThreadState::Resolved)
        .unwrap();
    assert_eq!(t.state.value, ThreadState::Resolved);
}

/// `Locked -> Archived` is a legal transition.
#[test]
fn transition_locked_to_archived_is_legal() {
    let t = lifecycle_in(ThreadState::Locked)
        .transition(ThreadState::Archived)
        .unwrap();
    assert_eq!(t.state.value, ThreadState::Archived);
}

/// `Resolved -> Open` is a legal transition.
#[test]
fn transition_resolved_to_open_is_legal() {
    let t = lifecycle_in(ThreadState::Resolved)
        .transition(ThreadState::Open)
        .unwrap();
    assert_eq!(t.state.value, ThreadState::Open);
}

/// `Resolved -> Archived` is a legal transition.
#[test]
fn transition_resolved_to_archived_is_legal() {
    let t = lifecycle_in(ThreadState::Resolved)
        .transition(ThreadState::Archived)
        .unwrap();
    assert_eq!(t.state.value, ThreadState::Archived);
}

// ---------------------------------------------------------------------------
// Subtask 2 — transition() state-machine: illegal / terminal paths
// ---------------------------------------------------------------------------

/// `Archived -> Open` is illegal; `Archived` is terminal.
#[test]
fn transition_archived_to_open_is_rejected() {
    assert_eq!(
        lifecycle_in(ThreadState::Archived).transition(ThreadState::Open),
        Err(ChatError::InvalidThreadStateTransition)
    );
}

/// `Archived -> Locked` is illegal; `Archived` is terminal.
#[test]
fn transition_archived_to_locked_is_rejected() {
    assert_eq!(
        lifecycle_in(ThreadState::Archived).transition(ThreadState::Locked),
        Err(ChatError::InvalidThreadStateTransition)
    );
}

/// `Archived -> Resolved` is illegal; `Archived` is terminal.
#[test]
fn transition_archived_to_resolved_is_rejected() {
    assert_eq!(
        lifecycle_in(ThreadState::Archived).transition(ThreadState::Resolved),
        Err(ChatError::InvalidThreadStateTransition)
    );
}

/// `Archived -> Archived` (self-loop on terminal state) is illegal.
#[test]
fn transition_archived_to_archived_self_loop_is_rejected() {
    assert_eq!(
        lifecycle_in(ThreadState::Archived).transition(ThreadState::Archived),
        Err(ChatError::InvalidThreadStateTransition)
    );
}

/// `Open -> Open` same-state no-op is illegal.
#[test]
fn transition_open_to_open_same_state_is_rejected() {
    assert_eq!(
        open_lifecycle().transition(ThreadState::Open),
        Err(ChatError::InvalidThreadStateTransition)
    );
}

/// `Resolved -> Locked` is an illegal transition (Resolved cannot move to Locked).
#[test]
fn transition_resolved_to_locked_is_rejected() {
    assert_eq!(
        lifecycle_in(ThreadState::Resolved).transition(ThreadState::Locked),
        Err(ChatError::InvalidThreadStateTransition)
    );
}

/// `Resolved -> Resolved` same-state no-op is illegal.
#[test]
fn transition_resolved_to_resolved_same_state_is_rejected() {
    assert_eq!(
        lifecycle_in(ThreadState::Resolved).transition(ThreadState::Resolved),
        Err(ChatError::InvalidThreadStateTransition)
    );
}

/// `Locked -> Locked` same-state no-op is illegal.
#[test]
fn transition_locked_to_locked_same_state_is_rejected() {
    assert_eq!(
        lifecycle_in(ThreadState::Locked).transition(ThreadState::Locked),
        Err(ChatError::InvalidThreadStateTransition)
    );
}

/// `transition()` returns a *new* `ThreadLifecycle`; the source is not mutated
/// (confirmed by checking the original still holds its previous state).
#[test]
fn transition_returns_new_value_leaving_source_unchanged() {
    let original = open_lifecycle();
    let next = original.transition(ThreadState::Locked).unwrap();
    assert_eq!(original.state.value, ThreadState::Open, "source unchanged");
    assert_eq!(next.state.value, ThreadState::Locked, "new value updated");
}

/// The `thread_id` and `tenant_id` are preserved across a transition.
#[test]
fn transition_preserves_thread_and_tenant_id() {
    let t = open_lifecycle().transition(ThreadState::Locked).unwrap();
    assert_eq!(t.thread_id.value, "thread-red-1");
    assert_eq!(t.tenant_id.value, "tenant-red-1");
}

// ---------------------------------------------------------------------------
// Subtask 3 — ThreadSubscription pillar-isolation invariants
// ---------------------------------------------------------------------------

/// A same-pillar Work subscription is accepted and defaults to `Follow` mode.
#[test]
fn same_pillar_work_subscription_defaults_to_follow() {
    let sub = work_subscription();
    assert_eq!(sub.mode.value, ThreadSubscriptionMode::Follow);
}

/// A same-pillar Personal subscription is accepted.
#[test]
fn same_pillar_personal_subscription_is_accepted() {
    let sub = personal_subscription();
    assert_eq!(sub.mode.value, ThreadSubscriptionMode::Follow);
}

/// `participant_ref` is tagged `PII_IDENTIFYING` (mirrors `MessageGovernance`).
#[test]
fn subscription_participant_ref_is_tagged_pii_identifying() {
    use oya_data_boundary_kernel::PrivacyDataClass;
    let sub = work_subscription();
    assert_eq!(
        sub.participant_ref.data_class,
        DataClassification::Privacy(PrivacyDataClass::pii_identifying()),
        "participant_ref must be PII_IDENTIFYING"
    );
}

/// `participant_pillar` and `thread_pillar` are tagged `InternalOnly`.
#[test]
fn subscription_pillar_fields_are_tagged_internal_only() {
    let internal: DataClassification = DataClass::InternalOnly.into();
    let sub = work_subscription();
    assert_eq!(
        sub.participant_pillar.data_class, internal,
        "participant_pillar"
    );
    assert_eq!(sub.thread_pillar.data_class, internal, "thread_pillar");
}

/// `thread_id` and `tenant_id` on the subscription are tagged `InternalOnly`.
#[test]
fn subscription_thread_and_tenant_id_are_tagged_internal_only() {
    let internal: DataClassification = DataClass::InternalOnly.into();
    let sub = work_subscription();
    assert_eq!(sub.thread_id.data_class, internal, "thread_id");
    assert_eq!(sub.tenant_id.data_class, internal, "tenant_id");
}

/// Cross-pillar: Work participant on a Personal thread is denied.
#[test]
fn cross_pillar_work_participant_on_personal_thread_is_denied() {
    assert_eq!(
        ThreadSubscription::new(ThreadSubscriptionCreate {
            thread_id: "thread-1".into(),
            tenant_id: "tenant-1".into(),
            participant_ref: "user:alice@work.example.com".into(),
            participant_pillar: OwnershipPillar::Work,
            thread_pillar: OwnershipPillar::Personal,
        }),
        Err(ChatError::CrossPillarSubscriptionDenied)
    );
}

/// Cross-pillar: Personal participant on a Work thread is denied.
#[test]
fn cross_pillar_personal_participant_on_work_thread_is_denied() {
    assert_eq!(
        ThreadSubscription::new(ThreadSubscriptionCreate {
            thread_id: "thread-1".into(),
            tenant_id: "tenant-1".into(),
            participant_ref: "user:bob@personal.example.com".into(),
            participant_pillar: OwnershipPillar::Personal,
            thread_pillar: OwnershipPillar::Work,
        }),
        Err(ChatError::CrossPillarSubscriptionDenied)
    );
}

/// `follow()` returns `Follow` mode.
#[test]
fn follow_method_returns_follow_mode() {
    let sub = work_subscription();
    assert_eq!(sub.follow(), ThreadSubscriptionMode::Follow);
}

/// `mute()` returns `Mute` mode.
#[test]
fn mute_method_returns_mute_mode() {
    let sub = work_subscription();
    assert_eq!(sub.mute(), ThreadSubscriptionMode::Mute);
}

/// `with_mode(Mute)` produces a subscription in `Mute` mode.
#[test]
fn with_mode_mute_produces_muted_subscription() {
    let sub = work_subscription().with_mode(ThreadSubscriptionMode::Mute);
    assert_eq!(sub.mode.value, ThreadSubscriptionMode::Mute);
}

/// Follow → Mute → Follow round-trip is consistent.
#[test]
fn follow_mute_follow_round_trip_is_consistent() {
    let original = work_subscription(); // Follow
    let muted = original.with_mode(original.mute());
    assert_eq!(muted.mode.value, ThreadSubscriptionMode::Mute, "after mute");
    let followed = muted.with_mode(muted.follow());
    assert_eq!(
        followed.mode.value,
        ThreadSubscriptionMode::Follow,
        "after follow"
    );
}

/// Empty `thread_id` on subscription is rejected with `InvalidThreadId`.
#[test]
fn subscription_empty_thread_id_returns_invalid_thread_id() {
    assert_eq!(
        ThreadSubscription::new(ThreadSubscriptionCreate {
            thread_id: String::new(),
            tenant_id: "tenant-1".into(),
            participant_ref: "user:alice@work.example.com".into(),
            participant_pillar: OwnershipPillar::Work,
            thread_pillar: OwnershipPillar::Work,
        }),
        Err(ChatError::InvalidThreadId)
    );
}

/// Empty `tenant_id` on subscription is rejected with `InvalidTenantId`.
#[test]
fn subscription_empty_tenant_id_returns_invalid_tenant_id() {
    assert_eq!(
        ThreadSubscription::new(ThreadSubscriptionCreate {
            thread_id: "thread-1".into(),
            tenant_id: String::new(),
            participant_ref: "user:alice@work.example.com".into(),
            participant_pillar: OwnershipPillar::Work,
            thread_pillar: OwnershipPillar::Work,
        }),
        Err(ChatError::InvalidTenantId)
    );
}

/// Empty `participant_ref` on subscription is rejected with `InvalidParticipantRef`.
#[test]
fn subscription_empty_participant_ref_returns_invalid_participant_ref() {
    assert_eq!(
        ThreadSubscription::new(ThreadSubscriptionCreate {
            thread_id: "thread-1".into(),
            tenant_id: "tenant-1".into(),
            participant_ref: String::new(),
            participant_pillar: OwnershipPillar::Work,
            thread_pillar: OwnershipPillar::Work,
        }),
        Err(ChatError::InvalidParticipantRef)
    );
}

/// Whitespace-only `participant_ref` on subscription is rejected with
/// `InvalidParticipantRef`.
#[test]
fn subscription_whitespace_participant_ref_returns_invalid_participant_ref() {
    assert_eq!(
        ThreadSubscription::new(ThreadSubscriptionCreate {
            thread_id: "thread-1".into(),
            tenant_id: "tenant-1".into(),
            participant_ref: "   ".into(),
            participant_pillar: OwnershipPillar::Work,
            thread_pillar: OwnershipPillar::Work,
        }),
        Err(ChatError::InvalidParticipantRef)
    );
}

// ---------------------------------------------------------------------------
// Public re-export surface (lib.rs wiring)
// ---------------------------------------------------------------------------

/// All key types must be reachable directly from the crate root.
/// The compiler enforces visibility; this test exercises the import path.
#[test]
fn public_types_are_reachable_from_crate_root() {
    let _: ThreadState;
    let _: ThreadLifecycle;
    let _: ThreadLifecycleCreate;
    let _: ThreadSubscription;
    let _: ThreadSubscriptionCreate;
    let _: ThreadSubscriptionMode;
}
