//! `oya gate validate loop-recovery-patterns` runner.
//!
//! This gate makes the autonomous-Foundry repeat-mistake harness executable
//! without expanding shell-hook logic: `scripts/hooks/pre-push.sh` continues
//! to delegate to `oya verify`, while `oya verify`/`gate run-all` invokes this
//! Rust lane. The lane validates the deterministic `score_cards` contract from
//! `specs/agent-durable-goal.json`, the concrete score-card inventory, and the
//! repeat-loop patterns under `registry/loop-recovery-patterns/`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

use crate::usage;

const REQUIRED_SCORE_CARD_FIELDS: &[&str] = &[
    "id",
    "facet",
    "query",
    "pass_criterion",
    "score",
    "severity_tier",
];

const ALLOWED_PATTERN_STATUSES: &[&str] = &["active", "candidate", "retired"];
const ACTIVE_SCORE_CARD_STATUS: &str = "active";
const ADVISORY_SCORE_CARD_STATUS: &str = "advisory-until-validator";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LoopRecoveryPatternsValidateArgs {
    agent_durable_goal_path: PathBuf,
    score_cards_path: PathBuf,
    patterns_dir: PathBuf,
    mistakes_ledger_path: PathBuf,
}

impl Default for LoopRecoveryPatternsValidateArgs {
    fn default() -> Self {
        Self {
            agent_durable_goal_path: PathBuf::from("specs/agent-durable-goal.json"),
            score_cards_path: PathBuf::from("specs/score-cards.json"),
            patterns_dir: PathBuf::from("registry/loop-recovery-patterns"),
            mistakes_ledger_path: PathBuf::from("registry/mistakes-ledger.json"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LoopRecoveryPatternsReport {
    pub score_card_schema_fields_checked: usize,
    pub score_cards_checked: usize,
    pub score_card_commands_executed: usize,
    pub patterns_checked: usize,
    pub active_blockers_checked: usize,
    pub mistakes_ledger_refs_checked: usize,
    pub anomaly_watch_signals_checked: usize,
}

pub(crate) fn parse_loop_recovery_patterns_validate_args(
    args: Vec<String>,
) -> Result<LoopRecoveryPatternsValidateArgs, String> {
    let mut parsed = LoopRecoveryPatternsValidateArgs::default();
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(value) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--agent-durable-goal" => parsed.agent_durable_goal_path = PathBuf::from(value),
            "--score-cards" => parsed.score_cards_path = PathBuf::from(value),
            "--patterns-dir" => parsed.patterns_dir = PathBuf::from(value),
            "--mistakes-ledger" => parsed.mistakes_ledger_path = PathBuf::from(value),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_loop_recovery_patterns_gate(
    args: LoopRecoveryPatternsValidateArgs,
) -> Result<LoopRecoveryPatternsReport, String> {
    let agent_goal = read_json("agent-durable-goal", &args.agent_durable_goal_path)?;
    let score_card_schema_fields_checked = validate_agent_durable_goal_score_cards(
        &agent_goal,
        &args.agent_durable_goal_path,
        &args.score_cards_path,
    )?;
    let anomaly_watch_signals_checked =
        validate_autonomous_foundry_loop_references(&agent_goal, &args.agent_durable_goal_path)?;

    let score_card_inventory = read_score_card_inventory(&args.score_cards_path)?;
    let mistake_ids = read_mistakes_ledger_ids(&args.mistakes_ledger_path)?;
    let pattern_report = validate_loop_patterns(
        &args.patterns_dir,
        &score_card_inventory.score_card_ids,
        &mistake_ids,
        &args.score_cards_path,
        &args.mistakes_ledger_path,
    )?;

    Ok(LoopRecoveryPatternsReport {
        score_card_schema_fields_checked,
        score_cards_checked: score_card_inventory.score_card_ids.len(),
        score_card_commands_executed: score_card_inventory.score_card_commands_executed,
        patterns_checked: pattern_report.patterns_checked,
        active_blockers_checked: pattern_report.active_blockers_checked,
        mistakes_ledger_refs_checked: pattern_report.mistakes_ledger_refs_checked,
        anomaly_watch_signals_checked,
    })
}

fn validate_agent_durable_goal_score_cards(
    agent_goal: &Value,
    agent_goal_path: &Path,
    score_cards_path: &Path,
) -> Result<usize, String> {
    let score_cards = required_object(agent_goal, "score_cards", agent_goal_path)?;
    let description = required_str_in_object(score_cards, "description", agent_goal_path)?;
    require_keywords(
        "score_cards.description",
        description,
        &["deterministic", "llm", "pass/fail"],
        agent_goal_path,
    )?;
    let design_principle =
        required_str_in_object(score_cards, "design_principle", agent_goal_path)?;
    require_keywords(
        "score_cards.design_principle",
        design_principle,
        &["llm", "forbidden", "pass/fail"],
        agent_goal_path,
    )?;

    let check_schema = score_cards
        .get("check_schema")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            format!(
                "{} missing score_cards.check_schema object",
                agent_goal_path.display()
            )
        })?;
    for field in REQUIRED_SCORE_CARD_FIELDS {
        if !check_schema.contains_key(*field) {
            return Err(format!(
                "{} score_cards.check_schema missing required field `{field}`",
                agent_goal_path.display()
            ));
        }
    }

    let spec_path = required_str_in_object(score_cards, "spec_path", agent_goal_path)?;
    let expected = score_cards_path.to_string_lossy();
    if !spec_path.contains(expected.as_ref()) {
        return Err(format!(
            "{} score_cards.spec_path must reference `{}`",
            agent_goal_path.display(),
            score_cards_path.display()
        ));
    }
    Ok(REQUIRED_SCORE_CARD_FIELDS.len())
}

fn validate_autonomous_foundry_loop_references(
    agent_goal: &Value,
    agent_goal_path: &Path,
) -> Result<usize, String> {
    let autonomous_foundry = required_object(agent_goal, "autonomous_foundry", agent_goal_path)?;
    let stuck_loop = autonomous_foundry
        .get("stuck_loop_recovery")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            format!(
                "{} missing autonomous_foundry.stuck_loop_recovery object",
                agent_goal_path.display()
            )
        })?;
    let step_3 = required_str_in_object(stuck_loop, "step_3_if_resolves", agent_goal_path)?;
    require_keywords(
        "autonomous_foundry.stuck_loop_recovery.step_3_if_resolves",
        step_3,
        &["registry/loop-recovery-patterns"],
        agent_goal_path,
    )?;

    let first_of_kind = autonomous_foundry
        .get("first_of_kind_protocol")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            format!(
                "{} missing autonomous_foundry.first_of_kind_protocol object",
                agent_goal_path.display()
            )
        })?;
    let detection = required_str_in_object(first_of_kind, "detection", agent_goal_path)?;
    require_keywords(
        "autonomous_foundry.first_of_kind_protocol.detection",
        detection,
        &["registry/loop-recovery-patterns", "registry/incidents"],
        agent_goal_path,
    )?;

    let anomaly_watch = autonomous_foundry
        .get("meta_agent_anomaly_watch")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            format!(
                "{} missing autonomous_foundry.meta_agent_anomaly_watch object",
                agent_goal_path.display()
            )
        })?;
    let watched_signals = anomaly_watch
        .get("watched_signals")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            format!(
                "{} missing autonomous_foundry.meta_agent_anomaly_watch.watched_signals array",
                agent_goal_path.display()
            )
        })?;
    if watched_signals.len() < 5 {
        return Err(format!(
            "{} meta_agent_anomaly_watch requires at least 5 watched signals",
            agent_goal_path.display()
        ));
    }
    Ok(watched_signals.len())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ScoreCardInventoryReport {
    score_card_ids: BTreeSet<String>,
    score_card_commands_executed: usize,
}

fn read_score_card_inventory(path: &Path) -> Result<ScoreCardInventoryReport, String> {
    let score_cards = read_json("score-cards", path)?;
    let checks = score_cards
        .get("checks")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{} missing top-level `checks` array", path.display()))?;
    if checks.is_empty() {
        return Err(format!("{} checks array must be non-empty", path.display()));
    }
    let mut ids = BTreeSet::new();
    let mut score_card_commands_executed = 0usize;
    for (index, check) in checks.iter().enumerate() {
        let check_object = check
            .as_object()
            .ok_or_else(|| format!("{} checks[{index}] must be a JSON object", path.display()))?;
        for field in REQUIRED_SCORE_CARD_FIELDS {
            if !check_object.contains_key(*field) {
                return Err(format!(
                    "{} checks[{index}] missing required field `{field}`",
                    path.display()
                ));
            }
        }
        let id = required_str_in_object(check_object, "id", path)?;
        if !ids.insert(id.to_string()) {
            return Err(format!("{} duplicate score-card id `{id}`", path.display()));
        }
        let query = required_str_in_object(check_object, "query", path)?;
        if !contains_any_keyword(query, &["oya", "grep", "jq", "cargo"]) {
            return Err(format!(
                "{} score-card `{id}` query must be a deterministic oya/grep/jq/cargo command",
                path.display()
            ));
        }
        if contains_any_keyword(query, &["llm", "manual"]) {
            return Err(format!(
                "{} score-card `{id}` query must be deterministic and must not depend on LLM/manual judgment",
                path.display()
            ));
        }
        let severity = required_str_in_object(check_object, "severity_tier", path)?;
        if !matches!(
            severity,
            "INFO" | "MINOR" | "MAJOR" | "CRITICAL" | "BLOCKER"
        ) {
            return Err(format!(
                "{} score-card `{id}` has invalid severity_tier `{severity}`",
                path.display()
            ));
        }
        let score = check_object
            .get("score")
            .and_then(Value::as_u64)
            .ok_or_else(|| {
                format!(
                    "{} score-card `{id}` field `score` must be an integer",
                    path.display()
                )
            })?;
        if score > 10 {
            return Err(format!(
                "{} score-card `{id}` field `score` must be <= 10",
                path.display()
            ));
        }
        // Git history is the audit log; committed evidence JSON is not required.
        let enforcement_status = check_object
            .get("enforcement_status")
            .and_then(Value::as_str)
            .unwrap_or(ACTIVE_SCORE_CARD_STATUS);
        match enforcement_status {
            ACTIVE_SCORE_CARD_STATUS => {
                if execute_score_card_query(id, query, path)? {
                    score_card_commands_executed += 1;
                }
            }
            ADVISORY_SCORE_CARD_STATUS => {
                validate_advisory_score_card(id, check_object, path)?;
            }
            _ => {
                return Err(format!(
                    "{} score-card `{id}` has invalid enforcement_status `{enforcement_status}`",
                    path.display()
                ));
            }
        }
    }
    Ok(ScoreCardInventoryReport {
        score_card_ids: ids,
        score_card_commands_executed,
    })
}

fn validate_advisory_score_card(
    id: &str,
    check_object: &serde_json::Map<String, Value>,
    path: &Path,
) -> Result<(), String> {
    let planned_verification_ref =
        required_str_in_object(check_object, "planned_verification_ref", path)?;
    if planned_verification_ref.trim().is_empty() {
        return Err(format!(
            "{} score-card `{id}` planned_verification_ref must be non-empty",
            path.display()
        ));
    }
    let activation_requires = check_object
        .get("activation_requires")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            format!(
                "{} score-card `{id}` advisory row missing activation_requires array",
                path.display()
            )
        })?;
    if activation_requires.is_empty() {
        return Err(format!(
            "{} score-card `{id}` activation_requires array must be non-empty",
            path.display()
        ));
    }
    for (index, item) in activation_requires.iter().enumerate() {
        let Some(text) = item.as_str() else {
            return Err(format!(
                "{} score-card `{id}` activation_requires[{index}] must be a string",
                path.display()
            ));
        };
        if text.trim().is_empty() {
            return Err(format!(
                "{} score-card `{id}` activation_requires[{index}] must be non-empty",
                path.display()
            ));
        }
    }
    Ok(())
}

fn execute_score_card_query(id: &str, query: &str, path: &Path) -> Result<bool, String> {
    match query {
        "grep -rhoE 'uses:[[:space:]]*[^[:space:]]+' .github/workflows | grep -vE '@[0-9a-f]{40}$'" =>
        {
            require_third_party_actions_are_sha_pinned(id, Path::new(".github/workflows"), path)?;
            Ok(true)
        }
        "grep -qF '[profile.ci]' .config/nextest.toml && cargo nextest list --profile ci --workspace" =>
        {
            require_file_contains_literal(
                id,
                Path::new(".config/nextest.toml"),
                "[profile.ci]",
                path,
            )?;
            // Skip execution when cargo-nextest is not installed in the current
            // environment (hermetic buck2 lanes, fresh CI containers before the
            // nextest install step, etc.). The file-contains check above still
            // validates that the nextest CI profile exists; the list command is
            // treated as best-effort in environments without the binary.
            let nextest_available = Command::new("cargo")
                .args(["nextest", "--version"])
                .env_remove("RUSTC_WRAPPER")
                .output()
                .is_ok_and(|o| o.status.success());
            if nextest_available {
                run_score_card_command(
                    id,
                    "cargo",
                    &["nextest", "list", "--profile", "ci", "--workspace"],
                    path,
                )?;
            }
            Ok(nextest_available)
        }
        "find scripts -type f -perm -111 -exec grep -L '^#!' {} + | head -1" => {
            require_executable_scripts_have_shebang(id, Path::new("scripts"), path)?;
            Ok(true)
        }
        "oya gate validate loop-recovery-patterns" => {
            // The current Rust lane is the executable form of this score card.
            // Re-running it here would recurse forever; the surrounding gate
            // validates the same invariant directly.
            Ok(false)
        }
        _ => Err(format!(
            "{} score-card `{id}` query `{query}` is not an allowed executable deterministic score-card command",
            path.display()
        )),
    }
}

/// The SHA-pinning half of the retired `check-supply-chain` evidence bundle, re-homed
/// as a NATIVE score-card predicate rather than dropped with the kernel.
///
/// The kernel it came from was fail-fast and demanded `cargo_deny_check_wired` /
/// `cargo_audit_check_wired`, neither of which this tree may satisfy (shell plus a
/// network-fetching advisory index are forbidden), so it could never run. Its
/// `third_party_actions_pinned` clause, by contrast, is checkable here with no shell,
/// no network and no clock — so it is kept and made to actually read the workflows
/// instead of asking a gate that no longer exists.
///
/// A mutable tag (`@v4`) or a branch (`@main`) is a supply-chain hole: the ref can be
/// repointed at new code after review. Only a full 40-hex commit SHA is accepted.
/// Local reusable workflows (`./.github/workflows/x.yml`) carry no ref and are exempt
/// by construction — they are this repository's own reviewed content.
fn require_third_party_actions_are_sha_pinned(
    id: &str,
    workflows_dir: &Path,
    score_cards_path: &Path,
) -> Result<(), String> {
    let workflow_paths = collect_files_recursively(workflows_dir)?;
    let mut unpinned = Vec::new();
    let mut references = 0usize;
    for workflow_path in &workflow_paths {
        let extension = workflow_path
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default();
        if extension != "yml" && extension != "yaml" {
            continue;
        }
        let contents = fs::read_to_string(workflow_path).map_err(|error| {
            format!(
                "{} score-card `{id}` could not read {}: {error}",
                score_cards_path.display(),
                workflow_path.display()
            )
        })?;
        for line in contents.lines() {
            let trimmed = line.trim_start().trim_start_matches("- ").trim_start();
            let Some(reference) = trimmed.strip_prefix("uses:") else {
                continue;
            };
            let reference = reference.trim();
            if reference.is_empty() || reference.starts_with('.') {
                continue;
            }
            references += 1;
            let pinned = reference.rsplit_once('@').is_some_and(|(_, git_ref)| {
                git_ref.len() == 40 && git_ref.bytes().all(|byte| byte.is_ascii_hexdigit())
            });
            if !pinned {
                unpinned.push(format!("{}: {reference}", workflow_path.display()));
            }
        }
    }
    // Anti-vacuity: a walk that finds no references at all proves nothing, and would
    // turn this card green by collapsing rather than by the tree being clean.
    if references == 0 {
        return Err(format!(
            "{} score-card `{id}` found zero `uses:` references under {} — refuse a vacuously green scan",
            score_cards_path.display(),
            workflows_dir.display()
        ));
    }
    if !unpinned.is_empty() {
        return Err(format!(
            "{} score-card `{id}`: {} of {references} `uses:` references are not pinned to a 40-hex commit SHA:\n  {}",
            score_cards_path.display(),
            unpinned.len(),
            unpinned.join("\n  ")
        ));
    }
    Ok(())
}

fn run_score_card_command<P>(id: &str, program: P, args: &[&str], path: &Path) -> Result<(), String>
where
    P: AsRef<std::ffi::OsStr>,
{
    let program_label = program.as_ref().to_string_lossy().into_owned();
    let output = Command::new(program.as_ref())
        .args(args)
        .env_remove("RUSTC_WRAPPER")
        .output()
        .map_err(|error| {
            format!(
                "{} score-card `{id}` command `{program_label} {}` failed to start: {error}",
                path.display(),
                args.join(" ")
            )
        })?;
    if !output.status.success() {
        return Err(format!(
            "{} score-card `{id}` command `{program_label} {}` failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
            path.display(),
            args.join(" "),
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

fn require_file_contains_literal(
    id: &str,
    file_path: &Path,
    literal: &str,
    score_cards_path: &Path,
) -> Result<(), String> {
    let contents = fs::read_to_string(file_path).map_err(|error| {
        format!(
            "{} score-card `{id}` could not read {}: {error}",
            score_cards_path.display(),
            file_path.display()
        )
    })?;
    if !contents.contains(literal) {
        return Err(format!(
            "{} score-card `{id}` requires literal `{literal}` in {}",
            score_cards_path.display(),
            file_path.display()
        ));
    }
    Ok(())
}

fn require_executable_scripts_have_shebang(
    id: &str,
    scripts_dir: &Path,
    score_cards_path: &Path,
) -> Result<(), String> {
    let script_paths = collect_files_recursively(scripts_dir)?;
    for script_path in script_paths {
        let metadata = fs::metadata(&script_path).map_err(|error| {
            format!(
                "{} score-card `{id}` could not stat {}: {error}",
                score_cards_path.display(),
                script_path.display()
            )
        })?;
        if !is_executable(&metadata) {
            continue;
        }
        let contents = fs::read(&script_path).map_err(|error| {
            format!(
                "{} score-card `{id}` could not read {}: {error}",
                score_cards_path.display(),
                script_path.display()
            )
        })?;
        if !contents.starts_with(b"#!") {
            return Err(format!(
                "{} score-card `{id}` executable script {} is missing a shebang",
                score_cards_path.display(),
                script_path.display()
            ));
        }
    }
    Ok(())
}

fn collect_files_recursively(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    collect_files_recursively_into(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files_recursively_into(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(root)
        .map_err(|error| format!("{} is not readable: {error}", root.display()))?
    {
        let path = entry
            .map_err(|error| format!("{} entry is unreadable: {error}", root.display()))?
            .path();
        if path.is_dir() {
            collect_files_recursively_into(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(unix)]
fn is_executable(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;

    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable(metadata: &fs::Metadata) -> bool {
    !metadata.permissions().readonly()
}

fn read_mistakes_ledger_ids(path: &Path) -> Result<BTreeSet<String>, String> {
    let ledger = read_json("mistakes-ledger", path)?;
    let entries = ledger
        .get("entries")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{} missing top-level `entries` array", path.display()))?;
    let mut ids = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate() {
        let entry_object = entry
            .as_object()
            .ok_or_else(|| format!("{} entries[{index}] must be a JSON object", path.display()))?;
        let id = required_str_in_object(entry_object, "id", path)?;
        ids.insert(id.to_string());
    }
    Ok(ids)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PatternValidationReport {
    patterns_checked: usize,
    active_blockers_checked: usize,
    mistakes_ledger_refs_checked: usize,
}

fn validate_loop_patterns(
    patterns_dir: &Path,
    score_card_ids: &BTreeSet<String>,
    mistake_ids: &BTreeSet<String>,
    score_cards_path: &Path,
    mistakes_ledger_path: &Path,
) -> Result<PatternValidationReport, String> {
    let mut pattern_paths = fs::read_dir(patterns_dir)
        .map_err(|error| format!("loop-recovery patterns dir unreadable: {error}"))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    pattern_paths.sort();
    if pattern_paths.is_empty() {
        return Err(format!(
            "{} must contain at least one documented repeat-loop pattern",
            patterns_dir.display()
        ));
    }

    let mut pattern_ids = BTreeSet::new();
    let mut active_blockers_checked = 0usize;
    let mut mistakes_ledger_refs_checked = 0usize;
    for path in &pattern_paths {
        let pattern = read_json("loop-recovery-pattern", path)?;
        let pattern_object = pattern
            .as_object()
            .ok_or_else(|| format!("{} must be a JSON object", path.display()))?;
        let pattern_id = required_str_in_object(pattern_object, "pattern_id", path)?;
        if !pattern_ids.insert(pattern_id.to_string()) {
            return Err(format!("duplicate loop-recovery pattern_id `{pattern_id}`"));
        }
        let status = required_str_in_object(pattern_object, "status", path)?;
        if !ALLOWED_PATTERN_STATUSES.contains(&status) {
            return Err(format!(
                "{} pattern `{pattern_id}` has invalid status `{status}`",
                path.display()
            ));
        }
        require_non_empty_string_field(pattern_object, "trigger_signature", pattern_id, path)?;
        require_non_empty_string_field(pattern_object, "failure_mode", pattern_id, path)?;
        require_non_empty_string_field(pattern_object, "detection_query", pattern_id, path)?;
        require_non_empty_string_field(pattern_object, "recovery_action", pattern_id, path)?;
        require_non_empty_string_field(pattern_object, "owner_team", pattern_id, path)?;
        require_non_empty_string_array(pattern_object, "evidence_refs", pattern_id, path)?;
        require_non_empty_string_array(pattern_object, "sources_scanned", pattern_id, path)?;

        let score_refs =
            required_string_array(pattern_object, "deterministic_score_card_refs", path)?;
        if score_refs.is_empty() {
            return Err(format!(
                "{} pattern `{pattern_id}` must cite at least one deterministic score card",
                path.display()
            ));
        }
        for score_ref in score_refs {
            if !score_card_ids.contains(&score_ref) {
                return Err(format!(
                    "{} pattern `{pattern_id}` references unknown score card `{score_ref}` (inventory: {})",
                    path.display(),
                    score_cards_path.display()
                ));
            }
        }

        let mistake_refs = required_string_array(pattern_object, "mistakes_ledger_refs", path)?;
        if mistake_refs.is_empty() {
            return Err(format!(
                "{} pattern `{pattern_id}` must cite at least one mistakes-ledger row",
                path.display()
            ));
        }
        for mistake_ref in mistake_refs {
            if !mistake_ids.contains(&mistake_ref) {
                return Err(format!(
                    "{} pattern `{pattern_id}` references unknown mistakes-ledger id `{mistake_ref}` (ledger: {})",
                    path.display(),
                    mistakes_ledger_path.display()
                ));
            }
            mistakes_ledger_refs_checked += 1;
        }

        let pre_push_blocker = pattern_object
            .get("pre_push_blocker")
            .and_then(Value::as_bool)
            .ok_or_else(|| {
                format!(
                    "{} pattern `{pattern_id}` field `pre_push_blocker` must be boolean",
                    path.display()
                )
            })?;
        if status == "active" {
            if !pre_push_blocker {
                return Err(format!(
                    "{} active pattern `{pattern_id}` must set pre_push_blocker=true",
                    path.display()
                ));
            }
            active_blockers_checked += 1;
        }
    }

    if active_blockers_checked == 0 {
        return Err(format!(
            "{} must contain at least one active pre-push blocker pattern",
            patterns_dir.display()
        ));
    }

    Ok(PatternValidationReport {
        patterns_checked: pattern_paths.len(),
        active_blockers_checked,
        mistakes_ledger_refs_checked,
    })
}

fn read_json(label: &str, path: &Path) -> Result<Value, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("{label} unreadable {}: {error}", path.display()))?;
    serde_json::from_str(&contents)
        .map_err(|error| format!("{label} JSON invalid {}: {error}", path.display()))
}

fn required_object<'a>(
    value: &'a Value,
    key: &str,
    path: &Path,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    value
        .get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{} missing top-level `{key}` object", path.display()))
}

fn required_str_in_object<'a>(
    object: &'a serde_json::Map<String, Value>,
    key: &str,
    path: &Path,
) -> Result<&'a str, String> {
    object.get(key).and_then(Value::as_str).ok_or_else(|| {
        format!(
            "{} missing required non-empty string field `{key}`",
            path.display()
        )
    })
}

fn require_non_empty_string_field(
    object: &serde_json::Map<String, Value>,
    key: &str,
    pattern_id: &str,
    path: &Path,
) -> Result<(), String> {
    let value = required_str_in_object(object, key, path)?;
    if value.trim().is_empty() {
        return Err(format!(
            "{} pattern `{pattern_id}` field `{key}` must be non-empty",
            path.display()
        ));
    }
    Ok(())
}

fn require_non_empty_string_array(
    object: &serde_json::Map<String, Value>,
    key: &str,
    pattern_id: &str,
    path: &Path,
) -> Result<(), String> {
    let values = required_string_array(object, key, path)?;
    if values.is_empty() {
        return Err(format!(
            "{} pattern `{pattern_id}` field `{key}` must be non-empty",
            path.display()
        ));
    }
    Ok(())
}

fn required_string_array(
    object: &serde_json::Map<String, Value>,
    key: &str,
    path: &Path,
) -> Result<Vec<String>, String> {
    let values = object
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{} missing required array field `{key}`", path.display()))?;
    let mut parsed = Vec::with_capacity(values.len());
    for (index, value) in values.iter().enumerate() {
        let Some(item) = value.as_str() else {
            return Err(format!(
                "{} `{key}`[{index}] must be a string",
                path.display()
            ));
        };
        if item.trim().is_empty() {
            return Err(format!(
                "{} `{key}`[{index}] must be non-empty",
                path.display()
            ));
        }
        parsed.push(item.to_string());
    }
    Ok(parsed)
}

fn require_keywords(
    label: &str,
    value: &str,
    keywords: &[&str],
    path: &Path,
) -> Result<(), String> {
    let lowercase = value.to_ascii_lowercase();
    for keyword in keywords {
        if !lowercase.contains(&keyword.to_ascii_lowercase()) {
            return Err(format!(
                "{} field `{label}` must contain `{keyword}`",
                path.display()
            ));
        }
    }
    Ok(())
}

fn contains_any_keyword(value: &str, keywords: &[&str]) -> bool {
    let lowercase = value.to_ascii_lowercase();
    keywords
        .iter()
        .any(|keyword| lowercase.contains(&keyword.to_ascii_lowercase()))
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn advisory_score_card_is_inventory_only_until_validator_lands() {
        let fixture = score_cards_fixture(true);

        let report = read_score_card_inventory(&fixture).expect("advisory score-card is valid");

        assert_eq!(report.score_card_ids.len(), 1);
        assert_eq!(report.score_card_commands_executed, 0);
    }

    #[test]
    fn advisory_score_card_requires_activation_evidence() {
        let fixture = score_cards_fixture(false);

        let error = read_score_card_inventory(&fixture).expect_err("activation evidence required");

        assert!(error.contains("activation_requires"));
    }

    fn score_cards_fixture(include_activation_requires: bool) -> PathBuf {
        let fixture_id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let variant = if include_activation_requires {
            "with-activation"
        } else {
            "missing-activation"
        };
        let root = std::env::temp_dir().join(format!(
            "oya-loop-recovery-advisory-score-card-{}-{variant}-{fixture_id}",
            std::process::id(),
        ));
        fs::create_dir_all(&root).expect("fixture dir created");
        let evidence_path = root.join("evidence.json");
        fs::write(&evidence_path, "{}\n").expect("evidence file written");
        let mut check = serde_json::json!({
            "id": "score-card:hyperscaler:future-advisory",
            "facet": "hyperscaler-pattern",
            "query": "oya gate validate future-hyperscaler-score-card",
            "pass_criterion": "exit 0 after the validator exists and is branch-protected",
            "score": 8,
            "severity_tier": "MAJOR",
            "empirical_evidence_path": evidence_path,
            "enforcement_status": "advisory-until-validator",
            "planned_verification_ref": "ADR-0134"
        });
        if include_activation_requires {
            check["activation_requires"] = serde_json::json!([
                "validator crate exists",
                "workflow is branch-protected",
                "fixture-tree integration tests pass"
            ]);
        }
        let score_cards = serde_json::json!({
            "checks": [check]
        });
        let score_cards_path = root.join("score-cards.json");
        fs::write(
            &score_cards_path,
            serde_json::to_string_pretty(&score_cards).expect("score-card fixture serializes"),
        )
        .expect("score-cards fixture written");
        score_cards_path
    }
}
