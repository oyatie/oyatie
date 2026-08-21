//! Cedar-gated quorum crypto-shred (story G002; ADR-0536 D-8: per-tenant
//! KEKs make destroy-the-KEK the tenant-data deletion primitive; AWS KMS
//! ScheduleKeyDeletion precedent).
//!
//! The state machine is carried by ownership, not status flags:
//!
//! - [`ScheduledKeyDeletion::schedule`] requires a Cedar PERMIT (via
//!   [`ShredAuthorizationPort`] — the G04 PDP plugs in behind this port per
//!   the cross-lane integration law) and takes the [`KekVersionChain`] by
//!   value, demoting it to decrypt-only custody for the waiting window.
//! - [`ScheduledKeyDeletion::cancel`] is the only way to get the
//!   encrypt-capable chain back.
//! - [`ScheduledKeyDeletion::execute`] consumes the deletion AND the chain;
//!   every key version zeroizes on drop. "Use after shred" is not a runtime
//!   error — it does not compile.
//!
//! Quorum: M distinct approvers, none of whom is the requester. Execution
//! before the waiting window elapses or before quorum is reached fails
//! closed and returns custody intact.

use std::collections::BTreeSet;
use std::num::NonZeroU32;

use secrets_kms_domain::KeyDestructionRequest;
use secrets_kms_domain::envelope_keys::KekId;

use crate::EnclaveError;
use crate::chain::KekVersionChain;
use crate::material::DekMaterial;
use crate::token::WrappedDek;

/// Cedar authorization decision evidence for a shred-lifecycle action.
/// Carried into the destruction proof so every shred chains to the policy
/// decision that allowed it (audit record per decision).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShredDecisionEvidence {
    /// PDP decision identifier.
    pub decision_id: String,
    /// Policy-store version the decision was evaluated against.
    pub policy_version: String,
}

/// Outcome of a shred authorization check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShredDecision {
    /// The action is permitted; evidence must be retained.
    Permit(ShredDecisionEvidence),
    /// The action is forbidden.
    Deny(ShredDecisionEvidence),
}

/// Authorization request for scheduling or cancelling a crypto-shred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShredAuthorizationRequest {
    /// Tenant whose KEK is being shredded.
    pub tenant_id: String,
    /// The KEK identifier.
    pub kek_id: KekId,
    /// Principal requesting the action.
    pub actor: String,
    /// The lifecycle action being authorized.
    pub action: ShredAction,
    /// Epoch seconds at request time.
    pub requested_at_epoch_seconds: u64,
}

/// Shred lifecycle actions a PDP can be asked about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShredAction {
    /// Schedule the deletion (start the waiting window).
    Schedule,
    /// Cancel a pending deletion (restore encrypt-capable custody).
    Cancel,
}

/// Port to the policy decision point. The G04 embedded Cedar PDP implements
/// this; the enclave kernel never links a policy engine directly (would the
/// trait change at W5 cutover? No — it models "ask the PDP, get a decision
/// with evidence").
pub trait ShredAuthorizationPort {
    /// Evaluate a shred-lifecycle authorization request.
    fn authorize(&self, request: &ShredAuthorizationRequest) -> ShredDecision;
}

/// Errors from the shred lifecycle. Every variant is fail-closed: custody is
/// returned to the caller wherever the operation did not complete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShredError {
    /// The PDP denied the action.
    NotPermitted {
        /// Evidence of the denying decision.
        evidence: ShredDecisionEvidence,
    },
    /// Fewer distinct approvals than the quorum requires.
    QuorumNotReached {
        /// Approvals currently held.
        have: u32,
        /// Approvals required.
        need: u32,
    },
    /// The mandatory waiting window has not elapsed.
    WindowNotElapsed {
        /// Earliest epoch second at which execution is allowed.
        earliest_at_epoch_seconds: u64,
    },
    /// This approver already approved.
    DuplicateApprover,
    /// The requester cannot approve their own shred.
    RequesterCannotApprove,
    /// The waiting window must be at least the floor (no instant shred).
    WindowTooShort {
        /// Minimum window length in seconds.
        floor_seconds: u64,
    },
}

impl std::fmt::Display for ShredError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotPermitted { evidence } => {
                write!(f, "shred: PDP denied (decision {})", evidence.decision_id)
            }
            Self::QuorumNotReached { have, need } => {
                write!(f, "shred: quorum not reached ({have}/{need})")
            }
            Self::WindowNotElapsed {
                earliest_at_epoch_seconds,
            } => {
                write!(
                    f,
                    "shred: waiting window runs until {earliest_at_epoch_seconds}"
                )
            }
            Self::DuplicateApprover => f.write_str("shred: approver already counted"),
            Self::RequesterCannotApprove => f.write_str("shred: requester cannot self-approve"),
            Self::WindowTooShort { floor_seconds } => {
                write!(f, "shred: waiting window below {floor_seconds}s floor")
            }
        }
    }
}

impl std::error::Error for ShredError {}

/// Minimum waiting window (AWS KMS floor: 7 days).
pub const MIN_WAITING_WINDOW_SECONDS: u64 = 7 * 24 * 60 * 60;

/// Quorum policy: how many distinct approvers (excluding the requester) must
/// approve before execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuorumPolicy {
    /// Required number of distinct approvals.
    pub required_approvals: NonZeroU32,
}

/// Decrypt-only custody of a KEK chain during the waiting window: existing
/// data stays readable (tenants can export before the shred lands) but no
/// new data can be encrypted under a key marked for destruction. There is no
/// wrap API on this type.
pub struct PendingDeletionChain {
    chain: KekVersionChain,
}

impl PendingDeletionChain {
    /// Unwrap a DEK for read-path availability during the window.
    pub fn unwrap_dek(&self, wrapped: &WrappedDek) -> Result<DekMaterial, EnclaveError> {
        self.chain.unwrap_dek(wrapped)
    }

    /// KEK identifier under pending deletion.
    pub fn kek_id(&self) -> &KekId {
        self.chain.kek_id()
    }
}

impl std::fmt::Debug for PendingDeletionChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PendingDeletionChain {{ kek_id: {}, keys: [REDACTED] }}",
            self.kek_id()
        )
    }
}

/// Evidence that a scheduled crypto-shred was CANCELLED. Cancellation keeps
/// tenant data alive past a deletion request — the insider-relevant action —
/// so it is exactly as attributable as execution: who cancelled, under which
/// policy decision, what the original schedule decision was, and which
/// approvals existed at cancel time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelEvidence {
    /// The KEK whose deletion was cancelled.
    pub kek_id: KekId,
    /// Tenant whose data remains live.
    pub tenant_id: String,
    /// Principal that cancelled the deletion.
    pub cancelled_by: String,
    /// Cedar decision that permitted the cancellation.
    pub cancel_decision: ShredDecisionEvidence,
    /// Cedar decision that had permitted the original scheduling.
    pub schedule_decision: ShredDecisionEvidence,
    /// Approvals that had accumulated before the cancellation.
    pub approvals_at_cancel: Vec<String>,
    /// When the deletion had been scheduled.
    pub scheduled_at_epoch_seconds: u64,
    /// When the deletion was cancelled.
    pub cancelled_at_epoch_seconds: u64,
}

/// Proof that a crypto-shred completed, chaining to the authorizing policy
/// decision and the quorum that approved it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShredProof {
    /// The shredded KEK.
    pub kek_id: KekId,
    /// Tenant whose data is now unrecoverable.
    pub tenant_id: String,
    /// Cedar decision that permitted scheduling.
    pub schedule_decision: ShredDecisionEvidence,
    /// Distinct approvers that formed the quorum.
    pub approvers: Vec<String>,
    /// Number of key versions destroyed.
    pub versions_destroyed: u64,
    /// When the deletion was scheduled.
    pub scheduled_at_epoch_seconds: u64,
    /// When the material was destroyed.
    pub executed_at_epoch_seconds: u64,
}

impl ShredProof {
    /// Project this proof into the domain's destruction-request shape so the
    /// existing `KmsRepo::destroy_key` lifecycle records it.
    pub fn to_destruction_request(&self, proof_ref: String) -> KeyDestructionRequest {
        KeyDestructionRequest {
            key_id: self.kek_id.value().to_owned(),
            tenant_id: self.tenant_id.clone(),
            proof_ref,
            requested_at_epoch_seconds: self.scheduled_at_epoch_seconds,
            completed_at_epoch_seconds: self.executed_at_epoch_seconds,
        }
    }
}

/// A scheduled, Cedar-permitted, quorum-gated key deletion holding the KEK
/// chain in decrypt-only custody until execution or cancellation.
pub struct ScheduledKeyDeletion {
    pending: PendingDeletionChain,
    tenant_id: String,
    requester: String,
    schedule_decision: ShredDecisionEvidence,
    approvals: BTreeSet<String>,
    quorum: QuorumPolicy,
    scheduled_at_epoch_seconds: u64,
    earliest_shred_at_epoch_seconds: u64,
}

impl ScheduledKeyDeletion {
    /// Schedule a crypto-shred. Requires a PDP PERMIT; takes the chain into
    /// decrypt-only custody and starts the waiting window. On deny, the
    /// chain is returned untouched alongside the error.
    #[allow(clippy::result_large_err)] // custody must travel back on failure
    pub fn schedule(
        chain: KekVersionChain,
        tenant_id: String,
        requester: String,
        pdp: &dyn ShredAuthorizationPort,
        quorum: QuorumPolicy,
        waiting_window_seconds: u64,
        now_epoch_seconds: u64,
    ) -> Result<Self, (KekVersionChain, ShredError)> {
        if waiting_window_seconds < MIN_WAITING_WINDOW_SECONDS {
            return Err((
                chain,
                ShredError::WindowTooShort {
                    floor_seconds: MIN_WAITING_WINDOW_SECONDS,
                },
            ));
        }
        let request = ShredAuthorizationRequest {
            tenant_id: tenant_id.clone(),
            kek_id: chain.kek_id().clone(),
            actor: requester.clone(),
            action: ShredAction::Schedule,
            requested_at_epoch_seconds: now_epoch_seconds,
        };
        match pdp.authorize(&request) {
            ShredDecision::Permit(evidence) => Ok(Self {
                pending: PendingDeletionChain { chain },
                tenant_id,
                requester,
                schedule_decision: evidence,
                approvals: BTreeSet::new(),
                quorum,
                scheduled_at_epoch_seconds: now_epoch_seconds,
                earliest_shred_at_epoch_seconds: now_epoch_seconds
                    .saturating_add(waiting_window_seconds),
            }),
            ShredDecision::Deny(evidence) => Err((chain, ShredError::NotPermitted { evidence })),
        }
    }

    /// Decrypt-only custody for read-path availability during the window.
    pub fn pending_chain(&self) -> &PendingDeletionChain {
        &self.pending
    }

    /// Record one approval. Approvers must be distinct and must not be the
    /// requester.
    pub fn approve(&mut self, approver: String) -> Result<u32, ShredError> {
        if approver == self.requester {
            return Err(ShredError::RequesterCannotApprove);
        }
        if !self.approvals.insert(approver) {
            return Err(ShredError::DuplicateApprover);
        }
        Ok(u32::try_from(self.approvals.len()).unwrap_or(u32::MAX))
    }

    /// Earliest epoch second execution is allowed.
    pub fn earliest_shred_at_epoch_seconds(&self) -> u64 {
        self.earliest_shred_at_epoch_seconds
    }

    /// Cancel the pending deletion (PDP-gated like scheduling) and return
    /// full encrypt-capable custody PLUS the attribution evidence — a
    /// cancellation keeps tenant data alive past a deletion request, so it
    /// must chain to its policy decision exactly like execution does.
    #[allow(clippy::result_large_err)] // custody must stay inside on failure
    pub fn cancel(
        self,
        actor: String,
        pdp: &dyn ShredAuthorizationPort,
        now_epoch_seconds: u64,
    ) -> Result<(KekVersionChain, CancelEvidence), (Self, ShredError)> {
        let request = ShredAuthorizationRequest {
            tenant_id: self.tenant_id.clone(),
            kek_id: self.pending.kek_id().clone(),
            actor: actor.clone(),
            action: ShredAction::Cancel,
            requested_at_epoch_seconds: now_epoch_seconds,
        };
        match pdp.authorize(&request) {
            ShredDecision::Permit(cancel_decision) => {
                let evidence = CancelEvidence {
                    kek_id: self.pending.kek_id().clone(),
                    tenant_id: self.tenant_id,
                    cancelled_by: actor,
                    cancel_decision,
                    schedule_decision: self.schedule_decision,
                    approvals_at_cancel: self.approvals.into_iter().collect(),
                    scheduled_at_epoch_seconds: self.scheduled_at_epoch_seconds,
                    cancelled_at_epoch_seconds: now_epoch_seconds,
                };
                Ok((self.pending.chain, evidence))
            }
            ShredDecision::Deny(evidence) => Err((self, ShredError::NotPermitted { evidence })),
        }
    }

    /// Execute the shred: consumes the deletion and the chain. Every key
    /// version's material zeroizes as it drops; the returned proof is the
    /// only artifact that survives. Fails closed (custody returned) if the
    /// quorum or the waiting window is not satisfied.
    #[allow(clippy::result_large_err)] // custody must travel back on failure
    pub fn execute(self, now_epoch_seconds: u64) -> Result<ShredProof, (Self, ShredError)> {
        let need = self.quorum.required_approvals.get();
        let have = u32::try_from(self.approvals.len()).unwrap_or(u32::MAX);
        if have < need {
            return Err((self, ShredError::QuorumNotReached { have, need }));
        }
        if now_epoch_seconds < self.earliest_shred_at_epoch_seconds {
            let earliest_at_epoch_seconds = self.earliest_shred_at_epoch_seconds;
            return Err((
                self,
                ShredError::WindowNotElapsed {
                    earliest_at_epoch_seconds,
                },
            ));
        }
        let chain = self.pending.chain;
        let versions_destroyed = 1 + chain.retired_versions().count() as u64;
        let proof = ShredProof {
            kek_id: chain.kek_id().clone(),
            tenant_id: self.tenant_id,
            schedule_decision: self.schedule_decision,
            approvers: self.approvals.into_iter().collect(),
            versions_destroyed,
            scheduled_at_epoch_seconds: self.scheduled_at_epoch_seconds,
            executed_at_epoch_seconds: now_epoch_seconds,
        };
        // `chain` drops here: every MlockedKey zeroizes and unlocks. The KEK
        // is unrecoverable; per-tenant data under it is crypto-shredded.
        drop(chain);
        Ok(proof)
    }
}

impl std::fmt::Debug for ScheduledKeyDeletion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ScheduledKeyDeletion {{ kek_id: {}, tenant: {}, approvals: {}/{}, window_until: {}, keys: [REDACTED] }}",
            self.pending.kek_id(),
            self.tenant_id,
            self.approvals.len(),
            self.quorum.required_approvals.get(),
            self.earliest_shred_at_epoch_seconds
        )
    }
}
