---
doc_class: MigrationGuide
template_id: TPL-MIGRATION-GUIDE
microservice: docs
status: Deprecated
deprecation_date: 2026-05-17
removal_target: advisory — HG-DOCS accepts at p99 SLOs sustained 30d
related_adrs: [ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-DOCS-0001, ADR-DOCS-0002, ADR-DOCS-0003, ADR-DOCS-0004, ADR-DOCS-0005, ADR-DOCS-0006]
related_specs: [/specs/microservices/docs.json, /specs/microservices/docs/docs.json]
owner_team: axis-docs
date: 2026-05-17
doc_status: published
---

# Migration: `oya-connect-docs-*` → `oya-docs-*`

This document applies the Strangler Pattern from the agent-skills `deprecation-and-migration` skill to the **docs** µservice. It is the consumer-facing companion to ADR-0134 (cross-µservice migration policy) and ADR-0135 (target topology).

## Status

**Deprecated as of 2026-05-17 — replacement available and production-proven in dev cluster.**

| Field | Value |
|---|---|
| Replacement | `oya-docs-*` crate family under `microservices/docs/src/crates/` |
| Removal date | **Advisory** — concrete target is HG-DOCS accepts at p99 SLOs sustained 30d (per ADR-0135 retirement trigger #3) |
| Reason | ADR-0132 no-suite forward-policy + ADR-0130 per-µservice SLO authority + ADR-0131 per-µservice flat layout + the 8-BC docs surface (document-store / collab-crdt / block-types / comments-and-suggestions / version-history / sharing-and-permissions / export-import / embed-resolver) is only addressable at µservice granularity, not at Connect-suite granularity |
| Migration owner (Churn Rule) | axis-docs |
| Migration window | Phase 2 adapter + Phase 3 canary = ~6 months; Phase 5 removal sweep in month 7 (per ADR-0134) |

## Replacement

The 8 bounded-contexts of the `docs` µservice live under `microservices/docs/src/crates/` per ADR-0131. The legacy `oya-connect-docs-domain` crate bundled all eight surfaces; each replacement is BC-decomposed.

### Crate import-path map

The legacy connect-docs surface was a single bundled domain crate. Per ADR-0131 + ADR-0105 (13-layer enum), the new layout splits per bounded context. Migration imports from the legacy bundled `oya-connect-docs-domain` must each pick the specific replacement BC.

| Legacy `oya-connect-docs-*` path | New `oya-docs-*` path |
|---|---|
| `oya-connect-docs-domain` (bundled) | split per BC; see below |
| `oya-connect-docs-document-kernel` | `oya-docs-document-store-kernel` |
| `oya-connect-docs-document-domain` | `oya-docs-document-store-domain` |
| `oya-connect-docs-document-usecase` | `oya-docs-document-store-usecase` |
| `oya-connect-docs-document-api` | `oya-docs-document-store-api` |
| `oya-connect-docs-document-adapter` | `oya-docs-document-store-adapter` |
| `oya-connect-docs-document-adapter-postgres` | `oya-docs-document-store-adapter-postgres` |
| `oya-connect-docs-document-adapter-s3` | `oya-docs-document-store-adapter-s3` |
| `oya-connect-docs-document-rest` | `oya-docs-document-store-rest` |
| `oya-connect-docs-document-worker` | `oya-docs-document-store-worker` |
| `oya-connect-docs-document-sdk` | `oya-docs-document-store-sdk` |
| `oya-connect-docs-document-app` | `oya-docs-document-store-app` |
| `oya-connect-docs-collab-kernel` | `oya-docs-collab-crdt-kernel` |
| `oya-connect-docs-collab-domain` | `oya-docs-collab-crdt-domain` |
| `oya-connect-docs-collab-usecase` | `oya-docs-collab-crdt-usecase` |
| `oya-connect-docs-collab-api` | `oya-docs-collab-crdt-api` |
| `oya-connect-docs-collab-adapter` | `oya-docs-collab-crdt-adapter` (Loro wrapping per ADR-DOCS-0001) |
| `oya-connect-docs-collab-adapter-redis` | `oya-docs-collab-crdt-adapter-redis` |
| `oya-connect-docs-collab-worker` | `oya-docs-collab-crdt-worker` |
| `oya-connect-docs-collab-sdk` | `oya-docs-collab-crdt-sdk` |
| `oya-connect-docs-collab-app` | `oya-docs-collab-crdt-app` |
| `oya-connect-docs-blocks-kernel` | `oya-docs-block-types-kernel` |
| `oya-connect-docs-blocks-domain` | `oya-docs-block-types-domain` |
| `oya-connect-docs-blocks-usecase` | `oya-docs-block-types-usecase` |
| `oya-connect-docs-blocks-api` | `oya-docs-block-types-api` |
| `oya-connect-docs-blocks-adapter` | `oya-docs-block-types-adapter` |
| `oya-connect-docs-blocks-sdk` | `oya-docs-block-types-sdk` |
| `oya-connect-docs-blocks-app` | `oya-docs-block-types-app` |
| `oya-connect-docs-comments-kernel` | `oya-docs-comments-and-suggestions-kernel` |
| `oya-connect-docs-comments-domain` | `oya-docs-comments-and-suggestions-domain` |
| `oya-connect-docs-comments-usecase` | `oya-docs-comments-and-suggestions-usecase` |
| `oya-connect-docs-comments-api` | `oya-docs-comments-and-suggestions-api` |
| `oya-connect-docs-comments-adapter` | `oya-docs-comments-and-suggestions-adapter` |
| `oya-connect-docs-comments-adapter-postgres` | `oya-docs-comments-and-suggestions-adapter-postgres` |
| `oya-connect-docs-comments-rest` | `oya-docs-comments-and-suggestions-rest` |
| `oya-connect-docs-comments-worker` | `oya-docs-comments-and-suggestions-worker` |
| `oya-connect-docs-comments-app` | `oya-docs-comments-and-suggestions-app` |
| `oya-connect-docs-version-kernel` | `oya-docs-version-history-kernel` |
| `oya-connect-docs-version-domain` | `oya-docs-version-history-domain` |
| `oya-connect-docs-version-usecase` | `oya-docs-version-history-usecase` |
| `oya-connect-docs-version-api` | `oya-docs-version-history-api` |
| `oya-connect-docs-version-adapter` | `oya-docs-version-history-adapter` |
| `oya-connect-docs-version-adapter-postgres` | `oya-docs-version-history-adapter-postgres` |
| `oya-connect-docs-version-worker` | `oya-docs-version-history-worker` |
| `oya-connect-docs-version-app` | `oya-docs-version-history-app` |
| `oya-connect-docs-sharing-kernel` | `oya-docs-sharing-and-permissions-kernel` |
| `oya-connect-docs-sharing-domain` | `oya-docs-sharing-and-permissions-domain` |
| `oya-connect-docs-sharing-usecase` | `oya-docs-sharing-and-permissions-usecase` |
| `oya-connect-docs-sharing-api` | `oya-docs-sharing-and-permissions-api` |
| `oya-connect-docs-sharing-adapter` | `oya-docs-sharing-and-permissions-adapter` |
| `oya-connect-docs-sharing-adapter-postgres` | `oya-docs-sharing-and-permissions-adapter-postgres` |
| `oya-connect-docs-sharing-rest` | `oya-docs-sharing-and-permissions-rest` |
| `oya-connect-docs-sharing-app` | `oya-docs-sharing-and-permissions-app` |
| `oya-connect-docs-export-kernel` | `oya-docs-export-import-kernel` |
| `oya-connect-docs-export-domain` | `oya-docs-export-import-domain` |
| `oya-connect-docs-export-usecase` | `oya-docs-export-import-usecase` |
| `oya-connect-docs-export-api` | `oya-docs-export-import-api` |
| `oya-connect-docs-export-adapter` | `oya-docs-export-import-adapter` |
| `oya-connect-docs-export-adapter-pandoc` | `oya-docs-export-import-adapter-pandoc` (per ADR-DOCS-0003 — backend-qualified) |
| `oya-connect-docs-export-adapter-weasyprint` | `oya-docs-export-import-adapter-weasyprint` (per ADR-DOCS-0003) |
| `oya-connect-docs-export-rest` | `oya-docs-export-import-rest` |
| `oya-connect-docs-export-worker` | `oya-docs-export-import-worker` |
| `oya-connect-docs-export-app` | `oya-docs-export-import-app` |
| `oya-connect-docs-embed-kernel` | `oya-docs-embed-resolver-kernel` |
| `oya-connect-docs-embed-domain` | `oya-docs-embed-resolver-domain` |
| `oya-connect-docs-embed-usecase` | `oya-docs-embed-resolver-usecase` |
| `oya-connect-docs-embed-api` | `oya-docs-embed-resolver-api` |
| `oya-connect-docs-embed-adapter` | `oya-docs-embed-resolver-adapter` |
| `oya-connect-docs-embed-rest` | `oya-docs-embed-resolver-rest` |
| `oya-connect-docs-embed-worker` | `oya-docs-embed-resolver-worker` |
| `oya-connect-docs-embed-app` | `oya-docs-embed-resolver-app` |

> **`oya-connect-docs-domain` split.** The legacy bundled crate bundled documents + collab + blocks + comments + versions + sharing + export + embed into a single domain-layer crate. Per ADR-0131 + ADR-0105 (13-layer enum), the new layout splits the domain layer per bounded context. Migration imports from the legacy bundled `oya-connect-docs-domain` must each pick the specific replacement BC; a one-line wholesale `use oya_docs::*` import is not supported.

### Net-new boundaries (no legacy counterpart)

The new µservice introduces capabilities that did NOT exist in `oya-connect-docs-*`. They are therefore not part of the migration surface — they are clean replacement-boundary features:

- **`oya-docs-collab-crdt-adapter` (Loro 1.x)** — first-class CRDT engine per ADR-DOCS-0001; the legacy `connect-docs-collab-*` used a primitive last-write-wins synchroniser. See Hyrum's-Law surface #1 below.
- **`oya-docs-block-types-*`** — Notion-class block-based schema per ADR-DOCS-0002; the legacy surface had only flat-text + heading + list.
- **`oya-docs-export-import-adapter-chromium`** — high-fidelity Chromium-headless PDF backend per ADR-DOCS-0003 (opt-in alternative to WeasyPrint).
- **`oya-docs-sharing-and-permissions-*` per-block ACL** — Notion-class per-block grant per ADR-DOCS-0004; the legacy surface had whole-doc-only ACL.
- **`oya-docs-embed-resolver-*`** — cross-µservice embed resolution with policy-bounded refresh; the legacy surface had no cross-µservice embed.
- **AI writing-assist (T1/T2) per ADR-DOCS-0005** — auto-summary, grammar-check, auto-translate, auto-cite; the legacy surface had none.
- **Audit-chain emission per ADR-0028** — Ed25519 + Merkle on every doc lifecycle; the legacy surface had only access logs.
- **eIDAS PAdES B-LT signed PDF export (pack-eu)** — new in M03-onward1; no legacy counterpart.

### Concrete import migration recipes

```rust
// BEFORE
use oya_connect_docs_document_kernel::{Document, BlockTree};
use oya_connect_docs_document_usecase::CreateDocument;
use oya_connect_docs_collab_kernel::{CollabOp, ServerSynchroniser};
use oya_connect_docs_comments_kernel::{Comment, Thread};

// AFTER
use oya_docs_document_store_kernel::{Document, BlockTree};
use oya_docs_document_store_usecase::CreateDocument;
use oya_docs_collab_crdt_kernel::{CrdtOp, CrdtMergeEngine};
use oya_docs_comments_and_suggestions_kernel::{Comment, Thread};
```

```toml
# BEFORE — Cargo.toml of a downstream consumer
[dependencies]
oya-connect-docs-document-kernel = { workspace = true }
oya-connect-docs-document-usecase = { workspace = true }
oya-connect-docs-collab-kernel = { workspace = true }
oya-connect-docs-comments-kernel = { workspace = true }

# AFTER
[dependencies]
oya-docs-document-store-kernel = { workspace = true }
oya-docs-document-store-usecase = { workspace = true }
oya-docs-collab-crdt-kernel = { workspace = true }
oya-docs-comments-and-suggestions-kernel = { workspace = true }
```

## Reason

The legacy `oya-connect-docs-*` family was authored before the following ADRs crystallised:

1. **ADR-0132 — no-suite forward-policy.** `connect-*` encodes bundle membership at the architecture layer; bundle membership is a brand-layer concept and must not appear in crate names.
2. **ADR-0130 — per-µservice SLO authority.** Docs needs independent SLO targets per surface (doc-open-latency, save-latency, collab-cursor-sync-latency, export-pdf-latency, search-within-doc-latency, doc-list-latency, crdt-merge-no-silent-loss 100% target, share-acl-enforcement-correctness 100% target, pandoc-export-pipeline-availability). A `connect-*` umbrella SLO cannot honour those.
3. **ADR-0131 — per-µservice flat layout.** Docs's IaC, runbooks, threat-model, DPIA, compliance, capacity-model, cost-budget, incident-response, failure-modes, multi-region, SDK plan, competitor-parity-matrix, backfill-replay all need to live under one folder (`microservices/docs/`).
4. **ADR-0133 — 11-pack-overlay program.** pack-kr (KR PIPA + 전자문서법), pack-eu (GDPR + EU AI Act + eIDAS PAdES), pack-us, pack-us-healthcare (HIPAA + FDA 21 CFR Part 11), pack-jp (APPI), pack-sg (PDPA), pack-au (Privacy Act), pack-in (DPDPA), pack-br (LGPD), pack-ae (UAE PDPL + Hijri overlay), pack-ksa (KSA PDPL + Hijri overlay + Sharia retention) — each lives as `microservices/docs/iac/kustomize/overlays/pack-<region>/`.
5. **ADR-DOCS-0001 → ADR-DOCS-0006** — docs-specific decisions (CRDT library shared with workflow-studio, block-type system, export pipeline, per-block ACL, AI writing-assist scope, DOCX import fidelity) need to live at per-µservice ADR granularity.
6. **Cross-µservice CRDT library alignment** with workflow-studio (per ADR-WS-0001 + ADR-DOCS-0001) — the legacy connect-docs collab adapter could not honour this because it predated ADR-WS-0001.

## Migration Guide (step-by-step)

For each consumer crate that imports `oya-connect-docs-*`:

### Step 1 — Add the new dependency

```bash
# In your consumer crate's Cargo.toml, add the new mapped dependency.
# Keep the legacy dependency for now (Phase 2 adapter soak).
```

### Step 2 — Update imports per the import-path map above

```bash
# Use this command per file as a guided rewrite (review every hit;
# manual disambiguation needed for the `oya-connect-docs-domain`
# split case):
rg -l "oya_connect_docs_" --type rust path/to/your/crate
```

### Step 3 — Verify behavioural parity

```bash
# Inside your consumer crate:
cargo nextest run --features connect-docs-strangler-canary
```

Run with the feature flag enabled to route through the new µservice; without to route through the legacy adapter. Compare:

- error variant ordering (Hyrum's Law — see surfaces below).
- p99 latency (must be ≤ legacy + 5% per ADR-0134 Phase 3 canary gate).
- CRDT op envelope shape (per ADR-DOCS-0001 — the new envelope is cross-µservice-consistent with workflow-studio; legacy was bespoke).
- Block-tree projection (per ADR-DOCS-0002 — new block schema is strictly typed; legacy was untyped string-with-markers).
- DOCX export byte stability (per ADR-DOCS-0003 — new pipeline uses Pandoc 3.x pinned LTS; legacy used Pandoc 2.x).

### Step 4 — Remove the legacy dependency

Only after your consumer crate's tests pass against the new imports AND the docs µservice's Phase 3 canary reaches 100% traffic (per ADR-0134), remove the legacy dependency from your `Cargo.toml`:

```toml
# Remove this line:
oya-connect-docs-document-kernel = { workspace = true }
```

### Step 5 — Verify zero residual

```bash
# Per ADR-0134 Phase 4 verification:
cargo tree -e normal -p your-crate | grep oya-connect-docs   # expect empty
rg "use oya_connect_docs_" --type rust path/to/your/crate    # expect zero hits
```

## Configuration delta

| Configuration key | Legacy | New |
|---|---|---|
| Feature flag namespace | `connect.docs.*` | `docs.*` |
| OpenSLO file | bundled in `Connect.openslo.yaml` (umbrella) | `microservices/docs/slos/*.openslo.yaml` (per-µservice, 9 files) |
| Helm chart values key | `.Values.connect.docs.*` | `.Values.docs.*` |
| K8s namespace | `connect` | `docs` |
| Cedar policy fragment path | `policy/connect/docs/*.cedar` | `microservices/docs/policy/*.cedar` |
| pack-kr overlay path | `policy/connect/docs/pack-kr/*` | `microservices/docs/iac/kustomize/overlays/pack-kr/*` + per-pack section in `threat-model.md` / `dpia.md` / `compliance.md` / `multi-region.md` |
| Workflow event prefix | `connect.docs.*` | `docs.*` (e.g., `docs.document.lifecycle.v1`, `docs.sharing.v1`, `docs.comments.v1`, `docs.suggestions.v1`, `docs.export.v1`, `docs.import.v1`, `docs.embed.v1`, `docs.version.v1`) |
| Ontology type prefix | `Connect.Docs.*` | `Docs.*` (e.g., `Docs.Document`, `Docs.Block`, `Docs.Comment`, `Docs.Suggestion`, `Docs.ShareGrant`, `Docs.Version`, `Docs.LegalHold`) |
| Telemetry metric prefix | `oya_connect_docs_*` | `oya_docs_*` |
| Tracing span attribute namespace | `connect.docs.*` | `docs.*` |
| CRDT engine | bespoke last-write-wins synchroniser | Loro 1.x per ADR-DOCS-0001 (cross-µservice consistent with workflow-studio) |
| Block schema | untyped string + markers | strictly-typed block schema per ADR-DOCS-0002 |
| Export pipeline | Pandoc 2.x in-process | Pandoc 3.x + WeasyPrint default + Chromium-headless opt-in inside gVisor sandbox per ADR-DOCS-0003 |
| ACL granularity | whole-doc | per-block per ADR-DOCS-0004 |
| AI writing-assist | none | T0/T1/T2 capability tier per ADR-DOCS-0005 |
| DOCX import fidelity | unbounded (could silently lose features) | best-effort tier with named edge-case matrix per ADR-DOCS-0006 |

## Dual-context isolation invariant (preserved + strengthened)

The Personal ↔ Professional context isolation invariant from the Bominal ADR-0208 dual-context inheritance is preserved verbatim in `oya-docs-document-store-kernel`. Specifically:

- The `DocumentContextBoundaryGuard` port trait keeps the same method signatures.
- Cross-context attempts (Professional → Personal document read) emit the same 403 + same audit-chain event variant (`DocsCrossContextRefused`).
- The kernel-layer refusal (not adapter-layer) invariant is preserved.
- **Strengthened**: cross-context attempts are also refused at the Cedar policy layer per `policy/editor-isolation.md`; the kernel refusal is the defence-in-depth backup.
- **Strengthened**: per-block ACL adds a third defence-in-depth layer per ADR-DOCS-0004.

## Hyrum's-Law surfaces — explicit callouts

Per the deprecation-and-migration skill SKILL.md §"Hyrum's Law Makes Removal Hard", these are the legacy docs surfaces with observable behaviour that may be depended on. Each is preserved verbatim during the canary; consumers must re-test after Phase 5 removal in case they had a long-tail dependency:

1. **CRDT operation ordering**. The legacy in-house synchroniser used last-write-wins by `wall_clock_at` timestamp. `Loro 1.x` uses RGA-tree semantics with version-vector ordering; concurrent edits to the same node now surface as conflicts rather than silent overwrite. Consumers that pattern-match on `wall_clock_at`-determined order MUST migrate to checking the new `Conflict` envelope. This is a deliberate strengthening (the legacy behaviour violated AC-06 never-silent-loss); the canary does NOT mask the divergence.
2. **Document-export byte stability across pandoc versions**. Legacy Pandoc 2.x emitted DOCX with stable byte sequences for a given input; new Pandoc 3.x has minor byte-level differences in OOXML emitter output (Pandoc 3.x emits more conformant DOCX per ECMA-376; legacy was looser). Consumers that pattern-match on byte-equality of exported DOCX MUST re-baseline. The runtime BEHAVIOUR (the parsed DOCX result in Word) is preserved; only the byte stream differs.
3. **Embed-resolver retry semantics**. Legacy embed resolution was best-effort fire-and-forget. New embed-resolver retries with exponential backoff + single-flight coalescing + stale-fallback (per `embed-source-stale-detection.md` runbook). Consumers that wrote retry logic on top of the legacy resolver MUST remove that wrapper (the new resolver subsumes it).
4. **Attachment URL signing TTL**. Legacy issued attachment URLs with 60-minute TTL fixed. New attachment URLs are signed with per-tenant TTL (default 15 minutes; configurable up to 24h via tenant policy). Consumers that cached attachment URLs past 15 minutes MUST refresh more aggressively or set their tenant policy to a longer TTL.
5. **Block enumeration order**. Legacy bundled `Document::blocks()` returned blocks in insertion-order regardless of CRDT operations. New `Document::blocks()` returns blocks in CRDT-tree-traversal order, which matches the user's visible order after merges. Consumers that depended on insertion-order MUST switch to `Document::blocks_in_insertion_order()` (preserved as an additive method); the default `blocks()` returns CRDT-tree order now.
6. **Comment anchor stability across edits**. Legacy anchors were `(byte_offset, length)`-based and broke on any text insertion. New anchors are CRDT-aware (`(block_id, start_tree_id, end_tree_id)`) and survive arbitrary edits. Consumers that pattern-matched on `byte_offset` MUST migrate to the new `Anchor` shape; the adapter provides a one-time anchor-migration utility.
7. **Suggestion auto-acceptance behaviour**. Legacy auto-accepted suggestions after 7d if not explicitly rejected. New µservice REQUIRES explicit accept-or-reject by the author; no auto-acceptance. Consumers that relied on auto-acceptance MUST update their tenant workflow.
8. **Export job blocking vs async**. Legacy export was synchronous (REST request blocked until export complete). New export is async (returns `ExportJob` with status URL; client polls or subscribes to `DocumentExported` event). Consumers that expected synchronous response MUST switch to async polling; SDK helper `awaitExport(job)` provides a blocking compatibility shim.

## Runbook continuity table

| Legacy runbook (under `policy/connect/docs/runbooks/`) | New runbook (under `microservices/docs/runbooks/`) | Status |
|---|---|---|
| `doc-restore.md` | `doc-version-restore-corruption.md` | preserved + expanded for CRDT op-log forensic |
| `share-acl-drift.md` | `share-acl-drift.md` | preserved + expanded for per-block ACL (ADR-DOCS-0004) |
| `attachment-restore.md` | `attachment-restore.md` | preserved + expanded with S3 Object Lock recovery |
| (no legacy counterpart) | `collab-conflict-resolution.md` | NEW per ADR-DOCS-0001 + AC-06 silent-loss invariant |
| (no legacy counterpart) | `export-pipeline-failure-pandoc-rollback.md` | NEW per ADR-DOCS-0003 |
| (no legacy counterpart) | `editor-session-storm-throttle.md` | NEW per editor-session-lease pressure |
| (no legacy counterpart) | `embed-source-stale-detection.md` | NEW per cross-µservice embed-resolver |

## Phases (per ADR-0134)

| Phase | Description | Status (docs) | Exit condition |
|---|---|---|---|
| 1. Parallel ship | New µservice + legacy coexist | **active** | HG-DOCS passes at p99 SLOs in dev cluster sustained 7d |
| 2. Adapter soak | `oya-connect-docs-migration-adapter` shims legacy symbols → new impl | pending | All consumers compile against adapter; 3-month soak elapses |
| 3. Feature-flagged canary | 10% → 50% → 100% traffic shift over 6 weeks | pending | New µservice carries 100% traffic for 7 consecutive days |
| 4. Zero-active-usage verification | Dependency-graph + telemetry + grep all clean | pending | Verification commands all exit 0 |
| 5. Code removal sweep | Delete legacy crates + Cargo.toml entries + spec pointers | pending | `cargo build --workspace` exits 0; no `oya_connect_docs_*` symbol resolves |
| 6. Umbrella retirement | Conditional on all 8 sub-µservices reaching their own Phase 5 | pending | All 8 HG-<MS> gates green at p99 SLO sustained 30d |

## Verification checklist (per skill SKILL.md §"Verification")

Per the deprecation-and-migration skill, every deprecation closeout must satisfy these checks:

- [ ] **Replacement is production-proven and covers all critical use cases.**
  ```bash
  cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice docs
  # expect: HG-DOCS accepts at p99 SLOs sustained 30d
  ```
- [ ] **Migration guide exists with concrete steps and examples.**
  ```bash
  test -f microservices/docs/migration-from-connect.md   # this file
  ```
- [ ] **All active consumers have been migrated** (per Phase 4):
  ```bash
  cargo tree -e normal -p oya-connect-docs-domain --invert    | grep -v 'oya-connect-docs-migration-adapter' | wc -l   # expect 0
  rg "use oya_connect_docs_" --type rust    | rg -v "migration-adapter|legacy_in_process|tests/"    | wc -l   # expect 0
  ```
- [ ] **Old code, tests, documentation, configuration removed** (per Phase 5):
  ```bash
  find crates -maxdepth 1 -type d -name "oya-connect-docs-*" | wc -l   # expect 0
  test ! -f /specs/microservices/docs.json                          # expect file absent
  ```
- [ ] **No references to the deprecated system remain in the codebase** (excluding historical ADR / RETIRED.md / git-log surfaces):
  ```bash
  rg "oya_connect_docs" --type rust    | rg -v "docs/decisions/|RETIRED.md|tests/golden/"    | wc -l   # expect 0
  ```
- [ ] **Deprecation notices removed (they served their purpose)** (per Phase 5):
  ```bash
  test ! -f microservices/docs/deprecation-notice.md          # expect file absent
  test ! -f microservices/docs/migration-from-connect.md      # expect file absent (this file removes itself in Phase 5)
  ```

## Breaking changes (flagged per `feedback_no_silent_regression`)

This migration is **NOT a breaking change** during Phases 1–4 for the core symbol surface: the adapter preserves the legacy symbol surface verbatim, including error variant ordering and timing characteristics within the +5% canary tolerance.

**There ARE eight behavioural strengthenings** that may visibly differ from legacy and are NOT preserved by the adapter (per `feedback_no_silent_regression`):

1. **Loro CRDT ordering replaces last-write-wins** (per ADR-DOCS-0001 + Hyrum #1). Adapter does NOT mask divergence; documented in migration guide. This is a deliberate strengthening that closes the AC-06 silent-loss vulnerability.
2. **Pandoc 3.x DOCX byte-level differences** (per ADR-DOCS-0003 + Hyrum #2). Adapter does NOT mask; documented.
3. **Embed-resolver retry semantics** (Hyrum #3). Adapter does NOT mask; consumers must remove their own retry wrappers.
4. **Attachment URL signing TTL configurable** (Hyrum #4). Adapter does NOT mask; consumers refresh.
5. **Block enumeration order** (Hyrum #5). Adapter provides an additive `blocks_in_insertion_order()` method; default `blocks()` returns CRDT-tree-traversal order.
6. **Comment anchor CRDT-aware** (Hyrum #6). Adapter provides one-time anchor-migration utility.
7. **Suggestion no longer auto-accepts after 7d** (Hyrum #7). Adapter does NOT mask; consumers update workflow.
8. **Export is async, not sync** (Hyrum #8). Adapter provides `awaitExport(job)` shim.

Phase 5 (code removal) **IS a breaking change** for any consumer that did not migrate during the 6-month adapter+canary window. Per `feedback_no_silent_regression`:

- Sunset schedule (advisory): 7 months from this document's `deprecation_date` (2026-05-17), so a target advisory removal date of **2026-12-17** (subject to the HG-DOCS retirement trigger gating).
- Owning axis (axis-docs) ships migration ChangeSets for every internal consumer per the Churn Rule before Phase 5.
- External consumers (reading `/specs/microservices/docs.json`) receive a 7-month sunset window; the spec file's `deprecated: true` + `replacement_path: /specs/microservices/docs/docs.json` fields render in the agent-coordination dashboard.

## References

- ADR-0135: Connect super-app expansion into 8 flat µservices.
- ADR-0131: Per-microservice flat layout.
- ADR-0132: No-suite forward-policy.
- ADR-0133: Industry best-practice conformance program.
- ADR-0134: Connect dissolution Strangler migration (operational policy).
- ADR-DOCS-0001: CRDT library — Loro 1.x (cross-µservice consistent with workflow-studio ADR-WS-0001).
- ADR-DOCS-0002: Block-type system.
- ADR-DOCS-0003: Export pipeline architecture.
- ADR-DOCS-0004: ACL granularity (per-block).
- ADR-DOCS-0005: AI writing-assist EU AI Act bounds.
- ADR-DOCS-0006: DOCX import fidelity policy.
- ADR-WS-0001: workflow-studio CRDT library selection (Loro) — cross-µservice alignment.
- `microservices/docs/PRD.md` — full target-state product definition.
- `microservices/docs/PHASE-01-DOCS-FOUNDATION.md` — phase plan.
- `microservices/docs/deprecation-notice.md` — formal deprecation notice.
- `feedback_no_silent_regression.md` — no-silent-regression principle.
- agent-skills deprecation-and-migration SKILL.md — Strangler Pattern + Adapter Pattern + Churn Rule + Verification.
- ECMA-376 — OOXML reference (cited by ADR-DOCS-0006).
- ISO 19005-1 (PDF/A-1b); ISO 19005-2 (PDF/A-2u) — archival PDF.
- CommonMark spec + GFM — Markdown.
- HTML5 specification — HTML export.
- EPUB 3 spec — EPUB export.
