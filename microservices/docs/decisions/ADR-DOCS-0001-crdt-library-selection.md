---
id: ADR-DOCS-0001
title: CRDT library selection for docs µservice — Loro 1.x (cross-µservice consistent with workflow-studio)
microservice: docs
status: Accepted
date: 2026-05-17
owner: axis-docs + council-architecture
deciders: council-architecture, axis-docs, axis-workflow, ops-security
supersedes: []
superseded_by: []
related: [ADR-0065, ADR-0105, ADR-0131, ADR-WS-0001]
related_specs: [/specs/microservices/docs.json, /specs/microservices/workflow-studio.json]
related_artifacts:
  - microservices/docs/PRD.md (FR-03, AC-02, AC-05, AC-06)
  - microservices/docs/PHASE-01-DOCS-FOUNDATION.md (IP-006)
  - microservices/docs/dashboards/collab-health.json
  - microservices/docs/IP-006-collab-crdt-kernel-domain.md
  - microservices/workflow-studio/decisions/ADR-WS-0001-crdt-library-selection.md
purpose: |
  Resolve PRD Open Question (CRDT pick) for the docs µservice's collab-crdt
  bounded context, with the round-trip byte-equality (AC-02) and never-silent-
  loss (AC-06) invariants as load-bearing constraints. Cross-µservice CRDT
  consistency with workflow-studio per ADR-WS-0001 is a separate load-bearing
  constraint — the docs CrdtOp envelope shape MUST be cross-µservice-aligned
  with workflow-studio's so SDK clients + observability + audit-chain can
  treat both µservices' op streams identically.
doc_status: published
---

# ADR-DOCS-0001: CRDT library selection — Loro 1.x (cross-µservice aligned)

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

The docs µservice's `collab-crdt` bounded context (PRD §"Bounded Contexts") requires a conflict-free replicated data type (CRDT) substrate for real-time multi-user editing of documents. Per PRD §"Tenant Outcome 2" and AC-06, the load-bearing invariant is **"never silent loss"**: two business users editing the same document concurrently MUST have their non-conflicting edits merged, and conflicting edits MUST surface an explicit conflict UI — last-writer-wins is forbidden.

Documents are tree-shaped (root → blocks → nested blocks → inline runs). The block schema per ADR-DOCS-0002 (Notion-style block primitives) is a JSON canonical form. AC-02 demands that **load(emit(doc)) is byte-equal to the original spec** over a 100-doc golden corpus. The CRDT layer sits between the editor canvas and the document-store, so the CRDT's serialized state MUST be projectable to the canonical JSON without lossy conversion or key-ordering nondeterminism.

Performance envelope (PRD §"Performance"):
- Collab CRDT cursor sync p99 ≤ 150ms (mirrors workflow-studio).
- Save p99 ≤ 100ms.
- Up to 50k concurrent editor sessions per cell.
- 1M active documents per cell baseline.

**Critical cross-µservice constraint**: the workflow-studio µservice settled on **Loro 1.x** per ADR-WS-0001 for its collab-crdt BC. Docs and workflow-studio are siblings (both n8n/Notion-class hero products), and tenants will routinely:
1. Embed workflow-studio canvases into docs (per embed-resolver BC).
2. Build workflows that mutate docs (per workflow-engine triggers consuming `DocumentEdited` events).
3. Use the same SDK clients for both µservices (per `sdk-plan.md`).

If docs picks a different CRDT library, the cross-µservice CrdtOp envelope schema diverges; SDK clients fragment; cross-µservice observability bifurcates; cross-µservice audit-chain replay logic doubles. The cost of divergence is qualitatively higher than the cost of accepting workflow-studio's library choice.

Substrate constraints:
- Rust kernel (ADR-0105 layer enum); CRDT merge engine lives in `oya-docs-collab-crdt-domain` (pure, deterministic).
- Browser-WASM target (per ADR-0065 Leptos WASM frontend); CRDT library MUST compile to `wasm32-unknown-unknown`.
- Ephemeral CRDT state in Valkey (PRD §"Horizontal Scalability" mixed-state strategy); reconstructable from Postgres on cold-start.
- WebSocket gateway long-lived worker (`-worker` crate) fans out CRDT ops via consistent-hash on `document_id` to keep collaborators on the same pod.

Operational constraints:
- The CRDT library is supply-chain critical.
- License must be permissive (MIT / Apache-2.0); copyleft (GPL/AGPL) is incompatible with the closed-source SDK distribution model.

## Decision

Adopt **Loro 1.x** (`crates.io/crates/loro`) as the docs µservice's CRDT library, with the following constraints aligned to ADR-WS-0001:

1. **Cross-µservice envelope shape pinned**: the docs CrdtOp envelope schema (per `contracts/proto/docs.proto` + `contracts/asyncapi/docs-events.yaml` `docs.collab.crdt.v1` channel) is BYTE-COMPATIBLE with workflow-studio's CrdtOp envelope. A shared `oya-shared-crdt-envelope` crate publishes the type definition; both µservices' `-kernel` crates re-export from it. A new BLOCKER CI lane `oya-governance-crdt-cross-microservice-consistency` validates byte-compatibility at every PR; divergence is a build failure.
2. **Loro version pin matches workflow-studio**: both µservices pin to the same Loro semver line (`^1.0` at M03). Major-version upgrades are co-decided by axis-docs + axis-workflow; a fresh round-trip-corpus drill against the 100-doc golden corpus (docs side) + 100-spec corpus (workflow-studio side) is required before either µservice merges the upgrade.
3. **Loro types wrapped in port traits**: the same shape as workflow-studio. `oya-docs-collab-crdt-kernel` declares `CrdtMergeEngine`, `CrdtState`, `MergeOp`, `Conflict` port traits; Loro is an implementation detail of the `-domain` + `-adapter` crates. Public APIs across BCs MUST NOT leak Loro types.
4. **Loro tree CRDT backs the block tree**. Loro maps back block attribute dictionaries; Loro lists back block children ordering; Loro text backs inline runs. Conflict surfacing uses Loro's built-in version-vector + frontier API; the `Conflict` entity in the kernel wraps Loro `Frontiers` into a UI-renderable shape.
5. **CRDT-to-spec projection (`emit`) deterministically orders Loro nodes by their stable `TreeID`s, with map keys lex-sorted at the document-store boundary. This makes Loro state projection deterministic and is the seam that lets AC-02 (round-trip byte-equality) hold even though Loro's internal op log is not byte-canonical.
6. **Loro snapshot encoding is used for Valkey persistence; JSON projection is used only for the canonical spec emission**.
7. **Version-aligned op-log compaction**: per the discussion in PRD §"Open Questions" #1, op-log is compacted at version increments (default every 100 versions). Compaction runs through the same pinned Loro version + re-projects to canonical block tree; AC-02 byte-equality is preserved.
8. **The collab-crdt CI lane** (`oya-governance-crdt-no-silent-loss`) runs Loro's example test suite + docs's own AC-06 property test (10 concurrent editors, randomized op interleaving, assertion that every accepted op is reachable from final state OR surfaced as conflict — never silently dropped).
9. **Loro authorship at WS gateway**: every CRDT op carries OIDC-derived author SPIFFE-identity + Ed25519 signature added at the WS gateway boundary (per `policy/editor-isolation.md` §"CRDT Op Authenticity"). Unsigned ops refused at adapter boundary.

## Alternatives Considered

### Alternative A — Yjs (via yrs, the Rust port)

Yjs is the most-deployed CRDT library in production (Notion's collaborative editor uses Yjs).

- **Pros**
  - Largest production install base in document-collab; Notion validated at scale.
  - Mature ecosystem (yjs-prosemirror, y-websocket reference server).
  - Strong tree handling via `Y.XmlFragment` / `Y.Array`.
- **Cons**
  - **Divergence with workflow-studio's Loro pick (ADR-WS-0001) is the dominant cost**: SDK clients would have to handle two CRDT libraries; cross-µservice observability bifurcates; audit-chain replay logic doubles. ~6 engineer-months of incidental complexity over the M03–M05 horizon.
  - Yjs's `Y.XmlFragment` is a fit for prose-only docs but is awkward for the Notion-style block-tree (per ADR-DOCS-0002) where each block has rich typed attributes; would need an emulation layer.
  - yrs lags Yjs feature parity by ~2 minor versions historically.
- **Rejected reason**: cross-µservice divergence cost dominates. Yjs's production install base does not outweigh the cost of two CRDT libraries in one product family.

### Alternative B — Automerge 2.0

- **Pros**
  - Academic provenance (Kleppmann et al.).
  - JSON-document fit; documents are JSON-shaped natively.
  - Byte-canonical encoded changes (`save()` is deterministic for a given history).
  - Apache-2.0 licensed.
- **Cons**
  - Same divergence cost as Yjs (workflow-studio is on Loro).
  - No native tree CRDT; trees emulated via maps + list-of-children.
  - WASM bundle size larger than Loro by ~2-3x (Automerge ~600 KB gzip vs Loro ~250 KB gzip per upstream 2025-Q4 benchmarks); the editor TTI budget at M03 cannot absorb this.
  - History-preservation-first design retains every op forever unless compacted; storage cost concern for long-lived docs (per `cost-budget.md`).
- **Rejected reason**: cross-µservice divergence + bundle size + storage cost together exceed the value.

### Alternative C — Bespoke Rust CRDT

A docs-authored CRDT for exactly the operations the block tree needs.

- **Pros**
  - Zero external dependency.
  - Tailored to docs's canonical block-tree projection.
  - No third-party version-pinning + upgrade tax.
- **Cons**
  - CRDT correctness proofs are research-grade work; bespoke implementations have historically taken multiple years to reach production stability.
  - No external review surface; AC-06 violations only caught by oyatie's own property tests.
  - **Still creates cross-µservice divergence with workflow-studio**: bespoke would need cross-µservice consistency review with axis-workflow which has not approved a bespoke option.
  - Owns the WASM build target end-to-end.
- **Rejected reason**: build-vs-buy unfavorable; cross-µservice divergence cost remains.

### Alternative D — Hand-rolled Operational Transform (OT)

The pre-CRDT industry standard; Google Docs OT.

- **Pros**
  - Mature literature.
  - Server-mediated; simpler when one server is authoritative.
- **Cons**
  - Requires a single authoritative server per document; incompatible with multi-cell horizontal scale-out (PRD §"Horizontal Scalability").
  - OT transform functions are notoriously fragile to new operation types (every new op needs O(n²) transform functions).
  - "Never silent loss" requires explicit conflict resolution on every transform; in practice OT systems fall back to last-writer-wins under load.
- **Rejected reason**: incompatible with horizontal scale-out + the AC-06 invariant. Workflow-studio settled this question already; docs follows.

## Consequences

### Architectural

- The `oya-docs-collab-crdt-adapter` depends on Loro 1.x (pinned `^1.0` with vendor advisory subscription matched to workflow-studio's pin).
- `oya-docs-collab-crdt-kernel` declares `CrdtMergeEngine`, `CrdtState`, `MergeOp`, `Conflict` port traits with no Loro types in the signature; the Loro adapter lives behind these ports.
- The document-store reads the CRDT projection through a single `project_to_canonical(state) -> CanonicalBlockTree` boundary; this is the seam that makes AC-02 byte-equality hold despite Loro's non-canonical internal encoding.
- The cross-µservice envelope crate `oya-shared-crdt-envelope` is added to the workspace at the repo root (this is a brand-layer name; outside any µservice's `src/crates/`); both `oya-docs-collab-crdt-kernel` and `oya-workflow-studio-collab-crdt-kernel` import from it.
- WASM bundle gains ~250 KB gzip from Loro (same as workflow-studio).

### Downstream impact on other µservices and IPs

1. **IP-006 + IP-007** — adopt Loro as the concrete merge engine; property test suite for AC-06 runs Loro op streams + randomized-interleaving fuzzer.
2. **workflow-studio** — co-decided upgrade cadence; co-authored migration ADR when Loro 2.x lands.
3. **embed-resolver BC** — `oya-docs-embed-resolver` consumes workflow-studio canvas snapshots; the CRDT op envelope shape is reused at the cross-µservice mTLS boundary.
4. **observability** — `dashboards/collab-health.json` includes `crdt_library=loro` dimension shared with workflow-studio.
5. **SDK** — Rust + TypeScript SDKs share the Loro client binding; cross-µservice SDK consumption uses the same envelope shape.

### SLOs gaining new dimensions

- `docs.collab_merge_latency_p99` — tagged with `crdt_library=loro`.
- `docs.collab_no_silent_loss_count` — Sev-1 alert if non-zero in any 24h window (AC-06).
- `docs.crdt_wasm_bundle_size_bytes` — release-gated.
- `docs.crdt_envelope_schema_mismatch_total` — cross-µservice consistency lane drives this metric; Sev-2 alert if non-zero.

### Supply-chain + security

- Loro added to `cargo deny` allowlist with explicit version pin matching workflow-studio.
- Major-version Loro upgrade gated on: (a) 100-doc round-trip-corpus drill green, (b) AC-06 property test suite green, (c) WASM bundle size delta ≤ +50 KB gzip, (d) workflow-studio co-decision evidence on file.
- Loro upstream maintainers notified out-of-band of any CRDT-correctness issue oyatie surfaces.

### Risk register

- **Risk**: Loro pre-1.0 vintage of any subsequent breaking-change release. **Mitigation**: pin to `^1.0` only; major-version upgrades require fresh corpus + property tests + cross-µservice co-decision.
- **Risk**: Loro maintainer-attrition. **Mitigation**: kernel port-trait wrapper makes library swap a contained refactor; co-decided with workflow-studio.
- **Risk**: Cross-µservice envelope schema divergence over time (drift between docs + workflow-studio). **Mitigation**: BLOCKER CI lane `oya-governance-crdt-cross-microservice-consistency`.
- **Risk**: Bundle-size creep over Loro minor versions. **Mitigation**: `crdt_wasm_bundle_size_bytes` SLO + release gate.

## References

- PRD `microservices/docs/PRD.md` FR-03, AC-02, AC-05, AC-06.
- `microservices/docs/PHASE-01-DOCS-FOUNDATION.md` IP-006, IP-007.
- `microservices/docs/dashboards/collab-health.json`.
- ADR-WS-0001 — workflow-studio CRDT library selection (primary cross-µservice authority).
- ADR-DOCS-0002 — block-type system (canonical form).
- Loro — `loro.dev`, `github.com/loro-dev/loro`.
- Yjs / yrs — `github.com/yjs/y-crdt`.
- Automerge 2.0 — `automerge.org`.
- Shapiro, M. et al. (2011), "Conflict-free Replicated Data Types," INRIA RR-7687.
- Preguiça, N. (2018), "Conflict-free Replicated Data Types: An Overview," arXiv:1806.10254.
