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

use crate::model::CodemodError;

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
/// * >1 plan -> `Err(CodemodError::MultipleMovePlans)` — fail-closed (#65). More than one committed
///   plan in a single PR is a contributor error the materialization must NOT silently first-win.
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
/// * the >1-committed-plan guard ([`resolve_committed_move_plan`]) runs FIRST and REGARDLESS of
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
