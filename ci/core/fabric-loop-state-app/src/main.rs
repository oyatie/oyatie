//! `oya-fabric-loop-state` — local lane-runtime bridge over the two-plane
//! drive-loop state ports. LOCAL BRIDGE ONLY (retirement-marked per the
//! founder CLI directive 2026-06-09): this bin carries NO merge, CI, or plan
//! authority. Every read and write routes through the single-writer facades
//! (`LoopStateService`, `FlowMetricsService`) over the port traits
//! (`CoordinationPlanePort` / `ExecutionPlanePort` / `FlowMetricsPort`), so
//! the bin cannot reach a concrete store except through the ports. The named
//! owned cutover target for both planes is the cloud/cloud-ci-owned
//! loop-state service recorded in
//! `specs/fabric-drive-loop-state.json#cutover_target`; once that service
//! lands, this bin retires with the filesystem bridge adapters it fronts.
//!
//! Merge authority stays the single required `oya-ci-required` context
//! (ADR-0515); plan authority stays `/specs/masterplan.json`. Mutations here
//! touch only the local bridge stores: the in-repo PR-governed coordination
//! plane (whose changes still ride coordinator-only commits through a
//! protected PR) and the gitignored operational execution plane.
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use fabric_loop_state_app::{
    BlockKind, CardFlowTimeline, CardStatus, CutoverTarget, DEFAULT_DURABLE_PLANE_ROOT,
    DEFAULT_FLOW_METRICS_ROOT, DEFAULT_OPERATIONAL_PLANE_ROOT, DisjointnessPhase,
    DisjointnessVerdict, FlowMetricsService, FsCoordinationStore, FsExecutionStore,
    FsFlowMetricsStore, LaneWorkSurface, LoopCard, LoopStateService, PlaneError,
    check_lane_disjointness,
};

const USAGE: &str = "oya-fabric-loop-state [--repo-root PATH] <command>
Local lane-runtime bridge over the two-plane drive-loop state ports.
No merge, CI, or plan authority; retirement-marked (cutover target:
specs/fabric-drive-loop-state.json#cutover_target).

commands:
  snapshot                                   read-only two-plane state snapshot (default)
  define-card --card-id ID --title T --program P [--depends-on DEP]...
                                             define a Definition/Seed-class card (durable plane)
  claim       --card-id ID --lane L          claim a DAG-ready card for a lane
  start       --card-id ID --lane L          mark the claimed run as running
  heartbeat   --card-id ID --lane L          record a liveness heartbeat
  block       --card-id ID --lane L --kind K --note N
                                             mark the run blocked (typed block kind)
  complete    --card-id ID --lane L --evidence REF...
                                             complete with evidence -> claimed-done-unverified
  verify-done --card-id ID --evidence REF    promote to done-verified with verification evidence
  record-pass [--timeline CARD,LANE,CLAIMED,REVIEW_REQ,FIRST_VERDICT,COMPLETED,ROUNDS]...
                                             append the next per-pass flow-metrics record
                                             (idle passes record with zero timelines)
  check-disjoint --phase pre-flight|post-run --reviewer-capacity N
                 [--surface LANE:CARD=PATH[,PATH]...]...
                                             mechanical lane-disjointness verdict (path/ownership
                                             overlap): prints the JSON report; exits non-zero
                                             unless every lane pair is path-disjoint";

/// Wall-clock epoch seconds. Fails closed on clock error: records must never
/// be stamped with a fabricated epoch-0 time.
fn now_epoch_s() -> Result<u64, PlaneError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|err| PlaneError::Io(format!("system clock before unix epoch: {err}")))
}

/// Bin-local error surface separating operator CLI misuse from genuine
/// store/plane failures. `Usage` covers malformed arguments that never
/// touched a store and prints with a `usage error: ` prefix; `Plane` wraps a
/// real `PlaneError` and preserves its Display text unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
enum CliError {
    /// The operator invoked the bin with malformed arguments.
    Usage(String),
    /// A genuine store/plane failure surfaced through the port facades.
    Plane(PlaneError),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage(detail) => write!(f, "usage error: {detail}"),
            Self::Plane(err) => write!(f, "{err}"),
        }
    }
}

impl From<PlaneError> for CliError {
    fn from(err: PlaneError) -> Self {
        Self::Plane(err)
    }
}

struct Args {
    args: Vec<String>,
    cursor: usize,
}

impl Args {
    fn next(&mut self) -> Option<String> {
        let value = self.args.get(self.cursor).cloned();
        if value.is_some() {
            self.cursor += 1;
        }
        value
    }

    fn required_value(&mut self, flag: &str) -> Result<String, String> {
        self.next()
            .ok_or_else(|| format!("{flag} requires a value"))
    }
}

struct Runtime {
    loop_state: LoopStateService<FsCoordinationStore, FsExecutionStore>,
    flow_metrics: FlowMetricsService<FsFlowMetricsStore>,
}

impl Runtime {
    fn open(repo_root: &Path) -> Self {
        Self {
            loop_state: LoopStateService::new(
                FsCoordinationStore::open(repo_root.join(DEFAULT_DURABLE_PLANE_ROOT)),
                FsExecutionStore::open(repo_root.join(DEFAULT_OPERATIONAL_PLANE_ROOT)),
            ),
            flow_metrics: FlowMetricsService::new(FsFlowMetricsStore::open(
                repo_root.join(DEFAULT_FLOW_METRICS_ROOT),
            )),
        }
    }
}

fn main() -> ExitCode {
    let mut args = Args {
        args: std::env::args().skip(1).collect(),
        cursor: 0,
    };
    let mut repo_root = PathBuf::from(".");
    let mut command: Option<String> = None;
    let mut card_id = String::new();
    let mut lane_id = String::new();
    let mut title = String::new();
    let mut program = String::new();
    let mut kind = String::new();
    let mut note = String::new();
    let mut depends_on: Vec<String> = Vec::new();
    let mut evidence: Vec<String> = Vec::new();
    let mut timelines: Vec<String> = Vec::new();
    let mut phase = String::new();
    let mut reviewer_capacity = String::new();
    let mut surfaces: Vec<String> = Vec::new();

    while let Some(arg) = args.next() {
        let flag_value = |args: &mut Args, flag: &str| args.required_value(flag);
        match arg.as_str() {
            "--repo-root" => match flag_value(&mut args, "--repo-root") {
                Ok(value) => repo_root = PathBuf::from(value),
                Err(err) => return fail(&err),
            },
            "--card-id" => match flag_value(&mut args, "--card-id") {
                Ok(value) => card_id = value,
                Err(err) => return fail(&err),
            },
            "--lane" => match flag_value(&mut args, "--lane") {
                Ok(value) => lane_id = value,
                Err(err) => return fail(&err),
            },
            "--title" => match flag_value(&mut args, "--title") {
                Ok(value) => title = value,
                Err(err) => return fail(&err),
            },
            "--program" => match flag_value(&mut args, "--program") {
                Ok(value) => program = value,
                Err(err) => return fail(&err),
            },
            "--kind" => match flag_value(&mut args, "--kind") {
                Ok(value) => kind = value,
                Err(err) => return fail(&err),
            },
            "--note" => match flag_value(&mut args, "--note") {
                Ok(value) => note = value,
                Err(err) => return fail(&err),
            },
            "--depends-on" => match flag_value(&mut args, "--depends-on") {
                Ok(value) => depends_on.push(value),
                Err(err) => return fail(&err),
            },
            "--evidence" => match flag_value(&mut args, "--evidence") {
                Ok(value) => evidence.push(value),
                Err(err) => return fail(&err),
            },
            "--timeline" => match flag_value(&mut args, "--timeline") {
                Ok(value) => timelines.push(value),
                Err(err) => return fail(&err),
            },
            "--phase" => match flag_value(&mut args, "--phase") {
                Ok(value) => phase = value,
                Err(err) => return fail(&err),
            },
            "--reviewer-capacity" => match flag_value(&mut args, "--reviewer-capacity") {
                Ok(value) => reviewer_capacity = value,
                Err(err) => return fail(&err),
            },
            "--surface" => match flag_value(&mut args, "--surface") {
                Ok(value) => surfaces.push(value),
                Err(err) => return fail(&err),
            },
            "--help" | "-h" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            other if command.is_none() && !other.starts_with('-') => {
                command = Some(other.to_owned());
            }
            other => return fail(&format!("unknown argument: {other}")),
        }
    }

    let mut runtime = Runtime::open(&repo_root);
    let at = match now_epoch_s() {
        Ok(at) => at,
        Err(err) => return fail(&err.to_string()),
    };
    let result: Result<(), CliError> = match command.as_deref().unwrap_or("snapshot") {
        "snapshot" => return snapshot(&runtime),
        "define-card" => define_card(
            &mut runtime,
            &card_id,
            &title,
            &program,
            depends_on.clone(),
        ),
        "claim" => runtime
            .loop_state
            .claim_ready(&card_id, &lane_id, at)
            .map(|claim| {
                println!(
                    "claimed {} for lane {} at {}",
                    claim.card_id, claim.lane_id, claim.claimed_at_epoch_s
                );
            })
            .map_err(CliError::from),
        "start" => runtime
            .loop_state
            .start_run(&card_id, &lane_id, at)
            .map(|()| println!("run started for {card_id} on lane {lane_id} at {at}"))
            .map_err(CliError::from),
        "heartbeat" => runtime
            .loop_state
            .heartbeat(&card_id, &lane_id, at)
            .map(|beat| {
                println!(
                    "heartbeat for {} on lane {} at {}",
                    beat.card_id, beat.lane_id, beat.beat_at_epoch_s
                );
            })
            .map_err(CliError::from),
        "block" => BlockKind::parse(&kind)
            .and_then(|kind| {
                runtime
                    .loop_state
                    .mark_blocked(&card_id, &lane_id, kind, &note, at)
                    .map(|()| println!("{card_id} blocked ({}) on lane {lane_id}", kind.as_str()))
            })
            .map_err(CliError::from),
        "complete" => runtime
            .loop_state
            .complete(&card_id, &lane_id, &evidence, at)
            .map(|()| {
                println!(
                    "{card_id} completed on lane {lane_id} at {at} -> claimed-done-unverified ({} evidence refs)",
                    evidence.len()
                );
            })
            .map_err(CliError::from),
        "verify-done" => verify_done_evidence(&evidence).and_then(|reference| {
            runtime
                .loop_state
                .verify_done(&card_id, reference)
                .map(|()| println!("{card_id} -> done-verified"))
                .map_err(CliError::from)
        }),
        "record-pass" => record_pass(&mut runtime, &timelines, at),
        "check-disjoint" => {
            return check_disjoint(&phase, &reviewer_capacity, &surfaces, at);
        }
        other => {
            eprintln!("unknown command: {other}\n{USAGE}");
            return ExitCode::FAILURE;
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => fail(&err.to_string()),
    }
}

fn fail(message: &str) -> ExitCode {
    eprintln!("{message}");
    ExitCode::FAILURE
}

/// Validate the `verify-done` `--evidence` arity: exactly one REF. Arity
/// violations are operator usage errors, not store corruption.
fn verify_done_evidence(evidence: &[String]) -> Result<&str, CliError> {
    match evidence {
        [reference] => Ok(reference.as_str()),
        _ => Err(CliError::Usage(format!(
            "verify-done requires exactly one --evidence REF; got {}",
            evidence.len()
        ))),
    }
}

/// Validate the `define-card` required flags. Missing or blank `--title` /
/// `--program` is an operator usage error, not store corruption.
fn require_define_card_flags(title: &str, program: &str) -> Result<(), CliError> {
    if title.trim().is_empty() || program.trim().is_empty() {
        return Err(CliError::Usage(
            "define-card requires --title and --program".into(),
        ));
    }
    Ok(())
}

fn define_card(
    runtime: &mut Runtime,
    card_id: &str,
    title: &str,
    program: &str,
    depends_on: Vec<String>,
) -> Result<(), CliError> {
    require_define_card_flags(title, program)?;
    runtime.loop_state.define_card(&LoopCard {
        card_id: card_id.to_owned(),
        title: title.to_owned(),
        program_id: program.to_owned(),
        depends_on,
        status: CardStatus::Defined,
        evidence_refs: Vec::new(),
    })?;
    println!("defined card {card_id} (program {program})");
    Ok(())
}

fn parse_timeline(raw: &str) -> Result<CardFlowTimeline, CliError> {
    let parts: Vec<&str> = raw.split(',').collect();
    let [
        card_id,
        lane_id,
        claimed,
        review_req,
        first_verdict,
        completed,
        rounds,
    ] = parts[..]
    else {
        return Err(CliError::Usage(format!(
            "--timeline expects CARD,LANE,CLAIMED,REVIEW_REQ,FIRST_VERDICT,COMPLETED,ROUNDS; got {raw:?}"
        )));
    };
    let num = |label: &str, text: &str| -> Result<u64, CliError> {
        text.parse::<u64>()
            .map_err(|_| CliError::Usage(format!("--timeline {label} is not a u64: {text:?}")))
    };
    Ok(CardFlowTimeline {
        card_id: card_id.to_owned(),
        lane_id: lane_id.to_owned(),
        claimed_at_epoch_s: num("CLAIMED", claimed)?,
        review_requested_at_epoch_s: num("REVIEW_REQ", review_req)?,
        review_first_verdict_at_epoch_s: num("FIRST_VERDICT", first_verdict)?,
        completed_at_epoch_s: num("COMPLETED", completed)?,
        review_rounds: num("ROUNDS", rounds)?,
    })
}

fn record_pass(runtime: &mut Runtime, timelines: &[String], at: u64) -> Result<(), CliError> {
    let parsed = timelines
        .iter()
        .map(|raw| parse_timeline(raw))
        .collect::<Result<Vec<_>, _>>()?;
    let pass = runtime.flow_metrics.record_next_pass(&parsed, at)?;
    println!(
        "recorded flow-metrics pass {} at {} ({} cards, rework {})",
        pass.pass_seq,
        pass.recorded_at_epoch_s,
        pass.cards_measured(),
        pass.total_rework_count()
    );
    Ok(())
}

/// Parse one `--surface LANE:CARD=PATH[,PATH]...` spec.
fn parse_surface(raw: &str) -> Result<LaneWorkSurface, CliError> {
    let malformed = || {
        CliError::Usage(format!(
            "--surface expects LANE:CARD=PATH[,PATH]...; got {raw:?}"
        ))
    };
    let (ids, paths) = raw.split_once('=').ok_or_else(malformed)?;
    let (lane_id, card_id) = ids.split_once(':').ok_or_else(malformed)?;
    if lane_id.is_empty() || card_id.is_empty() || paths.is_empty() {
        return Err(malformed());
    }
    Ok(LaneWorkSurface {
        lane_id: lane_id.to_owned(),
        card_id: card_id.to_owned(),
        paths: paths.split(',').map(str::to_owned).collect(),
    })
}

/// Run the mechanical lane-disjointness detector and print the JSON report.
/// Gate semantics: exits zero ONLY on a `disjoint` verdict — an overlap
/// verdict prints the report (evidence) and fails, routing the colliding
/// work to the serialized integrator lane instead of parallel dispatch.
fn check_disjoint(
    phase: &str,
    reviewer_capacity: &str,
    surface_specs: &[String],
    at: u64,
) -> ExitCode {
    let run = || -> Result<fabric_loop_state_app::DisjointnessReport, CliError> {
        let phase = DisjointnessPhase::parse(phase)?;
        let capacity = reviewer_capacity.parse::<usize>().map_err(|_| {
            PlaneError::InvalidSurface {
                lane_id: "<cli>".into(),
                detail: format!(
                    "--reviewer-capacity must be a non-negative integer; got {reviewer_capacity:?}"
                ),
            }
        })?;
        let surfaces = surface_specs
            .iter()
            .map(|raw| parse_surface(raw))
            .collect::<Result<Vec<_>, _>>()?;
        check_lane_disjointness(phase, &surfaces, capacity, at).map_err(CliError::from)
    };
    match run() {
        Ok(report) => {
            print!("{}", report.to_json().to_canonical_string());
            match report.verdict {
                DisjointnessVerdict::Disjoint => ExitCode::SUCCESS,
                DisjointnessVerdict::OverlapSerializeToIntegrator { .. } => {
                    eprintln!(
                        "overlap detected: colliding lanes route to the serialized integrator lane"
                    );
                    ExitCode::FAILURE
                }
            }
        }
        Err(err) => fail(&err.to_string()),
    }
}

fn snapshot(runtime: &Runtime) -> ExitCode {
    let cards = match runtime.loop_state.cards() {
        Ok(cards) => cards,
        Err(err) => {
            eprintln!("coordination plane read failed: {err}");
            return ExitCode::FAILURE;
        }
    };
    let (coordination_desc, execution_desc) = runtime.loop_state.plane_descriptors();
    let target = CutoverTarget::canonical();
    println!("two-plane drive-loop state snapshot (read-only bridge view)");
    println!(
        "  coordination plane: {} ({} cards)",
        coordination_desc.store_root,
        cards.len()
    );
    println!("  execution plane:    {}", execution_desc.store_root);
    for card in &cards {
        let claim = match runtime.loop_state.active_claim(&card.card_id) {
            Ok(claim) => claim,
            Err(err) => {
                eprintln!("execution plane read failed for {}: {err}", card.card_id);
                return ExitCode::FAILURE;
            }
        };
        let lane = claim
            .map(|c| format!(" [claimed by {}]", c.lane_id))
            .unwrap_or_default();
        println!(
            "  - {} status={} deps={:?} evidence={}{}",
            card.card_id,
            card.status.as_str(),
            card.depends_on,
            card.evidence_refs.len(),
            lane
        );
    }
    match runtime.flow_metrics.latest_pass() {
        Ok(Some(pass)) => println!(
            "  flow metrics:       pass {} at {} ({} cards, rework {}, max cycle {:?}s, max review latency {:?}s)",
            pass.pass_seq,
            pass.recorded_at_epoch_s,
            pass.cards_measured(),
            pass.total_rework_count(),
            pass.max_cycle_time_s(),
            pass.max_review_latency_s()
        ),
        Ok(None) => println!(
            "  flow metrics:       {} (no passes recorded)",
            runtime.flow_metrics.descriptor().store_root
        ),
        Err(err) => {
            eprintln!("flow-metrics read failed: {err}");
            return ExitCode::FAILURE;
        }
    }
    println!(
        "  cutover target: {} (home {}, owner {})",
        target.service_name, target.destination_home, target.owner
    );
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_done_arity_violation_is_usage_error_not_plane() {
        let none: Vec<String> = Vec::new();
        assert!(matches!(
            verify_done_evidence(&none),
            Err(CliError::Usage(_))
        ));
        let two = vec!["ref-a".to_owned(), "ref-b".to_owned()];
        let err = verify_done_evidence(&two).unwrap_err();
        assert_eq!(
            err,
            CliError::Usage("verify-done requires exactly one --evidence REF; got 2".into())
        );
        assert_eq!(
            err.to_string(),
            "usage error: verify-done requires exactly one --evidence REF; got 2"
        );
    }

    #[test]
    fn verify_done_single_evidence_is_accepted() {
        let one = vec!["evidence:ref".to_owned()];
        assert_eq!(verify_done_evidence(&one), Ok("evidence:ref"));
    }

    #[test]
    fn define_card_missing_flags_are_usage_errors_not_plane() {
        let err = require_define_card_flags("", "program").unwrap_err();
        assert_eq!(
            err,
            CliError::Usage("define-card requires --title and --program".into())
        );
        assert!(matches!(
            require_define_card_flags("title", "   "),
            Err(CliError::Usage(_))
        ));
        assert_eq!(require_define_card_flags("title", "program"), Ok(()));
    }

    #[test]
    fn malformed_timeline_specs_are_usage_errors() {
        // Wrong field count.
        assert!(matches!(
            parse_timeline("card,lane,1,2,3"),
            Err(CliError::Usage(_))
        ));
        // Non-u64 numeric field.
        assert!(matches!(
            parse_timeline("card,lane,nope,2,3,4,5"),
            Err(CliError::Usage(_))
        ));
        // Well-formed spec still parses.
        let timeline = parse_timeline("card,lane,1,2,3,4,5").expect("well-formed timeline");
        assert_eq!(timeline.card_id, "card");
        assert_eq!(timeline.lane_id, "lane");
        assert_eq!(timeline.review_rounds, 5);
    }

    #[test]
    fn malformed_surface_specs_are_usage_errors() {
        for bad in [
            "no-equals",
            ":card=path",
            "lane:=path",
            "lane:card=",
            "lanecard=path",
        ] {
            assert!(
                matches!(parse_surface(bad), Err(CliError::Usage(_))),
                "surface spec {bad:?} must be a usage error"
            );
        }
        let surface = parse_surface("lane:card=a/b,c/d").expect("well-formed surface");
        assert_eq!(surface.lane_id, "lane");
        assert_eq!(surface.card_id, "card");
        assert_eq!(surface.paths, vec!["a/b".to_owned(), "c/d".to_owned()]);
    }

    #[test]
    fn plane_errors_convert_to_plane_and_keep_display_text() {
        let plane = PlaneError::Io("store read failed".into());
        let expected = plane.to_string();
        let err = CliError::from(plane.clone());
        assert_eq!(err, CliError::Plane(plane));
        assert_eq!(err.to_string(), expected);
        assert!(!err.to_string().starts_with("usage error: "));
    }
}
