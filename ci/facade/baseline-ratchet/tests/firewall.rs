// cloud-ci-firewall — the single required GO-LIVE status check. Regenerates the gate
// baseline over the LIVE tree, loads the FROZEN merge-base reference (the gate-baseline
// face at `git merge-base <base_ref> HEAD`, materialized out-of-graph by the scm-facts
// emitter — ADR-0551, fixes FRIC-1781112000) + the sign-off door, and runs both pure
// predicates (compare-mode + ratchet-invariant). The reference is NEVER the PR-local face:
// the settle protocol mandates regeneration and registry-drift mandates
// committed==regenerated, so a PR-local reference is grown by the very regen the protocol
// requires (the PR #670 laundering exhibit — pinned below as a foil). This is the proof
// that, with the baseline frozen at the merge-base, the firewall is GREEN on the current
// corpus (no NEW debt) yet still blocks any NEW finite violation.
// ADR-0083 Tier-3: integration tests use unwrap/expect to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ci_baseline_ratchet::{
    Baseline, FROZEN_SNAPSHOT_PATH, FrozenBaseline, RATCHET_POLICY_PATH, SIGNOFF_FIXER_COMMAND,
    SIGNOFF_PATH, SignOff, baseline_keys_map, evaluate_firewall, ratchet_growth,
    relabel_baseline_for_renames,
};
use serde_json::Value;

const ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS_ENV: &str =
    "OYA_CI_ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS";
const ENFORCEMENT_LIVENESS_CODEX_HOOKS_ENV: &str = "OYA_CI_ENFORCEMENT_LIVENESS_CODEX_HOOKS";
const ENFORCEMENT_LIVENESS_HOOKS_DIR_ENV: &str = "OYA_CI_ENFORCEMENT_LIVENESS_HOOKS_DIR";
const ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS: &str = ".claude/settings.json";
const ENFORCEMENT_LIVENESS_CODEX_HOOKS: &str = ".codex/hooks.json";
const ENFORCEMENT_LIVENESS_HOOKS_DIR: &str = "tools/hooks";

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root from test current_dir");
}

fn faces_dir(root: &Path) -> PathBuf {
    root.join("ci/facade/artifact-inventory-registry")
}

// The firewall-side file paths are lib constants (single owner) shared with the signoff
// fixer — gate and fixer can never disagree about which files they police.
fn signoff_path(root: &Path) -> PathBuf {
    root.join(SIGNOFF_PATH)
}

fn frozen_snapshot_path(root: &Path) -> PathBuf {
    root.join(FROZEN_SNAPSHOT_PATH)
}

fn ratchet_policy_path(root: &Path) -> PathBuf {
    root.join(RATCHET_POLICY_PATH)
}

/// Detect `old_path -> new_path` renames between the merge-base and the working tree,
/// using git's own similarity detection (`--find-renames`).
///
/// This is the I/O half of the rename relabel; the decision logic is the pure
/// `relabel_baseline_for_renames` kernel, which applies the existence and non-collision
/// guards. Kept here rather than in the kernel so the kernel stays filesystem-free.
///
/// FAIL-OPEN BY DESIGN, and deliberately so: if git cannot answer (detached checkout,
/// missing merge-base, shallow clone) this returns an EMPTY map, which relabels nothing
/// and leaves the ratchet in its strict pre-existing behaviour. The failure mode of this
/// helper is therefore a false RED that a human investigates, never a false GREEN.
fn detect_renames(root: &Path, merge_base: &str) -> BTreeMap<String, String> {
    let mut renames = BTreeMap::new();
    if merge_base.is_empty() {
        return renames;
    }
    let Ok(out) = Command::new("git")
        .current_dir(root)
        .args([
            "diff",
            // THRESHOLD 30%, not git's 50% default, and the reason is specific: a
            // relocated `Cargo.toml` MUST rewrite its package name, its lib name, and
            // every relative path dependency. On a ~20-line manifest those few lines are
            // a large fraction of the total BYTES, so the default similarity index drops
            // below 50% and git reports add+delete — missing precisely the file a crate
            // move is guaranteed to touch. Measured on this repo's first move: 50% and
            // 40% both miss it, 30% pairs it, 25% finds nothing further. 30% is the knee.
            //
            // Mis-pairing risk is bounded: git only ever pairs a deletion with an
            // addition and takes the best match, and the pure kernel then applies the
            // existence guard (never invent a key) and the non-collision guard (never
            // merge two baselined rows), so a wrong pair cannot shrink the baseline.
            "--find-renames=30%",
            "--diff-filter=R",
            "--name-status",
            "-z",
            merge_base,
        ])
        .output()
    else {
        return renames;
    };
    if !out.status.success() {
        return renames;
    }
    // `-z` output for a rename is three NUL-terminated fields: "R<score>", old, new.
    let fields: Vec<&str> = std::str::from_utf8(&out.stdout)
        .unwrap_or_default()
        .split('\0')
        .filter(|s| !s.is_empty())
        .collect();
    let mut i = 0;
    while i + 2 < fields.len() + 1 {
        let Some(status) = fields.get(i) else { break };
        if !status.starts_with('R') {
            // Not a rename record; advance one field and resync.
            i += 1;
            continue;
        }
        let (Some(old), Some(new)) = (fields.get(i + 1), fields.get(i + 2)) else {
            break;
        };
        renames.insert((*old).to_owned(), (*new).to_owned());
        i += 3;
    }
    renames
}

/// Load the FROZEN merge-base reference. FAIL-CLOSED: a missing or invalid snapshot is a
/// hard failure with the exact remediation, never a silent fall-back to the PR-local face
/// (the FRIC-1781112000 laundering hole this gate exists to close).
fn load_frozen_baseline(root: &Path) -> FrozenBaseline {
    let path = frozen_snapshot_path(root);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "FAIL-CLOSED: merge-base frozen baseline snapshot missing at {} ({e}). The \
             firewall compares against the gate-baseline face at `git merge-base <base_ref> \
             HEAD` (ADR-0551, FRIC-1781112000), never the PR-local copy. Materialize it: \
             buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root . (CI runs this before every \
             gate lane).",
            path.display()
        )
    });
    let value: Value =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
    FrozenBaseline::from_value(&value)
        .unwrap_or_else(|e| panic!("invalid frozen baseline snapshot {}: {e}", path.display()))
}

fn load_json(path: &Path) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

/// Regenerate the gate-baseline face from the LIVE tree (in --stdout sandbox mode),
/// HERMETICALLY. The producer binary must be provided by `OYA_CI_PRODUCER_BIN`; missing env fails
/// closed so tests cannot silently fall back to Cargo. The producer reads the materialized scm-facts
/// face (a declared input); it never calls git.
fn regenerate_baseline(root: &Path) -> Value {
    let scm_facts = faces_dir(root).join("scm-facts.generated.json");
    let producer_bin = std::env::var("OYA_CI_PRODUCER_BIN").ok();
    let bin = producer_binary(root, producer_bin.as_deref()).unwrap_or_else(|e| panic!("{e}"));
    let mut command = Command::new(bin);
    command
        .arg("--repo-root")
        .arg(root)
        .arg("--scm-facts")
        .arg(&scm_facts);
    append_declared_enforcement_liveness_corpus_args(&mut command, root);
    let output = command
        .arg("--stdout")
        .arg("--face")
        .arg("baseline")
        .current_dir(root)
        .output()
        .expect("run producer binary");
    assert!(
        output.status.success(),
        "producer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("baseline stdout is valid JSON")
}

fn producer_binary(root: &Path, producer_bin: Option<&str>) -> Result<PathBuf, String> {
    let Some(bin) = producer_bin else {
        return Err(
            "FAIL-CLOSED: missing OYA_CI_PRODUCER_BIN; Cargo fallback is forbidden".to_owned(),
        );
    };
    Ok(resolve_bin(root, bin))
}

#[test]
fn producer_binary_env_is_required_for_hermetic_gate() {
    let err = producer_binary(Path::new("/repo"), None)
        .expect_err("missing OYA_CI_PRODUCER_BIN must fail closed");
    assert!(err.contains("OYA_CI_PRODUCER_BIN"));
}

fn resolve_bin(root: &Path, bin: &str) -> PathBuf {
    let path = PathBuf::from(bin);
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}

fn append_declared_enforcement_liveness_corpus_args(command: &mut Command, root: &Path) {
    append_enforcement_liveness_corpus_paths(
        command,
        &declared_corpus_file(
            root,
            ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS_ENV,
            ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS,
            "settings.json",
        ),
        &declared_corpus_file(
            root,
            ENFORCEMENT_LIVENESS_CODEX_HOOKS_ENV,
            ENFORCEMENT_LIVENESS_CODEX_HOOKS,
            "hooks.json",
        ),
        &declared_corpus_path(
            root,
            ENFORCEMENT_LIVENESS_HOOKS_DIR_ENV,
            ENFORCEMENT_LIVENESS_HOOKS_DIR,
        ),
    );
}

fn declared_corpus_file(
    root: &Path,
    env_key: &str,
    fallback_rel: &str,
    file_name: &str,
) -> PathBuf {
    let path = declared_corpus_path(root, env_key, fallback_rel);
    if path.is_file() {
        return path;
    }
    let nested = path.join(file_name);
    if nested.is_file() {
        return nested;
    }
    path
}

fn declared_corpus_path(root: &Path, env_key: &str, fallback_rel: &str) -> PathBuf {
    declared_corpus_path_from_env(
        root,
        env_key,
        fallback_rel,
        std::env::var("OYA_CI_PRODUCER_BIN").is_ok(),
        std::env::var(env_key).ok().as_deref(),
    )
}

fn declared_corpus_path_from_env(
    root: &Path,
    env_key: &str,
    fallback_rel: &str,
    buck_backed_producer: bool,
    env_value: Option<&str>,
) -> PathBuf {
    if let Some(value) = env_value {
        return resolve_bin(root, value);
    }
    assert!(
        !buck_backed_producer,
        "FAIL-CLOSED: buck-backed firewall producer invocation is missing declared corpus env {env_key}"
    );
    root.join(fallback_rel)
}

fn append_enforcement_liveness_corpus_paths(
    command: &mut Command,
    claude_settings: &Path,
    codex_hooks: &Path,
    hooks_dir: &Path,
) {
    command
        .arg("--enforcement-liveness-claude-settings")
        .arg(claude_settings)
        .arg("--enforcement-liveness-codex-hooks")
        .arg(codex_hooks)
        .arg("--enforcement-liveness-hooks-dir")
        .arg(hooks_dir);
}

#[test]
fn baseline_regeneration_declares_enforcement_liveness_corpus_args() {
    let mut command = Command::new("/tmp/producer");
    append_enforcement_liveness_corpus_paths(
        &mut command,
        Path::new("/repo/.claude/settings.json"),
        Path::new("/repo/.codex/hooks.json"),
        Path::new("/repo/tools/hooks"),
    );

    let args: Vec<String> = command
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();

    assert!(args.windows(2).any(|pair| {
        pair == [
            "--enforcement-liveness-claude-settings",
            "/repo/.claude/settings.json",
        ]
    }));
    assert!(args.windows(2).any(|pair| {
        pair == [
            "--enforcement-liveness-codex-hooks",
            "/repo/.codex/hooks.json",
        ]
    }));
    assert!(
        args.windows(2)
            .any(|pair| { pair == ["--enforcement-liveness-hooks-dir", "/repo/tools/hooks",] })
    );
}

#[test]
fn buck_backed_firewall_requires_declared_corpus_env() {
    let panic = std::panic::catch_unwind(|| {
        declared_corpus_path_from_env(
            Path::new("/repo"),
            "MISSING_CORPUS_ENV",
            "fallback",
            true,
            None,
        );
    })
    .expect_err("buck-backed missing corpus env must fail closed");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");
    assert!(message.contains("FAIL-CLOSED"));
    assert!(message.contains("MISSING_CORPUS_ENV"));
}

fn fixture_dir(root: &Path) -> PathBuf {
    root.join("specs/fixtures/cloud-ci-firewall")
}

fn current_from_value(value: &Value) -> BTreeMap<String, BTreeMap<String, BTreeSet<String>>> {
    let mut out: BTreeMap<String, BTreeMap<String, BTreeSet<String>>> = BTreeMap::new();
    if let Some(gates) = value.as_object() {
        for (gate, codes) in gates {
            let mut code_map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
            if let Some(codes_obj) = codes.as_object() {
                for (code, keys) in codes_obj {
                    let set: BTreeSet<String> = keys
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(Value::as_str)
                                .map(str::to_owned)
                                .collect()
                        })
                        .unwrap_or_default();
                    code_map.insert(code.clone(), set);
                }
            }
            out.insert(gate.clone(), code_map);
        }
    }
    out
}

/// Fixture-driven RED/GREEN corpus: each tc-*.json carries a merge_base_baseline (the
/// FROZEN reference) + current + proposed_baseline + signoff and the expected firewall
/// verdict / failing codes / ratchet growth count. The compare-mode + ratchet-invariant
/// predicates are pure, so the fixtures drive them with zero scanner special-cases (the
/// per-code behaviour is DATA: mode + frozen_empty). This is the data-under-test contract,
/// mirroring the four gate corpora.
#[test]
fn firewall_fixtures_execute_red_green_cases() {
    let dir = fixture_dir(&repo_root());
    let mut tc_paths: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("tc-") && n.ends_with(".json"))
        })
        .collect();
    tc_paths.sort();
    assert!(
        !tc_paths.is_empty(),
        "firewall fixture corpus must not be empty"
    );

    let mut seen_green = false;
    let mut seen_red = false;

    for path in &tc_paths {
        let fixture = load_json(path);
        let label = path.file_name().unwrap().to_string_lossy().to_string();

        assert!(
            fixture.get("committed_baseline").is_none(),
            "{label}: stale fixture field committed_baseline — the frozen reference is the \
             merge-base baseline (merge_base_baseline) per ADR-0551/FRIC-1781112000"
        );
        let frozen = Baseline::from_value(&fixture["merge_base_baseline"]).unwrap();
        let proposed = Baseline::from_value(&fixture["proposed_baseline"]).unwrap();
        let signoff = SignOff::from_value(&fixture["signoff"]);
        let current = current_from_value(&fixture["current"]);

        let report = evaluate_firewall(&frozen, &proposed, &current, &signoff);

        let expected_growth = fixture["expected_ratchet_growth"].as_u64().unwrap_or(0) as usize;
        assert_eq!(
            report.ratchet_growth.len(),
            expected_growth,
            "{label}: ratchet_growth count mismatch (growth = {:?})",
            report.ratchet_growth
        );

        let expected_inert = fixture["expected_inert_signoff"].as_u64().unwrap_or(0) as usize;
        assert_eq!(
            report.inert_signoff.len(),
            expected_inert,
            "{label}: inert_signoff count mismatch (inert = {:?})",
            report.inert_signoff
        );

        let expected_failing: BTreeSet<String> = fixture["expected_failing_codes"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        let actual_failing: BTreeSet<String> = report
            .codes
            .iter()
            .filter(|r| r.fails())
            .map(|r| r.code.clone())
            .collect();
        assert_eq!(
            actual_failing, expected_failing,
            "{label}: failing-code set mismatch"
        );

        match fixture["expected_firewall"].as_str() {
            Some("GREEN") => {
                seen_green = true;
                assert!(report.is_green(), "{label} must be GREEN");
            }
            Some("RED") => {
                seen_red = true;
                assert!(!report.is_green(), "{label} must be RED");
            }
            other => panic!("{label} has unsupported expected_firewall {other:?}"),
        }
    }

    assert!(
        seen_green && seen_red,
        "firewall fixtures must include BOTH RED and GREEN cases"
    );
}

/// THE GO-LIVE PROOF: with the FROZEN merge-base reference, the firewall is GREEN on the
/// live corpus. A settled change only ever SHRINKS the baseline relative to the merge-base
/// (or grows it through the sign-off door), so:
///   - compare-mode: current keys ⊆ frozen ∪ signed-off => zero regressions => no
///     baseline-block-on-new code fails (advisory codes report their counts but never fail);
///   - ratchet-invariant: blocking proposed keys ⊆ frozen ∪ signed-off => zero growth.
#[test]
fn firewall_is_green_on_the_live_corpus_with_the_baseline() {
    let root = repo_root();

    // The FROZEN reference: the gate-baseline face at `git merge-base <base_ref> HEAD`,
    // materialized out-of-graph by the scm-facts emitter. NEVER the PR-local face.
    let frozen = load_frozen_baseline(&root);

    // The proposed baseline = what TODAY's corpus would freeze.
    let proposed_value = regenerate_baseline(&root);
    let proposed = Baseline::from_value(&proposed_value).unwrap();

    // The sign-off door (the one-way exemption; empty = ratchet fully closed).
    let signoff = SignOff::from_value(&load_json(&signoff_path(&root)));

    // The live "current" keyed violations == the proposed baseline's keys (the producer
    // captured them via evaluate_keyed over the live faces).
    let current = baseline_keys_map(&proposed);

    // ADR-0562 STRUCTURAL MOVES: the baseline is PATH-KEYED, so a pure `git mv` scores as
    // regressions at the destination and fixes at the source — net-zero debt that still
    // REDs the gate, because the ratchet blocks on any new key. Relabel the frozen keys
    // through git's own rename detection (merge-base -> HEAD) so a relocation behaves like
    // an in-place edit and the debt follows the file. Not a waiver: the key count per code
    // is unchanged, the violation stays tolerated, and debt ADDED alongside a move still
    // regresses. See `relabel_baseline_for_renames` for the existence/non-collision guards.
    let renames = detect_renames(&root, &frozen.merge_base);
    if !renames.is_empty() {
        eprintln!(
            "FIREWALL rename-relabel: {} path rename(s) detected since merge-base {}; \
             frozen baseline keys relabelled so pure moves are neither regressions nor fixes.",
            renames.len(),
            frozen.merge_base
        );
    }
    let frozen_relabelled = relabel_baseline_for_renames(&frozen.baseline, &renames);

    let report = evaluate_firewall(&frozen_relabelled, &proposed, &current, &signoff);

    // Evidence digest: per-code current/baseline/regressions/fixed/tolerated/signed-off.
    eprintln!(
        "FIREWALL GO-LIVE report (live corpus vs frozen {} @ merge-base {}):",
        frozen.base_ref, frozen.merge_base
    );
    for r in &report.codes {
        eprintln!(
            "  [{}] {:48} mode={:22} current={:6} baseline={:6} regressions={:4} fixed={:4} tolerated={:6} signed_off={:4}{}",
            r.gate,
            r.code,
            r.mode,
            r.current,
            r.baseline,
            r.regressions.len(),
            r.fixed.len(),
            r.tolerated.len(),
            r.signed_off.len(),
            if r.fails() { "  <-- FAIL" } else { "" }
        );
    }
    eprintln!(
        "  ratchet_growth (un-signed-off blocking baseline additions vs merge-base): {}",
        report.ratchet_growth.len()
    );
    eprintln!(
        "  inert_signoff (door entries exempting nothing — retire them): {}",
        report.inert_signoff.len()
    );

    let failing: Vec<&str> = report
        .codes
        .iter()
        .filter(|r| r.fails())
        .map(|r| r.code.as_str())
        .collect();
    // ADR-0555: a FAIL is never a bare flag — print, per failing code, the offending keys
    // AND the exact registration edit (or the precise design decision needed) from the
    // disposition DATA.
    let regression_detail: Vec<String> = report
        .codes
        .iter()
        .filter(|r| r.fails())
        .map(|r| {
            format!(
                "[{}] {} regressions {:?}\n  REGISTRATION REQUIRED: {}",
                r.gate,
                r.code,
                r.regressions,
                r.remediation
                    .as_deref()
                    .unwrap_or("(no remediation stamped — fix the disposition DATA)")
            )
        })
        .collect();
    // BOOTSTRAP WINDOW: when the gate-baseline face was absent at the merge-base (e.g. the
    // one-time hotfix PR that re-introduces a face that was incorrectly de-committed), the
    // frozen reference is empty.  Comparing against an empty baseline produces only false
    // positives — the corpus has not grown, the REFERENCE was simply absent.  We skip the
    // empty-baseline regression comparison but enforce two real invariants:
    //
    // (1) COMMITTED-BLOB CHECK: the face must be committed at HEAD (not just materialized on
    //     disk by the CI materialize step).  Uses `git cat-file -e HEAD:<path>` — the same
    //     oracle the emitter uses for the merge-base side — rather than `is_file()`.
    //
    // (2) NO NEW RATCHET GROWTH vs PRE-DECOMMIT BASELINE: we load the last-good baseline
    //     from `git show <merge_base>^:<face_path>` (the parent of the broken merge-base,
    //     i.e. the commit just before the de-commit PR landed) and use it as the growth-check
    //     reference.  This correctly distinguishes pre-existing debt (present in both the
    //     last-good and the re-introduced baseline) from genuinely new debt introduced by
    //     this PR.  Against an empty frozen (as `ratchet_growth` with `&frozen.baseline`
    //     would use), ALL 48k pre-existing keys appear as "unsigned growth" — a false alarm
    //     that would make the bootstrap PR permanently un-mergeable without signing off every
    //     pre-existing key.  Using the parent commit restores the correct debt ceiling.
    //
    // Once this PR merges, future PRs will have the face at their merge-base,
    // `missing_at_merge_base=false`, and the normal GO-LIVE path resumes.
    if frozen.missing_at_merge_base {
        let policy = load_json(&ratchet_policy_path(&root));
        let face_path = policy["frozen_reference"]["face_path"]
            .as_str()
            .expect("ratchet policy frozen_reference.face_path");

        // (1) Committed-blob check: the face must be a committed git blob at HEAD.
        let cat = Command::new("git")
            .args(["cat-file", "-e", &format!("HEAD:{face_path}")])
            .current_dir(&root)
            .status()
            .expect("git cat-file");
        assert!(
            cat.success(),
            "BOOTSTRAP: frozen.missing_at_merge_base is true but {face_path} is NOT \
             committed at HEAD — the face must be committed (not just materialized to disk) \
             for a legitimate bootstrap. Check ratchet-policy.json frozen_reference.face_path."
        );

        // (2) Load the last-good baseline from the parent of the broken merge-base commit
        // (`<merge_base>^`), i.e. the state just before the de-commit PR landed.  This is
        // the correct debt ceiling: the bootstrap PR is allowed to freeze exactly what was
        // already in the corpus, but no new unsigned blocking keys.
        let parent_ref = format!("{}^:{face_path}", frozen.merge_base);
        let parent_output = Command::new("git")
            .args(["show", &parent_ref])
            .current_dir(&root)
            .output()
            .expect("git show parent baseline");
        assert!(
            parent_output.status.success(),
            "BOOTSTRAP: could not load the pre-decommit baseline from {parent_ref} — \
             the parent of the broken merge-base must carry the last-good face. \
             stderr: {}",
            String::from_utf8_lossy(&parent_output.stderr)
        );
        let parent_value: Value =
            serde_json::from_slice(&parent_output.stdout).expect("parse parent baseline JSON");
        let pre_decommit = Baseline::from_value(&parent_value).unwrap();

        let new_growth: Vec<(String, String, String)> =
            ratchet_growth(&pre_decommit, &proposed, &signoff);
        let new_growth_detail: Vec<String> = new_growth
            .iter()
            .map(|(gate, code, key)| {
                let remediation = proposed
                    .gates
                    .get(gate)
                    .and_then(|codes| codes.get(code))
                    .and_then(|cb| cb.remediation.as_deref())
                    .unwrap_or("(no remediation stamped — fix the disposition DATA)");
                format!(
                    "[{gate}] {code} new unsigned key {key:?}\n  REGISTRATION REQUIRED: \
                     {remediation}"
                )
            })
            .collect();
        assert!(
            new_growth.is_empty(),
            "BOOTSTRAP: the re-introduction PR carries NEW unsigned blocking debt vs the \
             pre-decommit baseline ({parent_ref}).  New debt must pass through the sign-off \
             door or be removed.  New unsigned keys:\n{}",
            new_growth_detail.join("\n")
        );

        assert!(
            !proposed.gates.is_empty(),
            "BOOTSTRAP: candidate baseline must be non-empty even during bootstrap window"
        );
        eprintln!(
            "BOOTSTRAP WINDOW: frozen.missing_at_merge_base=true; skipping empty-baseline \
             regression check (merge-base {} has no face).  Committed-blob check passed; \
             zero new growth vs pre-decommit parent {} confirmed.",
            frozen.merge_base, parent_ref
        );
        return;
    }

    assert!(
        failing.is_empty(),
        "GO-LIVE: firewall must be GREEN on today's corpus (no NEW debt vs the merge-base), \
         but these codes FAIL: {failing:?};\n{}",
        regression_detail.join("\n")
    );
    let growth_detail: Vec<String> = report
        .ratchet_growth
        .iter()
        .map(|(gate, code, key)| {
            let remediation = proposed
                .gates
                .get(gate)
                .and_then(|codes| codes.get(code))
                .and_then(|cb| cb.remediation.as_deref())
                .unwrap_or("(no remediation stamped — fix the disposition DATA)");
            format!("[{gate}] {code} grew {key}\n  REGISTRATION REQUIRED: {remediation}")
        })
        .collect();
    assert!(
        report.ratchet_growth.is_empty(),
        "GO-LIVE: blocking baseline keys must shrink (or pass the sign-off door) relative \
         to the merge-base, got growth:\n{}",
        growth_detail.join("\n")
    );
    assert!(
        report.inert_signoff.is_empty(),
        "GO-LIVE: every sign-off door entry must exempt a key the CANDIDATE tree still \
         carries (current or proposed) — an entry the candidate has orphaned is a standing \
         re-introduction ticket (FRIC-1781460000: read against the candidate, not the \
         merge-base frozen face, so PR-tier and push-tier agree). \
         Remediation (auto-derives + applies the retirement): {SIGNOFF_FIXER_COMMAND} \
         Inert: {:?}",
        report.inert_signoff
    );
    assert!(
        report.is_green(),
        "firewall must be GREEN with the merge-base frozen reference"
    );

    // Sanity: the baseline is NON-trivial (the frozen pre-existing corpus debt is real).
    let total_baselined: usize = report.codes.iter().map(|r| r.baseline).sum();
    assert!(
        total_baselined > 0,
        "the baseline must freeze the real pre-existing corpus debt"
    );
}

/// THE ADR-0555 CONVERSION PROOF (FRIC-1781330000), against the LIVE corpus + LIVE
/// producer: the exists-but-unaccounted codes (unowned, unreachable, no_ttl_class,
/// untyped_staleness) are stamped baseline-block-on-new by the disposition DATA, and in
/// the ARMED configuration — the frozen reference a PR sees at the first merge-base
/// advance after the flip merges (frozen := today's regenerated face) — a NEW unowned
/// file and a NEW unreachable file EACH make the firewall fail, while the grandfathered
/// pre-existing key sets stay tolerated. In-flight window stated honestly: at THIS PR's
/// own merge-base the frozen modes are still advisory (frozen-mode-wins, ADR-0551 §6),
/// so the flip cannot brick the PR that carries it; it arms one merge later.
#[test]
fn converted_accounting_codes_block_new_keys_when_armed() {
    let root = repo_root();
    let proposed_value = regenerate_baseline(&root);

    // (1) The disposition flip is LIVE in the producer DATA (not just in fixtures).
    let ta = &proposed_value["gates"]["cloud-ci-total-accounting"];
    for code in ["unowned", "unreachable", "no_ttl_class"] {
        assert_eq!(
            ta[code]["mode"], "baseline-block-on-new",
            "ADR-0555: {code} must be stamped blocking by the live disposition table"
        );
    }
    let sr = &proposed_value["gates"]["cloud-ci-staleness-reaper"];
    assert_eq!(
        sr["untyped_staleness"]["mode"], "baseline-block-on-new",
        "ADR-0555: untyped_staleness must be stamped blocking"
    );
    assert_eq!(
        sr["stale_over_budget_unreachable"]["mode"], "advisory-until-infra",
        "ADR-0555: time-driven decay stays advisory BY DESIGN (reaper reconciler surface)"
    );
    // A FAIL is never a bare flag: the converted codes carry the registration remediation.
    for code in ["unowned", "unreachable"] {
        assert!(
            ta[code]["remediation"]
                .as_str()
                .is_some_and(|t| !t.is_empty()),
            "ADR-0555: {code} must stamp its registration remediation as DATA"
        );
    }

    // (2) ARMED configuration: frozen := today's face (what the merge-base holds one
    // merge after the flip lands). The grandfathered debt is the frozen key set. This test
    // isolates the ADR-0555 accounting-code conversion, so it uses an EMPTY sign-off door:
    // the live committed door (gate-baseline.signoff.json) carries founder-admitted go-live
    // keys that are not present in this synthetic frozen/proposed pair, which #701's
    // inert-door detector (FRIC-1781280001) would correctly flag — that is the inert-door
    // gate's concern, exercised by its own pins, not the conversion's. An empty door keeps
    // is_green() a faithful read of the conversion predicates alone.
    let proposed = Baseline::from_value(&proposed_value).unwrap();
    let frozen = proposed.clone();
    let signoff = SignOff::default();

    // Clean tree: everything tolerated, GREEN (the grandfather works).
    let clean = baseline_keys_map(&proposed);
    let report = evaluate_firewall(&frozen, &proposed, &clean, &signoff);
    assert!(
        report.is_green(),
        "armed frozen reference must tolerate the grandfathered debt"
    );

    // A NEW unowned file is UNMERGEABLE.
    let mut with_unowned = clean.clone();
    with_unowned
        .entry("cloud-ci-total-accounting".to_owned())
        .or_default()
        .entry("unowned".to_owned())
        .or_default()
        .insert("SYNTHETIC/new-service/born-unowned.rs".to_owned());
    let report = evaluate_firewall(&frozen, &proposed, &with_unowned, &signoff);
    let unowned = report
        .codes
        .iter()
        .find(|r| r.gate == "cloud-ci-total-accounting" && r.code == "unowned")
        .expect("unowned code present");
    assert!(
        unowned
            .regressions
            .contains("SYNTHETIC/new-service/born-unowned.rs")
            && unowned.fails(),
        "a NEW unowned file must FAIL the armed firewall"
    );
    assert!(
        unowned
            .remediation
            .as_deref()
            .is_some_and(|t| t.contains("OWNERS")),
        "the unowned FAIL must carry the exact registration edit, never a bare flag"
    );
    assert!(!report.is_green());

    // A NEW unreachable file is UNMERGEABLE.
    let mut with_unreachable = clean.clone();
    with_unreachable
        .entry("cloud-ci-total-accounting".to_owned())
        .or_default()
        .entry("unreachable".to_owned())
        .or_default()
        .insert("SYNTHETIC/docs/born-unreachable.md".to_owned());
    let report = evaluate_firewall(&frozen, &proposed, &with_unreachable, &signoff);
    let unreachable = report
        .codes
        .iter()
        .find(|r| r.gate == "cloud-ci-total-accounting" && r.code == "unreachable")
        .expect("unreachable code present");
    assert!(
        unreachable
            .regressions
            .contains("SYNTHETIC/docs/born-unreachable.md")
            && unreachable.fails(),
        "a NEW unreachable file must FAIL the armed firewall"
    );
    assert!(
        unreachable
            .remediation
            .as_deref()
            .is_some_and(|t| t.contains("reachability-registry")),
        "the unreachable FAIL must carry the exact registration edit, never a bare flag"
    );
    assert!(!report.is_green());

    // (3) Laundering via regen is still closed in the armed configuration: the same-PR
    // settle regen carries the new key into proposed too — that is ratchet growth.
    let mut laundered_value = regenerate_baseline(&root);
    if let Some(keys) = laundered_value
        .get_mut("gates")
        .and_then(|g| g.get_mut("cloud-ci-total-accounting"))
        .and_then(|g| g.get_mut("unowned"))
        .and_then(|c| c.get_mut("keys"))
        .and_then(Value::as_array_mut)
    {
        keys.push(Value::String(
            "SYNTHETIC/new-service/born-unowned.rs".to_owned(),
        ));
    }
    let laundered = Baseline::from_value(&laundered_value).unwrap();
    let report = evaluate_firewall(
        &frozen,
        &laundered,
        &baseline_keys_map(&laundered),
        &signoff,
    );
    assert!(
        report.ratchet_growth.iter().any(|(_, code, key)| {
            code == "unowned" && key == "SYNTHETIC/new-service/born-unowned.rs"
        }),
        "the armed conversion must catch same-PR regen laundering as ratchet growth"
    );
    assert!(!report.is_green());
}

/// The frozen snapshot's provenance must agree with the committed ratchet policy: same
/// configurable base_ref (R0 policy-as-data), a full revision id, and the exact face path —
/// the audit trail naming WHICH frozen point the firewall compared against. Under
/// frozen-policy-wins (FRIC-1781280000) the snapshot's facts come from the MERGE-BASE
/// policy, so this doubles as the live pin that the candidate and frozen policies agree —
/// any divergence (e.g. a same-PR repoint) goes RED here.
#[test]
fn frozen_snapshot_provenance_matches_ratchet_policy() {
    let root = repo_root();
    let frozen = load_frozen_baseline(&root);
    let policy = load_json(&ratchet_policy_path(&root));
    assert_eq!(
        frozen.base_ref,
        policy["base_ref"].as_str().expect("policy base_ref"),
        "snapshot base_ref (the FROZEN merge-base policy's) must agree with the candidate \
         ratchet-policy.json"
    );
    assert_eq!(frozen.merge_base.len(), 40, "full hex revision id");
    assert_eq!(
        frozen.frozen_policy_source, "merge-base",
        "this repo's ratchet policy exists at the merge-base (merged in PR #698): the \
         frozen-policy-wins path must be the one actually exercised, never the \
         candidate-bootstrap fallback"
    );
    let snapshot = load_json(&frozen_snapshot_path(&root));
    // FROZEN-POLICY-WINS (FRIC-1781280000): the snapshot's face_path is read from the
    // ratchet policy AS COMMITTED AT THE MERGE-BASE, never the candidate policy. The ci keystone
    // move (cloud/cloud-ci/gates -> ci/facade/, ADR-0562/0563) has MERGED, so the merge-base
    // policy now points at the NEW ci/facade face path — the merge-base and candidate policies
    // have RE-CONVERGED (the transitional OLD-path assertion that held during the move PR is
    // retired here, as its own comment anticipated). Asserting the frozen face_path is the
    // concrete merge-base path (never a candidate-bootstrap fallback) still guards the
    // self-laundering hole this property exists for.
    const FROZEN_MERGE_BASE_FACE_PATH: &str =
        "ci/facade/artifact-inventory-registry/gate-baseline.generated.json";
    assert_eq!(
        snapshot["face_path"].as_str(),
        Some(FROZEN_MERGE_BASE_FACE_PATH),
        "snapshot must record the FROZEN merge-base policy face path (the OLD path, \
         frozen-policy-wins); the candidate policy's repointed ci/facade path is NOT the \
         frozen reference"
    );
    // BOOTSTRAP WINDOW: when missing_at_merge_base=true the face was absent at the
    // merge-base.  There are exactly two legitimate causes:
    //   (a) wrong path — the emitter extracted a non-existent path (BUG);
    //   (b) one-time re-introduction PR — the face is being re-committed after it was
    //       incorrectly de-committed (e.g. ADR-0595 false-premise hotfix).
    // We distinguish (a) from (b) by verifying the face is committed at HEAD via
    // `git cat-file -e HEAD:<path>` — the same oracle the emitter uses for the merge-base
    // side.  `is_file()` (disk read) would be wrong here: the CI materialize step writes
    // the face to disk before gates run, so it returns true regardless of commit state.
    // Once this PR merges, future PRs will see the face at their merge-base (normal path).
    if frozen.missing_at_merge_base {
        let face_path = policy["frozen_reference"]["face_path"]
            .as_str()
            .expect("ratchet policy frozen_reference.face_path");
        // `git cat-file -e HEAD:<face_path>` exits 0 iff the blob is committed at HEAD.
        let cat = Command::new("git")
            .args(["cat-file", "-e", &format!("HEAD:{face_path}")])
            .current_dir(&root)
            .status()
            .expect("git cat-file");
        assert!(
            cat.success(),
            "frozen.missing_at_merge_base is true AND the candidate face {face_path} is NOT \
             committed at HEAD — this is case (a): the emitter extracted the wrong path, or \
             the face was not committed (only materialized to disk by CI). Check \
             ratchet-policy.json frozen_reference.face_path."
        );
        // Case (b) confirmed: bootstrap window for the re-introduction PR.  No further
        // assertion needed here — firewall_is_green_on_the_live_corpus_with_the_baseline
        // enforces committed-blob + zero unsigned ratchet growth.
        return;
    }
    // Normal steady-state path: gate-baseline exists at the merge-base.
    // (This assertion is structurally redundant after the early-return above, but kept as
    // an explicit invariant statement for readability.)
    assert!(
        !frozen.missing_at_merge_base,
        "this repo's gate-baseline face exists at the merge-base; a missing-face snapshot \
         here means the emitter extracted the wrong path"
    );
}

/// THE F1 PIN (defense-in-depth on top of frozen-policy-wins, never instead of it): the
/// committed comparison root is `origin/dev`. A same-PR `base_ref` repoint can no longer
/// select the PR's own frozen reference (the emitter reads the frozen-side policy from the
/// merge-base tree), but it could still change post-merge behavior silently — this pin
/// makes any repoint require a visible edit to THIS test as well.
#[test]
fn ratchet_policy_base_ref_is_pinned_to_origin_dev() {
    let root = repo_root();
    let policy = load_json(&ratchet_policy_path(&root));
    assert_eq!(
        policy["base_ref"], "origin/dev",
        "ratchet-policy.json base_ref repointed: the frozen comparison root for this \
         repository is origin/dev (ADR-0551). If this repoint is intentional it requires \
         founder sign-off, an update to the out-of-band bootstrap (--frozen-base-ref / \
         DEFAULT_FROZEN_BOOTSTRAP_REF in the scm-facts emitter), and an edit to this pin \
         (FRIC-1781280000)."
    );
}

/// THE FRIC-1781112000 PIN. A PR adds new debt AND regenerates the baseline face in the
/// same change — exactly what the settle protocol mandates and registry-drift enforces, so
/// the PR-local face always equals the regenerated face. FOIL: against the PR-local
/// reference the laundering is structurally invisible (GREEN). GATE: against the FROZEN
/// merge-base reference the same state is RED — both as compare-mode regressions and as
/// ratchet growth. This is the misattribution shape from PR #670 (a new
/// manifest-hygiene debt key absorbed by a same-PR baseline regen, 21/21 checks green).
#[test]
fn firewall_blocks_same_pr_baseline_regen_laundering() {
    let root = repo_root();
    let frozen = load_frozen_baseline(&root);

    // The laundering PR: new debt appears in the regenerated (and therefore committed)
    // baseline face and in the live current set simultaneously.
    let mut proposed_value = regenerate_baseline(&root);
    if let Some(keys) = proposed_value
        .get_mut("gates")
        .and_then(|g| g.get_mut("cloud-ci-total-accounting"))
        .and_then(|g| g.get_mut("unjustified"))
        .and_then(|c| c.get_mut("keys"))
        .and_then(Value::as_array_mut)
    {
        keys.push(Value::String(
            "SYNTHETIC/laundered-in-same-pr.rs".to_owned(),
        ));
    }
    let proposed = Baseline::from_value(&proposed_value).unwrap();
    let pr_local_reference = proposed.clone(); // settle protocol: committed == regenerated
    let current = baseline_keys_map(&proposed);

    // FOIL — the historical hole: the PR-local reference cannot see its own laundering.
    // This leg isolates the COMPARE-MODE + RATCHET laundering predicates, so it uses an
    // EMPTY sign-off door: the live committed door (gate-baseline.signoff.json) exempts
    // go-live keys against the FROZEN merge-base reference, but here the reference is the
    // synthetic PR-LOCAL `proposed` (== regenerated). After the ADR-0555 registration
    // retires some unowned/unreachable debt, those door keys are absent from `proposed`,
    // which #701's inert-door detector (FRIC-1781280001) would correctly flag against this
    // synthetic reference — that is the inert-door gate's concern (the LIVE door is verified
    // clean against the merge-base by the signoff fixer), not this laundering foil's. An
    // empty door keeps the foil a faithful read of the laundering predicates alone.
    let empty_door = SignOff::default();
    let laundered = evaluate_firewall(&pr_local_reference, &proposed, &current, &empty_door);
    assert!(
        laundered.is_green(),
        "FOIL: against the PR-local reference the laundering must be invisible — if this \
         fails, the foil no longer demonstrates the hole and the pin needs re-derivation"
    );

    // THE GATE: the frozen merge-base reference blocks it, on BOTH predicates.
    let report = evaluate_firewall(&frozen.baseline, &proposed, &current, &empty_door);
    assert!(
        report.ratchet_growth.iter().any(|(_, code, key)| {
            code == "unjustified" && key == "SYNTHETIC/laundered-in-same-pr.rs"
        }),
        "same-PR baseline regen must be ratchet growth vs the merge-base: {:?}",
        report.ratchet_growth
    );
    let unjust = report
        .codes
        .iter()
        .find(|r| r.gate == "cloud-ci-total-accounting" && r.code == "unjustified")
        .expect("unjustified code present");
    assert!(
        unjust
            .regressions
            .contains("SYNTHETIC/laundered-in-same-pr.rs"),
        "the laundered key must also be a compare-mode regression vs the merge-base"
    );
    assert!(
        !report.is_green(),
        "firewall must be RED on same-PR laundering"
    );
}

/// RED-on-NEW proof against the LIVE corpus: inject ONE synthetic NEW key into the live
/// "current" set for a baseline-block-on-new code and assert the firewall FAILS — proving
/// the gate still blocks any new finite violation that is not in the frozen baseline.
#[test]
fn firewall_goes_red_on_a_synthetic_new_violation() {
    let root = repo_root();
    let frozen = load_frozen_baseline(&root);
    let proposed = Baseline::from_value(&regenerate_baseline(&root)).unwrap();
    let signoff = SignOff::from_value(&load_json(&signoff_path(&root)));

    let mut current = baseline_keys_map(&proposed);
    // Add a NEW unjustified path that is NOT in the frozen merge-base baseline.
    current
        .entry("cloud-ci-total-accounting".to_owned())
        .or_default()
        .entry("unjustified".to_owned())
        .or_default()
        .insert("SYNTHETIC/new-unjustified-file.rs".to_owned());

    let report = evaluate_firewall(&frozen.baseline, &proposed, &current, &signoff);
    let unjust = report
        .codes
        .iter()
        .find(|r| r.gate == "cloud-ci-total-accounting" && r.code == "unjustified")
        .expect("unjustified code present");
    assert!(
        unjust
            .regressions
            .contains("SYNTHETIC/new-unjustified-file.rs"),
        "the synthetic NEW file must show up as a regression"
    );
    assert!(
        unjust.fails(),
        "a NEW unjustified file must FAIL the firewall"
    );
    assert!(
        !report.is_green(),
        "firewall must be RED on a NEW finite violation"
    );
}

/// RATCHET proof against the LIVE corpus: a regen that GROWS the baseline (without sign-off)
/// relative to the FROZEN merge-base reference is a ratchet_regression — debt cannot be
/// laundered into the baseline by re-running the producer.
#[test]
fn firewall_blocks_baseline_growth_without_signoff() {
    let root = repo_root();
    let frozen = load_frozen_baseline(&root);
    // A proposed baseline that ADDS a key beyond the frozen set.
    let mut proposed_value = regenerate_baseline(&root);
    if let Some(keys) = proposed_value
        .get_mut("gates")
        .and_then(|g| g.get_mut("cloud-ci-total-accounting"))
        .and_then(|g| g.get_mut("unjustified"))
        .and_then(|c| c.get_mut("keys"))
        .and_then(Value::as_array_mut)
    {
        keys.push(Value::String("SYNTHETIC/laundered-debt.rs".to_owned()));
    }
    let proposed = Baseline::from_value(&proposed_value).unwrap();
    let current: BTreeMap<String, BTreeMap<String, BTreeSet<String>>> = BTreeMap::new();

    // Empty sign-off => the grown key is NOT exempt.
    let report = evaluate_firewall(&frozen.baseline, &proposed, &current, &SignOff::default());
    assert!(
        report
            .ratchet_growth
            .iter()
            .any(|(_, code, key)| code == "unjustified" && key == "SYNTHETIC/laundered-debt.rs"),
        "growing the baseline without sign-off must be a ratchet_regression"
    );
    assert!(!report.is_green());
}
