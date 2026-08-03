//! Committed move-plan DISCOVERY + fail-closed multi-plan guard for the `manifest`
//! materialization (task #65).
//!
//! A MOVE PR commits exactly ONE plan at `specs/reorg/<capability>-move-plan.json`; the committed
//! move-manifest is then a pure function of `(committed plan + candidate tree)`. The materialization
//! step previously took the FIRST glob match silently when more than one plan was committed
//! (`move_plans[0]`), so a contributor error — two plans landing in one PR — would produce a
//! manifest derived from an arbitrary one of them and pass the gate green. That is a non-deterministic
//! destination the engine cannot resolve, so it must FAIL-CLOSED.
//!
//! [`discover_committed_move_plans`] enumerates the glob deterministically (sorted). [`select_move_plan`]
//! applies the policy: zero plans -> `None` (a no-move PR emits the canonical EMPTY manifest), exactly
//! one -> that plan, more than one -> [`CodemodError::MultipleMovePlans`] (hard error). The
//! enumeration is a pure function of the directory listing (no transform decision rides on it), so it
//! stays deterministic and unit-testable.

use std::path::{Path, PathBuf};

use crate::model::{CodemodError, MovePlan};

/// The conventional committed-plan directory, repo-relative.
pub const REORG_PLAN_DIR: &str = "specs/reorg";

/// The committed-plan filename suffix the materialization globs (`<capability>-move-plan.json`).
pub const MOVE_PLAN_SUFFIX: &str = "-move-plan.json";

/// Enumerate every committed `specs/reorg/*-move-plan.json` under `repo_root`, sorted by file name
/// for deterministic ordering. Returns an empty vec when the dir is absent (a no-move PR) — that is
/// not an error. Only the IO of listing the directory can fail; a missing dir is treated as empty.
pub fn discover_committed_move_plans(repo_root: &Path) -> Result<Vec<PathBuf>, CodemodError> {
    let dir = repo_root.join(REORG_PLAN_DIR);
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        // A missing reorg dir means no committed plans (a fresh tree / no-move PR), not an error.
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => {
            return Err(CodemodError::Io {
                context: format!("read_dir {}", dir.display()),
                message: e.to_string(),
            })
        }
    };
    let mut plans: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| CodemodError::Io {
            context: format!("read_dir entry in {}", dir.display()),
            message: e.to_string(),
        })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = match path.file_name().and_then(|s| s.to_str()) {
            Some(n) => n,
            None => continue,
        };
        // Match `<capability>-move-plan.json` with a NON-EMPTY capability stem, so the suffix file
        // itself (`-move-plan.json`) or the regenerated `move-manifest.generated.json` never counts.
        if name.ends_with(MOVE_PLAN_SUFFIX) && name.len() > MOVE_PLAN_SUFFIX.len() {
            plans.push(path);
        }
    }
    plans.sort();
    Ok(plans)
}

/// Apply the materialization plan-selection policy to a discovered set:
/// * 0 plans -> `Ok(None)` (a no-move PR emits the canonical EMPTY manifest);
/// * 1 plan  -> `Ok(Some(path))`;
/// * more than one plan -> `Err(CodemodError::MultipleMovePlans)` — fail-closed (#65), because a
///   single PR with multiple committed plans is ambiguous and must not silently first-win.
pub fn select_move_plan(plans: &[PathBuf]) -> Result<Option<PathBuf>, CodemodError> {
    match plans {
        [] => Ok(None),
        [only] => Ok(Some(only.clone())),
        many => Err(CodemodError::MultipleMovePlans {
            count: many.len(),
            paths: many
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>(),
        }),
    }
}

/// Discover the committed plans under `repo_root` AND apply the selection policy in one call — the
/// authoritative entry point the `manifest` materialization uses to pick the single committed plan
/// (or fail-closed on more than one). When the caller passes an EXPLICIT plan path the materialization
/// uses that path directly, but it STILL runs this guard so a >1-plan tree fails-closed regardless of
/// which plan was named (the candidate tree is ambiguous, full stop).
pub fn resolve_committed_move_plan(repo_root: &Path) -> Result<Option<PathBuf>, CodemodError> {
    let plans = discover_committed_move_plans(repo_root)?;
    select_move_plan(&plans)
}

/// Resolve the EFFECTIVE plan path the `manifest` materialization should load, given an OPTIONAL
/// explicit `--plan` and the committed candidate tree. This is the precedence the materialization
/// runs verbatim; it lives here (not in `main.rs`) so the fail-closed wiring is unit-testable rather
/// than reachable only through the binary:
/// * the multi-committed-plan guard ([`resolve_committed_move_plan`]) runs FIRST and REGARDLESS of
///   `--plan` — an ambiguous candidate tree is rejected even when a specific plan is named;
/// * then `explicit.or(discovered)` — an explicit `--plan` wins over the single committed plan, and
///   with no `--plan` the codemod itself SELECTS the single committed plan (the materialization is
///   the authority), so a no-move PR (zero plans) still resolves to `None` (canonical empty manifest).
pub fn resolve_effective_move_plan(
    explicit: Option<PathBuf>,
    repo_root: &Path,
) -> Result<Option<PathBuf>, CodemodError> {
    let discovered = resolve_committed_move_plan(repo_root)?;
    Ok(explicit.or(discovered))
}

/// A committed plan is ALREADY LANDED iff it declares >=1 move and EVERY move's old crate-dir is
/// absent from the merge-base tree (the move is in immutable history). An empty-moves plan is NOT
/// landed (a degenerate no-op, never inert).
///
/// MUST-PASS #5 (straddle DoS): a merged move-plan that a prior PR never cleaned up would otherwise
/// trip the single-plan guard and HARD-ERROR every subsequent PR's manifest materialization. A
/// landed plan is INERT — it contributes no manifest pairs (the resolver reads the move's NEW name,
/// present at the merge-base) — so scoping its lifetime this way (excluding it BEFORE the count
/// guard) makes a merged plan self-heal without a per-PR cleanup lag. Laundering-safe: excluding a
/// plan only REMOVES relabel pairs (never adds one), and a genuinely-pending plan (any old crate-dir
/// still present at the merge-base) is NEVER excluded, so the >1-PENDING-move fail-closed guard is
/// preserved (and sharpened — it no longer false-positives on stale-landed leftovers).
/// The paths a plan contributes to the LANDED probe: crate-move old dirs PLUS artifact old paths.
///
/// Exists as its own function because the defect it fixes lived in the caller, not in
/// `plan_is_landed`. An ARTIFACT-ONLY plan (`moves: []`) previously produced an EMPTY probe input,
/// and `plan_is_landed` is false for empty input — so the plan stayed ACTIVE forever, could never
/// self-heal, and a second one would raise `MultipleMovePlans` from step 1 of the universal
/// materializer (fail-closed, every CI leg and every local gate lane), wedging every subsequent PR.
/// `model.rs` already blessed the artifact-only shape; only this probe was never extended to it.
///
/// Takes the WHOLE plan rather than two path slices on purpose: the defect was a caller deriving
/// the probe input from `moves` alone, so a signature that still lets a caller pass `&[]` for the
/// artifact side leaves the same defect reachable. One argument, no way to under-supply it.
pub fn plan_probe_paths(plan: &MovePlan) -> Vec<String> {
    plan.moves
        .iter()
        .map(|m| m.old_path.clone())
        .chain(plan.artifacts.iter().map(|a| a.old_path.clone()))
        .collect()
}

/// The probe is FALLIBLE on purpose. It answers "is this old path absent at the merge-base?", and
/// a `bool` cannot say "I could not tell" — so an unresolvable merge-base used to be coerced into
/// `false` ("present"), which reads as "pending", which makes EVERY committed plan ACTIVE, which
/// surfaces as `MultipleMovePlans` from the universal materializer. `Result` makes that coercion
/// unrepresentable: uncertainty must be either answered or reported.
pub fn plan_is_landed(
    old_crate_dirs: &[String],
    old_dir_absent_at_merge_base: &impl Fn(&str) -> Result<bool, CodemodError>,
) -> Result<bool, CodemodError> {
    if old_crate_dirs.is_empty() {
        return Ok(false);
    }
    for dir in old_crate_dirs {
        if !old_dir_absent_at_merge_base(dir)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Select the single ACTIVE (non-landed) committed plan, applying the fail-closed single-plan guard
/// to the active set only. `load_old_crate_dirs` yields a plan's move old crate-dirs (git/parse in
/// prod, a fake in tests); `old_dir_absent_at_merge_base` probes immutable history.
///
/// The probe degrades in two DIFFERENT directions on purpose. A per-path git failure at a resolved
/// rev is LOCAL uncertainty: it answers `Ok(false)`/present, so that plan stays ACTIVE and the
/// guard stays sharp. A merge-base that does not resolve at all is a GLOBAL input failure — no plan
/// can be classified — so the probe returns `Err` and this selector propagates it verbatim instead
/// of mislabelling every landed plan as active and raising a bogus `MultipleMovePlans`.
///
/// An EMPTY plan set never calls the probe, so a no-move PR stays green on a ref-less checkout.
pub fn select_active_move_plan<L, A>(
    plans: &[PathBuf],
    load_old_crate_dirs: L,
    old_dir_absent_at_merge_base: A,
) -> Result<Option<PathBuf>, CodemodError>
where
    L: Fn(&Path) -> Result<Vec<String>, CodemodError>,
    A: Fn(&str) -> Result<bool, CodemodError>,
{
    let mut active: Vec<PathBuf> = Vec::new();
    for plan in plans {
        let old_dirs = load_old_crate_dirs(plan)?;
        if !plan_is_landed(&old_dirs, &old_dir_absent_at_merge_base)? {
            active.push(plan.clone());
        }
    }
    select_move_plan(&active)
}

/// Discover committed plans and select the single ACTIVE one (excluding already-landed plans),
/// then apply the explicit-`--plan` precedence — the merge-base-aware analogue of
/// [`resolve_effective_move_plan`] the `manifest` materialization runs.
pub fn resolve_effective_active_move_plan<L, A>(
    explicit: Option<PathBuf>,
    repo_root: &Path,
    load_old_crate_dirs: L,
    old_dir_absent_at_merge_base: A,
) -> Result<Option<PathBuf>, CodemodError>
where
    L: Fn(&Path) -> Result<Vec<String>, CodemodError>,
    A: Fn(&str) -> Result<bool, CodemodError>,
{
    let discovered = discover_committed_move_plans(repo_root)?;
    let active = select_active_move_plan(&discovered, load_old_crate_dirs, old_dir_absent_at_merge_base)?;
    Ok(explicit.or(active))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn tmp_root(tag: &str) -> PathBuf {
        let unique = format!(
            "oya-reorg-manifest-{}-{}-{}",
            tag,
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let root = std::env::temp_dir().join(unique);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    fn write_plan(root: &Path, name: &str) {
        let dir = root.join(REORG_PLAN_DIR);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(name),
            "{\"capability\":\"x\",\"moves\":[],\"artifacts\":[]}\n",
        )
        .unwrap();
    }

    #[test]
    fn no_plans_is_ok_none() {
        let root = tmp_root("none");
        // The reorg dir does not exist at all -> a no-move PR, not an error.
        assert!(resolve_committed_move_plan(&root).unwrap().is_none());
        // An EMPTY reorg dir is also a no-move PR.
        std::fs::create_dir_all(root.join(REORG_PLAN_DIR)).unwrap();
        assert!(resolve_committed_move_plan(&root).unwrap().is_none());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn exactly_one_plan_is_selected() {
        let root = tmp_root("one");
        write_plan(&root, "billing-move-plan.json");
        // The regenerated manifest + an unrelated json must NOT count as plans.
        let dir = root.join(REORG_PLAN_DIR);
        std::fs::write(dir.join("move-manifest.generated.json"), "{}\n").unwrap();
        std::fs::write(dir.join("parked-move-plan.PARKED.json"), "{}\n").unwrap();
        std::fs::write(dir.join("notes.json"), "{}\n").unwrap();
        let selected = resolve_committed_move_plan(&root).unwrap().unwrap();
        assert_eq!(selected.file_name().unwrap(), "billing-move-plan.json");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// #65: two committed move-plans in one PR is a contributor error the materialization must NOT
    /// silently first-win on. It must HARD-ERROR (fail-closed) so the ambiguous candidate tree is
    /// rejected by the gate rather than producing a manifest derived from an arbitrary plan.
    #[test]
    fn more_than_one_plan_hard_errors_fail_closed() {
        let root = tmp_root("two");
        write_plan(&root, "billing-move-plan.json");
        write_plan(&root, "iam-move-plan.json");
        let err = resolve_committed_move_plan(&root).unwrap_err();
        match &err {
            CodemodError::MultipleMovePlans { count, paths } => {
                assert_eq!(*count, 2, "both plans counted");
                assert_eq!(paths.len(), 2);
                // Deterministically sorted (billing before iam).
                assert!(paths[0].ends_with("billing-move-plan.json"), "{paths:?}");
                assert!(paths[1].ends_with("iam-move-plan.json"), "{paths:?}");
            }
            other => panic!("expected MultipleMovePlans, got {other:?}"),
        }
        // The Display message names the count + paths (operator-actionable).
        let msg = err.to_string();
        assert!(msg.contains("more than one"), "{msg}");
        assert!(msg.contains("billing-move-plan.json"), "{msg}");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// MED-3 (#769 review): the materialization's plan-resolution precedence + fail-closed wiring
    /// (formerly inline in `main.rs`, untested) is now a pure-lib function locked here against all
    /// six (explicit Some/None) x (0/1/2 committed plans) combinations.
    #[test]
    fn effective_plan_precedence_and_fail_closed() {
        let explicit = PathBuf::from("/some/where/explicit-move-plan.json");

        // 0 committed plans.
        let root0 = tmp_root("eff0");
        // no `--plan`, no committed plan -> None (no-move PR, canonical empty manifest).
        assert!(resolve_effective_move_plan(None, &root0).unwrap().is_none());
        // explicit `--plan`, no committed plan -> the explicit path wins.
        assert_eq!(
            resolve_effective_move_plan(Some(explicit.clone()), &root0)
                .unwrap()
                .unwrap(),
            explicit
        );
        let _ = std::fs::remove_dir_all(&root0);

        // 1 committed plan.
        let root1 = tmp_root("eff1");
        write_plan(&root1, "billing-move-plan.json");
        // no `--plan` -> the codemod SELECTS the single committed plan.
        assert_eq!(
            resolve_effective_move_plan(None, &root1)
                .unwrap()
                .unwrap()
                .file_name()
                .unwrap(),
            "billing-move-plan.json"
        );
        // explicit `--plan` -> explicit WINS over the single committed plan (its presence is fine).
        assert_eq!(
            resolve_effective_move_plan(Some(explicit.clone()), &root1)
                .unwrap()
                .unwrap(),
            explicit
        );
        let _ = std::fs::remove_dir_all(&root1);

        // 2 committed plans -> FAIL-CLOSED regardless of `--plan` (ambiguous candidate tree).
        let root2 = tmp_root("eff2");
        write_plan(&root2, "billing-move-plan.json");
        write_plan(&root2, "iam-move-plan.json");
        assert!(matches!(
            resolve_effective_move_plan(None, &root2),
            Err(CodemodError::MultipleMovePlans { count: 2, .. })
        ));
        // The guard fires EVEN WITH an explicit `--plan` — naming one plan does not disambiguate the
        // tree, so the materialization still refuses rather than silently first-winning.
        assert!(matches!(
            resolve_effective_move_plan(Some(explicit.clone()), &root2),
            Err(CodemodError::MultipleMovePlans { count: 2, .. })
        ));
        let _ = std::fs::remove_dir_all(&root2);
    }

    /// MUST-PASS #5: a plan whose every move old-dir is absent at the merge-base is LANDED (inert);
    /// an empty-moves plan is NOT landed; a plan with any pending old-dir is NOT landed.
    #[test]
    fn plan_is_landed_semantics() {
        let absent = |_d: &str| Ok(true); // everything absent at merge-base
        let present = |_d: &str| Ok(false); // everything present at merge-base
        assert!(
            plan_is_landed(&["old/a".to_owned()], &absent).unwrap(),
            "all-absent => landed"
        );
        assert!(
            !plan_is_landed(&["old/a".to_owned()], &present).unwrap(),
            "old dir still present => pending, not landed"
        );
        assert!(
            !plan_is_landed(&[], &absent).unwrap(),
            "empty-moves plan is never landed"
        );
        // Mixed: one dir landed, one still pending => the plan is NOT landed (fail toward pending).
        let only_a_absent = |d: &str| Ok(d == "old/a");
        assert!(
            !plan_is_landed(&["old/a".to_owned(), "old/b".to_owned()], &only_a_absent).unwrap(),
            "any pending old dir keeps the plan active"
        );
    }

    /// GIT UNCERTAINTY MUST NOT WEDGE THE REPO — the regression pin.
    ///
    /// When the merge-base does not resolve, the probe cannot classify ANY plan. The selector must
    /// surface THAT input failure, naming the ref that would not resolve. Before the fix the probe
    /// was a bare `bool` and `None` was coerced to `false` ("still present" => "pending"), so all N
    /// committed-and-landed plans read ACTIVE and this call returned
    /// `Err(MultipleMovePlans { count: N })` — raised from step 1 of the universal materializer, so
    /// fail-closed on every CI leg and every local gate lane, repo-wide, under a message that named
    /// the wrong problem. Reproduced end-to-end on a clone with no `refs/remotes/origin/dev`:
    /// `MultipleMovePlans { count: 7 }`.
    ///
    /// The `n = 7` here is the live `specs/reorg/` plan count at the time of the fix, so the pin
    /// exercises the exact shape that wedged.
    #[test]
    fn unresolvable_merge_base_reports_itself_not_multiple_move_plans() {
        let plans: Vec<PathBuf> = (0..7)
            .map(|i| PathBuf::from(format!("specs/reorg/p{i}-move-plan.json")))
            .collect();
        let load = |_p: &Path| Ok(vec!["cloud/cloud-iam/slos/iam.openslo.yaml".to_owned()]);
        // Exactly what main.rs's probe does when `git merge-base origin/dev HEAD` yields nothing.
        let unresolvable = |_d: &str| {
            Err(CodemodError::MergeBaseUnresolved {
                base_ref: "origin/dev".to_owned(),
            })
        };
        match select_active_move_plan(&plans, load, unresolvable) {
            Err(CodemodError::MergeBaseUnresolved { base_ref }) => {
                assert_eq!(base_ref, "origin/dev", "the error must name the ref");
            }
            other => panic!("expected MergeBaseUnresolved, got {other:?}"),
        }

        // ...and the message must point at the CHECKOUT, not at deleting move plans.
        let rendered = CodemodError::MergeBaseUnresolved {
            base_ref: "origin/dev".to_owned(),
        }
        .to_string();
        assert!(rendered.contains("origin/dev"), "{rendered}");
        assert!(rendered.contains("do NOT delete move plans"), "{rendered}");
    }

    /// The same ref-less checkout on a NO-MOVE PR (zero committed plans) stays green: with nothing
    /// to classify the probe is never consulted, so the fix cannot turn an unresolvable merge-base
    /// into a new repo-wide wedge of its own.
    #[test]
    fn unresolvable_merge_base_with_no_plans_is_still_none() {
        let load = |_p: &Path| Ok(vec!["never/called".to_owned()]);
        let unresolvable = |_d: &str| {
            Err(CodemodError::MergeBaseUnresolved {
                base_ref: "origin/dev".to_owned(),
            })
        };
        assert!(select_active_move_plan(&[], load, unresolvable)
            .expect("a no-move PR must not need a merge-base")
            .is_none());
    }

    fn crate_move(old: &str) -> crate::model::CrateMove {
        crate::model::CrateMove {
            old_path: old.to_owned(),
            new_path: format!("moved/{old}"),
            old_cargo_name: "oya-old".to_owned(),
            new_cargo_name: "new".to_owned(),
        }
    }

    fn artifact_only(old: &str) -> MovePlan {
        MovePlan {
            capability: "backfill".to_owned(),
            moves: vec![],
            artifacts: vec![crate::model::ArtifactMove {
                old_path: old.to_owned(),
                new_path: format!("moved/{old}"),
            }],
        }
    }

    /// The probe input is derived from the WHOLE plan: crate-move old dirs PLUS artifact old paths.
    /// An artifact-ONLY plan (`moves: []`) must contribute its artifact old paths — deriving the
    /// probe from `moves` alone yields EMPTY, and `plan_is_landed` is false for empty, so the plan
    /// would be ACTIVE FOREVER and could never self-heal.
    #[test]
    fn artifact_only_plan_probes_its_artifact_paths() {
        let plan = artifact_only("cloud/cloud-iam/observability/slos/iam.openslo.yaml");
        let probe = plan_probe_paths(&plan);
        assert_eq!(
            probe,
            vec!["cloud/cloud-iam/observability/slos/iam.openslo.yaml".to_owned()],
            "an artifact-only plan must contribute a NON-EMPTY probe input"
        );
        let absent = |_p: &str| Ok(true);
        assert!(
            plan_is_landed(&probe, &absent).unwrap(),
            "an artifact-only plan whose old paths are absent at the merge-base IS landed"
        );

        // Mixed plan: both sides contribute, moves first (deterministic order).
        let mixed = MovePlan {
            capability: "iam".to_owned(),
            moves: vec![crate_move("libs/oya-iam-domain")],
            artifacts: vec![crate::model::ArtifactMove {
                old_path: "cloud/cloud-iam/slos/iam.openslo.yaml".to_owned(),
                new_path: "iam/slos/iam.openslo.yaml".to_owned(),
            }],
        };
        assert_eq!(
            plan_probe_paths(&mixed),
            vec![
                "libs/oya-iam-domain".to_owned(),
                "cloud/cloud-iam/slos/iam.openslo.yaml".to_owned(),
            ]
        );
    }

    /// THE WEDGE, end-to-end through the selector. Two committed artifact-only plans, one already
    /// landed: the landed one must be excluded so the pending one is selected. With a moves-only
    /// probe BOTH are permanently active and this raises `MultipleMovePlans` — an error step 1 of
    /// the UNIVERSAL materializer raises fail-closed on every CI leg and every local gate lane,
    /// wedging every subsequent PR in the repo until a human deletes a plan file.
    #[test]
    fn two_artifact_only_plans_one_landed_do_not_wedge_the_repo() {
        let landed_plan = PathBuf::from("specs/reorg/iam-backfill-move-plan.json");
        let pending_plan = PathBuf::from("specs/reorg/ci-backfill-move-plan.json");
        let load = |p: &Path| -> Result<Vec<String>, CodemodError> {
            let plan = if p.ends_with("iam-backfill-move-plan.json") {
                artifact_only("cloud/cloud-iam/slos/iam.openslo.yaml")
            } else {
                artifact_only("cloud/cloud-ci/slos/ci.openslo.yaml")
            };
            Ok(plan_probe_paths(&plan))
        };
        let old_dir_absent = |p: &str| Ok(p == "cloud/cloud-iam/slos/iam.openslo.yaml");
        let selected = select_active_move_plan(
            &[landed_plan, pending_plan.clone()],
            load,
            old_dir_absent,
        )
        .expect("a landed artifact-only plan must not wedge the single-plan guard")
        .expect("the pending artifact-only plan is selected");
        assert_eq!(selected, pending_plan);
    }

    /// MUST-PASS #5: a stale LANDED plan is excluded BEFORE the single-plan guard, so it can no
    /// longer hard-error the materialization of the one genuinely-pending move (the straddle DoS).
    #[test]
    fn landed_plan_excluded_pending_plan_selected() {
        let root = tmp_root("active-vs-landed");
        write_plan(&root, "a-landed-move-plan.json");
        write_plan(&root, "z-pending-move-plan.json");
        // The alphabetically-first plan is landed; the later plan is pending. A first-match caller
        // would select the wrong plan, while the authoritative active selector must choose z.
        let load = |p: &Path| -> Result<Vec<String>, CodemodError> {
            if p.ends_with("z-pending-move-plan.json") {
                Ok(vec!["cloud/cloud-ci/gates/oya-cloud-ci-firewall-app".to_owned()])
            } else {
                Ok(vec!["libs/oya-shared-pdp-adapter-cedar".to_owned()])
            }
        };
        let old_dir_absent = |d: &str| Ok(d == "libs/oya-shared-pdp-adapter-cedar");
        let selected = resolve_effective_active_move_plan(None, &root, load, old_dir_absent)
            .unwrap()
            .unwrap();
        assert_eq!(
            selected.file_name().unwrap(),
            "z-pending-move-plan.json",
            "the stale landed plan must not block the pending one"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Two genuinely-PENDING plans still FAIL-CLOSED (the guard's intent is preserved).
    #[test]
    fn two_pending_plans_still_hard_error() {
        let a = PathBuf::from("specs/reorg/a-move-plan.json");
        let b = PathBuf::from("specs/reorg/b-move-plan.json");
        let load = |_p: &Path| Ok(vec!["some/pending/dir".to_owned()]);
        let old_dir_absent = |_d: &str| Ok(false); // both pending
        assert!(matches!(
            select_active_move_plan(&[a, b], load, old_dir_absent),
            Err(CodemodError::MultipleMovePlans { count: 2, .. })
        ));
    }

    /// All plans landed => no active plan => canonical empty manifest (None), never an error.
    #[test]
    fn all_landed_is_none() {
        let a = PathBuf::from("specs/reorg/a-move-plan.json");
        let b = PathBuf::from("specs/reorg/b-move-plan.json");
        let load = |_p: &Path| Ok(vec!["landed/dir".to_owned()]);
        let old_dir_absent = |_d: &str| Ok(true); // all landed
        assert!(select_active_move_plan(&[a, b], load, old_dir_absent).unwrap().is_none());
    }

    #[test]
    fn discover_is_sorted_and_filters_by_suffix() {
        let root = tmp_root("sort");
        write_plan(&root, "zeta-move-plan.json");
        write_plan(&root, "alpha-move-plan.json");
        // The bare suffix file (empty capability stem) must NOT count.
        let dir = root.join(REORG_PLAN_DIR);
        std::fs::write(dir.join("-move-plan.json"), "{}\n").unwrap();
        let plans = discover_committed_move_plans(&root).unwrap();
        let names: Vec<String> = plans
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert_eq!(
            names,
            vec!["alpha-move-plan.json", "zeta-move-plan.json"],
            "sorted, suffix-filtered, empty-stem excluded"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
