//! Thread-lifecycle state machine kernel.
//!
//! Typed `ThreadState` enum, `ThreadLifecycle` struct with validated
//! constructors and immutable-where-required transition logic, and
//! per-participant `ThreadSubscription` follow/mute invariants with
//! `OwnershipPillar` cross-pillar isolation consistent with the
//! `PresenceState::CrossPillarPresenceDenied` pattern in `governance.rs`.
//!
//! Pure domain logic — no I/O, no REST/usecase dependencies.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

use crate::{ChatError, OwnershipPillar};

// ---------------------------------------------------------------------------
// ThreadState
// ---------------------------------------------------------------------------

/// State of a chat thread within the workspace messenger.
///
/// `Archived` is terminal — no outbound transitions are permitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ThreadState {
    Open,
    Locked,
    Resolved,
    Archived,
}

// ---------------------------------------------------------------------------
// ThreadLifecycle
// ---------------------------------------------------------------------------

/// Input record for constructing a `ThreadLifecycle` (plain fields, no `Classified`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThreadLifecycleCreate {
    pub thread_id: String,          // data_class: INTERNAL_ONLY
    pub tenant_id: String,          // data_class: INTERNAL_ONLY
    pub initial_state: ThreadState, // data_class: INTERNAL_ONLY
}

/// Canonical thread-lifecycle record.  All fields are `Classified` per
/// ADR-0083 data-boundary tagging.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThreadLifecycle {
    pub thread_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub state: Classified<ThreadState>, // data_class: INTERNAL_ONLY
}

impl ThreadLifecycle {
    /// Construct a validated `ThreadLifecycle`.
    ///
    /// Returns `Err(ChatError::InvalidThreadId)` when `thread_id` is empty or
    /// whitespace-only, and `Err(ChatError::InvalidTenantId)` for the same
    /// violation on `tenant_id`.
    pub fn new(input: ThreadLifecycleCreate) -> Result<Self, ChatError> {
        thread_lifecycle_validate_non_empty(&input.thread_id, ChatError::InvalidThreadId)?;
        thread_lifecycle_validate_non_empty(&input.tenant_id, ChatError::InvalidTenantId)?;
        Ok(Self {
            thread_id: internal(input.thread_id),
            tenant_id: internal(input.tenant_id),
            state: internal(input.initial_state),
        })
    }

    /// Attempt a state transition.  Returns a new `ThreadLifecycle` with the
    /// updated `state` on success, or `Err(ChatError::InvalidThreadStateTransition)`
    /// when the transition is illegal (including same-state and any outbound
    /// transition from `Archived`).
    pub fn transition(&self, next: ThreadState) -> Result<Self, ChatError> {
        let current = self.state.value;
        let legal = match current {
            ThreadState::Open => matches!(
                next,
                ThreadState::Locked | ThreadState::Resolved | ThreadState::Archived
            ),
            ThreadState::Locked => matches!(
                next,
                ThreadState::Open | ThreadState::Resolved | ThreadState::Archived
            ),
            ThreadState::Resolved => matches!(next, ThreadState::Open | ThreadState::Archived),
            // Archived is terminal — no outbound transitions
            ThreadState::Archived => false,
        };
        if !legal {
            return Err(ChatError::InvalidThreadStateTransition);
        }
        Ok(Self {
            thread_id: internal(self.thread_id.value.clone()),
            tenant_id: internal(self.tenant_id.value.clone()),
            state: internal(next),
        })
    }
}

// ---------------------------------------------------------------------------
// ThreadSubscription
// ---------------------------------------------------------------------------

/// Subscription mode for a thread participant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThreadSubscriptionMode {
    Follow,
    Mute,
}

/// Input record for constructing a `ThreadSubscription`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThreadSubscriptionCreate {
    pub thread_id: String,                   // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub participant_ref: String,             // data_class: PII_IDENTIFYING
    pub participant_pillar: OwnershipPillar, // data_class: INTERNAL_ONLY
    pub thread_pillar: OwnershipPillar,      // data_class: INTERNAL_ONLY
}

/// Canonical thread-subscription record with follow/mute invariants.
///
/// A participant from a different `OwnershipPillar` than the thread's pillar
/// is rejected at construction time with
/// `ChatError::CrossPillarSubscriptionDenied`, mirroring the
/// `PresenceState::CrossPillarPresenceDenied` guard in `governance.rs`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThreadSubscription {
    pub thread_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub participant_ref: Classified<String>, // data_class: PII_IDENTIFYING
    pub participant_pillar: Classified<OwnershipPillar>, // data_class: INTERNAL_ONLY
    pub thread_pillar: Classified<OwnershipPillar>, // data_class: INTERNAL_ONLY
    pub mode: Classified<ThreadSubscriptionMode>, // data_class: INTERNAL_ONLY
}

impl ThreadSubscription {
    /// Construct a validated `ThreadSubscription` in `Follow` mode.
    ///
    /// Fails with:
    /// - `ChatError::InvalidThreadId` — empty / whitespace thread_id
    /// - `ChatError::InvalidTenantId` — empty / whitespace tenant_id
    /// - `ChatError::InvalidParticipantRef` — empty / whitespace participant_ref
    /// - `ChatError::CrossPillarSubscriptionDenied` — pillar mismatch
    pub fn new(input: ThreadSubscriptionCreate) -> Result<Self, ChatError> {
        thread_lifecycle_validate_non_empty(&input.thread_id, ChatError::InvalidThreadId)?;
        thread_lifecycle_validate_non_empty(&input.tenant_id, ChatError::InvalidTenantId)?;
        thread_lifecycle_validate_non_empty(
            &input.participant_ref,
            ChatError::InvalidParticipantRef,
        )?;
        if input.participant_pillar != input.thread_pillar {
            return Err(ChatError::CrossPillarSubscriptionDenied);
        }
        Ok(Self {
            thread_id: internal(input.thread_id),
            tenant_id: internal(input.tenant_id),
            participant_ref: Classified::new(
                input.participant_ref,
                PrivacyDataClass::pii_identifying(),
            ),
            participant_pillar: internal(input.participant_pillar),
            thread_pillar: internal(input.thread_pillar),
            mode: internal(ThreadSubscriptionMode::Follow),
        })
    }

    /// Return a new `ThreadSubscription` with mode set to `Follow`.
    pub fn follow(&self) -> ThreadSubscriptionMode {
        ThreadSubscriptionMode::Follow
    }

    /// Return a new `ThreadSubscription` with mode set to `Mute`.
    pub fn mute(&self) -> ThreadSubscriptionMode {
        ThreadSubscriptionMode::Mute
    }

    /// Return a copy of this subscription with the given mode applied.
    pub fn with_mode(&self, mode: ThreadSubscriptionMode) -> Self {
        Self {
            thread_id: internal(self.thread_id.value.clone()),
            tenant_id: internal(self.tenant_id.value.clone()),
            participant_ref: Classified::new(
                self.participant_ref.value.clone(),
                PrivacyDataClass::pii_identifying(),
            ),
            participant_pillar: internal(self.participant_pillar.value),
            thread_pillar: internal(self.thread_pillar.value),
            mode: internal(mode),
        }
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn thread_lifecycle_validate_non_empty(value: &str, error: ChatError) -> Result<(), ChatError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use data_boundary_kernel::{DataClass, DataClassification};

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn open_thread() -> ThreadLifecycle {
        ThreadLifecycle::new(ThreadLifecycleCreate {
            thread_id: "thread-1".into(),
            tenant_id: "tenant-1".into(),
            initial_state: ThreadState::Open,
        })
        .unwrap()
    }

    fn thread_in(state: ThreadState) -> ThreadLifecycle {
        let mut t = open_thread();
        if state != ThreadState::Open {
            t = t.transition(state).unwrap();
        }
        t
    }

    fn work_sub() -> ThreadSubscription {
        ThreadSubscription::new(ThreadSubscriptionCreate {
            thread_id: "thread-1".into(),
            tenant_id: "tenant-1".into(),
            participant_ref: "user:alice@example.com".into(),
            participant_pillar: OwnershipPillar::Work,
            thread_pillar: OwnershipPillar::Work,
        })
        .unwrap()
    }

    fn personal_sub() -> ThreadSubscription {
        ThreadSubscription::new(ThreadSubscriptionCreate {
            thread_id: "thread-2".into(),
            tenant_id: "tenant-2".into(),
            participant_ref: "user:bob@personal.com".into(),
            participant_pillar: OwnershipPillar::Personal,
            thread_pillar: OwnershipPillar::Personal,
        })
        .unwrap()
    }

    // -----------------------------------------------------------------------
    // Subtask 1 — constructor invariants
    // -----------------------------------------------------------------------

    #[test]
    fn constructor_valid_open() {
        let t = open_thread();
        assert_eq!(t.state.value, ThreadState::Open);
        assert_eq!(t.thread_id.data_class, DataClass::InternalOnly.into());
        assert_eq!(t.tenant_id.data_class, DataClass::InternalOnly.into());
        assert_eq!(t.state.data_class, DataClass::InternalOnly.into());
    }

    #[test]
    fn empty_thread_id_rejected() {
        assert_eq!(
            ThreadLifecycle::new(ThreadLifecycleCreate {
                thread_id: String::new(),
                tenant_id: "tenant-1".into(),
                initial_state: ThreadState::Open,
            }),
            Err(ChatError::InvalidThreadId)
        );
    }

    #[test]
    fn whitespace_thread_id_rejected() {
        assert_eq!(
            ThreadLifecycle::new(ThreadLifecycleCreate {
                thread_id: "   ".into(),
                tenant_id: "tenant-1".into(),
                initial_state: ThreadState::Open,
            }),
            Err(ChatError::InvalidThreadId)
        );
    }

    #[test]
    fn empty_tenant_id_rejected() {
        assert_eq!(
            ThreadLifecycle::new(ThreadLifecycleCreate {
                thread_id: "thread-1".into(),
                tenant_id: String::new(),
                initial_state: ThreadState::Open,
            }),
            Err(ChatError::InvalidTenantId)
        );
    }

    // -----------------------------------------------------------------------
    // Subtask 2 — transition table
    // -----------------------------------------------------------------------

    // --- legal transitions from Open ---

    #[test]
    fn open_to_locked_legal() {
        let t = open_thread().transition(ThreadState::Locked).unwrap();
        assert_eq!(t.state.value, ThreadState::Locked);
    }

    #[test]
    fn open_to_resolved_legal() {
        let t = open_thread().transition(ThreadState::Resolved).unwrap();
        assert_eq!(t.state.value, ThreadState::Resolved);
    }

    #[test]
    fn open_to_archived_legal() {
        let t = open_thread().transition(ThreadState::Archived).unwrap();
        assert_eq!(t.state.value, ThreadState::Archived);
    }

    // --- legal transitions from Locked ---

    #[test]
    fn locked_to_open_legal() {
        let t = thread_in(ThreadState::Locked)
            .transition(ThreadState::Open)
            .unwrap();
        assert_eq!(t.state.value, ThreadState::Open);
    }

    #[test]
    fn locked_to_resolved_legal() {
        let t = thread_in(ThreadState::Locked)
            .transition(ThreadState::Resolved)
            .unwrap();
        assert_eq!(t.state.value, ThreadState::Resolved);
    }

    #[test]
    fn locked_to_archived_legal() {
        let t = thread_in(ThreadState::Locked)
            .transition(ThreadState::Archived)
            .unwrap();
        assert_eq!(t.state.value, ThreadState::Archived);
    }

    // --- legal transitions from Resolved ---

    #[test]
    fn resolved_to_open_legal() {
        let t = thread_in(ThreadState::Resolved)
            .transition(ThreadState::Open)
            .unwrap();
        assert_eq!(t.state.value, ThreadState::Open);
    }

    #[test]
    fn resolved_to_archived_legal() {
        let t = thread_in(ThreadState::Resolved)
            .transition(ThreadState::Archived)
            .unwrap();
        assert_eq!(t.state.value, ThreadState::Archived);
    }

    // --- Archived is terminal ---

    #[test]
    fn archived_terminal_open() {
        assert_eq!(
            thread_in(ThreadState::Archived).transition(ThreadState::Open),
            Err(ChatError::InvalidThreadStateTransition)
        );
    }

    #[test]
    fn archived_terminal_locked() {
        assert_eq!(
            thread_in(ThreadState::Archived).transition(ThreadState::Locked),
            Err(ChatError::InvalidThreadStateTransition)
        );
    }

    #[test]
    fn archived_terminal_resolved() {
        assert_eq!(
            thread_in(ThreadState::Archived).transition(ThreadState::Resolved),
            Err(ChatError::InvalidThreadStateTransition)
        );
    }

    // --- same-state (no-op) always rejected ---

    #[test]
    fn same_state_open_rejected() {
        assert_eq!(
            open_thread().transition(ThreadState::Open),
            Err(ChatError::InvalidThreadStateTransition)
        );
    }

    #[test]
    fn same_state_archived_rejected() {
        assert_eq!(
            thread_in(ThreadState::Archived).transition(ThreadState::Archived),
            Err(ChatError::InvalidThreadStateTransition)
        );
    }

    // --- specific illegal transitions ---

    #[test]
    fn resolved_to_locked_rejected() {
        assert_eq!(
            thread_in(ThreadState::Resolved).transition(ThreadState::Locked),
            Err(ChatError::InvalidThreadStateTransition)
        );
    }

    // -----------------------------------------------------------------------
    // Subtask 3 — subscription pillar invariants
    // -----------------------------------------------------------------------

    #[test]
    fn same_pillar_work_follow_succeeds() {
        let sub = work_sub();
        assert_eq!(sub.mode.value, ThreadSubscriptionMode::Follow);
        assert_eq!(
            sub.participant_ref.data_class,
            DataClassification::Privacy(PrivacyDataClass::pii_identifying())
        );
    }

    #[test]
    fn same_pillar_personal_mute_succeeds() {
        let sub = personal_sub().with_mode(ThreadSubscriptionMode::Mute);
        assert_eq!(sub.mode.value, ThreadSubscriptionMode::Mute);
    }

    #[test]
    fn cross_pillar_work_on_personal_denied() {
        assert_eq!(
            ThreadSubscription::new(ThreadSubscriptionCreate {
                thread_id: "thread-1".into(),
                tenant_id: "tenant-1".into(),
                participant_ref: "user:alice@example.com".into(),
                participant_pillar: OwnershipPillar::Work,
                thread_pillar: OwnershipPillar::Personal,
            }),
            Err(ChatError::CrossPillarSubscriptionDenied)
        );
    }

    #[test]
    fn cross_pillar_personal_on_work_denied() {
        assert_eq!(
            ThreadSubscription::new(ThreadSubscriptionCreate {
                thread_id: "thread-1".into(),
                tenant_id: "tenant-1".into(),
                participant_ref: "user:bob@personal.com".into(),
                participant_pillar: OwnershipPillar::Personal,
                thread_pillar: OwnershipPillar::Work,
            }),
            Err(ChatError::CrossPillarSubscriptionDenied)
        );
    }

    #[test]
    fn follow_mute_round_trip() {
        let sub = work_sub(); // starts as Follow
        assert_eq!(sub.follow(), ThreadSubscriptionMode::Follow);

        let muted = sub.with_mode(sub.mute());
        assert_eq!(muted.mode.value, ThreadSubscriptionMode::Mute);

        let followed = muted.with_mode(muted.follow());
        assert_eq!(followed.mode.value, ThreadSubscriptionMode::Follow);
    }

    #[test]
    fn invalid_thread_id_rejected() {
        assert_eq!(
            ThreadSubscription::new(ThreadSubscriptionCreate {
                thread_id: String::new(),
                tenant_id: "tenant-1".into(),
                participant_ref: "user:alice@example.com".into(),
                participant_pillar: OwnershipPillar::Work,
                thread_pillar: OwnershipPillar::Work,
            }),
            Err(ChatError::InvalidThreadId)
        );
    }

    #[test]
    fn invalid_participant_ref_rejected() {
        assert_eq!(
            ThreadSubscription::new(ThreadSubscriptionCreate {
                thread_id: "thread-1".into(),
                tenant_id: "tenant-1".into(),
                participant_ref: "  ".into(),
                participant_pillar: OwnershipPillar::Work,
                thread_pillar: OwnershipPillar::Work,
            }),
            Err(ChatError::InvalidParticipantRef)
        );
    }
}
