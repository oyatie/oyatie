//! Binary entrypoint: forensically ingest the Hermes board snapshot and
//! materialize (a) the per-card done-claim entries spliced into
//! /specs/masterplan.json#masterplan_v2 and (b) the forensic ledger evidence
//! artifact. Read-only toward the board; single-writer toward the two owned
//! masterplan keys; no shell/python in the extraction path.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::ExitCode;

use oya_masterplan_hermes_forensic_ledger_app::json::{Json, to_pretty};
use oya_masterplan_hermes_forensic_ledger_app::ledger::{
    build_claims, extract_board, insert_key_before, iso_utc, replace_key_value, scan_evidence_refs,
};
use oya_masterplan_hermes_forensic_ledger_app::sha256::sha256_hex;
use oya_masterplan_hermes_forensic_ledger_app::sqlite::SqliteDb;

const USAGE: &str = "usage: oya-masterplan-hermes-forensic-ledger \
--db <snapshot.db> --repo-root <dir> --ledger-out <repo-relative.json> \
[--source-label <path-label>] [--source-board <board>] [--seed-done-count <n>]";

const DEFAULT_SOURCE_LABEL: &str = "~/.hermes/kanban/boards/oyatie/kanban.db";
const READER_CONTRACT: &str =
    "read-only forensic ingest via owned-Rust SQLite reader; no shell/python board extraction";
const TOOL_REF: &str = "tools/oya-masterplan-hermes-forensic-ledger-app";
const MASTERPLAN_LEDGER_REF: &str = "/specs/masterplan.json#masterplan_v2.hermes_done_card_imports";
const INVARIANT: &str = "no hermes done-card claim may carry a verified/done masterplan_status \
without attached evidence refs; every import is claimed-done-unverified and evidence attachment \
is a mechanical cross-reference, not verification";
const ENFORCEMENT: &str = "cloud/cloud-ci/gates/oya-cloud-ci-cross-artifact-agreement-app \
(masterplan_evidence_state_invalid + masterplan_plan_evidence_drift findings; live-corpus \
contract tests in tests/cross_artifact_agreement.rs)";
const EVIDENCE_REF_SEMANTICS: &str = "an attached evidence_ref is a repo evidence artifact that \
mentions the card id, or a merged-PR/gate-run/review URL recorded in the card's own result \
text; attachment upgrades evidence_state to evidence-attached but never upgrades \
masterplan_status past claimed-done-unverified";

struct Args {
    db: PathBuf,
    repo_root: PathBuf,
    ledger_out: String,
    source_label: String,
    source_board: String,
    seed_done_count: i64,
}

fn parse_args() -> Result<Args, String> {
    let mut db = None;
    let mut repo_root = None;
    let mut ledger_out = None;
    let mut source_label = DEFAULT_SOURCE_LABEL.to_owned();
    let mut source_board = "oyatie".to_owned();
    let mut seed_done_count = 814i64;
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let mut value = || args.next().ok_or(format!("missing value for {flag}"));
        match flag.as_str() {
            "--db" => db = Some(PathBuf::from(value()?)),
            "--repo-root" => repo_root = Some(PathBuf::from(value()?)),
            "--ledger-out" => ledger_out = Some(value()?),
            "--source-label" => source_label = value()?,
            "--source-board" => source_board = value()?,
            "--seed-done-count" => {
                seed_done_count = value()?
                    .parse()
                    .map_err(|e| format!("bad --seed-done-count: {e}"))?;
            }
            other => return Err(format!("unknown flag {other}\n{USAGE}")),
        }
    }
    Ok(Args {
        db: db.ok_or(format!("--db is required\n{USAGE}"))?,
        repo_root: repo_root.ok_or(format!("--repo-root is required\n{USAGE}"))?,
        ledger_out: ledger_out.ok_or(format!("--ledger-out is required\n{USAGE}"))?,
        source_label,
        source_board,
        seed_done_count,
    })
}

fn run() -> Result<(), String> {
    let args = parse_args()?;

    let db_bytes = std::fs::read(&args.db).map_err(|e| format!("read {:?}: {e}", args.db))?;
    let snapshot_sha256 = sha256_hex(&db_bytes);
    let snapshot_mtime = std::fs::metadata(&args.db)
        .and_then(|m| m.modified())
        .map_err(|e| format!("mtime {:?}: {e}", args.db))?
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("mtime before epoch: {e}"))?
        .as_secs();
    let snapshot_mtime_utc = iso_utc(i64::try_from(snapshot_mtime).map_err(|e| e.to_string())?);

    let db = SqliteDb::open(db_bytes)?;
    let board = extract_board(&db)?;
    let ids: BTreeSet<String> = board.done_cards.iter().map(|c| c.id.clone()).collect();
    // The tool's own outputs never count as evidence: the ledger artifact
    // mentions every card id by construction, so counting it would launder
    // every claim into evidence-attached on regeneration.
    let excluded_rel: BTreeSet<String> = [args.ledger_out.clone()].into();
    let corpus_refs = scan_evidence_refs(&args.repo_root, "evidence", &ids, &excluded_rel)?;
    let built = build_claims(&board.done_cards, &corpus_refs);

    let extracted_done_count = i64::try_from(board.done_cards.len()).map_err(|e| e.to_string())?;
    let status_counts = Json::Obj(
        board
            .status_counts
            .iter()
            .map(|(status, count)| {
                (
                    status.clone(),
                    Json::Int(i64::try_from(*count).unwrap_or(i64::MAX)),
                )
            })
            .collect(),
    );

    let count_drift_note = format!(
        "the Seed snapshot declared {} done cards; the live board advanced to {} done cards by \
the forensic snapshot pinned at sha256 {}; the extracted count is authoritative for this \
ledger and the Seed count is retained as ingest provenance",
        args.seed_done_count, extracted_done_count, snapshot_sha256
    );

    let summary = Json::Obj(vec![
        ("source_system".into(), Json::str("hermes")),
        ("source_board".into(), Json::str(&args.source_board)),
        ("source_path".into(), Json::str(&args.source_label)),
        ("snapshot_sha256".into(), Json::str(&snapshot_sha256)),
        ("snapshot_mtime_utc".into(), Json::str(&snapshot_mtime_utc)),
        ("board_status_counts".into(), status_counts.clone()),
        (
            "seed_declared_done_count".into(),
            Json::Int(args.seed_done_count),
        ),
        (
            "extracted_done_count".into(),
            Json::Int(extracted_done_count),
        ),
        ("count_drift_note".into(), Json::str(&count_drift_note)),
        ("verified_count".into(), Json::Int(0)),
        (
            "evidence_attached_count".into(),
            Json::Int(i64::try_from(built.evidence_attached_count).map_err(|e| e.to_string())?),
        ),
        (
            "unverified_pending_evidence_count".into(),
            Json::Int(i64::try_from(built.unverified_pending_count).map_err(|e| e.to_string())?),
        ),
        ("invariant".into(), Json::str(INVARIANT)),
        ("invariant_enforcement".into(), Json::str(ENFORCEMENT)),
        (
            "evidence_ref_semantics".into(),
            Json::str(EVIDENCE_REF_SEMANTICS),
        ),
        ("extraction_tool".into(), Json::str(TOOL_REF)),
        ("reader_contract".into(), Json::str(READER_CONTRACT)),
        (
            "single_writer".into(),
            Json::str(format!(
                "{TOOL_REF} is the only writer of masterplan_v2.hermes_done_card_imports and \
masterplan_v2.hermes_done_card_import_summary; hand edits are drift and regeneration is \
idempotent for an identical snapshot"
            )),
        ),
        ("ledger_artifact".into(), Json::str(&args.ledger_out)),
    ]);

    let artifact_id = args
        .ledger_out
        .rsplit('/')
        .next()
        .unwrap_or(&args.ledger_out)
        .trim_end_matches(".json")
        .to_owned();
    let card_rows: Vec<Json> = board
        .done_cards
        .iter()
        .zip(built.claims.iter())
        .map(|(card, claim)| {
            let (evidence_state, evidence_refs) = match claim {
                Json::Obj(members) => (
                    members
                        .iter()
                        .find(|(k, _)| k == "evidence_state")
                        .map(|(_, v)| v.clone())
                        .unwrap_or(Json::Null),
                    members
                        .iter()
                        .find(|(k, _)| k == "evidence_refs")
                        .map(|(_, v)| v.clone())
                        .unwrap_or(Json::Arr(Vec::new())),
                ),
                _ => (Json::Null, Json::Arr(Vec::new())),
            };
            let mut row = vec![
                ("source_card_id".into(), Json::str(card.id.clone())),
                ("title".into(), Json::str(card.title.clone())),
                (
                    "completed_at_utc".into(),
                    card.completed_at
                        .map_or(Json::Null, |t| Json::str(iso_utc(t))),
                ),
                (
                    "masterplan_status".into(),
                    Json::str("claimed-done-unverified"),
                ),
                ("evidence_state".into(), evidence_state),
                ("evidence_refs".into(), evidence_refs),
            ];
            if let Some(branch) = &card.branch_name {
                row.push(("branch_name".into(), Json::str(branch.clone())));
            }
            if let Some(result) = &card.result {
                row.push(("source_result_text".into(), Json::str(result.clone())));
            }
            Json::Obj(row)
        })
        .collect();

    let ledger_artifact = Json::Obj(vec![
        ("artifact_id".into(), Json::str(&artifact_id)),
        (
            "purpose".into(),
            Json::str(
                "Forensic provenance ledger for every Hermes done-card completion claim \
imported into masterplan v2: full card identity, completion timestamps, evidence \
cross-references where they already exist, and the flagged unverified remainder.",
            ),
        ),
        (
            "claim_ceiling".into(),
            Json::str(
                "Forensic provenance only. No completion, verification, readiness, or \
production claim; attached evidence refs are mechanical cross-references pending \
verification.",
            ),
        ),
        (
            "masterplan_ledger_ref".into(),
            Json::str(MASTERPLAN_LEDGER_REF),
        ),
        ("source_snapshot".into(), summary.clone()),
        ("done_cards".into(), Json::Arr(card_rows)),
    ]);

    let ledger_path = args.repo_root.join(&args.ledger_out);
    if let Some(parent) = ledger_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir {parent:?}: {e}"))?;
    }
    let mut ledger_text = to_pretty(&ledger_artifact, 0);
    ledger_text.push('\n');
    std::fs::write(&ledger_path, ledger_text).map_err(|e| format!("write {ledger_path:?}: {e}"))?;

    let masterplan_path = args.repo_root.join("specs/masterplan.json");
    let masterplan_text = std::fs::read_to_string(&masterplan_path)
        .map_err(|e| format!("read {masterplan_path:?}: {e}"))?;
    let spliced = replace_key_value(
        &masterplan_text,
        "hermes_done_card_imports",
        &Json::Arr(built.claims),
    )?;
    let spliced = insert_key_before(
        &spliced,
        "hermes_done_card_import_summary",
        &summary,
        "hermes_done_card_imports",
    )?;
    std::fs::write(&masterplan_path, spliced)
        .map_err(|e| format!("write {masterplan_path:?}: {e}"))?;

    println!(
        "hermes forensic ledger: {} done cards ({} evidence-attached, {} unverified-pending-evidence, 0 verified); snapshot sha256 {}; ledger {}; masterplan spliced",
        extracted_done_count,
        built.evidence_attached_count,
        built.unverified_pending_count,
        snapshot_sha256,
        args.ledger_out
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("oya-masterplan-hermes-forensic-ledger: {err}");
            ExitCode::FAILURE
        }
    }
}
