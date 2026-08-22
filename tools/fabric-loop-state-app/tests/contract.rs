//! Port-level contract tests for the ADR-0516 fabric two-plane drive-loop
//! state (`specs/fabric-drive-loop-state.json`).
//!
//! What is proven here:
//! 1. ALL loop-state reads/writes route through the two plane ports — the
//!    single-writer facade is exercised over instrumented spy ports and every
//!    lifecycle operation is observed as port calls (the facade is generic
//!    over the traits and owns no I/O, so there is no other path).
//! 2. The named owned cloud-ci cutover target is recorded on the port
//!    contract itself and agrees with the machine-readable spec.
//! 3. The execution plane root is gitignored (repo-adjacent operational
//!    state) while the durable coordination plane root is PR-governed.
//! 4. The filesystem bridge adapters round-trip both planes across reopen and
//!    enforce lane-exclusive claims.
//! 5. Per-pass flow metrics (cycle time, review latency, rework count) are
//!    recorded through the flow-metrics port on every dispatch pass and
//!    persist across passes and process boundaries (append-only, strictly
//!    monotonic ledger on the durable plane).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;

use fabric_loop_state_app::{
    BlockKind, CUTOVER_CRITERIA, CUTOVER_TARGET_DESTINATION_HOME, CUTOVER_TARGET_OWNER,
    CUTOVER_TARGET_SERVICE_NAME, CardFlowMetrics, CardFlowTimeline, CardStatus, ClaimRecord,
    CoordinationPlanePort, CutoverTarget, DEFAULT_DURABLE_PLANE_ROOT, DEFAULT_FLOW_METRICS_ROOT,
    DEFAULT_OPERATIONAL_PLANE_ROOT, ExecutionPlanePort, FlowMetricsPort, FlowMetricsService,
    FsCoordinationStore, FsExecutionStore, FsFlowMetricsStore, HeartbeatRecord,
    InMemoryCoordinationStore, InMemoryExecutionStore, InMemoryFlowMetricsStore, JsonValue,
    LoopCard, LoopStateService, PORT_CONTRACT_SPEC_PATH, PassFlowMetrics, PlaneDescriptor,
    PlaneError, RunRecord, RunState,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Walk up from the test's working directory to the repo root (the dir holding
/// the canonical `AGENTS.md`). Mirrors the sibling gates.
fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("AGENTS.md").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("repo root marker AGENTS.md not found");
}

/// Unique scratch dir under the OS temp root (no third-party tempdir dep).
fn scratch_dir(label: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "fabric-loop-state-{label}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn card(id: &str, deps: &[&str], status: CardStatus) -> LoopCard {
    LoopCard {
        card_id: id.into(),
        title: format!("card {id}"),
        program_id: "P-FABRIC".into(),
        depends_on: deps.iter().map(|d| (*d).to_owned()).collect(),
        status,
        evidence_refs: Vec::new(),
    }
}

type CallCounts = Rc<RefCell<BTreeMap<&'static str, u64>>>;

fn bump(counts: &CallCounts, method: &'static str) {
    *counts.borrow_mut().entry(method).or_insert(0) += 1;
}

/// Spy coordination port: forwards to an inner port, counting every call.
struct SpyCoordination<P: CoordinationPlanePort> {
    inner: P,
    counts: CallCounts,
}

impl<P: CoordinationPlanePort> CoordinationPlanePort for SpyCoordination<P> {
    fn put_card(&mut self, card: &LoopCard) -> Result<(), PlaneError> {
        bump(&self.counts, "coordination.put_card");
        self.inner.put_card(card)
    }
    fn card(&self, card_id: &str) -> Result<Option<LoopCard>, PlaneError> {
        bump(&self.counts, "coordination.card");
        self.inner.card(card_id)
    }
    fn cards(&self) -> Result<Vec<LoopCard>, PlaneError> {
        bump(&self.counts, "coordination.cards");
        self.inner.cards()
    }
    fn descriptor(&self) -> PlaneDescriptor {
        self.inner.descriptor()
    }
}

/// Spy execution port: forwards to an inner port, counting every call.
struct SpyExecution<P: ExecutionPlanePort> {
    inner: P,
    counts: CallCounts,
}

impl<P: ExecutionPlanePort> ExecutionPlanePort for SpyExecution<P> {
    fn claim(
        &mut self,
        card_id: &str,
        lane_id: &str,
        at_epoch_s: u64,
    ) -> Result<ClaimRecord, PlaneError> {
        bump(&self.counts, "execution.claim");
        self.inner.claim(card_id, lane_id, at_epoch_s)
    }
    fn active_claim(&self, card_id: &str) -> Result<Option<ClaimRecord>, PlaneError> {
        bump(&self.counts, "execution.active_claim");
        self.inner.active_claim(card_id)
    }
    fn heartbeat(
        &mut self,
        card_id: &str,
        lane_id: &str,
        at_epoch_s: u64,
    ) -> Result<HeartbeatRecord, PlaneError> {
        bump(&self.counts, "execution.heartbeat");
        self.inner.heartbeat(card_id, lane_id, at_epoch_s)
    }
    fn last_heartbeat(&self, card_id: &str) -> Result<Option<HeartbeatRecord>, PlaneError> {
        bump(&self.counts, "execution.last_heartbeat");
        self.inner.last_heartbeat(card_id)
    }
    fn record_run(&mut self, record: &RunRecord) -> Result<(), PlaneError> {
        bump(&self.counts, "execution.record_run");
        self.inner.record_run(record)
    }
    fn run_record(&self, card_id: &str) -> Result<Option<RunRecord>, PlaneError> {
        bump(&self.counts, "execution.run_record");
        self.inner.run_record(card_id)
    }
    fn release(&mut self, card_id: &str, lane_id: &str) -> Result<(), PlaneError> {
        bump(&self.counts, "execution.release");
        self.inner.release(card_id, lane_id)
    }
    fn descriptor(&self) -> PlaneDescriptor {
        self.inner.descriptor()
    }
}

// ---------------------------------------------------------------------------
// 1. All reads/writes route through the ports
// ---------------------------------------------------------------------------

#[test]
fn all_loop_state_reads_and_writes_route_through_the_ports() {
    let counts: CallCounts = Rc::new(RefCell::new(BTreeMap::new()));
    let mut service = LoopStateService::new(
        SpyCoordination {
            inner: InMemoryCoordinationStore::new(),
            counts: Rc::clone(&counts),
        },
        SpyExecution {
            inner: InMemoryExecutionStore::new(),
            counts: Rc::clone(&counts),
        },
    );

    // Full lifecycle: define -> ready -> claim -> heartbeat -> run -> block ->
    // complete-with-evidence -> verify. Every operation below is a facade call;
    // the facade owns no store and no I/O, so the spies observe every touch.
    service
        .define_card(&card("MPV2-DEP", &[], CardStatus::Ready))
        .unwrap();
    service
        .define_card(&card("MPV2-WORK", &["MPV2-DEP"], CardStatus::Ready))
        .unwrap();
    service.claim_ready("MPV2-DEP", "lane-a", 1).unwrap();
    service.heartbeat("MPV2-DEP", "lane-a", 2).unwrap();
    service.start_run("MPV2-DEP", "lane-a", 3).unwrap();
    service
        .mark_blocked("MPV2-DEP", "lane-a", BlockKind::NeedsReview, "review", 4)
        .unwrap();
    service
        .complete("MPV2-DEP", "lane-a", &["evidence/goals/dep.json".into()], 5)
        .unwrap();
    service
        .verify_done("MPV2-DEP", "evidence/goals/dep-verify.json")
        .unwrap();
    let ready = service.ready_cards().unwrap();
    assert_eq!(ready.len(), 1, "DAG must admit MPV2-WORK after verify");
    service.claim_ready("MPV2-WORK", "lane-b", 6).unwrap();
    service.last_heartbeat("MPV2-DEP").unwrap();
    service.run_record("MPV2-WORK").unwrap();
    service.active_claim("MPV2-WORK").unwrap();
    service.cards().unwrap();
    service.card("MPV2-WORK").unwrap();

    let observed = counts.borrow().clone();
    // Every port method is observed: no loop-state read or write happened
    // outside the two port traits.
    for method in [
        "coordination.put_card",
        "coordination.card",
        "coordination.cards",
        "execution.claim",
        "execution.active_claim",
        "execution.heartbeat",
        "execution.last_heartbeat",
        "execution.record_run",
        "execution.run_record",
        "execution.release",
    ] {
        assert!(
            observed.get(method).copied().unwrap_or(0) > 0,
            "port method {method} was never crossed; observed: {observed:?}"
        );
    }

    // Durable writes: 2 defines + 1 complete + 1 verify — exactly 4 put_card
    // port crossings, i.e. the coordination plane has a single mutation path.
    assert_eq!(observed.get("coordination.put_card"), Some(&4));
    // Operational claim lifecycle: 2 claims, 1 release (complete releases).
    assert_eq!(observed.get("execution.claim"), Some(&2));
    assert_eq!(observed.get("execution.release"), Some(&1));
}

// ---------------------------------------------------------------------------
// 2. Cutover target recorded in the port contract and the spec
// ---------------------------------------------------------------------------

#[test]
fn cutover_target_is_recorded_in_port_contract_and_this_spec() {
    // The port contract itself names the owned cloud-ci destination: the
    // defaulted trait method reports the canonical target on EVERY adapter.
    let coordination = InMemoryCoordinationStore::new();
    let execution = InMemoryExecutionStore::new();
    let canonical = CutoverTarget::canonical();
    assert_eq!(coordination.cutover_target(), canonical);
    assert_eq!(execution.cutover_target(), canonical);

    let fs_coordination = FsCoordinationStore::open(scratch_dir("cutover-coord"));
    let fs_execution = FsExecutionStore::open(scratch_dir("cutover-exec"));
    assert_eq!(fs_coordination.cutover_target(), canonical);
    assert_eq!(fs_execution.cutover_target(), canonical);

    let service = LoopStateService::new(fs_coordination, fs_execution);
    let (via_coordination, via_execution) = service.cutover_targets();
    assert_eq!(via_coordination, canonical);
    assert_eq!(via_execution, canonical);

    assert_eq!(canonical.service_name, CUTOVER_TARGET_SERVICE_NAME);
    assert_eq!(canonical.owner, CUTOVER_TARGET_OWNER);
    assert_eq!(canonical.destination_home, CUTOVER_TARGET_DESTINATION_HOME);

    // The machine-readable spec records the SAME named target and criteria.
    let spec_path = repo_root().join(PORT_CONTRACT_SPEC_PATH);
    let spec_text = std::fs::read_to_string(&spec_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", spec_path.display()));
    let spec = JsonValue::parse(&spec_text).expect("port contract spec parses as canonical JSON");
    let target = spec.get("cutover_target").expect("spec#cutover_target");
    assert_eq!(
        target.get("service_name").and_then(JsonValue::as_str),
        Some(CUTOVER_TARGET_SERVICE_NAME)
    );
    assert_eq!(
        target.get("owner").and_then(JsonValue::as_str),
        Some(CUTOVER_TARGET_OWNER)
    );
    assert_eq!(
        target.get("destination_home").and_then(JsonValue::as_str),
        Some(CUTOVER_TARGET_DESTINATION_HOME)
    );
    let criteria = target
        .get("cutover_criteria")
        .and_then(JsonValue::as_arr)
        .expect("spec#cutover_target.cutover_criteria");
    let spec_criteria: Vec<&str> = criteria.iter().filter_map(JsonValue::as_str).collect();
    assert_eq!(spec_criteria, CUTOVER_CRITERIA.to_vec());

    // The spec pins the plane roots the adapters default to.
    let planes = spec.get("planes").expect("spec#planes");
    assert_eq!(
        planes
            .get("coordination")
            .and_then(|p| p.get("default_store_root"))
            .and_then(JsonValue::as_str),
        Some(DEFAULT_DURABLE_PLANE_ROOT)
    );
    assert_eq!(
        planes
            .get("execution")
            .and_then(|p| p.get("default_store_root"))
            .and_then(JsonValue::as_str),
        Some(DEFAULT_OPERATIONAL_PLANE_ROOT)
    );
}

// ---------------------------------------------------------------------------
// 3. Plane durability: operational root gitignored, durable root PR-governed
// ---------------------------------------------------------------------------

#[test]
fn operational_plane_root_is_gitignored_and_durable_plane_root_is_not() {
    let gitignore = std::fs::read_to_string(repo_root().join(".gitignore")).expect(".gitignore");
    let has_operational_ignore = gitignore
        .lines()
        .any(|line| line.trim() == format!("/{DEFAULT_OPERATIONAL_PLANE_ROOT}/"));
    assert!(
        has_operational_ignore,
        "execution plane root /{DEFAULT_OPERATIONAL_PLANE_ROOT}/ must be gitignored"
    );
    let durable_ignored = gitignore.lines().any(|line| {
        let line = line.trim();
        !line.starts_with('#') && line.contains(DEFAULT_DURABLE_PLANE_ROOT)
    });
    assert!(
        !durable_ignored,
        "coordination plane root {DEFAULT_DURABLE_PLANE_ROOT} must stay PR-governed (tracked)"
    );

    // The adapters self-describe the same durability split.
    let coordination = FsCoordinationStore::open(scratch_dir("durability-coord"));
    let execution = FsExecutionStore::open(scratch_dir("durability-exec"));
    assert_eq!(
        format!("{:?}", coordination.descriptor().durability),
        "InRepoPrGoverned"
    );
    assert_eq!(
        format!("{:?}", execution.descriptor().durability),
        "RepoAdjacentGitignored"
    );
}

// ---------------------------------------------------------------------------
// 4. Filesystem bridges: round-trip across reopen + lane-exclusive claims
// ---------------------------------------------------------------------------

#[test]
fn fs_bridge_adapters_round_trip_both_planes_across_reopen() {
    let coord_root = scratch_dir("reopen-coord");
    let exec_root = scratch_dir("reopen-exec");

    {
        let mut coordination = FsCoordinationStore::open(&coord_root);
        let mut work = card("MPV2-FS", &["MPV2-OTHER"], CardStatus::Defined);
        work.evidence_refs.push("evidence/goals/fs.json".into());
        coordination.put_card(&work).unwrap();

        let mut execution = FsExecutionStore::open(&exec_root);
        execution.claim("MPV2-FS", "lane-fs", 100).unwrap();
        execution.heartbeat("MPV2-FS", "lane-fs", 101).unwrap();
        execution
            .record_run(&RunRecord {
                card_id: "MPV2-FS".into(),
                lane_id: "lane-fs".into(),
                state: RunState::Blocked(BlockKind::NeedsInfra),
                note: "waiting on runner".into(),
                updated_at_epoch_s: 102,
            })
            .unwrap();
    }

    // Reopen both stores: durable and operational state survive the process
    // boundary and decode through the ports.
    let coordination = FsCoordinationStore::open(&coord_root);
    let restored = coordination
        .card("MPV2-FS")
        .unwrap()
        .expect("card survives");
    assert_eq!(restored.status, CardStatus::Defined);
    assert_eq!(restored.depends_on, vec!["MPV2-OTHER".to_owned()]);
    assert_eq!(
        restored.evidence_refs,
        vec!["evidence/goals/fs.json".to_owned()]
    );
    assert_eq!(coordination.cards().unwrap().len(), 1);

    let mut execution = FsExecutionStore::open(&exec_root);
    let claim = execution
        .active_claim("MPV2-FS")
        .unwrap()
        .expect("claim survives");
    assert_eq!(claim.lane_id, "lane-fs");
    assert_eq!(claim.claimed_at_epoch_s, 100);
    let beat = execution
        .last_heartbeat("MPV2-FS")
        .unwrap()
        .expect("heartbeat survives");
    assert_eq!(beat.beat_at_epoch_s, 101);
    let run = execution
        .run_record("MPV2-FS")
        .unwrap()
        .expect("run record survives");
    assert!(matches!(
        run.state,
        RunState::Blocked(BlockKind::NeedsInfra)
    ));
    assert_eq!(run.note, "waiting on runner");

    // Release drops the claim durably.
    execution.release("MPV2-FS", "lane-fs").unwrap();
    let execution = FsExecutionStore::open(&exec_root);
    assert!(execution.active_claim("MPV2-FS").unwrap().is_none());
}

#[test]
fn execution_plane_enforces_lane_exclusive_claims() {
    let mut execution = FsExecutionStore::open(scratch_dir("claims"));
    execution.claim("MPV2-X", "lane-a", 1).unwrap();

    // Second claim by another lane is a mechanical conflict, not a judgment.
    assert!(matches!(
        execution.claim("MPV2-X", "lane-b", 2),
        Err(PlaneError::AlreadyClaimed { holder_lane, .. }) if holder_lane == "lane-a"
    ));
    // Heartbeat / run / release by a non-holder lane are refused.
    assert!(matches!(
        execution.heartbeat("MPV2-X", "lane-b", 3),
        Err(PlaneError::WrongLane { .. })
    ));
    assert!(matches!(
        execution.record_run(&RunRecord {
            card_id: "MPV2-X".into(),
            lane_id: "lane-b".into(),
            state: RunState::Running,
            note: String::new(),
            updated_at_epoch_s: 4,
        }),
        Err(PlaneError::WrongLane { .. })
    ));
    assert!(matches!(
        execution.release("MPV2-X", "lane-b"),
        Err(PlaneError::WrongLane { .. })
    ));
    // Unclaimed cards refuse heartbeats outright.
    assert!(matches!(
        execution.heartbeat("MPV2-Y", "lane-a", 5),
        Err(PlaneError::NotClaimed(_))
    ));
    // Path traversal cannot escape the store root.
    assert!(matches!(
        execution.claim("../escape", "lane-a", 6),
        Err(PlaneError::InvalidId(_))
    ));
}

// ---------------------------------------------------------------------------
// 5. Evidence discipline + DAG readiness through the facade over fs bridges
// ---------------------------------------------------------------------------

#[test]
fn completion_requires_evidence_and_readiness_derives_from_the_dag() {
    let mut service = LoopStateService::new(
        FsCoordinationStore::open(scratch_dir("facade-coord")),
        FsExecutionStore::open(scratch_dir("facade-exec")),
    );
    service
        .define_card(&card("MPV2-A", &[], CardStatus::Ready))
        .unwrap();
    service
        .define_card(&card("MPV2-B", &["MPV2-A"], CardStatus::Ready))
        .unwrap();

    // Cards can never be born done (status claims require evidence-carrying
    // transitions).
    assert!(matches!(
        service.define_card(&card("MPV2-C", &[], CardStatus::DoneVerified)),
        Err(PlaneError::InvalidTransition { .. })
    ));

    // Ready set derives from the DAG: only the dependency-free card.
    let ready: Vec<String> = service
        .ready_cards()
        .unwrap()
        .into_iter()
        .map(|c| c.card_id)
        .collect();
    assert_eq!(ready, vec!["MPV2-A".to_owned()]);

    service.claim_ready("MPV2-A", "lane-a", 1).unwrap();
    // A claimed card leaves the ready set (claim visibility via the port).
    assert!(service.ready_cards().unwrap().is_empty());

    // Completion without evidence is refused; with evidence it lands as
    // claimed-done-unverified, which still does NOT satisfy the DAG.
    assert!(matches!(
        service.complete("MPV2-A", "lane-a", &[String::new()], 2),
        Err(PlaneError::MissingEvidence(_))
    ));
    service
        .complete("MPV2-A", "lane-a", &["evidence/goals/a.json".into()], 3)
        .unwrap();
    assert!(service.ready_cards().unwrap().is_empty());
    assert!(matches!(
        service.claim_ready("MPV2-B", "lane-b", 4),
        Err(PlaneError::DependencyUnsatisfied { missing, .. }) if missing == vec!["MPV2-A".to_owned()]
    ));

    // Verification attaches evidence and unlocks the dependent card.
    service
        .verify_done("MPV2-A", "evidence/goals/a-verify.json")
        .unwrap();
    let verified = service.card("MPV2-A").unwrap().unwrap();
    assert_eq!(verified.status, CardStatus::DoneVerified);
    assert_eq!(
        verified.evidence_refs,
        vec![
            "evidence/goals/a.json".to_owned(),
            "evidence/goals/a-verify.json".to_owned()
        ]
    );
    service.claim_ready("MPV2-B", "lane-b", 5).unwrap();
}

// ---------------------------------------------------------------------------
// 6. Per-pass flow metrics: recorded through the port, persisted across passes
// ---------------------------------------------------------------------------

fn flow_timeline(card_id: &str, lane_id: &str, base: u64, rounds: u64) -> CardFlowTimeline {
    CardFlowTimeline {
        card_id: card_id.into(),
        lane_id: lane_id.into(),
        claimed_at_epoch_s: base,
        review_requested_at_epoch_s: base + 100,
        review_first_verdict_at_epoch_s: base + 130,
        completed_at_epoch_s: base + 200,
        review_rounds: rounds,
    }
}

/// Spy flow-metrics port: forwards to an inner port, counting every call.
struct SpyFlowMetrics<P: FlowMetricsPort> {
    inner: P,
    counts: CallCounts,
}

impl<P: FlowMetricsPort> FlowMetricsPort for SpyFlowMetrics<P> {
    fn record_pass(&mut self, pass: &PassFlowMetrics) -> Result<(), PlaneError> {
        bump(&self.counts, "metrics.record_pass");
        self.inner.record_pass(pass)
    }
    fn pass(&self, pass_seq: u64) -> Result<Option<PassFlowMetrics>, PlaneError> {
        bump(&self.counts, "metrics.pass");
        self.inner.pass(pass_seq)
    }
    fn passes(&self) -> Result<Vec<PassFlowMetrics>, PlaneError> {
        bump(&self.counts, "metrics.passes");
        self.inner.passes()
    }
    fn descriptor(&self) -> PlaneDescriptor {
        self.inner.descriptor()
    }
}

#[test]
fn flow_metric_recording_routes_through_the_metrics_port() {
    let counts: CallCounts = Rc::new(RefCell::new(BTreeMap::new()));
    let mut service = FlowMetricsService::new(SpyFlowMetrics {
        inner: InMemoryFlowMetricsStore::new(),
        counts: Rc::clone(&counts),
    });

    // One idle pass and one measured pass — metrics record on EVERY pass.
    service.record_next_pass(&[], 1).unwrap();
    service
        .record_next_pass(&[flow_timeline("MPV2-M1", "lane-a", 1_000, 2)], 2)
        .unwrap();
    service.pass(1).unwrap();
    service.passes().unwrap();
    service.latest_pass().unwrap();

    let observed = counts.borrow().clone();
    for method in ["metrics.record_pass", "metrics.pass", "metrics.passes"] {
        assert!(
            observed.get(method).copied().unwrap_or(0) > 0,
            "port method {method} was never crossed; observed: {observed:?}"
        );
    }
    // The facade owns no store: exactly the two passes crossed record_pass.
    assert_eq!(observed.get("metrics.record_pass"), Some(&2));
}

#[test]
fn flow_metrics_are_recorded_and_persisted_across_passes() {
    let root = scratch_dir("flow-metrics");

    // Pass 1 recorded in one process "session"...
    {
        let mut service = FlowMetricsService::new(FsFlowMetricsStore::open(&root));
        let pass = service
            .record_next_pass(
                &[
                    flow_timeline("MPV2-M1", "lane-a", 1_000, 1),
                    flow_timeline("MPV2-M2", "lane-b", 1_000, 3),
                ],
                1_780_000_000,
            )
            .unwrap();
        assert_eq!(pass.pass_seq, 1);
    }

    // ...survives reopen, and pass 2 appends against the persisted head.
    {
        let mut service = FlowMetricsService::new(FsFlowMetricsStore::open(&root));
        let restored = service.pass(1).unwrap().expect("pass 1 survives reopen");
        assert_eq!(restored.cards_measured(), 2);
        assert_eq!(restored.total_rework_count(), 2);
        assert_eq!(restored.max_cycle_time_s(), Some(200));
        assert_eq!(restored.max_review_latency_s(), Some(30));

        let pass = service.record_next_pass(&[], 1_780_000_100).unwrap();
        assert_eq!(
            pass.pass_seq, 2,
            "sequence continues from the persisted head"
        );
    }

    // Both passes are durable across a third reopen with full metric content.
    let mut store = FsFlowMetricsStore::open(&root);
    let passes = store.passes().unwrap();
    assert_eq!(
        passes.iter().map(|p| p.pass_seq).collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(passes[0].recorded_at_epoch_s, 1_780_000_000);
    assert_eq!(passes[0].cards.len(), 2);
    let m1 = &passes[0].cards[0];
    assert_eq!(
        (m1.card_id.as_str(), m1.lane_id.as_str()),
        ("MPV2-M1", "lane-a")
    );
    assert_eq!(m1.cycle_time_s, 200);
    assert_eq!(m1.review_latency_s, 30);
    assert_eq!(m1.rework_count, 0);
    assert_eq!(passes[0].cards[1].rework_count, 2);
    assert!(passes[1].cards.is_empty(), "idle pass is still recorded");
    assert_eq!(store.latest_pass().unwrap().unwrap().pass_seq, 2);

    // Replays / out-of-order writes stay refused after reopen (append-only,
    // strictly monotonic against the PERSISTED head — a mechanical property,
    // not discipline).
    assert!(matches!(
        store.record_pass(&PassFlowMetrics {
            pass_seq: 2,
            recorded_at_epoch_s: 1_780_000_200,
            cards: Vec::new(),
        }),
        Err(PlaneError::NonMonotonicPass {
            pass_seq: 2,
            latest_recorded: 2
        })
    ));

    // The ledger self-describes as durable-plane data behind the port and
    // models the same owned cloud-ci cutover target.
    let descriptor = store.descriptor();
    assert_eq!(format!("{:?}", descriptor.durability), "InRepoPrGoverned");
    assert_eq!(store.cutover_target(), CutoverTarget::canonical());
    assert!(
        DEFAULT_FLOW_METRICS_ROOT.starts_with(DEFAULT_DURABLE_PLANE_ROOT),
        "flow metrics default root rides the durable coordination plane"
    );

    // Derived metrics fail closed on malformed timelines even at this level.
    let mut inverted = flow_timeline("MPV2-M3", "lane-a", 1_000, 1);
    inverted.completed_at_epoch_s = 1;
    assert!(matches!(
        CardFlowMetrics::derive(&inverted),
        Err(PlaneError::InvalidMetrics { .. })
    ));

    // A foreign file planted in the ledger directory is surfaced fail-closed,
    // never silently skipped (no shadow ledger can hide beside the passes).
    std::fs::write(root.join("passes").join("pass-shadow.json"), "{}").unwrap();
    assert!(matches!(
        store.passes(),
        Err(PlaneError::Corrupt(detail)) if detail.contains("pass-shadow.json")
    ));
}

#[test]
fn non_canonical_pass_filenames_are_refused_fail_closed() {
    let root = scratch_dir("flow-metrics-non-canonical");

    // Seed one canonical pass so the ledger head exists, then verify the
    // canonical file written by record_pass round-trips exactly.
    let mut store = FsFlowMetricsStore::open(&root);
    store
        .record_pass(&PassFlowMetrics {
            pass_seq: 1,
            recorded_at_epoch_s: 1_780_000_000,
            cards: Vec::new(),
        })
        .unwrap();
    assert!(
        root.join("passes")
            .join(format!("pass-{:020}.json", 1))
            .is_file(),
        "record_pass writes the canonical zero-padded 20-digit path"
    );
    assert_eq!(store.pass(1).unwrap().unwrap().pass_seq, 1);
    assert_eq!(store.passes().unwrap().len(), 1);

    // A non-canonical numeric filename (`pass-7.json`) would bump the
    // monotonic head in record_pass while pass()/passes() only read the
    // canonical zero-padded path — so it MUST be refused fail-closed, never
    // silently admitted into the sequence.
    std::fs::write(root.join("passes").join("pass-7.json"), "{}").unwrap();
    assert!(matches!(
        store.passes(),
        Err(PlaneError::Corrupt(detail)) if detail.contains("pass-7")
    ));
    assert!(matches!(
        store.record_pass(&PassFlowMetrics {
            pass_seq: 2,
            recorded_at_epoch_s: 1_780_000_100,
            cards: Vec::new(),
        }),
        Err(PlaneError::Corrupt(detail)) if detail.contains("pass-7")
    ));

    // Removing the planted file restores the ledger, and the canonical
    // record still round-trips with full content.
    std::fs::remove_file(root.join("passes").join("pass-7.json")).unwrap();
    let restored = store.pass(1).unwrap().expect("canonical pass survives");
    assert_eq!(restored.recorded_at_epoch_s, 1_780_000_000);
    assert!(restored.cards.is_empty());
}

/// The mechanical lane-disjointness detector (path/ownership-overlap) is the
/// sole decider of what runs concurrently: disjoint declared surfaces verdict
/// `disjoint` pre-flight and re-verdict `disjoint` post-run over actual
/// touched paths; any equal-or-nested path pair verdicts
/// `overlap-serialize-to-integrator`; and the lane count is capped by
/// independent reviewer capacity fail-closed. The JSON report is the evidence
/// shape captured for every parallel dispatch.
#[test]
fn lane_disjointness_detector_verdicts_parallel_dispatch_mechanically() {
    use fabric_loop_state_app::{
        DisjointnessPhase, DisjointnessVerdict, LaneWorkSurface, check_lane_disjointness,
    };

    let surface = |lane: &str, card_id: &str, paths: &[&str]| LaneWorkSurface {
        lane_id: lane.into(),
        card_id: card_id.into(),
        paths: paths.iter().map(|p| (*p).to_owned()).collect(),
    };
    let declared = [
        surface(
            "lane-fabric-b",
            "MPV2-0000.C003",
            &[
                "tools/fabric-loop-state-app/src/lib.rs",
                "tools/fabric-loop-state-app/tests/contract.rs",
            ],
        ),
        surface(
            "lane-fabric-c",
            "MPV2-0000.C004",
            &["tools/fabric-loop-state-app/src/main.rs"],
        ),
    ];

    // Pre-flight over declared surfaces and post-run over actual touched
    // paths use the SAME mechanical check; both verdict disjoint here.
    for phase in [DisjointnessPhase::PreFlight, DisjointnessPhase::PostRun] {
        let report = check_lane_disjointness(phase, &declared, 2, 1_783_000_000).unwrap();
        assert_eq!(report.verdict, DisjointnessVerdict::Disjoint);
        let json = report.to_json().to_canonical_string();
        let parsed = JsonValue::parse(&json).unwrap();
        assert_eq!(
            parsed.get("phase").and_then(JsonValue::as_str),
            Some(phase.as_str())
        );
        assert_eq!(
            parsed.get("verdict").and_then(JsonValue::as_str),
            Some("disjoint")
        );
        assert_eq!(
            parsed
                .get("collisions")
                .and_then(JsonValue::as_arr)
                .map(<[JsonValue]>::len),
            Some(0)
        );
    }

    // A shared root (one lane claims the crate dir, the other a file inside
    // it) is a mechanical collision: the verdict routes the work to the
    // serialized integrator lane instead of parallel dispatch.
    let colliding = [
        surface(
            "lane-fabric-b",
            "MPV2-0000.C003",
            &["tools/fabric-loop-state-app"],
        ),
        surface(
            "lane-fabric-c",
            "MPV2-0000.C004",
            &["tools/fabric-loop-state-app/src/main.rs"],
        ),
    ];
    let report =
        check_lane_disjointness(DisjointnessPhase::PreFlight, &colliding, 2, 1_783_000_000)
            .unwrap();
    assert_eq!(report.verdict.as_str(), "overlap-serialize-to-integrator");

    // Parallel-safety cap: more lanes than independent reviewer capacity is
    // refused fail-closed before any disjointness reasoning.
    assert!(matches!(
        check_lane_disjointness(DisjointnessPhase::PreFlight, &declared, 1, 1_783_000_000),
        Err(PlaneError::LaneCapacityExceeded {
            lanes: 2,
            reviewer_capacity: 1
        })
    ));
}
