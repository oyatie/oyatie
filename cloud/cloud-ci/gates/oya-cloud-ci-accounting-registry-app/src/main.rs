//! Binary entry point for the accounting-registry producer.
//!
//! Runs git plumbing (`git ls-files`, `git log`) + reads the real reachability/owner/
//! justification sources, then delegates to the deterministic library to build the
//! registry + companion faces. This is the buck2 `rust_binary`
//! `//cloud/cloud-ci/gates:oya-cloud-ci-accounting-registry-app` (register #20 — NOT an `oya` CLI).
//!
//! Usage:
//!   oya-cloud-ci-accounting-registry-app [--repo-root <path>] [--out-dir <path>] [--stdout]
//!
//! With `--stdout` the registry is written to stdout (used by the registry-drift gate
//! to regenerate in a sandbox and byte-diff). Default writes the four faces under
//! `<out-dir>` (default `<repo-root>/cloud/cloud-ci/gates`).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use oya_cloud_ci_accounting_registry_app::{
    build_decision_crosswalk, build_enforcement_inventory, build_gate_baseline, build_registry,
    to_canonical_json, CrosswalkInputs, DecisionCrosswalkRow, EnforcementInputs, EnforcementRow,
    GateInputs, Policy, ProducerError, RepoInputs,
};
use oya_check_brand_residue::forbidden_vocab::{
    census_findings_with, is_path_carved_out_with, CensusDocument, VocabPolicy,
};
use serde_json::{json, Value};

fn main() {
    if let Err(error) = run() {
        eprintln!("oya-cloud-ci-accounting-registry-app: {error}");
        std::process::exit(1);
    }
}

#[derive(Debug)]
enum CliError {
    Producer(ProducerError),
    Io(String),
    Git(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Producer(e) => write!(f, "{e}"),
            CliError::Io(e) => write!(f, "io: {e}"),
            CliError::Git(e) => write!(f, "git: {e}"),
        }
    }
}

impl From<ProducerError> for CliError {
    fn from(e: ProducerError) -> Self {
        CliError::Producer(e)
    }
}

fn run() -> Result<(), CliError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut repo_root: Option<PathBuf> = None;
    let mut out_dir: Option<PathBuf> = None;
    let mut to_stdout = false;
    // Which face to emit to stdout: default registry. The gate self-tests + registry-drift
    // regenerate a single face in a sandbox via `--stdout --face <name>`.
    let mut face = "registry".to_owned();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--repo-root" => {
                i += 1;
                repo_root = args.get(i).map(PathBuf::from);
            }
            "--out-dir" => {
                i += 1;
                out_dir = args.get(i).map(PathBuf::from);
            }
            "--face" => {
                i += 1;
                if let Some(value) = args.get(i) {
                    face = value.clone();
                }
            }
            "--stdout" => to_stdout = true,
            other => return Err(CliError::Io(format!("unknown argument {other}"))),
        }
        i += 1;
    }

    // Root discovery uses the bundled default's markers (chicken/egg: we need a root before we
    // can read a root-relative oya-ci.toml). Discovery markers are not repo-policy, so this is
    // safe even when an adopter overrides them in the file.
    let bootstrap_cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();

    let repo_root = match repo_root {
        Some(root) => root,
        None => discover_repo_root(&bootstrap_cfg)?,
    };
    let out_dir = out_dir
        .unwrap_or_else(|| repo_root.join("cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app"));

    // The oya-ci policy (naming/vocab/manifest/roots/sources/gates) is sourced from the repo's
    // `oya-ci.toml` (OYA-CI-CONFORMANCE-FLOOR-PLAN §3.3 / Stage 2), via the CLOSED-schema loader;
    // when the file is absent the compiled-in bundled default applies (zero-config does
    // something useful). oyatie's checked-in file reproduces today's values, so every existing
    // face is byte-for-byte unchanged.
    let cfg = load_config(&repo_root)?;
    let config_digest = cfg.digest();

    let policy = Policy::from_config(&cfg)?;
    let inputs = collect_repo_inputs(&repo_root, &cfg)?;
    let registry = build_registry(&inputs, &policy)?;
    let crosswalk = build_decision_crosswalk(&collect_crosswalk_inputs(&repo_root, &cfg))?;
    let enforcement = build_enforcement_inventory(&collect_enforcement_inputs(&repo_root, &cfg))?;

    // The fifth face freezes TODAY's accepted-violation KEYS per (gate, code). It runs each
    // gate's pure evaluate_keyed over the four live faces. Two faces need a light per-gate
    // adaptation to the shape the gate evaluator consumes (identical to each gate's
    // born-blocking self-test): the staleness rows are aged from git commit timestamps, and
    // the automation matrix is derived from the enforcement-inventory face.
    let staleness_input = build_staleness_input(&repo_root, &registry)?;
    let automation_matrix = build_automation_matrix(&enforcement);
    // The fifth gate (cloud-ci-brand-residue) scans the raw tracked corpus for the forbidden
    // vocab stems and freezes the per-(stem,file) residue as the shrink-only-ratchet baseline.
    let brand_residue = collect_brand_residue(&repo_root, &inputs.tracked_paths, &cfg);
    // The §2.5#4 bnf-layer-suffix gate input: the first-party oya-* crate names enumerated from
    // the tracked Cargo.toml manifests. The gate's evaluate_keyed resolves the role carve-out-
    // aware and reuses oya_governance_predictable_naming_kernel::check.
    let bnf_layer_suffix = collect_bnf_layer_suffix(&repo_root, &inputs.tracked_paths, &cfg);
    // The §2.5#7 manifest-hygiene gate input: per-crate Cargo.toml hygiene flags.
    let manifest_hygiene = collect_manifest_hygiene(&repo_root, &inputs.tracked_paths, &cfg);
    // The ADR-0017 cargo-prefix gate input: the first-party oya-* workspace members + their
    // package names. The gate's evaluate_keyed reuses
    // oya_intelligence_cargo_prefix_domain::validate_cargo_prefix per crate.
    let cargo_prefix = collect_cargo_prefix(&repo_root, &inputs.tracked_paths, &cfg);
    let gate_inputs = GateInputs {
        total_accounting: &registry,
        cross_artifact: &crosswalk,
        automation_ratchet: &automation_matrix,
        staleness: &staleness_input,
        bnf_layer_suffix: &bnf_layer_suffix,
        manifest_hygiene: &manifest_hygiene,
        cargo_prefix: &cargo_prefix,
        brand_residue: &brand_residue,
    };
    let baseline = build_gate_baseline(&cfg, &gate_inputs, &config_digest)?;

    if to_stdout {
        let value = match face.as_str() {
            "registry" => &registry,
            "decision-crosswalk" => &crosswalk,
            "enforcement-inventory" => &enforcement,
            "ttl-policy" => &policy.ttl_policy_face(),
            "bnf-layer-suffix" => &bnf_layer_suffix,
            "manifest-hygiene" => &manifest_hygiene,
            "cargo-prefix" => &cargo_prefix,
            "baseline" => &baseline,
            other => return Err(CliError::Io(format!("unknown --face {other}"))),
        };
        print!("{}", to_canonical_json(value)?);
        return Ok(());
    }

    write_face(
        &out_dir.join("accounting-registry.generated.json"),
        &registry,
    )?;
    write_face(
        &out_dir.join("ttl-policy.generated.json"),
        &policy.ttl_policy_face(),
    )?;
    write_face(
        &out_dir.join("decision-crosswalk.generated.json"),
        &crosswalk,
    )?;
    write_face(
        &out_dir.join("enforcement-inventory.generated.json"),
        &enforcement,
    )?;
    write_face(&out_dir.join("gate-baseline.generated.json"), &baseline)?;

    let rows = registry["rows"].as_array().map(Vec::len).unwrap_or(0);
    eprintln!("oya-cloud-ci-accounting-registry-app: {rows} rows -> {}", out_dir.display());
    Ok(())
}

/// Walk up from cwd to the repo root (the dir holding `specs/root-hub-pointers.json`),
/// matching the existing kernel-test convention.
/// Load the repo's `oya-ci.toml` (OYA-CI-CONFORMANCE-FLOOR-PLAN §3.3). When the file is present
/// it is parsed by the CLOSED-schema loader (a malformed file / unknown key is a hard error, so
/// a broken config fails LOUDLY rather than silently reverting policy); when it is absent the
/// compiled-in bundled default applies (zero-config = today's language-agnostic posture).
fn load_config(repo_root: &Path) -> Result<oya_ci_config_kernel::OyaCiConfig, CliError> {
    let path = repo_root.join("oya-ci.toml");
    match std::fs::read_to_string(&path) {
        Ok(text) => oya_ci_config_kernel::OyaCiConfig::from_toml_str(&text)
            .map_err(|e| CliError::Io(format!("{}: {e}", path.display()))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Ok(oya_ci_config_kernel::OyaCiConfig::bundled_default())
        }
        Err(e) => Err(CliError::Io(format!("{}: {e}", path.display()))),
    }
}

fn discover_repo_root(cfg: &oya_ci_config_kernel::OyaCiConfig) -> Result<PathBuf, CliError> {
    let markers = &cfg.repo.root_markers;
    let mut dir = std::env::current_dir().map_err(|e| CliError::Io(e.to_string()))?;
    for _ in 0..16 {
        if markers.iter().any(|m| dir.join(m).is_file()) {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    Err(CliError::Io(format!(
        "failed to locate repo root (no {} up-tree)",
        markers.join(" / ")
    )))
}

fn write_face(path: &Path, value: &Value) -> Result<(), CliError> {
    let text = to_canonical_json(value)?;
    std::fs::write(path, text).map_err(|e| CliError::Io(format!("{}: {e}", path.display())))
}

/// Build the GATE-3 staleness-reaper input from the registry by aging each row from its
/// `last_touch_commit` against the HEAD commit time (deterministic per-checkout, no
/// wall-clock). Mirrors the gate's born-blocking self-test exactly so the baseline freezes
/// the same keys the gate would flag today.
fn build_staleness_input(repo_root: &Path, registry: &Value) -> Result<Value, CliError> {
    let now_secs = git_head_secs(repo_root)?;
    let commit_ts = git_commit_timestamps(repo_root)?;
    let rows = registry["rows"].as_array().cloned().unwrap_or_default();
    let mut aged_rows: Vec<Value> = Vec::with_capacity(rows.len());
    for row in rows {
        let sha = row["last_touch_commit"].as_str().unwrap_or("");
        let age_days = commit_ts
            .get(sha)
            .map(|ts| (now_secs.saturating_sub(*ts)) / 86_400)
            .unwrap_or(0);
        let mut aged = row.clone();
        if let Value::Object(map) = &mut aged {
            map.insert("age_days".into(), json!(age_days));
        }
        aged_rows.push(aged);
    }
    Ok(json!({ "rows": aged_rows }))
}

/// Build the GATE-4 automation-ratchet matrix input by adapting the enforcement-inventory
/// face rows into matrix rows. Mirrors the gate's born-blocking self-test exactly: a surface
/// that maps a blocking invariant to oya CLI is classified blocking with that target; an
/// unwired claim carries claims_enforced.
fn build_automation_matrix(enforcement: &Value) -> Value {
    let surfaces = enforcement["rows"].as_array().cloned().unwrap_or_default();
    let mut matrix_rows: Vec<Value> = Vec::with_capacity(surfaces.len());
    for surface in &surfaces {
        let id = surface["id"].as_str().unwrap_or("");
        let src = surface["source_artifact"].as_str().unwrap_or("");
        let claims = surface["claims_enforced"].as_bool() == Some(true);
        let wired = surface["has_wired_buck2_target"].as_bool() == Some(true);
        let maps_oya = surface["maps_to_oya_cli"].as_bool() == Some(true);
        matrix_rows.push(json!({
            "id": id,
            "source_artifact": src,
            "requirement": "Live enforcement surface inventoried by the producer.",
            "classification": if maps_oya { "automated_blocking_now" } else { "automated_advisory_until_p0_0" },
            "owner": "platform-governance",
            "target_gate_or_controller": if maps_oya { "oya gate / oya gen verified_by authority" } else { src },
            "blocking_fixture": "specs/fixtures/phase0-automation-ratchet/",
            "retirement_phase": "P0.0",
            "evidence_path": src,
            "no_new_oya_cli_surface": !maps_oya,
            "claims_enforced": claims,
            "has_wired_buck2_target": wired
        }));
    }
    json!({ "rows": matrix_rows })
}

/// Build the cloud-ci-brand-residue gate's `code -> keys` (the forbidden-vocab shrink-only
/// ratchet). Scans the raw tracked corpus (NOT a generated face) for the four forbidden stems
/// and freezes ONE key per `(stem, file)` so the firewall blocks any NEW occurrence while the
/// historical residue ages out. Carve-outs (the deny-list source, the catalog spec, the
/// Palantir proper-noun prose, the append-only audit chain, the `_legacy-foundry/` archive,
/// and the generated faces) are DATA in `oya_check_brand_residue::forbidden_vocab`, applied
/// here so wholly-carved files are never even read.
///
/// Deterministic + churn-free: per-file keys are stable under in-file edits (line numbers
/// never enter the key), so editing prose in an already-listed file stays GREEN; only fully
/// cleaning a file shrinks the set.
fn collect_brand_residue(
    repo_root: &Path,
    tracked_paths: &[String],
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> BTreeMap<String, BTreeSet<String>> {
    // The forbidden-stem table + carve-outs are sourced from the oya-ci config `[vocab]` section
    // (§3.3 / Stage 3); the bundled default reproduces today's consts, so the census is
    // byte-for-byte unchanged.
    let policy = vocab_policy(&cfg.vocab);
    // Read each non-carved-out tracked file once; non-UTF-8 (binary) files read as empty and
    // contribute nothing (the stems are ASCII text tokens).
    let contents: Vec<(String, String)> = tracked_paths
        .iter()
        .filter(|path| !is_path_carved_out_with(path, &policy))
        .map(|path| (path.clone(), read_text(&repo_root.join(path))))
        .collect();
    let documents = contents.iter().map(|(path, body)| CensusDocument {
        path: path.as_str(),
        contents: body.as_str(),
    });

    let mut grouped: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for finding in census_findings_with(documents, &policy) {
        grouped.entry(finding.code).or_default().insert(finding.key);
    }
    grouped
}

/// Map the oya-ci config `[vocab]` section onto the brand crate's injected [`VocabPolicy`]
/// (§3.3 / Stage 3). The kind enum is mirrored 1:1; the bundled default reproduces today's
/// `FORBIDDEN_VOCAB_STEMS` + `CARVE_OUT_RULES`.
fn vocab_policy(cfg: &oya_ci_config_kernel::VocabConfig) -> VocabPolicy {
    use oya_check_brand_residue::forbidden_vocab::{CarveOutKind, OwnedCarveOut, OwnedStem};
    use oya_ci_config_kernel::VocabCarveOutKind;
    VocabPolicy {
        stems: cfg
            .forbidden_stems
            .iter()
            .map(|s| OwnedStem {
                stem: s.stem.clone(),
                code: s.code.clone(),
            })
            .collect(),
        carve_outs: cfg
            .carve_outs
            .iter()
            .map(|c| OwnedCarveOut {
                kind: match c.kind {
                    VocabCarveOutKind::PathPrefix => CarveOutKind::PathPrefix,
                    VocabCarveOutKind::PathExact => CarveOutKind::PathExact,
                    VocabCarveOutKind::PathSuffix => CarveOutKind::PathSuffix,
                    VocabCarveOutKind::LineContainsCi => CarveOutKind::LineContainsCi,
                },
                value: c.value.clone(),
            })
            .collect(),
    }
}

/// Enumerate the first-party `oya-*` crate package names from the tracked Cargo.toml manifests
/// (the §2.5#4 gate's I/O). Skips vendored `third-party/` manifests and the virtual workspace
/// root (which has no `[package]`). Deterministic: names go through a BTreeSet (sorted+deduped)
/// so committed==regenerated holds byte-for-byte. Scoped to `oya-*` (the BNF rule's domain);
/// the intentional bare `registry-drift` rust_test is not flagged.
fn collect_bnf_layer_suffix(
    repo_root: &Path,
    tracked_paths: &[String],
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> Value {
    let prefix = cfg.naming.required_prefix.as_str();
    let mut names: BTreeSet<String> = BTreeSet::new();
    for path in tracked_paths {
        if !path.ends_with("Cargo.toml") {
            continue;
        }
        if is_path_excluded(path, cfg) {
            continue;
        }
        if let Some(name) = parse_package_name(&read_text(&repo_root.join(path))) {
            if name.starts_with(prefix) {
                names.insert(name);
            }
        }
    }
    let rows: Vec<Value> = names
        .into_iter()
        .map(|name| json!({ "crate_name": name }))
        .collect();
    json!({ "rows": rows })
}

/// Enumerate the first-party `oya-*` workspace members + their package names (the ADR-0017
/// cargo-prefix gate's I/O). For each tracked `<dir>/Cargo.toml` with an `oya-*` `[package].name`
/// it emits a row of `{"member_path": "<dir>", "package_name": "<name>"}` — the same
/// member_path + package_name pair the dev-cli's cargo-prefix validator builds (the gate reuses
/// `validate_cargo_prefix` per crate). Skips vendored `third-party/` manifests + the virtual
/// workspace root (no `[package]`). Deterministic: rows go through a BTreeMap keyed by member_path
/// (sorted+deduped) so committed==regenerated holds byte-for-byte. Scoped to `oya-*` (the rule's
/// domain) so the intentional bare `registry-drift` rust_test is not flagged.
fn collect_cargo_prefix(
    repo_root: &Path,
    tracked_paths: &[String],
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> Value {
    let prefix = cfg.naming.required_prefix.as_str();
    let mut by_member: BTreeMap<String, String> = BTreeMap::new();
    for path in tracked_paths {
        if !path.ends_with("Cargo.toml") {
            continue;
        }
        if is_path_excluded(path, cfg) {
            continue;
        }
        let Some(name) = parse_package_name(&read_text(&repo_root.join(path))) else {
            continue;
        };
        if !name.starts_with(prefix) {
            continue;
        }
        // member_path = the directory holding the Cargo.toml (the workspace member path).
        let member_path = path
            .strip_suffix("/Cargo.toml")
            .unwrap_or(path)
            .to_owned();
        by_member.insert(member_path, name);
    }
    let rows: Vec<Value> = by_member
        .into_iter()
        .map(|(member_path, package_name)| {
            json!({ "member_path": member_path, "package_name": package_name })
        })
        .collect();
    json!({ "rows": rows })
}

/// Extract `name = "..."` from the `[package]` table of a Cargo.toml. Lightweight line-scan
/// (no `toml` dependency — minimal-deps doctrine); `name` is never workspace-inherited, so it
/// is always a string literal under `[package]`.
fn parse_package_name(contents: &str) -> Option<String> {
    let mut in_package = false;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if in_package {
            if let Some(rest) = trimmed.strip_prefix("name") {
                if let Some(rest) = rest.trim_start().strip_prefix('=') {
                    let value = rest.trim().trim_matches('"');
                    if !value.is_empty() {
                        return Some(value.to_owned());
                    }
                }
            }
        }
    }
    None
}

/// Per-crate §2.5#7 manifest-hygiene flags parsed from a Cargo.toml.
#[derive(Default)]
struct ManifestFlags {
    version_workspace: bool,
    rust_version_workspace: bool,
    publish_false: bool,
    license: bool,
    lints_workspace: bool,
    has_lib: bool,
    lib_doctest_false: bool,
}

/// Enumerate the first-party `oya-*` crates and emit their §2.5#7 manifest-hygiene flags (the
/// gate's I/O). The gate's `evaluate_keyed` turns missing flags into Findings. Deterministic
/// (BTreeMap, sorted) so committed==regenerated holds byte-for-byte. Scoped to `oya-*`.
fn collect_manifest_hygiene(
    repo_root: &Path,
    tracked_paths: &[String],
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> Value {
    let prefix = cfg.naming.required_prefix.as_str();
    let mut by_name: BTreeMap<String, ManifestFlags> = BTreeMap::new();
    for path in tracked_paths {
        if !path.ends_with("Cargo.toml") {
            continue;
        }
        if is_path_excluded(path, cfg) {
            continue;
        }
        let contents = read_text(&repo_root.join(path));
        let Some(name) = parse_package_name(&contents) else {
            continue;
        };
        if !name.starts_with(prefix) {
            continue;
        }
        by_name.insert(name, parse_manifest_flags(&contents));
    }
    let rows: Vec<Value> = by_name
        .into_iter()
        .map(|(name, f)| {
            json!({
                "crate_name": name,
                "has_version_workspace": f.version_workspace,
                "has_rust_version_workspace": f.rust_version_workspace,
                "has_publish_false": f.publish_false,
                "has_license": f.license,
                "has_lints_workspace": f.lints_workspace,
                "has_lib": f.has_lib,
                "has_lib_doctest_false": f.lib_doctest_false,
            })
        })
        .collect();
    json!({ "rows": rows })
}

/// Section-aware line-scan of a Cargo.toml for the §2.5#7 hygiene fields (no `toml` dependency —
/// minimal-deps doctrine). Tracks the current table so `[package]` fields, `[lints] workspace`,
/// and `[lib] doctest` are read in their own sections.
fn parse_manifest_flags(contents: &str) -> ManifestFlags {
    let mut f = ManifestFlags::default();
    let mut section = "";
    for raw in contents.lines() {
        // Strip an end-of-line comment (Cargo.toml hygiene values carry no '#').
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix('[') {
            section = match rest.split(']').next().unwrap_or("").trim() {
                "package" => "package",
                "lints" => "lints",
                "lib" => "lib",
                _ => "other",
            };
            if section == "lib" {
                f.has_lib = true;
            }
            continue;
        }
        match section {
            "package" => {
                if is_workspace_inherited(line, "version") {
                    f.version_workspace = true;
                }
                if is_workspace_inherited(line, "rust-version") {
                    f.rust_version_workspace = true;
                }
                if line.starts_with("publish") && line.contains('=') && line.contains("false") {
                    f.publish_false = true;
                }
                if line.starts_with("license") && line.contains('=') {
                    f.license = true;
                }
            }
            "lints" => {
                if line.starts_with("workspace") && line.contains('=') && line.contains("true") {
                    f.lints_workspace = true;
                }
            }
            "lib" => {
                if line.starts_with("doctest") && line.contains('=') && line.contains("false") {
                    f.lib_doctest_false = true;
                }
            }
            _ => {}
        }
    }
    f
}

/// True when `<key>` inherits the workspace: `<key>.workspace = true` or
/// `<key> = { workspace = true }`. The exact-prefix check keeps `version` from matching
/// `rust-version`.
fn is_workspace_inherited(line: &str, key: &str) -> bool {
    let dotted = format!("{key}.workspace");
    if line.starts_with(&dotted) && line.contains("true") {
        return true;
    }
    if let Some(rest) = line.strip_prefix(key) {
        let rest = rest.trim_start();
        if rest.starts_with('=') && rest.contains("workspace") && rest.contains("true") {
            return true;
        }
    }
    false
}

/// HEAD commit time in epoch seconds (the deterministic "now" for aging the corpus).
fn git_head_secs(repo_root: &Path) -> Result<u64, CliError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["log", "-1", "--format=%ct"])
        .output()
        .map_err(|e| CliError::Git(format!("log HEAD time: {e}")))?;
    if !output.status.success() {
        return Err(CliError::Git(format!(
            "log HEAD time exit {:?}",
            output.status.code()
        )));
    }
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .map_err(|e| CliError::Git(format!("parse HEAD time: {e}")))
}

/// One `git log --format` pass builds the commit-sha -> author-timestamp map (epoch secs).
fn git_commit_timestamps(repo_root: &Path) -> Result<BTreeMap<String, u64>, CliError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["log", "--format=%H %ct"])
        .output()
        .map_err(|e| CliError::Git(format!("log timestamps: {e}")))?;
    if !output.status.success() {
        return Err(CliError::Git(format!(
            "log timestamps exit {:?}",
            output.status.code()
        )));
    }
    let mut map = BTreeMap::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Some((sha, ts)) = line.split_once(' ')
            && let Ok(ts) = ts.trim().parse::<u64>()
        {
            map.insert(sha.to_owned(), ts);
        }
    }
    Ok(map)
}

/// Collect the GATE-1 cross-artifact facts from the live corpus: ADR front-matter
/// (status + reciprocal supersession edges), spec/masterplan/roadmap presence, the
/// duplicate-id collision (two files carrying one id), and the generated-face axes drift
/// (catalog.json vs contracts.json `axes_count`). Single pass over the ADR corpus.
fn collect_crosswalk_inputs(
    repo_root: &Path,
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> CrosswalkInputs {
    let decisions_dir = repo_root.join(&cfg.justification.adr_dir);
    let masterplan = read_text(&repo_root.join(&cfg.reachability.masterplan));
    let roadmap = read_text(&repo_root.join(&cfg.justification.roadmap));

    // id -> the decision files carrying it (dup detection is files-per-id > 1).
    let mut files_by_id: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    if let Ok(entries) = std::fs::read_dir(&decisions_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if adr_id_from_filename(&name).is_some() {
                let path = entry.path();
                let id = front_matter_field(&read_text(&path), "id")
                    .unwrap_or_else(|| adr_id_from_filename(&name).unwrap_or_default());
                files_by_id.entry(id).or_default().push(path);
            }
        }
    }

    let mut decisions: Vec<DecisionCrosswalkRow> = Vec::new();
    let mut duplicate_ids: Vec<String> = Vec::new();
    for (id, files) in &files_by_id {
        if files.len() > 1 {
            duplicate_ids.push(id.clone());
        }
        // Use the first file (path-sorted) as the row's source of front-matter facts.
        let mut sorted = files.clone();
        sorted.sort();
        let body = sorted.first().map(|p| read_text(p)).unwrap_or_default();
        let status = front_matter_field(&body, "status").unwrap_or_default();
        let in_spec = id_in_spec_corpus(repo_root, id);
        let in_masterplan = masterplan.contains(id.as_str());
        let in_roadmap = roadmap.contains(id.as_str());
        decisions.push(DecisionCrosswalkRow {
            id: id.clone(),
            status,
            in_spec,
            in_masterplan,
            in_roadmap,
            supersedes: front_matter_id_array(&body, "supersedes"),
            superseded_by: front_matter_id_array(&body, "superseded_by"),
        });
    }
    duplicate_ids.sort();

    let mut generated_face_axes: BTreeMap<String, i64> = BTreeMap::new();
    if let Some(value) = json_number_at(repo_root, "docs/machine-readable/catalog.json", &["_metadata", "axes_count"]) {
        generated_face_axes.insert("catalog.json".into(), value);
    }
    if let Some(value) = json_number_at(repo_root, "docs/machine-readable/contracts.json", &["_metadata", "axes_count"]) {
        generated_face_axes.insert("contracts.json".into(), value);
    }

    CrosswalkInputs {
        decisions,
        duplicate_ids,
        generated_face_axes,
    }
}

/// Whether a decision id appears in the spec corpus (its own ADR + any spec file mention).
fn id_in_spec_corpus(repo_root: &Path, id: &str) -> bool {
    // The ADR file itself is the decision's spec presence; treat any decision with an
    // on-disk ADR as in_spec. (The masterplan/roadmap checks are the propagation faces.)
    let _ = repo_root;
    !id.is_empty()
}

/// Read a numeric field at a JSON path (object keys) from a face, dependency-free.
fn json_number_at(repo_root: &Path, rel: &str, path: &[&str]) -> Option<i64> {
    let text = read_text(&repo_root.join(rel));
    let value: Value = serde_json::from_str(&text).ok()?;
    let mut cursor = &value;
    for key in path {
        cursor = cursor.get(key)?;
    }
    cursor.as_i64()
}

/// Collect the GATE-4 enforcement surfaces from the live corpus: the governance kernel
/// crates (claim "enforce" by name; wired only if a BUCK gate target exists), the
/// governance lanes (diataxis-doc-class / prd-axis-coverage), and the ADR `verified_by`
/// lines that route a blocking invariant through an `oya gate`/`oya gen` CLI call.
fn collect_enforcement_inputs(
    repo_root: &Path,
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> EnforcementInputs {
    let mut rows: Vec<EnforcementRow> = Vec::new();
    let governance_substr = cfg.enforcement.governance_crate_substr.clone();

    // (1) oya-governance-* kernel crates: they name themselves "governance" enforcers,
    // but none is wired into the cloud-ci gate build graph (no gate BUCK target backs them).
    for cargo in tracked_paths_matching(repo_root, |p| {
        p.contains(&governance_substr) && p.ends_with("/Cargo.toml")
    }) {
        let crate_dir = cargo.trim_end_matches("/Cargo.toml");
        let id = crate_dir.rsplit('/').next().unwrap_or(crate_dir).to_owned();
        rows.push(EnforcementRow {
            id: format!("governance-crate:{id}"),
            source_artifact: cargo,
            claims_enforced: true,
            has_wired_buck2_target: false,
            maps_to_oya_cli: false,
        });
    }

    // (2) Governance lanes that claim a doc/coverage class is enforced.
    for lane in &cfg.enforcement.governance_lanes {
        let lane = lane.as_str();
        if repo_root.join(lane).is_file() {
            let id = lane.rsplit('/').next().unwrap_or(lane).trim_end_matches(".md");
            rows.push(EnforcementRow {
                id: format!("governance-lane:{id}"),
                source_artifact: lane.to_owned(),
                claims_enforced: true,
                has_wired_buck2_target: false,
                maps_to_oya_cli: false,
            });
        }
    }

    // (3) ADR `verified_by:` lines that name an `oya gate`/`oya gen` CLI invocation
    // (ADR-0365's retired CLI authority). Each is a blocking invariant mapped to oya CLI.
    let decisions_dir = repo_root.join(&cfg.justification.adr_dir);
    if let Ok(entries) = std::fs::read_dir(&decisions_dir) {
        let mut files: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .and_then(adr_id_from_filename)
                    .is_some()
            })
            .collect();
        files.sort();
        for path in files {
            let adr = path
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(adr_id_from_filename)
                .unwrap_or_default();
            let rel = path
                .strip_prefix(repo_root)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.to_string_lossy().to_string());
            let body = read_text(&path);
            let mut line_no = 0u64;
            for line in body.lines() {
                line_no += 1;
                let trimmed = line.trim();
                if trimmed.starts_with("verified_by")
                    && (trimmed.contains("oya gate") || trimmed.contains("oya gen"))
                {
                    rows.push(EnforcementRow {
                        id: format!("{adr}-verified_by-L{line_no}"),
                        source_artifact: rel.clone(),
                        claims_enforced: true,
                        has_wired_buck2_target: false,
                        maps_to_oya_cli: true,
                    });
                }
            }
        }
    }

    EnforcementInputs { rows }
}

/// Tracked paths matching a predicate (one `git ls-files` pass).
fn tracked_paths_matching(repo_root: &Path, pred: impl Fn(&str) -> bool) -> Vec<String> {
    git_ls_files(repo_root)
        .unwrap_or_default()
        .into_iter()
        .filter(|p| pred(p))
        .collect()
}

/// Read the value of a top-level YAML-ish front-matter scalar field (`key: value`).
fn front_matter_field(body: &str, key: &str) -> Option<String> {
    for line in front_matter_lines(body) {
        if let Some(rest) = line.strip_prefix(&format!("{key}:")) {
            let value = rest.trim().trim_matches('"').trim_matches('\'').trim();
            if !value.is_empty() {
                return Some(value.to_owned());
            }
        }
    }
    None
}

/// Parse a `key: [A, B, C]` front-matter array of ADR ids.
fn front_matter_id_array(body: &str, key: &str) -> Vec<String> {
    for line in front_matter_lines(body) {
        if let Some(rest) = line.strip_prefix(&format!("{key}:")) {
            let trimmed = rest.trim();
            if let Some(inner) = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                return inner
                    .split(',')
                    .map(|x| x.trim().trim_matches('"').to_owned())
                    .filter(|x| !x.is_empty())
                    .collect();
            }
        }
    }
    Vec::new()
}

/// The lines inside the leading `---` front-matter block.
fn front_matter_lines(body: &str) -> Vec<&str> {
    let mut lines = body.lines();
    if lines.next().map(str::trim) != Some("---") {
        return Vec::new();
    }
    let mut out = Vec::new();
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        out.push(line.trim_start());
    }
    out
}

/// Collect the real repo facts. git plumbing via std::process (no new crate); the
/// owner/justification/reachability maps are derived from the live repo sources.
fn collect_repo_inputs(
    repo_root: &Path,
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> Result<RepoInputs, CliError> {
    let tracked_paths = git_ls_files(repo_root)?;
    let last_touch = git_last_touch(repo_root)?;
    let owners = resolve_owners(repo_root, &tracked_paths, cfg);
    let reachability = resolve_reachability(repo_root, &tracked_paths, cfg);
    let justifications = resolve_justifications(repo_root, &tracked_paths, cfg);

    Ok(RepoInputs {
        tracked_paths,
        last_touch,
        owners,
        justifications,
        reachability,
        dup_of: BTreeMap::new(),
    })
}

fn git_ls_files(repo_root: &Path) -> Result<Vec<String>, CliError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("ls-files")
        .output()
        .map_err(|e| CliError::Git(format!("ls-files: {e}")))?;
    if !output.status.success() {
        return Err(CliError::Git(format!(
            "ls-files exit {:?}",
            output.status.code()
        )));
    }
    let mut paths: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .filter(|p| !p.is_empty())
        .collect();
    paths.sort();
    paths.dedup();
    Ok(paths)
}

/// One `git log --name-only` pass builds the path -> last-touch-commit map for the
/// whole tree (far cheaper than 24k per-path `git log -1` calls).
fn git_last_touch(repo_root: &Path) -> Result<BTreeMap<String, String>, CliError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["log", "--name-only", "--format=commit:%H"])
        .output()
        .map_err(|e| CliError::Git(format!("log: {e}")))?;
    if !output.status.success() {
        return Err(CliError::Git(format!("log exit {:?}", output.status.code())));
    }
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Some(sha) = line.strip_prefix("commit:") {
            current = Some(sha.to_owned());
        } else if !line.is_empty()
            && let Some(sha) = &current
        {
            // first time we see a path (walking newest-first) is its last touch
            map.entry(line.to_owned()).or_insert_with(|| sha.clone());
        }
    }
    Ok(map)
}

/// Resolve the nearest up-tree `OWNERS` file for each path. With zero OWNERS files
/// on the tree today this returns an empty map (every row ⇒ unowned), which is the
/// born-blocking exhibit — the gap is DATA (no OWNERS rows), not scanner code.
fn resolve_owners(
    repo_root: &Path,
    paths: &[String],
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> BTreeMap<String, String> {
    let owners_file = cfg.owners.file_name.as_str();
    let owners_dirs: BTreeSet<String> = paths
        .iter()
        .filter(|p| p.ends_with(&format!("/{owners_file}")) || p.as_str() == owners_file)
        .map(|p| {
            p.rsplit_once('/')
                .map(|(dir, _)| dir.to_owned())
                .unwrap_or_default()
        })
        .collect();
    let _ = repo_root; // OWNERS content parsing is the A-STRUCT follow-on; existence drives the gap
    let mut map = BTreeMap::new();
    if owners_dirs.is_empty() {
        return map;
    }
    for path in paths {
        if let Some(owner_dir) = nearest_ancestor(path, &owners_dirs) {
            map.insert(path.clone(), format!("OWNERS:{owner_dir}"));
        }
    }
    map
}

fn nearest_ancestor(path: &str, dirs: &BTreeSet<String>) -> Option<String> {
    let mut cursor = path;
    while let Some((parent, _)) = cursor.rsplit_once('/') {
        if dirs.contains(parent) {
            return Some(parent.to_owned());
        }
        cursor = parent;
    }
    if dirs.contains("") {
        return Some(String::new());
    }
    None
}

/// Reachability: a path is reachable if a live registry points at it. We resolve the
/// real registries (masterplan.json / root-hub-pointers.json / Cargo.toml members /
/// DOC-CATALOG) and mark each tracked path with the registries that mention it.
fn resolve_reachability(
    repo_root: &Path,
    paths: &[String],
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> BTreeMap<String, Vec<String>> {
    let masterplan = read_text(&repo_root.join(&cfg.reachability.masterplan));
    let root_hub = read_text(&repo_root.join(&cfg.reachability.root_hub));
    let doc_catalog = read_text(&repo_root.join(&cfg.reachability.doc_catalog));
    let cargo_members = read_cargo_member_prefixes(repo_root);

    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for path in paths {
        let mut reach: Vec<String> = Vec::new();
        if masterplan.contains(path.as_str()) {
            reach.push("masterplan".into());
        }
        if root_hub.contains(path.as_str()) {
            reach.push("root-hub".into());
        }
        if doc_catalog.contains(path.as_str()) {
            reach.push("doc-catalog".into());
        }
        if cargo_members.iter().any(|m| path.starts_with(m.as_str())) {
            reach.push("cargo-members".into());
        }
        if !reach.is_empty() {
            map.insert(path.clone(), reach);
        }
    }
    map
}

/// Member directory prefixes from the workspace Cargo.toml — a path under a member
/// crate dir is reachable from `cargo-members`.
fn read_cargo_member_prefixes(repo_root: &Path) -> Vec<String> {
    let text = read_text(&repo_root.join("Cargo.toml"));
    let mut prefixes = Vec::new();
    let mut in_members = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("members") && trimmed.contains('[') {
            in_members = true;
            continue;
        }
        if in_members {
            if trimmed.starts_with(']') {
                break;
            }
            if let Some(start) = trimmed.find('"')
                && let Some(end) = trimmed[start + 1..].find('"')
            {
                let member = &trimmed[start + 1..start + 1 + end];
                if !member.is_empty() {
                    prefixes.push(format!("{member}/"));
                }
            }
        }
    }
    prefixes
}

/// Justification: a path traces to a decision if an ADR mentions it (front-matter
/// `affected_surfaces` / body refs) or it lives under a decision-owned tree. Resolved
/// from the real ADR corpus.
///
/// Built as a single pass over the ADR corpus (NOT O(paths x ADRs)): each ADR body is
/// tokenized once into the repo-relative path-like tokens it references, populating a
/// `token -> first ADR id` index. Per-path lookup is then an O(1) map hit.
fn resolve_justifications(
    repo_root: &Path,
    paths: &[String],
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> BTreeMap<String, String> {
    let tracked: BTreeSet<&str> = paths.iter().map(String::as_str).collect();
    let decisions_dir = repo_root.join(&cfg.justification.adr_dir);

    // token (a tracked path mentioned in an ADR) -> first ADR id mentioning it.
    let mut mentioned: BTreeMap<String, String> = BTreeMap::new();
    let mut adr_files: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&decisions_dir) {
        for entry in entries.flatten() {
            if adr_id_from_filename(&entry.file_name().to_string_lossy()).is_some() {
                adr_files.push(entry.path());
            }
        }
    }
    adr_files.sort();

    for adr_path in &adr_files {
        let adr_id = match adr_path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(adr_id_from_filename)
        {
            Some(id) => id,
            None => continue,
        };
        let body = read_text(adr_path);
        // Walk whitespace/quote-delimited tokens; keep those that are tracked paths.
        for raw in body.split(|c: char| c.is_whitespace() || matches!(c, '"' | '`' | '(' | ')' | ',' | ';' | '[' | ']')) {
            let token = raw.trim_matches(|c: char| matches!(c, '.' | ':' | '#' | '*'));
            if token.len() >= 4 && token.contains('/') && tracked.contains(token) {
                mentioned.entry(token.to_owned()).or_insert_with(|| adr_id.clone());
            }
        }
    }

    let mut map = BTreeMap::new();
    for path in paths {
        // An ADR file justifies itself.
        if let Some(adr_id) = path
            .rsplit_once('/')
            .map(|(_, name)| name)
            .and_then(adr_id_from_filename)
        {
            map.insert(path.clone(), adr_id);
            continue;
        }
        if let Some(adr_id) = mentioned.get(path) {
            map.insert(path.clone(), adr_id.clone());
        }
    }
    map
}

fn adr_id_from_filename(name: &str) -> Option<String> {
    let rest = name.strip_prefix("ADR-")?;
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.len() == 4 {
        Some(format!("ADR-{digits}"))
    } else {
        None
    }
}

fn read_text(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

/// Whether a tracked path is excluded by the config's `[repo].path_excludes` (the producer's
/// `collect_*` filter). Reproduces the legacy `third-party/` semantics exactly: a path is
/// excluded iff, for some configured prefix P, it `starts_with(P)` OR `contains("/" + P)`
/// (so both a top-level `third-party/...` and a nested `.../third-party/...` are caught).
fn is_path_excluded(path: &str, cfg: &oya_ci_config_kernel::OyaCiConfig) -> bool {
    cfg.repo.path_excludes.iter().any(|prefix| {
        path.starts_with(prefix.as_str()) || path.contains(&format!("/{prefix}"))
    })
}
