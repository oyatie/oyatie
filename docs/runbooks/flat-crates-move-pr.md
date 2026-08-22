---
purpose: Oyatie Runbook — Flat Crates Move PR
doc_status: published
---

# Oyatie Runbook — Flat Crates Move PR

> **Status:** Active
> **Owner:** `council-architecture`
> **Severity supported:** Sev 3
> **Last verified:** 2026-05-11 by Codex in local drill
> **Related:** [ADR-0015](../decisions/ADR-0015-architectural-flattening-target.md), [PRD.md](../PRD.md), [DESIGN.md](../DESIGN.md), [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md)

---

## Trigger

Open this runbook when a PR moves, renames, creates, or splits a workspace crate as part of the ADR-0015 flat-crates migration.

Use it for changes that touch any of:
- root `Cargo.toml [workspace.members]`
- `crates/oya-<context>-<role>[-<capability>]/`
- `registry/catalog/<crate>.yaml`
- imports or dependency edges for the moved crate

Do not use this runbook for ordinary in-place code edits that do not change crate topology.

---

## Pre-checks (5 minutes max)

- [ ] Confirm the target crate name starts with `oya-` and matches `crates/<package-name>` — verify by reading the crate `Cargo.toml` and root workspace member entry.
- [ ] Confirm no top-level `modules/`, `services/`, or `platform/` tree is introduced — verify with `find . -maxdepth 1 -type d`.
- [ ] Confirm the move PR is the only active PR changing root `Cargo.toml [workspace.members]` — verify from the merge queue or PR list.
- [ ] Confirm the crate has or will get a `registry/catalog/<package-name>.yaml` row with a valid role.

If any pre-check fails, stop the move and route to [workspace-members-merge-queue.md](workspace-members-merge-queue.md) or [per-context-flatten-phase.md](per-context-flatten-phase.md).

---

## Steps

1. ☐ Move the crate directory to `crates/<package-name>` without changing behavior.
   Expected: the old path no longer contains source files; the new path has the same crate identity and tests.
   If differs: revert the move before changing imports.

2. ☐ Update root `Cargo.toml [workspace.members]` in the same edit.
   Expected: exactly one workspace member points to the moved package.
   If differs: stop and resolve duplicate or missing workspace entries.

3. ☐ Update package-local paths and imports only as needed for the new location.
   Expected: no broad rename, no blanket search-and-replace, no public API drift.
   If differs: split behavior changes into a separate PR.

4. ☐ Add or update `registry/catalog/<package-name>.yaml`.
   Expected: `role` is one of `kernel`, `domain`, `app`, `api`, `worker`, `adapter`, or `runtime`; `context`, `plane`, `slo`, and `data_classes_owned` are present.
   If differs: stop and fix the catalog record before running gates.

5. ☐ Run the flat-crates guard.
   Command: `oya gate validate architecture-boundaries --self-test && oya gate validate architecture-boundaries`
   Expected: self-test passes and the architecture boundary check reports the workspace crate count.
   If differs: fix path, catalog, role, or dependency-direction errors.

6. ☐ Run targeted Rust verification for the moved crate.
   Command: `cargo test -p <package-name> --all-features`
   Expected: tests pass without adding fallback behavior or suppressing failures.
   If differs: fix the move rather than bypassing tests.

---

## Rollback

1. Restore the previous directory path.
2. Restore the previous root `Cargo.toml [workspace.members]` entry.
3. Restore or remove the corresponding `registry/catalog/<package-name>.yaml` change.
4. Re-run `oya gate validate architecture-boundaries` to prove the workspace is back to a valid state.

---

## Verification

- [ ] `cargo fmt --all -- --check` passes.
- [ ] `cargo check --workspace --all-targets --all-features` passes.
- [ ] `oya gate validate architecture-boundaries --self-test` passes.
- [ ] `oya gate validate architecture-boundaries` passes.
- [ ] `cargo run -p tooling-cli-dev-runtime -- catalog validate` passes.
- [ ] `cargo run -p tooling-cli-dev-runtime -- gate validate cargo-prefix` passes.
- [ ] `cargo run -p tooling-cli-dev-runtime --bin repoctl -- pre-push` passes or the PR records a local-resource blocker plus the targeted substitutes above.

---

## Post-incident updates

If the move exposed a missing guard or caused a failed merge:
- [ ] Add a row to [MISTAKES-LEDGER.md](../MISTAKES-LEDGER.md).
- [ ] Update this runbook with the new prevention.
- [ ] Add or update a CI lane in [standards/ci-lanes.md](../standards/ci-lanes.md) and [registry/quality/lanes.yaml](../../registry/quality/lanes.yaml).

---

## Audit-chain emission

Each invocation records an engineering evidence event with:
- runbook id: `flat-crates-move-pr`
- crate package name
- old path and new path
- workspace member diff summary
- verification commands and outcomes
- reviewer verdict

---

## Sources scanned

- [ADR-0015](../decisions/ADR-0015-architectural-flattening-target.md)
- [PRD.md §6](../PRD.md#6-constraints-hard)
- [DESIGN.md §8](../DESIGN.md#8-architectural-flattening-per-adr-0015)
- [templates/runbook-template.md](../templates/runbook-template.md)
