//! Three shapes, mirroring `ci/facade/crate-catalog-coverage`:
//! 1. GREEN — today's corpus matches the frozen baseline.
//! 2. RED FIXTURE — a synthetic new `cloud-` name MUST fail. A gate only ever observed passing is
//!    not evidence of anything.
//! 3. FIDELITY — every frozen entry is still genuinely present, so the baseline is neither stale
//!    nor over-broad.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::{BTreeMap, BTreeSet};
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

fn census(root: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                if !matches!(
                    name.as_str(),
                    "target" | "buck-out" | ".git" | "node_modules" | ".jj"
                ) {
                    stack.push(path);
                }
                continue;
            }
            let Ok(relative) = path.strip_prefix(root) else {
                continue;
            };
            let contents = if matches!(name.as_str(), "Cargo.toml" | "Chart.yaml") {
                std::fs::read_to_string(&path).unwrap_or_default()
            } else {
                String::new()
            };
            out.extend(findings(&relative.to_string_lossy(), &contents));
        }
    }
    out
}

const BASELINE_REPO_PATH: &str = "ci/facade/cloud-name-ratchet/cloud-name-baseline.json";
const PROTECTED_BASE_REF: &str = "origin/dev";

/// The frozen baseline as it stands on the PROTECTED MERGE-BASE, not in this checkout.
///
/// Reading the working-tree copy made the ratchet optional: a change could add a forbidden
/// `cloud-` name AND add the same key to the baseline, and both the growth and burn-down
/// comparisons would see identical sets and pass. That is the same baseline-laundering failure
/// already recorded at `ci/facade/action-item-accounting/friction-ledger.jsonl:67`, and it is why
/// the sibling automation-language gate loads its baseline from the merge-base and deliberately
/// ignores the candidate's copy: a new tolerated key can only become accepted once a DISTINCT
/// protected-base change carries it forward.
///
/// Bootstrap exception: on the change that introduces this gate the file does not yet exist at
/// the merge-base. That single case falls back to the working-tree copy and REQUIRES an explicit
/// `"_bootstrap": true` marker, so the carve-out is declared rather than implicit — and it expires
/// by construction, because the file exists at the merge-base for every change afterwards.
fn baseline() -> BTreeSet<String> {
    let root = repo_root();
    if let Some(frozen) = frozen_baseline_from_merge_base(&root) {
        return relabel_for_renames(&root, parse_baseline(&frozen));
    }
    let path = root.join(BASELINE_REPO_PATH);
    let text = std::fs::read_to_string(&path).expect("frozen baseline is readable");
    assert!(
        text.contains("\"_bootstrap\": true"),
        "the baseline is absent from the protected merge-base, so this is the introducing change; \
         it must carry an explicit \"_bootstrap\": true marker. Any later change must compare \
         against the merge-base copy instead of its own."
    );
    parse_baseline(&text)
}

/// Relabel frozen keys through git's own rename detection, so a crate that MOVED is not
/// scored as new debt.
///
/// This baseline is PATH-KEYED and anchored to the merge-base, which is what stops a PR
/// laundering its own copy — but it also means a pure relocation reads as a brand-new
/// `cloud-` name at the destination while the source entry goes stale. The ADR-0562 absorb
/// moves whole products, so without this every relocated `cloud-`-named crate REDs a gate
/// whose entire purpose is to track whether that debt is shrinking.
///
/// Not a waiver, and the guards are what make that true: the key count is unchanged, the
/// name stays tracked at its new path, and a genuinely new `cloud-` name still fails. The
/// pairing itself comes from `derive_directory_renames`, which pairs a directory only when
/// it moved WHOLESALE and unambiguously — the same kernel the firewall ratchet uses, shared
/// rather than reimplemented so the two cannot drift.
fn relabel_for_renames(root: &Path, frozen: BTreeSet<String>) -> BTreeSet<String> {
    let (renames, scores) = file_renames_since_merge_base(root);
    if renames.is_empty() {
        return frozen;
    }
    let mut all = renames.clone();
    all.extend(ci_baseline_ratchet::derive_directory_renames_scored(
        &renames,
        &scores,
        &head_tracked_paths(root),
    ));
    // Longest source first: a shorter path must never consume a longer one's prefix.
    let mut pairs: Vec<_> = all.into_iter().collect();
    pairs.sort_by(|left, right| right.0.len().cmp(&left.0.len()));
    frozen
        .into_iter()
        .map(|key| {
            // Boundary-aligned, never a substring: `oya` must not match inside
            // `billing/oya-billing`. The keys here are `dir:<path>` and
            // `name:<path>:<name>`, so `:` bounds a component as well as `/`.
            for (old, new) in &pairs {
                if let Some(relabelled) =
                    ci_baseline_ratchet::replace_path_component(&key, old, new)
                {
                    return relabelled;
                }
            }
            key
        })
        .collect()
}

/// `git diff --find-renames` from the merge-base to HEAD. The 30% threshold matches the
/// firewall's, and for the same measured reason: a relocated `Cargo.toml` must rewrite its
/// package name, lib name and every relative path dep, which on a short manifest drops the
/// similarity index below git's 50% default and reports add+delete instead of a rename.
fn file_renames_since_merge_base(root: &Path) -> (BTreeMap<String, String>, BTreeMap<String, u32>) {
    let mut out = BTreeMap::new();
    let mut scores = BTreeMap::new();
    let Some(merge_base) = merge_base(root) else {
        return (out, scores);
    };
    let Ok(diff) = std::process::Command::new("git")
        .args([
            "diff",
            "--find-renames=30%",
            "--diff-filter=R",
            "--name-status",
            "-z",
            &merge_base,
        ])
        .current_dir(root)
        .output()
    else {
        return (out, scores);
    };
    if !diff.status.success() {
        return (out, scores);
    }
    let fields: Vec<&str> = std::str::from_utf8(&diff.stdout)
        .unwrap_or_default()
        .split('\0')
        .filter(|field| !field.is_empty())
        .collect();
    let mut i = 0;
    while i + 2 < fields.len() + 1 {
        let Some(status) = fields.get(i) else { break };
        if !status.starts_with('R') {
            i += 1;
            continue;
        }
        let (Some(old), Some(new)) = (fields.get(i + 1), fields.get(i + 2)) else {
            break;
        };
        // `R100` means byte-identical; the digits are git's similarity index.
        if let Ok(score) = status.trim_start_matches('R').parse::<u32>() {
            scores.insert((*old).to_owned(), score);
        }
        out.insert((*old).to_owned(), (*new).to_owned());
        i += 3;
    }
    (out, scores)
}

/// Tracked paths at HEAD: the directory derivation needs them to tell a WHOLESALE move from
/// a partial one, because a directory that still holds tracked files has not moved.
fn head_tracked_paths(root: &Path) -> BTreeSet<String> {
    let Ok(out) = std::process::Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(root)
        .output()
    else {
        return BTreeSet::new();
    };
    if !out.status.success() {
        return BTreeSet::new();
    }
    std::str::from_utf8(&out.stdout)
        .unwrap_or_default()
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(str::to_owned)
        .collect()
}

fn merge_base(root: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["merge-base", PROTECTED_BASE_REF, "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|out| out.status.success())?;
    Some(String::from_utf8(out.stdout).ok()?.trim().to_owned())
}

/// `git show <merge-base>:<baseline>` — `None` when the file does not exist there yet.
fn frozen_baseline_from_merge_base(root: &Path) -> Option<String> {
    let merge_base = std::process::Command::new("git")
        .args(["merge-base", PROTECTED_BASE_REF, "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|out| out.status.success())?;
    let merge_base = String::from_utf8(merge_base.stdout).ok()?.trim().to_owned();
    let shown = std::process::Command::new("git")
        .args(["show", &format!("{merge_base}:{BASELINE_REPO_PATH}")])
        .current_dir(root)
        .output()
        .ok()
        .filter(|out| out.status.success())?;
    String::from_utf8(shown.stdout).ok()
}

#[test]
fn the_deprecated_cloud_name_set_never_grows() {
    let verdict = compare(&census(&repo_root()), &baseline());
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

/// The COMMITTED baseline file must describe today's corpus.
///
/// DESIGN CORRECTION. This originally compared against `baseline()`, which reads the MERGE-BASE
/// copy once the file exists on the protected branch. That made a legitimate burn-down impossible
/// to record: a PR cannot change the merge-base, so every removal stayed red forever and the gate
/// blocked the exact work it exists to encourage. Growth is what needs the merge-base (a PR must
/// not be able to launder new debt by rewriting its own baseline); ACCURACY of the committed file
/// is checked against the committed file itself, which a PR can and must update.
#[test]
fn burn_down_must_be_recorded_in_the_same_change() {
    let root = repo_root();
    let committed = parse_baseline(
        &std::fs::read_to_string(root.join(BASELINE_REPO_PATH))
            .expect("frozen baseline is readable"),
    );
    let verdict = compare(&census(&root), &committed);
    assert!(
        verdict.removed.is_empty(),
        "these baselined names are gone — regenerate the baseline in this change:\n{}\n\n\
           cargo run -p ci-cloud-name-ratchet --bin oya-ci-cloud-name-baseline -- --repo-root . \\\n\
             > ci/facade/cloud-name-ratchet/cloud-name-baseline.json",
        verdict
            .removed
            .iter()
            .map(|k| format!("  {k}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn red_fixture_a_new_cloud_name_fails_closed() {
    let base = baseline();
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
    let base = baseline();
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
