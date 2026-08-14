//! Binary entry point for the accounting-registry producer.
//!
//! A PURE function of declared inputs: it reads `scm-facts.generated.json` (the committed,
//! registry-drift-protected snapshot of SCM boundary facts emitted by the out-of-graph
//! `oya-cloud-ci-scm-facts-emitter-app`) + the real reachability/owner/justification sources, then
//! delegates to the deterministic library to build the registry + companion faces. NO ambient
//! git — the producer never shells out (OYA-CI-HERMETIC-EXECUTION-DESIGN §1.3, Option C). This is
//! the buck2 `rust_binary` `//ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin`
//! (register #20 — NOT an `oya` CLI).
//!
//! Usage:
//!   oya-cloud-ci-accounting-registry-app [--repo-root <path>] [--out-dir <path>] [--stdout]
//!                                        [--policy-root <path>]
//!                                        [--scm-facts <path>]
//!                                        [--enforcement-liveness-claude-settings <path>]
//!                                        [--enforcement-liveness-codex-hooks <path>]
//!                                        [--enforcement-liveness-hooks-dir <path>]
//!                                        [--fix-owners <dir>=<owner>]
//!                                        [--fix-reachability <prefix>=<anchor>]
//!                                        [--check-paths <path>...] [--check-diff <merge-base>]
//!
//! `--check-paths`/`--check-diff` is the AUTHOR-SIDE pre-push check (FRIC #1328): for each
//! ADDED tracked file it reports reachable?/justified? and, if it would RED the
//! `[cloud-ci-total-accounting]` firewall, the exact remediation — reusing the SAME resolvers +
//! face-builder + firewall evaluator, and NEEDING NO materialized scm-facts face.
//!
//! With `--stdout` one generated face is written to stdout (used by the registry-drift gate
//! to regenerate in a sandbox and byte-diff). Default writes all generated faces under
//! `<out-dir>` (default `<repo-root>/ci/facade/artifact-inventory-registry`). `--scm-facts` defaults to the
//! committed `scm-facts.generated.json` beside the faces.
//!
//! `--fix-owners` / `--fix-reachability` are TRANSITIONAL local registration bridges
//! (ADR-0555; cli_surface_policy: retirement-marked local bridges, NEVER merge authority —
//! their successors are the GatePolicy/Baseline/Exception/GateRun reconcilers of ADR-0548
//! D3). Each takes the human DESIGN DECISION as input (who owns / why reached), applies the
//! exact registration edit, and SELF-VALIDATES by re-running the derivation before
//! reporting — automation applies edits, it never invents decisions.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ci_artifact_inventory_registry::{
    CrosswalkInputs, DecisionCrosswalkRow, EnforcementInputs, EnforcementRow, GateInputs,
    OwnersIntegrity, Policy, ProducerError, RepoInputs, adr_id_from_filename, allocate_next_adr_id,
    build_decision_crosswalk, build_enforcement_inventory, build_gate_baseline, build_registry,
    ENVELOPE_PREFIX_OWNERSHIP_SOURCE, ENVELOPES_RELPATH, fix_owners, fix_reachability,
    front_matter_field, load_envelope_prefix_allows, load_reachability_registry,
    registration_matches, resolve_owners, to_canonical_json,
};
use oya_check_brand_residue::forbidden_vocab::{
    CensusDocument, VocabPolicy, census_findings_with, is_path_carved_out_with,
    strict_zero_retired_brand_finding,
};
use serde_json::{Value, json};

const SCM_FACTS_SCHEMA: &str = "oya-ci/scm-facts/v2";

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
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Producer(e) => write!(f, "{e}"),
            CliError::Io(e) => write!(f, "io: {e}"),
        }
    }
}

/// The committed snapshot of STABLE SCM boundary facts (emitted by
/// `oya-cloud-ci-scm-facts-emitter-app`, schema v2 — ADR-0552). The producer is a pure
/// function of {this face + oya-ci.toml + the declared tracked tree}: every field is
/// TREE-derived, so the committed faces are invariant under history rewriting
/// (squash-merge) and under faces-only settle commits. History-derived volatile facts
/// (last-touch revision ids, timestamps, the aging anchor) are NOT here — they live in the
/// untracked scm-volatile-facts snapshot, consumed at evaluation time by the staleness
/// gate, never by this producer (FRIC-1781234047).
struct ScmFacts {
    /// The tracked-paths universe (sorted+deduped), as supplied by the ScmFactsSource.
    tracked_paths: Vec<String>,
}

/// Read + parse the scm-facts face. A missing/malformed face is a hard error (it must be
/// regenerated by the emitter), so a degraded checkout fails LOUDLY rather than silently
/// producing false-green faces.
fn load_scm_facts(path: &Path) -> Result<ScmFacts, CliError> {
    let text = std::fs::read_to_string(path).map_err(|e| {
        CliError::Io(format!(
            "{}: {e} (run oya-cloud-ci-scm-facts-emitter-app to regenerate the scm-facts face)",
            path.display()
        ))
    })?;
    let value: Value = serde_json::from_str(&text)
        .map_err(|e| CliError::Io(format!("{}: parse scm-facts: {e}", path.display())))?;

    let schema = value["schema"]
        .as_str()
        .ok_or_else(|| CliError::Io(format!("{}: missing schema", path.display())))?;
    if schema != SCM_FACTS_SCHEMA {
        return Err(CliError::Io(format!(
            "{}: unsupported scm-facts schema {schema:?} (expected {SCM_FACTS_SCHEMA})",
            path.display()
        )));
    }

    let tracked_path_values = value["tracked_paths"]
        .as_array()
        .ok_or_else(|| CliError::Io(format!("{}: missing tracked_paths", path.display())))?;
    let mut tracked_paths = Vec::with_capacity(tracked_path_values.len());
    for (index, tracked_path) in tracked_path_values.iter().enumerate() {
        let Some(tracked_path) = tracked_path.as_str() else {
            return Err(CliError::Io(format!(
                "{}: malformed tracked_paths[{index}]: expected string",
                path.display()
            )));
        };
        tracked_paths.push(decode_tracked_path(tracked_path)?);
    }
    // Decoding reorders relative to the emitter's sort over the QUOTED spellings (`"` sorts
    // below every path character), so re-establish the sorted+deduped invariant the face
    // declares and every consumer assumes.
    tracked_paths.sort();
    tracked_paths.dedup();

    Ok(ScmFacts { tracked_paths })
}

/// THE ingestion boundary for a Git pathname: decode the C-quoted spelling ONCE, here, so
/// every downstream key — the accounting row `path`, the ownership walk, the justification
/// lookup, the reachability match, the brand-residue census key — is the real pathname.
/// Before this, a quoted path was keyed by its `"…\302\265…"` spelling, so the ownership walk
/// climbed ancestors of `"marketplace` and matched no OWNERS, no justification and no
/// reachability entry: permanently unowned/unjustified/unreachable.
///
/// NON-UTF-8 POLICY: fail closed. Such bytes cannot round-trip through the JSON `path` field,
/// and a lossily-renamed key would silently reintroduce exactly the mismatch class this decode
/// removes — a wrong key that LOOKS right is worse than a loud stop. Rename the file to UTF-8.
fn decode_tracked_path(tracked_path: &str) -> Result<String, CliError> {
    String::from_utf8(decode_git_path(tracked_path)?).map_err(|_| {
        CliError::Io(format!(
            "tracked path {tracked_path:?} decodes to non-UTF-8 bytes: it cannot be a canonical \
             accounting key without a lossy rename, which would silently mis-key ownership, \
             justification and reachability. Rename the file to UTF-8."
        ))
    })
}

impl From<ProducerError> for CliError {
    fn from(e: ProducerError) -> Self {
        CliError::Producer(e)
    }
}

impl CliError {
    /// Map a registration-BRIDGE error (`fix_owners` / `fix_reachability`, slice 2.5) back to
    /// the binary's `CliError`. The bridges previously raised `CliError::Io` for every failure,
    /// so their library variants (`Io`/`Validation`/`Refused`) re-wrap into `CliError::Io` —
    /// keeping the `io: <message>` stderr byte-identical to before the extraction. The
    /// face-builder variants (`Policy`/`Serialize`) keep their `CliError::Producer` mapping.
    fn from_bridge(e: ProducerError) -> Self {
        match e.bridge_message() {
            Some(message) => CliError::Io(message.to_owned()),
            None => CliError::Producer(e),
        }
    }
}

fn run() -> Result<(), CliError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut repo_root: Option<PathBuf> = None;
    let mut policy_root: Option<PathBuf> = None;
    let mut out_dir: Option<PathBuf> = None;
    let mut scm_facts_path: Option<PathBuf> = None;
    let mut enforcement_liveness_claude_settings: Option<PathBuf> = None;
    let mut enforcement_liveness_codex_hooks: Option<PathBuf> = None;
    let mut enforcement_liveness_hooks_dir: Option<PathBuf> = None;
    let mut to_stdout = false;
    // Allocator mode (FRIC-1781320000): print the next unallocated decision number derived
    // from the tree and exit. Lanes allocate ADR numbers by running this, never by
    // convention or leader memory.
    let mut next_adr_mode = false;
    // Which face to emit to stdout: default registry. The gate self-tests + registry-drift
    // regenerate a single face in a sandbox via `--stdout --face <name>`.
    let mut face = "registry".to_owned();
    // The TRANSITIONAL registration bridges (ADR-0555; cli_surface_policy local bridge).
    let mut fix_owners_spec: Option<String> = None;
    let mut fix_reachability_spec: Option<String> = None;
    // AUTHOR-SIDE pre-push check (FRIC #1328): "will these newly-added paths RED the
    // [cloud-ci-total-accounting] firewall?" `--check-paths <path>...` names them explicitly;
    // `--check-diff <merge-base>` derives added tracked files via `git diff --diff-filter=A`.
    // Additive flags only — CI's default face-production path is untouched.
    let mut check_paths: Option<Vec<String>> = None;
    let mut check_diff_base: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--repo-root" => {
                i += 1;
                repo_root = args.get(i).map(PathBuf::from);
            }
            "--policy-root" => {
                i += 1;
                policy_root = args.get(i).map(PathBuf::from);
            }
            "--out-dir" => {
                i += 1;
                out_dir = args.get(i).map(PathBuf::from);
            }
            "--scm-facts" => {
                i += 1;
                scm_facts_path = args.get(i).map(PathBuf::from);
            }
            "--enforcement-liveness-claude-settings" => {
                i += 1;
                enforcement_liveness_claude_settings = args.get(i).map(PathBuf::from);
            }
            "--enforcement-liveness-codex-hooks" => {
                i += 1;
                enforcement_liveness_codex_hooks = args.get(i).map(PathBuf::from);
            }
            "--enforcement-liveness-hooks-dir" => {
                i += 1;
                enforcement_liveness_hooks_dir = args.get(i).map(PathBuf::from);
            }
            "--face" => {
                i += 1;
                if let Some(value) = args.get(i) {
                    face = value.clone();
                }
            }
            "--fix-owners" => {
                i += 1;
                fix_owners_spec = args.get(i).cloned();
            }
            "--fix-reachability" => {
                i += 1;
                fix_reachability_spec = args.get(i).cloned();
            }
            "--check-paths" => {
                // Multi-value: consume following args until the next `--flag` or end.
                let mut collected = Vec::new();
                while i + 1 < args.len() && !args[i + 1].starts_with("--") {
                    i += 1;
                    collected.push(args[i].clone());
                }
                check_paths = Some(collected);
            }
            "--check-diff" => {
                i += 1;
                check_diff_base = args.get(i).cloned();
            }
            "--stdout" => to_stdout = true,
            "--next-adr" => next_adr_mode = true,
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
    let out_dir =
        out_dir.unwrap_or_else(|| repo_root.join("ci/facade/artifact-inventory-registry"));
    let policy_root = policy_root.as_deref();

    // Allocator mode: derive the next free decision number from the tree and exit.
    // Single-owner: the SAME `allocate_next_adr_id` the crosswalk pass and the slice-3
    // register_crate app call — one ADR-parsing implementation, no convention or leader
    // memory. Same config-declared decisions directory, same scan, byte-identical output.
    if next_adr_mode {
        let cfg = load_policy_config(&repo_root, policy_root)?;
        let decisions_dir = repo_root.join(&cfg.justification.adr_dir);
        println!("{}", allocate_next_adr_id(&decisions_dir)?);
        return Ok(());
    }

    // AUTHOR-SIDE pre-push check mode (FRIC #1328). It answers "will my new files RED the
    // firewall?" BEFORE push, WITHOUT a materialized scm-facts face: the tracked-path universe
    // is the added set itself, resolved with the SAME producer resolvers + face-builder + the
    // firewall's own evaluator (drift-proof). No scm-facts load, no all-face derivation.
    if check_paths.is_some() || check_diff_base.is_some() {
        let cfg = load_policy_config(&repo_root, policy_root)?;
        let policy = Policy::from_config(&cfg)?;
        let mut paths: Vec<String> = check_paths.unwrap_or_default();
        if let Some(base) = check_diff_base {
            paths.extend(git_added_paths(&repo_root, &base)?);
        }
        paths.sort();
        paths.dedup();
        let verdicts = check_added_paths(&repo_root, &cfg, &policy, &paths)?;
        if !report_check(&verdicts) {
            // Distinct exit code (2) so a pre-push wrapper can gate on "would RED" without
            // confusing it with a usage/IO error (exit 1).
            std::process::exit(2);
        }
        return Ok(());
    }

    // The committed scm-facts face (the declared input that replaces ambient git). Defaults to
    // the face beside the accounting faces; CI / the local regen hook re-run the emitter to keep
    // it current, and registry-drift byte-diffs it like the other faces.
    let scm_facts_path = scm_facts_path.unwrap_or_else(|| {
        repo_root.join("ci/facade/artifact-inventory-registry/scm-facts.generated.json")
    });
    let scm_facts = load_scm_facts(&scm_facts_path)?;

    // The oya-ci policy (naming/vocab/manifest/roots/sources/gates) is sourced from the active
    // policy root's `oya-ci.toml` (OYA-CI-CONFORMANCE-FLOOR-PLAN §3.3 / Stage 2), via the
    // CLOSED-schema loader. `--policy-root` is the cloud-ci product boundary: the candidate repo
    // supplies the corpus under test, while a controller can source gate policy from trusted
    // control state. With no `--policy-root`, the default remains `<repo-root>/oya-ci.toml`,
    // preserving today's byte-for-byte first-party behaviour. Frozen-reference regeneration runs
    // the candidate producer over a historical merge-base source tree. Schema v1 line rules had
    // no `exempt_stems` and skipped the whole matching line, so this exact output mode uses the
    // config kernel's bounded v1 compatibility parser. Every candidate production/check path
    // remains on the strict v2 parser.
    let cfg = if to_stdout && face == "baseline" {
        load_frozen_reference_policy_config(&repo_root, policy_root)?
    } else {
        load_policy_config(&repo_root, policy_root)?
    };
    let config_digest = cfg.digest();

    // The TRANSITIONAL registration bridges run INSTEAD of face generation: apply the
    // human-decided registration edit, self-validate the derivation, report, exit. The
    // bridge LOGIC lives in the library (slice 2.5 — callable cross-crate by the slice-3
    // register_crate app, no subprocess); this binary is the thin retirement-marked CLI
    // adapter that maps the library `ProducerError` back to its `CliError` (so the io-shaped
    // messages + exit code are byte-identical to before the extraction).
    if let Some(spec) = fix_owners_spec {
        let message = fix_owners(&repo_root, &cfg, &scm_facts.tracked_paths, &spec)
            .map_err(CliError::from_bridge)?;
        println!("{message}");
        return Ok(());
    }
    if let Some(spec) = fix_reachability_spec {
        let message = fix_reachability(&repo_root, &cfg, &scm_facts.tracked_paths, &spec)
            .map_err(CliError::from_bridge)?;
        println!("{message}");
        return Ok(());
    }

    let policy = Policy::from_config(&cfg)?;
    let (inputs, owners_integrity) = collect_repo_inputs(&repo_root, &cfg, &scm_facts)?;
    // Fast path for the license-policy producer face: this new face only needs the tracked path
    // universe plus resolved workspace manifests. Avoid forcing callers such as the gate's live
    // corpus test through the expensive all-face registry/baseline derivation when they requested
    // exactly `--stdout --face license-policy`.
    if to_stdout && face == "license-policy" {
        let license_policy = collect_license_policy(&repo_root, &inputs.tracked_paths, &cfg)?;
        print!("{}", to_canonical_json(&license_policy)?);
        return Ok(());
    }
    // OWNERS integrity remediation (ADR-0555 hardening, FRIC-1781400000): name the exact
    // fix for every OWNERS file that failed the content schema or the breadth bound — the
    // affected paths stay UNOWNED (fail-closed) and the firewall's unowned remediation
    // carries the same fix, so a FAIL is never a bare flag.
    for (file, defect) in &owners_integrity.invalid {
        eprintln!(
            "owners integrity: {file} is NOT a valid ownership marker — {defect}; its \
             subtree stays unowned (fail-closed, no fall-through to a broader ancestor); \
             exact fix: rewrite {file} to the OWNERS schema — one owner principal per \
             line (lowercase alphanumeric + interior hyphens), `#` comments allowed, at \
             least one principal required"
        );
    }
    for (file, coverage) in &owners_integrity.over_broad {
        let bound = cfg.owners.max_paths_per_owners_file;
        eprintln!(
            "owners integrity: {file} covers {coverage} tracked paths, over the [owners] \
             max_paths_per_owners_file bound ({bound}); the excess stays unowned; exact \
             fix: split the registration — add OWNERS files in child subtrees so no \
             single file covers more than {bound} paths"
        );
    }
    let registry = build_registry(&inputs, &policy)?;
    let crosswalk_inputs = collect_crosswalk_inputs(&repo_root, &cfg);
    if !crosswalk_inputs.duplicate_ids.is_empty() || !crosswalk_inputs.id_mismatches.is_empty() {
        // Decision-id integrity remediation (FRIC-1781320000): name the exact renumber
        // target so a collision/mismatch is fixable from this output alone.
        eprintln!(
            "decision-id integrity: duplicate ids {:?}; filename/front-matter mismatches {:?}; \
             remediation: renumber the newer decision (filename AND front-matter id) to the next \
             free number {} (allocate via --next-adr)",
            crosswalk_inputs.duplicate_ids,
            crosswalk_inputs.id_mismatches,
            crosswalk_inputs.next_free_id
        );
    }
    let crosswalk = build_decision_crosswalk(&crosswalk_inputs)?;
    let enforcement =
        build_enforcement_inventory(&collect_enforcement_inputs(&repo_root, &cfg, &scm_facts))?;

    // The fifth face freezes TODAY's accepted-violation KEYS per (gate, code). It runs each
    // gate's pure evaluate_keyed over the live faces. The automation matrix is derived from
    // the enforcement-inventory face. Staleness keys are deliberately ABSENT from the
    // committed baseline (ADR-0552): they derive from HISTORY-volatile aging data, so
    // freezing them in a committed face would re-create the squash-merge un-settle defect
    // through the back door. The staleness gate ages rows from the untracked volatile
    // snapshot at evaluation time; its blocking authority is its own gate lane.
    let automation_matrix = build_automation_matrix(&enforcement);
    // The fifth gate (cloud-ci-brand-residue) scans the raw tracked corpus only when baseline
    // materialization needs it. Other single-face stdout queries never consume this input.
    let brand_residue = if should_collect_brand_residue(to_stdout, &face) {
        collect_brand_residue(&repo_root, &inputs.tracked_paths, &cfg)?
    } else {
        BTreeMap::new()
    };
    // The §2.5#4 bnf-layer-suffix gate input: the first-party oya-* crate names enumerated from
    // the tracked Cargo.toml manifests. The gate's evaluate_keyed resolves the role carve-out-
    // aware and reuses oya_governance_predictable_naming_kernel::check.
    let bnf_layer_suffix = collect_bnf_layer_suffix(&repo_root, &inputs.tracked_paths, &cfg);
    // The §2.5#7 manifest-hygiene gate input: per-crate Cargo.toml hygiene flags.
    let manifest_hygiene = collect_manifest_hygiene(&repo_root, &inputs.tracked_paths, &cfg);
    // The ADR-0017 cargo-prefix gate input: every tracked first-party workspace member
    // candidate + package name. De-branded candidates stay visible but are advisory-scoped so
    // expanding the corpus cannot create new born-blocking `cargo_prefix_violation` debt.
    let cargo_prefix = collect_cargo_prefix(&repo_root, &inputs.tracked_paths, &cfg)?;
    // The SLO coverage gate input: the config-declared catalog record globs expanded over the
    // tracked-path universe. This makes the lane input contract portable DATA instead of an
    // Oyatie-only hardcoded directory walk.
    let slo_coverage = collect_slo_coverage(&repo_root, &inputs.tracked_paths, &cfg)?;
    // The license-policy gate input: workspace package-license rows from resolved member
    // manifests. The producer owns all filesystem I/O; the gate remains pure and surface-all.
    let license_policy = collect_license_policy(&repo_root, &inputs.tracked_paths, &cfg)?;
    // The catalog-liveness gate input: the config-declared catalog globs expanded over the
    // tracked-path universe, each row tagged with whether its stem is a LIVE workspace crate-id
    // (resolved IN-PROCESS via oya-workspace-members-kernel — no shell-out) and its explicit
    // non-live marker (status:/non_claims). The gate enforces the founder live-OR-marked policy.
    let catalog_liveness = collect_catalog_liveness(&repo_root, &inputs.tracked_paths, &cfg)?;
    // The ADR-0538 workspace-glob-coverage gate input: root member entries plus concrete
    // first-party crate-dir coverage against the canonical glob-aware workspace-member resolver.
    let workspace_glob_coverage =
        collect_workspace_glob_coverage(&repo_root, &inputs.tracked_paths, &cfg)?;
    let target_parity = collect_target_parity(&repo_root, &inputs.tracked_paths, &cfg)?;
    let collect_declared_enforcement_liveness = || -> Result<Value, CliError> {
        let enforcement_liveness_corpus = EnforcementLivenessCorpus::from_args(
            &inputs.tracked_paths,
            enforcement_liveness_claude_settings.as_deref(),
            enforcement_liveness_codex_hooks.as_deref(),
            enforcement_liveness_hooks_dir.as_deref(),
        )?;
        collect_enforcement_liveness(&inputs.tracked_paths, &enforcement_liveness_corpus)
    };
    let build_baseline_face = |enforcement_liveness: &Value| -> Result<Value, CliError> {
        let gate_inputs = GateInputs {
            total_accounting: &registry,
            cross_artifact: &crosswalk,
            automation_ratchet: &automation_matrix,
            bnf_layer_suffix: &bnf_layer_suffix,
            manifest_hygiene: &manifest_hygiene,
            cargo_prefix: &cargo_prefix,
            slo_coverage: &slo_coverage,
            license_policy: &license_policy,
            catalog_liveness: &catalog_liveness,
            workspace_glob_coverage: &workspace_glob_coverage,
            target_parity: &target_parity,
            enforcement_liveness,
            brand_residue: &brand_residue,
        };
        Ok(build_gate_baseline(&cfg, &gate_inputs, &config_digest)?)
    };

    if to_stdout {
        match face.as_str() {
            "registry" => print!("{}", to_canonical_json(&registry)?),
            "decision-crosswalk" => print!("{}", to_canonical_json(&crosswalk)?),
            "enforcement-inventory" => print!("{}", to_canonical_json(&enforcement)?),
            "ttl-policy" => print!("{}", to_canonical_json(&policy.ttl_policy_face())?),
            "bnf-layer-suffix" => print!("{}", to_canonical_json(&bnf_layer_suffix)?),
            "manifest-hygiene" => print!("{}", to_canonical_json(&manifest_hygiene)?),
            "cargo-prefix" => print!("{}", to_canonical_json(&cargo_prefix)?),
            "slo-coverage" => print!("{}", to_canonical_json(&slo_coverage)?),
            "license-policy" => print!("{}", to_canonical_json(&license_policy)?),
            "catalog-liveness" => print!("{}", to_canonical_json(&catalog_liveness)?),
            "workspace-glob-coverage" => print!("{}", to_canonical_json(&workspace_glob_coverage)?),
            "target-parity" => print!("{}", to_canonical_json(&target_parity)?),
            "enforcement-liveness" => {
                let enforcement_liveness = collect_declared_enforcement_liveness()?;
                print!("{}", to_canonical_json(&enforcement_liveness)?);
            }
            "baseline" => {
                let enforcement_liveness = collect_declared_enforcement_liveness()?;
                let baseline = build_baseline_face(&enforcement_liveness)?;
                print!("{}", to_canonical_json(&baseline)?);
            }
            other => return Err(CliError::Io(format!("unknown --face {other}"))),
        }
        return Ok(());
    }

    let enforcement_liveness = collect_declared_enforcement_liveness()?;
    let baseline = build_baseline_face(&enforcement_liveness)?;

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
    write_face(
        &out_dir.join("enforcement-liveness.generated.json"),
        &enforcement_liveness,
    )?;
    write_face(&out_dir.join("gate-baseline.generated.json"), &baseline)?;

    let rows = registry["rows"].as_array().map(Vec::len).unwrap_or(0);
    eprintln!(
        "oya-cloud-ci-accounting-registry-app: {rows} rows -> {}",
        out_dir.display()
    );
    Ok(())
}

/// Load the cloud-ci policy config. `repo_root` is the candidate corpus under test;
/// `policy_root`, when present, is trusted control state carrying `oya-ci.toml`.
fn load_policy_config(
    repo_root: &Path,
    policy_root: Option<&Path>,
) -> Result<oya_ci_config_kernel::OyaCiConfig, CliError> {
    load_config(policy_root.unwrap_or(repo_root))
}

/// Load policy for ADR-0616 merge-base regeneration. This compatibility path exists only for
/// `--stdout --face baseline`: it reproduces schema v1's whole-line exception semantics while the
/// strict candidate loader rejects missing v2 `exempt_stems`.
fn load_frozen_reference_policy_config(
    repo_root: &Path,
    policy_root: Option<&Path>,
) -> Result<oya_ci_config_kernel::OyaCiConfig, CliError> {
    load_frozen_reference_config(policy_root.unwrap_or(repo_root))
}

/// Load a root's `oya-ci.toml` (OYA-CI-CONFORMANCE-FLOOR-PLAN §3.3). When the file is present
/// it is parsed by the CLOSED-schema loader (a malformed file / unknown key is a hard error, so
/// a broken config fails LOUDLY rather than silently reverting policy); when it is absent the
/// compiled-in bundled default applies (zero-config = today's language-agnostic posture).
fn load_config(config_root: &Path) -> Result<oya_ci_config_kernel::OyaCiConfig, CliError> {
    let path = config_root.join("oya-ci.toml");
    match std::fs::read_to_string(&path) {
        Ok(text) => oya_ci_config_kernel::OyaCiConfig::from_toml_str(&text)
            .map_err(|e| CliError::Io(format!("{}: {e}", path.display()))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Ok(oya_ci_config_kernel::OyaCiConfig::bundled_default())
        }
        Err(e) => Err(CliError::Io(format!("{}: {e}", path.display()))),
    }
}

fn load_frozen_reference_config(
    config_root: &Path,
) -> Result<oya_ci_config_kernel::OyaCiConfig, CliError> {
    let path = config_root.join("oya-ci.toml");
    match std::fs::read_to_string(&path) {
        Ok(text) => oya_ci_config_kernel::OyaCiConfig::from_frozen_reference_toml_str(&text)
            .map_err(|e| CliError::Io(format!("{}: {e}", path.display()))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Ok(oya_ci_config_kernel::OyaCiConfig::frozen_reference_bundled_default())
        }
        Err(e) => Err(CliError::Io(format!("{}: {e}", path.display()))),
    }
}

/// Config-driven repo-root discovery (ADR-0533 §Decision item 4 — the config-driven test/run
/// harness). The root is located by, in order: (1) the `OYA_CI_REPO_ROOT` env override (a portable
/// escape hatch so an adopter's runner — or a hermetic test — can pin the root without an on-disk
/// marker walk); (2) walking up-tree until any `[repo].root_markers` entry is present. The markers
/// are DATA (the neutral profile uses the generic `.git`; oyatie uses `specs/root-hub-pointers.json`),
/// so the producer is not hardcoded to the oyatie marker.
fn discover_repo_root(cfg: &oya_ci_config_kernel::OyaCiConfig) -> Result<PathBuf, CliError> {
    if let Some(root) = std::env::var_os("OYA_CI_REPO_ROOT") {
        return Ok(PathBuf::from(root));
    }
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
            "target_gate_or_controller": if maps_oya { "retired oya CLI authority claim" } else { src },
            "blocking_fixture": "specs/fixtures/phase0-automation-ratchet/",
            "retirement_phase": "P0.0",
            "evidence_path": src,
            "no_new_oya_cli_surface": !maps_oya,
            "claims_enforced": claims,
            "has_wired_buck2_target": wired,
            "requires_pre_merge_review_authority": surface["requires_pre_merge_review_authority"].as_bool() == Some(true),
            "review_authority_live": surface["review_authority_live"].as_bool() == Some(true),
            "review_authority_source": surface["review_authority_source"].as_str().unwrap_or(""),
            "has_durable_review_evidence": surface["has_durable_review_evidence"].as_bool() == Some(true),
            "has_machine_verifiable_review_status": surface["has_machine_verifiable_review_status"].as_bool() == Some(true),
            "binds_pr_number": surface["binds_pr_number"].as_bool() == Some(true),
            "binds_head_sha": surface["binds_head_sha"].as_bool() == Some(true),
            "binds_author_identity": surface["binds_author_identity"].as_bool() == Some(true),
            "binds_reviewer_identity": surface["binds_reviewer_identity"].as_bool() == Some(true),
            "binds_review_verdict": surface["binds_review_verdict"].as_bool() == Some(true),
            "review_blocks_merge": surface["review_blocks_merge"].as_bool() == Some(true),
            "reviewer_identity_distinct_from_author": surface["reviewer_identity_distinct_from_author"].as_bool() == Some(true)
        }));
    }
    json!({ "rows": matrix_rows })
}

fn should_collect_brand_residue(to_stdout: bool, face: &str) -> bool {
    !to_stdout || face == "baseline"
}

/// Build the cloud-ci-brand-residue gate's `code -> keys`. The legacy forbidden-vocab census
/// remains a shrink-only ratchet with policy carve-outs. The strict-zero retired-brand class is
/// folded into the same gate but separately reads every tracked pathname and raw blob, including
/// every path carved out of the legacy census.
///
/// Deterministic + churn-free: per-file keys are stable under in-file edits (line numbers
/// never enter the key), so editing prose in an already-listed file stays GREEN; only fully
/// cleaning a file shrinks the set.
fn collect_brand_residue(
    repo_root: &Path,
    tracked_paths: &[String],
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> Result<BTreeMap<String, BTreeSet<String>>, CliError> {
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
    for path in tracked_paths {
        let (raw_path, raw_blob) = read_tracked_blob(repo_root, path)?;
        if let Some(finding) = strict_zero_retired_brand_finding(path, &raw_path, &raw_blob) {
            grouped.entry(finding.code).or_default().insert(finding.key);
        }
    }
    Ok(grouped)
}

/// Read the exact working-tree representation of a tracked Git blob. Regular files are read
/// as arbitrary bytes. Symlinks contribute their link payload rather than the target contents,
/// matching Git's blob semantics. Every error is fatal so an incomplete checkout cannot erase a
/// strict-zero finding.
///
/// `tracked_path` is ALREADY decoded ([`decode_tracked_path`] at the ingestion boundary), so it
/// IS the real pathname and its bytes ARE the real path bytes. Decoding again here would be a
/// second, position-dependent decode: a file whose real name legitimately starts with `"` would
/// be decoded twice and resolve to the wrong path (or fail with a misleading quoting error).
fn read_tracked_blob(repo_root: &Path, tracked_path: &str) -> Result<(Vec<u8>, Vec<u8>), CliError> {
    let raw_path = tracked_path.as_bytes().to_vec();
    let relative_path = PathBuf::from(tracked_path);
    if relative_path.as_os_str().is_empty()
        || relative_path.is_absolute()
        || relative_path.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir | std::path::Component::RootDir
            )
        })
    {
        return Err(CliError::Io(format!(
            "strict-zero brand scan: tracked path is not repo-relative: {tracked_path:?}"
        )));
    }
    let path = repo_root.join(relative_path);
    let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
        CliError::Io(format!(
            "strict-zero brand scan: inspect tracked path {tracked_path:?}: {error}"
        ))
    })?;
    if metadata.file_type().is_symlink() {
        let target = std::fs::read_link(&path).map_err(|error| {
            CliError::Io(format!(
                "strict-zero brand scan: read tracked symlink {tracked_path:?}: {error}"
            ))
        })?;
        return Ok((raw_path, target.as_os_str().as_encoded_bytes().to_vec()));
    }
    if !metadata.is_file() {
        return Err(CliError::Io(format!(
            "strict-zero brand scan: tracked path {tracked_path:?} is not a file or symlink"
        )));
    }
    let raw_blob = std::fs::read(&path).map_err(|error| {
        CliError::Io(format!(
            "strict-zero brand scan: read tracked blob {tracked_path:?}: {error}"
        ))
    })?;
    Ok((raw_path, raw_blob))
}

/// Decode the line-oriented pathname representation emitted by `git ls-files` without `-z`.
/// Git wraps paths requiring quoting in double quotes and uses C escapes, including three-digit
/// octal escapes for arbitrary bytes. Unquoted entries are already their literal byte sequence.
fn decode_git_path(tracked_path: &str) -> Result<Vec<u8>, CliError> {
    let encoded = tracked_path.as_bytes();
    if encoded.first() != Some(&b'"') {
        return Ok(encoded.to_vec());
    }
    if encoded.len() < 2 || encoded.last() != Some(&b'"') {
        return Err(invalid_git_path(tracked_path, "unterminated quoted path"));
    }

    let mut decoded = Vec::with_capacity(encoded.len() - 2);
    let mut index = 1;
    let end = encoded.len() - 1;
    while index < end {
        let byte = encoded[index];
        if byte == b'"' {
            return Err(invalid_git_path(tracked_path, "unescaped quote"));
        }
        if byte != b'\\' {
            decoded.push(byte);
            index += 1;
            continue;
        }

        index += 1;
        if index >= end {
            return Err(invalid_git_path(tracked_path, "trailing escape"));
        }
        let escaped = encoded[index];
        match escaped {
            b'a' => decoded.push(0x07),
            b'b' => decoded.push(0x08),
            b't' => decoded.push(b'\t'),
            b'n' => decoded.push(b'\n'),
            b'v' => decoded.push(0x0b),
            b'f' => decoded.push(0x0c),
            b'r' => decoded.push(b'\r'),
            b'\\' => decoded.push(b'\\'),
            b'"' => decoded.push(b'"'),
            first @ b'0'..=b'3' => {
                if index + 2 >= end {
                    return Err(invalid_git_path(tracked_path, "short octal escape"));
                }
                let second = encoded[index + 1];
                let third = encoded[index + 2];
                if !(b'0'..=b'7').contains(&second) || !(b'0'..=b'7').contains(&third) {
                    return Err(invalid_git_path(tracked_path, "invalid octal escape"));
                }
                decoded.push(((first - b'0') << 6) | ((second - b'0') << 3) | (third - b'0'));
                index += 2;
            }
            _ => return Err(invalid_git_path(tracked_path, "unknown escape")),
        }
        index += 1;
    }
    if decoded.contains(&0) {
        return Err(invalid_git_path(tracked_path, "NUL byte"));
    }
    Ok(decoded)
}

fn invalid_git_path(tracked_path: &str, reason: &str) -> CliError {
    CliError::Io(format!(
        "strict-zero brand scan: invalid Git C-quoted tracked path {tracked_path:?}: {reason}"
    ))
}

// The former `path_buf_from_bytes` (unix / non-unix pair) is gone: canonical tracked paths are
// UTF-8 `String`s by construction (`decode_tracked_path` fails closed otherwise), so
// `PathBuf::from(&str)` is correct on every platform.

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
                exempt_stems: c.exempt_stems.clone(),
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

/// Enumerate every tracked first-party workspace member candidate + package name (the ADR-0017
/// cargo-prefix gate's I/O). For each resolved workspace member whose manifest is present in the
/// tracked-path universe, emit `{"member_path": "<dir>", "package_name": "<name>"}` plus an
/// explicit `cargo_prefix_scope`. Rows whose crate-id and package name still carry the configured
/// brand prefix remain blocking; partially or fully de-branded rows stay in the face as advisory
/// candidate coverage so visibility expansion does not become a northstar-debrand merge blocker.
fn cargo_prefix_crate_id(member_path: &str) -> &str {
    member_path.rsplit('/').next().unwrap_or(member_path)
}

fn cargo_prefix_scope(crate_id: &str, package_name: &str, required_prefix: &str) -> &'static str {
    if required_prefix.is_empty() {
        return "advisory";
    }
    if crate_id.starts_with(required_prefix) && package_name.starts_with(required_prefix) {
        "blocking"
    } else {
        "advisory"
    }
}

/// Return valid workspace member directories while preserving invalid matches for the dedicated
/// workspace-glob-coverage face. Structural manifest errors still fail every producer face.
fn scan_valid_member_dirs(repo_root: &Path, face: &str) -> Result<Vec<String>, CliError> {
    oya_workspace_members_kernel::scan_member_dirs(repo_root)
        .map(|scan| scan.member_dirs)
        .map_err(|error| CliError::Io(format!("{face} scan member dirs: {error}")))
}

fn collect_cargo_prefix(
    repo_root: &Path,
    tracked_paths: &[String],
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> Result<Value, CliError> {
    let tracked: BTreeSet<&str> = tracked_paths
        .iter()
        .filter(|path| !is_path_excluded(path, cfg))
        .map(String::as_str)
        .collect();
    let member_dirs = scan_valid_member_dirs(repo_root, "cargo-prefix")?;

    let mut by_member: BTreeMap<String, String> = BTreeMap::new();
    for member_path in member_dirs {
        let manifest_path = format!("{member_path}/Cargo.toml");
        if !tracked.contains(manifest_path.as_str()) {
            continue;
        }
        let Some(name) = parse_package_name(&read_text(&repo_root.join(&manifest_path))) else {
            continue;
        };
        by_member.insert(member_path, name);
    }

    let rows: Vec<Value> = by_member
        .into_iter()
        .map(|(member_path, package_name)| {
            let crate_id = cargo_prefix_crate_id(&member_path);
            let scope = cargo_prefix_scope(crate_id, &package_name, &cfg.naming.required_prefix);
            json!({
                "member_path": member_path,
                "package_name": package_name,
                "cargo_prefix_scope": scope,
            })
        })
        .collect();
    Ok(json!({ "rows": rows }))
}

/// Enumerate every tracked workspace member package name + package license for the
/// cloud-ci-license-policy gate. This is the producer-owned I/O counterpart to the pure
/// `oya-cloud-ci-license-policy-app` evaluator: workspace membership is resolved in-process via
/// the canonical glob-aware resolver, and the gate receives only data rows.
fn collect_license_policy(
    repo_root: &Path,
    tracked_paths: &[String],
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> Result<Value, CliError> {
    let tracked: BTreeSet<&str> = tracked_paths
        .iter()
        .filter(|path| !is_path_excluded(path, cfg))
        .map(String::as_str)
        .collect();
    let member_dirs = scan_valid_member_dirs(repo_root, "license-policy")?;

    let mut rows: Vec<Value> = Vec::new();
    for member_path in member_dirs {
        let manifest_path = format!("{member_path}/Cargo.toml");
        if !tracked.contains(manifest_path.as_str()) {
            continue;
        }
        let contents = read_text(&repo_root.join(&manifest_path));
        let Some(package_name) = parse_package_name(&contents) else {
            continue;
        };
        rows.push(json!({
            "package_name": package_name,
            "manifest_path": manifest_path,
            "license": parse_package_license(&contents),
        }));
    }
    rows.sort_by(|a, b| {
        a["package_name"]
            .as_str()
            .unwrap_or("")
            .cmp(b["package_name"].as_str().unwrap_or(""))
            .then_with(|| {
                a["manifest_path"]
                    .as_str()
                    .unwrap_or("")
                    .cmp(b["manifest_path"].as_str().unwrap_or(""))
            })
    });
    Ok(json!({ "rows": rows }))
}

/// Enumerate SLO catalog rows from the config-declared `[slo_coverage].catalog_record_globs`.
/// This replaces the legacy dev-cli's implicit `registry/catalog` walk with a portable, closed-
/// schema input contract. The current default still mirrors Oyatie's catalog source
/// (`registry/catalog/*.yaml`), but adopters can point the same gate at their own catalog layout
/// without forking the producer or evaluator.
fn collect_slo_coverage(
    repo_root: &Path,
    tracked_paths: &[String],
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> Result<Value, CliError> {
    // The slo-coverage gate composes the live-OR-marked predicate (PR-C3): a row with an SLO is
    // not enough if the catalog record itself is silently stale. Resolve the live crate-id
    // universe IN-PROCESS (no shell-out) so each row carries is_live + marker alongside slo.
    let live = live_workspace_crate_ids(repo_root)?;
    let mut records: Vec<(String, String, Option<String>, bool, Option<String>)> = Vec::new();
    for path in tracked_paths {
        if is_path_excluded(path, cfg) {
            continue;
        }
        if !cfg
            .slo_coverage
            .catalog_record_globs
            .iter()
            .any(|glob| path_glob_matches(path, glob))
        {
            continue;
        }
        let Some(crate_id) = file_stem(path) else {
            continue;
        };
        let contents = read_text(&repo_root.join(path));
        let is_live = live.contains(&crate_id);
        let marker = catalog_non_live_marker(&contents);
        records.push((
            crate_id,
            path.clone(),
            parse_catalog_slo(&contents),
            is_live,
            marker,
        ));
    }
    records.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let rows: Vec<Value> = records
        .into_iter()
        .map(|(crate_id, source_path, slo, is_live, marker)| {
            json!({
                "crate_id": crate_id,
                "source_path": source_path,
                "slo": slo,
                "is_live": is_live,
                "marker": marker,
            })
        })
        .collect();
    Ok(json!({ "rows": rows }))
}

/// The explicit non-live `status:` markers the catalog-liveness gate accepts (the founder
/// live-OR-explicitly-marked policy). A record whose stem is NOT a live workspace crate-id passes
/// the gate ONLY if it carries one of these markers (or a `non_claims` no-crate declaration). These
/// are the verbatim markers PR-C1/PR-C2 used to retire moved/never-built catalog rows:
///   - `retired-compatibility-row-no-crate`  — a compatibility row whose crate was removed/moved;
///   - `designed-ahead-row-no-crate`         — a row designed ahead of its (not-yet-built) crate;
///   - `audit_doctrine_only`                 — a doctrine/audit-only row with no runtime crate;
///   - `planned` / `aspirational`            — forward-looking rows with no crate yet.
const NON_LIVE_STATUS_MARKERS: [&str; 5] = [
    "retired-compatibility-row-no-crate",
    "designed-ahead-row-no-crate",
    "audit_doctrine_only",
    "planned",
    "aspirational",
];

/// Parse the top-level `status:` scalar from a catalog record (the same shallow top-level YAML
/// scan as `parse_catalog_slo`). Returns the verbatim value (no marker classification here).
fn parse_catalog_status(contents: &str) -> Option<String> {
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        if key.trim() == "status" {
            let v = value.trim();
            if v.is_empty() {
                return None;
            }
            return Some(v.to_owned());
        }
    }
    None
}

/// Detect a `non_claims` block whose entries explicitly state no matching crate exists — the
/// non-`status:` marker shape PR-C1/PR-C2 also used (e.g. "no matching crate exists in this
/// checkout"). This keeps the gate from false-REDing legitimately-retired rows that declared the
/// no-crate fact in prose rather than via `status:`. Scoped to entries that name the crate's
/// absence so generic non_claims (e.g. "no measured SLO") never launder a stale row as marked.
fn catalog_non_claims_declares_no_crate(contents: &str) -> bool {
    let mut in_non_claims = false;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("non_claims:") {
            in_non_claims = true;
            continue;
        }
        if in_non_claims {
            // The block ends at the next top-level key (a non-indented, non-list line).
            let indented = line.starts_with(' ') || line.starts_with('-') || line.starts_with('\t');
            if !indented && !trimmed.is_empty() {
                break;
            }
            let lower = trimmed.to_ascii_lowercase();
            if (lower.contains("no matching crate") || lower.contains("no live crate"))
                && (lower.contains("exist")
                    || lower.contains("checkout")
                    || lower.contains("crate"))
            {
                return true;
            }
        }
    }
    false
}

/// The explicit-non-live marker for a catalog record, or `None` if it carries none. The producer
/// owns this classification (the single source of truth the gate reads): a non-live `status:`
/// value wins; otherwise a `non_claims` no-crate declaration yields the synthetic
/// `non-claims-no-crate` marker. A LIVE record needs no marker (the gate checks live OR marked).
fn catalog_non_live_marker(contents: &str) -> Option<String> {
    if let Some(status) = parse_catalog_status(contents) {
        if NON_LIVE_STATUS_MARKERS.contains(&status.as_str()) {
            return Some(status);
        }
    }
    if catalog_non_claims_declares_no_crate(contents) {
        return Some("non-claims-no-crate".to_owned());
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LiveWorkspaceCrate {
    crate_id: String,
    member_path: String,
}

/// The LIVE workspace crate universe: the `[package].name` and member directory of every resolved
/// workspace member. Scanned IN-PROCESS via `oya-workspace-members-kernel` (the same glob-aware
/// oracle the cohesion gate + the workspace-glob/target-parity faces use) + a shallow Cargo.toml
/// `[package].name` parse. Invalid matches remain blocking workspace-glob-coverage rows. NEVER a
/// `cargo metadata`/`buck2` shell-out
/// (all-CLI-retirement + hermeticity). The catalog crate_id (the file stem) is compared to package
/// names, since de-brand path-as-namespace means the crate identity is `[package].name`, not the
/// directory basename.
fn live_workspace_crates(repo_root: &Path) -> Result<Vec<LiveWorkspaceCrate>, CliError> {
    let member_dirs = scan_valid_member_dirs(repo_root, "catalog-liveness")?;
    let mut rows = Vec::new();
    for dir in member_dirs {
        let manifest = repo_root.join(&dir).join("Cargo.toml");
        if let Some(name) = parse_package_name(&read_text(&manifest)) {
            rows.push(LiveWorkspaceCrate {
                crate_id: name,
                member_path: dir,
            });
        }
    }
    rows.sort_by(|a, b| {
        a.crate_id
            .cmp(&b.crate_id)
            .then_with(|| a.member_path.cmp(&b.member_path))
    });
    Ok(rows)
}

fn live_workspace_crate_ids(repo_root: &Path) -> Result<BTreeSet<String>, CliError> {
    Ok(live_workspace_crates(repo_root)?
        .into_iter()
        .map(|row| row.crate_id)
        .collect())
}

/// Parse `traceability.source_crate` from a shallow catalog YAML row. This is deliberately scoped to
/// the established registry/catalog row shape: a top-level `traceability:` block with indented
/// `source_crate: <repo-relative Cargo.toml>`.
fn parse_catalog_source_crate(contents: &str) -> Option<String> {
    let mut in_traceability = false;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("traceability:") {
            in_traceability = true;
            continue;
        }
        if in_traceability {
            let indented = line.starts_with(' ') || line.starts_with('\t');
            if !indented && !trimmed.is_empty() {
                break;
            }
            let Some((key, value)) = trimmed.split_once(':') else {
                continue;
            };
            if key.trim() == "source_crate" {
                let value = value.trim().trim_matches('"').trim_matches('\'');
                if value.is_empty() {
                    return None;
                }
                return Some(value.to_owned());
            }
        }
    }
    None
}

fn clean_repo_relative_path(path: &str) -> &str {
    path.trim()
        .strip_prefix("./")
        .unwrap_or(path.trim())
        .strip_prefix('/')
        .unwrap_or_else(|| path.trim().strip_prefix("./").unwrap_or(path.trim()))
}

fn repo_path_is_tracked_file(repo_root: &Path, tracked_paths: &BTreeSet<&str>, path: &str) -> bool {
    let rel = clean_repo_relative_path(path);
    tracked_paths.contains(rel) && repo_root.join(rel).is_file()
}

fn catalog_exemption_for_member(
    member_path: &str,
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> Option<Value> {
    cfg.catalog_liveness
        .workspace_member_exemptions
        .iter()
        .find(|exemption| path_glob_matches(member_path, &exemption.path_glob))
        .map(|exemption| {
            json!({
                "path_glob": &exemption.path_glob,
                "owner": &exemption.owner,
                "reason": &exemption.reason,
                "cutover": &exemption.cutover,
            })
        })
}

/// Enumerate catalog-liveness rows from the config-declared `[catalog_liveness]` policy. The face
/// is bidirectional:
///   - `rows`: catalog record -> live/marked/source-path facts;
///   - `live_crates`: governed live workspace member -> catalog row/exemption facts.
fn collect_catalog_liveness(
    repo_root: &Path,
    tracked_paths: &[String],
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> Result<Value, CliError> {
    let live = live_workspace_crates(repo_root)?;
    let live_ids: BTreeSet<String> = live.iter().map(|row| row.crate_id.clone()).collect();
    let tracked: BTreeSet<&str> = tracked_paths.iter().map(String::as_str).collect();
    let mut records: Vec<(String, String, bool, Option<String>, Option<String>, bool)> = Vec::new();
    for path in tracked_paths {
        if is_path_excluded(path, cfg) {
            continue;
        }
        if !cfg
            .catalog_liveness
            .catalog_record_globs
            .iter()
            .any(|glob| path_glob_matches(path, glob))
        {
            continue;
        }
        let Some(crate_id) = file_stem(path) else {
            continue;
        };
        let contents = read_text(&repo_root.join(path));
        let is_live = live_ids.contains(&crate_id);
        let marker = catalog_non_live_marker(&contents);
        let source_crate = parse_catalog_source_crate(&contents);
        let source_crate_exists = source_crate
            .as_deref()
            .is_some_and(|source| repo_path_is_tracked_file(repo_root, &tracked, source));
        records.push((
            crate_id,
            path.clone(),
            is_live,
            marker,
            source_crate,
            source_crate_exists,
        ));
    }
    records.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    let catalog_ids: BTreeSet<String> = records.iter().map(|record| record.0.clone()).collect();

    let rows: Vec<Value> = records
        .into_iter()
        .map(
            |(crate_id, source_path, is_live, marker, source_crate, source_crate_exists)| {
                json!({
                    "crate_id": crate_id,
                    "source_path": source_path,
                    "is_live": is_live,
                    "marker": marker,
                    "source_crate": source_crate,
                    "source_crate_exists": source_crate_exists,
                })
            },
        )
        .collect();

    let mut live_rows = Vec::new();
    for row in live {
        let governed = cfg
            .catalog_liveness
            .workspace_member_globs
            .iter()
            .any(|glob| path_glob_matches(&row.member_path, glob));
        if !governed {
            continue;
        }
        let has_catalog_row = catalog_ids.contains(&row.crate_id);
        let exemption = catalog_exemption_for_member(&row.member_path, cfg);
        live_rows.push(json!({
            "crate_id": row.crate_id,
            "member_path": row.member_path,
            "has_catalog_row": has_catalog_row,
            "exemption": exemption,
        }));
    }

    Ok(json!({ "rows": rows, "live_crates": live_rows }))
}

/// Enumerate ADR-0538 workspace-glob-coverage rows. Member-entry rows preserve the raw root
/// `[workspace].members` entries; member-match rows expose unexcluded concrete matches without a
/// manifest; crate-dir rows cover tracked first-party package manifests that are not the root
/// manifest, not repo-policy-excluded, and not inside nested workspaces. Expansion and coverage
/// come only from `oya-workspace-members-kernel`.
fn collect_workspace_glob_coverage(
    repo_root: &Path,
    tracked_paths: &[String],
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> Result<Value, CliError> {
    let entries = oya_workspace_members_kernel::read_workspace_manifest_entries(repo_root)
        .map_err(|error| {
            CliError::Io(format!(
                "workspace-glob-coverage read root workspace entries: {error}"
            ))
        })?;
    let member_scan =
        oya_workspace_members_kernel::scan_member_dirs(repo_root).map_err(|error| {
            CliError::Io(format!("workspace-glob-coverage scan member dirs: {error}"))
        })?;
    let covered_dirs: BTreeSet<String> = member_scan.member_dirs.into_iter().collect();

    let mut rows: Vec<Value> = entries
        .members
        .iter()
        .map(|member_entry| {
            json!({
                "member_entry": member_entry,
                "is_glob": member_entry.contains('*'),
            })
        })
        .collect();
    rows.extend(
        member_scan
            .missing_manifests
            .into_iter()
            .map(|member_match| {
                json!({
                    "member_match": member_match,
                    "has_manifest": false,
                })
            }),
    );

    let mut crate_dirs: BTreeMap<String, (bool, bool)> = BTreeMap::new();
    for path in tracked_paths {
        if path == "Cargo.toml" || !path.ends_with("/Cargo.toml") {
            continue;
        }
        if is_path_excluded(path, cfg) {
            continue;
        }
        let Some(crate_dir) = path.strip_suffix("/Cargo.toml") else {
            continue;
        };
        let contents = read_text(&repo_root.join(path));
        if parse_package_name(&contents).is_none() {
            continue;
        }
        if is_nested_workspace_package(repo_root, crate_dir, &contents) {
            continue;
        }
        let excluded = workspace_entry_excludes_dir(crate_dir, &entries.exclude);
        let covered = covered_dirs.contains(crate_dir);
        crate_dirs.insert(crate_dir.to_owned(), (covered, excluded));
    }

    rows.extend(
        crate_dirs
            .into_iter()
            .map(|(crate_dir, (covered, excluded))| {
                json!({
                    "crate_dir": crate_dir,
                    "covered": covered,
                    "excluded": excluded,
                })
            }),
    );

    Ok(json!({ "rows": rows }))
}

/// Enumerate ADR-0540 target-parity rows. Member enumeration comes from the canonical
/// glob-aware workspace resolver, then narrows to members present in the declared SCM tracked
/// paths face. The booleans are computed only from tracked files: BUCK presence, textual
/// `rust_test` rule presence, and Rust test-code markers in `tests/` or `src/**/*.rs`.
fn collect_target_parity(
    repo_root: &Path,
    tracked_paths: &[String],
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> Result<Value, CliError> {
    let tracked: BTreeSet<&str> = tracked_paths
        .iter()
        .filter(|path| !is_path_excluded(path, cfg))
        .map(String::as_str)
        .collect();
    let member_dirs: BTreeSet<String> = scan_valid_member_dirs(repo_root, "target-parity")?
        .into_iter()
        .filter(|member| tracked.contains(format!("{member}/Cargo.toml").as_str()))
        .collect();

    let mut rows: Vec<Value> = Vec::with_capacity(member_dirs.len());
    for member_path in member_dirs {
        let buck_path = format!("{member_path}/BUCK");
        let has_buck = tracked.contains(buck_path.as_str());
        let has_rust_test_target =
            has_buck && read_text(&repo_root.join(&buck_path)).contains("rust_test(");
        let has_test_code = member_has_test_code(repo_root, &member_path, &tracked);
        rows.push(json!({
            "member_path": member_path,
            "has_buck": has_buck,
            "has_rust_test_target": has_rust_test_target,
            "has_test_code": has_test_code,
        }));
    }

    Ok(json!({ "rows": rows }))
}

const CLAUDE_WIRING_FILE: &str = ".claude/settings.json";
const CODEX_WIRING_FILE: &str = ".codex/hooks.json";
const HOOKS_DIR: &str = "tools/hooks";
const COMPATIBILITY_STUB_MARKER: &str = "Compatibility stub only";

#[derive(Debug)]
struct EnforcementLivenessCorpus {
    texts: BTreeMap<String, String>,
}

impl EnforcementLivenessCorpus {
    fn from_args(
        tracked_paths: &[String],
        claude_settings: Option<&Path>,
        codex_hooks: Option<&Path>,
        hooks_dir: Option<&Path>,
    ) -> Result<Self, CliError> {
        match (claude_settings, codex_hooks, hooks_dir) {
            (Some(claude_settings), Some(codex_hooks), Some(hooks_dir)) => {
                Self::from_declared_paths(tracked_paths, claude_settings, codex_hooks, hooks_dir)
            }
            _ => Err(CliError::Io(
                "--enforcement-liveness-claude-settings, \
                 --enforcement-liveness-codex-hooks, and \
                 --enforcement-liveness-hooks-dir are required when producing \
                 enforcement-liveness or baseline faces"
                    .to_owned(),
            )),
        }
    }

    fn from_declared_paths(
        tracked_paths: &[String],
        claude_settings: &Path,
        codex_hooks: &Path,
        hooks_dir: &Path,
    ) -> Result<Self, CliError> {
        if !hooks_dir.is_dir() {
            return Err(CliError::Io(format!(
                "{}: declared enforcement-liveness hooks corpus is not a directory",
                hooks_dir.display()
            )));
        }

        let mut texts = BTreeMap::new();
        texts.insert(
            CLAUDE_WIRING_FILE.to_owned(),
            read_required_text(claude_settings, CLAUDE_WIRING_FILE)?,
        );
        texts.insert(
            CODEX_WIRING_FILE.to_owned(),
            read_required_text(codex_hooks, CODEX_WIRING_FILE)?,
        );
        for hook_path in tracked_paths
            .iter()
            .filter(|path| is_top_level_hook_script(path))
        {
            let Some(file_name) = hook_file_name(hook_path) else {
                continue;
            };
            texts.insert(
                hook_path.clone(),
                read_required_text(
                    &hooks_dir.join(file_name),
                    &format!("declared enforcement-liveness input {hook_path}"),
                )?,
            );
        }
        Ok(Self { texts })
    }

    fn text(&self, logical_path: &str) -> Result<&str, CliError> {
        self.texts
            .get(logical_path)
            .map(String::as_str)
            .ok_or_else(|| {
                CliError::Io(format!(
                    "declared enforcement-liveness corpus missing logical path {logical_path}"
                ))
            })
    }
}

fn collect_enforcement_liveness(
    tracked_paths: &[String],
    corpus: &EnforcementLivenessCorpus,
) -> Result<Value, CliError> {
    let tracked: BTreeSet<&str> = tracked_paths.iter().map(String::as_str).collect();
    let claude_refs =
        collect_hook_command_refs(CLAUDE_WIRING_FILE, corpus.text(CLAUDE_WIRING_FILE)?)?;
    let codex_refs = collect_hook_command_refs(CODEX_WIRING_FILE, corpus.text(CODEX_WIRING_FILE)?)?;

    let mut rows: Vec<Value> = Vec::new();
    for hook_path in tracked_paths
        .iter()
        .filter(|path| is_top_level_hook_script(path))
    {
        let body = corpus.text(hook_path)?;
        rows.push(json!({
            "row_type": "hook",
            "hook_path": hook_path,
            "wired_in_claude": claude_refs.contains(hook_path),
            "wired_in_codex": codex_refs.contains(hook_path),
            "stub_marked": body.contains(COMPATIBILITY_STUB_MARKER),
        }));
    }

    for (wiring_file, refs) in [
        (CLAUDE_WIRING_FILE, claude_refs),
        (CODEX_WIRING_FILE, codex_refs),
    ] {
        for command_path in refs {
            rows.push(json!({
                "row_type": "command_reference",
                "wiring_file": wiring_file,
                "command_path": command_path,
                "target_exists": tracked.contains(command_path.as_str()),
            }));
        }
    }

    Ok(json!({ "rows": rows }))
}

fn is_top_level_hook_script(path: &str) -> bool {
    let Some(name) = path.strip_prefix("tools/hooks/") else {
        return false;
    };
    !name.contains('/') && name.ends_with(".sh")
}

fn hook_file_name(hook_path: &str) -> Option<&str> {
    hook_path.strip_prefix(&format!("{HOOKS_DIR}/"))
}

fn collect_hook_command_refs(wiring_file: &str, text: &str) -> Result<BTreeSet<String>, CliError> {
    let value = serde_json::from_str::<Value>(text)
        .map_err(|e| CliError::Io(format!("{wiring_file}: parse hook wiring JSON: {e}")))?;
    let mut refs = BTreeSet::new();
    collect_command_values(&value, &mut refs);
    Ok(refs)
}

fn collect_command_values(value: &Value, refs: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, nested) in map {
                if key == "command"
                    && let Some(command) = nested.as_str()
                    && let Some(path) = normalize_hook_command(command)
                {
                    refs.insert(path);
                }
                collect_command_values(nested, refs);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_command_values(item, refs);
            }
        }
        _ => {}
    }
}

fn normalize_hook_command(command: &str) -> Option<String> {
    let first = command.trim().split_whitespace().next()?;
    let path = first.strip_prefix("./").unwrap_or(first);
    if is_top_level_hook_script(path) {
        Some(path.to_owned())
    } else {
        None
    }
}

fn member_has_test_code(repo_root: &Path, member_path: &str, tracked: &BTreeSet<&str>) -> bool {
    let tests_prefix = format!("{member_path}/tests/");
    if tracked.iter().any(|path| path.starts_with(&tests_prefix)) {
        return true;
    }

    let src_prefix = format!("{member_path}/src/");
    tracked
        .iter()
        .filter(|path| path.starts_with(&src_prefix) && path.ends_with(".rs"))
        .any(|path| {
            let body = read_text(&repo_root.join(path));
            body.contains("#[cfg(test)]") || body.contains("#[test]")
        })
}

fn workspace_entry_excludes_dir(dir: &str, exclude: &[String]) -> bool {
    exclude
        .iter()
        .any(|entry| dir == entry || dir.starts_with(&format!("{entry}/")))
}

fn is_nested_workspace_package(repo_root: &Path, crate_dir: &str, contents: &str) -> bool {
    if has_workspace_table(contents) {
        return true;
    }
    let mut current = Path::new(crate_dir).parent();
    while let Some(parent) = current {
        if parent.as_os_str().is_empty() {
            break;
        }
        let manifest = repo_root.join(parent).join("Cargo.toml");
        if manifest.is_file() && has_workspace_table(&read_text(&manifest)) {
            return true;
        }
        current = parent.parent();
    }
    false
}

fn has_workspace_table(contents: &str) -> bool {
    contents.lines().any(|line| {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix('[') else {
            return false;
        };
        let table = rest.split(']').next().unwrap_or("").trim();
        table == "workspace" || table.starts_with("workspace.")
    })
}

/// Minimal path-glob matcher for declared gate input contracts. Supports all-path (`**`), exact
/// paths, directory-prefix (`dir/**` and `dir/`), recursive extension (`**/*.yaml`), basename
/// extension (`*.yaml`), and one-level directory extension (`dir/*.yaml`). Unknown shapes fail closed.
fn path_glob_matches(path: &str, glob: &str) -> bool {
    let path = path.strip_prefix("./").unwrap_or(path);
    let glob = glob.strip_prefix("./").unwrap_or(glob);
    if glob == "**" {
        return true;
    }
    if path == glob {
        return true;
    }
    if let Some(prefix) = glob.strip_suffix("/**") {
        return path == prefix || path.starts_with(&format!("{prefix}/"));
    }
    if glob.ends_with('/') {
        return path.starts_with(glob);
    }
    if let Some(ext) = glob.strip_prefix("**/*.") {
        return path.ends_with(&format!(".{ext}"));
    }
    if let Some(ext) = glob.strip_prefix("*.") {
        return !path.contains('/') && path.ends_with(&format!(".{ext}"));
    }
    if let Some((dir, pattern)) = glob.rsplit_once("/*.")
        && !dir.is_empty()
    {
        let prefix = format!("{dir}/");
        if let Some(rest) = path.strip_prefix(&prefix) {
            return !rest.contains('/') && rest.ends_with(&format!(".{pattern}"));
        }
    }
    false
}

fn file_stem(path: &str) -> Option<String> {
    let name = path.rsplit('/').next().unwrap_or(path);
    let (stem, _) = name.rsplit_once('.')?;
    if stem.is_empty() {
        None
    } else {
        Some(stem.to_owned())
    }
}

fn parse_catalog_slo(contents: &str) -> Option<String> {
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        if key.trim() == "slo" {
            return Some(value.trim().to_owned());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEMP_REPO_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_temp_repo() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let counter = TEMP_REPO_COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "oya-accounting-registry-test-{}-{nanos}-{counter}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create temp repo");
        root
    }

    fn write_test_file(root: &Path, rel: &str, body: &str) -> PathBuf {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create test file parent");
        }
        fs::write(&path, body).expect("write test file");
        path
    }

    fn retired_coordination_brand_bytes() -> Vec<u8> {
        vec![104, 101, 114, 109, 101, 115]
    }

    #[test]
    fn brand_residue_scan_runs_only_when_baseline_materialization_needs_it() {
        assert!(should_collect_brand_residue(false, "registry"));
        assert!(should_collect_brand_residue(true, "baseline"));
        assert!(!should_collect_brand_residue(true, "registry"));
        assert!(!should_collect_brand_residue(true, "enforcement-inventory"));
    }

    #[test]
    fn strict_zero_collector_scans_every_tracked_path_class_and_binary_blob() {
        let root = unique_temp_repo();
        let needle = retired_coordination_brand_bytes();
        let upper: Vec<u8> = needle.iter().map(u8::to_ascii_uppercase).collect();
        let tracked_paths = vec![
            "docs/decisions/ADR-9999.md".to_owned(),
            "evidence/audit-chain.jsonl".to_owned(),
            "_archive/retired.md".to_owned(),
            "ci/facade/example.generated.json".to_owned(),
            "assets/blob.bin".to_owned(),
        ];
        for path in &tracked_paths[..4] {
            let full = root.join(path);
            fs::create_dir_all(full.parent().expect("fixture parent")).expect("create parent");
            fs::write(full, &needle).expect("write fixture");
        }
        let binary = root.join(&tracked_paths[4]);
        fs::create_dir_all(binary.parent().expect("binary parent")).expect("create parent");
        fs::write(binary, [&[0, 255][..], upper.as_slice()].concat()).expect("write binary");

        let cfg =
            oya_ci_config_kernel::OyaCiConfig::from_toml_str("").expect("default config parses");
        let grouped = collect_brand_residue(&root, &tracked_paths, &cfg)
            .expect("strict-zero collection succeeds");
        let keys =
            &grouped[oya_check_brand_residue::forbidden_vocab::STRICT_ZERO_RETIRED_BRAND_CODE];
        assert_eq!(keys, &tracked_paths.into_iter().collect());
        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[cfg(unix)]
    #[test]
    fn strict_zero_collector_scans_symlink_payload_without_following_target() {
        use std::os::unix::ffi::OsStringExt;
        use std::os::unix::fs::symlink;

        let root = unique_temp_repo();
        let link = root.join("links/retired-target");
        fs::create_dir_all(link.parent().expect("link parent")).expect("create parent");
        let target = std::ffi::OsString::from_vec(
            [
                b"../outside-".as_slice(),
                retired_coordination_brand_bytes().as_slice(),
            ]
            .concat(),
        );
        symlink(target, &link).expect("create symlink fixture");
        let tracked_paths = vec!["links/retired-target".to_owned()];
        let cfg =
            oya_ci_config_kernel::OyaCiConfig::from_toml_str("").expect("default config parses");

        let grouped = collect_brand_residue(&root, &tracked_paths, &cfg)
            .expect("dangling tracked symlink payload is readable");
        assert!(
            grouped[oya_check_brand_residue::forbidden_vocab::STRICT_ZERO_RETIRED_BRAND_CODE]
                .contains("links/retired-target")
        );
        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn strict_zero_collector_fails_closed_on_missing_tracked_blob() {
        let root = unique_temp_repo();
        let cfg =
            oya_ci_config_kernel::OyaCiConfig::from_toml_str("").expect("default config parses");
        let error = collect_brand_residue(&root, &["missing.bin".to_owned()], &cfg)
            .expect_err("missing tracked blob must fail closed");
        assert!(error.to_string().contains("missing.bin"));
        fs::remove_dir_all(root).expect("remove temp repo");
    }

    /// The decoded (canonical) spelling of a non-ASCII pathname resolves to the file's exact
    /// bytes and is the census key. Post-ingestion-decode this collector never sees a quoted
    /// spelling, so it must not decode again — see [`read_tracked_blob`].
    #[test]
    fn strict_zero_collector_reads_a_non_ascii_tracked_path_by_its_decoded_spelling() {
        let root = unique_temp_repo();
        let tracked_key = decode_tracked_path(
            r#""oya/developer-sdk/decisions/ADR-SDK-0003-tenancy-\302\265service.md""#,
        )
        .expect("quoted path decodes");
        assert_eq!(
            tracked_key,
            "oya/developer-sdk/decisions/ADR-SDK-0003-tenancy-\u{b5}service.md"
        );
        let full = root.join(&tracked_key);
        fs::create_dir_all(full.parent().expect("fixture parent")).expect("create parent");
        fs::write(&full, retired_coordination_brand_bytes()).expect("write quoted-path fixture");
        let cfg =
            oya_ci_config_kernel::OyaCiConfig::from_toml_str("").expect("default config parses");

        let grouped = collect_brand_residue(&root, std::slice::from_ref(&tracked_key), &cfg)
            .expect("decoded tracked pathname resolves to its exact bytes");
        assert!(
            grouped[oya_check_brand_residue::forbidden_vocab::STRICT_ZERO_RETIRED_BRAND_CODE]
                .contains(&tracked_key)
        );
        fs::remove_dir_all(root).expect("remove temp repo");
    }

    /// THE regression test for the ingestion boundary, driven from the FACE the producer
    /// actually reads: a Git C-quoted `tracked_paths` entry must arrive as its DECODED
    /// spelling, and therefore resolve the SAME owner as its unquoted sibling. Before the
    /// decode, the row keyed on `"marketplace/…`, so the ownership walk climbed ancestors of
    /// `"marketplace` and resolved `null` while every sibling resolved the directory OWNERS.
    /// Reverting the decode in `load_scm_facts` fails this test on the first assertion.
    #[test]
    fn scm_facts_ingestion_decodes_quoted_paths_so_they_resolve_the_sibling_owner() {
        let root = unique_temp_repo();
        let dir = "marketplace/developer-sdk/decisions";
        write_test_file(&root, &format!("{dir}/OWNERS"), "axis-ecosystem\n");
        let sibling = format!("{dir}/ADR-SDK-0001-plain.md");
        write_test_file(&root, &sibling, "# plain\n");
        let decoded = format!("{dir}/ADR-SDK-0003-\u{b5}service.md");
        write_test_file(&root, &decoded, "# quoted-name sibling\n");

        let face = write_test_file(
            &root,
            "scm-facts.generated.json",
            r#"{"schema":"oya-ci/scm-facts/v2","tracked_paths":[
                "marketplace/developer-sdk/decisions/OWNERS",
                "marketplace/developer-sdk/decisions/ADR-SDK-0001-plain.md",
                "\"marketplace/developer-sdk/decisions/ADR-SDK-0003-\\302\\265service.md\""
            ]}"#,
        );
        let tracked = load_scm_facts(&face)
            .expect("scm-facts face loads")
            .tracked_paths;

        assert!(
            tracked.contains(&decoded),
            "ingestion must yield the DECODED spelling, got: {tracked:?}"
        );
        assert!(
            !tracked.iter().any(|p| p.starts_with('"')),
            "no quoted spelling may survive ingestion: {tracked:?}"
        );

        let cfg =
            oya_ci_config_kernel::OyaCiConfig::from_toml_str("").expect("default config parses");
        let owners = resolve_owners(&root, &tracked, &cfg).by_path;
        assert_eq!(
            owners.get(&sibling).map(String::as_str),
            Some(format!("OWNERS:{dir}").as_str()),
            "the unquoted sibling must be owned (fixture sanity)"
        );
        assert_eq!(
            owners.get(&decoded),
            owners.get(&sibling),
            "a Git-quoted path must resolve the same OWNERS as its unquoted sibling"
        );
        fs::remove_dir_all(root).expect("remove temp repo");
    }

    /// Non-UTF-8 path bytes fail CLOSED rather than lossily renaming the key: a lossy key
    /// looks right and mis-keys ownership/justification/reachability all over again.
    #[test]
    fn a_non_utf8_tracked_path_fails_closed() {
        let error = decode_tracked_path(r#""bad-\377-name.md""#)
            .expect_err("non-UTF-8 path bytes must fail closed");
        assert!(
            error.to_string().contains("non-UTF-8"),
            "unexpected error: {error}"
        );
    }

    fn load_live_test_scm_facts(root: &Path) -> ScmFacts {
        let face = root.join("ci/facade/artifact-inventory-registry/scm-facts.generated.json");
        // Same class as the repo-root-hygiene / generated-artifact-policy gates: this face is the
        // ADR-0604 de-commit class, so it is absent in ANY clean worktree and this live-corpus
        // test cannot run. "run the producer-regen/materialization boundary" named no command,
        // which left an author with a red gate and nothing to do about it.
        assert!(
            face.is_file(),
            "missing materialized scm-facts face at {}.\n\nIt is generated (ADR-0604 de-commit \
             class), not tracked in git. Materialize it, then re-run:\n\n    buck2 run \
             //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin \
             -- --repo-root .\n",
            face.display()
        );
        load_scm_facts(&face).expect("materialized scm-facts face loads")
    }

    #[test]
    fn vocab_policy_mapping_preserves_line_exception_stem_scope() {
        let cfg = oya_ci_config_kernel::OyaCiConfig::from_toml_str(
            r#"
[[vocab.carve_outs]]
kind = "line_contains_ci"
value = "structural-marker"
exempt_stems = ["alpha"]
"#,
        )
        .expect("scoped vocab config parses");

        let policy = vocab_policy(&cfg.vocab);
        let rule = policy
            .carve_outs
            .iter()
            .find(|rule| rule.value == "structural-marker")
            .expect("mapped line rule");
        assert_eq!(rule.exempt_stems, vec!["alpha"]);
    }

    #[test]
    fn frozen_reference_loader_migrates_v1_line_scope_without_weakening_candidate_loader() {
        let root = unique_temp_repo();
        write_test_file(
            &root,
            "oya-ci.toml",
            r#"
[[vocab.forbidden_stems]]
stem = "alpha"
code = "forbidden_alpha"

[[vocab.forbidden_stems]]
stem = "beta"
code = "forbidden_beta"

[[vocab.carve_outs]]
kind = "line_contains_ci"
value = "legacy-marker"
"#,
        );

        assert!(
            load_config(&root).is_err(),
            "candidate policy must reject a schema v1 line rule"
        );
        let frozen = load_frozen_reference_config(&root)
            .expect("frozen-reference loader migrates schema v1 line scope");
        assert_eq!(
            frozen.vocab.carve_outs[0].exempt_stems,
            vec!["alpha", "beta"]
        );
        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn scm_facts_rejects_missing_schema() {
        let root = unique_temp_repo();
        let face = write_test_file(
            &root,
            "scm-facts.generated.json",
            r#"{"tracked_paths":["README.md"]}"#,
        );

        let Err(err) = load_scm_facts(&face) else {
            panic!("schema-less scm-facts face should be rejected");
        };

        assert!(
            err.to_string().contains("missing schema"),
            "error should identify missing schema, got: {err}"
        );
    }

    #[test]
    fn scm_facts_rejects_legacy_git_facts_schema() {
        let root = unique_temp_repo();
        let face = write_test_file(
            &root,
            "scm-facts.generated.json",
            r#"{"schema":"oya-ci/git-facts/v1","tracked_paths":["README.md"]}"#,
        );

        let Err(err) = load_scm_facts(&face) else {
            panic!("legacy git-facts face should be rejected");
        };

        assert!(
            err.to_string().contains("unsupported scm-facts schema"),
            "error should identify unsupported schema, got: {err}"
        );
    }

    #[test]
    fn scm_facts_rejects_non_string_tracked_paths_entries() {
        let root = unique_temp_repo();
        let face = write_test_file(
            &root,
            "scm-facts.generated.json",
            r#"{"schema":"oya-ci/scm-facts/v2","tracked_paths":["README.md",42,"docs/AGENTS.md"]}"#,
        );

        let Err(err) = load_scm_facts(&face) else {
            panic!("non-string tracked_paths entry should be rejected");
        };

        assert!(
            err.to_string().contains("tracked_paths[1]"),
            "error should identify malformed entry index, got: {err}"
        );
    }

    #[test]
    fn enforcement_liveness_declared_corpus_prevents_stale_repo_root_false_green() {
        let root = unique_temp_repo();
        let tracked_paths = vec![
            CLAUDE_WIRING_FILE.to_owned(),
            CODEX_WIRING_FILE.to_owned(),
            "tools/hooks/hermetic-check.sh".to_owned(),
        ];

        write_test_file(
            &root,
            CLAUDE_WIRING_FILE,
            r#"{"hooks":{"PreToolUse":[{"hooks":[{"command":"./tools/hooks/hermetic-check.sh"}]}]}}"#,
        );
        write_test_file(
            &root,
            CODEX_WIRING_FILE,
            r#"{"hooks":{"UserPromptSubmit":[{"command":"./tools/hooks/hermetic-check.sh"}]}}"#,
        );
        write_test_file(
            &root,
            "tools/hooks/hermetic-check.sh",
            "#!/usr/bin/env bash\n# Compatibility stub only\n",
        );

        let declared_root = root.join("buck-declared-corpus");
        let declared_claude = write_test_file(&declared_root, "settings.json", r#"{"hooks":{}}"#);
        let declared_codex = write_test_file(&declared_root, "hooks.json", r#"{"hooks":{}}"#);
        let declared_hooks_dir = declared_root.join("hooks");
        write_test_file(
            &declared_hooks_dir,
            "hermetic-check.sh",
            "#!/usr/bin/env bash\necho live hook\n",
        );

        let declared_corpus = EnforcementLivenessCorpus::from_declared_paths(
            &tracked_paths,
            &declared_claude,
            &declared_codex,
            &declared_hooks_dir,
        )
        .unwrap();
        let declared_face = collect_enforcement_liveness(&tracked_paths, &declared_corpus).unwrap();
        let hook_row = declared_face["rows"]
            .as_array()
            .expect("rows")
            .iter()
            .find(|row| row["hook_path"] == "tools/hooks/hermetic-check.sh")
            .expect("hermetic hook row");
        assert_eq!(hook_row["wired_in_claude"].as_bool(), Some(false));
        assert_eq!(hook_row["wired_in_codex"].as_bool(), Some(false));
        assert_eq!(hook_row["stub_marked"].as_bool(), Some(false));
        assert_eq!(
            ci_hook_wiring::evaluate(&declared_face).verdict,
            ci_hook_wiring::Verdict::Red,
            "declared Buck corpus, not stale repo-root content, must drive the face"
        );

        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn enforcement_liveness_rejects_missing_declared_corpus_args() {
        let tracked_paths = vec![
            CLAUDE_WIRING_FILE.to_owned(),
            CODEX_WIRING_FILE.to_owned(),
            "tools/hooks/hermetic-check.sh".to_owned(),
        ];

        let error = EnforcementLivenessCorpus::from_args(&tracked_paths, None, None, None)
            .unwrap_err()
            .to_string();

        assert!(
            error.contains("--enforcement-liveness-claude-settings"),
            "missing declared corpus error must name the claude-settings arg: {error}"
        );
        assert!(
            error.contains("--enforcement-liveness-codex-hooks"),
            "missing declared corpus error must name the codex-hooks arg: {error}"
        );
        assert!(
            error.contains("--enforcement-liveness-hooks-dir"),
            "missing declared corpus error must name the hooks-dir arg: {error}"
        );
        assert!(
            error.contains("required"),
            "missing declared corpus error must fail closed, not fall back: {error}"
        );
    }

    #[test]
    fn enforcement_liveness_declared_corpus_fails_on_missing_tracked_hook_input() {
        let root = unique_temp_repo();
        let tracked_paths = vec![
            CLAUDE_WIRING_FILE.to_owned(),
            CODEX_WIRING_FILE.to_owned(),
            "tools/hooks/missing-from-declared-corpus.sh".to_owned(),
        ];
        let declared_root = root.join("buck-declared-corpus");
        let declared_claude = write_test_file(&declared_root, "settings.json", r#"{"hooks":{}}"#);
        let declared_codex = write_test_file(&declared_root, "hooks.json", r#"{"hooks":{}}"#);
        let declared_hooks_dir = declared_root.join("hooks");
        fs::create_dir_all(&declared_hooks_dir).expect("create declared hooks dir");

        let error = EnforcementLivenessCorpus::from_declared_paths(
            &tracked_paths,
            &declared_claude,
            &declared_codex,
            &declared_hooks_dir,
        )
        .unwrap_err()
        .to_string();

        assert!(
            error.contains("tools/hooks/missing-from-declared-corpus.sh"),
            "missing declared hook path must be named in the error: {error}"
        );
        assert!(
            error.contains("declared enforcement-liveness input"),
            "missing input must fail as a declared corpus error: {error}"
        );

        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn justifications_preserve_leading_dot_paths() {
        let root = unique_temp_repo();
        let decisions = root.join("docs/decisions");
        fs::create_dir_all(&decisions).expect("create decisions dir");
        fs::write(
            decisions.join("ADR-9999-dot-path-test.md"),
            "The tracked bridge surface is `.omc/ultragoal/TEAMMATE-PREAMBLE.md.`\n",
        )
        .expect("write ADR");

        let cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();
        let paths = vec![".omc/ultragoal/TEAMMATE-PREAMBLE.md".to_owned()];
        let justifications = resolve_justifications(&root, &paths, &cfg);

        assert_eq!(
            justifications.get(".omc/ultragoal/TEAMMATE-PREAMBLE.md"),
            Some(&"ADR-9999".to_owned())
        );

        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn justifications_include_root_level_tracked_paths() {
        let root = unique_temp_repo();
        let decisions = root.join("docs/decisions");
        fs::create_dir_all(&decisions).expect("create decisions dir");
        fs::write(
            decisions.join("ADR-9998-root-path-test.md"),
            "The root dependency automation DATA contract is `oya-deps.toml`.\n",
        )
        .expect("write ADR");

        let cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();
        let paths = vec!["oya-deps.toml".to_owned()];
        let justifications = resolve_justifications(&root, &paths, &cfg);

        assert_eq!(
            justifications.get("oya-deps.toml"),
            Some(&"ADR-9998".to_owned())
        );

        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn collect_repo_inputs_excludes_configured_third_party_scm_path() {
        let root = unique_temp_repo();
        let cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();
        let excluded_path = "third-party/vendored/lib.rs".to_owned();

        assert!(
            is_path_excluded(&excluded_path, &cfg),
            "fixture path must be covered by the configured exclusion"
        );
        let (inputs, _) = collect_repo_inputs(
            &root,
            &cfg,
            &ScmFacts {
                tracked_paths: vec![excluded_path.clone()],
            },
        )
        .expect("collect repo inputs");

        assert!(
            !inputs.tracked_paths.contains(&excluded_path),
            "configured excluded SCM path must never enter RepoInputs"
        );
        fs::remove_dir_all(root).expect("remove temp repo");
    }

    /// FRIC #1328 — the verdict a pre-push author-side check-mode invocation would print for
    /// `path`, computed via `check_added_paths`. `find` panics if the path is absent.
    fn check_verdict(root: &Path, path: &str) -> AddedPathVerdict {
        let cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();
        let policy = Policy::from_config(&cfg).expect("policy from bundled default");
        let paths = vec![path.to_owned()];
        let mut verdicts =
            check_added_paths(root, &cfg, &policy, &paths).expect("check added paths");
        assert_eq!(verdicts.len(), 1, "one verdict per input path");
        verdicts.pop().expect("verdict present")
    }

    #[test]
    fn check_mode_added_unjustified_path_reports_code_and_remediation() {
        // A newly ADDED code file that no ADR names ⇒ the firewall's `unjustified` code, keyed
        // by the path — exactly the `[cloud-ci-total-accounting] unjustified regressions` class.
        let root = unique_temp_repo();
        fs::create_dir_all(root.join("docs/decisions")).expect("create decisions dir");

        let path = "newsvc/src/lib.rs";
        let verdict = check_verdict(&root, path);

        assert_eq!(verdict.unit_class, "code");
        assert!(verdict.justification.is_none(), "no ADR justifies it");
        assert!(
            verdict.blocking_codes.contains("unjustified"),
            "unjustified must be reported, got {:?}",
            verdict.blocking_codes
        );

        // The remediation names the EXACT path token + the ADR-0515 precedent.
        let remediation = unjustified_remediation(path);
        assert!(
            remediation.contains(&format!("`{path}`")),
            "names path token"
        );
        assert!(remediation.contains("ADR-0515"), "names ci/ gate precedent");
        assert!(remediation.contains("docs/decisions/"), "names the corpus");

        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn check_mode_added_justified_path_reports_clean_of_unjustified() {
        // The SAME path, once an ADR names its exact token, is no longer `unjustified` — the fix
        // the remediation prescribes actually clears the code.
        let root = unique_temp_repo();
        let decisions = root.join("docs/decisions");
        fs::create_dir_all(&decisions).expect("create decisions dir");

        let path = "newsvc/src/lib.rs";
        fs::write(
            decisions.join("ADR-9997-newsvc.md"),
            format!("The new service entrypoint is `{path}`.\n"),
        )
        .expect("write ADR");

        let verdict = check_verdict(&root, path);

        assert_eq!(verdict.justification.as_deref(), Some("ADR-9997"));
        assert!(
            !verdict.blocking_codes.contains("unjustified"),
            "a justified path is not unjustified, got {:?}",
            verdict.blocking_codes
        );

        fs::remove_dir_all(root).expect("remove temp repo");
    }

    /// The author-facing half of reached ⇒ justified: a path a live registry reaches is clean
    /// with NO ADR naming it, and the report NAMES the reaching source instead of printing
    /// `justified: NO` beside an OK verdict.
    #[test]
    fn check_mode_added_reached_path_is_justified_by_its_reaching_source() {
        let root = unique_temp_repo();
        fs::create_dir_all(root.join("docs/decisions")).expect("create decisions dir");
        fs::create_dir_all(root.join("specs")).expect("create specs dir");
        // A reviewed reachability registration — the only registry available in a temp repo with
        // no cargo workspace. No ADR mentions the path.
        fs::write(
            root.join("specs/reachability-registry.json"),
            r#"{"registered":[{"prefix":"newsvc/","anchor":"ADR-9998: the new service tree."}]}"#,
        )
        .expect("write registry");

        let verdict = check_verdict(&root, "newsvc/src/lib.rs");

        assert_eq!(verdict.reachable_from, vec!["reachability-registry"]);
        assert_eq!(
            verdict.justification.as_deref(),
            Some("reached:reachability-registry"),
            "the report must name the reaching source, not print NO"
        );
        assert!(
            verdict.blocking_codes.is_empty(),
            "a reached path REDs nothing, got {:?}",
            verdict.blocking_codes
        );

        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn check_mode_excluded_path_is_outside_the_accounting_universe() {
        // A `third-party/` path never enters the scm-facts tracked universe ⇒ cannot RED the
        // firewall, so the check must not flag it even without any ADR.
        let root = unique_temp_repo();
        fs::create_dir_all(root.join("docs/decisions")).expect("create decisions dir");

        let verdict = check_verdict(&root, "third-party/vendored/lib.rs");

        assert!(verdict.excluded, "path_excludes covers third-party/");
        assert!(
            verdict.blocking_codes.is_empty(),
            "excluded paths carry no blocking codes, got {:?}",
            verdict.blocking_codes
        );

        fs::remove_dir_all(root).expect("remove temp repo");
    }

    /// The AUTHOR-SIDE half of the OWNERS accounting floor. CI's path is covered by the
    /// total-accounting gate's live-corpus test; this covers the pre-push check, which builds
    /// its own `RepoInputs` and would silently miss the floor if the wiring were dropped.
    ///
    /// The failure mode being pinned is a false alarm, which is worse than useless here: an
    /// author adding a valid `os/OWNERS` would be told to WOULD RED, would go hand-write a
    /// reachability-registry row to "fix" it, and would land exactly the dead weight this
    /// change deletes. The invalid file is the control — it must still be reported.
    #[test]
    fn check_mode_accounts_a_valid_owners_file_and_still_reds_an_invalid_one() {
        let root = unique_temp_repo();
        fs::create_dir_all(root.join("good")).expect("create good dir");
        fs::create_dir_all(root.join("bad")).expect("create bad dir");
        fs::write(root.join("good/OWNERS"), "cloud-ci-platform\n").expect("write valid");
        fs::write(root.join("bad/OWNERS"), "# owner: TBD\n").expect("write invalid");

        let cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();
        let policy = Policy::from_config(&cfg).expect("policy");
        let paths = vec!["good/OWNERS".to_owned(), "bad/OWNERS".to_owned()];
        let verdicts = check_added_paths(&root, &cfg, &policy, &paths).expect("check added paths");

        let good = verdicts
            .iter()
            .find(|v| v.path == "good/OWNERS")
            .expect("good verdict");
        assert!(
            good.blocking_codes.is_empty(),
            "a schema-valid OWNERS file must not be reported as WOULD RED, got {:?}",
            good.blocking_codes
        );
        // The printed columns must agree with the verdict, or the report says OK directly
        // under "justified by NO · reachable via UNREACHABLE".
        assert_eq!(good.justification.as_deref(), Some("owners-schema"));
        assert_eq!(good.reachable_from, vec!["owners-schema".to_owned()]);

        let bad = verdicts
            .iter()
            .find(|v| v.path == "bad/OWNERS")
            .expect("bad verdict");
        for code in ["unjustified", "unreachable"] {
            assert!(
                bad.blocking_codes.contains(code),
                "a comment-only OWNERS file must still be reported as {code}, got {:?}",
                bad.blocking_codes
            );
        }
        assert_eq!(bad.justification, None);
        assert!(bad.reachable_from.is_empty());

        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn check_mode_verdicts_equal_producer_firewall_verdicts() {
        // PARITY: for the SAME inputs, check-mode's per-path blocking codes are byte-identical to
        // running the producer face-builder + the firewall's own evaluator (minus `unowned`, which
        // the pre-push partial set cannot soundly compute). This pins that no divergent verdict
        // logic is ever introduced — the check reuses the shared functions, it does not re-derive.
        let root = unique_temp_repo();
        let decisions = root.join("docs/decisions");
        fs::create_dir_all(&decisions).expect("create decisions dir");
        let justified_path = "alpha/src/lib.rs";
        fs::write(
            decisions.join("ADR-9996-alpha.md"),
            format!("The alpha crate is `{justified_path}`.\n"),
        )
        .expect("write ADR");

        let cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();
        let policy = Policy::from_config(&cfg).expect("policy from bundled default");
        let paths = vec![justified_path.to_owned(), "beta/src/lib.rs".to_owned()];

        // Producer reference: the exact pipeline CI runs, over the same inputs.
        let inputs = RepoInputs {
            tracked_paths: paths.clone(),
            owners: BTreeMap::new(),
            justifications: resolve_justifications(&root, &paths, &cfg),
            reachability: resolve_reachability(&root, &paths, &cfg).expect("reachability"),
            dup_of: BTreeMap::new(),
            valid_owners_files: resolve_owners(&root, &paths, &cfg).valid_files,
        };
        let registry = build_registry(&inputs, &policy).expect("build registry");
        let mut producer: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for finding in ci_artifact_accountability::evaluate_keyed(&registry) {
            if finding.code == "unowned" {
                continue;
            }
            producer
                .entry(finding.key)
                .or_default()
                .insert(finding.code);
        }

        let verdicts = check_added_paths(&root, &cfg, &policy, &paths).expect("check added paths");
        for verdict in &verdicts {
            let expected = producer.get(&verdict.path).cloned().unwrap_or_default();
            assert_eq!(
                verdict.blocking_codes, expected,
                "check-mode codes for {} must equal producer+firewall codes",
                verdict.path
            );
        }
        // And the substance held: beta is unjustified, alpha is not.
        let beta = verdicts
            .iter()
            .find(|v| v.path == "beta/src/lib.rs")
            .expect("beta");
        let alpha = verdicts
            .iter()
            .find(|v| v.path == justified_path)
            .expect("alpha");
        assert!(beta.blocking_codes.contains("unjustified"));
        assert!(!alpha.blocking_codes.contains("unjustified"));

        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn enforcement_inventory_flags_live_cli_authority_but_not_bridge_history() {
        let root = unique_temp_repo();
        let cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();
        let decisions = root.join(&cfg.justification.adr_dir);
        fs::create_dir_all(&decisions).expect("create decisions dir");
        fs::write(
            decisions.join("ADR-0100-live-cli-authority.md"),
            "---\nid: ADR-0100\nstatus: Proposed\n---\nMerge authority is `oya gate run-all`; the pipeline must require it before promotion.\n",
        )
        .expect("write live cli authority ADR");
        fs::write(
            decisions.join("ADR-0101-bridge-history.md"),
            "---\nid: ADR-0101\nstatus: Accepted\n---\nHistorical note: legacy `oya gate run-all` output is bridge evidence only, never merge authority.\n`oya gate` and `oya verify` may remain only as legacy/local migration wrappers until cloud-ci is live.\nUntil the cloud-ci required context is live, legacy `oya gate`/`oya verify`\noutput is migration evidence only and cannot be the merge/exit authority.\n",
        )
        .expect("write historical bridge ADR");
        fs::write(
            decisions.join("ADR-0112-oya-dev-cli-local-feedback.md"),
            "---\nid: ADR-0112\nstatus: Accepted\n---\nRetired `cargo run -p oya-dev-cli -- gate validate quality-lane` commands are transitional/local feedback only; cloud-ci/Rust gate authority owns merge admission.\n",
        )
        .expect("write oya-dev-cli local-feedback ADR");
        fs::write(
            decisions.join("ADR-0102-split-line-live-cli-authority.md"),
            "---\nid: ADR-0102\nstatus: Accepted\n---\nMerge authority is the required gate of record.\n`oya gate run-all`\n",
        )
        .expect("write split-line cli authority ADR");
        fs::write(
            decisions.join("ADR-0103-retired-contrast-live-cli-authority.md"),
            "---\nid: ADR-0103\nstatus: Accepted\n---\nUnlike the retired bridge, `oya gate run-all` is now the required context.\n",
        )
        .expect("write retired-contrast cli authority ADR");
        fs::write(
            decisions.join("ADR-0104-active-enforced-by-cli.md"),
            "---\nid: ADR-0104\nstatus: Accepted\nenforcement_status: active\nenforced_by: oya gate validate aspirational-enforcement\n---\n",
        )
        .expect("write active enforced_by cli ADR");
        fs::write(
            decisions.join("ADR-0105-future-blocking-enforced-by-cli.md"),
            "---\nid: ADR-0105\nstatus: Accepted\nenforcement_status: advisory-until-2026-08-15-blocker-thereafter\nenforced_by:\n  - oya gate validate emergency-services-bypass-attestation-chain\n---\n",
        )
        .expect("write future-blocking enforced_by cli ADR");
        fs::write(
            decisions.join("ADR-0106-ci-lane-refuses-merge.md"),
            "---\nid: ADR-0106\nstatus: Accepted\n---\nCI lane `oya gate validate multi-region-disposition` reads manifests and refuses merge on mismatch.\n",
        )
        .expect("write ci lane refuses merge ADR");
        fs::write(
            decisions.join("ADR-0107-superseded-history-cli.md"),
            "---\nid: ADR-0107\nstatus: Superseded\nsuperseded_by: [ADR-0110]\n---\nA migration period used local `oya verify` as the gate of record during transition.\n",
        )
        .expect("write superseded history cli ADR");
        fs::write(
            decisions.join("ADR-0108-bridge-adjacent-live-authority.md"),
            "---\nid: ADR-0108\nstatus: Accepted\n---\nLegacy `oya verify` output is bridge evidence only.\n`oya gate run-all` is now the required context.\n",
        )
        .expect("write bridge-adjacent live cli authority ADR");
        fs::write(
            decisions.join("ADR-0109-block-superseded-history-cli.md"),
            "---
id: ADR-0109
status: Accepted
superseded_by:
  - ADR-0110
---
A migration period used local `oya verify` as the gate of record during transition.
",
        )
        .expect("write block-superseded history cli ADR");
        fs::write(
            decisions.join("ADR-0110-adjacent-table-rows.md"),
            "---
id: ADR-0110
status: Accepted
---
| Surface | Notes |
| --- | --- |
| Validator crate | Exposed via `oya check active-artifact-contract` while integration is pending. |
| CI lane | Enforces the contract at PR time. |
",
        )
        .expect("write adjacent table rows ADR");
        fs::write(
            decisions.join("ADR-0111-adjacent-list-items.md"),
            "---
id: ADR-0111
status: Accepted
---
1. **Enforcement** — CI lane that BLOCKS PRs on violation.
2. **Verification** — Rust checker crate exposing `oya check <name>` for local verification.
",
        )
        .expect("write adjacent list items ADR");

        let inputs = collect_enforcement_inputs(
            &root,
            &cfg,
            &ScmFacts {
                tracked_paths: Vec::new(),
            },
        );

        assert!(
            inputs.rows.iter().any(|row| {
                row.source_artifact
                    .ends_with("ADR-0100-live-cli-authority.md")
                    && row.maps_to_oya_cli
            }),
            "a live CLI-as-merge-authority claim must be inventoried as an oya CLI surface: {:?}",
            inputs.rows
        );
        assert!(
            !inputs
                .rows
                .iter()
                .any(|row| row.source_artifact.ends_with("ADR-0101-bridge-history.md")),
            "historical bridge/local-feedback CLI references must not become new blocking authority rows: {:?}",
            inputs.rows
        );
        for adr in [
            "ADR-0102-split-line-live-cli-authority.md",
            "ADR-0103-retired-contrast-live-cli-authority.md",
            "ADR-0104-active-enforced-by-cli.md",
            "ADR-0105-future-blocking-enforced-by-cli.md",
            "ADR-0106-ci-lane-refuses-merge.md",
            "ADR-0108-bridge-adjacent-live-authority.md",
        ] {
            assert!(
                inputs
                    .rows
                    .iter()
                    .any(|row| row.source_artifact.ends_with(adr) && row.maps_to_oya_cli),
                "{adr} must be inventoried as a live retired-CLI authority surface: {:?}",
                inputs.rows
            );
        }
        for adr in [
            "ADR-0107-superseded-history-cli.md",
            "ADR-0109-block-superseded-history-cli.md",
            "ADR-0110-adjacent-table-rows.md",
            "ADR-0111-adjacent-list-items.md",
            "ADR-0112-oya-dev-cli-local-feedback.md",
        ] {
            assert!(
                !inputs
                    .rows
                    .iter()
                    .any(|row| row.source_artifact.ends_with(adr)),
                "superseded ADR history must not become a live retired-CLI authority row: {:?}",
                inputs.rows
            );
        }

        fs::remove_dir_all(root).expect("remove temp repo");
    }

    /// RED fixture for FRIC-1781320000: a duplicate-numbered ADR pair (the parallel-lane
    /// collision shape) must surface in `duplicate_ids`; a filename/front-matter id
    /// mismatch (the re-keying vector that can MASK such a collision) must surface in
    /// `id_mismatches`; and the allocator must derive the next free number past BOTH the
    /// filename and the front-matter id space.
    #[test]
    fn crosswalk_flags_duplicate_numbers_id_mismatches_and_allocates_next_free() {
        let root = unique_temp_repo();
        let decisions = root.join("docs/decisions");
        fs::create_dir_all(&decisions).expect("create decisions dir");
        fs::write(
            decisions.join("ADR-0001-first-lane.md"),
            "---\nid: ADR-0001\nstatus: Accepted\n---\n",
        )
        .expect("write first ADR");
        fs::write(
            decisions.join("ADR-0001-second-lane.md"),
            "---\nid: ADR-0001\nstatus: Proposed\n---\n",
        )
        .expect("write colliding ADR");
        fs::write(
            decisions.join("ADR-0002-mismatch.md"),
            "---\nid: ADR-0009\nstatus: Proposed\n---\n",
        )
        .expect("write mismatched ADR");

        let cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();
        let inputs = collect_crosswalk_inputs(&root, &cfg);

        assert!(
            inputs.duplicate_ids.contains(&"ADR-0001".to_owned()),
            "a duplicate-numbered ADR pair must be detected: {:?}",
            inputs.duplicate_ids
        );
        assert_eq!(
            inputs.id_mismatches,
            vec!["ADR-0002-mismatch.md:ADR-0002!=ADR-0009".to_owned()],
            "a filename/front-matter id disagreement must be detected"
        );
        // max(filename ids 1,1,2; front-matter ids 1,1,9) + 1 = 10.
        assert_eq!(inputs.next_free_id, "ADR-0010");

        fs::remove_dir_all(root).expect("remove temp repo");
    }

    /// RED fixture for FRIC-1781430000 (the phantom-0397 shape): an `ADR-NNNN` citation in
    /// a governed surface (a decision body, the roadmap, the masterplan `bound_adrs`) that
    /// resolves to NO on-disk decision id must surface as a `<cited>@<source>` edge in
    /// `phantom_citations`; a citation of an EXISTING decision must not; a citation of a
    /// GRANDFATHERED historical phantom (reviewed shrink-only DATA) must not; a masterplan
    /// mention OUTSIDE `bound_adrs` must not. Then MINTING the missing record at the cited
    /// number must heal the edge with zero retargeting — the ADR-0397 reconstruction
    /// mechanism, witnessed mechanically.
    #[test]
    fn crosswalk_flags_phantom_citations_and_minting_heals_them() {
        let root = unique_temp_repo();
        let cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();
        let decisions = root.join(&cfg.justification.adr_dir);
        fs::create_dir_all(&decisions).expect("create decisions dir");
        fs::write(
            decisions.join("ADR-0001-citer.md"),
            "---\nid: ADR-0001\nstatus: Accepted\n---\nCites ADR-0900 (phantom), \
             ADR-0002 (exists), and ADR-0436 (grandfathered inventory).\n",
        )
        .expect("write citing ADR");
        fs::write(
            decisions.join("ADR-0002-target.md"),
            "---\nid: ADR-0002\nstatus: Accepted\n---\n",
        )
        .expect("write existing target ADR");
        let roadmap_path = root.join(&cfg.justification.roadmap);
        fs::create_dir_all(roadmap_path.parent().expect("roadmap parent"))
            .expect("create specs dir");
        fs::write(&roadmap_path, r#"{"wave": "depends on ADR-0901"}"#).expect("write roadmap");
        let masterplan_path = root.join(&cfg.reachability.masterplan);
        fs::write(
            &masterplan_path,
            r#"{"phases":[{"bound_adrs":["ADR-0902","ADR-0002"],"narrative":"ADR-0903 outside bound_adrs is not a citation edge"}]}"#,
        )
        .expect("write masterplan");

        let inputs = collect_crosswalk_inputs(&root, &cfg);
        assert_eq!(
            inputs.phantom_citations,
            vec![
                format!("ADR-0900@{}/ADR-0001-citer.md", cfg.justification.adr_dir),
                format!("ADR-0901@{}", cfg.justification.roadmap),
                format!("ADR-0902@{}#bound_adrs", cfg.reachability.masterplan),
            ],
            "exactly the unresolved, non-grandfathered citation edges must surface"
        );

        // MINT the missing records at the cited numbers: every edge heals, zero retargeting.
        fs::write(
            decisions.join("ADR-0900-minted.md"),
            "---\nid: ADR-0900\nstatus: Proposed\n---\n",
        )
        .expect("write minted ADR");
        fs::write(
            decisions.join("ADR-0901-minted.md"),
            "---\nid: ADR-0901\nstatus: Proposed\n---\n",
        )
        .expect("write minted ADR");
        fs::write(
            decisions.join("ADR-0902-minted.md"),
            "---\nid: ADR-0902\nstatus: Proposed\n---\n",
        )
        .expect("write minted ADR");
        let healed = collect_crosswalk_inputs(&root, &cfg);
        assert!(
            healed.phantom_citations.is_empty(),
            "minting the record at the cited number must heal every edge: {:?}",
            healed.phantom_citations
        );

        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn masterplan_propagation_is_bound_or_explicitly_nonbinding_never_a_narrative_mention() {
        let masterplan = serde_json::json!({
            "planning_authority": {
                "bound_adrs": ["ADR-0001"]
            },
            "masterplan_v2": {
                "accepted_decision_propagation_dispositions": {
                    "decisions": [{
                        "id": "ADR-0002",
                        "lifecycle_state": "Accepted",
                        "planning_impact": false,
                        "sequencing_effect": "none",
                        "binding_plan_approval_effect": "none",
                        "execution_dispatch_effect": "none",
                        "hold_state": "HOLD(Planning)",
                        "disposition_ref": "/specs/master-plan-sequencing.json#_metadata.accepted_decision_propagation_dispositions"
                    }]
                },
                "narrative": "ADR-0003 is mentioned only as prose"
            }
        });
        let text = serde_json::to_string(&masterplan).expect("serialize masterplan fixture");

        assert!(masterplan_propagates_decision(&text, "ADR-0001"));
        assert!(masterplan_propagates_decision(&text, "ADR-0002"));
        assert!(
            !masterplan_propagates_decision(&text, "ADR-0003"),
            "a narrative mention must not satisfy masterplan propagation"
        );

        let mut laundered = masterplan;
        laundered["masterplan_v2"]["accepted_decision_propagation_dispositions"]["decisions"][0]
            ["disposition_ref"] =
            serde_json::json!("/specs/masterplan.json#planning_authority.bound_adrs");
        let text = serde_json::to_string(&laundered).expect("serialize laundered fixture");
        assert!(
            !masterplan_propagates_decision(&text, "ADR-0002"),
            "a nonbinding marker must not point into binding planning authority"
        );
    }

    /// The token scanner accepts exactly-four-digit ids and rejects longer digit runs.
    #[test]
    fn adr_citation_tokens_match_exactly_four_digits() {
        let tokens = adr_citation_tokens(
            "ADR-0001 then ADR-04031 (five digits: no match) then (ADR-0403's) and ADR-12",
        );
        let expected: BTreeSet<String> = ["ADR-0001", "ADR-0403"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        assert_eq!(tokens, expected);
    }

    /// The grandfathered inventory is shrink-only DATA: it must never contain ADR-0397
    /// (healed by the minted record — the exhibit that keeps this lane frozen-empty) and
    /// must stay sorted+deduped so reviews see a canonical list.
    #[test]
    fn grandfathered_phantom_inventory_is_canonical_and_excludes_healed_ids() {
        let list = GRANDFATHERED_PHANTOM_DECISION_IDS;
        assert!(
            !list.contains(&"ADR-0397"),
            "ADR-0397 was healed by minting the record; it must not be grandfathered"
        );
        let mut sorted = list.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.as_slice(),
            list,
            "inventory must be sorted + deduped"
        );
    }

    /// A clean corpus carries no mismatch signal and allocates max+1 from the filename ids.
    #[test]
    fn crosswalk_is_quiet_and_allocates_on_a_clean_corpus() {
        let root = unique_temp_repo();
        let decisions = root.join("docs/decisions");
        fs::create_dir_all(&decisions).expect("create decisions dir");
        fs::write(
            decisions.join("ADR-0007-clean.md"),
            "---\nid: ADR-0007\nstatus: Accepted\n---\n",
        )
        .expect("write clean ADR");

        let cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();
        let inputs = collect_crosswalk_inputs(&root, &cfg);

        assert!(inputs.duplicate_ids.is_empty());
        assert!(inputs.id_mismatches.is_empty());
        assert_eq!(inputs.next_free_id, "ADR-0008");

        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn slo_coverage_preserves_duplicate_basenames_to_prevent_false_green() {
        let root = unique_temp_repo();
        // A root workspace manifest is required: the slo-coverage face now composes the
        // live-OR-marked predicate, which resolves workspace members in-process. An empty
        // members array yields an empty live set (these `service` rows are not live crates).
        fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = []\n")
            .expect("write root manifest");
        let first = root.join("registry/catalog-a/service.yaml");
        let second = root.join("registry/catalog-b/service.yaml");
        fs::create_dir_all(first.parent().expect("first parent")).expect("create first parent");
        fs::create_dir_all(second.parent().expect("second parent")).expect("create second parent");
        // Both rows carry an explicit non-live marker so the composed liveness predicate does not
        // fire here — this test isolates the duplicate-stem SLO behaviour.
        fs::write(
            &first,
            "slo: preview-control-plane\nstatus: designed-ahead-row-no-crate\n",
        )
        .expect("write first catalog row");
        fs::write(
            &second,
            "# deliberately missing slo\nstatus: designed-ahead-row-no-crate\n",
        )
        .expect("write second catalog row");

        let mut cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();
        cfg.slo_coverage.catalog_record_globs = vec![
            "registry/catalog-a/*.yaml".to_owned(),
            "registry/catalog-b/*.yaml".to_owned(),
        ];
        let tracked_paths = vec![
            "registry/catalog-a/service.yaml".to_owned(),
            "registry/catalog-b/service.yaml".to_owned(),
        ];

        let face = collect_slo_coverage(&root, &tracked_paths, &cfg).expect("slo-coverage face");
        let rows = face["rows"].as_array().expect("rows");
        assert_eq!(rows.len(), 2, "duplicate stems must not collapse rows");
        assert_eq!(rows[0]["crate_id"], "service");
        assert_eq!(rows[0]["source_path"], "registry/catalog-a/service.yaml");
        assert_eq!(rows[1]["crate_id"], "service");
        assert_eq!(rows[1]["source_path"], "registry/catalog-b/service.yaml");

        let findings = ci_slo_coverage::evaluate_keyed(&face);
        assert!(
            findings.iter().any(|finding| {
                finding.code == "slo_missing_or_blank_slo" && finding.key == "service"
            }),
            "one duplicate row with a valid SLO must not hide the missing-SLO duplicate: {findings:?}"
        );

        fs::remove_dir_all(root).expect("remove temp repo");
    }
    #[test]
    fn cargo_prefix_emits_debranded_candidate_as_advisory_coverage() {
        let root = unique_temp_repo();
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"tools/*\"]\nresolver = \"2\"\n",
        )
        .expect("write root manifest");

        let crate_dir = root.join("tools/unprefixed-app");
        fs::create_dir_all(&crate_dir).expect("create crate dir");
        fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"unprefixed-app\"\n",
        )
        .expect("write crate manifest");

        let tracked_paths = vec![
            "Cargo.toml".to_owned(),
            "tools/unprefixed-app/Cargo.toml".to_owned(),
        ];
        let face = collect_cargo_prefix(
            &root,
            &tracked_paths,
            &oya_ci_config_kernel::OyaCiConfig::bundled_default(),
        )
        .expect("collect cargo-prefix");

        let rows = face["rows"].as_array().expect("rows");
        assert_eq!(rows.len(), 1, "producer must not hide candidate coverage");
        assert_eq!(rows[0]["member_path"], "tools/unprefixed-app");
        assert_eq!(rows[0]["package_name"], "unprefixed-app");
        assert_eq!(rows[0]["cargo_prefix_scope"], "advisory");

        let findings = ci_crate_name_prefix::evaluate_keyed(&face);
        assert!(
            findings.is_empty(),
            "de-branded advisory candidates must not become born-blocking cargo-prefix debt: {findings:?}"
        );

        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn catalog_liveness_face_is_bidirectional_and_tracks_source_crate_paths() {
        let root = unique_temp_repo();
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"audit/ports/emission-api\", \"audit/ports/missing-row\", \"audit/ports/exempt-row\"]\n",
        )
        .expect("write root manifest");
        for (dir, name) in [
            ("audit/ports/emission-api", "audit-emission-api"),
            ("audit/ports/missing-row", "audit-missing-row"),
            ("audit/ports/exempt-row", "audit-exempt-row"),
        ] {
            let dir = root.join(dir);
            fs::create_dir_all(&dir).expect("create member dir");
            fs::write(
                dir.join("Cargo.toml"),
                format!("[package]\nname = \"{name}\"\n"),
            )
            .expect("write member manifest");
        }
        write_test_file(
            &root,
            "registry/catalog/audit-emission-api.yaml",
            "traceability:\n  source_crate: crates/old-audit-emission-api/Cargo.toml\n",
        );

        let mut cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();
        cfg.catalog_liveness.workspace_member_exemptions =
            vec![oya_ci_config_kernel::CatalogLivenessExemption {
                path_glob: "audit/ports/exempt-row".to_owned(),
                owner: "platform-governance".to_owned(),
                reason: "temporary fixture exemption proves bounded exemptions are surfaced"
                    .to_owned(),
                cutover: "remove when fixture gains a catalog row".to_owned(),
            }];
        let tracked_paths = vec![
            "Cargo.toml".to_owned(),
            "audit/ports/emission-api/Cargo.toml".to_owned(),
            "audit/ports/missing-row/Cargo.toml".to_owned(),
            "audit/ports/exempt-row/Cargo.toml".to_owned(),
            "registry/catalog/audit-emission-api.yaml".to_owned(),
        ];

        let face =
            collect_catalog_liveness(&root, &tracked_paths, &cfg).expect("catalog-liveness face");
        let rows = face["rows"].as_array().expect("rows");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["crate_id"], "audit-emission-api");
        assert_eq!(rows[0]["is_live"].as_bool(), Some(true));
        assert_eq!(rows[0]["source_crate_exists"].as_bool(), Some(false));

        let live_crates = face["live_crates"].as_array().expect("live_crates");
        assert_eq!(live_crates.len(), 3);
        assert!(live_crates.iter().any(|row| {
            row["crate_id"] == "audit-emission-api"
                && row["has_catalog_row"].as_bool() == Some(true)
        }));
        assert!(live_crates.iter().any(|row| {
            row["crate_id"] == "audit-missing-row"
                && row["has_catalog_row"].as_bool() == Some(false)
                && row["exemption"].is_null()
        }));
        assert!(live_crates.iter().any(|row| {
            row["crate_id"] == "audit-exempt-row"
                && row["has_catalog_row"].as_bool() == Some(false)
                && row["exemption"]["owner"] == "platform-governance"
        }));

        let findings = ci_service_catalog_parity::evaluate_keyed(&face);
        assert!(findings.iter().any(|finding| {
            finding.code == "catalog_record_source_crate_missing"
                && finding.key == "audit-emission-api"
        }));
        assert!(findings.iter().any(|finding| {
            finding.code == "catalog_live_crate_without_row" && finding.key == "audit-missing-row"
        }));
        assert!(
            !findings
                .iter()
                .any(|finding| finding.key == "audit-exempt-row"),
            "bounded exemption must suppress only its own missing-row finding: {findings:?}"
        );

        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn workspace_glob_coverage_reports_explicit_members_and_uncovered_crates() {
        let root = unique_temp_repo();
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"libs/oya-*\", \"tools/oya-explicit-app\"]\nexclude = [\"cloud/cloud-kernel\"]\n",
        )
        .expect("write root manifest");

        for (dir, name) in [
            ("libs/oya-covered-kernel", "oya-covered-kernel"),
            ("tools/oya-explicit-app", "oya-explicit-app"),
            ("tools/oya-orphan-app", "oya-orphan-app"),
            (
                "cloud/cloud-kernel/crates/oya-kernel-domain",
                "oya-kernel-domain",
            ),
        ] {
            let dir = root.join(dir);
            fs::create_dir_all(&dir).expect("create crate dir");
            fs::write(
                dir.join("Cargo.toml"),
                format!("[package]\nname = \"{name}\"\n"),
            )
            .expect("write crate manifest");
        }
        fs::create_dir_all(root.join("cloud/cloud-kernel")).expect("create nested workspace");
        fs::write(
            root.join("cloud/cloud-kernel/Cargo.toml"),
            "[workspace]\nmembers = [\"crates/oya-kernel-domain\"]\n",
        )
        .expect("write nested workspace manifest");

        let tracked_paths = vec![
            "Cargo.toml".to_owned(),
            "libs/oya-covered-kernel/Cargo.toml".to_owned(),
            "tools/oya-explicit-app/Cargo.toml".to_owned(),
            "tools/oya-orphan-app/Cargo.toml".to_owned(),
            "cloud/cloud-kernel/Cargo.toml".to_owned(),
            "cloud/cloud-kernel/crates/oya-kernel-domain/Cargo.toml".to_owned(),
        ];

        let face = collect_workspace_glob_coverage(
            &root,
            &tracked_paths,
            &oya_ci_config_kernel::OyaCiConfig::bundled_default(),
        )
        .expect("collect workspace glob coverage");
        let findings = ci_workspace_member_coverage::evaluate_keyed(&face);

        assert!(findings.iter().any(|finding| {
            finding.code == "workspace_member_explicit_path"
                && finding.key == "tools/oya-explicit-app"
        }));
        assert!(findings.iter().any(|finding| {
            finding.code == "crate_dir_not_covered" && finding.key == "tools/oya-orphan-app"
        }));
        assert!(
            !face["rows"]
                .as_array()
                .expect("rows")
                .iter()
                .any(|row| row["crate_dir"] == "cloud/cloud-kernel/crates/oya-kernel-domain"),
            "nested workspace package dirs are skipped"
        );

        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn workspace_glob_coverage_reports_every_unexcluded_match_without_manifest() {
        let root = unique_temp_repo();
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"comms/*/*\"]\nexclude = [\"comms/messenger/fixtures\"]\n",
        )
        .expect("write root manifest");
        fs::create_dir_all(root.join("comms/messenger/chaos"))
            .expect("create non-crate member match");
        fs::create_dir_all(root.join("comms/messenger/resilience"))
            .expect("create second non-crate member match");
        fs::create_dir_all(root.join("comms/messenger/fixtures"))
            .expect("create excluded non-crate member match");

        let face = collect_workspace_glob_coverage(
            &root,
            &["Cargo.toml".to_owned()],
            &oya_ci_config_kernel::OyaCiConfig::bundled_default(),
        )
        .expect("collect workspace glob coverage");
        let findings = ci_workspace_member_coverage::evaluate_keyed(&face);

        let member_matches: BTreeSet<String> = face["rows"]
            .as_array()
            .expect("rows")
            .iter()
            .filter_map(|row| row["member_match"].as_str().map(str::to_owned))
            .collect();
        assert_eq!(
            member_matches,
            BTreeSet::from([
                "comms/messenger/chaos".to_owned(),
                "comms/messenger/resilience".to_owned(),
            ])
        );
        let missing_manifest_findings: BTreeSet<String> = findings
            .iter()
            .filter(|finding| finding.code == "workspace_member_missing_manifest")
            .map(|finding| finding.key.clone())
            .collect();
        assert_eq!(missing_manifest_findings, member_matches);

        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn cargo_reachability_uses_valid_members_while_coverage_reports_invalid_matches() {
        let root = unique_temp_repo();
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"libs/*\", \"comms/*/*\"]\nexclude = []\n",
        )
        .expect("write root manifest");
        fs::create_dir_all(root.join("libs/valid-kernel")).expect("create valid member");
        fs::write(
            root.join("libs/valid-kernel/Cargo.toml"),
            "[package]\nname = \"valid-kernel\"\n",
        )
        .expect("write member manifest");
        fs::create_dir_all(root.join("comms/messenger/chaos"))
            .expect("create invalid member match");

        assert_eq!(
            read_cargo_member_prefixes(&root).expect("scan valid cargo member prefixes"),
            vec!["libs/valid-kernel/".to_owned()]
        );

        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn reachability_registry_matches_prefixes_exactly_and_fails_loud() {
        let root = unique_temp_repo();
        let reg = root.join("specs/reachability-registry.json");
        fs::create_dir_all(reg.parent().expect("parent")).expect("create specs");
        fs::write(
            &reg,
            r#"{"registered":[{"prefix":"docs/decisions/","anchor":"decision corpus"},{"prefix":"specs/OWNERS","anchor":"ownership seed"}]}"#,
        )
        .expect("write registry");
        let entries = load_reachability_registry(&reg).expect("valid registry parses");
        assert_eq!(entries.len(), 2);
        // dir prefixes cover the subtree; the trailing '/' prevents sibling-dir bleed.
        assert!(registration_matches(
            "docs/decisions/ADR-0700-ci-admission-live-apex.md",
            "docs/decisions/"
        ));
        assert!(registration_matches(
            "docs/adr-archive/ADR-0001-cohesion-thesis-one-product-flat-catalog.md",
            "docs/adr-archive/"
        ));
        assert!(!registration_matches(
            "docs/decisions-evil/x.md",
            "docs/decisions/"
        ));
        // non-'/' entries are EXACT path matches.
        assert!(registration_matches("specs/OWNERS", "specs/OWNERS"));
        assert!(!registration_matches("specs/OWNERS-extra", "specs/OWNERS"));

        // fail-loud: an empty anchor is a rejected registration (never a bare exemption).
        fs::write(&reg, r#"{"registered":[{"prefix":"docs/","anchor":""}]}"#).expect("write");
        assert!(load_reachability_registry(&reg).is_err());
        // fail-loud: a registry without the declared 'registered' array is rejected.
        fs::write(&reg, "{}").expect("write");
        assert!(load_reachability_registry(&reg).is_err());
        // a MISSING file is the declared zero-config default (empty registry).
        fs::remove_file(&reg).expect("remove");
        assert!(
            load_reachability_registry(&reg)
                .expect("missing file is empty")
                .is_empty()
        );
        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn resolve_reachability_reports_registry_source() {
        let root = unique_temp_repo();
        fs::create_dir_all(root.join("specs")).expect("create specs");
        fs::write(
            root.join("specs/reachability-registry.json"),
            r#"{"registered":[{"prefix":"evidence/","anchor":"gate evidence corpus"}]}"#,
        )
        .expect("write registry");
        let cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();
        let paths = vec![
            "evidence/run.json".to_owned(),
            "oya/unregistered.rs".to_owned(),
        ];
        let map = resolve_reachability(&root, &paths, &cfg).expect("resolve");
        assert_eq!(
            map.get("evidence/run.json"),
            Some(&vec!["reachability-registry".to_owned()])
        );
        assert!(!map.contains_key("oya/unregistered.rs"));
        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn resolve_reachability_allows_envelope_prefix_without_tip_free() {
        let root = unique_temp_repo();
        fs::create_dir_all(root.join("specs")).expect("create specs");
        // Empty tip-free registry on purpose — envelope prefixes must be sufficient.
        fs::write(
            root.join("specs/reachability-registry.json"),
            r#"{"registered":[]}"#,
        )
        .expect("write empty registry");
        fs::write(
            root.join(ENVELOPES_RELPATH),
            r#"{
              "roots": {
                "compute": {
                  "branch": "integ/compute",
                  "envelope_globs": ["compute/**"]
                }
              }
            }"#,
        )
        .expect("write envelopes");
        let cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();
        let paths = vec![
            "compute/manifest.json".to_owned(),
            "compute/stale-path-hygiene-note.md".to_owned(),
            "foreign/not-owned.rs".to_owned(),
        ];
        let map = resolve_reachability(&root, &paths, &cfg).expect("resolve");
        assert_eq!(
            map.get("compute/manifest.json"),
            Some(&vec![ENVELOPE_PREFIX_OWNERSHIP_SOURCE.to_owned()])
        );
        assert_eq!(
            map.get("compute/stale-path-hygiene-note.md"),
            Some(&vec![ENVELOPE_PREFIX_OWNERSHIP_SOURCE.to_owned()])
        );
        assert!(!map.contains_key("foreign/not-owned.rs"));
        fs::remove_dir_all(root).expect("remove temp repo");
    }

    /// A text registry reaches a path when it NAMES the path — not when the path's characters
    /// occur somewhere inside it.
    ///
    /// RED before the fix: `masterplan.contains("OWNERS")` was satisfied by
    /// `docs/OWNERS-policy.md`, by the JSON key `"OWNERS"`, and by the bare word in prose, so
    /// every short path (and every path that is a proper prefix of a longer one) was reported
    /// reachable by coincidence.
    ///
    /// BLAST RADIUS, measured over the live 18853-path tracked universe: the substring probe
    /// over-reported exactly TWO paths, both at the repo root — `OWNERS` and `README.md`. NEITHER
    /// becomes unaccounted, which is why this fix needs no registry or baseline edit:
    /// `README.md` is genuinely NAMED by `root-hub-pointers.json` and `DOC-CATALOG.md`, and
    /// `OWNERS` resolves through `owners-schema` (OWNERS files are accounted by CONSTRUCTION —
    /// see `owners_files_are_never_registered_in_the_reachability_registry`, which actively
    /// FORBIDS registering one). Nothing gains reachability: a whole-token match is a strict
    /// subset of a substring match by construction.
    ///
    /// KNOWN CEILING (deliberate, not an oversight): a whole-token match still credits a bare
    /// PROSE word that happens to equal a root-level path — a sentence containing the word
    /// `OWNERS` reaches the root `OWNERS` file. Distinguishing "the word" from "the path"
    /// needs a typed reference field on the registries, not a better tokenizer. The exact-match
    /// fix removes the substring class (a path reached by being a fragment of a DIFFERENT
    /// path); the residual bare-word class is a much smaller surface, bounded to root-level
    /// paths whose whole name is an ordinary English token.
    #[test]
    fn text_registry_reachability_is_whole_path_not_substring() {
        let root = unique_temp_repo();
        fs::create_dir_all(root.join("specs")).expect("create specs");
        fs::create_dir_all(root.join("docs")).expect("create docs");
        // NAMES `docs/OWNERS-policy.md` and `.omc/plans/milestones/M02b/README.md`; NAMES
        // neither the root `OWNERS` nor the root `README.md`, though both occur as substrings
        // of what it does name. Also exercises the two real reference SHAPES the live
        // registries use: a root-anchored `/specs/...` ref and a `<path>#<fragment>` deep link.
        fs::write(
            root.join("specs/masterplan.json"),
            concat!(
                "{\n",
                "  \"policy\": \"docs/OWNERS-policy.md\",\n",
                "  \"readme_ref\": \".omc/plans/milestones/M02b/README.md\",\n",
                "  \"root_anchored\": \"/specs/root-hub-pointers.json\",\n",
                "  \"deep_link\": \"ci/facade/cross-artifact-agreement/src/lib.rs#evaluate_x\"\n",
                "}\n"
            ),
        )
        .expect("write masterplan");

        let cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();
        let paths: Vec<String> = [
            "OWNERS",
            "README.md",
            "docs/OWNERS-policy.md",
            "specs/root-hub-pointers.json",
            "ci/facade/cross-artifact-agreement/src/lib.rs",
        ]
        .iter()
        .map(|p| (*p).to_owned())
        .collect();
        let map = resolve_reachability(&root, &paths, &cfg).expect("resolve");

        // RED case: substring hits that are NOT references.
        assert!(
            !map.contains_key("OWNERS"),
            "root OWNERS must NOT be reachable from a registry that only mentions \
             docs/OWNERS-policy.md and the word OWNERS in prose"
        );
        assert!(
            !map.contains_key("README.md"),
            "root README.md must NOT be reachable from a registry that only names \
             .omc/plans/milestones/M02b/README.md"
        );

        // GREEN cases: the three shapes that ARE references must still reach.
        for named in [
            "docs/OWNERS-policy.md",
            "specs/root-hub-pointers.json",
            "ci/facade/cross-artifact-agreement/src/lib.rs",
        ] {
            assert_eq!(
                map.get(named),
                Some(&vec!["masterplan".to_owned()]),
                "{named} IS named by the registry and must stay reachable"
            );
        }
        fs::remove_dir_all(root).expect("remove temp repo");
    }

    /// ADR-0555 hardening (FRIC-1781400000): the OWNERS content schema + breadth bound
    /// RED/GREEN corpus, dir-loaded from `specs/fixtures/owners-schema/` (data-under-test
    /// — the fixtures are the reviewable spec of the schema).
    #[test]
    fn owners_schema_fixtures_execute_red_green_cases() {
        let fixtures_dir = {
            let mut dir = std::env::current_dir().expect("current_dir");
            loop {
                if dir.join("specs/root-hub-pointers.json").is_file() {
                    break dir.join("specs/fixtures/owners-schema");
                }
                assert!(
                    dir.pop(),
                    "failed to locate repo root from test current_dir"
                );
            }
        };
        let mut entries: Vec<PathBuf> = fs::read_dir(&fixtures_dir)
            .unwrap_or_else(|e| panic!("read {}: {e}", fixtures_dir.display()))
            .map(|entry| entry.expect("dir entry").path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
            .collect();
        entries.sort();
        assert!(
            entries.len() >= 6,
            "owners-schema fixture corpus must carry the RED set (empty / comment-only / \
             garbage / over-broad / poison) plus GREEN exemplars, got {entries:?}"
        );

        for path in entries {
            let text = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
            let fixture: Value = serde_json::from_str(&text)
                .unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
            let id = fixture["fixture_id"]
                .as_str()
                .expect("fixture_id")
                .to_owned();

            let root = unique_temp_repo();
            for (rel, content) in fixture["owners_files"].as_object().expect("owners_files") {
                let abs = root.join(rel);
                fs::create_dir_all(abs.parent().expect("parent")).expect("create owners dir");
                fs::write(&abs, content.as_str().expect("owners content")).expect("write");
            }
            let cfg = match fixture
                .get("max_paths_per_owners_file")
                .and_then(Value::as_u64)
            {
                Some(bound) => oya_ci_config_kernel::OyaCiConfig::from_toml_str(&format!(
                    "[owners]\nmax_paths_per_owners_file = {bound}\n"
                ))
                .expect("fixture bound parses"),
                None => oya_ci_config_kernel::OyaCiConfig::bundled_default(),
            };
            let tracked: Vec<String> = fixture["tracked_paths"]
                .as_array()
                .expect("tracked_paths")
                .iter()
                .map(|v| v.as_str().expect("path").to_owned())
                .collect();

            let resolution = resolve_owners(&root, &tracked, &cfg);

            let expected_owned: BTreeMap<String, String> = fixture["expected_owned"]
                .as_object()
                .expect("expected_owned")
                .iter()
                .map(|(k, v)| (k.clone(), v.as_str().expect("owner").to_owned()))
                .collect();
            assert_eq!(
                resolution.by_path, expected_owned,
                "{id}: owned map mismatch"
            );
            let expected_invalid = fixture["expected_invalid"]
                .as_object()
                .expect("expected_invalid");
            assert_eq!(
                resolution.integrity.invalid.len(),
                expected_invalid.len(),
                "{id}: invalid set mismatch: {:?}",
                resolution.integrity.invalid
            );
            for (file, defect_substr) in expected_invalid {
                let defect = resolution
                    .integrity
                    .invalid
                    .get(file)
                    .unwrap_or_else(|| panic!("{id}: {file} must be flagged invalid"));
                let needle = defect_substr.as_str().expect("defect substring");
                assert!(
                    defect.contains(needle),
                    "{id}: {file} defect {defect:?} must name {needle:?}"
                );
            }
            let expected_over_broad: BTreeMap<String, usize> = fixture["expected_over_broad"]
                .as_object()
                .expect("expected_over_broad")
                .iter()
                .map(|(k, v)| (k.clone(), v.as_u64().expect("coverage") as usize))
                .collect();
            assert_eq!(
                resolution.integrity.over_broad, expected_over_broad,
                "{id}: over-broad set mismatch"
            );

            fs::remove_dir_all(root).expect("remove temp repo");
        }
    }

    /// Live-corpus pin (zero-regression evidence for the ADR-0555 hardening): every
    /// tracked OWNERS file on the tree parses to the codified schema and sits under the
    /// breadth bound, so the conversion's grandfathered baseline cannot grow from this
    /// change. If this fails, fix the named OWNERS file — that is honest registration,
    /// not laundering.
    #[test]
    fn live_owners_corpus_is_schema_valid_and_under_breadth_bound() {
        let root = {
            let mut dir = std::env::current_dir().expect("current_dir");
            loop {
                if dir.join("specs/root-hub-pointers.json").is_file() {
                    break dir;
                }
                assert!(
                    dir.pop(),
                    "failed to locate repo root from test current_dir"
                );
            }
        };
        let scm_facts = load_live_test_scm_facts(&root);
        let cfg = load_config(&root).expect("repo oya-ci.toml loads");
        let resolution = resolve_owners(&root, &scm_facts.tracked_paths, &cfg);
        assert!(
            resolution.integrity.invalid.is_empty(),
            "every live OWNERS file must parse to the codified schema (fix the file): {:?}",
            resolution.integrity.invalid
        );
        assert!(
            resolution.integrity.over_broad.is_empty(),
            "every live OWNERS file must sit under the [owners] max_paths_per_owners_file \
             bound ({}) — split the named registration: {:?}",
            cfg.owners.max_paths_per_owners_file,
            resolution.integrity.over_broad
        );
        assert!(
            !resolution.by_path.is_empty(),
            "the live corpus carries valid OWNERS registrations; an empty owned set means \
            the resolver regressed"
        );
    }

    #[test]
    fn policy_root_overrides_candidate_oya_ci_config() {
        let candidate_root = unique_temp_repo();
        let trusted_root = unique_temp_repo();
        fs::write(candidate_root.join("oya-ci.toml"), "profile = 'neutral'\n")
            .expect("write candidate policy");
        fs::write(trusted_root.join("oya-ci.toml"), "profile = 'oyatie'\n")
            .expect("write trusted policy");

        let cfg = load_policy_config(&candidate_root, Some(&trusted_root))
            .expect("trusted policy root loads");

        assert_eq!(cfg.profile, oya_ci_config_kernel::Profile::Oyatie);
        fs::remove_dir_all(candidate_root).expect("remove candidate temp repo");
        fs::remove_dir_all(trusted_root).expect("remove trusted temp repo");
    }

    #[test]
    fn policy_root_defaults_to_candidate_oya_ci_config() {
        let candidate_root = unique_temp_repo();
        fs::write(candidate_root.join("oya-ci.toml"), "profile = 'neutral'\n")
            .expect("write candidate policy");

        let cfg = load_policy_config(&candidate_root, None).expect("candidate policy root loads");

        assert_eq!(cfg.profile, oya_ci_config_kernel::Profile::Neutral);
        fs::remove_dir_all(candidate_root).expect("remove candidate temp repo");
    }
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

/// Extract `license = "..."` from the `[package]` table of a Cargo.toml. Missing licenses stay
/// as `None` so the cloud-ci app can map them to a keyed surface-all finding instead of the
/// legacy dev-cli's first-error string.
fn parse_package_license(contents: &str) -> Option<String> {
    let mut in_package = false;
    for raw in contents.lines() {
        let trimmed = raw.split('#').next().unwrap_or("").trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        if key.trim() != "license" {
            continue;
        }
        return Some(value.trim().trim_matches('"').to_owned());
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
    is_workspace_root: bool,
}

/// Enumerate the first-party `oya-*` crates and emit their §2.5#7 manifest-hygiene flags (the
/// gate's I/O). The gate's `evaluate_keyed` turns missing flags into Findings. Deterministic
/// (BTreeMap, sorted) so committed==regenerated holds byte-for-byte. Scoped to `oya-*`.
///
/// KEYED BY MANIFEST PATH, not package name: a rehomed destination crate can share its package
/// name with the retained legacy source (integ/procurement absorb, PR #1672). A name-keyed map
/// would let the sorted-later legacy manifest overwrite the destination's concrete
/// `version`/`rust-version` and non-workspace lint flags, so the live-corpus test and
/// oya-ci-required would pass without ever checking the newly tracked destination manifest.
/// Path keys preserve identity: BOTH manifests appear with their own flags, and
/// `is_workspace_root` lets the gate accept a workspace-root manifest's concrete values.
fn collect_manifest_hygiene(
    repo_root: &Path,
    tracked_paths: &[String],
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> Value {
    let prefix = cfg.naming.required_prefix.as_str();
    let mut by_path: BTreeMap<String, (String, ManifestFlags)> = BTreeMap::new();
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
        by_path.insert(path.clone(), (name, parse_manifest_flags(&contents)));
    }
    let rows: Vec<Value> = by_path
        .into_iter()
        .map(|(manifest_path, (name, f))| {
            json!({
                "manifest_path": manifest_path,
                "crate_name": name,
                "has_version_workspace": f.version_workspace,
                "has_rust_version_workspace": f.rust_version_workspace,
                "has_publish_false": f.publish_false,
                "has_license": f.license,
                "has_lints_workspace": f.lints_workspace,
                "is_workspace_root": f.is_workspace_root,
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
    // A manifest declaring its own `[workspace]` table IS a workspace root: it cannot inherit
    // `version`/`rust-version`/`[lints]` from a parent workspace, so concrete values there are
    // the legitimate form (e.g. a parked nested-workspace-root destination crate). The gate
    // skips the workspace-inheritance checks for such rows while enforcing every other field.
    f.is_workspace_root = has_workspace_table(contents);
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

/// The HISTORICAL phantom-citation inventory (FRIC-1781430000): decision ids that governed
/// surfaces cite TODAY with no decision file on disk, inventoried 2026-06-12 during the
/// ADR-0397 reconstruction (audit register H-19). This is reviewed, shrink-only carve-out
/// DATA — the same doctrine as the brand-residue carve-outs: exceptions live as DATA, never
/// as evaluator branches. Each id is ledgered as its own friction-ledger row listing its
/// citation sites; healing an id (minting the record at the number, or retargeting every
/// citer) REMOVES it here. ADDING an id is forbidden — a new phantom citation is exactly
/// the defect the `phantom_decision_citation` lane blocks (its baseline is frozen-empty;
/// any non-grandfathered phantom edge is born-blocking). ADR-0397 itself is deliberately
/// NOT in this list: it was healed by minting the record, which is what keeps this lane's
/// live key set empty.
const GRANDFATHERED_PHANTOM_DECISION_IDS: [&str; 62] = [
    "ADR-0000", "ADR-0012", "ADR-0033", "ADR-0037", "ADR-0041", "ADR-0050", "ADR-0086", "ADR-0088",
    "ADR-0125", "ADR-0126", "ADR-0127", "ADR-0224", "ADR-0231", "ADR-0232", "ADR-0247", "ADR-0322",
    "ADR-0323", "ADR-0327", "ADR-0342", "ADR-0345", "ADR-0395", "ADR-0399", "ADR-0403", "ADR-0406",
    "ADR-0407", "ADR-0408", "ADR-0409", "ADR-0411", "ADR-0413", "ADR-0416", "ADR-0418", "ADR-0419",
    "ADR-0420", "ADR-0421", "ADR-0423", "ADR-0428", "ADR-0429", "ADR-0434", "ADR-0436", "ADR-0441",
    "ADR-0443", "ADR-0448", "ADR-0449", "ADR-0450", "ADR-0451", "ADR-0454", "ADR-0457", "ADR-0458",
    "ADR-0459", "ADR-0460", "ADR-0461", "ADR-0462", "ADR-0466", "ADR-0468", "ADR-0472", "ADR-0473",
    "ADR-0474", "ADR-0475", "ADR-0477", "ADR-0483", "ADR-0484", "ADR-0488"
];

/// Every `ADR-NNNN` token in a text (exactly four digits, not followed by a fifth digit).
/// Hand-rolled scanner (minimal-deps doctrine: no regex crate in the producer).
fn adr_citation_tokens(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let bytes = text.as_bytes();
    let mut from = 0;
    while let Some(rel) = text[from..].find("ADR-") {
        let start = from + rel;
        let digits_start = start + 4;
        let digits_end = digits_start + 4;
        if digits_end <= bytes.len()
            && bytes[digits_start..digits_end]
                .iter()
                .all(u8::is_ascii_digit)
            && !bytes.get(digits_end).is_some_and(u8::is_ascii_digit)
        {
            // SAFETY-free slice: the matched region is pure ASCII, so the str slice is valid.
            out.insert(text[start..digits_end].to_owned());
            from = digits_end;
        } else {
            from = start + 4;
        }
    }
    out
}

/// Collect the `ADR-NNNN` citation edges of the governed surfaces that resolve to NO
/// on-disk decision id, excluding the grandfathered historical inventory. Governed
/// surfaces: every decision file body, the roadmap/sequencing artifact, and the
/// masterplan's `bound_adrs` arrays. Edge key shape: `<cited-id>@<source-path>`
/// (e.g. `ADR-0397@docs/decisions/ADR-0709-general-live-apex.md`),
/// matching the GATE-1 `phantom_decision_citation` finding key.
fn collect_phantom_citations(
    known_ids: &BTreeSet<String>,
    decision_bodies: &[(String, String)],
    roadmap_path: &str,
    roadmap: &str,
    masterplan_path: &str,
    masterplan: &str,
) -> Vec<String> {
    let grandfathered: BTreeSet<&str> = GRANDFATHERED_PHANTOM_DECISION_IDS.into_iter().collect();
    let mut edges: BTreeSet<String> = BTreeSet::new();

    let record = |cited: &str, source: &str, edges: &mut BTreeSet<String>| {
        if !known_ids.contains(cited) && !grandfathered.contains(cited) {
            edges.insert(format!("{cited}@{source}"));
        }
    };

    for (source, body) in decision_bodies {
        for cited in adr_citation_tokens(body) {
            record(&cited, source, &mut edges);
        }
    }
    for cited in adr_citation_tokens(roadmap) {
        record(&cited, roadmap_path, &mut edges);
    }
    // masterplan: only the `bound_adrs` arrays are citation edges (the rest of the
    // masterplan is generated narrative; its decision binding IS bound_adrs).
    if let Ok(value) = serde_json::from_str::<Value>(masterplan) {
        let mut bound: BTreeSet<String> = BTreeSet::new();
        collect_bound_adrs(&value, &mut bound);
        let source = format!("{masterplan_path}#bound_adrs");
        for cited in bound {
            record(&cited, &source, &mut edges);
        }
    }
    edges.into_iter().collect()
}

/// Recursively collect the string items of every `bound_adrs` array in a JSON document.
fn collect_bound_adrs(value: &Value, out: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, nested) in map {
                if key == "bound_adrs"
                    && let Some(items) = nested.as_array()
                {
                    for item in items.iter().filter_map(Value::as_str) {
                        out.extend(adr_citation_tokens(item));
                    }
                }
                collect_bound_adrs(nested, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_bound_adrs(item, out);
            }
        }
        _ => {}
    }
}

fn masterplan_propagated_decisions(masterplan: &str) -> BTreeSet<String> {
    let Ok(value) = serde_json::from_str::<Value>(masterplan) else {
        return BTreeSet::new();
    };
    let mut propagated = BTreeSet::new();
    collect_bound_adrs(&value, &mut propagated);
    let lifecycle_only = value
        .pointer(
            "/masterplan_v2/accepted_decision_propagation_dispositions/decisions",
        )
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|decision| {
            decision
                .get("lifecycle_state")
                .and_then(Value::as_str)
                .is_some_and(|state| state.eq_ignore_ascii_case("accepted"))
                && decision.get("planning_impact").and_then(Value::as_bool) == Some(false)
                && decision.get("sequencing_effect").and_then(Value::as_str) == Some("none")
                && decision
                    .get("binding_plan_approval_effect")
                    .and_then(Value::as_str)
                    == Some("none")
                && decision
                    .get("execution_dispatch_effect")
                    .and_then(Value::as_str)
                    == Some("none")
                && decision.get("hold_state").and_then(Value::as_str)
                    == Some("HOLD(Planning)")
                && decision.get("disposition_ref").and_then(Value::as_str)
                    == Some(
                        "/specs/master-plan-sequencing.json#_metadata.accepted_decision_propagation_dispositions",
                    )
        })
        .filter_map(|decision| decision.get("id").and_then(Value::as_str));
    propagated.extend(lifecycle_only.map(str::to_owned));
    propagated
}

fn masterplan_propagates_decision(masterplan: &str, decision_id: &str) -> bool {
    masterplan_propagated_decisions(masterplan).contains(decision_id)
}

/// Collect the GATE-1 cross-artifact facts from the live corpus: ADR front-matter
/// (status + reciprocal supersession edges), spec/masterplan/roadmap presence, the
/// duplicate-id collision (two files carrying one id), the phantom citation edges
/// (`ADR-NNNN` cited from a governed surface with no decision file on disk), and the
/// generated-face axes drift (catalog.json vs contracts.json `axes_count`). Single pass
/// over the ADR corpus.
fn collect_crosswalk_inputs(
    repo_root: &Path,
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> CrosswalkInputs {
    let decisions_dir = repo_root.join(&cfg.justification.adr_dir);
    let masterplan = read_text(&repo_root.join(&cfg.reachability.masterplan));
    let roadmap = read_text(&repo_root.join(&cfg.justification.roadmap));

    // id -> the decision files carrying it (dup detection is files-per-id > 1).
    let mut files_by_id: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    // Filename-vs-front-matter id disagreements. Because the dup map keys by the
    // front-matter id when present, a mismatched front-matter id silently re-keys the
    // file and can MASK a filename-number collision (the FRIC-1781320000 parallel-lane
    // shape); each mismatch is therefore surfaced as its own crosswalk signal.
    let mut id_mismatches: Vec<String> = Vec::new();
    // Every id ANY decision file carries (filename or front-matter) — the resolution
    // universe for phantom-citation detection (FRIC-1781430000): a citation resolves iff
    // some on-disk decision file carries the cited id under either identity.
    let mut known_ids: BTreeSet<String> = BTreeSet::new();
    // The governed decision bodies, kept for the citation scan (one read per file).
    let mut decision_bodies: Vec<(String, String)> = Vec::new();
    // Live decisions only contribute crosswalk decision rows; the historical archive
    // still contributes known_ids + citation bodies so apex supersedes edges resolve.
    for corpus_dir in adr_corpus_dirs(repo_root, cfg) {
        let rel = corpus_dir
            .strip_prefix(repo_root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| corpus_dir.display().to_string());
        let is_live = corpus_dir == decisions_dir;
        if let Ok(entries) = std::fs::read_dir(&corpus_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Some(filename_id) = adr_id_from_filename(&name) {
                    let path = entry.path();
                    let body = read_text(&path);
                    let front_id = front_matter_field(&body, "id");
                    decision_bodies.push((format!("{rel}/{name}"), body));
                    known_ids.insert(filename_id.clone());
                    if let Some(front_id) = &front_id {
                        known_ids.insert(front_id.clone());
                        if is_live && front_id != &filename_id {
                            id_mismatches.push(format!("{name}:{filename_id}!={front_id}"));
                        }
                    }
                    if is_live {
                        let id = front_id.unwrap_or(filename_id);
                        files_by_id.entry(id).or_default().push(path);
                    }
                }
            }
        }
    }
    id_mismatches.sort();
    decision_bodies.sort_by(|a, b| a.0.cmp(&b.0));
    // The SINGLE allocator (slice 2.5): the next free decision number is derived by the
    // library's `allocate_next_adr_id` (max over filename AND front-matter ids, plus one),
    // not re-implemented here — so `--next-adr`, this crosswalk pass, and the slice-3
    // register_crate app all share one allocator with no duplication. Infallible in
    // practice (a missing dir yields ADR-0001), but propagated for a uniform contract.
    let next_free_id =
        allocate_next_adr_id(&decisions_dir).unwrap_or_else(|_| "ADR-0001".to_owned());

    let phantom_citations = collect_phantom_citations(
        &known_ids,
        &decision_bodies,
        &cfg.justification.roadmap,
        &roadmap,
        &cfg.reachability.masterplan,
        &masterplan,
    );
    let propagated_masterplan_decisions = masterplan_propagated_decisions(&masterplan);

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
        let in_masterplan = propagated_masterplan_decisions.contains(id);
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
    if let Some(value) = json_number_at(
        repo_root,
        "docs/machine-readable/catalog.json",
        &["_metadata", "axes_count"],
    ) {
        generated_face_axes.insert("catalog.json".into(), value);
    }
    if let Some(value) = json_number_at(
        repo_root,
        "docs/machine-readable/contracts.json",
        &["_metadata", "axes_count"],
    ) {
        generated_face_axes.insert("contracts.json".into(), value);
    }

    CrosswalkInputs {
        decisions,
        duplicate_ids,
        id_mismatches,
        phantom_citations,
        grandfathered_phantom_ids: GRANDFATHERED_PHANTOM_DECISION_IDS
            .iter()
            .map(|id| (*id).to_owned())
            .collect(),
        next_free_id,
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

fn oya_cli_authority_row_kind(
    trimmed: &str,
    previous: &str,
    next: &str,
    enforcement_status_is_blocking: bool,
) -> Option<&'static str> {
    let line_lower = trimmed.to_ascii_lowercase();
    let previous_lower = previous.to_ascii_lowercase();
    let next_lower = next.to_ascii_lowercase();
    let previous_shares_record = shares_markdown_logical_record(previous, trimmed);
    let next_shares_record = shares_markdown_logical_record(trimmed, next);
    let context_lower = format!(
        "{} {line_lower} {}",
        if previous_shares_record {
            previous_lower.as_str()
        } else {
            ""
        },
        if next_shares_record {
            next_lower.as_str()
        } else {
            ""
        }
    );
    let line_mentions_cli = mentions_oya_cli(&line_lower);
    let context_mentions_cli = mentions_oya_cli(&context_lower);
    if !context_mentions_cli {
        return None;
    }

    let line_is_explicit_non_authority = cli_reference_is_explicit_non_authority(&line_lower);
    let adjacent_is_explicit_non_authority = [
        (previous_shares_record, previous_lower.as_str()),
        (next_shares_record, next_lower.as_str()),
    ]
    .iter()
    .any(|(shares_record, line)| *shares_record && cli_reference_is_explicit_non_authority(line));
    if line_is_explicit_non_authority
        || (line_mentions_cli
            && adjacent_is_explicit_non_authority
            && (line_lower.contains("legacy") || line_lower.starts_with("until ")))
    {
        return None;
    }

    let current_is_yaml_list_item = trimmed.starts_with("- ");
    let verified_by_context = line_lower.contains("verified_by:")
        || (line_mentions_cli && current_is_yaml_list_item && previous_lower == "verified_by:");
    let enforced_by_context = line_lower.contains("enforced_by:")
        || (line_mentions_cli && current_is_yaml_list_item && previous_lower == "enforced_by:");
    if line_mentions_cli && verified_by_context {
        return Some("verified_by");
    }
    if line_mentions_cli && enforcement_status_is_blocking && enforced_by_context {
        return Some("enforced_by");
    }

    let line_claims_authority = cli_reference_claims_live_authority(&line_lower);
    let adjacent_live_cli = [
        (previous_shares_record, previous_lower.as_str()),
        (next_shares_record, next_lower.as_str()),
    ]
    .iter()
    .any(|(shares_record, line)| {
        *shares_record && mentions_oya_cli(line) && !cli_reference_is_explicit_non_authority(line)
    });
    if !line_mentions_cli && !(line_claims_authority && adjacent_live_cli) {
        return None;
    }
    if cli_reference_claims_live_authority(&context_lower) {
        return Some("cli-authority");
    }
    None
}

fn shares_markdown_logical_record(left: &str, right: &str) -> bool {
    let left = left.trim();
    let right = right.trim();
    if left.is_empty() || right.is_empty() {
        return false;
    }
    if is_markdown_table_row(left) || is_markdown_table_row(right) {
        return false;
    }
    if is_markdown_heading(left) || is_markdown_heading(right) {
        return false;
    }
    if is_markdown_thematic_break(left) || is_markdown_thematic_break(right) {
        return false;
    }
    if is_fenced_code_marker(left) || is_fenced_code_marker(right) {
        return false;
    }
    if starts_markdown_list_item(left) && starts_markdown_list_item(right) {
        return false;
    }
    if looks_like_front_matter_key(left) || looks_like_front_matter_key(right) {
        return false;
    }
    true
}

fn is_markdown_table_row(line: &str) -> bool {
    line.starts_with('|')
}

fn is_markdown_heading(line: &str) -> bool {
    line.starts_with('#')
}

fn is_fenced_code_marker(line: &str) -> bool {
    line.starts_with("```") || line.starts_with("~~~")
}

fn is_markdown_thematic_break(line: &str) -> bool {
    matches!(line, "---" | "***" | "___")
}

fn starts_markdown_list_item(line: &str) -> bool {
    line.starts_with("- ")
        || line.starts_with("* ")
        || line.split_once('.').is_some_and(|(prefix, suffix)| {
            !prefix.is_empty()
                && prefix.chars().all(|ch| ch.is_ascii_digit())
                && suffix.starts_with(' ')
        })
}

fn looks_like_front_matter_key(line: &str) -> bool {
    let Some((key, _)) = line.split_once(':') else {
        return false;
    };
    !key.is_empty()
        && key
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}

fn mentions_oya_cli(lower: &str) -> bool {
    [
        "oya gate",
        "oya gen",
        "oya verify",
        "oya check",
        "oya-dev-cli",
        "./bin/oya",
        "bin/oya",
        "cargo run -p oya-dev-cli",
        "cargo run -q -p oya-dev-cli",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn cli_reference_is_explicit_non_authority(lower: &str) -> bool {
    [
        "bridge evidence only",
        "bridge-only",
        "bridge/local",
        "legacy/local",
        "local feedback",
        "transitional/local feedback",
        "transitional local feedback",
        "local/bridge",
        "migration evidence",
        "migration evidence only",
        "migration wrapper",
        "migration wrappers",
        "never merge authority",
        "not merge authority",
        "cannot be merge",
        "cannot be the merge",
        "cannot be promotion",
        "cannot be the promotion",
        "never promotion authority",
        "not promotion authority",
        "cloud-ci/rust gate authority",
        "cloud-ci rust gate authority",
        "provenance only",
        "only as legacy",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn cli_reference_claims_live_authority(lower: &str) -> bool {
    [
        "merge authority",
        "promotion authority",
        "exit authority",
        "protected-branch authority",
        "required context",
        "required status",
        "require it before promotion",
        "required before promotion",
        "blocks merge",
        "blocking invariant",
        "gate of record",
        "refuses merge",
        "refuse merge",
        "rejects merge",
        "enforces",
        "ci gate",
        "ci lane",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn adr_is_live_for_cli_authority_scan(body: &str) -> bool {
    let status = front_matter_field(body, "status")
        .unwrap_or_default()
        .to_ascii_lowercase();
    !status.contains("superseded")
        && !status.contains("retired")
        && !status.contains("withdrawn")
        && !front_matter_has_inline_or_block_list(body, "superseded_by")
}

fn adr_enforcement_status_is_blocking(body: &str) -> bool {
    let status = front_matter_field(body, "enforcement_status")
        .unwrap_or_default()
        .to_ascii_lowercase();
    status.contains("active") || status.contains("blocker") || status.contains("blocking")
}

/// Collect the GATE-4 enforcement surfaces from the live corpus: the governance kernel
/// crates (claim "enforce" by name; wired only if a BUCK gate target exists), the
/// governance lanes (diataxis-doc-class / prd-axis-coverage), and ADR lines that route a
/// blocking invariant through an `oya gate`/`oya gen`/`oya verify`/`oya-dev-cli` CLI call.
fn collect_enforcement_inputs(
    repo_root: &Path,
    cfg: &oya_ci_config_kernel::OyaCiConfig,
    scm_facts: &ScmFacts,
) -> EnforcementInputs {
    let mut rows: Vec<EnforcementRow> = Vec::new();
    let governance_substr = cfg.enforcement.governance_crate_substr.clone();

    // (1) oya-governance-* kernel crates: they name themselves "governance" enforcers,
    // but none is wired into the cloud-ci gate build graph (no gate BUCK target backs them).
    for cargo in tracked_paths_matching(scm_facts, |p| {
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
            ..EnforcementRow::default()
        });
    }

    // (2) Governance lanes that claim a doc/coverage class is enforced.
    for lane in &cfg.enforcement.governance_lanes {
        let lane = lane.as_str();
        if repo_root.join(lane).is_file() {
            let id = lane
                .rsplit('/')
                .next()
                .unwrap_or(lane)
                .trim_end_matches(".md");
            rows.push(EnforcementRow {
                id: format!("governance-lane:{id}"),
                source_artifact: lane.to_owned(),
                claims_enforced: true,
                has_wired_buck2_target: false,
                maps_to_oya_cli: false,
                ..EnforcementRow::default()
            });
        }
    }

    collect_review_authority_row(repo_root, &mut rows);

    // (3) ADR `verified_by:` lines and live-authority prose that name an `oya` CLI invocation
    // (ADR-0365's retired CLI authority). Bridge/history/local-feedback mentions are evidence,
    // not authority; live CLI-as-merge-authority claims remain blocking inventory rows.
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
            if !adr_is_live_for_cli_authority_scan(&body) {
                continue;
            }
            let enforcement_status_is_blocking = adr_enforcement_status_is_blocking(&body);
            let lines: Vec<&str> = body.lines().collect();
            for (index, line) in lines.iter().enumerate() {
                let line_no = index as u64 + 1;
                let trimmed = line.trim();
                let previous = index
                    .checked_sub(1)
                    .and_then(|previous| lines.get(previous))
                    .map_or("", |line| line.trim());
                let next = lines.get(index + 1).map_or("", |line| line.trim());
                if let Some(row_kind) = oya_cli_authority_row_kind(
                    trimmed,
                    previous,
                    next,
                    enforcement_status_is_blocking,
                ) {
                    rows.push(EnforcementRow {
                        id: format!("{adr}-{row_kind}-L{line_no}"),
                        source_artifact: rel.clone(),
                        claims_enforced: true,
                        has_wired_buck2_target: false,
                        maps_to_oya_cli: true,
                        ..EnforcementRow::default()
                    });
                }
            }
        }
    }

    EnforcementInputs { rows }
}

/// Surface the checked-in branch-protection target as a GATE-4 review-authority gap row. The
/// current file is target/shadow state, not live authority, so it can identify the requirement but
/// must not satisfy the requirement. This row is deliberately born-blocking until a live
/// review-status/PR-review evidence producer supplies durable, distinct-reviewer authority.
fn collect_review_authority_row(repo_root: &Path, rows: &mut Vec<EnforcementRow>) {
    let rel = "infra/branch-protection/dev.json";
    let path = repo_root.join(rel);
    if !path.is_file() {
        return;
    }

    rows.push(EnforcementRow {
        id: "branch-protection:dev-pre-merge-review-authority".to_owned(),
        source_artifact: rel.to_owned(),
        claims_enforced: true,
        has_wired_buck2_target: true,
        maps_to_oya_cli: false,
        requires_pre_merge_review_authority: true,
        review_authority_live: false,
        review_authority_source: "target_branch_protection_shadow_only".to_owned(),
        has_durable_review_evidence: false,
        has_machine_verifiable_review_status: false,
        binds_pr_number: false,
        binds_head_sha: false,
        binds_author_identity: false,
        binds_reviewer_identity: false,
        binds_review_verdict: false,
        review_blocks_merge: false,
        reviewer_identity_distinct_from_author: false,
    });
}

/// Tracked paths matching a predicate (filtered from the declared scm-facts tracked-paths).
fn tracked_paths_matching(scm_facts: &ScmFacts, pred: impl Fn(&str) -> bool) -> Vec<String> {
    scm_facts
        .tracked_paths
        .iter()
        .filter(|p| pred(p))
        .cloned()
        .collect()
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

fn front_matter_has_inline_or_block_list(body: &str, key: &str) -> bool {
    let lines = front_matter_lines(body);
    let key_prefix = format!("{key}:");
    for (index, line) in lines.iter().enumerate() {
        if let Some(rest) = line.strip_prefix(&key_prefix) {
            let trimmed = rest.trim();
            if let Some(inner) = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                return inner
                    .split(',')
                    .any(|x| !x.trim().trim_matches('"').is_empty());
            }
            if trimmed.is_empty() {
                return lines[index + 1..]
                    .iter()
                    .map(|line| line.trim())
                    .take_while(|line| line.starts_with("- "))
                    .any(|line| {
                        !line
                            .trim_start_matches("- ")
                            .trim()
                            .trim_matches('"')
                            .is_empty()
                    });
            }
            return true;
        }
    }
    false
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

/// Collect the real repo facts. The tracked-paths universe comes from the declared stable
/// scm-facts face (no ambient git, no history-derived data — ADR-0552); the
/// owner/justification/reachability maps are derived from the declared repo sources.
/// Fallible because the reachability registry is fail-loud (ADR-0555): a malformed
/// registration file must never silently degrade to "everything unreachable" nor "nothing
/// registered".
fn collect_repo_inputs(
    repo_root: &Path,
    cfg: &oya_ci_config_kernel::OyaCiConfig,
    scm_facts: &ScmFacts,
) -> Result<(RepoInputs, OwnersIntegrity), CliError> {
    let tracked_paths: Vec<String> = scm_facts
        .tracked_paths
        .iter()
        .filter(|path| !is_path_excluded(path, cfg))
        .cloned()
        .collect();
    let owners_resolution = resolve_owners(repo_root, &tracked_paths, cfg);
    let reachability = resolve_reachability(repo_root, &tracked_paths, cfg)?;
    let justifications = resolve_justifications(repo_root, &tracked_paths, cfg);

    Ok((
        RepoInputs {
            tracked_paths,
            owners: owners_resolution.by_path,
            justifications,
            reachability,
            dup_of: BTreeMap::new(),
            valid_owners_files: owners_resolution.valid_files,
        },
        owners_resolution.integrity,
    ))
}

/// Reachability: a path is reachable if a live registry points at it. We resolve the
/// real registries (masterplan.json / root-hub-pointers.json / Cargo.toml members /
/// DOC-CATALOG / the reviewed reachability registry / envelope `envelope_globs` prefixes)
/// and mark each tracked path with the registries that mention it.
///
/// Envelope prefix ownership (admission.policy / path_ownership law): paths under an owned
/// `roots.*.envelope_globs` prefix (e.g. `compute/**` → `compute/`) are in-domain — they MUST
/// NOT require per-file tip-free / reachability-registry rows. Source tag:
/// [`ENVELOPE_PREFIX_OWNERSHIP_SOURCE`].
///
/// The three TEXT registries are matched by whole path token, NOT by substring. A registry
/// reaches a path when it NAMES the path; `masterplan.contains("OWNERS")` is also true of
/// `docs/OWNERS-policy.md`, of the JSON key `"OWNERS"`, and of the bare word in prose — so
/// every short path (and every path that is a prefix of a longer one) was reported reachable
/// by coincidence. That is a fail-OPEN accounting error: the whole point of the `unreachable`
/// firewall code is that nothing lives in the tree without a live registry pointing at it.
fn resolve_reachability(
    repo_root: &Path,
    paths: &[String],
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> Result<BTreeMap<String, Vec<String>>, CliError> {
    let masterplan = read_text(&repo_root.join(&cfg.reachability.masterplan));
    let root_hub = read_text(&repo_root.join(&cfg.reachability.root_hub));
    let doc_catalog = read_text(&repo_root.join(&cfg.reachability.doc_catalog));
    let masterplan = mentioned_path_index(&masterplan);
    let root_hub = mentioned_path_index(&root_hub);
    let doc_catalog = mentioned_path_index(&doc_catalog);
    let cargo_members = read_cargo_member_prefixes(repo_root)?;
    let registrations = load_reachability_registry(&repo_root.join(&cfg.reachability.registry))?;
    let envelope_prefixes = load_envelope_prefix_allows(&repo_root.join(ENVELOPES_RELPATH))?;

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
        if registrations
            .iter()
            .any(|entry| registration_matches(path, &entry.prefix))
        {
            reach.push("reachability-registry".into());
        }
        if envelope_prefixes
            .iter()
            .any(|entry| registration_matches(path, &entry.prefix))
        {
            reach.push(ENVELOPE_PREFIX_OWNERSHIP_SOURCE.into());
        }
        if !reach.is_empty() {
            map.insert(path.clone(), reach);
        }
    }
    Ok(map)
}

/// The repo-relative path-like tokens a corpus document names, in document order.
///
/// This is the tokenizer `resolve_justifications` has always used for the ADR corpus, lifted
/// so reachability can share it: split on whitespace and the JSON/markdown delimiters that
/// surround a path (`"`, backtick, parens, comma, semicolon, brackets), then trim the
/// leader/trailer punctuation a path never carries (`:`, `#`, `*`, and a sentence-final `.`).
/// Callers apply their own membership test; nothing here is filtered by length, so
/// `resolve_justifications` keeps its exact `len() >= 4 && tracked.contains(..)` behaviour.
fn path_like_tokens(body: &str) -> impl Iterator<Item = &str> {
    body.split(|c: char| {
        c.is_whitespace() || matches!(c, '"' | '`' | '(' | ')' | ',' | ';' | '[' | ']')
    })
    .map(|raw| {
        raw.trim_matches(|c: char| matches!(c, ':' | '#' | '*'))
            .trim_end_matches('.')
    })
}

/// The set of repo-relative paths a registry document MENTIONS — the exact-match index that
/// replaces the old `registry_text.contains(path)` substring probe in [`resolve_reachability`].
///
/// Two normalizations the ADR corpus does not need, both measured against the live registries
/// rather than assumed:
///
/// 1. A leading `/` is stripped. The repo's spec surface spells root-anchored references as
///    `/specs/root-hub-pointers.json` (see `CLAUDE.md`, `masterplan.json`
///    `live_gate_input_refs`) while the tracked universe spells the same file
///    `specs/root-hub-pointers.json`.
/// 2. A `#fragment` suffix is cut. `masterplan.json` names most of its evidence anchors as
///    `<path>#<symbol>` — `/infra/branch-protection/dev.json#required_status_checks`,
///    `ci/facade/cross-artifact-agreement/src/lib.rs#evaluate_masterplan_v2_projection_freshness`,
///    `governance/capability-registry.json#meta_directories[kernel/]`. Those ARE references to the
///    file; only the deep link is extra.
///
/// The old substring test matched both shapes by accident. An exact test has to normalize them
/// on purpose — without this, the fix would report ~7 genuinely-registered spec/gate files as
/// newly unreachable, trading a fail-open bug for a fail-closed one.
fn mentioned_path_index(body: &str) -> BTreeSet<&str> {
    path_like_tokens(body)
        .map(|token| {
            let token = token.trim_start_matches('/');
            token.split_once('#').map_or(token, |(path, _fragment)| path)
        })
        .filter(|token| !token.is_empty())
        .collect()
}

/// Member directory prefixes from the workspace Cargo.toml — a path under a member
/// crate dir is reachable from `cargo-members`. Resolved via the canonical
/// `oya-workspace-members-kernel` (reuse, not re-derive): the root manifest lists
/// members as GLOBS (`libs/oya-*`, `cloud/*/crates/oya-*`, ...), so the member set MUST
/// be expanded against the tree; a textual read of the array would only see the `*`
/// literals and would mark every crate path unreachable from `cargo-members`.
///
/// FAIL-CLOSED, mirroring the `read_dir` NotFound-vs-other distinction: a tree with NO root
/// Cargo.toml at all (a fixture with no cargo workspace) tolerates "zero cargo-members"; a root
/// Cargo.toml that EXISTS but fails to resolve (malformed TOML, missing `[workspace]` shape)
/// must propagate, not silently resolve to "zero members" — that would mark every crate path
/// unreachable from `cargo-members` and could misclassify real crates as orphaned instead of
/// erroring loud on a genuinely corrupt manifest. Missing member manifests are retained by the
/// diagnostic scan and emitted through the workspace-glob-coverage face, so reachability can be
/// produced without hiding or short-circuiting that blocking finding.
fn read_cargo_member_prefixes(repo_root: &Path) -> Result<Vec<String>, CliError> {
    if !repo_root.join("Cargo.toml").is_file() {
        return Ok(Vec::new());
    }
    Ok(scan_valid_member_dirs(repo_root, "cargo-members")?
        .into_iter()
        .map(|dir| format!("{dir}/"))
        .collect())
}

/// Justification: a path traces to a decision if an ADR mentions it (front-matter
/// `affected_surfaces` / body refs) or it lives under a decision-owned tree. Resolved
/// from the real ADR corpus.
///
/// Built as a single pass over the ADR corpus (NOT O(paths x ADRs)): each ADR body is
/// tokenized once into the repo-relative path-like tokens it references, populating a
/// `token -> first ADR id` index. Per-path lookup is then an O(1) map hit.

/// Live decisions dir plus the historical ADR archive (when present).
/// Archive is outside the P3 direct-child census root but still supplies
/// path-justification tokens and known decision ids for phantom resolution.
fn adr_corpus_dirs(repo_root: &Path, cfg: &oya_ci_config_kernel::OyaCiConfig) -> Vec<PathBuf> {
    let mut dirs = vec![repo_root.join(&cfg.justification.adr_dir)];
    let archive = repo_root.join("docs/adr-archive");
    if archive.is_dir() {
        dirs.push(archive);
    }
    dirs
}

fn resolve_justifications(
    repo_root: &Path,
    paths: &[String],
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> BTreeMap<String, String> {
    let tracked: BTreeSet<&str> = paths.iter().map(String::as_str).collect();
    // token (a tracked path mentioned in an ADR) -> first ADR id mentioning it.
    let mut mentioned: BTreeMap<String, String> = BTreeMap::new();
    let mut adr_files: Vec<PathBuf> = Vec::new();
    for decisions_dir in adr_corpus_dirs(repo_root, cfg) {
        if let Ok(entries) = std::fs::read_dir(&decisions_dir) {
            for entry in entries.flatten() {
                if adr_id_from_filename(&entry.file_name().to_string_lossy()).is_some() {
                    adr_files.push(entry.path());
                }
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
        for token in path_like_tokens(&body) {
            if token.len() >= 4 && tracked.contains(token) {
                mentioned
                    .entry(token.to_owned())
                    .or_insert_with(|| adr_id.clone());
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

/// One added path's pre-push verdict (FRIC #1328).
struct AddedPathVerdict {
    path: String,
    unit_class: String,
    /// Excluded by `[repo].path_excludes` ⇒ never enters the accounting universe.
    excluded: bool,
    /// The ADR id that justifies the path, or `None` ⇒ would be `unjustified`.
    justification: Option<String>,
    /// The registries that reach the path; empty ⇒ would be `unreachable`.
    reachable_from: Vec<String>,
    /// The firewall codes this NEW path would introduce as regressions. Owner-independent:
    /// `unowned` is dropped (see [`check_added_paths`]).
    blocking_codes: BTreeSet<String>,
}

/// Added tracked files between `merge_base` and `HEAD` (`git diff --diff-filter=A`). The
/// author-side convenience input for `--check-diff`; the pre-push flow commits new files, then
/// runs this. Fail-loud: an unknown/unfetched base is a hard error, never a silent empty set.
fn git_added_paths(repo_root: &Path, merge_base: &str) -> Result<Vec<String>, CliError> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["diff", "--name-only", "--diff-filter=A", merge_base, "HEAD"])
        .output()
        .map_err(|e| CliError::Io(format!("git diff (added paths): {e}")))?;
    if !output.status.success() {
        return Err(CliError::Io(format!(
            "git diff --name-only --diff-filter=A {merge_base} HEAD failed (exit {:?}): {} — \
             fetch the base ref or pass a reachable merge-base",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    // `git diff --name-only` C-quotes the same pathnames `ls-files` does, so this is the SECOND
    // ingestion boundary and takes the SAME decode — otherwise the author-side check silently
    // disagrees with the full gate on exactly the paths that need it most.
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(decode_tracked_path)
        .collect()
}

/// Resolve the firewall verdict for a set of ADDED paths, reusing the EXACT producer
/// resolvers + face-builder + the firewall's own evaluator — no reimplementation, no drift.
///
/// The added set IS the tracked-path universe here, so no materialized scm-facts face is
/// needed (that is why authors miss the failure): `resolve_justifications` /
/// `resolve_reachability` read the ADR corpus + registries straight from the working tree.
/// A NEW path is never grandfathered by the merge-base baseline, so any per-row finding on it
/// is guaranteed to be a regression — the check needs no baseline.
///
/// `unowned` is intentionally not reported: owner resolution is FULL-TREE (the granting up-tree
/// `OWNERS` file is usually not in the added set), so it is not soundly computable from a
/// partial set. The full gate owns ownership; this check covers justification + reachability
/// (+ the path-only `scratch_artifact` / `no_ttl_class` classes).
fn check_added_paths(
    repo_root: &Path,
    cfg: &oya_ci_config_kernel::OyaCiConfig,
    policy: &Policy,
    paths: &[String],
) -> Result<Vec<AddedPathVerdict>, CliError> {
    // Excluded paths never enter the scm-facts tracked universe; split them out with the SAME
    // predicate the producer applies so the check matches CI's accounting boundary exactly.
    let accounted: Vec<String> = paths
        .iter()
        .filter(|path| !is_path_excluded(path, cfg))
        .cloned()
        .collect();

    let justifications = resolve_justifications(repo_root, &accounted, cfg);
    let reachability = resolve_reachability(repo_root, &accounted, cfg)?;
    // The OWNERS accounting floor is derived here too, or this author-side check would
    // report WOULD RED for a newly-added valid OWNERS file that CI then passes — the exact
    // false alarm that makes a pre-push check untrustworthy. Unlike OWNER resolution (which
    // is full-tree and therefore unsound on a partial set, see the doc comment above), the
    // per-file SCHEMA verdict is locally computable: the added OWNERS file is itself in the
    // set and is read + parsed straight from the working tree. Only `valid_files` is taken;
    // `by_path` stays empty so `unowned` remains out of scope exactly as before.
    let valid_owners_files = resolve_owners(repo_root, &accounted, cfg).valid_files;

    // Route the added rows through the producer's OWN face-builder and the firewall's OWN
    // evaluator: the unjustified/unreachable/scratch/ttl verdicts are byte-identical to CI.
    let inputs = RepoInputs {
        tracked_paths: accounted,
        owners: BTreeMap::new(),
        justifications: justifications.clone(),
        reachability: reachability.clone(),
        dup_of: BTreeMap::new(),
        valid_owners_files,
    };
    let registry = build_registry(&inputs, policy)?;
    let mut codes_by_key: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for finding in ci_artifact_accountability::evaluate_keyed(&registry) {
        if finding.code == "unowned" {
            continue;
        }
        codes_by_key
            .entry(finding.key)
            .or_default()
            .insert(finding.code);
    }

    // Report the values off the BUILT ROWS, not off the raw resolver maps. The rows are what
    // `evaluate_keyed` just judged, so the printed "justified by X · reachable via Y" columns
    // and the WOULD-RED verdict cannot disagree — a row carrying a DERIVED accounting source
    // (the OWNERS floor, or the reached ⇒ justified rule) would otherwise print "justified by
    // NO · reachable via UNREACHABLE" directly above an `OK` line. Both columns must come from
    // the row for that to hold: reading `justification_ref` back but re-reading the raw
    // reachability map still lets an OWNERS-floor row print `UNREACHABLE` beside `OK`.
    // Re-deriving either rule here instead would duplicate it and let the two copies drift;
    // reading it back cannot. Paths with no row (excluded / unit_class ephemeral) fall back to
    // the resolver maps and print their own lines anyway.
    let mut row_accounting: BTreeMap<String, (Option<String>, Vec<String>)> = BTreeMap::new();
    for row in registry["rows"].as_array().into_iter().flatten() {
        let Some(path) = row["path"].as_str() else {
            continue;
        };
        row_accounting.insert(
            path.to_owned(),
            (
                row["justification_ref"].as_str().map(str::to_owned),
                row["reachable_from"]
                    .as_array()
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(serde_json::Value::as_str)
                            .map(str::to_owned)
                            .collect()
                    })
                    .unwrap_or_default(),
            ),
        );
    }

    Ok(paths
        .iter()
        .map(|path| {
            let excluded = is_path_excluded(path, cfg);
            let (justification, reachable_from) = row_accounting.get(path).cloned().unwrap_or_else(
                || {
                    (
                        justifications.get(path).cloned(),
                        reachability.get(path).cloned().unwrap_or_default(),
                    )
                },
            );
            AddedPathVerdict {
                unit_class: policy.classify(path).to_owned(),
                justification,
                reachable_from,
                blocking_codes: if excluded {
                    BTreeSet::new()
                } else {
                    codes_by_key.get(path).cloned().unwrap_or_default()
                },
                excluded,
                path: path.clone(),
            }
        })
        .collect())
}

/// The exact, actionable remediation for an `unjustified` added path (FRIC #1328). Since
/// `build_registry` treats REACHED as justified, this code now fires only on a path that is
/// ALSO unreachable — so registering reachability is the paved road and clears BOTH codes at
/// once. Writing the path into ADR prose is the fallback for an artifact no live registry can
/// reach. Extracted so the tests can pin the author-facing text.
fn unjustified_remediation(path: &str) -> String {
    format!(
        "register `{path}` in a live reachability registry (masterplan / root-hub-pointers / \
         DOC-CATALOG / the reviewed reachability-registry / an owned envelope_globs prefix in \
         {ENVELOPES_RELPATH}), or land it under a workspace Cargo member — a reached path is \
         justified by the registry that reaches it, so this clears `unreachable` too. In-domain \
         paths under envelope prefixes need no per-file tip-free row. Only if NO live registry \
         can reach it, add the exact path token `{path}` to the governing ADR under \
         docs/decisions/ — precedent: ADR-0515 for ci/ gate surfaces, ADR-0251 for compliance \
         artifacts"
    )
}

/// Print the per-path pre-push report and return whether the added set is clean (no path would
/// RED `[cloud-ci-total-accounting]`). Remediation is exact and actionable per code.
fn report_check(verdicts: &[AddedPathVerdict]) -> bool {
    let mut clean = true;
    for verdict in verdicts {
        if verdict.excluded {
            println!(
                "{}: OK — excluded by [repo].path_excludes; outside the accounting universe",
                verdict.path
            );
            continue;
        }
        if verdict.unit_class == "ephemeral" {
            println!(
                "{}: OK — unit_class=ephemeral; carved out of the registry (no row, cannot RED)",
                verdict.path
            );
            continue;
        }
        let reach = if verdict.reachable_from.is_empty() {
            "UNREACHABLE".to_owned()
        } else {
            verdict.reachable_from.join(",")
        };
        let justified = verdict.justification.as_deref().unwrap_or("NO");
        if verdict.blocking_codes.is_empty() {
            println!(
                "{}: OK — justified by {justified} · reachable via {reach}",
                verdict.path
            );
            continue;
        }
        clean = false;
        let codes: Vec<&str> = verdict.blocking_codes.iter().map(String::as_str).collect();
        println!(
            "{}: WOULD RED [cloud-ci-total-accounting] — {} (reachable via {reach} · justified: {justified})",
            verdict.path,
            codes.join(", ")
        );
        if verdict.blocking_codes.contains("unjustified") {
            println!(
                "    fix (unjustified): {}",
                unjustified_remediation(&verdict.path)
            );
        }
        if verdict.blocking_codes.contains("unreachable") {
            println!(
                "    fix (unreachable): register `{}` in a live reachability registry (masterplan / \
                 root-hub-pointers / DOC-CATALOG / the reviewed reachability-registry), land it \
                 under a workspace Cargo member, OR place it under an owned envelope_globs \
                 prefix in {ENVELOPES_RELPATH} (in-domain — no per-file tip-free required)",
                verdict.path
            );
        }
        if verdict.blocking_codes.contains("scratch_artifact") {
            println!(
                "    fix (scratch_artifact): `{}` matches a build/test scratch shape \
                 (unit-class-policy) — zero-tolerance, never grandfathered; relocate/rename it out \
                 of the scratch class",
                verdict.path
            );
        }
    }
    if clean {
        println!(
            "check: OK — {} added path(s); none would RED [cloud-ci-total-accounting]",
            verdicts.len()
        );
    } else {
        println!(
            "check: WOULD RED — fix the paths above, then re-run --check-paths before pushing"
        );
    }
    clean
}

fn read_text(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

fn read_required_text(path: &Path, label: &str) -> Result<String, CliError> {
    std::fs::read_to_string(path).map_err(|e| {
        CliError::Io(format!(
            "{label}: read declared enforcement-liveness input {}: {e}",
            path.display()
        ))
    })
}

/// Whether a tracked path is excluded by the config's `[repo].path_excludes` (the producer's
/// `collect_*` filter). Reproduces the legacy `third-party/` semantics exactly: a path is
/// excluded iff, for some configured prefix P, it `starts_with(P)` OR `contains("/" + P)`
/// (so both a top-level `third-party/...` and a nested `.../third-party/...` are caught).
fn is_path_excluded(path: &str, cfg: &oya_ci_config_kernel::OyaCiConfig) -> bool {
    cfg.repo
        .path_excludes
        .iter()
        .any(|prefix| path.starts_with(prefix.as_str()) || path.contains(&format!("/{prefix}")))
}
