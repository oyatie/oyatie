---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M01-foundation
phase: P02-shard-1-atomic-rename
status: Complete
acceptance_lanes: []
entry_gate: 'P01-bnf-v4-adrs-finalized complete; ADR-0056 v4.1 accepted; rename plan
  v4.1

  user-approved; /tmp/rename-map-v4.1.tsv contains exactly 114 rows; grit

  branch grit/shard-1-atomic-rename-2026-05-13 checked out.

  '
exit_gate: 'All 114 v3 crate dirs renamed on disk; zero `oya-platform-*`, `oya-workspace-*`,

  `oya-foundation-*`, `oya-tooling-*` directories exist under crates/; all 4 LEAN

  check crates present at new names; cargo check --workspace --all-features exits
  0;

  cargo build exits 0; cargo clippy exits 0; cargo deny check exits 0; cargo doc

  exits 0; cargo nextest run exits 0; Cargo.lock rewritten; root Cargo.toml

  workspace members updated to v4.1 names; ICM context-oyatie row emitted;

  grit done OR fallback git merge to main with ICM rationale row.

  '
depends_on:
- milestone: M01
  phase: P01-bnf-v4-adrs-finalized
  reason: BNF v4.1 ADRs must be accepted before rename is executed to prevent naming
    drift
owner_team: council-architecture
purpose: Auto-backfilled purpose for phase-spec.md
---
# P02-shard-1-atomic-rename: Shard 1 atomic 114-row rename to BNF v4.1

## Purpose

Executes the atomic rename of all 114 Shard-1-scoped crates from v3 names to
BNF v4.1 names. The 26 PROTOCOL-UNKNOWN rows are explicitly excluded (deferred
to P03). After this phase, no `oya-platform-*`, `oya-workspace-*`,
`oya-foundation-*`, or `oya-tooling-*` directories remain on disk. The workspace
compiles clean under the new names. This is the critical-path gate for all
downstream M01 and M02 work.

Advances Master Plan principles: clean architecture self-enforces via Cargo
(BNF v4.1 names encode layer); flat catalog (workspace renamed to connect;
platform prefix retired); single lockfile event (Hybrid C topology per ADR-0057).

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| platform (23 Shard-1 rows) | tenancy, identity, audit-chain, eventing, ontology, observability, policy-cedar, data-boundary, residency, dsr, metering, cell, regional-pack, secrets | 23 crate dirs + Cargo.tomls | `oya-tenancy-domain`, `oya-identity-domain`, etc. |
| cloud (18 Shard-1 rows) | cell, resource, region, compute, billing, capacity, finops, marketplace, dcops, data, kms, storage, surface, network, observability | 18 crate dirs | `oya-cloud-*-domain/application` |
| foundry non-check (19 Shard-1 rows) | adapter, bypass, capability, catalog, cloud-mutation, evidence, eval, mcp-gateway, policy, run, step, api-semver, mdbook, openapi, cargo-prefix | 19 crate dirs | `oya-foundry-*-domain/application/adapter` |
| foundry check (29 rows) | all v3 fitness crates | 29 crate dirs | `oya-check-*` flat namespace |
| connect/workspace (22 Shard-1 rows) | address-book, calendar, messenger, collab-runtime, document-format, dlp, ediscovery, docs, drive, dsr, forms, mail, meet, notes, recordings, retention, sheets, sites, slides, tasks, translate, trust-portal | 22 crate dirs | `oya-connect-*-domain` |
| foundation + tooling (3 rows) | application-app, dev-cli, codeview-cli | 3 crate dirs | `oya-application-app`, `oya-dev-cli`, `oya-codeview-cli` |
| root Cargo.toml | workspace members + metadata | `Cargo.toml` | BNF v4.1 microservice registry |
| Cargo.lock | lockfile | `Cargo.lock` | lockfile-rename via xtask |

Naming justifications for all 114 renames per §3 audit body in
`docs/plans/rename-plan-v4-clean-arch-2026-05-13.md`.

### Out-of-scope

- 26 PROTOCOL-UNKNOWN rows (`*-api` crates requiring protocol classification) — deferred to P03-shard-1-5-protocol-unknown-deferred.
- STUB-pending-iter-4 layer_evidence cells — deferred to P04-iter-4-src-inspection.
- 4 LEAN check crate implementations (populated in Shard 0 as scaffolds) — remain scaffolds through M01; implementation in M02.
- New check crates (`oya-check-architecture`, `oya-check-bounded-contexts`, `oya-check-supply-chain`, `oya-check-semver`) — also Shard 0 scaffolds; renamed from `oya-shared-*-check-cli` in this phase.

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | Execute 114-row atomic rename; verify acceptance gates | merged | `council-architecture` |

---

## Acceptance Gates

### Cargo / CI gates (exit 0 required)

```bash
cargo check --workspace --all-features                        # exit 0
cargo build --workspace --all-features                        # exit 0
cargo clippy --workspace --all-targets -- -D warnings         # exit 0
cargo nextest run --workspace || cargo test --workspace       # exit 0
cargo deny check                                              # exit 0
cargo doc --workspace --no-deps                               # exit 0
```

### Reality verification gates

```bash
# Zero old-name dirs remain
ls crates/oya-platform-* 2>&1 | grep -c "No such"    # must be > 0
ls crates/oya-workspace-* 2>&1 | grep -c "No such"   # must be > 0
ls crates/oya-foundation-* 2>&1 | grep -c "No such"  # must be > 0
ls crates/oya-tooling-* 2>&1 | grep -c "No such"     # must be > 0

# Zero old names in cargo metadata
cargo metadata --format-version 1 | \
  jq '.packages[].name' | \
  grep -E '"oya-(platform|workspace|foundation|tooling)-' | wc -l   # must be 0

# Cargo.lock contains zero old names
grep -cF "oya-platform-\|oya-workspace-\|oya-foundation-\|oya-tooling-" Cargo.lock || true  # must be 0
```

---

## Clean Architecture Compliance

### Layer assignments for renamed crates (representative)

| Crate (BNF v4.1) | Layer | Justification |
|---|---|---|
| `oya-tenancy-domain` | `domain` | business logic on kernel types; STUB-pending-iter-4 confirms |
| `oya-data-boundary-kernel` | `kernel` | pure types + ports (row 1; named-by-identity; ~95 consumers) |
| `oya-application-app` | `app` | composition-root binary; wires all layers (row 138) |
| `oya-codeview-cli` | `cli` | sanctioned-primitive READ CLI; has `[[bin]]`; layer = cli |
| `oya-dev-cli` | `cli` | `oya` + `repoctl` bins; layer = cli (row 139) |
| `oya-check-*` | check-namespace-exempt | flat `oya-check-<rule-name>`; BNF second production |
| `oya-connect-*-domain` | `domain` | workspace → connect rename; same layer (STUB-pending-iter-4) |
| `oya-foundry-*-adapter` | `adapter` | trait impl crates; layer = adapter |

### CI lanes (--report-only during M01; flip to BLOCKER at M02 exit)

All 4 LEAN check crates run `--report-only` post-rename. LEAN-A3 (supply-chain)
and LEAN-A4 (semver) are BLOCKER day-1 per ADR-0056 §"CI enforcement matrix".

---

## Grit Claim Symbols

```
# Workspace-scope work; grit session start used
# Fallback to ICM scaffold-locks-oyatie per ADR-0054 if session fails
Cargo.toml::workspace.members
Cargo.lock::all
crates/*/Cargo.toml::package.name
```

TTL: 7200s (2 h) per ADR-0057 §"Hybrid C Topology" freeze window.

Fallback log: ICM topic `direct-tool-invocations` with rationale
`grit session start failed; direct git branch + ICM coordination ledger used`.

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "P02-shard-1-atomic-rename COMPLETE. 114 crate dirs renamed to BNF v4.1. oya-platform-*/oya-workspace-*/oya-foundation-*/oya-tooling-* GONE from disk. Cargo.lock rewritten. All 6 acceptance gates exit 0. 4 LEAN check crates at new names (report-only). 26 PROTOCOL-UNKNOWN rows deferred to P03. grit done fallback: direct git merge to main per ADR-0053 bootstrap window." \
  -i high \
  -k "M01,P02,shard-1,rename,BNF-v4.1,phase-complete"
```

---

## References

- Rename plan: `docs/plans/rename-plan-v4-clean-arch-2026-05-13.md` §3, §8
- ADR-0056: `docs/decisions/ADR-0056-rust-clean-architecture-bnf.md`
- ADR-0057: `docs/decisions/ADR-0057-cutover-mechanics-rename-plan-v4.md`
- TSV: `/tmp/rename-map-v4.1.tsv` (114 rows)
- Memory: `feedback_grit_claim_work_done.md`, `feedback_naming_justification.md`, `feedback_clean_architecture_requirements.md`
