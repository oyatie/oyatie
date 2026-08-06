---
id: ADR-0538
title: "Globbed root workspace membership and coverage gate"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-10
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-0700]
amended_by: [ADR-0637]
depends_on: [ADR-0017, ADR-0132, ADR-0515, ADR-0525, ADR-0527]
amends: []
related: [ADR-0017, ADR-0083, ADR-0131, ADR-0132, ADR-0363, ADR-0515, ADR-0525, ADR-0526, ADR-0527]
related_specs:
  - /specs/masterplan.json
  - /specs/root-hub-pointers.json
milestone: W0
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0538: Globbed root workspace membership and coverage gate

## Status

**Proposed - 2026-06-10 (authored for founder sign-off; door: one-way).**

## Context

Concurrent crate lanes were all editing the root `Cargo.toml` `[workspace].members` array. That made
the array a shared merge-conflict point and produced one class of lockfile conflicts even when the
actual crates were independent. The founder manually resolved the latest lock conflict
(FRIC-1781069288) and ordered structural elimination rather than repeated manual merge repair.

The root workspace also feeds several repo-local tools and gates. A textual parser that previously
read concrete member strings would treat glob entries like `libs/oya-*` as literal paths, producing
phantom missing-member findings and hiding real member coverage. The change therefore needs both a
canonical expansion kernel and a regression gate that proves the root workspace stays globbed and
covers every first-party crate manifest that should be part of the root workspace.

## Decision

The root workspace uses globbed membership:

```toml
members = [
  "libs/oya-*",
  "cloud/*/crates/oya-*",
  "cloud/cloud-ci/gates/*",
  "oya/*/crates/oya-*",
  "oya/office/oya-*",
  "tools/oya-*",
]
exclude = [
  "cloud/cloud-kernel",
  "ci/facade/automation-language-policy",
]
```

Consumers requiring a Cargo-valid concrete member set MUST call
`libs/oya-workspace-members-kernel::resolve_member_dirs(repo_root)`. Diagnostic gate producers that
must preserve every invalid glob match MUST call
`libs/oya-workspace-members-kernel::scan_member_dirs(repo_root)`. Neither consumer category may
reimplement `[workspace].members` expansion. The kernel expands single-component `*` and
partial-component patterns, follows matched directory symlinks like Cargo, skips Cargo-unmatched
dangling and cyclic symlinks, fails closed on other glob-expansion I/O errors, fails when an
unexcluded matched directory lacks `Cargo.toml`, and applies each reviewed explicit `exclude` to
the whole subtree before manifest validation.

The cloud-ci floor adds one single-concern gate,
`ci/facade/workspace-member-coverage`, with stable violation codes:

- `workspace_member_explicit_path`: a root workspace member entry is not a glob.
- `workspace_member_missing_manifest`: an unexcluded concrete member-glob match does not contain
  `Cargo.toml` and would make Cargo reject the workspace.
- `crate_dir_not_covered`: a tracked first-party package manifest directory is neither covered by
  the resolved member set nor explicitly excluded.

The accounting-registry producer emits the gate face rows:
`{member_entry,is_glob}` for every raw member entry, `{member_match,has_manifest}` for every invalid
unexcluded concrete glob match, and `{crate_dir,covered,excluded}` for every eligible crate manifest
directory. The evaluator itself is pure policy over booleans: no filesystem access, no Cargo
invocation, and no duplicated glob expansion.

## Consequences

### Concrete file and crate changes

| Path / Crate | Change type | BNF v4.1 name | Layer |
|---|---|---|---|
| `Cargo.toml` | update root workspace members/exclude | - | - |
| `libs/oya-workspace-members-kernel/` | create canonical resolver kernel | `oya-workspace-members-kernel` | kernel |
| `libs/oya-workspace-members-kernel/tests/cargo_differential.rs` | add hermetic Cargo-boundary differential fixtures, including the Windows `ERROR_CANT_RESOLVE_FILENAME` case | `oya-workspace-members-kernel` | kernel test |
| `libs/oya-workspace-members-kernel/OWNERS` | register narrow `cloud-ci-platform` ownership for the resolver and its differential fixtures | - | ownership data |
| `marketplace/facade/dev-cli/src/workspace_manifest.rs` | migrate parser to kernel | `marketplace-dev-cli` | cli |
| `marketplace/facade/dev-cli/src/workspace_topology_gate.rs` | consume expanded members and skip nested workspaces | `marketplace-dev-cli` | cli |
| `tools/oya-xtask-metadata-augment-app/src/metadata.rs` | migrate metadata member enumeration to kernel | `oya-xtask-metadata-augment-app` | app |
| `ci/facade/workspace-member-coverage/` | create gate crate | `oya-cloud-ci-workspace-glob-coverage-app` | app |
| `ci/facade/artifact-inventory-registry/` | emit producer face and baseline keys | `oya-cloud-ci-accounting-registry-app` | app |
| `libs/oya-ci-config/` and `oya-ci.toml` | register gate face and disposition data | `oya-ci-config-kernel` | kernel |
| `.github/workflows/oya-ci-required.yml` | bind the Cargo differential in the self-hosted Buck2 lane | - | - |

### Integration via Workflow + Ontology

Not applicable. This ADR changes repository membership and CI gate input contracts only; it does
not introduce new Workflow events or Ontology writes.

### Positive

- Independent crate lanes no longer edit the same root `members` array.
- Workspace membership expansion has one implementation surface.
- Existing topology and metadata tools see concrete members, not glob literals.
- The new gate turns the intended glob discipline into a baseline-backed cloud-ci invariant.

### Negative

- Tools that need raw member entries must distinguish raw manifest entries from expanded member
  dirs.
- The root workspace now depends on resolver parity with Cargo's single-component glob behavior.
- Existing uncovered crate dirs, if introduced before their owning glob, become explicit
  `crate_dir_not_covered` debt instead of being silently ignored.

### Operational

- Buck2 remains the primary local verification surface for the changed crates and gate targets.
- `cargo metadata --format-version 1 --no-deps` is permitted only for the required member-set
  equivalence proof and lock refresh.
- The Buck-owned Cargo differential runs as a binding target in the self-hosted Linux/ARM64 Buck2
  lane. Its portable Unix fixtures prove Cargo parity today; the cfg(windows)
  `ERROR_CANT_RESOLVE_FILENAME` (`Win32 1921`) fixture remains dormant until real Windows capacity
  exists. Talos is Linux, so this invariant does not justify a hosted runner or emulated Windows VM.
- The gate is born-blocking through `gate-baseline.generated.json`; the current clean root glob
  corpus freezes empty for `workspace_member_explicit_path` and reports any future explicit member
  entry as new debt.

## Clean Architecture Impact

| Lane | Impact | Action required |
|---|---|---|
| `dependency-direction` | Affected | New kernel is dependency-free except `toml`; callers depend inward on it. |
| `cross-product-refusal` | Not affected | No product boundary is introduced. |
| `port-location` | Not affected | No port trait moves. |
| `layer-correctness` | Affected | New kernel and app crate names carry explicit BNF layers. |
| `composition-root-only` | Not affected | No runtime composition root changes. |
| `sdk-kernel-only` | Affected | Developer-sdk tooling consumes the kernel instead of parsing members locally. |

## Verification

- `buck2 test //libs/oya-workspace-members-kernel:oya-workspace-members-kernel-unittest`
- `buck2 test //libs/oya-workspace-members-kernel:oya-workspace-members-kernel-cargo-differential`
- `buck2 test //ci/facade/workspace-member-coverage:oya-cloud-ci-workspace-glob-coverage-app-unittest`
- `buck2 test //ci/facade/workspace-member-coverage:oya-cloud-ci-workspace-glob-coverage-app-gate`
- `buck2 test //ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin-unittest`
- `cargo metadata --format-version 1 --no-deps` before/after member-set equivalence proof recorded
  in the PR body.
- Follow-up fail-closed producer-evidence guard:
  `evidence/multispectrum/cloud-ci-workspace-glob-no-rows-20260625-1782426431.json`.
