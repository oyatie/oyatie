//! Deterministic, reversible capability-move CODEMOD + pre-move green-snapshot oracle
//! (ADR-0562 Phase-0 reorg machinery).
//!
//! # What this is (and what it is NOT)
//!
//! This crate BUILDS the machinery the later strangler moves will use; it does **not** move
//! any real capability. A capability move (ADR-0562 §3 placement rule) re-homes a set of
//! crates from their current `{cloud,oya}/...` paths to `<capability>/{core,ports,adapters,
//! facade}/...`. The move is mechanical + manifest-level (paths + package names), never a
//! code-behavior change (ADR-0562 Consequences §Neutral).
//!
//! The unit of operation is a [`MovePlan`]: an explicit, total list of crate-move tuples
//! `(old_path, new_path, old_cargo_name, new_cargo_name)`. The strangler derives one plan
//! per capability from `governance/capability-registry.json`; this tool APPLIES a plan
//! deterministically. Keeping the plan explicit means the per-crate FACE classification
//! (which sub-fold a crate lands in) stays the strangler's decision and out of this engine,
//! so the engine is a pure, testable transform.
//!
//! # The transforms (forward)
//!
//! For a given [`MovePlan`] the engine performs, atomically:
//!
//! 1. **directory move** (`git mv`, history-preserving) — `old_path` -> `new_path`;
//! 2. **`[package].name` rewrite** in each moved crate's `Cargo.toml`
//!    (`old_cargo_name` -> `new_cargo_name`) plus the `[lib].name`/`[[bin]].name` snake
//!    mirror;
//! 3. **`[dependencies]` key + `path=` rewrite** across EVERY workspace `Cargo.toml`: a
//!    dependency on a moved crate is renamed to its new cargo name, and EVERY relative
//!    `path = "../.."` dep is recomputed against the post-move layout — the ~200 move-fatal
//!    files (ADR-0562 Context: deep `../../../` path-deps break the instant a crate moves);
//! 4. **root workspace `members`/`exclude`** rewrite via the
//!    [`workspace_members_kernel`] resolver — only when a moved path falls outside the
//!    existing globs (the globbed membership absorbs most moves with zero edit, FRIC-1781069288);
//! 5. **Rust `use`/`extern crate`/path import** rewrite (kebab cargo name -> snake crate
//!    name) across `.rs` sources;
//! 6. **BUCK `name`/`deps`/`visibility`/`//path:target` label** rewrite + the `crate=`/
//!    `crate_root=` snake mirror.
//!
//! # Reversibility-by-construction
//!
//! Every transform is driven by the [`MovePlan`], and a plan inverts by swapping each
//! tuple's old/new sides ([`MovePlan::inverse`]). `--revert` applies the inverse, restoring
//! the tree byte-identically (proven by the fixture round-trip test).
//!
//! # The pre-move green-snapshot oracle
//!
//! Before any move the [`oracle`] captures `cargo metadata` + `buck2 targets //...` as the
//! committed rollback oracle. A **dry-run / shadow-apply** mode applies the plan into a
//! throwaway copy of the tree and PROVES resolution (`cargo metadata` resolves + `buck2
//! targets //...` resolves) WITHOUT landing it. Post-move green is necessary but not
//! sufficient; the dry-run gate is the safety. The engine is **fail-closed**: it refuses
//! (non-zero) if the move would leave the workspace non-resolving, if a path-dep recompute
//! is ambiguous, or if a target name collides.
//!
//! # Determinism
//!
//! All file walks are sorted; all TOML edits go through `toml_edit` (format-preserving);
//! all path recomputes are pure functions of the plan + the relative geometry. The engine
//! never reads ambient git state into a transform — `git mv` is invoked only to preserve
//! history, and its output never feeds a decision.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod buck;
pub mod cargo;
pub mod manifest;
pub mod model;
pub mod oracle;
pub mod plan;
pub mod rust_src;

pub use manifest::{
    discover_committed_move_plans, plan_is_landed, plan_probe_paths, resolve_committed_move_plan,
    resolve_effective_active_move_plan, resolve_effective_move_plan, select_active_move_plan,
    select_move_plan,
};
pub use model::{
    CodemodError, CrateMove, Mapping, MappingRow, MovePlan, REORG_MOVE_MANIFEST_SCHEMA,
    move_manifest_value,
};
pub use plan::{ApplyOptions, ApplyOutcome, apply_plan};
