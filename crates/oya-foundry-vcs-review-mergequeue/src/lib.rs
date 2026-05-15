//! Review/fix/rebase/merge-queue loop for Oya VCS.
//!
//! This crate is pure and provider-free. It models controller-owned review,
//! rebase, and merge-queue events and returns bounded agent fixup work without
//! letting agents own direct rebase/merge operations.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

use oya_foundry_vcs_kernel::{ChangeSet, CiState, QueueState, ReviewState, SymbolId};

pub const REVIEW_MERGEQUEUE_SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_ISSUE_DIGEST_SLA_SECONDS: u64 = 86_400;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FixupSource {
    ReviewComment,
    CiFailure,
    SecurityFinding,
    RebaseConflict,
    MergeQueueFailure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FixupOwner {
    Agent,
    Controller,
    SecurityReviewer,
    Queue,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FixupState {
    Open,
    InProgress,
    Resolved,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockRef {
    pub claim_id: String, // data_class: INTERNAL_ONLY
    pub symbol: SymbolId, // data_class: INTERNAL_ONLY
}

impl LockRef {
    pub fn new(claim_id: impl Into<String>, symbol: SymbolId) -> Result<Self, ReviewQueueError> {
        Ok(Self {
            claim_id: validate_prefixed(claim_id.into(), "claim_", ReviewQueueError::InvalidLock)?,
            symbol,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewComment {
    pub id: String,              // data_class: INTERNAL_ONLY
    pub changeset_id: String,    // data_class: INTERNAL_ONLY
    pub lock_refs: Vec<LockRef>, // data_class: INTERNAL_ONLY
    pub body: String,            // data_class: INTERNAL_ONLY
    pub blocking: bool,          // data_class: INTERNAL_ONLY
}

impl ReviewComment {
    pub fn blocking(
        id: impl Into<String>,
        changeset_id: impl Into<String>,
        lock_refs: Vec<LockRef>,
        body: impl Into<String>,
    ) -> Result<Self, ReviewQueueError> {
        Self::new(id, changeset_id, lock_refs, body, true)
    }

    pub fn new(
        id: impl Into<String>,
        changeset_id: impl Into<String>,
        lock_refs: Vec<LockRef>,
        body: impl Into<String>,
        blocking: bool,
    ) -> Result<Self, ReviewQueueError> {
        ensure_locks(&lock_refs)?;
        Ok(Self {
            id: validate_prefixed(id.into(), "rvw_", ReviewQueueError::InvalidReview)?,
            changeset_id: validate_prefixed(
                changeset_id.into(),
                "cs_",
                ReviewQueueError::InvalidChangeSet,
            )?,
            lock_refs,
            body: normalize_non_empty(body.into(), ReviewQueueError::InvalidReview)?,
            blocking,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CiFailure {
    pub id: String,              // data_class: INTERNAL_ONLY
    pub suite: String,           // data_class: INTERNAL_ONLY
    pub command: String,         // data_class: INTERNAL_ONLY
    pub lock_refs: Vec<LockRef>, // data_class: INTERNAL_ONLY
}

impl CiFailure {
    pub fn new(
        id: impl Into<String>,
        suite: impl Into<String>,
        command: impl Into<String>,
        lock_refs: Vec<LockRef>,
    ) -> Result<Self, ReviewQueueError> {
        ensure_locks(&lock_refs)?;
        Ok(Self {
            id: validate_prefixed(id.into(), "ci_", ReviewQueueError::InvalidCiFailure)?,
            suite: normalize_non_empty(suite.into(), ReviewQueueError::InvalidCiFailure)?,
            command: normalize_non_empty(command.into(), ReviewQueueError::InvalidCiFailure)?,
            lock_refs,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityFinding {
    pub id: String,                 // data_class: INTERNAL_ONLY
    pub severity: SecuritySeverity, // data_class: INTERNAL_ONLY
    pub summary: String,            // data_class: INTERNAL_ONLY
    pub lock_refs: Vec<LockRef>,    // data_class: INTERNAL_ONLY
}

impl SecurityFinding {
    pub fn new(
        id: impl Into<String>,
        severity: SecuritySeverity,
        summary: impl Into<String>,
        lock_refs: Vec<LockRef>,
    ) -> Result<Self, ReviewQueueError> {
        ensure_locks(&lock_refs)?;
        Ok(Self {
            id: validate_prefixed(id.into(), "sec_", ReviewQueueError::InvalidSecurityFinding)?,
            severity,
            summary: normalize_non_empty(summary.into(), ReviewQueueError::InvalidSecurityFinding)?,
            lock_refs,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RebaseConflict {
    pub id: String,                    // data_class: INTERNAL_ONLY
    pub controller_id: String,         // data_class: INTERNAL_ONLY
    pub base_generation: u64,          // data_class: INTERNAL_ONLY
    pub lock_refs: Vec<LockRef>,       // data_class: INTERNAL_ONLY
    pub conflicted_paths: Vec<String>, // data_class: INTERNAL_ONLY
}

impl RebaseConflict {
    pub fn new(
        id: impl Into<String>,
        controller_id: impl Into<String>,
        base_generation: u64,
        lock_refs: Vec<LockRef>,
        conflicted_paths: Vec<String>,
    ) -> Result<Self, ReviewQueueError> {
        let controller_id = controller_id.into();
        ensure_controller(&controller_id)?;
        ensure_locks(&lock_refs)?;
        if base_generation == 0 || conflicted_paths.is_empty() {
            return Err(ReviewQueueError::InvalidRebaseConflict);
        }
        Ok(Self {
            id: validate_prefixed(id.into(), "rb_", ReviewQueueError::InvalidRebaseConflict)?,
            controller_id,
            base_generation,
            lock_refs,
            conflicted_paths: normalize_vec(
                conflicted_paths,
                ReviewQueueError::InvalidRebaseConflict,
            )?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MergeQueueFailure {
    pub id: String,              // data_class: INTERNAL_ONLY
    pub queue_id: String,        // data_class: INTERNAL_ONLY
    pub reason: String,          // data_class: INTERNAL_ONLY
    pub lock_refs: Vec<LockRef>, // data_class: INTERNAL_ONLY
}

impl MergeQueueFailure {
    pub fn new(
        id: impl Into<String>,
        queue_id: impl Into<String>,
        reason: impl Into<String>,
        lock_refs: Vec<LockRef>,
    ) -> Result<Self, ReviewQueueError> {
        ensure_locks(&lock_refs)?;
        Ok(Self {
            id: validate_prefixed(id.into(), "mq_", ReviewQueueError::InvalidMergeQueueFailure)?,
            queue_id: normalize_non_empty(
                queue_id.into(),
                ReviewQueueError::InvalidMergeQueueFailure,
            )?,
            reason: normalize_non_empty(reason.into(), ReviewQueueError::InvalidMergeQueueFailure)?,
            lock_refs,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixupTask {
    pub id: String,                 // data_class: INTERNAL_ONLY
    pub changeset_id: String,       // data_class: INTERNAL_ONLY
    pub source: FixupSource,        // data_class: INTERNAL_ONLY
    pub owner: FixupOwner,          // data_class: INTERNAL_ONLY
    pub state: FixupState,          // data_class: INTERNAL_ONLY
    pub lock_refs: Vec<LockRef>,    // data_class: INTERNAL_ONLY
    pub issue_refs: Vec<String>,    // data_class: INTERNAL_ONLY
    pub bounded_paths: Vec<String>, // data_class: INTERNAL_ONLY
    pub controller_owned: bool,     // data_class: INTERNAL_ONLY
}

impl FixupTask {
    pub fn new(
        id: impl Into<String>,
        changeset_id: impl Into<String>,
        source: FixupSource,
        owner: FixupOwner,
        lock_refs: Vec<LockRef>,
        issue_refs: Vec<String>,
        bounded_paths: Vec<String>,
    ) -> Result<Self, ReviewQueueError> {
        ensure_locks(&lock_refs)?;
        let controller_owned = matches!(owner, FixupOwner::Controller | FixupOwner::Queue);
        let task = Self {
            id: validate_prefixed(id.into(), "fix_", ReviewQueueError::InvalidFixupTask)?,
            changeset_id: validate_prefixed(
                changeset_id.into(),
                "cs_",
                ReviewQueueError::InvalidChangeSet,
            )?,
            source,
            owner,
            state: FixupState::Open,
            lock_refs,
            issue_refs: normalize_vec(issue_refs, ReviewQueueError::InvalidFixupTask)?,
            bounded_paths: normalize_vec(bounded_paths, ReviewQueueError::UnboundedFixup)?,
            controller_owned,
        };
        task.validate_ownership_and_bounds()?;
        Ok(task)
    }

    pub fn validate_ownership_and_bounds(&self) -> Result<(), ReviewQueueError> {
        if self.lock_refs.is_empty() || self.bounded_paths.is_empty() {
            return Err(ReviewQueueError::UnboundedFixup);
        }
        if matches!(self.source, FixupSource::RebaseConflict) && self.owner == FixupOwner::Agent {
            return Err(ReviewQueueError::AgentOwnedRebaseRejected);
        }
        if matches!(self.source, FixupSource::MergeQueueFailure) && self.owner != FixupOwner::Queue
        {
            return Err(ReviewQueueError::InvalidFixupOwner);
        }
        Ok(())
    }

    pub fn accept_fix(&mut self) -> Result<(), ReviewQueueError> {
        if !matches!(self.state, FixupState::Open | FixupState::InProgress) {
            return Err(ReviewQueueError::InvalidFixupTransition);
        }
        self.state = FixupState::Resolved;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminalState {
    Merged,
    Rejected,
    Superseded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewLoopState {
    pub changeset: ChangeSet,                  // data_class: INTERNAL_ONLY
    pub fixups: Vec<FixupTask>,                // data_class: INTERNAL_ONLY
    pub terminal_state: Option<TerminalState>, // data_class: INTERNAL_ONLY
    pub lock_released: bool,                   // data_class: INTERNAL_ONLY
}

impl ReviewLoopState {
    pub fn new(changeset: ChangeSet) -> Self {
        Self {
            changeset,
            fixups: Vec::new(),
            terminal_state: None,
            lock_released: false,
        }
    }

    pub fn has_open_fixups(&self) -> bool {
        self.fixups
            .iter()
            .any(|task| !matches!(task.state, FixupState::Resolved | FixupState::Cancelled))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReviewLoopEvent {
    ReviewRejected(Vec<ReviewComment>),
    ReviewApproved,
    CiFailed(Vec<CiFailure>),
    CiPassed,
    SecurityFindings(Vec<SecurityFinding>),
    ControllerRebaseConflict(RebaseConflict),
    AgentRequestedRebase {
        agent_id: String,
        changeset_id: String,
    },
    MergeQueueFailed(MergeQueueFailure),
    FixAccepted {
        task_id: String,
    },
    Enqueued,
    VirtualMerged,
    Merged,
    Rejected,
    Superseded,
    ReleaseLock,
}

pub fn reduce_terminal_state(
    state: &mut ReviewLoopState,
    event: ReviewLoopEvent,
) -> Result<(), ReviewQueueError> {
    if state.lock_released {
        return Err(ReviewQueueError::LockAlreadyReleased);
    }
    if state.terminal_state.is_some() && !matches!(event, ReviewLoopEvent::ReleaseLock) {
        return Err(ReviewQueueError::TerminalStateTransitionRejected);
    }
    match event {
        ReviewLoopEvent::ReviewRejected(comments) => {
            if comments.is_empty() {
                return Err(ReviewQueueError::InvalidReview);
            }
            state.changeset.attach_review(ReviewState::ChangesRequested);
            for comment in comments.into_iter().filter(|comment| comment.blocking) {
                let paths = paths_from_locks(&comment.lock_refs);
                state.fixups.push(FixupTask::new(
                    format!("fix_review_{}", comment.id),
                    comment.changeset_id,
                    FixupSource::ReviewComment,
                    FixupOwner::Agent,
                    comment.lock_refs,
                    vec![comment.id],
                    paths,
                )?);
            }
        }
        ReviewLoopEvent::ReviewApproved => {
            if state.has_open_fixups() {
                return Err(ReviewQueueError::OpenFixupsBlockTerminalState);
            }
            state.changeset.attach_review(ReviewState::Approved);
        }
        ReviewLoopEvent::CiFailed(failures) => {
            if failures.is_empty() {
                return Err(ReviewQueueError::InvalidCiFailure);
            }
            state.changeset.attach_ci(CiState::Failed);
            for failure in failures {
                let paths = paths_from_locks(&failure.lock_refs);
                state.fixups.push(FixupTask::new(
                    format!("fix_ci_{}", failure.id),
                    state.changeset.id.clone(),
                    FixupSource::CiFailure,
                    FixupOwner::Agent,
                    failure.lock_refs,
                    vec![failure.id],
                    paths,
                )?);
            }
        }
        ReviewLoopEvent::CiPassed => {
            if state.has_open_fixups() {
                return Err(ReviewQueueError::OpenFixupsBlockTerminalState);
            }
            state.changeset.attach_ci(CiState::Passed);
        }
        ReviewLoopEvent::SecurityFindings(findings) => {
            if findings.is_empty() {
                return Err(ReviewQueueError::InvalidSecurityFinding);
            }
            for finding in findings {
                let owner = match finding.severity {
                    SecuritySeverity::Low | SecuritySeverity::Medium => FixupOwner::Agent,
                    SecuritySeverity::High | SecuritySeverity::Critical => {
                        FixupOwner::SecurityReviewer
                    }
                };
                let paths = paths_from_locks(&finding.lock_refs);
                state.fixups.push(FixupTask::new(
                    format!("fix_sec_{}", finding.id),
                    state.changeset.id.clone(),
                    FixupSource::SecurityFinding,
                    owner,
                    finding.lock_refs,
                    vec![finding.id],
                    paths,
                )?);
            }
        }
        ReviewLoopEvent::ControllerRebaseConflict(conflict) => {
            let paths = conflict.conflicted_paths.clone();
            state.fixups.push(FixupTask::new(
                format!("fix_rebase_{}", conflict.id),
                state.changeset.id.clone(),
                FixupSource::RebaseConflict,
                FixupOwner::Controller,
                conflict.lock_refs,
                vec![conflict.id],
                paths,
            )?);
        }
        ReviewLoopEvent::AgentRequestedRebase { .. } => {
            return Err(ReviewQueueError::AgentOwnedRebaseRejected);
        }
        ReviewLoopEvent::MergeQueueFailed(failure) => {
            state.changeset.queue_state = QueueState::Draft;
            let paths = paths_from_locks(&failure.lock_refs);
            state.fixups.push(FixupTask::new(
                format!("fix_mq_{}", failure.id),
                state.changeset.id.clone(),
                FixupSource::MergeQueueFailure,
                FixupOwner::Queue,
                failure.lock_refs,
                vec![failure.id],
                paths,
            )?);
        }
        ReviewLoopEvent::FixAccepted { task_id } => {
            let task = state
                .fixups
                .iter_mut()
                .find(|task| task.id == task_id)
                .ok_or(ReviewQueueError::FixupNotFound)?;
            task.accept_fix()?;
        }
        ReviewLoopEvent::Enqueued => {
            if state.has_open_fixups() {
                return Err(ReviewQueueError::OpenFixupsBlockTerminalState);
            }
            state
                .changeset
                .mark_ready_for_queue()
                .map_err(|_| ReviewQueueError::NotQueueAdmissible)?;
        }
        ReviewLoopEvent::VirtualMerged => {
            if state.changeset.queue_state != QueueState::Ready {
                return Err(ReviewQueueError::NotQueueAdmissible);
            }
            state.changeset.queue_state = QueueState::VirtualMerged;
        }
        ReviewLoopEvent::Merged => {
            if state.has_open_fixups() || state.changeset.queue_state != QueueState::VirtualMerged {
                return Err(ReviewQueueError::OpenFixupsBlockTerminalState);
            }
            state.changeset.queue_state = QueueState::PhysicallyMerged;
            state.terminal_state = Some(TerminalState::Merged);
        }
        ReviewLoopEvent::Rejected => {
            state.terminal_state = Some(TerminalState::Rejected);
        }
        ReviewLoopEvent::Superseded => {
            state.changeset.queue_state = QueueState::Superseded;
            state.terminal_state = Some(TerminalState::Superseded);
        }
        ReviewLoopEvent::ReleaseLock => {
            if state.terminal_state.is_none() {
                return Err(ReviewQueueError::LockReleaseBeforeTerminalState);
            }
            state.lock_released = true;
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssueDigest {
    pub digest: String,         // data_class: INTERNAL_ONLY
    pub observed_at_epoch: u64, // data_class: INTERNAL_ONLY
    pub sla_seconds: u64,       // data_class: INTERNAL_ONLY
}

impl IssueDigest {
    pub fn new(
        digest: impl Into<String>,
        observed_at_epoch: u64,
        sla_seconds: u64,
    ) -> Result<Self, ReviewQueueError> {
        if observed_at_epoch == 0 || sla_seconds == 0 {
            return Err(ReviewQueueError::InvalidIssueDigest);
        }
        Ok(Self {
            digest: validate_prefixed(
                digest.into(),
                "issue_digest_",
                ReviewQueueError::InvalidIssueDigest,
            )?,
            observed_at_epoch,
            sla_seconds,
        })
    }

    pub fn is_fresh_at(&self, now_epoch: u64) -> bool {
        self.observed_at_epoch <= now_epoch
            && now_epoch.saturating_sub(self.observed_at_epoch) <= self.sla_seconds
    }
}

pub fn admit_promotion(
    state: &ReviewLoopState,
    digest: &IssueDigest,
    now_epoch: u64,
) -> Result<(), ReviewQueueError> {
    if !digest.is_fresh_at(now_epoch) {
        return Err(ReviewQueueError::StaleIssueDigestBlocksPromotion);
    }
    if state.has_open_fixups() || state.changeset.queue_state != QueueState::VirtualMerged {
        return Err(ReviewQueueError::NotQueueAdmissible);
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderReview {
    Accepted,
    Rejected(Vec<ReviewComment>),
}

#[derive(Clone, Debug)]
pub struct FakeReviewProvider {
    outcomes: BTreeMap<String, VecDeque<ProviderReview>>,
}

impl FakeReviewProvider {
    pub fn new() -> Self {
        Self {
            outcomes: BTreeMap::new(),
        }
    }

    pub fn inject(&mut self, changeset_id: impl Into<String>, outcome: ProviderReview) {
        self.outcomes
            .entry(changeset_id.into())
            .or_default()
            .push_back(outcome);
    }

    pub fn review(&mut self, changeset: &ChangeSet) -> ProviderReview {
        self.outcomes
            .get_mut(&changeset.id)
            .and_then(VecDeque::pop_front)
            .unwrap_or(ProviderReview::Accepted)
    }
}

impl Default for FakeReviewProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderCi {
    Passed,
    Failed(Vec<CiFailure>),
}

#[derive(Clone, Debug)]
pub struct FakeCiProvider {
    outcomes: BTreeMap<String, VecDeque<ProviderCi>>,
}

impl FakeCiProvider {
    pub fn new() -> Self {
        Self {
            outcomes: BTreeMap::new(),
        }
    }

    pub fn inject(&mut self, changeset_id: impl Into<String>, outcome: ProviderCi) {
        self.outcomes
            .entry(changeset_id.into())
            .or_default()
            .push_back(outcome);
    }

    pub fn run(&mut self, changeset: &ChangeSet) -> ProviderCi {
        self.outcomes
            .get_mut(&changeset.id)
            .and_then(VecDeque::pop_front)
            .unwrap_or(ProviderCi::Passed)
    }
}

impl Default for FakeCiProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueueAdmission {
    Enqueued,
    Failed(MergeQueueFailure),
    BlockedBy(String),
}

#[derive(Clone, Debug)]
pub struct FakeMergeQueueAdapter {
    outcomes: BTreeMap<String, VecDeque<QueueAdmission>>,
    admitted: Vec<String>,
}

impl FakeMergeQueueAdapter {
    pub fn new() -> Self {
        Self {
            outcomes: BTreeMap::new(),
            admitted: Vec::new(),
        }
    }

    pub fn inject(&mut self, changeset_id: impl Into<String>, outcome: QueueAdmission) {
        self.outcomes
            .entry(changeset_id.into())
            .or_default()
            .push_back(outcome);
    }

    pub fn admitted(&self) -> &[String] {
        &self.admitted
    }

    pub fn submit(
        &mut self,
        candidate: &mut ReviewLoopState,
        all_states: &[ReviewLoopState],
    ) -> Result<QueueAdmission, ReviewQueueError> {
        if candidate.has_open_fixups() {
            return Ok(QueueAdmission::BlockedBy(candidate.changeset.id.clone()));
        }
        if let Some(blocker) = all_states.iter().find(|state| {
            state.changeset.id != candidate.changeset.id
                && state.has_open_fixups()
                && conflicts(&state.changeset, &candidate.changeset)
        }) {
            return Ok(QueueAdmission::BlockedBy(blocker.changeset.id.clone()));
        }
        let outcome = self
            .outcomes
            .get_mut(&candidate.changeset.id)
            .and_then(VecDeque::pop_front)
            .unwrap_or(QueueAdmission::Enqueued);
        match &outcome {
            QueueAdmission::Enqueued => {
                reduce_terminal_state(candidate, ReviewLoopEvent::Enqueued)?;
                self.admitted.push(candidate.changeset.id.clone());
            }
            QueueAdmission::Failed(failure) => {
                reduce_terminal_state(
                    candidate,
                    ReviewLoopEvent::MergeQueueFailed(failure.clone()),
                )?;
            }
            QueueAdmission::BlockedBy(_) => {}
        }
        Ok(outcome)
    }
}

impl Default for FakeMergeQueueAdapter {
    fn default() -> Self {
        Self::new()
    }
}

pub fn drive_review_and_ci(
    state: &mut ReviewLoopState,
    reviews: &mut FakeReviewProvider,
    ci: &mut FakeCiProvider,
) -> Result<(), ReviewQueueError> {
    match reviews.review(&state.changeset) {
        ProviderReview::Accepted => reduce_terminal_state(state, ReviewLoopEvent::ReviewApproved)?,
        ProviderReview::Rejected(comments) => {
            reduce_terminal_state(state, ReviewLoopEvent::ReviewRejected(comments))?
        }
    }
    if !state.has_open_fixups() {
        match ci.run(&state.changeset) {
            ProviderCi::Passed => reduce_terminal_state(state, ReviewLoopEvent::CiPassed)?,
            ProviderCi::Failed(failures) => {
                reduce_terminal_state(state, ReviewLoopEvent::CiFailed(failures))?
            }
        }
    }
    Ok(())
}

fn conflicts(left: &ChangeSet, right: &ChangeSet) -> bool {
    let left_writes = left
        .write_symbols
        .iter()
        .map(|symbol| symbol.value.clone())
        .collect::<BTreeSet<_>>();
    let right_writes = right
        .write_symbols
        .iter()
        .map(|symbol| symbol.value.clone())
        .collect::<BTreeSet<_>>();
    !left_writes.is_disjoint(&right_writes)
}

fn paths_from_locks(lock_refs: &[LockRef]) -> Vec<String> {
    let mut paths = lock_refs
        .iter()
        .map(|lock_ref| lock_ref.symbol.artifact.path.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if paths.is_empty() {
        paths.push("<unknown>".to_string());
    }
    paths
}

fn ensure_locks(lock_refs: &[LockRef]) -> Result<(), ReviewQueueError> {
    if lock_refs.is_empty() {
        return Err(ReviewQueueError::UnboundedFixup);
    }
    Ok(())
}

fn ensure_controller(value: &str) -> Result<(), ReviewQueueError> {
    let value = normalize_non_empty(value.to_string(), ReviewQueueError::InvalidRebaseConflict)?;
    if value.starts_with("agent-") || value.starts_with("agent_") {
        return Err(ReviewQueueError::AgentOwnedRebaseRejected);
    }
    Ok(())
}

fn normalize_vec(
    values: Vec<String>,
    error: ReviewQueueError,
) -> Result<Vec<String>, ReviewQueueError> {
    if values.is_empty() {
        return Err(error);
    }
    values
        .into_iter()
        .map(|value| normalize_non_empty(value, error.clone()))
        .collect()
}

fn normalize_non_empty(value: String, error: ReviewQueueError) -> Result<String, ReviewQueueError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        Err(error)
    } else {
        Ok(value)
    }
}

fn validate_prefixed(
    value: String,
    prefix: &str,
    error: ReviewQueueError,
) -> Result<String, ReviewQueueError> {
    let value = normalize_non_empty(value, error.clone())?;
    if value.starts_with(prefix) && value.len() > prefix.len() {
        Ok(value)
    } else {
        Err(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReviewQueueError {
    InvalidChangeSet,
    InvalidLock,
    InvalidReview,
    InvalidCiFailure,
    InvalidSecurityFinding,
    InvalidRebaseConflict,
    InvalidMergeQueueFailure,
    InvalidFixupTask,
    InvalidFixupOwner,
    InvalidFixupTransition,
    FixupNotFound,
    UnboundedFixup,
    AgentOwnedRebaseRejected,
    OpenFixupsBlockTerminalState,
    LockReleaseBeforeTerminalState,
    LockAlreadyReleased,
    TerminalStateTransitionRejected,
    NotQueueAdmissible,
    InvalidIssueDigest,
    StaleIssueDigestBlocksPromotion,
}

impl fmt::Display for ReviewQueueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for ReviewQueueError {}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_foundry_vcs_kernel::{
        ArtifactPointer, ChangeSetDraft, ChangeSetLineage, SymbolLanguage,
    };

    fn symbol(path: &str, name: &str) -> SymbolId {
        SymbolId::new(
            SymbolLanguage::Rust,
            ArtifactPointer::file(path).expect("artifact"),
            name,
        )
        .expect("symbol")
    }

    fn changeset(id: &str, path: &str, symbol_name: &str) -> ChangeSet {
        let sym = symbol(path, symbol_name);
        ChangeSet::new(ChangeSetDraft {
            id: id.to_string(),
            agent_id: format!("agent-{id}"),
            target_branch: "dev".to_string(),
            base_sha: "0123456789abcdef0123456789abcdef01234567".to_string(),
            branch_or_workspace_ref: format!("workspace/{id}"),
            patch_id: format!("patch_{id}"),
            write_symbols: vec![sym.clone()],
            read_symbols: Vec::new(),
            touched_files: vec![sym.artifact.clone()],
            dependencies: Vec::new(),
            lineage: ChangeSetLineage::new("wi_ip007", "ip_007", Vec::new()).expect("lineage"),
            evidence_refs: vec![
                "evidence/gitops-vcs/ip-007-review-mergequeue.json".to_string(),
            ],
        })
        .expect("changeset")
    }

    fn lock_for(cs: &ChangeSet) -> LockRef {
        LockRef::new("claim_ip007", cs.write_symbols[0].clone()).expect("lock")
    }

    #[test]
    fn terminal_state_reducer_blocks_release_until_terminal() {
        let mut state = ReviewLoopState::new(changeset("cs_release", "src/release.rs", "release"));
        assert_eq!(
            reduce_terminal_state(&mut state, ReviewLoopEvent::ReleaseLock),
            Err(ReviewQueueError::LockReleaseBeforeTerminalState)
        );

        reduce_terminal_state(&mut state, ReviewLoopEvent::Rejected).expect("reject terminal");
        reduce_terminal_state(&mut state, ReviewLoopEvent::ReleaseLock).expect("release terminal");
        assert!(state.lock_released);
    }

    #[test]
    fn terminal_states_absorb_until_lock_release() {
        let mut rejected = ReviewLoopState::new(changeset(
            "cs_terminal_rejected",
            "crates/demo/src/rejected.rs",
            "demo::rejected",
        ));
        reduce_terminal_state(&mut rejected, ReviewLoopEvent::Rejected).expect("reject terminal");
        assert_eq!(
            reduce_terminal_state(&mut rejected, ReviewLoopEvent::ReviewApproved),
            Err(ReviewQueueError::TerminalStateTransitionRejected)
        );
        assert_eq!(
            reduce_terminal_state(&mut rejected, ReviewLoopEvent::Enqueued),
            Err(ReviewQueueError::TerminalStateTransitionRejected)
        );
        reduce_terminal_state(&mut rejected, ReviewLoopEvent::ReleaseLock)
            .expect("release after terminal remains allowed");

        let mut merged = ReviewLoopState::new(changeset(
            "cs_terminal_merged",
            "crates/demo/src/merged.rs",
            "demo::merged",
        ));
        reduce_terminal_state(&mut merged, ReviewLoopEvent::ReviewApproved).unwrap();
        reduce_terminal_state(&mut merged, ReviewLoopEvent::CiPassed).unwrap();
        reduce_terminal_state(&mut merged, ReviewLoopEvent::Enqueued).unwrap();
        reduce_terminal_state(&mut merged, ReviewLoopEvent::VirtualMerged).unwrap();
        reduce_terminal_state(&mut merged, ReviewLoopEvent::Merged).unwrap();
        assert_eq!(
            reduce_terminal_state(
                &mut merged,
                ReviewLoopEvent::FixAccepted {
                    task_id: "fix_after_merge".into()
                }
            ),
            Err(ReviewQueueError::TerminalStateTransitionRejected)
        );
    }

    #[test]
    fn fixup_task_requires_owner_and_bounded_locks() {
        let cs = changeset("cs_bounds", "src/bounds.rs", "bounds");
        let lock = lock_for(&cs);
        let task = FixupTask::new(
            "fix_bounds",
            cs.id.clone(),
            FixupSource::ReviewComment,
            FixupOwner::Agent,
            vec![lock.clone()],
            vec!["rvw_bounds".to_string()],
            vec!["src/bounds.rs".to_string()],
        )
        .expect("bounded task");
        assert_eq!(task.owner, FixupOwner::Agent);

        assert_eq!(
            FixupTask::new(
                "fix_unbounded",
                cs.id,
                FixupSource::ReviewComment,
                FixupOwner::Agent,
                Vec::new(),
                vec!["rvw_bounds".to_string()],
                vec!["src/bounds.rs".to_string()],
            ),
            Err(ReviewQueueError::UnboundedFixup)
        );

        assert_eq!(
            FixupTask::new(
                "fix_rebase_agent",
                "cs_rebase",
                FixupSource::RebaseConflict,
                FixupOwner::Agent,
                vec![lock],
                vec!["rb_bad".to_string()],
                vec!["src/bounds.rs".to_string()],
            ),
            Err(ReviewQueueError::AgentOwnedRebaseRejected)
        );
    }

    #[test]
    fn fake_review_ci_and_merge_queue_inject_failures() {
        let mut state = ReviewLoopState::new(changeset("cs_fake", "src/fake.rs", "fake"));
        let lock = lock_for(&state.changeset);
        let mut reviews = FakeReviewProvider::new();
        let mut ci = FakeCiProvider::new();
        let mut queue = FakeMergeQueueAdapter::new();

        reviews.inject(state.changeset.id.clone(), ProviderReview::Accepted);
        ci.inject(
            state.changeset.id.clone(),
            ProviderCi::Failed(vec![
                CiFailure::new("ci_fake", "unit", "rustc --test fake", vec![lock.clone()])
                    .expect("ci failure"),
            ]),
        );
        drive_review_and_ci(&mut state, &mut reviews, &mut ci).expect("drive providers");
        assert_eq!(state.changeset.review_state, ReviewState::Approved);
        assert_eq!(state.changeset.ci_state, CiState::Failed);
        assert_eq!(state.fixups[0].source, FixupSource::CiFailure);

        let fix_id = state.fixups[0].id.clone();
        reduce_terminal_state(&mut state, ReviewLoopEvent::FixAccepted { task_id: fix_id })
            .expect("accept ci fix");
        reduce_terminal_state(&mut state, ReviewLoopEvent::CiPassed).expect("ci pass");
        queue.inject(
            state.changeset.id.clone(),
            QueueAdmission::Failed(
                MergeQueueFailure::new("mq_fake", "dev", "projection failed", vec![lock])
                    .expect("queue failure"),
            ),
        );
        assert!(matches!(
            queue.submit(&mut state, &[]).expect("queue submit"),
            QueueAdmission::Failed(_)
        ));
        assert_eq!(
            state.fixups.last().unwrap().source,
            FixupSource::MergeQueueFailure
        );
    }

    #[test]
    fn rejected_review_fix_reenters_queue_and_independent_change_bypasses_failed_item() {
        let mut failed = ReviewLoopState::new(changeset("cs_failed", "src/failed.rs", "failed"));
        let mut independent = ReviewLoopState::new(changeset("cs_independent", "src/ok.rs", "ok"));
        let failed_lock = lock_for(&failed.changeset);
        let comment = ReviewComment::blocking(
            "rvw_failed",
            failed.changeset.id.clone(),
            vec![failed_lock],
            "tighten invariant",
        )
        .expect("comment");

        reduce_terminal_state(&mut failed, ReviewLoopEvent::ReviewRejected(vec![comment]))
            .expect("review reject");
        assert!(failed.has_open_fixups());

        reduce_terminal_state(&mut independent, ReviewLoopEvent::ReviewApproved)
            .expect("review ok");
        reduce_terminal_state(&mut independent, ReviewLoopEvent::CiPassed).expect("ci ok");
        let mut queue = FakeMergeQueueAdapter::new();
        assert_eq!(
            queue
                .submit(&mut independent, &[failed.clone()])
                .expect("queue independent"),
            QueueAdmission::Enqueued
        );
        assert_eq!(queue.admitted(), &["cs_independent".to_string()]);

        let fix_id = failed.fixups[0].id.clone();
        reduce_terminal_state(
            &mut failed,
            ReviewLoopEvent::FixAccepted { task_id: fix_id },
        )
        .expect("accept fix");
        reduce_terminal_state(&mut failed, ReviewLoopEvent::ReviewApproved).expect("review pass");
        reduce_terminal_state(&mut failed, ReviewLoopEvent::CiPassed).expect("ci pass");
        assert_eq!(
            queue
                .submit(&mut failed, &[independent])
                .expect("queue fixed"),
            QueueAdmission::Enqueued
        );
        assert_eq!(queue.admitted().last().unwrap(), "cs_failed");
    }

    #[test]
    fn negative_agent_owned_rebase_is_rejected() {
        let mut state = ReviewLoopState::new(changeset("cs_rebase", "src/rebase.rs", "rebase"));
        let changeset_id = state.changeset.id.clone();
        assert_eq!(
            reduce_terminal_state(
                &mut state,
                ReviewLoopEvent::AgentRequestedRebase {
                    agent_id: "agent-cs_rebase".to_string(),
                    changeset_id,
                },
            ),
            Err(ReviewQueueError::AgentOwnedRebaseRejected)
        );

        assert_eq!(
            RebaseConflict::new(
                "rb_agent",
                "agent-cs_rebase",
                1,
                vec![lock_for(&state.changeset)],
                vec!["src/rebase.rs".to_string()],
            ),
            Err(ReviewQueueError::AgentOwnedRebaseRejected)
        );
    }

    #[test]
    fn stale_issue_digest_blocks_promotion_after_sla() {
        let mut state = ReviewLoopState::new(changeset("cs_digest", "src/digest.rs", "digest"));
        reduce_terminal_state(&mut state, ReviewLoopEvent::ReviewApproved).expect("review pass");
        reduce_terminal_state(&mut state, ReviewLoopEvent::CiPassed).expect("ci pass");
        reduce_terminal_state(&mut state, ReviewLoopEvent::Enqueued).expect("queue ready");
        reduce_terminal_state(&mut state, ReviewLoopEvent::VirtualMerged).expect("virtual merge");

        let digest = IssueDigest::new("issue_digest_ip007", 100, 10).expect("digest");
        assert_eq!(
            admit_promotion(&state, &digest, 111),
            Err(ReviewQueueError::StaleIssueDigestBlocksPromotion)
        );
        assert_eq!(admit_promotion(&state, &digest, 110), Ok(()));
    }
}
