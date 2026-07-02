//! # oya-fabric-loop-state-app — ADR-0516 fabric two-plane drive-loop state
//!
//! Loop state is TWO-PLANE (Seed contract; `specs/fabric-drive-loop-state.json`):
//!
//! - **Coordination plane** (durable): loop-card definitions in the masterplan v2
//!   single work-item ID space, dependency references, durable statuses, and
//!   attached evidence references. Lives IN-REPO (default root
//!   [`DEFAULT_DURABLE_PLANE_ROOT`]) and is PR-governed — coordinator-only
//!   commits, never worker writes.
//! - **Execution plane** (operational): claim / heartbeat / block-kind / run
//!   state for in-flight cards. Lives REPO-ADJACENT and GITIGNORED (default root
//!   [`DEFAULT_OPERATIONAL_PLANE_ROOT`]); it is never merge or plan authority.
//!
//! BOTH planes sit behind port traits ([`CoordinationPlanePort`],
//! [`ExecutionPlanePort`]) whose contract models the owned cloud-ci-native
//! destination: the named cutover target is the cloud/cloud-ci-owned loop-state
//! service [`CUTOVER_TARGET_SERVICE_NAME`] (destination home
//! [`CUTOVER_TARGET_DESTINATION_HOME`]), recorded on the port contract itself via
//! [`CutoverTarget::canonical`] and mirrored in the spec. The bundled filesystem
//! stores are retirement-marked local bridges (founder CLI directive 2026-06-09)
//! with a bridge-then-retire disposition once the cutover criteria hold.
//!
//! The single-writer facade [`LoopStateService`] is generic over the two port
//! traits and performs no I/O of its own, so every loop-state read and write is
//! forced through the ports by construction. Port-level tests live in
//! `tests/contract.rs`.
//!
//! Zero third-party dependencies: persistence is an owned canonical-JSON
//! encoder/decoder (`jsonio`), matching the repo's no-shell/no-python owned-Rust
//! deliverable bar.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Port contract constants — the named owned cloud-ci cutover target
// ---------------------------------------------------------------------------

/// The named owned cloud-ci cutover target: the cloud/cloud-ci-owned loop-state
/// service that BOTH plane ports model. Recorded here on the port contract and
/// in `specs/fabric-drive-loop-state.json#cutover_target.service_name`.
pub const CUTOVER_TARGET_SERVICE_NAME: &str = "oya-cloud-ci-loop-state-service";

/// The destination composition-root crate home for the cutover target
/// (ADR-0056 BNF `-app` layer suffix under the `cloud/*/crates/oya-*`
/// workspace glob).
pub const CUTOVER_TARGET_DESTINATION_HOME: &str =
    "cloud/cloud-ci/crates/oya-cloud-ci-loop-state-app";

/// The owning tree of the cutover target service.
pub const CUTOVER_TARGET_OWNER: &str = "cloud/cloud-ci";

/// The Seed-fixed cutover criteria, in order. The local bridge stores demote to
/// read-only bridge and then retire only after criteria 1-2 hold.
pub const CUTOVER_CRITERIA: [&str; 3] = [
    "The service passes the same six-gate plan suite (ID uniqueness, DAG acyclicity, orphan detection, projection freshness, plan-vs-evidence drift, read-contract/entry-surface) with fail-closed authz.",
    "A shadow-parity period drives real cards through the service with zero regressions versus the local bridge stores.",
    "The local bridge stores are demoted to read-only bridge and then retired.",
];

/// Repo-relative path of the machine-readable port contract spec this crate is
/// verified against.
pub const PORT_CONTRACT_SPEC_PATH: &str = "specs/fabric-drive-loop-state.json";

/// Default in-repo, PR-governed root of the durable coordination plane.
pub const DEFAULT_DURABLE_PLANE_ROOT: &str = "plan/fabric-loop";

/// Default repo-adjacent, gitignored root of the operational execution plane.
pub const DEFAULT_OPERATIONAL_PLANE_ROOT: &str = ".oya-loop-state";

/// Default in-repo, PR-governed root of the per-pass flow-metrics ledger
/// (closed-loop improvement layer of the ADR-0516 drive loop; rides the
/// durable coordination plane, coordinator-only commits).
pub const DEFAULT_FLOW_METRICS_ROOT: &str = "plan/fabric-loop/flow-metrics";

/// The named owned cloud-ci cutover target as a typed record. Every port
/// implementation reports this via the defaulted
/// [`CoordinationPlanePort::cutover_target`] /
/// [`ExecutionPlanePort::cutover_target`] contract methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CutoverTarget {
    /// Deployed service identity (`oya-cloud-ci-loop-state-service`).
    pub service_name: &'static str,
    /// Owning tree (`cloud/cloud-ci`).
    pub owner: &'static str,
    /// Destination composition-root crate home.
    pub destination_home: &'static str,
    /// Ordered cutover criteria.
    pub criteria: [&'static str; 3],
}

impl CutoverTarget {
    /// The canonical cutover target recorded by the port contract.
    #[must_use]
    pub const fn canonical() -> Self {
        Self {
            service_name: CUTOVER_TARGET_SERVICE_NAME,
            owner: CUTOVER_TARGET_OWNER,
            destination_home: CUTOVER_TARGET_DESTINATION_HOME,
            criteria: CUTOVER_CRITERIA,
        }
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Typed loop-state error surfaced by both plane ports and the facade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaneError {
    /// Underlying store I/O failed.
    Io(String),
    /// Stored bytes failed to decode as the canonical record shape.
    Corrupt(String),
    /// The referenced card does not exist on the coordination plane.
    UnknownCard(String),
    /// A card with this id already exists (single ID space; no silent upsert).
    DuplicateCard(String),
    /// An id failed the filesystem-safe id contract.
    InvalidId(String),
    /// Claim conflict: the card is already claimed by another lane.
    AlreadyClaimed {
        /// The contested card.
        card_id: String,
        /// The lane holding the claim.
        holder_lane: String,
    },
    /// The operation requires an active claim and none exists.
    NotClaimed(String),
    /// The operation was attempted by a lane that does not hold the claim.
    WrongLane {
        /// The contested card.
        card_id: String,
        /// The lane holding the claim.
        holder_lane: String,
    },
    /// A done-class status change was attempted without evidence references.
    MissingEvidence(String),
    /// The card is not ready: unsatisfied dependencies in the plan DAG.
    DependencyUnsatisfied {
        /// The card whose claim was refused.
        card_id: String,
        /// Dependency ids that are not yet done-verified.
        missing: Vec<String>,
    },
    /// The card is not in a status that permits the requested transition.
    InvalidTransition {
        /// The card.
        card_id: String,
        /// Human-readable refusal detail.
        detail: String,
    },
    /// A flow-metrics pass was recorded out of order (the ledger is
    /// append-only with a strictly monotonic pass sequence).
    NonMonotonicPass {
        /// The refused pass sequence number.
        pass_seq: u64,
        /// The latest recorded pass sequence number (0 when none).
        latest_recorded: u64,
    },
    /// A flow-metrics record failed mechanical validation.
    InvalidMetrics {
        /// The pass or card the record belongs to.
        subject: String,
        /// Human-readable refusal detail.
        detail: String,
    },
    /// A lane work surface failed mechanical validation (disjointness
    /// detector input: bad path shape, duplicate lane/card, empty surface).
    InvalidSurface {
        /// The lane whose surface was refused (or `<batch>` for batch-level
        /// violations such as duplicate lane ids).
        lane_id: String,
        /// Human-readable refusal detail.
        detail: String,
    },
    /// More concurrent lanes were requested than independent reviewer
    /// capacity admits (parallel-safety constraint: max concurrent lanes <=
    /// available independent reviewer capacity).
    LaneCapacityExceeded {
        /// Requested concurrent lane count.
        lanes: usize,
        /// Available independent reviewer capacity.
        reviewer_capacity: usize,
    },
}

impl fmt::Display for PlaneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(detail) => write!(f, "loop-state io error: {detail}"),
            Self::Corrupt(detail) => write!(f, "loop-state corrupt record: {detail}"),
            Self::UnknownCard(id) => write!(f, "unknown loop card: {id}"),
            Self::DuplicateCard(id) => write!(f, "duplicate loop card id: {id}"),
            Self::InvalidId(id) => write!(f, "invalid loop-state id: {id:?}"),
            Self::AlreadyClaimed {
                card_id,
                holder_lane,
            } => write!(f, "card {card_id} already claimed by lane {holder_lane}"),
            Self::NotClaimed(id) => write!(f, "card {id} has no active claim"),
            Self::WrongLane {
                card_id,
                holder_lane,
            } => write!(f, "card {card_id} is held by lane {holder_lane}"),
            Self::MissingEvidence(id) => {
                write!(f, "card {id}: done-class status requires evidence refs")
            }
            Self::DependencyUnsatisfied { card_id, missing } => {
                write!(f, "card {card_id} not ready; unsatisfied deps: {missing:?}")
            }
            Self::InvalidTransition { card_id, detail } => {
                write!(f, "card {card_id}: invalid transition: {detail}")
            }
            Self::NonMonotonicPass {
                pass_seq,
                latest_recorded,
            } => write!(
                f,
                "flow-metrics pass {pass_seq} is not after latest recorded pass {latest_recorded}"
            ),
            Self::InvalidMetrics { subject, detail } => {
                write!(f, "invalid flow metrics for {subject}: {detail}")
            }
            Self::InvalidSurface { lane_id, detail } => {
                write!(f, "invalid lane work surface for {lane_id}: {detail}")
            }
            Self::LaneCapacityExceeded {
                lanes,
                reviewer_capacity,
            } => write!(
                f,
                "lane capacity exceeded: {lanes} concurrent lanes requested, reviewer capacity {reviewer_capacity}"
            ),
        }
    }
}

impl std::error::Error for PlaneError {}

// ---------------------------------------------------------------------------
// Domain model
// ---------------------------------------------------------------------------

/// Durable card status on the coordination plane. Evidence discipline follows
/// masterplan v2 `evidence_state_policy`: done-class statuses require attached
/// evidence references, and `claimed-done-unverified` never surfaces as done.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardStatus {
    /// Defined but not yet dispatchable.
    Defined,
    /// Dispatch-eligible (subject to DAG readiness).
    Ready,
    /// Durably blocked (dependency/decision escalated to the plan).
    Blocked,
    /// Completion claimed with evidence attached, verification pending.
    ClaimedDoneUnverified,
    /// Completion verified against evidence.
    DoneVerified,
}

impl CardStatus {
    /// Canonical wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Defined => "defined",
            Self::Ready => "ready",
            Self::Blocked => "blocked",
            Self::ClaimedDoneUnverified => "claimed-done-unverified",
            Self::DoneVerified => "done-verified",
        }
    }

    /// Parse the canonical wire string.
    pub fn parse(text: &str) -> Result<Self, PlaneError> {
        match text {
            "defined" => Ok(Self::Defined),
            "ready" => Ok(Self::Ready),
            "blocked" => Ok(Self::Blocked),
            "claimed-done-unverified" => Ok(Self::ClaimedDoneUnverified),
            "done-verified" => Ok(Self::DoneVerified),
            other => Err(PlaneError::Corrupt(format!(
                "unknown card status {other:?}"
            ))),
        }
    }
}

/// Why a lane is operationally blocked on a card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    /// Waiting on a human/founder decision.
    NeedsDecision,
    /// Waiting on another work item.
    NeedsDependency,
    /// Waiting on review capacity.
    NeedsReview,
    /// Waiting on infrastructure.
    NeedsInfra,
    /// Any other block; the run note carries detail.
    Other,
}

impl BlockKind {
    /// Canonical wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NeedsDecision => "needs-decision",
            Self::NeedsDependency => "needs-dependency",
            Self::NeedsReview => "needs-review",
            Self::NeedsInfra => "needs-infra",
            Self::Other => "other",
        }
    }

    /// Parse the canonical wire string.
    pub fn parse(text: &str) -> Result<Self, PlaneError> {
        match text {
            "needs-decision" => Ok(Self::NeedsDecision),
            "needs-dependency" => Ok(Self::NeedsDependency),
            "needs-review" => Ok(Self::NeedsReview),
            "needs-infra" => Ok(Self::NeedsInfra),
            "other" => Ok(Self::Other),
            other => Err(PlaneError::Corrupt(format!("unknown block kind {other:?}"))),
        }
    }
}

/// Operational run state of a claimed card on the execution plane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunState {
    /// Claimed, work not yet started.
    Claimed,
    /// Work in flight.
    Running,
    /// Operationally blocked with a typed block kind.
    Blocked(BlockKind),
    /// Claim released without completion.
    Released,
    /// Completion recorded (evidence lives on the coordination plane).
    Completed,
}

impl RunState {
    /// Canonical wire string (block kind is a sibling field).
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Claimed => "claimed",
            Self::Running => "running",
            Self::Blocked(_) => "blocked",
            Self::Released => "released",
            Self::Completed => "completed",
        }
    }
}

/// Durable loop card: the Definition/Seed card class projection of one
/// masterplan v2 work item onto the drive loop. `card_id` lives in the single
/// MPV2 work-item ID space; the card contract is runtime-agnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopCard {
    /// Canonical work-item id (single MPV2 ID space).
    pub card_id: String,
    /// Short human title.
    pub title: String,
    /// Owning program shard id (e.g. `P-FABRIC`).
    pub program_id: String,
    /// Dependency edges into the plan DAG (card ids that must be done-verified).
    pub depends_on: Vec<String>,
    /// Durable status.
    pub status: CardStatus,
    /// Evidence references backing done-class statuses.
    pub evidence_refs: Vec<String>,
}

/// Lane-exclusive claim on a card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimRecord {
    /// Claimed card.
    pub card_id: String,
    /// Claiming lane.
    pub lane_id: String,
    /// Claim time (unix seconds).
    pub claimed_at_epoch_s: u64,
}

/// Latest liveness heartbeat for a claimed card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeartbeatRecord {
    /// Claimed card.
    pub card_id: String,
    /// Claiming lane.
    pub lane_id: String,
    /// Beat time (unix seconds).
    pub beat_at_epoch_s: u64,
}

/// Latest run-state record for a card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRecord {
    /// The card.
    pub card_id: String,
    /// The lane that wrote the record.
    pub lane_id: String,
    /// Operational state.
    pub state: RunState,
    /// Free-form note (block detail, release reason, ...).
    pub note: String,
    /// Write time (unix seconds).
    pub updated_at_epoch_s: u64,
}

/// Raw per-card timeline observed during one dispatch pass, from which the
/// per-card flow metrics derive mechanically (no judgment).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardFlowTimeline {
    /// The measured card (single MPV2 ID space).
    pub card_id: String,
    /// The lane that drove the card.
    pub lane_id: String,
    /// When the lane claimed the card (unix seconds).
    pub claimed_at_epoch_s: u64,
    /// When review was requested (unix seconds).
    pub review_requested_at_epoch_s: u64,
    /// When the first review verdict landed (unix seconds).
    pub review_first_verdict_at_epoch_s: u64,
    /// When completion was recorded (unix seconds).
    pub completed_at_epoch_s: u64,
    /// Total review rounds driven (>= 1; round 1 is not rework).
    pub review_rounds: u64,
}

/// Per-card flow metrics measured within one dispatch pass (closed-loop
/// improvement layer): cycle time, review latency, and rework count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardFlowMetrics {
    /// The measured card.
    pub card_id: String,
    /// The lane that drove the card.
    pub lane_id: String,
    /// Claim -> completion wall time (seconds).
    pub cycle_time_s: u64,
    /// Review request -> first verdict wall time (seconds).
    pub review_latency_s: u64,
    /// Review rounds beyond the first (each extra round is one rework).
    pub rework_count: u64,
}

impl CardFlowMetrics {
    /// Derive the metrics from a raw timeline. Fails closed on inverted
    /// timestamps or a zero review-round count.
    pub fn derive(timeline: &CardFlowTimeline) -> Result<Self, PlaneError> {
        validate_id(&timeline.card_id)?;
        validate_id(&timeline.lane_id)?;
        let invalid = |detail: &str| PlaneError::InvalidMetrics {
            subject: timeline.card_id.clone(),
            detail: detail.to_owned(),
        };
        if timeline.review_rounds == 0 {
            return Err(invalid("review_rounds must be >= 1"));
        }
        if timeline.review_requested_at_epoch_s < timeline.claimed_at_epoch_s {
            return Err(invalid("review requested before claim"));
        }
        if timeline.review_first_verdict_at_epoch_s < timeline.review_requested_at_epoch_s {
            return Err(invalid("review verdict before review request"));
        }
        if timeline.completed_at_epoch_s < timeline.claimed_at_epoch_s {
            return Err(invalid("completion before claim"));
        }
        if timeline.completed_at_epoch_s < timeline.review_first_verdict_at_epoch_s {
            return Err(invalid("completion before first review verdict"));
        }
        Ok(Self {
            card_id: timeline.card_id.clone(),
            lane_id: timeline.lane_id.clone(),
            cycle_time_s: timeline.completed_at_epoch_s - timeline.claimed_at_epoch_s,
            review_latency_s: timeline.review_first_verdict_at_epoch_s
                - timeline.review_requested_at_epoch_s,
            rework_count: timeline.review_rounds - 1,
        })
    }
}

/// Flow metrics of one dispatch pass. Recorded on EVERY pass — including idle
/// passes with zero measured cards (constant-work: the recording load is the
/// same idle or peak) — as an append-only, strictly monotonic ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassFlowMetrics {
    /// 1-based, strictly increasing dispatch-pass sequence number.
    pub pass_seq: u64,
    /// Recording time (unix seconds).
    pub recorded_at_epoch_s: u64,
    /// Per-card metrics measured in this pass.
    pub cards: Vec<CardFlowMetrics>,
}

impl PassFlowMetrics {
    /// Number of cards measured in this pass.
    #[must_use]
    pub fn cards_measured(&self) -> u64 {
        u64::try_from(self.cards.len()).unwrap_or(u64::MAX)
    }

    /// Total rework count across the pass.
    #[must_use]
    pub fn total_rework_count(&self) -> u64 {
        self.cards.iter().map(|c| c.rework_count).sum()
    }

    /// Worst per-card cycle time in the pass (`None` when idle).
    #[must_use]
    pub fn max_cycle_time_s(&self) -> Option<u64> {
        self.cards.iter().map(|c| c.cycle_time_s).max()
    }

    /// Worst per-card review latency in the pass (`None` when idle).
    #[must_use]
    pub fn max_review_latency_s(&self) -> Option<u64> {
        self.cards.iter().map(|c| c.review_latency_s).max()
    }
}

/// Which plane a port serves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaneKind {
    /// Durable coordination plane.
    Coordination,
    /// Operational execution plane.
    Execution,
}

/// Durability class of a plane store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaneDurability {
    /// In-repo, PR-governed (coordinator-only commits).
    InRepoPrGoverned,
    /// Repo-adjacent, gitignored operational state.
    RepoAdjacentGitignored,
    /// The owned cloud-ci loop-state service (post-cutover).
    CloudCiService,
}

/// Self-description every port implementation must report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaneDescriptor {
    /// Which plane this port serves.
    pub plane: PlaneKind,
    /// Durability class of the backing store.
    pub durability: PlaneDurability,
    /// Store root (or service endpoint identity post-cutover).
    pub store_root: String,
}

// ---------------------------------------------------------------------------
// Ports — the ONLY loop-state read/write surface
// ---------------------------------------------------------------------------

/// Durable coordination-plane port. Models the owned cloud-ci-native
/// destination ([`CutoverTarget::canonical`]); the filesystem bridge adapter is
/// transient behind this trait.
pub trait CoordinationPlanePort {
    /// Persist a card (create or replace by `card_id`).
    fn put_card(&mut self, card: &LoopCard) -> Result<(), PlaneError>;
    /// Read one card by id.
    fn card(&self, card_id: &str) -> Result<Option<LoopCard>, PlaneError>;
    /// Read all cards, ordered by `card_id`.
    fn cards(&self) -> Result<Vec<LoopCard>, PlaneError>;
    /// Describe the backing store.
    fn descriptor(&self) -> PlaneDescriptor;
    /// The named owned cloud-ci cutover target this port models. The default
    /// is the canonical record; implementations MUST NOT diverge from it.
    fn cutover_target(&self) -> CutoverTarget {
        CutoverTarget::canonical()
    }
}

/// Operational execution-plane port (claim / heartbeat / run state). Models the
/// owned cloud-ci-native destination ([`CutoverTarget::canonical`]); the
/// gitignored filesystem bridge adapter is transient behind this trait.
pub trait ExecutionPlanePort {
    /// Take a lane-exclusive claim. Fails with [`PlaneError::AlreadyClaimed`]
    /// when another lane holds the card.
    fn claim(
        &mut self,
        card_id: &str,
        lane_id: &str,
        at_epoch_s: u64,
    ) -> Result<ClaimRecord, PlaneError>;
    /// Read the active claim, if any.
    fn active_claim(&self, card_id: &str) -> Result<Option<ClaimRecord>, PlaneError>;
    /// Record a liveness heartbeat; requires the claim to be held by `lane_id`.
    fn heartbeat(
        &mut self,
        card_id: &str,
        lane_id: &str,
        at_epoch_s: u64,
    ) -> Result<HeartbeatRecord, PlaneError>;
    /// Read the latest heartbeat, if any.
    fn last_heartbeat(&self, card_id: &str) -> Result<Option<HeartbeatRecord>, PlaneError>;
    /// Record a run-state transition; requires the claim to be held by the
    /// record's `lane_id`.
    fn record_run(&mut self, record: &RunRecord) -> Result<(), PlaneError>;
    /// Read the latest run record, if any.
    fn run_record(&self, card_id: &str) -> Result<Option<RunRecord>, PlaneError>;
    /// Drop the claim; requires the claim to be held by `lane_id`.
    fn release(&mut self, card_id: &str, lane_id: &str) -> Result<(), PlaneError>;
    /// Describe the backing store.
    fn descriptor(&self) -> PlaneDescriptor;
    /// The named owned cloud-ci cutover target this port models. The default
    /// is the canonical record; implementations MUST NOT diverge from it.
    fn cutover_target(&self) -> CutoverTarget {
        CutoverTarget::canonical()
    }
}

/// Per-pass flow-metrics port (closed-loop improvement layer). The metrics
/// ledger rides the durable coordination plane (in-repo, PR-governed,
/// coordinator-only commits) and models the same owned cloud-ci-native
/// destination ([`CutoverTarget::canonical`]); the filesystem bridge adapter
/// is transient behind this trait.
pub trait FlowMetricsPort {
    /// Append one dispatch-pass record. The ledger is append-only and
    /// strictly monotonic: `pass.pass_seq` MUST be greater than the latest
    /// recorded sequence, or the write fails with
    /// [`PlaneError::NonMonotonicPass`].
    fn record_pass(&mut self, pass: &PassFlowMetrics) -> Result<(), PlaneError>;
    /// Read one pass by sequence number.
    fn pass(&self, pass_seq: u64) -> Result<Option<PassFlowMetrics>, PlaneError>;
    /// Read all passes, ordered by `pass_seq`.
    fn passes(&self) -> Result<Vec<PassFlowMetrics>, PlaneError>;
    /// The latest recorded pass, if any.
    fn latest_pass(&self) -> Result<Option<PassFlowMetrics>, PlaneError> {
        Ok(self.passes()?.into_iter().next_back())
    }
    /// Describe the backing store.
    fn descriptor(&self) -> PlaneDescriptor;
    /// The named owned cloud-ci cutover target this port models. The default
    /// is the canonical record; implementations MUST NOT diverge from it.
    fn cutover_target(&self) -> CutoverTarget {
        CutoverTarget::canonical()
    }
}

/// Filesystem-safe id contract shared by card and lane ids: 1..=128 chars of
/// `[A-Za-z0-9._-]`, not starting with `.` (defends the bridge adapters
/// against path traversal and dotfile shadowing).
pub fn validate_id(id: &str) -> Result<(), PlaneError> {
    let ok_len = !id.is_empty() && id.len() <= 128;
    let ok_start = !id.starts_with('.');
    let ok_chars = id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-');
    if ok_len && ok_start && ok_chars {
        Ok(())
    } else {
        Err(PlaneError::InvalidId(id.to_owned()))
    }
}

fn ensure_owned(
    existing: Option<&ClaimRecord>,
    card_id: &str,
    lane_id: &str,
) -> Result<(), PlaneError> {
    match existing {
        None => Err(PlaneError::NotClaimed(card_id.to_owned())),
        Some(claim) if claim.lane_id == lane_id => Ok(()),
        Some(claim) => Err(PlaneError::WrongLane {
            card_id: card_id.to_owned(),
            holder_lane: claim.lane_id.clone(),
        }),
    }
}

/// Shared fail-closed validation of one pass record against the ledger head.
/// Every adapter calls this, so no write path can bypass the contract.
fn validate_pass_record(pass: &PassFlowMetrics, latest_recorded: u64) -> Result<(), PlaneError> {
    if pass.pass_seq == 0 || pass.pass_seq <= latest_recorded {
        return Err(PlaneError::NonMonotonicPass {
            pass_seq: pass.pass_seq,
            latest_recorded,
        });
    }
    let mut seen: Vec<&str> = Vec::with_capacity(pass.cards.len());
    for card in &pass.cards {
        validate_id(&card.card_id)?;
        validate_id(&card.lane_id)?;
        if seen.contains(&card.card_id.as_str()) {
            return Err(PlaneError::InvalidMetrics {
                subject: format!("pass-{}", pass.pass_seq),
                detail: format!("duplicate card {} in one pass", card.card_id),
            });
        }
        seen.push(&card.card_id);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Owned canonical JSON (persistence wire format; zero third-party deps)
// ---------------------------------------------------------------------------

/// Minimal owned JSON value for the loop-state wire format. Numbers are
/// restricted to unsigned integers (the only numeric shape this crate emits).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonValue {
    /// `null`
    Null,
    /// `true` / `false`
    Bool(bool),
    /// Unsigned integer.
    Num(u64),
    /// String.
    Str(String),
    /// Array.
    Arr(Vec<JsonValue>),
    /// Object with insertion-ordered keys.
    Obj(Vec<(String, JsonValue)>),
}

impl JsonValue {
    /// Object field lookup.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            Self::Obj(fields) => fields.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// String projection.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(s) => Some(s),
            _ => None,
        }
    }

    /// Unsigned-integer projection.
    #[must_use]
    pub fn as_num(&self) -> Option<u64> {
        match self {
            Self::Num(n) => Some(*n),
            _ => None,
        }
    }

    /// Array projection.
    #[must_use]
    pub fn as_arr(&self) -> Option<&[JsonValue]> {
        match self {
            Self::Arr(items) => Some(items),
            _ => None,
        }
    }

    /// Canonical 2-space-indented, LF, trailing-newline serialization
    /// (matches the repo canonical-JSON dialect: literal UTF-8, no key sort).
    #[must_use]
    pub fn to_canonical_string(&self) -> String {
        let mut out = String::new();
        write_json(self, &mut out, 0);
        out.push('\n');
        out
    }

    /// Parse a JSON document (full string must be consumed).
    pub fn parse(text: &str) -> Result<Self, PlaneError> {
        let mut parser = JsonParser {
            bytes: text.as_bytes(),
            pos: 0,
        };
        parser.skip_ws();
        let value = parser.value()?;
        parser.skip_ws();
        if parser.pos != parser.bytes.len() {
            return Err(PlaneError::Corrupt(format!(
                "trailing bytes at offset {}",
                parser.pos
            )));
        }
        Ok(value)
    }
}

fn write_json(value: &JsonValue, out: &mut String, indent: usize) {
    match value {
        JsonValue::Null => out.push_str("null"),
        JsonValue::Bool(true) => out.push_str("true"),
        JsonValue::Bool(false) => out.push_str("false"),
        JsonValue::Num(n) => out.push_str(&n.to_string()),
        JsonValue::Str(s) => write_json_string(s, out),
        JsonValue::Arr(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push('\n');
                push_indent(out, indent + 1);
                write_json(item, out, indent + 1);
            }
            out.push('\n');
            push_indent(out, indent);
            out.push(']');
        }
        JsonValue::Obj(fields) => {
            if fields.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push('{');
            for (i, (key, val)) in fields.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push('\n');
                push_indent(out, indent + 1);
                write_json_string(key, out);
                out.push_str(": ");
                write_json(val, out, indent + 1);
            }
            out.push('\n');
            push_indent(out, indent);
            out.push('}');
        }
    }
}

fn push_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push_str("  ");
    }
}

fn write_json_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

struct JsonParser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl JsonParser<'_> {
    fn skip_ws(&mut self) {
        while let Some(&b) = self.bytes.get(self.pos) {
            if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn corrupt(&self, what: &str) -> PlaneError {
        PlaneError::Corrupt(format!("{what} at offset {}", self.pos))
    }

    fn expect_literal(&mut self, literal: &str) -> Result<(), PlaneError> {
        if self.bytes[self.pos..].starts_with(literal.as_bytes()) {
            self.pos += literal.len();
            Ok(())
        } else {
            Err(self.corrupt("invalid literal"))
        }
    }

    fn value(&mut self) -> Result<JsonValue, PlaneError> {
        match self.peek() {
            Some(b'n') => {
                self.expect_literal("null")?;
                Ok(JsonValue::Null)
            }
            Some(b't') => {
                self.expect_literal("true")?;
                Ok(JsonValue::Bool(true))
            }
            Some(b'f') => {
                self.expect_literal("false")?;
                Ok(JsonValue::Bool(false))
            }
            Some(b'"') => Ok(JsonValue::Str(self.string()?)),
            Some(b'[') => self.array(),
            Some(b'{') => self.object(),
            Some(b'0'..=b'9') => self.number(),
            _ => Err(self.corrupt("unexpected byte")),
        }
    }

    fn number(&mut self) -> Result<JsonValue, PlaneError> {
        let start = self.pos;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        if matches!(self.peek(), Some(b'.' | b'e' | b'E' | b'-' | b'+')) {
            return Err(self.corrupt("non-integer number (wire format is u64-only)"));
        }
        let text = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| self.corrupt("invalid utf-8 in number"))?;
        text.parse::<u64>()
            .map(JsonValue::Num)
            .map_err(|_| self.corrupt("number out of u64 range"))
    }

    fn string(&mut self) -> Result<String, PlaneError> {
        if self.peek() != Some(b'"') {
            return Err(self.corrupt("expected string"));
        }
        self.pos += 1;
        let mut out = String::new();
        loop {
            let Some(b) = self.peek() else {
                return Err(self.corrupt("unterminated string"));
            };
            match b {
                b'"' => {
                    self.pos += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.pos += 1;
                    let Some(esc) = self.peek() else {
                        return Err(self.corrupt("unterminated escape"));
                    };
                    self.pos += 1;
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000C}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let unit = self.hex4()?;
                            if (0xD800..=0xDBFF).contains(&unit) {
                                self.expect_literal("\\u")?;
                                let low = self.hex4()?;
                                if !(0xDC00..=0xDFFF).contains(&low) {
                                    return Err(self.corrupt("invalid low surrogate"));
                                }
                                let code = 0x10000 + ((unit - 0xD800) << 10) + (low - 0xDC00);
                                let c = char::from_u32(code)
                                    .ok_or_else(|| self.corrupt("invalid surrogate pair"))?;
                                out.push(c);
                            } else {
                                let c = char::from_u32(unit)
                                    .ok_or_else(|| self.corrupt("invalid \\u escape"))?;
                                out.push(c);
                            }
                        }
                        _ => return Err(self.corrupt("unknown escape")),
                    }
                }
                _ => {
                    // Consume one UTF-8 scalar.
                    let rest = std::str::from_utf8(&self.bytes[self.pos..])
                        .map_err(|_| self.corrupt("invalid utf-8 in string"))?;
                    let Some(c) = rest.chars().next() else {
                        return Err(self.corrupt("unterminated string"));
                    };
                    if (c as u32) < 0x20 {
                        return Err(self.corrupt("raw control character in string"));
                    }
                    out.push(c);
                    self.pos += c.len_utf8();
                }
            }
        }
    }

    fn hex4(&mut self) -> Result<u32, PlaneError> {
        let end = self.pos + 4;
        if end > self.bytes.len() {
            return Err(self.corrupt("truncated \\u escape"));
        }
        let text = std::str::from_utf8(&self.bytes[self.pos..end])
            .map_err(|_| self.corrupt("invalid \\u escape"))?;
        let unit = u32::from_str_radix(text, 16).map_err(|_| self.corrupt("invalid \\u escape"))?;
        self.pos = end;
        Ok(unit)
    }

    fn array(&mut self) -> Result<JsonValue, PlaneError> {
        self.pos += 1; // consume '['
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(JsonValue::Arr(items));
        }
        loop {
            self.skip_ws();
            items.push(self.value()?);
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(JsonValue::Arr(items));
                }
                _ => return Err(self.corrupt("expected ',' or ']'")),
            }
        }
    }

    fn object(&mut self) -> Result<JsonValue, PlaneError> {
        self.pos += 1; // consume '{'
        let mut fields: Vec<(String, JsonValue)> = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(JsonValue::Obj(fields));
        }
        loop {
            self.skip_ws();
            let key = self.string()?;
            if fields.iter().any(|(k, _)| *k == key) {
                return Err(self.corrupt("duplicate object key"));
            }
            self.skip_ws();
            if self.peek() != Some(b':') {
                return Err(self.corrupt("expected ':'"));
            }
            self.pos += 1;
            self.skip_ws();
            let value = self.value()?;
            fields.push((key, value));
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(JsonValue::Obj(fields));
                }
                _ => return Err(self.corrupt("expected ',' or '}'")),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Record <-> JSON codecs
// ---------------------------------------------------------------------------

fn str_field(value: &JsonValue, key: &str) -> Result<String, PlaneError> {
    value
        .get(key)
        .and_then(JsonValue::as_str)
        .map(str::to_owned)
        .ok_or_else(|| PlaneError::Corrupt(format!("missing string field {key:?}")))
}

fn num_field(value: &JsonValue, key: &str) -> Result<u64, PlaneError> {
    value
        .get(key)
        .and_then(JsonValue::as_num)
        .ok_or_else(|| PlaneError::Corrupt(format!("missing u64 field {key:?}")))
}

fn str_array_field(value: &JsonValue, key: &str) -> Result<Vec<String>, PlaneError> {
    let items = value
        .get(key)
        .and_then(JsonValue::as_arr)
        .ok_or_else(|| PlaneError::Corrupt(format!("missing array field {key:?}")))?;
    items
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_owned)
                .ok_or_else(|| PlaneError::Corrupt(format!("non-string entry in {key:?}")))
        })
        .collect()
}

fn str_arr(values: &[String]) -> JsonValue {
    JsonValue::Arr(values.iter().cloned().map(JsonValue::Str).collect())
}

impl LoopCard {
    /// Canonical JSON projection.
    #[must_use]
    pub fn to_json(&self) -> JsonValue {
        JsonValue::Obj(vec![
            ("card_id".into(), JsonValue::Str(self.card_id.clone())),
            ("title".into(), JsonValue::Str(self.title.clone())),
            ("program_id".into(), JsonValue::Str(self.program_id.clone())),
            ("depends_on".into(), str_arr(&self.depends_on)),
            ("status".into(), JsonValue::Str(self.status.as_str().into())),
            ("evidence_refs".into(), str_arr(&self.evidence_refs)),
        ])
    }

    /// Decode the canonical JSON projection.
    pub fn from_json(value: &JsonValue) -> Result<Self, PlaneError> {
        Ok(Self {
            card_id: str_field(value, "card_id")?,
            title: str_field(value, "title")?,
            program_id: str_field(value, "program_id")?,
            depends_on: str_array_field(value, "depends_on")?,
            status: CardStatus::parse(&str_field(value, "status")?)?,
            evidence_refs: str_array_field(value, "evidence_refs")?,
        })
    }
}

impl ClaimRecord {
    fn to_json(&self) -> JsonValue {
        JsonValue::Obj(vec![
            ("card_id".into(), JsonValue::Str(self.card_id.clone())),
            ("lane_id".into(), JsonValue::Str(self.lane_id.clone())),
            (
                "claimed_at_epoch_s".into(),
                JsonValue::Num(self.claimed_at_epoch_s),
            ),
        ])
    }

    fn from_json(value: &JsonValue) -> Result<Self, PlaneError> {
        Ok(Self {
            card_id: str_field(value, "card_id")?,
            lane_id: str_field(value, "lane_id")?,
            claimed_at_epoch_s: num_field(value, "claimed_at_epoch_s")?,
        })
    }
}

impl HeartbeatRecord {
    fn to_json(&self) -> JsonValue {
        JsonValue::Obj(vec![
            ("card_id".into(), JsonValue::Str(self.card_id.clone())),
            ("lane_id".into(), JsonValue::Str(self.lane_id.clone())),
            (
                "beat_at_epoch_s".into(),
                JsonValue::Num(self.beat_at_epoch_s),
            ),
        ])
    }

    fn from_json(value: &JsonValue) -> Result<Self, PlaneError> {
        Ok(Self {
            card_id: str_field(value, "card_id")?,
            lane_id: str_field(value, "lane_id")?,
            beat_at_epoch_s: num_field(value, "beat_at_epoch_s")?,
        })
    }
}

impl RunRecord {
    fn to_json(&self) -> JsonValue {
        let mut fields = vec![
            ("card_id".into(), JsonValue::Str(self.card_id.clone())),
            ("lane_id".into(), JsonValue::Str(self.lane_id.clone())),
            ("state".into(), JsonValue::Str(self.state.as_str().into())),
        ];
        if let RunState::Blocked(kind) = &self.state {
            fields.push(("block_kind".into(), JsonValue::Str(kind.as_str().into())));
        }
        fields.push(("note".into(), JsonValue::Str(self.note.clone())));
        fields.push((
            "updated_at_epoch_s".into(),
            JsonValue::Num(self.updated_at_epoch_s),
        ));
        JsonValue::Obj(fields)
    }

    fn from_json(value: &JsonValue) -> Result<Self, PlaneError> {
        let state_text = str_field(value, "state")?;
        let state = match state_text.as_str() {
            "claimed" => RunState::Claimed,
            "running" => RunState::Running,
            "blocked" => RunState::Blocked(BlockKind::parse(&str_field(value, "block_kind")?)?),
            "released" => RunState::Released,
            "completed" => RunState::Completed,
            other => {
                return Err(PlaneError::Corrupt(format!("unknown run state {other:?}")));
            }
        };
        Ok(Self {
            card_id: str_field(value, "card_id")?,
            lane_id: str_field(value, "lane_id")?,
            state,
            note: str_field(value, "note")?,
            updated_at_epoch_s: num_field(value, "updated_at_epoch_s")?,
        })
    }
}

impl CardFlowMetrics {
    fn to_json(&self) -> JsonValue {
        JsonValue::Obj(vec![
            ("card_id".into(), JsonValue::Str(self.card_id.clone())),
            ("lane_id".into(), JsonValue::Str(self.lane_id.clone())),
            ("cycle_time_s".into(), JsonValue::Num(self.cycle_time_s)),
            (
                "review_latency_s".into(),
                JsonValue::Num(self.review_latency_s),
            ),
            ("rework_count".into(), JsonValue::Num(self.rework_count)),
        ])
    }

    fn from_json(value: &JsonValue) -> Result<Self, PlaneError> {
        Ok(Self {
            card_id: str_field(value, "card_id")?,
            lane_id: str_field(value, "lane_id")?,
            cycle_time_s: num_field(value, "cycle_time_s")?,
            review_latency_s: num_field(value, "review_latency_s")?,
            rework_count: num_field(value, "rework_count")?,
        })
    }
}

impl PassFlowMetrics {
    /// Canonical JSON projection.
    #[must_use]
    pub fn to_json(&self) -> JsonValue {
        JsonValue::Obj(vec![
            ("pass_seq".into(), JsonValue::Num(self.pass_seq)),
            (
                "recorded_at_epoch_s".into(),
                JsonValue::Num(self.recorded_at_epoch_s),
            ),
            (
                "cards".into(),
                JsonValue::Arr(self.cards.iter().map(CardFlowMetrics::to_json).collect()),
            ),
        ])
    }

    /// Decode the canonical JSON projection.
    pub fn from_json(value: &JsonValue) -> Result<Self, PlaneError> {
        let cards = value
            .get("cards")
            .and_then(JsonValue::as_arr)
            .ok_or_else(|| PlaneError::Corrupt("missing array field \"cards\"".into()))?
            .iter()
            .map(CardFlowMetrics::from_json)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            pass_seq: num_field(value, "pass_seq")?,
            recorded_at_epoch_s: num_field(value, "recorded_at_epoch_s")?,
            cards,
        })
    }
}

// ---------------------------------------------------------------------------
// In-memory adapters (test doubles / spy bases)
// ---------------------------------------------------------------------------

/// In-memory coordination-plane adapter (test double).
#[derive(Debug, Default)]
pub struct InMemoryCoordinationStore {
    cards: BTreeMap<String, LoopCard>,
}

impl InMemoryCoordinationStore {
    /// Empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl CoordinationPlanePort for InMemoryCoordinationStore {
    fn put_card(&mut self, card: &LoopCard) -> Result<(), PlaneError> {
        validate_id(&card.card_id)?;
        self.cards.insert(card.card_id.clone(), card.clone());
        Ok(())
    }

    fn card(&self, card_id: &str) -> Result<Option<LoopCard>, PlaneError> {
        validate_id(card_id)?;
        Ok(self.cards.get(card_id).cloned())
    }

    fn cards(&self) -> Result<Vec<LoopCard>, PlaneError> {
        Ok(self.cards.values().cloned().collect())
    }

    fn descriptor(&self) -> PlaneDescriptor {
        PlaneDescriptor {
            plane: PlaneKind::Coordination,
            durability: PlaneDurability::InRepoPrGoverned,
            store_root: "<in-memory>".into(),
        }
    }
}

/// In-memory execution-plane adapter (test double).
#[derive(Debug, Default)]
pub struct InMemoryExecutionStore {
    claims: BTreeMap<String, ClaimRecord>,
    heartbeats: BTreeMap<String, HeartbeatRecord>,
    runs: BTreeMap<String, RunRecord>,
}

impl InMemoryExecutionStore {
    /// Empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl ExecutionPlanePort for InMemoryExecutionStore {
    fn claim(
        &mut self,
        card_id: &str,
        lane_id: &str,
        at_epoch_s: u64,
    ) -> Result<ClaimRecord, PlaneError> {
        validate_id(card_id)?;
        validate_id(lane_id)?;
        if let Some(existing) = self.claims.get(card_id) {
            return Err(PlaneError::AlreadyClaimed {
                card_id: card_id.to_owned(),
                holder_lane: existing.lane_id.clone(),
            });
        }
        let claim = ClaimRecord {
            card_id: card_id.to_owned(),
            lane_id: lane_id.to_owned(),
            claimed_at_epoch_s: at_epoch_s,
        };
        self.claims.insert(card_id.to_owned(), claim.clone());
        Ok(claim)
    }

    fn active_claim(&self, card_id: &str) -> Result<Option<ClaimRecord>, PlaneError> {
        validate_id(card_id)?;
        Ok(self.claims.get(card_id).cloned())
    }

    fn heartbeat(
        &mut self,
        card_id: &str,
        lane_id: &str,
        at_epoch_s: u64,
    ) -> Result<HeartbeatRecord, PlaneError> {
        validate_id(card_id)?;
        validate_id(lane_id)?;
        ensure_owned(self.claims.get(card_id), card_id, lane_id)?;
        let beat = HeartbeatRecord {
            card_id: card_id.to_owned(),
            lane_id: lane_id.to_owned(),
            beat_at_epoch_s: at_epoch_s,
        };
        self.heartbeats.insert(card_id.to_owned(), beat.clone());
        Ok(beat)
    }

    fn last_heartbeat(&self, card_id: &str) -> Result<Option<HeartbeatRecord>, PlaneError> {
        validate_id(card_id)?;
        Ok(self.heartbeats.get(card_id).cloned())
    }

    fn record_run(&mut self, record: &RunRecord) -> Result<(), PlaneError> {
        validate_id(&record.card_id)?;
        validate_id(&record.lane_id)?;
        ensure_owned(
            self.claims.get(&record.card_id),
            &record.card_id,
            &record.lane_id,
        )?;
        self.runs.insert(record.card_id.clone(), record.clone());
        Ok(())
    }

    fn run_record(&self, card_id: &str) -> Result<Option<RunRecord>, PlaneError> {
        validate_id(card_id)?;
        Ok(self.runs.get(card_id).cloned())
    }

    fn release(&mut self, card_id: &str, lane_id: &str) -> Result<(), PlaneError> {
        validate_id(card_id)?;
        validate_id(lane_id)?;
        ensure_owned(self.claims.get(card_id), card_id, lane_id)?;
        self.claims.remove(card_id);
        Ok(())
    }

    fn descriptor(&self) -> PlaneDescriptor {
        PlaneDescriptor {
            plane: PlaneKind::Execution,
            durability: PlaneDurability::RepoAdjacentGitignored,
            store_root: "<in-memory>".into(),
        }
    }
}

/// In-memory flow-metrics adapter (test double).
#[derive(Debug, Default)]
pub struct InMemoryFlowMetricsStore {
    passes: BTreeMap<u64, PassFlowMetrics>,
}

impl InMemoryFlowMetricsStore {
    /// Empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl FlowMetricsPort for InMemoryFlowMetricsStore {
    fn record_pass(&mut self, pass: &PassFlowMetrics) -> Result<(), PlaneError> {
        let latest = self.passes.keys().next_back().copied().unwrap_or(0);
        validate_pass_record(pass, latest)?;
        self.passes.insert(pass.pass_seq, pass.clone());
        Ok(())
    }

    fn pass(&self, pass_seq: u64) -> Result<Option<PassFlowMetrics>, PlaneError> {
        Ok(self.passes.get(&pass_seq).cloned())
    }

    fn passes(&self) -> Result<Vec<PassFlowMetrics>, PlaneError> {
        Ok(self.passes.values().cloned().collect())
    }

    fn descriptor(&self) -> PlaneDescriptor {
        PlaneDescriptor {
            plane: PlaneKind::Coordination,
            durability: PlaneDurability::InRepoPrGoverned,
            store_root: "<in-memory>".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Filesystem bridge adapters (retirement-marked local bridges)
// ---------------------------------------------------------------------------

fn io_err(context: &str, err: &std::io::Error) -> PlaneError {
    PlaneError::Io(format!("{context}: {err}"))
}

fn write_atomic(path: &Path, content: &str) -> Result<(), PlaneError> {
    let parent = path
        .parent()
        .ok_or_else(|| PlaneError::Io(format!("no parent for {}", path.display())))?;
    fs::create_dir_all(parent).map_err(|e| io_err("create_dir_all", &e))?;
    let tmp = path.with_extension(format!("json.tmp.{}", std::process::id()));
    {
        let mut file = fs::File::create(&tmp).map_err(|e| io_err("create tmp", &e))?;
        file.write_all(content.as_bytes())
            .map_err(|e| io_err("write tmp", &e))?;
        file.sync_all().map_err(|e| io_err("sync tmp", &e))?;
    }
    fs::rename(&tmp, path).map_err(|e| io_err("rename tmp", &e))
}

fn read_record(path: &Path) -> Result<Option<JsonValue>, PlaneError> {
    match fs::read_to_string(path) {
        Ok(text) => JsonValue::parse(&text).map(Some),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(io_err("read", &err)),
    }
}

/// Recognize a leftover of [`write_atomic`]'s own tmp protocol
/// (`<name>.json.tmp.<pid>`). Such a file is a crash/concurrency artifact of
/// the atomic writer itself: its content was never acknowledged (the rename
/// never happened), so tolerating it keeps crash recovery and concurrent
/// listing working. Everything else foreign fails the listing closed.
fn is_atomic_write_tmp(name: &str) -> bool {
    match name.rsplit_once(".json.tmp.") {
        Some((stem, pid)) => {
            !stem.is_empty() && !pid.is_empty() && pid.bytes().all(|b| b.is_ascii_digit())
        }
        None => false,
    }
}

/// Strict fail-closed ledger listing: a directory entry is admitted iff its
/// name is `<id>.json` with `id` passing [`validate_id`] AND the entry is a
/// regular file (directly, or through a symlink that resolves to a regular
/// file). A non-UTF-8 name, a `.json` name whose id fails the contract
/// (unicode digits, spaces, dotfiles), or any other foreign name surfaces as
/// [`PlaneError::Corrupt`] — never silently skipped (a ledger the lister
/// cannot fully reason about is never admitted as clean). The file-type rule
/// applies only after a name passes the id contract: a name-admitted entry
/// that is a directory, a dangling symlink, or a symlink to a non-file also
/// surfaces as [`PlaneError::Corrupt`] instead of degrading later into a
/// silent drop (dangling symlink read as `NotFound`) or an environmental
/// [`PlaneError::Io`] (`EISDIR` on read). The single carve-out is the atomic
/// writer's own tmp shape (see [`is_atomic_write_tmp`]).
fn list_json_ids(dir: &Path) -> Result<Vec<String>, PlaneError> {
    let mut ids = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(ids),
        Err(err) => return Err(io_err("read_dir", &err)),
    };
    for entry in entries {
        let entry = entry.map_err(|e| io_err("read_dir entry", &e))?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            return Err(PlaneError::Corrupt(format!(
                "non-UTF-8 filename in ledger directory {}: {}",
                dir.display(),
                name.to_string_lossy()
            )));
        };
        if let Some(id) = name.strip_suffix(".json")
            && validate_id(id).is_ok()
        {
            require_regular_file(dir, &entry, name)?;
            ids.push(id.to_owned());
            continue;
        }
        if is_atomic_write_tmp(name) {
            continue;
        }
        return Err(PlaneError::Corrupt(format!(
            "foreign file in ledger directory {}: {name}",
            dir.display()
        )));
    }
    ids.sort();
    Ok(ids)
}

/// Fail-closed file-type admission for a name-admitted ledger entry. Only a
/// regular file (directly, or via a symlink resolving to a regular file) is
/// admitted: a valid symlink to a regular file reads identically through
/// [`read_record`]. Every other shape is a ledger-structure violation and
/// surfaces as [`PlaneError::Corrupt`] naming the directory and entry — a
/// dangling symlink would otherwise be silently dropped by the readers
/// (`NotFound` maps to `Ok(None)`) and a directory would degrade into a
/// misleading environmental [`PlaneError::Io`] (`EISDIR`). [`PlaneError::Io`]
/// is reserved for genuine environmental read failures while inspecting the
/// entry.
fn require_regular_file(dir: &Path, entry: &fs::DirEntry, name: &str) -> Result<(), PlaneError> {
    // `DirEntry::file_type` does NOT follow symlinks, so the symlink case is
    // distinguished here before any resolution.
    let file_type = entry
        .file_type()
        .map_err(|e| io_err("entry file_type", &e))?;
    if file_type.is_file() {
        return Ok(());
    }
    if file_type.is_symlink() {
        // `fs::metadata` follows symlinks: `NotFound` here means the link
        // dangles (the entry itself was just enumerated, so it exists).
        return match fs::metadata(entry.path()) {
            Ok(meta) if meta.is_file() => Ok(()),
            Ok(_) => Err(PlaneError::Corrupt(format!(
                "symlink to non-file in ledger directory {}: {name}",
                dir.display()
            ))),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                Err(PlaneError::Corrupt(format!(
                    "dangling symlink in ledger directory {}: {name}",
                    dir.display()
                )))
            }
            Err(err) => Err(io_err("resolve symlink target", &err)),
        };
    }
    let shape = if file_type.is_dir() {
        "directory"
    } else {
        "non-regular file"
    };
    Err(PlaneError::Corrupt(format!(
        "{shape} in ledger directory {}: {name}",
        dir.display()
    )))
}

/// Filesystem bridge adapter for the durable coordination plane. LOCAL BRIDGE
/// ONLY (bridge-then-retire per the port contract spec): the destination is
/// the owned cloud-ci loop-state service named by [`CutoverTarget::canonical`].
/// The store root defaults to the in-repo, PR-governed
/// [`DEFAULT_DURABLE_PLANE_ROOT`].
#[derive(Debug)]
pub struct FsCoordinationStore {
    root: PathBuf,
}

impl FsCoordinationStore {
    /// Open (lazily creating on first write) a durable-plane store at `root`.
    #[must_use]
    pub fn open(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn card_path(&self, card_id: &str) -> PathBuf {
        self.root.join("cards").join(format!("{card_id}.json"))
    }
}

impl CoordinationPlanePort for FsCoordinationStore {
    fn put_card(&mut self, card: &LoopCard) -> Result<(), PlaneError> {
        validate_id(&card.card_id)?;
        write_atomic(
            &self.card_path(&card.card_id),
            &card.to_json().to_canonical_string(),
        )
    }

    fn card(&self, card_id: &str) -> Result<Option<LoopCard>, PlaneError> {
        validate_id(card_id)?;
        match read_record(&self.card_path(card_id))? {
            Some(value) => Ok(Some(LoopCard::from_json(&value)?)),
            None => Ok(None),
        }
    }

    fn cards(&self) -> Result<Vec<LoopCard>, PlaneError> {
        let mut cards = Vec::new();
        for id in list_json_ids(&self.root.join("cards"))? {
            if let Some(card) = self.card(&id)? {
                cards.push(card);
            }
        }
        Ok(cards)
    }

    fn descriptor(&self) -> PlaneDescriptor {
        PlaneDescriptor {
            plane: PlaneKind::Coordination,
            durability: PlaneDurability::InRepoPrGoverned,
            store_root: self.root.display().to_string(),
        }
    }
}

/// Filesystem bridge adapter for the operational execution plane. LOCAL BRIDGE
/// ONLY (bridge-then-retire per the port contract spec): the destination is
/// the owned cloud-ci loop-state service named by [`CutoverTarget::canonical`].
/// The store root defaults to the repo-adjacent, gitignored
/// [`DEFAULT_OPERATIONAL_PLANE_ROOT`].
#[derive(Debug)]
pub struct FsExecutionStore {
    root: PathBuf,
}

impl FsExecutionStore {
    /// Open (lazily creating on first write) an operational-plane store at `root`.
    #[must_use]
    pub fn open(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn claim_path(&self, card_id: &str) -> PathBuf {
        self.root.join("claims").join(format!("{card_id}.json"))
    }

    fn heartbeat_path(&self, card_id: &str) -> PathBuf {
        self.root.join("heartbeats").join(format!("{card_id}.json"))
    }

    fn run_path(&self, card_id: &str) -> PathBuf {
        self.root.join("runs").join(format!("{card_id}.json"))
    }

    fn claim_record(&self, card_id: &str) -> Result<Option<ClaimRecord>, PlaneError> {
        match read_record(&self.claim_path(card_id))? {
            Some(value) => Ok(Some(ClaimRecord::from_json(&value)?)),
            None => Ok(None),
        }
    }
}

impl ExecutionPlanePort for FsExecutionStore {
    fn claim(
        &mut self,
        card_id: &str,
        lane_id: &str,
        at_epoch_s: u64,
    ) -> Result<ClaimRecord, PlaneError> {
        validate_id(card_id)?;
        validate_id(lane_id)?;
        if let Some(existing) = self.claim_record(card_id)? {
            return Err(PlaneError::AlreadyClaimed {
                card_id: card_id.to_owned(),
                holder_lane: existing.lane_id,
            });
        }
        let claim = ClaimRecord {
            card_id: card_id.to_owned(),
            lane_id: lane_id.to_owned(),
            claimed_at_epoch_s: at_epoch_s,
        };
        write_atomic(
            &self.claim_path(card_id),
            &claim.to_json().to_canonical_string(),
        )?;
        Ok(claim)
    }

    fn active_claim(&self, card_id: &str) -> Result<Option<ClaimRecord>, PlaneError> {
        validate_id(card_id)?;
        self.claim_record(card_id)
    }

    fn heartbeat(
        &mut self,
        card_id: &str,
        lane_id: &str,
        at_epoch_s: u64,
    ) -> Result<HeartbeatRecord, PlaneError> {
        validate_id(card_id)?;
        validate_id(lane_id)?;
        let claim = self.claim_record(card_id)?;
        ensure_owned(claim.as_ref(), card_id, lane_id)?;
        let beat = HeartbeatRecord {
            card_id: card_id.to_owned(),
            lane_id: lane_id.to_owned(),
            beat_at_epoch_s: at_epoch_s,
        };
        write_atomic(
            &self.heartbeat_path(card_id),
            &beat.to_json().to_canonical_string(),
        )?;
        Ok(beat)
    }

    fn last_heartbeat(&self, card_id: &str) -> Result<Option<HeartbeatRecord>, PlaneError> {
        validate_id(card_id)?;
        match read_record(&self.heartbeat_path(card_id))? {
            Some(value) => Ok(Some(HeartbeatRecord::from_json(&value)?)),
            None => Ok(None),
        }
    }

    fn record_run(&mut self, record: &RunRecord) -> Result<(), PlaneError> {
        validate_id(&record.card_id)?;
        validate_id(&record.lane_id)?;
        let claim = self.claim_record(&record.card_id)?;
        ensure_owned(claim.as_ref(), &record.card_id, &record.lane_id)?;
        write_atomic(
            &self.run_path(&record.card_id),
            &record.to_json().to_canonical_string(),
        )
    }

    fn run_record(&self, card_id: &str) -> Result<Option<RunRecord>, PlaneError> {
        validate_id(card_id)?;
        match read_record(&self.run_path(card_id))? {
            Some(value) => Ok(Some(RunRecord::from_json(&value)?)),
            None => Ok(None),
        }
    }

    fn release(&mut self, card_id: &str, lane_id: &str) -> Result<(), PlaneError> {
        validate_id(card_id)?;
        validate_id(lane_id)?;
        let claim = self.claim_record(card_id)?;
        ensure_owned(claim.as_ref(), card_id, lane_id)?;
        fs::remove_file(self.claim_path(card_id)).map_err(|e| io_err("remove claim", &e))
    }

    fn descriptor(&self) -> PlaneDescriptor {
        PlaneDescriptor {
            plane: PlaneKind::Execution,
            durability: PlaneDurability::RepoAdjacentGitignored,
            store_root: self.root.display().to_string(),
        }
    }
}

/// Filesystem bridge adapter for the per-pass flow-metrics ledger. LOCAL
/// BRIDGE ONLY (bridge-then-retire per the port contract spec): the
/// destination is the owned cloud-ci loop-state service named by
/// [`CutoverTarget::canonical`]. The store root defaults to the in-repo,
/// PR-governed [`DEFAULT_FLOW_METRICS_ROOT`] (durable coordination plane).
#[derive(Debug)]
pub struct FsFlowMetricsStore {
    root: PathBuf,
}

impl FsFlowMetricsStore {
    /// Open (lazily creating on first write) a flow-metrics store at `root`.
    #[must_use]
    pub fn open(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn pass_path(&self, pass_seq: u64) -> PathBuf {
        self.root
            .join("passes")
            .join(format!("pass-{pass_seq:020}.json"))
    }

    fn pass_seqs(&self) -> Result<Vec<u64>, PlaneError> {
        let mut seqs = Vec::new();
        for id in list_json_ids(&self.root.join("passes"))? {
            let Some(seq) = id.strip_prefix("pass-").and_then(|s| s.parse::<u64>().ok()) else {
                // Fail closed: a foreign or corrupt filename in the ledger
                // directory is surfaced, never silently ignored.
                return Err(PlaneError::Corrupt(format!(
                    "foreign file in flow-metrics ledger: {id}.json"
                )));
            };
            // Fail closed: only the canonical zero-padded 20-digit filename
            // (`pass-{seq:020}.json`, exactly as written by `pass_path`) may
            // influence the ledger head. A non-canonical numeric name (e.g.
            // `pass-7.json`) would bump the monotonic head while its content
            // stays invisible to `pass()` / `passes()`.
            if format!("pass-{seq:020}") != id {
                return Err(PlaneError::Corrupt(format!(
                    "non-canonical pass filename in flow-metrics ledger: {id}.json"
                )));
            }
            seqs.push(seq);
        }
        seqs.sort_unstable();
        Ok(seqs)
    }
}

impl FlowMetricsPort for FsFlowMetricsStore {
    fn record_pass(&mut self, pass: &PassFlowMetrics) -> Result<(), PlaneError> {
        let latest = self.pass_seqs()?.last().copied().unwrap_or(0);
        validate_pass_record(pass, latest)?;
        write_atomic(
            &self.pass_path(pass.pass_seq),
            &pass.to_json().to_canonical_string(),
        )
    }

    fn pass(&self, pass_seq: u64) -> Result<Option<PassFlowMetrics>, PlaneError> {
        match read_record(&self.pass_path(pass_seq))? {
            Some(value) => Ok(Some(PassFlowMetrics::from_json(&value)?)),
            None => Ok(None),
        }
    }

    fn passes(&self) -> Result<Vec<PassFlowMetrics>, PlaneError> {
        let mut passes = Vec::new();
        for seq in self.pass_seqs()? {
            if let Some(pass) = self.pass(seq)? {
                passes.push(pass);
            }
        }
        Ok(passes)
    }

    fn descriptor(&self) -> PlaneDescriptor {
        PlaneDescriptor {
            plane: PlaneKind::Coordination,
            durability: PlaneDurability::InRepoPrGoverned,
            store_root: self.root.display().to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Single-writer facade — no I/O of its own; ports are the only path
// ---------------------------------------------------------------------------

/// The single-writer loop-state facade. Generic over the two plane ports and
/// free of any I/O of its own, so EVERY loop-state read and write crosses
/// [`CoordinationPlanePort`] / [`ExecutionPlanePort`] by construction.
#[derive(Debug)]
pub struct LoopStateService<C: CoordinationPlanePort, E: ExecutionPlanePort> {
    coordination: C,
    execution: E,
}

impl<C: CoordinationPlanePort, E: ExecutionPlanePort> LoopStateService<C, E> {
    /// Compose the facade over the two plane ports.
    pub fn new(coordination: C, execution: E) -> Self {
        Self {
            coordination,
            execution,
        }
    }

    /// The cutover targets both ports model. Both MUST equal
    /// [`CutoverTarget::canonical`]; the port-level contract test enforces it.
    pub fn cutover_targets(&self) -> (CutoverTarget, CutoverTarget) {
        (
            self.coordination.cutover_target(),
            self.execution.cutover_target(),
        )
    }

    /// Plane descriptors of both backing stores.
    pub fn plane_descriptors(&self) -> (PlaneDescriptor, PlaneDescriptor) {
        (self.coordination.descriptor(), self.execution.descriptor())
    }

    /// Define a new durable card (Definition/Seed card class). Refuses
    /// duplicate ids (single work-item ID space) and done-class initial
    /// statuses (status claims require the evidence-carrying transitions).
    pub fn define_card(&mut self, card: &LoopCard) -> Result<(), PlaneError> {
        validate_id(&card.card_id)?;
        if self.coordination.card(&card.card_id)?.is_some() {
            return Err(PlaneError::DuplicateCard(card.card_id.clone()));
        }
        if matches!(
            card.status,
            CardStatus::ClaimedDoneUnverified | CardStatus::DoneVerified
        ) {
            return Err(PlaneError::InvalidTransition {
                card_id: card.card_id.clone(),
                detail: "cards cannot be defined in a done-class status".into(),
            });
        }
        self.coordination.put_card(card)
    }

    /// Read one durable card.
    pub fn card(&self, card_id: &str) -> Result<Option<LoopCard>, PlaneError> {
        self.coordination.card(card_id)
    }

    /// Read all durable cards.
    pub fn cards(&self) -> Result<Vec<LoopCard>, PlaneError> {
        self.coordination.cards()
    }

    /// Latest heartbeat for a card.
    pub fn last_heartbeat(&self, card_id: &str) -> Result<Option<HeartbeatRecord>, PlaneError> {
        self.execution.last_heartbeat(card_id)
    }

    /// Latest run record for a card.
    pub fn run_record(&self, card_id: &str) -> Result<Option<RunRecord>, PlaneError> {
        self.execution.run_record(card_id)
    }

    /// Active claim for a card.
    pub fn active_claim(&self, card_id: &str) -> Result<Option<ClaimRecord>, PlaneError> {
        self.execution.active_claim(card_id)
    }

    /// Dependency ids of `card` that are not yet done-verified.
    fn missing_dependencies(&self, card: &LoopCard) -> Result<Vec<String>, PlaneError> {
        let mut missing = Vec::new();
        for dep in &card.depends_on {
            let satisfied = matches!(
                self.coordination.card(dep)?.map(|c| c.status),
                Some(CardStatus::DoneVerified)
            );
            if !satisfied {
                missing.push(dep.clone());
            }
        }
        Ok(missing)
    }

    /// Ready work derives from the plan DAG: a card is ready iff its durable
    /// status is `defined`/`ready`, every dependency is done-verified, and no
    /// lane holds an active claim.
    pub fn ready_cards(&self) -> Result<Vec<LoopCard>, PlaneError> {
        let mut ready = Vec::new();
        for card in self.coordination.cards()? {
            if !matches!(card.status, CardStatus::Defined | CardStatus::Ready) {
                continue;
            }
            if !self.missing_dependencies(&card)?.is_empty() {
                continue;
            }
            if self.execution.active_claim(&card.card_id)?.is_some() {
                continue;
            }
            ready.push(card);
        }
        Ok(ready)
    }

    /// Claim a DAG-ready card for a lane and record the initial run state.
    pub fn claim_ready(
        &mut self,
        card_id: &str,
        lane_id: &str,
        at_epoch_s: u64,
    ) -> Result<ClaimRecord, PlaneError> {
        let card = self
            .coordination
            .card(card_id)?
            .ok_or_else(|| PlaneError::UnknownCard(card_id.to_owned()))?;
        if !matches!(card.status, CardStatus::Defined | CardStatus::Ready) {
            return Err(PlaneError::InvalidTransition {
                card_id: card_id.to_owned(),
                detail: format!("status {} is not claimable", card.status.as_str()),
            });
        }
        let missing = self.missing_dependencies(&card)?;
        if !missing.is_empty() {
            return Err(PlaneError::DependencyUnsatisfied {
                card_id: card_id.to_owned(),
                missing,
            });
        }
        let claim = self.execution.claim(card_id, lane_id, at_epoch_s)?;
        self.execution.record_run(&RunRecord {
            card_id: card_id.to_owned(),
            lane_id: lane_id.to_owned(),
            state: RunState::Claimed,
            note: String::new(),
            updated_at_epoch_s: at_epoch_s,
        })?;
        Ok(claim)
    }

    /// Record a liveness heartbeat for a claimed card.
    pub fn heartbeat(
        &mut self,
        card_id: &str,
        lane_id: &str,
        at_epoch_s: u64,
    ) -> Result<HeartbeatRecord, PlaneError> {
        self.execution.heartbeat(card_id, lane_id, at_epoch_s)
    }

    /// Mark the claimed card's run as started.
    pub fn start_run(
        &mut self,
        card_id: &str,
        lane_id: &str,
        at_epoch_s: u64,
    ) -> Result<(), PlaneError> {
        self.execution.record_run(&RunRecord {
            card_id: card_id.to_owned(),
            lane_id: lane_id.to_owned(),
            state: RunState::Running,
            note: String::new(),
            updated_at_epoch_s: at_epoch_s,
        })
    }

    /// Mark the claimed card as operationally blocked with a typed block kind.
    pub fn mark_blocked(
        &mut self,
        card_id: &str,
        lane_id: &str,
        kind: BlockKind,
        note: &str,
        at_epoch_s: u64,
    ) -> Result<(), PlaneError> {
        self.execution.record_run(&RunRecord {
            card_id: card_id.to_owned(),
            lane_id: lane_id.to_owned(),
            state: RunState::Blocked(kind),
            note: note.to_owned(),
            updated_at_epoch_s: at_epoch_s,
        })
    }

    /// Release a claim without completion (the run record keeps the reason).
    pub fn abandon(
        &mut self,
        card_id: &str,
        lane_id: &str,
        note: &str,
        at_epoch_s: u64,
    ) -> Result<(), PlaneError> {
        self.execution.record_run(&RunRecord {
            card_id: card_id.to_owned(),
            lane_id: lane_id.to_owned(),
            state: RunState::Released,
            note: note.to_owned(),
            updated_at_epoch_s: at_epoch_s,
        })?;
        self.execution.release(card_id, lane_id)
    }

    /// Complete a claimed card. Evidence discipline: at least one evidence
    /// reference is REQUIRED, and the durable status becomes
    /// `claimed-done-unverified` (never `done-verified`) — verification is a
    /// separate evidence-attaching step ([`Self::verify_done`]).
    pub fn complete(
        &mut self,
        card_id: &str,
        lane_id: &str,
        evidence_refs: &[String],
        at_epoch_s: u64,
    ) -> Result<(), PlaneError> {
        if evidence_refs.iter().all(|r| r.trim().is_empty()) {
            return Err(PlaneError::MissingEvidence(card_id.to_owned()));
        }
        let mut card = self
            .coordination
            .card(card_id)?
            .ok_or_else(|| PlaneError::UnknownCard(card_id.to_owned()))?;
        self.execution.record_run(&RunRecord {
            card_id: card_id.to_owned(),
            lane_id: lane_id.to_owned(),
            state: RunState::Completed,
            note: String::new(),
            updated_at_epoch_s: at_epoch_s,
        })?;
        self.execution.release(card_id, lane_id)?;
        card.status = CardStatus::ClaimedDoneUnverified;
        for evidence in evidence_refs {
            if !evidence.trim().is_empty() && !card.evidence_refs.contains(evidence) {
                card.evidence_refs.push(evidence.clone());
            }
        }
        self.coordination.put_card(&card)
    }

    /// Promote a `claimed-done-unverified` card to `done-verified`, attaching
    /// the verification evidence reference.
    pub fn verify_done(
        &mut self,
        card_id: &str,
        verification_evidence: &str,
    ) -> Result<(), PlaneError> {
        if verification_evidence.trim().is_empty() {
            return Err(PlaneError::MissingEvidence(card_id.to_owned()));
        }
        let mut card = self
            .coordination
            .card(card_id)?
            .ok_or_else(|| PlaneError::UnknownCard(card_id.to_owned()))?;
        if card.status != CardStatus::ClaimedDoneUnverified {
            return Err(PlaneError::InvalidTransition {
                card_id: card_id.to_owned(),
                detail: format!(
                    "only claimed-done-unverified cards verify; status is {}",
                    card.status.as_str()
                ),
            });
        }
        card.status = CardStatus::DoneVerified;
        if !card
            .evidence_refs
            .contains(&verification_evidence.to_owned())
        {
            card.evidence_refs.push(verification_evidence.to_owned());
        }
        self.coordination.put_card(&card)
    }
}

// ---------------------------------------------------------------------------
// Flow-metrics facade — closed-loop improvement layer, single writer
// ---------------------------------------------------------------------------

/// Single-writer facade over the per-pass flow-metrics ledger. Generic over
/// [`FlowMetricsPort`] and free of any I/O of its own, so every metric read
/// and write crosses the port by construction. The coordinator records one
/// [`PassFlowMetrics`] on EVERY dispatch pass (closed-loop improvement layer
/// of the ADR-0516 fabric drive loop).
#[derive(Debug)]
pub struct FlowMetricsService<M: FlowMetricsPort> {
    metrics: M,
}

impl<M: FlowMetricsPort> FlowMetricsService<M> {
    /// Compose the facade over the metrics port.
    pub fn new(metrics: M) -> Self {
        Self { metrics }
    }

    /// The cutover target the port models (MUST equal
    /// [`CutoverTarget::canonical`]).
    pub fn cutover_target(&self) -> CutoverTarget {
        self.metrics.cutover_target()
    }

    /// Descriptor of the backing store.
    pub fn descriptor(&self) -> PlaneDescriptor {
        self.metrics.descriptor()
    }

    /// Record an explicit pass record (append-only, strictly monotonic).
    pub fn record_pass(&mut self, pass: &PassFlowMetrics) -> Result<(), PlaneError> {
        self.metrics.record_pass(pass)
    }

    /// Record the next pass in sequence: derives per-card metrics from the
    /// raw timelines mechanically and allocates `latest + 1` as the pass
    /// sequence. Idle passes (no timelines) are still recorded.
    pub fn record_next_pass(
        &mut self,
        timelines: &[CardFlowTimeline],
        recorded_at_epoch_s: u64,
    ) -> Result<PassFlowMetrics, PlaneError> {
        let cards = timelines
            .iter()
            .map(CardFlowMetrics::derive)
            .collect::<Result<Vec<_>, _>>()?;
        let pass_seq = self
            .metrics
            .latest_pass()?
            .map_or(1, |latest| latest.pass_seq + 1);
        let pass = PassFlowMetrics {
            pass_seq,
            recorded_at_epoch_s,
            cards,
        };
        self.metrics.record_pass(&pass)?;
        Ok(pass)
    }

    /// Read one pass by sequence number.
    pub fn pass(&self, pass_seq: u64) -> Result<Option<PassFlowMetrics>, PlaneError> {
        self.metrics.pass(pass_seq)
    }

    /// Read all passes, ordered by sequence.
    pub fn passes(&self) -> Result<Vec<PassFlowMetrics>, PlaneError> {
        self.metrics.passes()
    }

    /// The latest recorded pass, if any.
    pub fn latest_pass(&self) -> Result<Option<PassFlowMetrics>, PlaneError> {
        self.metrics.latest_pass()
    }
}

// ---------------------------------------------------------------------------
// Mechanical lane-disjointness detector (path/ownership-overlap)
// ---------------------------------------------------------------------------

/// Detector phase: parallel safety is verdicted MECHANICALLY twice per
/// dispatch — pre-flight over the DECLARED lane work surfaces (before any
/// lane runs) and post-run over the ACTUAL touched paths (proving zero
/// collisions after the fact). Judgment never decides what runs concurrently;
/// this detector does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisjointnessPhase {
    /// Declared surfaces, before dispatch. An overlap verdict here routes the
    /// colliding lanes to the serialized integrator lane instead of parallel
    /// dispatch.
    PreFlight,
    /// Actual touched paths, after the lanes ran. An overlap verdict here is
    /// a collision escape (dispatch bug): the run must not merge as parallel.
    PostRun,
}

impl DisjointnessPhase {
    /// Canonical wire string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PreFlight => "pre-flight",
            Self::PostRun => "post-run",
        }
    }

    /// Parse the canonical wire string.
    pub fn parse(text: &str) -> Result<Self, PlaneError> {
        match text {
            "pre-flight" => Ok(Self::PreFlight),
            "post-run" => Ok(Self::PostRun),
            other => Err(PlaneError::Corrupt(format!(
                "unknown disjointness phase {other:?}"
            ))),
        }
    }
}

/// One lane's work surface: the repo-relative paths (files or directory
/// roots) the lane declares it will touch (pre-flight) or actually touched
/// (post-run) while driving one card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneWorkSurface {
    /// The lane.
    pub lane_id: String,
    /// The card the lane drives (one card per lane per pass).
    pub card_id: String,
    /// Repo-relative paths: no leading `/`, no `.`/`..` components, no
    /// backslashes, no trailing `/`. A directory path claims its whole
    /// subtree.
    pub paths: Vec<String>,
}

/// One mechanical path collision between two lanes: equal paths or one path
/// inside the other's claimed subtree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathCollision {
    /// First lane (in surface order).
    pub lane_a: String,
    /// Colliding path declared/touched by `lane_a`.
    pub path_a: String,
    /// Second lane.
    pub lane_b: String,
    /// Colliding path declared/touched by `lane_b`.
    pub path_b: String,
}

/// The mechanical verdict. There is no judgment outcome: either every pair of
/// lane surfaces is path-disjoint, or the colliding work is routed to the
/// serialized integrator lane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisjointnessVerdict {
    /// Zero collisions: the lanes may run (pre-flight) / ran (post-run) in
    /// parallel.
    Disjoint,
    /// Overlap detected: the colliding lanes MUST NOT run in parallel;
    /// shared-surface work auto-routes to the serialized integrator lane.
    OverlapSerializeToIntegrator {
        /// Every colliding path pair, deterministically ordered.
        collisions: Vec<PathCollision>,
    },
}

impl DisjointnessVerdict {
    /// Canonical wire string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Disjoint => "disjoint",
            Self::OverlapSerializeToIntegrator { .. } => "overlap-serialize-to-integrator",
        }
    }
}

/// One detector run: phase, inputs, and verdict — the evidence-record shape
/// captured pre-flight and post-run for every parallel dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisjointnessReport {
    /// Which phase produced this report.
    pub phase: DisjointnessPhase,
    /// The checked surfaces (embedded so the report is self-contained
    /// evidence).
    pub surfaces: Vec<LaneWorkSurface>,
    /// Available independent reviewer capacity the lane count was checked
    /// against.
    pub reviewer_capacity: usize,
    /// When the detector ran (unix seconds).
    pub checked_at_epoch_s: u64,
    /// The mechanical verdict.
    pub verdict: DisjointnessVerdict,
}

impl DisjointnessReport {
    /// Canonical JSON projection (evidence-record shape).
    #[must_use]
    pub fn to_json(&self) -> JsonValue {
        let surfaces = self
            .surfaces
            .iter()
            .map(|surface| {
                JsonValue::Obj(vec![
                    ("lane_id".into(), JsonValue::Str(surface.lane_id.clone())),
                    ("card_id".into(), JsonValue::Str(surface.card_id.clone())),
                    (
                        "paths".into(),
                        JsonValue::Arr(
                            surface
                                .paths
                                .iter()
                                .map(|p| JsonValue::Str(p.clone()))
                                .collect(),
                        ),
                    ),
                ])
            })
            .collect();
        let collisions = match &self.verdict {
            DisjointnessVerdict::Disjoint => Vec::new(),
            DisjointnessVerdict::OverlapSerializeToIntegrator { collisions } => collisions
                .iter()
                .map(|c| {
                    JsonValue::Obj(vec![
                        ("lane_a".into(), JsonValue::Str(c.lane_a.clone())),
                        ("path_a".into(), JsonValue::Str(c.path_a.clone())),
                        ("lane_b".into(), JsonValue::Str(c.lane_b.clone())),
                        ("path_b".into(), JsonValue::Str(c.path_b.clone())),
                    ])
                })
                .collect(),
        };
        JsonValue::Obj(vec![
            ("phase".into(), JsonValue::Str(self.phase.as_str().into())),
            (
                "verdict".into(),
                JsonValue::Str(self.verdict.as_str().into()),
            ),
            (
                "lane_count".into(),
                JsonValue::Num(self.surfaces.len() as u64),
            ),
            (
                "reviewer_capacity".into(),
                JsonValue::Num(self.reviewer_capacity as u64),
            ),
            (
                "checked_at_epoch_s".into(),
                JsonValue::Num(self.checked_at_epoch_s),
            ),
            ("surfaces".into(), JsonValue::Arr(surfaces)),
            ("collisions".into(), JsonValue::Arr(collisions)),
        ])
    }
}

/// Validate one repo-relative surface path. Fail closed: absolute paths,
/// traversal components, backslashes, empty components, and trailing slashes
/// are refused — a path the detector cannot reason about must never be
/// admitted as "disjoint".
fn validate_surface_path(lane_id: &str, path: &str) -> Result<(), PlaneError> {
    let refuse = |detail: String| PlaneError::InvalidSurface {
        lane_id: lane_id.to_owned(),
        detail,
    };
    if path.is_empty() {
        return Err(refuse("empty path".into()));
    }
    if path.starts_with('/') {
        return Err(refuse(format!("absolute path {path:?}")));
    }
    if path.contains('\\') {
        return Err(refuse(format!("backslash in path {path:?}")));
    }
    if path.ends_with('/') {
        return Err(refuse(format!("trailing slash in path {path:?}")));
    }
    for component in path.split('/') {
        if component.is_empty() {
            return Err(refuse(format!("empty component in path {path:?}")));
        }
        if component == "." || component == ".." {
            return Err(refuse(format!("traversal component in path {path:?}")));
        }
    }
    Ok(())
}

/// True iff `a` and `b` collide: equal, or one is a directory-prefix of the
/// other (component-wise, so `src/lib.rs` does NOT collide with
/// `src/lib.rs.bak` but `src` collides with `src/lib.rs`).
fn paths_collide(a: &str, b: &str) -> bool {
    a == b
        || a.strip_prefix(b).is_some_and(|rest| rest.starts_with('/'))
        || b.strip_prefix(a).is_some_and(|rest| rest.starts_with('/'))
}

/// The mechanical path/ownership-overlap detector. Parallel safety is decided
/// here, not by judgment: ready work may dispatch concurrently ONLY when this
/// returns a [`DisjointnessVerdict::Disjoint`] report, and the same check
/// re-verdicts the actual touched paths post-run.
///
/// Fail-closed contract:
/// - every lane/card id passes [`validate_id`]; duplicate lane ids and
///   duplicate card ids across lanes are refused;
/// - every path passes the repo-relative shape check; an empty surface, an
///   empty path list, or zero surfaces is refused (nothing to reason about);
/// - `surfaces.len()` above `reviewer_capacity` is refused
///   ([`PlaneError::LaneCapacityExceeded`]): max concurrent lanes <=
///   available independent reviewer capacity, computed from the lane
///   registry by the caller.
pub fn check_lane_disjointness(
    phase: DisjointnessPhase,
    surfaces: &[LaneWorkSurface],
    reviewer_capacity: usize,
    checked_at_epoch_s: u64,
) -> Result<DisjointnessReport, PlaneError> {
    if surfaces.is_empty() {
        return Err(PlaneError::InvalidSurface {
            lane_id: "<batch>".into(),
            detail: "no lane work surfaces to check".into(),
        });
    }
    if surfaces.len() > reviewer_capacity {
        return Err(PlaneError::LaneCapacityExceeded {
            lanes: surfaces.len(),
            reviewer_capacity,
        });
    }
    let mut seen_lanes: Vec<&str> = Vec::new();
    let mut seen_cards: Vec<&str> = Vec::new();
    for surface in surfaces {
        validate_id(&surface.lane_id)?;
        validate_id(&surface.card_id)?;
        if seen_lanes.contains(&surface.lane_id.as_str()) {
            return Err(PlaneError::InvalidSurface {
                lane_id: "<batch>".into(),
                detail: format!("duplicate lane id {}", surface.lane_id),
            });
        }
        if seen_cards.contains(&surface.card_id.as_str()) {
            return Err(PlaneError::InvalidSurface {
                lane_id: "<batch>".into(),
                detail: format!("card {} appears in more than one lane", surface.card_id),
            });
        }
        seen_lanes.push(&surface.lane_id);
        seen_cards.push(&surface.card_id);
        if surface.paths.is_empty() {
            return Err(PlaneError::InvalidSurface {
                lane_id: surface.lane_id.clone(),
                detail: "empty path set".into(),
            });
        }
        for path in &surface.paths {
            validate_surface_path(&surface.lane_id, path)?;
        }
    }
    let mut collisions = Vec::new();
    for (i, a) in surfaces.iter().enumerate() {
        for b in &surfaces[i + 1..] {
            for path_a in &a.paths {
                for path_b in &b.paths {
                    if paths_collide(path_a, path_b) {
                        collisions.push(PathCollision {
                            lane_a: a.lane_id.clone(),
                            path_a: path_a.clone(),
                            lane_b: b.lane_id.clone(),
                            path_b: path_b.clone(),
                        });
                    }
                }
            }
        }
    }
    let verdict = if collisions.is_empty() {
        DisjointnessVerdict::Disjoint
    } else {
        DisjointnessVerdict::OverlapSerializeToIntegrator { collisions }
    };
    Ok(DisjointnessReport {
        phase,
        surfaces: surfaces.to_vec(),
        reviewer_capacity,
        checked_at_epoch_s,
        verdict,
    })
}

// ---------------------------------------------------------------------------
// Founder-ratification dispatch gate (masterplan v2 sequencing)
// ---------------------------------------------------------------------------

/// Repo-relative path of the single live plan authority the dispatch gate
/// reads. Sequencing ratification binds to the content at this path only.
pub const MASTERPLAN_REPO_PATH: &str = "specs/masterplan.json";

/// The pinned digest recipe shared with
/// `masterplan_v2.sequencing.sequencing_identity.hash_recipe` and the
/// cross-artifact-agreement gate. [`compute_sequencing_digest`] is the
/// executable form of this sentence; the two must never drift.
pub const SEQUENCING_DIGEST_RECIPE: &str = "sha256 over canonical JSON (sorted keys, separators ',' ':') of {\"dependency_edges\": sorted [from, to] pairs, \"execution_waves\": wave work_item_ids arrays in wave order, \"work_item_order\": work_item_id strings in index order}";

/// Mechanical refusal of execution-wave dispatch by the founder-ratification
/// gate. Every variant is fail-closed: dispatch surfaces MUST refuse to run
/// when this is returned, and there is no override flag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RatificationRefusal {
    /// `specs/masterplan.json` is missing, unreadable, or not parseable —
    /// the gate cannot establish sequencing identity, so dispatch is refused.
    MasterplanUnreadable(String),
    /// The sequencing content exists but is structurally malformed (missing
    /// or ill-typed `dependency_edges`, `execution_waves`,
    /// `work_item_order`, or `sequencing_identity`), or the embedded
    /// identity hash disagrees with the recomputed digest.
    SequencingMalformed(String),
    /// The founder-ratification record is absent or not a recorded, ratified
    /// decision (`decision_recorded != true`, `decision_status !=
    /// "ratified"`, or no `ratified_sequencing_digest`).
    RecordAbsent(String),
    /// The ratified digest no longer binds to the digest recomputed from the
    /// live sequencing content: the content mutated after ratification, which
    /// voids the decision (masterplan v2 `invalidation_rule`).
    StaleDigest {
        /// The digest the founder ratified.
        ratified_sequencing_digest: String,
        /// The digest recomputed from the live sequencing content.
        computed_sequencing_digest: String,
    },
    /// The ratified sequencing version does not equal the live
    /// `sequencing_identity.sequencing_version` (or is missing from the
    /// record): the record and the identity ledger disagree.
    StaleVersion {
        /// The version the record claims was ratified (`None` when absent).
        ratified_sequencing_version: Option<u64>,
        /// The live embedded sequencing version.
        current_sequencing_version: u64,
    },
}

impl fmt::Display for RatificationRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "founder-ratification gate: ")?;
        match self {
            Self::MasterplanUnreadable(detail) => {
                write!(f, "masterplan unreadable: {detail}")?;
            }
            Self::SequencingMalformed(detail) => {
                write!(f, "sequencing content malformed: {detail}")?;
            }
            Self::RecordAbsent(detail) => {
                write!(f, "ratification record absent: {detail}")?;
            }
            Self::StaleDigest {
                ratified_sequencing_digest,
                computed_sequencing_digest,
            } => {
                write!(
                    f,
                    "stale ratification: ratified_sequencing_digest {ratified_sequencing_digest} does not match the digest {computed_sequencing_digest} recomputed from the live sequencing content; the ratified content mutated, so a fresh re-derivation and a fresh founder-ratification record are required"
                )?;
            }
            Self::StaleVersion {
                ratified_sequencing_version,
                current_sequencing_version,
            } => {
                write!(
                    f,
                    "stale ratification: ratified_sequencing_version {ratified_sequencing_version:?} does not match live sequencing_identity.sequencing_version {current_sequencing_version}"
                )?;
            }
        }
        write!(f, "; execution-wave dispatch stays fail-closed")
    }
}

impl std::error::Error for RatificationRefusal {}

/// Positive gate verdict: the founder-ratification record binds to the live
/// sequencing content of `specs/masterplan.json`. Dispatch surfaces may run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RatificationGrant {
    /// The digest both ratified and recomputed from live content.
    pub sequencing_digest: String,
    /// The bound sequencing version.
    pub sequencing_version: u64,
    /// The durable decision-record reference, when the record carries one.
    pub decision_ref: Option<String>,
}

/// Compute the stable sequencing content digest for `masterplan_v2` — the
/// executable form of [`SEQUENCING_DIGEST_RECIPE`], byte-identical to the
/// cross-artifact-agreement gate's `compute_masterplan_v2_sequencing_digest`.
///
/// The digest covers exactly the ratifiable sequencing content: the
/// dependency DAG edges (as sorted `[from, to]` pairs), the execution-wave
/// `work_item_ids` arrays in wave order, and the `work_item_order` ids in
/// index order. Identity metadata, ratification records, and dispatch state
/// are deliberately OUTSIDE the hash scope so recording a ratification never
/// invalidates itself.
///
/// Returns `None` when the sequencing content is structurally malformed;
/// callers fail closed on `None`.
#[must_use]
pub fn compute_sequencing_digest(masterplan_v2: &JsonValue) -> Option<String> {
    let edges = masterplan_v2.get("dependency_edges")?.as_arr()?;
    let sequencing = masterplan_v2.get("sequencing")?;

    let mut edge_pairs = Vec::with_capacity(edges.len());
    for edge in edges {
        let from = edge.get("from")?.as_str()?;
        let to = edge.get("to")?.as_str()?;
        edge_pairs.push((from.to_owned(), to.to_owned()));
    }
    edge_pairs.sort();

    let mut waves = Vec::new();
    for wave in sequencing.get("execution_waves")?.as_arr()? {
        let ids = wave.get("work_item_ids")?.as_arr()?;
        let mut wave_ids = Vec::with_capacity(ids.len());
        for id in ids {
            wave_ids.push(id.as_str()?.to_owned());
        }
        waves.push(wave_ids);
    }

    let mut order = Vec::new();
    for entry in sequencing.get("work_item_order")?.as_arr()? {
        order.push(entry.get("work_item_id")?.as_str()?.to_owned());
    }

    // Canonical JSON is built by hand so the byte stream is deterministic:
    // object keys in sorted order, compact ','/':' separators, standard JSON
    // string escaping (identical to the cross-artifact gate's byte stream).
    let mut canonical = String::from("{\"dependency_edges\":[");
    for (index, (from, to)) in edge_pairs.iter().enumerate() {
        if index > 0 {
            canonical.push(',');
        }
        canonical.push('[');
        push_compact_json_string(from, &mut canonical);
        canonical.push(',');
        push_compact_json_string(to, &mut canonical);
        canonical.push(']');
    }
    canonical.push_str("],\"execution_waves\":[");
    for (index, wave_ids) in waves.iter().enumerate() {
        if index > 0 {
            canonical.push(',');
        }
        canonical.push('[');
        for (id_index, id) in wave_ids.iter().enumerate() {
            if id_index > 0 {
                canonical.push(',');
            }
            push_compact_json_string(id, &mut canonical);
        }
        canonical.push(']');
    }
    canonical.push_str("],\"work_item_order\":[");
    for (index, id) in order.iter().enumerate() {
        if index > 0 {
            canonical.push(',');
        }
        push_compact_json_string(id, &mut canonical);
    }
    canonical.push_str("]}");

    Some(format!("sha256:{}", sha256_hex(canonical.as_bytes())))
}

/// Serialize one string as a compact JSON string literal with serde_json's
/// escaping rules (`\b`/`\f`/`\n`/`\r`/`\t` short escapes, `\u00xx` lowercase
/// hex for other control characters), so the canonical byte stream is
/// identical to the sha2-based cross-artifact gate's.
fn push_compact_json_string(value: &str, out: &mut String) {
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000C}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// The mechanical founder-ratification gate over a masterplan document.
///
/// Fail-closed contract (Sub-AC: execution-wave dispatch surfaces refuse to
/// run unless every check passes):
/// 1. the document parses and `masterplan_v2.sequencing` exists;
/// 2. the sequencing content hashes under the pinned recipe;
/// 3. a founder-ratification record is present and recorded
///    (`decision_recorded == true`, `decision_status == "ratified"`,
///    `ratified_sequencing_digest` present) — else [`RatificationRefusal::RecordAbsent`];
/// 4. the ratified digest equals the digest recomputed from the LIVE content —
///    else [`RatificationRefusal::StaleDigest`];
/// 5. the ratified version equals the live embedded
///    `sequencing_identity.sequencing_version` — else
///    [`RatificationRefusal::StaleVersion`];
/// 6. the embedded `sequencing_identity.sequencing_hash` also equals the
///    recomputed digest (identity-ledger drift fails closed).
pub fn check_dispatch_ratification(
    masterplan_text: &str,
) -> Result<RatificationGrant, RatificationRefusal> {
    let doc = JsonValue::parse(masterplan_text)
        .map_err(|err| RatificationRefusal::MasterplanUnreadable(err.to_string()))?;
    let Some(masterplan_v2) = doc.get("masterplan_v2") else {
        return Err(RatificationRefusal::SequencingMalformed(
            "masterplan_v2 is missing".into(),
        ));
    };
    let Some(sequencing) = masterplan_v2.get("sequencing") else {
        return Err(RatificationRefusal::SequencingMalformed(
            "masterplan_v2.sequencing is missing".into(),
        ));
    };
    let computed = compute_sequencing_digest(masterplan_v2).ok_or_else(|| {
        RatificationRefusal::SequencingMalformed(
            "sequencing content cannot be hashed (missing or ill-typed dependency_edges, execution_waves, or work_item_order)".into(),
        )
    })?;
    let identity = sequencing.get("sequencing_identity");
    let Some(current_version) = identity
        .and_then(|identity| identity.get("sequencing_version"))
        .and_then(JsonValue::as_num)
        .filter(|version| *version >= 1)
    else {
        return Err(RatificationRefusal::SequencingMalformed(
            "sequencing_identity.sequencing_version is missing or not a positive integer".into(),
        ));
    };

    let Some(record) = sequencing.get("founder_ratification") else {
        return Err(RatificationRefusal::RecordAbsent(
            "masterplan_v2.sequencing.founder_ratification is missing".into(),
        ));
    };
    if record.get("decision_recorded") != Some(&JsonValue::Bool(true)) {
        return Err(RatificationRefusal::RecordAbsent(
            "founder_ratification.decision_recorded is not true".into(),
        ));
    }
    if record.get("decision_status").and_then(JsonValue::as_str) != Some("ratified") {
        return Err(RatificationRefusal::RecordAbsent(
            "founder_ratification.decision_status is not \"ratified\"".into(),
        ));
    }
    let Some(ratified_digest) = record
        .get("ratified_sequencing_digest")
        .and_then(JsonValue::as_str)
    else {
        return Err(RatificationRefusal::RecordAbsent(
            "founder_ratification.ratified_sequencing_digest is missing".into(),
        ));
    };
    if ratified_digest != computed {
        return Err(RatificationRefusal::StaleDigest {
            ratified_sequencing_digest: ratified_digest.to_owned(),
            computed_sequencing_digest: computed,
        });
    }
    let ratified_version = record
        .get("ratified_sequencing_version")
        .and_then(JsonValue::as_num);
    if ratified_version != Some(current_version) {
        return Err(RatificationRefusal::StaleVersion {
            ratified_sequencing_version: ratified_version,
            current_sequencing_version: current_version,
        });
    }
    let embedded_hash = identity
        .and_then(|identity| identity.get("sequencing_hash"))
        .and_then(JsonValue::as_str);
    if embedded_hash != Some(computed.as_str()) {
        return Err(RatificationRefusal::SequencingMalformed(format!(
            "sequencing_identity.sequencing_hash {embedded_hash:?} does not match the digest {computed} recomputed from live sequencing content"
        )));
    }
    Ok(RatificationGrant {
        sequencing_digest: computed,
        sequencing_version: current_version,
        decision_ref: record
            .get("decision_ref")
            .and_then(JsonValue::as_str)
            .map(str::to_owned),
    })
}

/// [`check_dispatch_ratification`] over the masterplan at
/// `<repo_root>/specs/masterplan.json`. A missing or unreadable file is a
/// [`RatificationRefusal::MasterplanUnreadable`] refusal — dispatch surfaces
/// with no reachable plan authority fail closed, never open.
pub fn check_dispatch_ratification_at(
    repo_root: &Path,
) -> Result<RatificationGrant, RatificationRefusal> {
    let path = repo_root.join(MASTERPLAN_REPO_PATH);
    let text = fs::read_to_string(&path).map_err(|err| {
        RatificationRefusal::MasterplanUnreadable(format!("{}: {err}", path.display()))
    })?;
    check_dispatch_ratification(&text)
}

// ---------------------------------------------------------------------------
// Owned SHA-256 (FIPS 180-4; zero third-party deps)
// ---------------------------------------------------------------------------

const SHA256_K: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

/// Owned SHA-256 (FIPS 180-4), returned as lowercase hex. Kept dependency-free
/// on purpose: this crate's bar is zero third-party deps, and the digest is
/// verified byte-for-byte against the sha2-backed cross-artifact gate through
/// the live-masterplan binding test in `tests/ratification_gate.rs`.
#[must_use]
pub fn sha256_hex(data: &[u8]) -> String {
    let mut state: [u32; 8] = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut message = data.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());

    for block in message.chunks_exact(64) {
        let mut w = [0_u32; 64];
        for (i, word) in block.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        for (slot, word) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = slot.wrapping_add(word);
        }
    }

    let mut hex = String::with_capacity(64);
    for word in state {
        hex.push_str(&format!("{word:08x}"));
    }
    hex
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn card(id: &str, deps: &[&str]) -> LoopCard {
        LoopCard {
            card_id: id.into(),
            title: format!("card {id}"),
            program_id: "P-FABRIC".into(),
            depends_on: deps.iter().map(|d| (*d).to_owned()).collect(),
            status: CardStatus::Ready,
            evidence_refs: Vec::new(),
        }
    }

    #[test]
    fn json_round_trips_records() {
        let original = LoopCard {
            card_id: "MPV2-0001".into(),
            title: "two-plane loop state — \"ports\"\n".into(),
            program_id: "P-FABRIC".into(),
            depends_on: vec!["MPV2-0000".into()],
            status: CardStatus::ClaimedDoneUnverified,
            evidence_refs: vec!["evidence/goals/x.json".into()],
        };
        let text = original.to_json().to_canonical_string();
        let parsed = LoopCard::from_json(&JsonValue::parse(&text).unwrap()).unwrap();
        assert_eq!(parsed, original);

        let run = RunRecord {
            card_id: "MPV2-0001".into(),
            lane_id: "lane-a".into(),
            state: RunState::Blocked(BlockKind::NeedsReview),
            note: "waiting on reviewer capacity".into(),
            updated_at_epoch_s: 1_780_000_000,
        };
        let text = run.to_json().to_canonical_string();
        let parsed = RunRecord::from_json(&JsonValue::parse(&text).unwrap()).unwrap();
        assert_eq!(parsed, run);
    }

    #[test]
    fn json_parser_rejects_duplicates_floats_and_traversal_ids() {
        assert!(JsonValue::parse("{\"a\": 1, \"a\": 2}").is_err());
        assert!(JsonValue::parse("{\"a\": 1.5}").is_err());
        assert!(JsonValue::parse("[1, 2,]").is_err());
        assert!(validate_id("../escape").is_err());
        assert!(validate_id(".hidden").is_err());
        assert!(validate_id("a/b").is_err());
        assert!(validate_id("").is_err());
        assert!(validate_id("MPV2-0001").is_ok());
    }

    #[test]
    fn json_parser_handles_escapes_and_surrogates() {
        let parsed = JsonValue::parse("\"a\\u00e9\\ud83d\\ude00\\n\"").unwrap();
        assert_eq!(parsed.as_str(), Some("aé😀\n"));
    }

    #[test]
    fn in_memory_lifecycle_enforces_claims_and_evidence() {
        let mut service = LoopStateService::new(
            InMemoryCoordinationStore::new(),
            InMemoryExecutionStore::new(),
        );
        service.define_card(&card("dep", &[])).unwrap();
        service.define_card(&card("work", &["dep"])).unwrap();

        // Duplicate id refused (single ID space).
        assert!(matches!(
            service.define_card(&card("work", &[])),
            Err(PlaneError::DuplicateCard(_))
        ));

        // Not ready: dep is not done-verified.
        assert!(matches!(
            service.claim_ready("work", "lane-a", 1),
            Err(PlaneError::DependencyUnsatisfied { .. })
        ));

        // Drive dep through the lifecycle.
        service.claim_ready("dep", "lane-a", 2).unwrap();
        assert!(matches!(
            service.claim_ready("dep", "lane-b", 3),
            Err(PlaneError::AlreadyClaimed { .. })
        ));
        service.heartbeat("dep", "lane-a", 4).unwrap();
        assert!(matches!(
            service.heartbeat("dep", "lane-b", 5),
            Err(PlaneError::WrongLane { .. })
        ));
        service.start_run("dep", "lane-a", 6).unwrap();
        assert!(matches!(
            service.complete("dep", "lane-a", &[], 7),
            Err(PlaneError::MissingEvidence(_))
        ));
        service
            .complete("dep", "lane-a", &["evidence/goals/dep.json".into()], 8)
            .unwrap();
        assert_eq!(
            service.card("dep").unwrap().unwrap().status,
            CardStatus::ClaimedDoneUnverified
        );

        // claimed-done-unverified does NOT satisfy the DAG.
        assert!(matches!(
            service.claim_ready("work", "lane-a", 9),
            Err(PlaneError::DependencyUnsatisfied { .. })
        ));
        service
            .verify_done("dep", "evidence/goals/dep-verify.json")
            .unwrap();
        assert_eq!(
            service.card("dep").unwrap().unwrap().status,
            CardStatus::DoneVerified
        );

        // Now the DAG admits the dependent card.
        let ready: Vec<String> = service
            .ready_cards()
            .unwrap()
            .into_iter()
            .map(|c| c.card_id)
            .collect();
        assert_eq!(ready, vec!["work".to_owned()]);
        service.claim_ready("work", "lane-b", 10).unwrap();
        service
            .mark_blocked("work", "lane-b", BlockKind::NeedsReview, "review queue", 11)
            .unwrap();
        assert!(matches!(
            service.run_record("work").unwrap().unwrap().state,
            RunState::Blocked(BlockKind::NeedsReview)
        ));
        service
            .abandon("work", "lane-b", "handing back", 12)
            .unwrap();
        assert!(service.active_claim("work").unwrap().is_none());
    }

    #[test]
    fn cutover_target_constants_are_the_canonical_record() {
        let target = CutoverTarget::canonical();
        assert_eq!(target.service_name, "oya-cloud-ci-loop-state-service");
        assert_eq!(target.owner, "cloud/cloud-ci");
        assert_eq!(
            target.destination_home,
            "cloud/cloud-ci/crates/oya-cloud-ci-loop-state-app"
        );
        assert_eq!(target.criteria.len(), 3);
    }

    fn timeline(card_id: &str, base: u64, rounds: u64) -> CardFlowTimeline {
        CardFlowTimeline {
            card_id: card_id.into(),
            lane_id: "lane-a".into(),
            claimed_at_epoch_s: base,
            review_requested_at_epoch_s: base + 100,
            review_first_verdict_at_epoch_s: base + 130,
            completed_at_epoch_s: base + 200,
            review_rounds: rounds,
        }
    }

    #[test]
    fn flow_metrics_derive_computes_cycle_review_latency_and_rework() {
        let metrics = CardFlowMetrics::derive(&timeline("MPV2-0001", 1_000, 3)).unwrap();
        assert_eq!(metrics.cycle_time_s, 200);
        assert_eq!(metrics.review_latency_s, 30);
        assert_eq!(metrics.rework_count, 2);

        // Fail closed: inverted timelines and zero review rounds are refused.
        let mut inverted = timeline("MPV2-0002", 1_000, 1);
        inverted.completed_at_epoch_s = 900;
        assert!(matches!(
            CardFlowMetrics::derive(&inverted),
            Err(PlaneError::InvalidMetrics { .. })
        ));
        let mut verdict_first = timeline("MPV2-0003", 1_000, 1);
        verdict_first.review_first_verdict_at_epoch_s = 1_050;
        assert!(matches!(
            CardFlowMetrics::derive(&verdict_first),
            Err(PlaneError::InvalidMetrics { .. })
        ));
        assert!(matches!(
            CardFlowMetrics::derive(&timeline("MPV2-0004", 1_000, 0)),
            Err(PlaneError::InvalidMetrics { .. })
        ));
        let mut completed_before_verdict = timeline("MPV2-0005", 1_000, 1);
        completed_before_verdict.completed_at_epoch_s = 1_120; // first verdict at 1_130
        assert!(matches!(
            CardFlowMetrics::derive(&completed_before_verdict),
            Err(PlaneError::InvalidMetrics { .. })
        ));
    }

    #[test]
    fn flow_metrics_json_round_trips() {
        let pass = PassFlowMetrics {
            pass_seq: 7,
            recorded_at_epoch_s: 1_780_000_000,
            cards: vec![
                CardFlowMetrics::derive(&timeline("MPV2-0001", 1_000, 2)).unwrap(),
                CardFlowMetrics::derive(&timeline("MPV2-0002", 2_000, 1)).unwrap(),
            ],
        };
        let text = pass.to_json().to_canonical_string();
        let parsed = PassFlowMetrics::from_json(&JsonValue::parse(&text).unwrap()).unwrap();
        assert_eq!(parsed, pass);
        assert_eq!(parsed.cards_measured(), 2);
        assert_eq!(parsed.total_rework_count(), 1);
        assert_eq!(parsed.max_cycle_time_s(), Some(200));
        assert_eq!(parsed.max_review_latency_s(), Some(30));
    }

    #[test]
    fn flow_metrics_ledger_is_append_only_and_strictly_monotonic() {
        let mut service = FlowMetricsService::new(InMemoryFlowMetricsStore::new());

        // Every dispatch pass records — including an idle pass with no cards.
        let idle = service.record_next_pass(&[], 10).unwrap();
        assert_eq!(idle.pass_seq, 1);
        assert_eq!(idle.cards_measured(), 0);

        let busy = service
            .record_next_pass(&[timeline("MPV2-0001", 1_000, 2)], 20)
            .unwrap();
        assert_eq!(busy.pass_seq, 2);

        // Replays and out-of-order sequences are refused mechanically.
        for bad_seq in [0, 1, 2] {
            assert!(matches!(
                service.record_pass(&PassFlowMetrics {
                    pass_seq: bad_seq,
                    recorded_at_epoch_s: 30,
                    cards: Vec::new(),
                }),
                Err(PlaneError::NonMonotonicPass { .. })
            ));
        }
        // Duplicate card ids within one pass are refused.
        let dup = CardFlowMetrics::derive(&timeline("MPV2-0001", 1_000, 1)).unwrap();
        assert!(matches!(
            service.record_pass(&PassFlowMetrics {
                pass_seq: 3,
                recorded_at_epoch_s: 30,
                cards: vec![dup.clone(), dup],
            }),
            Err(PlaneError::InvalidMetrics { .. })
        ));

        let seqs: Vec<u64> = service
            .passes()
            .unwrap()
            .iter()
            .map(|p| p.pass_seq)
            .collect();
        assert_eq!(seqs, vec![1, 2]);
        assert_eq!(service.latest_pass().unwrap().unwrap().pass_seq, 2);
    }

    fn surface(lane: &str, card: &str, paths: &[&str]) -> LaneWorkSurface {
        LaneWorkSurface {
            lane_id: lane.into(),
            card_id: card.into(),
            paths: paths.iter().map(|p| (*p).to_owned()).collect(),
        }
    }

    #[test]
    fn disjoint_surfaces_get_a_disjoint_verdict_in_both_phases() {
        let surfaces = [
            surface(
                "lane-fabric-b",
                "MPV2-0000.C003",
                &[
                    "tools/oya-fabric-loop-state-app/src/lib.rs",
                    "tools/oya-fabric-loop-state-app/tests/contract.rs",
                ],
            ),
            surface(
                "lane-fabric-c",
                "MPV2-0000.C004",
                &["tools/oya-fabric-loop-state-app/src/main.rs"],
            ),
        ];
        for phase in [DisjointnessPhase::PreFlight, DisjointnessPhase::PostRun] {
            let report = check_lane_disjointness(phase, &surfaces, 2, 100).unwrap();
            assert_eq!(report.verdict, DisjointnessVerdict::Disjoint);
            assert_eq!(report.phase, phase);
            assert_eq!(report.surfaces.len(), 2);
            let json = report.to_json().to_canonical_string();
            let parsed = JsonValue::parse(&json).unwrap();
            assert_eq!(
                parsed.get("verdict").and_then(JsonValue::as_str),
                Some("disjoint")
            );
            assert_eq!(
                parsed.get("lane_count").and_then(JsonValue::as_num),
                Some(2)
            );
            assert_eq!(
                parsed.get("phase").and_then(JsonValue::as_str),
                Some(phase.as_str())
            );
        }
    }

    #[test]
    fn overlap_routes_to_the_serialized_integrator_lane() {
        // Sibling files in one directory are NOT a collision; equal paths and
        // directory-prefix containment ARE.
        assert!(!paths_collide("crate/src/lib.rs", "crate/src/main.rs"));
        assert!(!paths_collide("crate/src/lib.rs", "crate/src/lib.rs.bak"));
        assert!(paths_collide("crate/src", "crate/src/lib.rs"));
        assert!(paths_collide("crate/src/lib.rs", "crate/src/lib.rs"));

        let surfaces = [
            surface("lane-a", "MPV2-0000.C003", &["crate/src"]),
            surface(
                "lane-b",
                "MPV2-0000.C004",
                &["crate/src/main.rs", "crate/other.rs"],
            ),
        ];
        let report =
            check_lane_disjointness(DisjointnessPhase::PreFlight, &surfaces, 2, 100).unwrap();
        let DisjointnessVerdict::OverlapSerializeToIntegrator { collisions } = &report.verdict
        else {
            panic!("expected overlap verdict");
        };
        assert_eq!(
            collisions,
            &vec![PathCollision {
                lane_a: "lane-a".into(),
                path_a: "crate/src".into(),
                lane_b: "lane-b".into(),
                path_b: "crate/src/main.rs".into(),
            }]
        );
        let json = report.to_json().to_canonical_string();
        let parsed = JsonValue::parse(&json).unwrap();
        assert_eq!(
            parsed.get("verdict").and_then(JsonValue::as_str),
            Some("overlap-serialize-to-integrator")
        );
        assert_eq!(
            parsed
                .get("collisions")
                .and_then(JsonValue::as_arr)
                .map(<[JsonValue]>::len),
            Some(1)
        );
    }

    #[test]
    fn detector_fails_closed_on_capacity_and_malformed_surfaces() {
        let a = surface("lane-a", "C-1", &["x/a.rs"]);
        let b = surface("lane-b", "C-2", &["x/b.rs"]);
        let c = surface("lane-c", "C-3", &["x/c.rs"]);

        // Max concurrent lanes <= independent reviewer capacity.
        assert!(matches!(
            check_lane_disjointness(
                DisjointnessPhase::PreFlight,
                &[a.clone(), b.clone(), c],
                2,
                1
            ),
            Err(PlaneError::LaneCapacityExceeded {
                lanes: 3,
                reviewer_capacity: 2
            })
        ));

        // Zero surfaces, duplicate lanes, one card on two lanes, empty path
        // sets, and unreasonable paths are all refused.
        assert!(matches!(
            check_lane_disjointness(DisjointnessPhase::PreFlight, &[], 2, 1),
            Err(PlaneError::InvalidSurface { .. })
        ));
        assert!(matches!(
            check_lane_disjointness(
                DisjointnessPhase::PreFlight,
                &[a.clone(), surface("lane-a", "C-9", &["y/z.rs"])],
                2,
                1
            ),
            Err(PlaneError::InvalidSurface { .. })
        ));
        assert!(matches!(
            check_lane_disjointness(
                DisjointnessPhase::PreFlight,
                &[a.clone(), surface("lane-z", "C-1", &["y/z.rs"])],
                2,
                1
            ),
            Err(PlaneError::InvalidSurface { .. })
        ));
        assert!(matches!(
            check_lane_disjointness(
                DisjointnessPhase::PreFlight,
                &[surface("lane-a", "C-1", &[])],
                2,
                1
            ),
            Err(PlaneError::InvalidSurface { .. })
        ));
        for bad in ["/abs/path", "a//b", "a/../b", "./a", "a/b/", "a\\b", ""] {
            assert!(
                matches!(
                    check_lane_disjointness(
                        DisjointnessPhase::PreFlight,
                        &[surface("lane-a", "C-1", &[bad])],
                        2,
                        1
                    ),
                    Err(PlaneError::InvalidSurface { .. })
                ),
                "path {bad:?} must be refused"
            );
        }
        assert!(DisjointnessPhase::parse("pre-flight").is_ok());
        assert!(DisjointnessPhase::parse("post-run").is_ok());
        assert!(DisjointnessPhase::parse("mid-run").is_err());
        // Two well-formed surfaces still pass under exact capacity.
        assert!(check_lane_disjointness(DisjointnessPhase::PostRun, &[a, b], 2, 1).is_ok());
    }

    #[test]
    fn owned_sha256_matches_fips_180_4_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        // Multi-block input (crosses the 64-byte block boundary).
        assert_eq!(
            sha256_hex(&[b'a'; 1_000]),
            "41edece42d63e8d9bf515a9ba6932e1c20cbc9f5a5d134645adb5db1b9737ea3"
        );
    }

    /// A minimal ratified masterplan document. The digest is computed with
    /// the same recipe under test, then embedded, so the fixture is
    /// self-consistent by construction.
    fn ratified_fixture() -> JsonValue {
        let masterplan_v2_content = concat!(
            "{\"dependency_edges\": [{\"from\": \"MPV2-0000\", \"to\": \"MPV2-0001\"}],",
            " \"sequencing\": {",
            "\"execution_waves\": [{\"wave_index\": 0, \"work_item_ids\": [\"MPV2-0000\"]},",
            " {\"wave_index\": 1, \"work_item_ids\": [\"MPV2-0001\"]}],",
            " \"work_item_order\": [{\"index\": 0, \"work_item_id\": \"MPV2-0000\"},",
            " {\"index\": 1, \"work_item_id\": \"MPV2-0001\"}]}}"
        );
        let v2 = JsonValue::parse(masterplan_v2_content).unwrap();
        let digest = compute_sequencing_digest(&v2).unwrap();
        let doc_text = format!(
            concat!(
                "{{\"masterplan_v2\": {{\"dependency_edges\": [{{\"from\": \"MPV2-0000\", \"to\": \"MPV2-0001\"}}],",
                " \"sequencing\": {{",
                "\"sequencing_identity\": {{\"sequencing_version\": 1, \"sequencing_hash\": \"{digest}\"}},",
                " \"execution_waves\": [{{\"wave_index\": 0, \"work_item_ids\": [\"MPV2-0000\"]}},",
                " {{\"wave_index\": 1, \"work_item_ids\": [\"MPV2-0001\"]}}],",
                " \"work_item_order\": [{{\"index\": 0, \"work_item_id\": \"MPV2-0000\"}},",
                " {{\"index\": 1, \"work_item_id\": \"MPV2-0001\"}}],",
                " \"founder_ratification\": {{\"decision_recorded\": true,",
                " \"decision_status\": \"ratified\",",
                " \"decision_ref\": \"evidence/goals/ratification.json\",",
                " \"ratified_sequencing_digest\": \"{digest}\",",
                " \"ratified_sequencing_version\": 1}}}}}}}}"
            ),
            digest = digest
        );
        JsonValue::parse(&doc_text).unwrap()
    }

    fn obj_fields_mut<'a>(value: &'a mut JsonValue, key: &str) -> &'a mut JsonValue {
        let JsonValue::Obj(fields) = value else {
            panic!("expected object");
        };
        &mut fields
            .iter_mut()
            .find(|(k, _)| k == key)
            .unwrap_or_else(|| panic!("missing key {key}"))
            .1
    }

    fn remove_field(value: &mut JsonValue, key: &str) {
        let JsonValue::Obj(fields) = value else {
            panic!("expected object");
        };
        fields.retain(|(k, _)| k != key);
    }

    #[test]
    fn ratification_gate_grants_dispatch_when_record_binds_to_live_content() {
        let doc = ratified_fixture();
        let grant = check_dispatch_ratification(&doc.to_canonical_string()).unwrap();
        assert_eq!(grant.sequencing_version, 1);
        assert!(grant.sequencing_digest.starts_with("sha256:"));
        assert_eq!(
            grant.decision_ref.as_deref(),
            Some("evidence/goals/ratification.json")
        );
    }

    #[test]
    fn ratification_gate_refuses_dispatch_when_record_is_absent() {
        // Failure injection: strip the founder_ratification record entirely.
        let mut doc = ratified_fixture();
        let sequencing = obj_fields_mut(obj_fields_mut(&mut doc, "masterplan_v2"), "sequencing");
        remove_field(sequencing, "founder_ratification");
        assert!(matches!(
            check_dispatch_ratification(&doc.to_canonical_string()),
            Err(RatificationRefusal::RecordAbsent(_))
        ));

        // Failure injection: a record that exists but is not a recorded
        // decision is equally absent for dispatch purposes.
        let mut doc = ratified_fixture();
        let record = obj_fields_mut(
            obj_fields_mut(obj_fields_mut(&mut doc, "masterplan_v2"), "sequencing"),
            "founder_ratification",
        );
        *obj_fields_mut(record, "decision_recorded") = JsonValue::Bool(false);
        assert!(matches!(
            check_dispatch_ratification(&doc.to_canonical_string()),
            Err(RatificationRefusal::RecordAbsent(_))
        ));
    }

    #[test]
    fn ratification_gate_refuses_dispatch_when_ratified_digest_is_stale() {
        // Failure injection: mutate the ratifiable sequencing content (drop a
        // work item from the order) WITHOUT re-ratifying. The recomputed
        // digest changes and the recorded ratification is void.
        let mut doc = ratified_fixture();
        let sequencing = obj_fields_mut(obj_fields_mut(&mut doc, "masterplan_v2"), "sequencing");
        let JsonValue::Arr(order) = obj_fields_mut(sequencing, "work_item_order") else {
            panic!("work_item_order must be an array");
        };
        order.pop();
        assert!(matches!(
            check_dispatch_ratification(&doc.to_canonical_string()),
            Err(RatificationRefusal::StaleDigest { .. })
        ));
    }

    #[test]
    fn ratification_gate_refuses_dispatch_when_ratified_version_is_stale() {
        // Failure injection: a fresh re-derivation bumps the sequencing
        // version but the old ratification record survives — the version
        // binding breaks and dispatch stays closed.
        let mut doc = ratified_fixture();
        let identity = obj_fields_mut(
            obj_fields_mut(obj_fields_mut(&mut doc, "masterplan_v2"), "sequencing"),
            "sequencing_identity",
        );
        *obj_fields_mut(identity, "sequencing_version") = JsonValue::Num(2);
        assert!(matches!(
            check_dispatch_ratification(&doc.to_canonical_string()),
            Err(RatificationRefusal::StaleVersion {
                ratified_sequencing_version: Some(1),
                current_sequencing_version: 2,
            })
        ));
    }

    #[test]
    fn ratification_gate_refuses_dispatch_when_masterplan_is_unreadable() {
        // Missing file fails closed.
        let missing_root = std::env::temp_dir().join(format!(
            "oya-ratification-gate-missing-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        assert!(matches!(
            check_dispatch_ratification_at(&missing_root),
            Err(RatificationRefusal::MasterplanUnreadable(_))
        ));
        // Unparseable content fails closed.
        assert!(matches!(
            check_dispatch_ratification("not json"),
            Err(RatificationRefusal::MasterplanUnreadable(_))
        ));
    }
}
