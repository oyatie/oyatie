---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M01-foundation
phase: P01-translate-platform
status: Active
entry_gate: |
  ADR-0131 + ADR-0135 accepted; ADR-TRANSLATE-0001..0006 accepted; foundry-providers µservice deployed and reachable; foundry-runtime µservice deployed with in-house MT + QE + LangDetect endpoints; cloud-secrets (OpenBao) deployed; observability SLO ledger live; cargo workspace ready to accept new crates under microservices/translate/src/crates/.
exit_gate: |
  All 15 IPs merged; `oya-translate-credential-isolation` LEAN lane present in .github/branch-protection.yaml required_status_checks on dev and staging; `oya-translate-data-residency-correctness` BLOCKER lane present on all branches; translate-router decision p99 ≤ 5 ms verified via load test; translation-request p95 (in-house pair) ≤ 250 ms verified; real-time caption stream p99 ≤ 400 ms per-chunk verified; document translate 10-page DOCX p95 ≤ 8 s verified; bulk-translate 10 k-segment XLIFF p95 ≤ 60 s verified; `cargo nextest run --workspace` exits 0; `oya gate validate per-microservice-layout --microservice translate` exits 0; `oya gate validate authority-cohesion` exits 0; HG-TRANSLATE gate in /specs/hyperscaler-gates.json registers green.
depends_on:
  - milestone: M01-foundation
    phase: foundry-providers/P01-provider-adapter-substrate
    reason: External MT vendors (Anthropic / OpenAI / Google / DeepL) reach via foundry-providers ProviderInvoker; never direct
  - milestone: M01-foundation
    phase: foundry-runtime/P01-agent-runtime-and-capability-execution
    reason: In-house MT + QE + LangDetect model inference runs as capabilities in foundry-runtime
  - milestone: M01-foundation
    phase: cloud-secrets/P01-openbao-substrate
    reason: Vendor credentials resolved via OpenBao SecretReference (per foundry-providers credential isolation)
  - milestone: M01-foundation
    phase: observability/P01-agentic-slo-gated-promotion
    reason: SLO ledger consumes translate-router burn-rate signals for engine demote/recover
  - milestone: M01-foundation
    phase: tenancy/P01-rls-and-residency-pack
    reason: Per-tenant residency pack lookup determines engine eligibility
  - milestone: M01-foundation
    phase: audit-chain/P01-event-seal-substrate
    reason: Every translation seal flows to audit-chain
owner_team: axis-translate + ops-security + council-privacy
related_adrs: [ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-TRANSLATE-0001, ADR-TRANSLATE-0002, ADR-TRANSLATE-0003, ADR-TRANSLATE-0004, ADR-TRANSLATE-0005, ADR-TRANSLATE-0006]
related_specs: [/specs/per-microservice-flat-layout.json]
date: 2026-05-17
doc_status: published
---

# P01-translate-platform: Land `translate` µservice end-to-end

## Purpose

This phase ships the full ADR-0135 + ADR-TRANSLATE-0001..0006 design for the `translate` microservice: a `translate-router` capability-aware engine selector, per-vendor adapters (Anthropic / OpenAI / Google Cloud Translation / DeepL via foundry-providers + in-house via foundry-runtime), a per-tenant Translation Memory + termbase + glossary stack, a Quality Estimation + Language Detection pair, a document-translation round-trip pipeline (Pandoc + LibreOffice in gVisor), a real-time caption-translation stream, and a bulk-translate worker.

It is delivered as one phase in M01-foundation because every workload µservice (mail, messenger, social, docs, sheets, slides, meet, shorts, workflow-studio) depends on stable `TranslateInvoker` ports before they can advance past `dev` per the per-µservice gate posture.

This phase advances master-plan principles:
- **Hyperscaler-grade in every practice** (in-process router p99 ≤ 5 ms; per-call Ed25519 envelope seal; gVisor sandboxed document parsers; ≥ 99.95 % MT availability).
- **Nothing scheduled-for-distinct-tracked-work within scope** (every adapter ships with full credential isolation; gVisor sandbox on all parsers; EU AI Act disclosure baked into every call).
- **No silent regression** (per-tenant engine pin + content-class routing + canonical-base neutrality + LEAN-A10).
- **Per-microservice flat layout** (this phase authors under `microservices/translate/` per ADR-0131; no sibling µservice surface touched).
- **Canonical base + pack overlays** (KR + EU + JP + CN-stub overlays in `iac/kustomize/overlays/` per ADR-0064).

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `translate` | `translate-router`, `translation-memory`, `termbase-and-glossary`, `quality-estimation`, `language-detection`, `document-translation`, `bulk-translate`, `real-time-stream`, `engine-adapters` (per vendor + per backend) | All under `microservices/translate/` per ADR-0131 | `oya-translate-router-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}`, `oya-translate-tm-*`, `oya-translate-termbase-*`, `oya-translate-qe-*`, `oya-translate-langdetect-*`, `oya-translate-doc-*`, `oya-translate-bulk-*`, `oya-translate-stream-*`, `oya-translate-adapter-{postgres,redis,s3,meilisearch,foundry-runtime,anthropic,openai,google-translate,deepl,pandoc,libreoffice}` |

Plus these repo-wide artifacts (cross-cutting per ADR-0131):
- `.github/branch-protection.yaml` — add `oya-translate-credential-isolation` BLOCKER lane and `oya-translate-data-residency-correctness` BLOCKER lane to `required_status_checks` on `dev` + `staging`.
- `docs/standards/translate.md` (NEW) — cross-cutting adapter-conformance, residency-bound-inference invariants, EU AI Act disclosure schema, placeholder-preservation rules.
- `Cargo.toml` (workspace) — register new crates under `microservices/translate/src/crates/`.
- `/specs/hyperscaler-gates.json` — register HG-TRANSLATE gate.
- `/specs/regulatory-packs.json` — add `translate` µservice activation column.

### Out-of-scope

- Human-translator marketplace (Tier-G); tracked under Phase 3.
- TMX 2 (GALA TAPICC proposed) — track as it stabilizes; M01 supports TMX 1.4.
- CN-pack production activation — `pack-cn-stub` overlay scaffolding only in M01.
- Per-tenant fine-tuned domain-adapted models — tracked under ADR-0026 Phase 4.
- Yandex Translate adapter — not in M01.
- Apple Translate parity — not applicable (OS-bundled; we cover via in-house MT through shorts).

## Implementation Plans

Ordered list. Each IP is an executable ChangeSet under this phase folder. Dependencies inline.

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-iac-and-pack-overlays.md`](IP-001-iac-and-pack-overlays.md) | Helm chart + kustomize base + pack-kr/pack-eu/pack-jp/pack-cn-stub overlays; Postgres + Valkey + Meilisearch + S3 wiring | pending | axis-translate + ops-iac | — |
| [`IP-002-translate-router-kernel.md`](IP-002-translate-router-kernel.md) | `oya-translate-router-kernel`: port traits (`TranslateInvoker`, `EngineRouter`, `TmLeverageQuery`, `TermbaseQuery`, `QualityEstimator`, `LanguageDetector`, `DocumentTranslator`), entities, sealed traits | pending | axis-translate | IP-001 |
| [`IP-003-translate-router-domain.md`](IP-003-translate-router-domain.md) | `oya-translate-router-domain`: routing algebra (capability-fit × cost × residency × quality-tier × health); placeholder/plural preservation logic; pure | pending | axis-translate | IP-002 |
| [`IP-004-translate-router-usecase-and-api.md`](IP-004-translate-router-usecase-and-api.md) | `oya-translate-router-usecase` + `oya-translate-router-api`: orchestration + protocol-neutral typed contracts | pending | axis-translate | IP-003 |
| [`IP-005-translation-memory-stack.md`](IP-005-translation-memory-stack.md) | `oya-translate-tm-*` (kernel/domain/usecase/api/adapter-postgres/adapter-meilisearch/rest/worker/sdk/app); minhash-LSH leverage per ADR-TRANSLATE-0002 | pending | axis-translate | IP-002 |
| [`IP-006-termbase-and-glossary-stack.md`](IP-006-termbase-and-glossary-stack.md) | `oya-translate-termbase-*` (kernel/domain/usecase/api/adapter-postgres/rest/worker/sdk/app); TBX import/export | pending | axis-translate | IP-002 |
| [`IP-007-quality-estimation-stack.md`](IP-007-quality-estimation-stack.md) | `oya-translate-qe-*`; COMET-Kiwi-class via foundry-runtime; EU AI Act bounds per ADR-TRANSLATE-0003 | pending | axis-translate + council-privacy | IP-002 |
| [`IP-008-language-detection-stack.md`](IP-008-language-detection-stack.md) | `oya-translate-langdetect-*`; FastText/LangID-class detection via foundry-runtime | pending | axis-translate | IP-002 |
| [`IP-009-document-translation-stack.md`](IP-009-document-translation-stack.md) | `oya-translate-doc-*`; Pandoc + LibreOffice in gVisor sandbox per ADR-TRANSLATE-0005 | pending | axis-translate + ops-security | IP-002 |
| [`IP-010-bulk-translate-stack.md`](IP-010-bulk-translate-stack.md) | `oya-translate-bulk-*`; XLIFF/TMX/TBX I/O; async job worker | pending | axis-translate | IP-005, IP-006 |
| [`IP-011-real-time-stream-stack.md`](IP-011-real-time-stream-stack.md) | `oya-translate-stream-*`; sentence-piece chunking + correction-replay per ADR-TRANSLATE-0006 | pending | axis-translate + axis-meet | IP-002 |
| [`IP-012-engine-adapter-foundry-runtime.md`](IP-012-engine-adapter-foundry-runtime.md) | `oya-translate-adapter-foundry-runtime`: in-house MT/QE/LangDetect capability invocation | pending | axis-translate | IP-002 |
| [`IP-013-engine-adapters-external.md`](IP-013-engine-adapters-external.md) | `oya-translate-adapter-{anthropic,openai,google-translate,deepl}`; via foundry-providers | pending | axis-translate + ops-security | IP-002, foundry-providers |
| [`IP-014-router-rest-worker-app.md`](IP-014-router-rest-worker-app.md) | REST surface + worker (engine-health monitor + cost roll-up) + composition-root app binary | pending | axis-translate | IP-004, IP-012, IP-013, IP-005, IP-006, IP-007, IP-008, IP-009, IP-010, IP-011 |
| [`IP-015-hg-translate-gate-registration.md`](IP-015-hg-translate-gate-registration.md) | Register HG-TRANSLATE hyperscaler gate; SDK scaffold (Rust + TS); branch-protection lane additions | pending | axis-translate + gtm | IP-014 |

## Per-IP Test Coverage Threshold

| Layer | Line coverage | Branch coverage | Property tests | Notes |
|---|---|---|---|---|
| kernel | 90 % | 80 % | required | port-sealed; entity-invariant tests |
| domain | 95 % | 90 % | required | routing algebra + placeholder/plural rules are math; canonical worked examples |
| usecase | 85 % | 70 % | optional | orchestration |
| adapter (backend-store) | 80 % | 70 % | as-needed | testcontainers (Postgres/Redis/Meilisearch); honest integration tests under tests/integration |
| adapter (engine-vendor) | 80 % | 70 % | as-needed | upstream-mocked; live-key smoke tests behind feature flag |
| rest / worker / app | 70 % | 60 % | optional | thin composition layers |
| sdk | 80 % | 70 % | optional | client surface |
| document parser (gVisor sandboxed) | 75 % | 60 % | required | fuzz corpus for malformed DOCX/PDF/PPTX/XLSX/HTML |

## Verification

```bash
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice translate
cargo run -p oya-dev-cli -- gate validate authority-cohesion
cargo run -p oya-dev-cli -- gate validate credential-isolation --microservice translate
cargo run -p oya-dev-cli -- gate validate data-residency-correctness --microservice translate
cargo run -p oya-dev-cli -- gate validate eu-ai-act-disclosure --microservice translate
```

All exit 0.

Load tests (`tests/load/`) demonstrate every performance budget in PRD §"Performance". End-to-end drill: synthetic translation request routed across Anthropic API, DeepL, in-house adapter; each emits `TranslationCompleted` audit event with Ed25519 seal; pack-cn-stub deploy refuses all external engines (only in-house permitted).

## References

- `microservices/translate/PRD.md`.
- ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133.
- ADR-TRANSLATE-0001 .. ADR-TRANSLATE-0006.
- `/specs/per-microservice-flat-layout.json`.
- `docs/standards/observability-slo.md`.
