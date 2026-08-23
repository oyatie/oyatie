# Task #20 Import Manifest Freeze — 2026-06-09

## Outcome

This preflight freezes the safe import manifest for the 5 sibling snapshots plus the kernel snapshot before any raw consolidation import. It also preserves the founder-confirmed intentional agent-tooling deletions as part of the same branch.

**Change ID:** `task20-import-manifest-freeze-20260609`
**Worktree:** `/Users/jasonlee/Developer/oyatie-task20-import-manifest-freeze`
**Branch:** `agent/task20-import-manifest-freeze-20260609`
**Base HEAD:** `8a2f6ca251a6a90ebb92f7cfa8e554b8783598ba`

## Hyperscaler lens applied

- One owned destination per product boundary; no duplicate `cloud-*`, `oya/*`, or adapter roots.
- Productized domains win over snapshot-local scaffolds.
- Root workspace, lockfile, and generated accounting faces are serialized because they are global control-plane surfaces.
- Live/provider/CLI tests are env-gated unless CI provisions the dependency; no false-green claims.
- Kernel/OS/no_std/toolchain work is isolated until lower-risk lanes prove the import pattern.
- Evidence distinguishes design-target docs from implemented runtime code.

## Snapshot freeze

| Component | Snapshot SHA | Destination | Freeze disposition |
|---|---:|---|---|
| `office` | `9920b3a327b3` | `oya/office/` | `MERGE_RENAME_CONFORM` |
| `codex` | `0cf13018b26c` | `oya/intelligence/ provider/account adapter family by default` | `MERGE_BY_PROVIDER_FAMILY; keep cloud/cloud-intelligence only for runtime/gateway-boundary code` |
| `claude` | `483649d6a108` | `oya/intelligence/crates/intelligence-claude-agent-sdk` | `CREATE_CRATE_UNDER_EXISTING_INTELLIGENCE_HOME` |
| `oyago` | `8eeb4139feec` | `transpiler tooling area TBD` | `BLOCK_RAW_IMPORT_UNTIL_SINGLE_HOME_DECISION` |
| `oyapy` | `e7d1b80aac6c` | `transpiler tooling area TBD` | `BLOCK_RAW_IMPORT_UNTIL_SINGLE_HOME_DECISION` |
| `kubernetes` | `26173992778a` | `cloud/cloud-k8s/ plus managed-k8s service homes by boundary` | `MERGE_BY_BOUNDARY; do not dump runtime code into retired/read-only platform docs if managed-k8s owns service logic` |
| `db-data-docs` | `26173992778a` | `specs/ + docs/decisions/ renumbered; cloud/cloud-data as product context` | `MIGRATE_SPEC_AND_ADRS_ONLY_RENUMBERED; DROP pilot scaffold docs` |
| `kernel` | `26173992778a` | `cloud/cloud-kernel/` | `CREATE_LAST_ISOLATED` |
| `os` | `26173992778a` | `cloud/cloud-os/` | `CREATE_AFTER_KERNEL_OR_WITH_EXPLICIT_OS_PLAN` |


Full machine-readable manifest: [`../../evidence/consolidation/task20-import-manifest-freeze-20260609.json`](../../evidence/consolidation/task20-import-manifest-freeze-20260609.json).

## Destination decisions

- `office` lands in existing `oya/office/`; rename `oyaoffice-*` to `office-*` and reconcile deltas.
- `claude` and `codex` land in existing `oya/intelligence/` by provider/account/process-protocol family. `cloud/cloud-intelligence` remains a runtime/gateway boundary, not a duplicate SDK root.
- `kubernetes` uses existing `cloud/cloud-k8s/` for platform-level material and existing `cloud/managed-k8s-*` surfaces for service runtime material.
- `db-data` uses existing `cloud/cloud-data/` as product context, but imports only specs/renumbered ADRs from the snapshot; no runtime DB code is claimed.
- `kernel` creates `cloud/cloud-kernel/` later and last/isolated.
- `OS` creates `cloud/cloud-os/` later. `cloud/cloud-node-os/` was investigated as requested; no existing exact node-os home is present, so it is recorded only as an absent older alias.
- `oyago`/`oyapy` are blocked from raw import until one transpiler tooling home and registry model is selected.

## Intentional deletion preservation

The following 19 tracked deletions are preserved exactly because the founder stated the deletions are intentional:

- `.claude/commands/build.md`
- `.claude/commands/code-simplify.md`
- `.claude/commands/plan.md`
- `.claude/commands/review.md`
- `.claude/commands/ship.md`
- `.claude/commands/spec.md`
- `.claude/commands/test.md`
- `.claude/settings.json`
- `.claude/skills`
- `.codex/skills`
- `.gemini/commands/build.toml`
- `.gemini/commands/code-simplify.toml`
- `.gemini/commands/planning.toml`
- `.gemini/commands/review.toml`
- `.gemini/commands/ship.toml`
- `.gemini/commands/spec.toml`
- `.gemini/commands/test.toml`
- `.gemini/settings.json`
- `.gemini/skills`


## Parallelization plan

- Native Codex child agents: cap remains 6 by AGENTS.md protocol, despite the user allowing 50.
- OMX/team/runtime lanes: up to 50 are allowed when write sets are disjoint or read-only.
- Parallel now: read-only inventories, snapshot diffs, docs/spec renumber planning, verification planning.
- Serialized: root `Cargo.toml`, root `Cargo.lock`, generated accounting registry faces, shared `oya/intelligence` writes, shared `oya/office` writes, high-blast-radius kernel/OS workspace registration, push/PR operations.

## Backlog execution order unlocked by this freeze

1. Office merge/rename/conform lane (lowest-risk code import).
2. DB spec/ADR renumber lane (docs/spec-only; independent of code imports).
3. Claude SDK crate under `oya/intelligence` with fake CLI tests kept and live CLI tests env-gated/manual.
4. Codex provider/account diff and merge into existing intelligence/provider family.
5. Transpiler home decision, then oyago/oyapy fixture-preserving imports.
6. K8s boundary merge plan across `cloud/cloud-k8s` and managed-k8s services.
7. Kernel and OS imports last with toolchain isolation.

## Verification contract

Required for this preflight PR:

- Exact 19-path deletion invariant.
- This evidence JSON parses.
- Audit-chain entry parses and references this change ID.
- Secret-pattern scan over new docs/evidence artifacts.
- Accounting-registry producer + `registry-drift`, or any generated face delta is committed and explained.
- Targeted `manifest-hygiene` and `total-accounting` gates where feasible.

Full workspace/buck2/provider/deploy gates are intentionally not local prerequisites for this manifest-only preflight; the PR still requires the canonical remote `presubmit` gate.
