---
status: Superseded
deciders: council-architecture, council-foundry-vcs
date: 2026-05-16
owner: council-architecture
supersedes: []
superseded_by: [ADR-0709]
related:
  - ADR-0054-grit-scaffold-claim-pattern.md
  - ADR-0069-active-machine-readable-artifact-contract.md
  - ADR-0097-intelligence-account-adapter-rename-target-slot-last.md
purpose: Promote `registry/` (flat, singular) as the canonical home for all machine-readable registry entries; retire `registries/cross-cutting/` (plural + nested).
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0115: Registry consolidation: flat singular `registry/`

## Context

Two parallel directory roots accumulated through prior migrations:

- `registry/` (flat singular) — the canonical home that grew organically
  for product-scoped registry entries (`registry/catalog/`,
  `registry/quality/`, `registry/glossary/`, `registry/vcs/`,
  `registry/adr/`, `registry/accounts/`, `registry/capabilities/`, etc.).
  ~497 files.
- `registries/cross-cutting/` (plural + nested) — a parallel root used
  for cross-cutting catalogs (knowledge graphs, fixup tasks, microservice
  registry, mistakes ledger, merge-queue logs, etc.). ~20 files.

The user directive (2026-05-16): a single flat root `registry/` (singular)
where every direct child is a semantic class. No `cross-cutting/` subdir.
No plural form.

This consolidation also aligns the on-disk layout with the BNF in
`specs/oyatie-doctrine.json` P14, which already declares
`/registry/` as the first-class durable home.

## Decision

`registry/` (flat, singular) is the canonical machine-readable registry
root. Every direct child of `registry/` is a semantic class
(`catalog/`, `quality/`, `glossary/`, `vcs/`, `adr/`, `accounts/`,
`capabilities/`, `cells/`, `audit-chain/`, `placeholder-debt/`,
`graph/`, `claim-matrix/`, plus the flat-file cross-cutting registries
that landed at the root).

`registries/cross-cutting/` is retired. All 20 files moved to
`registry/<same-relpath>` via `git mv` (history preserved).

## Naming justification (BNF + 12-layer-enum conformance)

Per `feedback_naming_justification`: `registry/` is the **noun-singular**
form matching the v4 BNF `kind` token already declared by
`oyatie-doctrine.json` P14 (`kind = specs | registry | evidence |
templates | adrs | standards | plans`). It is **layer-neutral** in the
12-layer-enum sense — registry entries are passive data artifacts
(domain-data tier), not crates with layer membership; the layer enum
applies to Rust crates, not to data root names. The flat-singular form
maximizes BNF cleanliness and removes the redundant `cross-cutting/`
scope token, which the BNF already permits as the omitted-scope default
for cross-cutting artifacts.

## Rejected alternatives

| Alternative | Why rejected |
|-------------|--------------|
| `registries/` (plural, flat) | Diverges from BNF singular `kind` token; English plural noise without semantic value at the root level. |
| `registries/cross-cutting/` (plural + nested, status quo) | Two redundant tokens at the root; `cross-cutting/` is the implicit default scope per BNF P14, so the directory adds zero information. |
| `registries/shared/` | Same plural problem; `shared/` is also implicit (everything in oyatie is shared per `feedback_glossary_shared_not_platform`). |
| `registries/canonical/` | Triple redundancy: every registry entry under any canonical-root path is by definition canonical. |
| Keep both `registry/` and `registries/cross-cutting/` | Two parallel roots, two grep targets, two doc-update surfaces. The user explicitly approved consolidation. |

## Migration

### File moves (20 files)

Every file at `registries/cross-cutting/<RELPATH>` moved to
`registry/<RELPATH>` via `git mv` (preserving the subdir hierarchy,
including `claim-matrix/` and `graph/`).

Examples:

```
registries/cross-cutting/knowledge-graph-semantic.json   → registry/knowledge-graph-semantic.json
registries/cross-cutting/microservices.json              → registry/microservices.json
registries/cross-cutting/mistakes-ledger.json            → registry/mistakes-ledger.json
registries/cross-cutting/fixuptasks.jsonl                → registry/fixuptasks.jsonl
registries/cross-cutting/claim-matrix/ops-portal.json    → registry/claim-matrix/ops-portal.json
registries/cross-cutting/graph/architecture-map.json     → registry/graph/architecture-map.json
```

Empty `registries/cross-cutting/` and `registries/` directories removed.

### Reference rewrites (~131 files touched)

All source code, configs, workflows, docs, specs, registries, and plans
that referenced `registries/cross-cutting/` or the bare `registries/`
directory token were rewritten to `registry/`. Live-code occurrences
(e.g., `crates/oya-governance-architecture-map-freshness-kernel/src/lib.rs`
roots vector) updated. `specs/root-hub-pointers.json`
entry-point paths updated.

The doctrine BNF kind enum in `specs/oyatie-doctrine.json`
P14 updated from `specs | registries | evidence | templates | adrs |
standards | plans` to `specs | registry | evidence | templates | adrs |
standards | plans` to lock the singular form.

### Historical evidence preserved

Files under `evidence/debate/` and `evidence/per-change/` that document
prior migration sed strategies were intentionally **not** rewritten —
they are immutable historical records. They contain the literal
strings `registries/cross-cutting` as evidence of the prior layout, not
as live path references.

### Common-noun usage preserved

English plural "registries" as a common noun (e.g., crate doc comments
"crates, contracts, registries, and ownership edges") was preserved.
Only path tokens were rewritten.

## Consequences

**For agents.** One canonical registry root. `grep registries/` returns
zero hits in live code, configs, specs, and docs (only historical
evidence + common-noun prose remain). Path resolution is unambiguous.

**For CI.** Existing lanes that walk `registries/cross-cutting/` (e.g.,
the architecture-map freshness kernel) now walk `registry/`. No new
lane required; the move surface is mechanical.

**For contributors.** Add new cross-cutting registry entries at
`registry/<concept>.json`. Add new product-scoped or class-scoped
entries at `registry/<class>/<concept>.json`. The BNF kind token is now
`registry` (singular) — match the BNF in PR descriptions and ADRs.

**Sanctioned-primitives note.** Per user directive 2026-05-16:
grit/rtk/icm/vox external agent-coordination tooling is deprecated and
no longer required for code mutations on this PR or going forward;
subsequent ADR will formally retire the sanctioned-primitives section
of CLAUDE.md.

## Verification

| Lane | Result |
|------|--------|
| `cargo build --workspace` | (recorded in PR body) |
| `cargo test --workspace --no-run` | (recorded in PR body) |
| `cargo fmt --all -- --check` | (recorded in PR body) |
| `cargo clippy --workspace --all-targets --keep-going -- -D warnings` | (recorded in PR body) |
| `grep -rn 'registries/cross-cutting' .` (live tree, excl. evidence) | 0 hits |
| `grep -rn '/registries/' .` (live tree, excl. evidence) | 0 hits |

## Follow-up

- **`specs/` flattening** is a deliberate successor-IP
  (separate ADR + separate PR). The same flat-singular logic
  (`specs/`) will likely apply, but the surface is larger and is not
  included in this PR.
- **Bominal-inheritance ledger update.** Per
  `feedback_bominal_inheritance_precedence`, this Oyatie-side decision
  overrides any Bominal directory layout that retained
  `registries/cross-cutting/`. Bominal inheritance overrides registry
  (`registry/bominal-inheritance-overrides.json`, post-move) records
  this divergence on next sweep.
