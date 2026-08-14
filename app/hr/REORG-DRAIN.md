# app/hr reorg drain notes (`integ/hr`)

## Ownership (rule 3d / 3e)

- **Forever home:** `app/hr/**` (this rail).
- **Source (read-only):** `oya/hr/**` on `origin/dev` until shrink-only delete on `integ/oya`.
- **OVERRULE 3d:** migrated off shared `integ/app`.

## Completed

- Wave-1 absorb replayed from `integ/app@84ea18fc1` (40 files).

## Elevate (out of envelope, gate-required)

- The absorb's gate-required admission edits cross the `app/hr/**` envelope, and the envelope authorization must be routed through `integ/specs` (the envelopes file + waiver directory are integ/specs-owned per deliver.js ENVELOPE VERIFY — the HR lane must not self-edit them):
  1. **`governance/capability-registry.json`** membership mapping (`app/hr` in `membership_lint_coverage.app_products.current_dirs`) — needs an `integ/hr` hub waiver row + waiver file under `governance/check/integ-envelope/waivers/` recorded by integ/specs; gate-required for `ci-module-membership` (MEM-NEW-UNMAPPED-CRATE).
  2. **Root `Cargo.toml`** ADR-0538 exclude for `app/hr/crates` — needs an `integ/hr` adjunct claim in `specs/integ-branch-envelopes.json` recorded by integ/specs; gate-required for `cloud-ci-workspace-glob-coverage` (crate_dir_not_covered).
  Both were previously self-claimed on this tip and reverted here per the review thread; they expire at the integ/hr drain.

## Drain prerequisites (`integ/oya` shrink, after verify)

- **Retarget downstream IAM consumers** off `oya/hr` before deleting the source: `iam/facade/tenant-rbac-local-runtime-composition` and `iam/facade/tenant-rbac-local-inmemory-harness` reference `//oya/hr/crates/...` in their `BUCK` and `Cargo.toml` files; retarget to `app/hr` equivalents as part of the drain.
- **Relocate `HrEmploymentStoragePort`** out of `crates/oya-hr-employment-storage-adapter-inmemory` into a core/ports face when workspace membership flips to `app/*/crates/*` (port-placement gate keys its frozen exception to the old `oya/hr` path; shrink-only means the forever-home path cannot reuse it).
- **Extend tier-dependency coverage** to the app-product shape: add `app/*/crates/oya-*` to `crate_root_globs` and classify the root when the manifests are admitted (identical deps are still inspected through the live legacy copy until then).

## Out of envelope

- `oya/hr/**` deletes — `integ/oya` only.
