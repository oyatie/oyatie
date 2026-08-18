//! Three shapes, mirroring `ci/facade/crate-catalog-coverage`:
//! 1. GREEN — today's corpus matches the frozen baseline.
//! 2. RED FIXTURE — a synthetic new `cloud-` name MUST fail. A gate only ever observed passing is
//!    not evidence of anything.
//! 3. FIDELITY — every frozen entry is still genuinely present, so the baseline is neither stale
//!    nor over-broad.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ci_cloud_name_ratchet::{compare, findings, parse_baseline};

fn repo_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        assert!(
            dir.pop(),
            "repository root marker not found above the crate"
        );
    }
}

/// Census of TRACKED paths.
///
/// Enumerated from `git ls-files`, not from a `read_dir` walk. Walking the filesystem made the
/// repository's primary `cargo test --workspace` result depend on local state: an untracked or
/// ignored `cloud-build/output/` counted as a finding even though the name cannot enter the
/// repository, so the gate could go red on a developer's machine and green in CI, or record
/// phantom baseline entries that then fail in a clean checkout. Only tracked paths can carry a
/// durable name, so only tracked paths are scanned.
fn census(root: &Path) -> BTreeSet<String> {
    let listing =
        git(root, &["ls-files", "-z"]).expect("git ls-files enumerates the tracked corpus");
    let mut out = BTreeSet::new();
    for relative in listing.split('\0').filter(|p| !p.is_empty()) {
        // `findings` splits on `/` and matches forward-slash prefixes and manifest suffixes.
        // git reports forward slashes on every platform, but normalize defensively so a
        // separator can never silently empty the census.
        let relative = relative.replace('\\', "/");
        let name = relative.rsplit('/').next().unwrap_or(&relative).to_owned();
        // Catalog YAML must be READ, not merely walked. Passing an empty string for it left the
        // `declared_capability` branch dead in the live gate — the scanner existed but was never
        // handed anything to scan, so a catalog row declaring `capability: cloud-new-service` was
        // invisible. That is exactly how this gate's own capability came to be minted as
        // `cloud-ci-*` while the gate stayed green.
        let contents = if matches!(name.as_str(), "Cargo.toml" | "Chart.yaml")
            || (relative.starts_with("registry/catalog/")
                && (name.ends_with(".yaml") || name.ends_with(".yml")))
        {
            std::fs::read_to_string(root.join(&relative)).unwrap_or_default()
        } else {
            String::new()
        };
        out.extend(findings(&relative, &contents));
    }
    out
}

const BASELINE_REPO_PATH: &str = "ci/facade/cloud-name-ratchet/cloud-name-baseline.json";
const PROTECTED_BASE_REF: &str = "origin/dev";

/// The baseline THIS change proposes — the committed working-tree copy.
///
/// Never the authority on what debt is *accepted*; that is [`protected_baseline`]. It is the
/// authority on what this change *claims*, which is what the burn-down check must measure
/// against, so that deleting a line is a remediation the author can actually perform.
fn candidate_baseline() -> BTreeSet<String> {
    parse_baseline(&candidate_baseline_text())
}

fn candidate_baseline_text() -> String {
    let path = repo_root().join(BASELINE_REPO_PATH);
    std::fs::read_to_string(&path).expect("the committed baseline is readable")
}

/// The accepted debt, as it stands on the PROTECTED MERGE-BASE — or `Bootstrap` on the single
/// change that introduces the file.
enum Protected {
    Frozen(BTreeSet<String>),
    Bootstrap,
}

/// Reading the working-tree copy made the ratchet optional: a change could add a forbidden
/// `cloud-` name AND add the same key to the baseline, and both the growth and burn-down
/// comparisons would see identical sets and pass. That is the same baseline-laundering failure
/// already recorded at `ci/facade/action-item-accounting/friction-ledger.jsonl:67`, and it is why
/// the sibling automation-language gate loads its baseline from the merge-base and deliberately
/// ignores the candidate's copy: a new tolerated key can only become accepted once a DISTINCT
/// protected-base change carries it forward.
///
/// FAIL CLOSED. An earlier revision returned `None` whenever *either* git command failed and then
/// fell back to the candidate copy, which made "this is the introducing change" and "git could not
/// answer" indistinguishable — in a shallow clone, a fork without `origin/dev`, or any git error,
/// the laundering path reopened silently. Only a resolvable merge-base whose tree genuinely lacks
/// the file is bootstrap; every other failure is an error.
fn protected_baseline() -> Protected {
    let root = repo_root();
    let merge_base = git(&root, &["merge-base", PROTECTED_BASE_REF, "HEAD"]).unwrap_or_else(|e| {
        panic!(
            "cannot resolve the protected merge-base against {PROTECTED_BASE_REF}, so the accepted \
             baseline is unknown and this gate CANNOT be evaluated: {e}\n\n\
             This is deliberately fatal rather than a fallback to the candidate copy, which would \
             let a change add a forbidden name and its own baseline key together.\n\n\
             In a checkout without the protected ref, fetch it:\n    git fetch origin dev"
        )
    });
    match git(
        &root,
        &[
            "show",
            &format!("{}:{BASELINE_REPO_PATH}", merge_base.trim()),
        ],
    ) {
        Ok(text) => Protected::Frozen(parse_baseline(&text)),
        // The merge-base resolved, so git is healthy; the path simply is not in that tree yet.
        Err(_) => {
            assert!(
                candidate_baseline_text().contains("\"_bootstrap\": true"),
                "the baseline is absent from the protected merge-base, so this is the introducing \
                 change; it must carry an explicit \"_bootstrap\": true marker so the carve-out is \
                 declared rather than assumed."
            );
            Protected::Bootstrap
        }
    }
}

fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| format!("git {args:?} could not be run: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    String::from_utf8(out.stdout).map_err(|e| format!("git {args:?} emitted non-UTF-8: {e}"))
}

/// GROWTH is measured against the PROTECTED baseline, so a candidate cannot authorize its own
/// new debt by editing the file it is checked against.
#[test]
fn the_deprecated_cloud_name_set_never_grows() {
    let accepted = match protected_baseline() {
        Protected::Frozen(set) => set,
        // Introducing change: there is no protected copy to compare against yet.
        Protected::Bootstrap => return,
    };
    let verdict = compare(&census(&repo_root()), &accepted);
    assert!(
        verdict.added.is_empty(),
        "NEW deprecated `cloud-` names beyond the frozen baseline:\n{}\n\n\
         `cloud-` is deprecated. Name the new thing without it. If a rename genuinely requires a \
         transitional name, that is a founder call, not a baseline edit.",
        verdict
            .added
            .iter()
            .map(|k| format!("  {k}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The other half of the laundering defence: growth-against-protected alone is defeated in two
/// steps. A first change adds a key to the baseline while leaving the corpus untouched — nothing
/// compares the candidate file to the protected one, so it passes — and once merged it has become
/// accepted debt, so a second change can introduce the matching forbidden name and pass too.
///
/// Additions to the baseline are therefore rejected outright. Only removals may be proposed,
/// which is the only direction this ratchet exists to allow.
#[test]
fn the_candidate_baseline_may_not_add_keys() {
    let accepted = match protected_baseline() {
        Protected::Frozen(set) => set,
        Protected::Bootstrap => return,
    };
    let candidate = candidate_baseline();
    let added: Vec<&String> = candidate.difference(&accepted).collect();
    assert!(
        added.is_empty(),
        "this change ADDS {} key(s) to {BASELINE_REPO_PATH}:\n{}\n\n\
         The baseline is shrink-only. Adding a key here pre-authorizes debt that a later change \
         can then introduce and pass against — the two-step laundering path. Remove the deprecated \
         name instead; if a transitional name is genuinely required, that is a founder call.",
        added.len(),
        added
            .iter()
            .map(|k| format!("    {k:?},"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn burn_down_must_be_recorded_in_the_same_change() {
    // Shrink is the point, but the frozen file must not overstate the remaining debt: a rename
    // without a baseline edit leaves a phantom entry that hides the next real one.
    //
    // The remediation is a DECLARATIVE EDIT to the committed baseline, not a command. There is
    // deliberately no `--fix` binary and no regeneration CLI: the repository retires CLI surfaces,
    // and — more specifically — a one-command "regenerate my own baseline" tool is exactly the
    // laundering affordance `protected_baseline` exists to close, handed back in convenient form.
    // Emitting the removals is safe in a way that regeneration is not: this test has already
    // PROVEN each key below is absent from today's census, so applying them can only shrink the
    // file. Growth is what requires a distinct protected-base change.
    //
    // Measured against the CANDIDATE, not the protected copy. Measuring burn-down against the
    // merge-base made this test unsatisfiable: a legitimate rename dropped the key from the census
    // while the protected copy still held it, so `removed` stayed non-empty no matter what the
    // author did — and the instructed fix, deleting the line, changed a file the assertion never
    // read. Growth is what must resist the candidate's own edits; burn-down must respond to them.
    let verdict = compare(&census(&repo_root()), &candidate_baseline());
    assert!(
        verdict.removed.is_empty(),
        "{} baselined name(s) are gone — record the burn-down in THIS change by deleting these \
         exact lines from `cloud_prefixed_names` in {BASELINE_REPO_PATH}:\n{}",
        verdict.removed.len(),
        verdict
            .removed
            .iter()
            .map(|k| format!("    {k:?},"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn red_fixture_a_new_cloud_name_fails_closed() {
    let base = candidate_baseline();
    let mut synthetic = census(&repo_root());
    synthetic.insert("dir:secrets/cloud-brand-new-service".to_owned());
    let verdict = compare(&synthetic, &base);
    assert_eq!(
        verdict.added,
        BTreeSet::from(["dir:secrets/cloud-brand-new-service".to_owned()]),
        "a newly introduced `cloud-` name must be caught"
    );
}

#[test]
fn the_baseline_is_not_empty_and_is_shaped_as_expected() {
    let base = candidate_baseline();
    // NO minimum size. The bootstrap-era `> 100` floor would have failed `cargo test --workspace`
    // the moment legitimate renames took the baseline below 100 — permanently blocking the very
    // burn-down this ratchet exists to drive, and blocking zero outright. Shape is what must hold
    // at every size, including empty.
    assert!(
        base.iter()
            .all(|k| k.starts_with("dir:") || k.starts_with("name:")),
        "every key must name its rename unit"
    );
    assert!(
        base.iter().all(|k| !k.trim().is_empty() && k.trim() == k),
        "keys must be exact, untrimmed tokens"
    );
    assert!(
        base.iter().all(|k| k.split(':').count() >= 2),
        "every key must carry its kind and its subject"
    );
}

// --- Frozen-path fixtures -------------------------------------------------------------------
//
// On the change that INTRODUCES this gate the baseline is absent from the merge-base, so
// `protected_baseline()` reports `Bootstrap` and the growth and no-add checks above early-return.
// Their live green is therefore not evidence that they work — the same objection this file's own
// header raises about gates only ever observed passing.
//
// These exercise the comparison logic directly over synthetic sets, so the Frozen path is proven
// on the very change that ships it rather than first being exercised after merge.

fn set(keys: &[&str]) -> BTreeSet<String> {
    keys.iter().map(|k| (*k).to_owned()).collect()
}

/// Growth is judged against the PROTECTED set, so adding the key to the candidate cannot excuse
/// the name. This is the one-step laundering attempt.
#[test]
fn frozen_path_growth_is_caught_even_when_the_candidate_baselines_it() {
    let protected = set(&["dir:legacy/cloud-old"]);
    let census_now = set(&["dir:legacy/cloud-old", "dir:secrets/cloud-brand-new"]);
    let verdict = compare(&census_now, &protected);
    assert_eq!(
        verdict.added,
        set(&["dir:secrets/cloud-brand-new"]),
        "a new deprecated name must be caught against the protected baseline"
    );
}

/// The two-step laundering attempt: a first change pre-seeds the baseline with no corpus change.
/// Judged against the protected set, the candidate's extra key is visible and rejected.
#[test]
fn frozen_path_candidate_pre_seeding_is_caught() {
    let protected = set(&["dir:legacy/cloud-old"]);
    let candidate = set(&["dir:legacy/cloud-old", "dir:secrets/cloud-brand-new"]);
    let added: Vec<&String> = candidate.difference(&protected).collect();
    assert_eq!(
        added,
        vec!["dir:secrets/cloud-brand-new"],
        "a key added to the baseline without a corresponding corpus change must be rejected"
    );
    let honest: Vec<&String> = protected.difference(&protected).collect();
    assert!(honest.is_empty(), "an unchanged baseline must pass");
}

/// Burn-down is judged against the CANDIDATE, so deleting the line is a remediation that works.
/// Judged against the protected copy it never would, because the protected copy still holds the key.
#[test]
fn frozen_path_burn_down_responds_to_the_candidate_edit() {
    let protected = set(&["dir:legacy/cloud-old", "dir:legacy/cloud-gone"]);
    let census_now = set(&["dir:legacy/cloud-old"]);

    // Author renamed `cloud-gone` away but has not yet recorded it: burn-down must complain.
    let before = compare(&census_now, &protected);
    assert_eq!(
        before.removed,
        set(&["dir:legacy/cloud-gone"]),
        "an unrecorded burn-down must be reported"
    );

    // Author deletes that one line from the candidate baseline, as the failure message instructs.
    let candidate_after = set(&["dir:legacy/cloud-old"]);
    let after = compare(&census_now, &candidate_after);
    assert!(
        after.removed.is_empty(),
        "deleting the line must satisfy the burn-down check — measuring it against the protected \
         copy instead made the check unsatisfiable, because that copy still holds the key"
    );
    assert!(
        after.added.is_empty(),
        "recording a burn-down must not register as growth"
    );
}

/// The fail-closed contract: a git failure must surface as an error, never as a silent fallback
/// to the candidate copy.
#[test]
fn git_failures_are_errors_rather_than_silent_fallbacks() {
    let root = repo_root();
    assert!(
        git(&root, &["show", "definitely-not-a-real-object:nope"]).is_err(),
        "a failed git command must return Err so the caller can fail closed"
    );
    assert!(
        git(&root, &["rev-parse", "HEAD"]).is_ok(),
        "a healthy git command must still succeed"
    );
}
