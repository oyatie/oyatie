//! Deterministic lock-store adapter for Oya VCS.
//!
//! The adapter boundary is deliberately std-only.  Production providers can map
//! the same [`LockStorePort`] operations onto conditional object writes,
//! advisory locks, Kubernetes leases, or an event bus without exposing git/gh to
//! agents.  Tests use the same port with local and remote-fake backends.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use oya_foundry_vcs_kernel::{Claim, ClaimCompatibility, ClaimState, claim_compatibility};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LockStoreBackend {
    Local,
    RemoteFake,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LogicalTime(pub u64);

impl LogicalTime {
    pub fn plus(self, seconds: u64) -> Self {
        Self(self.0.saturating_add(seconds))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimRequest {
    pub claim: Claim, // data_class: INTERNAL_ONLY
}

impl ClaimRequest {
    pub fn new(claim: Claim) -> Self {
        Self { claim }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaleRecoveryEvidence {
    pub evidence_ref: String,             // data_class: INTERNAL_ONLY
    pub observed_owner: String,           // data_class: INTERNAL_ONLY
    pub observed_claim_id: String,        // data_class: INTERNAL_ONLY
    pub observed_expired_at: LogicalTime, // data_class: INTERNAL_ONLY
}

impl StaleRecoveryEvidence {
    pub fn new(
        evidence_ref: impl Into<String>,
        observed_owner: impl Into<String>,
        observed_claim_id: impl Into<String>,
        observed_expired_at: LogicalTime,
    ) -> Result<Self, LockStoreError> {
        let evidence_ref =
            normalize_non_empty(evidence_ref.into(), LockStoreError::MissingEvidence)?;
        if !evidence_ref.starts_with("evidence/") {
            return Err(LockStoreError::MissingEvidence);
        }
        Ok(Self {
            evidence_ref,
            observed_owner: normalize_non_empty(
                observed_owner.into(),
                LockStoreError::InvalidAgent,
            )?,
            observed_claim_id: normalize_non_empty(
                observed_claim_id.into(),
                LockStoreError::InvalidClaim,
            )?,
            observed_expired_at,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimOutcome {
    Granted {
        claim_id: String,
        expires_at: LogicalTime,
    },
    Queued {
        claim_id: String,
        position: usize,
    },
    AlreadyGranted {
        claim_id: String,
        expires_at: LogicalTime,
    },
    AlreadyQueued {
        claim_id: String,
        position: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeartbeatReceipt {
    pub claim_id: String,                 // data_class: INTERNAL_ONLY
    pub owner: String,                    // data_class: INTERNAL_ONLY
    pub previous_expires_at: LogicalTime, // data_class: INTERNAL_ONLY
    pub expires_at: LogicalTime,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseReceipt {
    pub claim_id: String, // data_class: INTERNAL_ONLY
    pub owner: String,    // data_class: INTERNAL_ONLY
    pub released: bool,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LockEventKind {
    ClaimGranted,
    ClaimQueued,
    Heartbeat,
    LeaseReleased,
    StaleRecovered,
    QueuePromoted,
    DuplicateCollapsed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockEvent {
    pub sequence: u64,             // data_class: INTERNAL_ONLY
    pub kind: LockEventKind,       // data_class: INTERNAL_ONLY
    pub claim_id: String,          // data_class: INTERNAL_ONLY
    pub owner: String,             // data_class: INTERNAL_ONLY
    pub idempotency_key: String,   // data_class: INTERNAL_ONLY
    pub backend: LockStoreBackend, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueueProjection {
    pub active_claims: Vec<String>, // data_class: INTERNAL_ONLY
    pub queued_claims: Vec<String>, // data_class: INTERNAL_ONLY
    pub backend: LockStoreBackend,  // data_class: INTERNAL_ONLY
}

pub trait LockStorePort {
    fn claim(
        &mut self,
        request: ClaimRequest,
        now: LogicalTime,
        stale_evidence: Option<StaleRecoveryEvidence>,
    ) -> Result<ClaimOutcome, LockStoreError>;

    fn release(
        &mut self,
        agent_id: &str,
        claim_id: &str,
        now: LogicalTime,
    ) -> Result<ReleaseReceipt, LockStoreError>;

    fn heartbeat(
        &mut self,
        agent_id: &str,
        claim_id: &str,
        now: LogicalTime,
    ) -> Result<HeartbeatReceipt, LockStoreError>;

    fn recover_stale(
        &mut self,
        requester_agent_id: &str,
        stale_claim_id: &str,
        evidence: StaleRecoveryEvidence,
        now: LogicalTime,
    ) -> Result<(), LockStoreError>;

    fn watch_since(&self, after_sequence: u64) -> Vec<LockEvent>;

    fn queue_projection(&self) -> QueueProjection;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LockStoreError {
    InvalidAgent,
    InvalidClaim,
    InvalidClaimState,
    MissingEvidence,
    StaleRecoveryEvidenceMismatch,
    StaleRecoveryRequiresExpiredLease,
    HeartbeatAfterLeaseExpired,
    NonOwnerReleaseRejected,
    NonOwnerHeartbeatRejected,
    ClaimNotFound,
}

impl fmt::Display for LockStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for LockStoreError {}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LeaseRecord {
    claim: Claim,
    owner: String,
    acquired_at: LogicalTime,
    expires_at: LogicalTime,
}

#[derive(Clone, Debug)]
pub struct DeterministicLockStore {
    backend: LockStoreBackend,
    active: BTreeMap<String, LeaseRecord>,
    released: BTreeMap<String, String>,
    queue: Vec<Claim>,
    events: Vec<LockEvent>,
    event_keys: BTreeSet<String>,
    next_sequence: u64,
}

impl DeterministicLockStore {
    pub fn local() -> Self {
        Self::new(LockStoreBackend::Local)
    }

    pub fn remote_fake() -> Self {
        Self::new(LockStoreBackend::RemoteFake)
    }

    pub fn new(backend: LockStoreBackend) -> Self {
        Self {
            backend,
            active: BTreeMap::new(),
            released: BTreeMap::new(),
            queue: Vec::new(),
            events: Vec::new(),
            event_keys: BTreeSet::new(),
            next_sequence: 1,
        }
    }

    /// Injects a provider event through the same duplicate-collapse path used by
    /// the adapter. This models remote watch replay where providers can redeliver
    /// the same conditional-write notification more than once.
    pub fn append_provider_event(
        &mut self,
        kind: LockEventKind,
        claim_id: impl Into<String>,
        owner: impl Into<String>,
        idempotency_key: impl Into<String>,
    ) -> Result<(), LockStoreError> {
        let claim_id = normalize_non_empty(claim_id.into(), LockStoreError::InvalidClaim)?;
        let owner = normalize_non_empty(owner.into(), LockStoreError::InvalidAgent)?;
        let key = normalize_non_empty(idempotency_key.into(), LockStoreError::InvalidClaim)?;
        self.emit(kind, claim_id, owner, key);
        Ok(())
    }

    fn grant(&mut self, claim: Claim, now: LogicalTime) -> Result<ClaimOutcome, LockStoreError> {
        let working = ensure_working(claim)?;
        let claim_id = working.id.clone();
        let owner = working.agent_id.clone();
        let expires_at = now.plus(working.ttl_seconds);
        self.active.insert(
            claim_id.clone(),
            LeaseRecord {
                claim: working,
                owner: owner.clone(),
                acquired_at: now,
                expires_at,
            },
        );
        self.emit(
            LockEventKind::ClaimGranted,
            claim_id.clone(),
            owner,
            format!("grant:{claim_id}"),
        );
        Ok(ClaimOutcome::Granted {
            claim_id,
            expires_at,
        })
    }

    fn enqueue(&mut self, claim: Claim) -> ClaimOutcome {
        let claim_id = claim.id.clone();
        let owner = claim.agent_id.clone();
        if let Some(position) = self.queue_position(&claim_id) {
            return ClaimOutcome::AlreadyQueued { claim_id, position };
        }
        self.queue.push(claim);
        let position = self.queue.len();
        self.emit(
            LockEventKind::ClaimQueued,
            claim_id.clone(),
            owner,
            format!("queue:{claim_id}"),
        );
        ClaimOutcome::Queued { claim_id, position }
    }

    fn conflicts(&self, claim: &Claim, now: LogicalTime) -> Vec<&LeaseRecord> {
        self.active
            .values()
            .filter(|lease| lease.expires_at.0 > now.0)
            .filter(|lease| {
                claim_compatibility(&lease.claim, claim) == ClaimCompatibility::Conflict
            })
            .collect()
    }

    fn expired_conflicts(&self, claim: &Claim, now: LogicalTime) -> Vec<&LeaseRecord> {
        self.active
            .values()
            .filter(|lease| lease.expires_at.0 <= now.0)
            .filter(|lease| {
                claim_compatibility(&lease.claim, claim) == ClaimCompatibility::Conflict
            })
            .collect()
    }

    fn validate_stale_evidence(
        &self,
        lease: &LeaseRecord,
        evidence: &StaleRecoveryEvidence,
        now: LogicalTime,
    ) -> Result<(), LockStoreError> {
        if lease.expires_at.0 > now.0 {
            return Err(LockStoreError::StaleRecoveryRequiresExpiredLease);
        }
        if evidence.observed_claim_id != lease.claim.id
            || evidence.observed_owner != lease.owner
            || evidence.observed_expired_at != lease.expires_at
        {
            return Err(LockStoreError::StaleRecoveryEvidenceMismatch);
        }
        Ok(())
    }

    fn remove_stale_with_evidence(
        &mut self,
        stale_claim_id: &str,
        evidence: &StaleRecoveryEvidence,
        now: LogicalTime,
    ) -> Result<(), LockStoreError> {
        let lease = self
            .active
            .get(stale_claim_id)
            .ok_or(LockStoreError::ClaimNotFound)?;
        self.validate_stale_evidence(lease, evidence, now)?;
        let removed = self
            .active
            .remove(stale_claim_id)
            .ok_or(LockStoreError::ClaimNotFound)?;
        self.emit(
            LockEventKind::StaleRecovered,
            removed.claim.id,
            removed.owner,
            format!("recover:{stale_claim_id}:{}", evidence.evidence_ref),
        );
        Ok(())
    }

    fn promote_queued(&mut self, now: LogicalTime) -> Result<(), LockStoreError> {
        let mut index = 0;
        while index < self.queue.len() {
            let queued = self.queue[index].clone();
            if self.conflicts(&queued, now).is_empty()
                && self.expired_conflicts(&queued, now).is_empty()
            {
                let claim = self.queue.remove(index);
                let claim_id = claim.id.clone();
                let owner = claim.agent_id.clone();
                self.emit(
                    LockEventKind::QueuePromoted,
                    claim_id.clone(),
                    owner,
                    format!("promote:{claim_id}"),
                );
                self.grant(claim, now)?;
            } else {
                index += 1;
            }
        }
        Ok(())
    }

    fn active_for(&self, claim_id: &str) -> Option<&LeaseRecord> {
        self.active.get(claim_id)
    }

    fn queue_position(&self, claim_id: &str) -> Option<usize> {
        self.queue
            .iter()
            .position(|claim| claim.id == claim_id)
            .map(|index| index + 1)
    }

    fn emit(
        &mut self,
        kind: LockEventKind,
        claim_id: String,
        owner: String,
        idempotency_key: String,
    ) {
        if !self.event_keys.insert(idempotency_key.clone()) {
            let collapse_key = format!("duplicate-collapsed:{idempotency_key}");
            if !self.event_keys.insert(collapse_key.clone()) {
                return;
            }
            self.push_event(
                LockEventKind::DuplicateCollapsed,
                claim_id,
                owner,
                collapse_key,
            );
            return;
        }
        self.push_event(kind, claim_id, owner, idempotency_key);
    }

    fn push_event(
        &mut self,
        kind: LockEventKind,
        claim_id: String,
        owner: String,
        idempotency_key: String,
    ) {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.events.push(LockEvent {
            sequence,
            kind,
            claim_id,
            owner,
            idempotency_key,
            backend: self.backend,
        });
    }
}

impl LockStorePort for DeterministicLockStore {
    fn claim(
        &mut self,
        request: ClaimRequest,
        now: LogicalTime,
        stale_evidence: Option<StaleRecoveryEvidence>,
    ) -> Result<ClaimOutcome, LockStoreError> {
        if let Some(lease) = self.active_for(&request.claim.id)
            && lease.owner == request.claim.agent_id
        {
            return Ok(ClaimOutcome::AlreadyGranted {
                claim_id: lease.claim.id.clone(),
                expires_at: lease.expires_at,
            });
        }
        if let Some(position) = self.queue_position(&request.claim.id) {
            return Ok(ClaimOutcome::AlreadyQueued {
                claim_id: request.claim.id,
                position,
            });
        }

        let active_conflicts = self.conflicts(&request.claim, now);
        if !active_conflicts.is_empty() {
            return Ok(self.enqueue(request.claim));
        }

        let expired_conflict_ids: Vec<String> = self
            .expired_conflicts(&request.claim, now)
            .iter()
            .map(|lease| lease.claim.id.clone())
            .collect();
        if !expired_conflict_ids.is_empty() {
            let evidence = match stale_evidence {
                Some(evidence) => evidence,
                None => return Ok(self.enqueue(request.claim)),
            };
            for stale_claim_id in expired_conflict_ids {
                self.remove_stale_with_evidence(&stale_claim_id, &evidence, now)?;
            }
        }

        self.grant(request.claim, now)
    }

    fn release(
        &mut self,
        agent_id: &str,
        claim_id: &str,
        now: LogicalTime,
    ) -> Result<ReleaseReceipt, LockStoreError> {
        let agent_id = normalize_non_empty(agent_id.to_string(), LockStoreError::InvalidAgent)?;
        let claim_id = normalize_non_empty(claim_id.to_string(), LockStoreError::InvalidClaim)?;
        let Some(lease) = self.active.get(&claim_id) else {
            if let Some(owner) = self.released.get(&claim_id) {
                if owner == &agent_id {
                    return Ok(ReleaseReceipt {
                        claim_id,
                        owner: agent_id,
                        released: false,
                    });
                }
                return Err(LockStoreError::NonOwnerReleaseRejected);
            }
            return Err(LockStoreError::ClaimNotFound);
        };
        if lease.owner != agent_id {
            return Err(LockStoreError::NonOwnerReleaseRejected);
        }
        let lease = self
            .active
            .remove(&claim_id)
            .ok_or(LockStoreError::ClaimNotFound)?;
        self.released.insert(claim_id.clone(), agent_id.clone());
        self.emit(
            LockEventKind::LeaseReleased,
            claim_id.clone(),
            agent_id.clone(),
            format!("release:{claim_id}"),
        );
        self.promote_queued(now)?;
        Ok(ReleaseReceipt {
            claim_id: lease.claim.id,
            owner: agent_id,
            released: true,
        })
    }

    fn heartbeat(
        &mut self,
        agent_id: &str,
        claim_id: &str,
        now: LogicalTime,
    ) -> Result<HeartbeatReceipt, LockStoreError> {
        let agent_id = normalize_non_empty(agent_id.to_string(), LockStoreError::InvalidAgent)?;
        let claim_id = normalize_non_empty(claim_id.to_string(), LockStoreError::InvalidClaim)?;
        let lease = self
            .active
            .get_mut(&claim_id)
            .ok_or(LockStoreError::ClaimNotFound)?;
        if lease.owner != agent_id {
            return Err(LockStoreError::NonOwnerHeartbeatRejected);
        }
        if now.0 >= lease.expires_at.0 {
            return Err(LockStoreError::HeartbeatAfterLeaseExpired);
        }
        let previous_expires_at = lease.expires_at;
        lease.expires_at = now.plus(lease.claim.ttl_seconds);
        let expires_at = lease.expires_at;
        self.emit(
            LockEventKind::Heartbeat,
            claim_id.clone(),
            agent_id.clone(),
            format!("heartbeat:{claim_id}:{}", now.0),
        );
        Ok(HeartbeatReceipt {
            claim_id,
            owner: agent_id,
            previous_expires_at,
            expires_at,
        })
    }

    fn recover_stale(
        &mut self,
        requester_agent_id: &str,
        stale_claim_id: &str,
        evidence: StaleRecoveryEvidence,
        now: LogicalTime,
    ) -> Result<(), LockStoreError> {
        let requester =
            normalize_non_empty(requester_agent_id.to_string(), LockStoreError::InvalidAgent)?;
        let stale_claim_id =
            normalize_non_empty(stale_claim_id.to_string(), LockStoreError::InvalidClaim)?;
        self.remove_stale_with_evidence(&stale_claim_id, &evidence, now)?;
        self.emit(
            LockEventKind::StaleRecovered,
            stale_claim_id,
            requester,
            format!("recover-request:{}", evidence.evidence_ref),
        );
        self.promote_queued(now)
    }

    fn watch_since(&self, after_sequence: u64) -> Vec<LockEvent> {
        self.events
            .iter()
            .filter(|event| event.sequence > after_sequence)
            .cloned()
            .collect()
    }

    fn queue_projection(&self) -> QueueProjection {
        QueueProjection {
            active_claims: self.active.keys().cloned().collect(),
            queued_claims: self.queue.iter().map(|claim| claim.id.clone()).collect(),
            backend: self.backend,
        }
    }
}

fn ensure_working(claim: Claim) -> Result<Claim, LockStoreError> {
    match claim.state {
        ClaimState::Requested => claim
            .grant()
            .start_work()
            .map_err(|_| LockStoreError::InvalidClaimState),
        ClaimState::Granted => claim
            .start_work()
            .map_err(|_| LockStoreError::InvalidClaimState),
        ClaimState::Working => Ok(claim),
        _ => Err(LockStoreError::InvalidClaimState),
    }
}

fn normalize_non_empty(value: String, error: LockStoreError) -> Result<String, LockStoreError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        Err(error)
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_foundry_vcs_kernel::{ArtifactPointer, SymbolId, SymbolLanguage};

    fn symbol(name: &str) -> SymbolId {
        SymbolId::new(
            SymbolLanguage::Rust,
            ArtifactPointer::file("crates/oya-foundry-vcs-lockstore-adapter/src/lib.rs").unwrap(),
            name,
        )
        .unwrap()
    }

    fn claim(id: &str, agent: &str, symbol_name: &str, ttl: u64) -> Claim {
        Claim::new(
            id,
            agent,
            "M-CC-P00-IP-002",
            vec![symbol(symbol_name)],
            vec![],
            ttl,
        )
        .unwrap()
    }

    fn evidence(owner: &str, claim_id: &str, expired_at: LogicalTime) -> StaleRecoveryEvidence {
        StaleRecoveryEvidence::new(
            "evidence/gitops-vcs/ip-002-lockstore.json",
            owner,
            claim_id,
            expired_at,
        )
        .unwrap()
    }

    #[test]
    fn idempotent_claim_and_owner_release() {
        let mut store = DeterministicLockStore::local();
        let now = LogicalTime(10);
        let first = store
            .claim(
                ClaimRequest::new(claim("claim_alpha", "agent-a", "alpha", 30)),
                now,
                None,
            )
            .unwrap();
        let second = store
            .claim(
                ClaimRequest::new(claim("claim_alpha", "agent-a", "alpha", 30)),
                now,
                None,
            )
            .unwrap();

        assert_eq!(
            first,
            ClaimOutcome::Granted {
                claim_id: "claim_alpha".into(),
                expires_at: LogicalTime(40)
            }
        );
        assert_eq!(
            second,
            ClaimOutcome::AlreadyGranted {
                claim_id: "claim_alpha".into(),
                expires_at: LogicalTime(40)
            }
        );
        assert_eq!(store.watch_since(0).len(), 1);

        assert_eq!(
            store
                .release("agent-a", "claim_alpha", LogicalTime(11))
                .unwrap(),
            ReleaseReceipt {
                claim_id: "claim_alpha".into(),
                owner: "agent-a".into(),
                released: true
            }
        );
        assert_eq!(
            store
                .release("agent-a", "claim_alpha", LogicalTime(12))
                .unwrap(),
            ReleaseReceipt {
                claim_id: "claim_alpha".into(),
                owner: "agent-a".into(),
                released: false
            }
        );
    }

    #[test]
    fn ttl_heartbeat_extends_active_lease() {
        let mut store = DeterministicLockStore::local();
        store
            .claim(
                ClaimRequest::new(claim("claim_alpha", "agent-a", "alpha", 10)),
                LogicalTime(5),
                None,
            )
            .unwrap();

        let receipt = store
            .heartbeat("agent-a", "claim_alpha", LogicalTime(12))
            .unwrap();

        assert_eq!(receipt.previous_expires_at, LogicalTime(15));
        assert_eq!(receipt.expires_at, LogicalTime(22));
        assert_eq!(store.watch_since(1)[0].kind, LockEventKind::Heartbeat);
    }

    #[test]
    fn duplicate_event_collapse_preserves_single_provider_event() {
        let mut store = DeterministicLockStore::remote_fake();
        store
            .append_provider_event(
                LockEventKind::ClaimGranted,
                "claim_remote",
                "agent-r",
                "provider-offset-7",
            )
            .unwrap();
        store
            .append_provider_event(
                LockEventKind::ClaimGranted,
                "claim_remote",
                "agent-r",
                "provider-offset-7",
            )
            .unwrap();
        store
            .append_provider_event(
                LockEventKind::ClaimGranted,
                "claim_remote",
                "agent-r",
                "provider-offset-7",
            )
            .unwrap();

        let events = store.watch_since(0);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind, LockEventKind::ClaimGranted);
        assert_eq!(events[1].kind, LockEventKind::DuplicateCollapsed);
    }

    #[test]
    fn local_and_remote_fake_ttl_watch_replay_and_stale_recovery() {
        for mut store in [
            DeterministicLockStore::local(),
            DeterministicLockStore::remote_fake(),
        ] {
            store
                .claim(
                    ClaimRequest::new(claim("claim_alpha", "agent-a", "alpha", 5)),
                    LogicalTime(0),
                    None,
                )
                .unwrap();
            assert_eq!(store.watch_since(0)[0].kind, LockEventKind::ClaimGranted);

            let stale_evidence = evidence("agent-a", "claim_alpha", LogicalTime(5));
            store
                .recover_stale("agent-b", "claim_alpha", stale_evidence, LogicalTime(6))
                .unwrap();

            assert!(store.queue_projection().active_claims.is_empty());
            assert!(
                store
                    .watch_since(0)
                    .iter()
                    .any(|event| event.kind == LockEventKind::StaleRecovered)
            );
        }
    }

    #[test]
    fn expired_owner_cannot_heartbeat_without_stale_recovery_evidence() {
        let mut store = DeterministicLockStore::local();
        store
            .claim(
                ClaimRequest::new(claim("claim_expiring", "agent-a", "expiring", 5)),
                LogicalTime(0),
                None,
            )
            .unwrap();
        assert_eq!(
            store.heartbeat("agent-a", "claim_expiring", LogicalTime(5)),
            Err(LockStoreError::HeartbeatAfterLeaseExpired)
        );

        assert_eq!(
            store
                .claim(
                    ClaimRequest::new(claim("claim_waiting", "agent-b", "expiring", 5)),
                    LogicalTime(6),
                    None,
                )
                .unwrap(),
            ClaimOutcome::Queued {
                claim_id: "claim_waiting".into(),
                position: 1
            }
        );
        store
            .recover_stale(
                "agent-b",
                "claim_expiring",
                evidence("agent-a", "claim_expiring", LogicalTime(5)),
                LogicalTime(6),
            )
            .unwrap();
        assert_eq!(
            store.queue_projection().active_claims,
            vec!["claim_waiting".to_string()]
        );
    }

    #[test]
    fn stale_agent_expires_and_queued_agent_can_claim() {
        let mut store = DeterministicLockStore::remote_fake();
        store
            .claim(
                ClaimRequest::new(claim("claim_alpha", "agent-a", "alpha", 5)),
                LogicalTime(0),
                None,
            )
            .unwrap();
        let queued = store
            .claim(
                ClaimRequest::new(claim("claim_beta", "agent-b", "alpha", 5)),
                LogicalTime(1),
                None,
            )
            .unwrap();
        assert_eq!(
            queued,
            ClaimOutcome::Queued {
                claim_id: "claim_beta".into(),
                position: 1
            }
        );

        store
            .recover_stale(
                "agent-b",
                "claim_alpha",
                evidence("agent-a", "claim_alpha", LogicalTime(5)),
                LogicalTime(6),
            )
            .unwrap();

        assert_eq!(store.queue_projection().active_claims, vec!["claim_beta"]);
        assert!(store.queue_projection().queued_claims.is_empty());
        assert!(
            store
                .watch_since(0)
                .iter()
                .any(|event| event.kind == LockEventKind::QueuePromoted)
        );
    }

    #[test]
    fn non_owner_stale_release_is_rejected() {
        let mut store = DeterministicLockStore::local();
        store
            .claim(
                ClaimRequest::new(claim("claim_alpha", "agent-a", "alpha", 5)),
                LogicalTime(0),
                None,
            )
            .unwrap();

        assert_eq!(
            store.release("agent-b", "claim_alpha", LogicalTime(6)),
            Err(LockStoreError::NonOwnerReleaseRejected)
        );
        assert_eq!(store.queue_projection().active_claims, vec!["claim_alpha"]);
    }

    #[test]
    fn stale_recovery_without_grit_evidence_is_rejected() {
        let mut store = DeterministicLockStore::remote_fake();
        store
            .claim(
                ClaimRequest::new(claim("claim_alpha", "agent-a", "alpha", 5)),
                LogicalTime(0),
                None,
            )
            .unwrap();

        let malformed = StaleRecoveryEvidence::new(
            "tmp/local-observation.json",
            "agent-a",
            "claim_alpha",
            LogicalTime(5),
        );
        assert_eq!(malformed, Err(LockStoreError::MissingEvidence));
        assert_eq!(
            store
                .recover_stale(
                    "agent-b",
                    "claim_alpha",
                    evidence("agent-a", "claim_alpha", LogicalTime(4)),
                    LogicalTime(6),
                )
                .unwrap_err(),
            LockStoreError::StaleRecoveryEvidenceMismatch
        );
    }
}
