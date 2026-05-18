---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-translate
microservice: translate
status: Accepted
sales_segment: shared-substrate + standalone
tier: hero-and-substrate
milestone_first_ship: M01-foundation
bominal_source: NONE  # net-new per ADR-0135 (no Bominal antecedent)
related_adrs: [ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-TRANSLATE-0001, ADR-TRANSLATE-0002, ADR-TRANSLATE-0003, ADR-TRANSLATE-0004, ADR-TRANSLATE-0005, ADR-TRANSLATE-0006]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
owner_team: axis-translate + ops-security + council-privacy
doc_status: published
---

# PRD-translate: Machine Translation + i18n Localization Platform

## Purpose

The `translate` microservice is oyatie's machine-translation (MT) and software-localization platform. It is **net-new per ADR-0135 "connect-super-app expansion"** — there is no Bominal antecedent and no `oya-connect-translate-*` crate to relocate. It owns:

- **Real-time text translation** (sub-250 ms p95 for ≤ 500-char segments) routed across in-house oyatie-trained models, external MT vendors (DeepL, Google Cloud Translation, Microsoft Translator, Amazon Translate), and frontier LLMs (Anthropic, OpenAI, Gemini) via `foundry-providers`.
- **Translation Memory (TM)** with leverage-match scoring: exact (100 %), in-context exact (ICE), fuzzy (75–99 % via minhash-LSH per OmegaT), and TM-leverage decisions per ADR-TRANSLATE-0002.
- **Termbase + glossary** with per-tenant terminology constraints; TBX (ISO 30042) import/export.
- **MT-engine routing + fallback** per ADR-TRANSLATE-0001: route to best engine per (language-pair × content-class × quality-tier × residency-pack).
- **Human-in-the-loop review** (translator → reviewer → approver roles; `workflow-engine` orchestrated) with post-edit-distance metric capture.
- **Quality Estimation (QE)** — predict edit-distance without a reference (COMET-Kiwi-class); EU AI Act bounds enforced per ADR-TRANSLATE-0003.
- **Document translation** (DOCX, PPTX, XLSX, PDF, HTML, Markdown, PO, XLIFF, ARB, .strings, .resx, .properties, XLSX-spreadsheet) with format-preserving round-trip per ADR-TRANSLATE-0005 (Pandoc 3.x + LibreOffice 24.x in gVisor sandboxes).
- **Real-time caption translation streaming** (sentence-piece chunking + correction-replay per ADR-TRANSLATE-0006) for `meet`, `messenger`, and `shorts` integration.
- **Language-pack management** per tenant (which supported languages a tenant has paid for; per-tenant whitelist).
- **Data-residency-bound inference** per ADR-TRANSLATE-0004 — sovereign tenants (KR PIPA Art. 28, EU GDPR Art. 44–50, CN PIPL Art. 38–43, IN DPDPA §16) never have content crossing region boundaries during inference.
- **Bulk-translate API + webhook** for asynchronous large-file workloads (10 k-segment XLIFF p95 ≤ 60 s).

This µservice is **shared substrate** (every other oyatie product calls it for in-editor translate, body translation in mail/messenger/social, caption translation in meet, and content-localization in docs/sheets/slides) **and** a standalone product (tenants buy it directly as a Lokalise/Smartling/Crowdin replacement). Per ADR-0132 §"no new bundle/suite µservices", translate is single-concern and flat; it does NOT bundle TM + termbase + QE — those are its primary BCs.

## Tenant Value

- **Tenant Outcome 1 — One platform from MT to TMS.** Tenants get raw MT (Google/DeepL-class) plus the full TMS workflow (TM + termbase + human review) in one µservice, vs. assembling Google Translate + Crowdin + Trados Studio separately.
- **Tenant Outcome 2 — Residency-bound inference.** Sovereign tenants (Korea PIPA, EU GDPR, CN PIPL, India DPDPA) see content stay in-region during inference; no Google/Microsoft/OpenAI/DeepL cross-border calls unless the per-pack matrix permits it.
- **Tenant Outcome 3 — TM leverage compounds value.** Per-tenant project-scoped TM accumulates with use; 30 % leverage at month 3, 60 %+ at month 12 (LISA + LocWorld reference benchmarks); translation cost drops proportionally.
- **Tenant Outcome 4 — In-editor translate everywhere.** Workflow Studio, mail, messenger, social, docs/sheets/slides, meet captions all call this µservice via stable `TranslateInvoker` + `TmLeverageQuery` ports; no per-product translation code.
- **Tenant Outcome 5 — EU AI Act compliance.** Per-call disclosure record (per ADR-TRANSLATE-0003) covers Art. 50 transparency obligations; QE deployment classified low-risk by default with documented bounds.
- **Tenant Outcome 6 — Open file-format support.** XLIFF 2.1 (OASIS) + TMX 1.4 (LISA OSCAR) + TBX (ISO 30042) + JSON-i18n / ARB / .po / .strings / .resx / .properties round-trip with placeholder + variable + ICU MessageFormat + CLDR plural-rule preservation.
- **Internal Outcome 7 — Substrate uniformity.** Every workload µservice sees one stable port surface (`TranslateInvoker`, `TmLeverageQuery`, `TermbaseQuery`, `QualityEstimator`, `LanguageDetector`, `DocumentTranslator`) regardless of which engine responds underneath.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | mail composer | to translate a single segment (≤ 500 chars) with source-lang + target-lang + context | I see a translation in ≤ 250 ms p95 | translation-request | Must |
| FR-02 | translate API client | to batch-translate up to 100 segments in one call | I see batch result in ≤ 1.5 s p95 | batch-translate | Must |
| FR-03 | language-detection client | to detect language of arbitrary text ≤ 4 KB | I see ISO 639-3 + BCP 47 tag in ≤ 50 ms p99 | language-detection | Must |
| FR-04 | TM leverage query client | to query TM for fuzzy/exact/ICE matches per source-segment | best-leverage candidate returned in ≤ 80 ms p99 | translation-memory | Must |
| FR-05 | termbase query client | to constrain MT output to enforce tenant-defined terminology | output respects glossary | termbase-and-glossary | Must |
| FR-06 | translation requester | to receive engine-routed translation respecting capability + cost + residency | MT engine selected per ADR-TRANSLATE-0001 | mt-engine-selection | Must |
| FR-07 | workflow orchestrator | to assign segments to translator/reviewer/approver roles | human-in-the-loop drives quality | human-in-the-loop-review | Must |
| FR-08 | quality-estimation client | to receive QE score (0–100) without reference | I can decide whether to flag for human review | quality-estimation | Must |
| FR-09 | QA tool | to back-translate and compare with source | I detect catastrophic translation failure | back-translation-qa | Should |
| FR-10 | content localizer | to extract translatable strings from a Markdown/HTML/PO/JSON-i18n/ARB/.strings/.resx/.properties file | I can translate them and re-merge | content-localization | Must |
| FR-11 | document translator | to translate a DOCX/PPTX/XLSX/PDF file end-to-end with format preserved | round-trip is fidelity-tier-bounded per ADR-TRANSLATE-0005 | document-translation | Must |
| FR-12 | meet caption stream | to translate audio-derived captions in real time with correction-replay | participants see translated captions ≤ 400 ms p99 from STT output | real-time-translation-stream | Must |
| FR-13 | i18n developer | to import XLIFF 2.1 / TMX 1.4 / TBX | TMS workflow honours OASIS/LISA/ISO standards | file-import | Must |
| FR-14 | i18n developer | to export to the same set | round-trip is lossless within fidelity bounds | file-export | Must |
| FR-15 | i18n developer | to translate ICU MessageFormat / Mustache / named-%s placeholders without breaking them | localized output remains a valid format string | placeholder-preservation | Must |
| FR-16 | localization manager | to localize plural forms per CLDR plural rules for target locale | output respects CLDR (one/few/many/other) | pluralization | Must |
| FR-17 | localization manager | to set per-locale formality + gender policy | output reflects per-tenant gender/formality contract | formality-and-gender | Must |
| FR-18 | bulk-translate client | to submit a 10k-segment XLIFF file and poll job state | result available in ≤ 60 s p95 | bulk-translate | Must |
| FR-19 | sovereign tenant admin | to assert "no cross-region inference" | engine routing never selects a non-resident endpoint | data-residency-bound-inference | Must |
| FR-20 | EU tenant operator | to receive Art. 50 disclosure on every QE/MT call | audit + regulatory posture | eu-ai-act-disclosure | Must |
| FR-21 | language-pack admin | to enable/disable language pairs per tenant | per-tenant pricing tiers reflected | language-pack-management | Must |
| FR-22 | TBX admin | to import per-tenant TBX termbase | local terminology enforced in MT output | termbase-import | Must |
| FR-23 | translate webhook subscriber | to receive `TranslationCompleted` / `BulkJobCompleted` events | async workflows can react | translate-events | Must |
| FR-24 | translation requester | to specify content-class (`marketing`, `legal`, `medical`, `code-comment`, `ui-string`, etc.) | engine selection routes per content-class | mt-engine-selection | Should |

## Non-Functional Requirements

### Performance (HARD budgets; LEAN-A8 enforced)

| Metric | p50 | p95 | p99 | p999 | Notes |
|---|---|---|---|---|---|
| Translation request (≤ 500 chars) | ≤ 90 ms | ≤ 250 ms | ≤ 400 ms | ≤ 800 ms | Excludes upstream vendor RTT when in-house; includes vendor RTT when external |
| Batch translate (100 segments) | ≤ 600 ms | ≤ 1.5 s | ≤ 2.5 s | ≤ 4 s | Concurrent fan-out to upstream + TM leverage |
| Language detection (≤ 4 KB) | ≤ 12 ms | ≤ 28 ms | ≤ 50 ms | ≤ 90 ms | FastText-class model + LangID heuristic |
| TM leverage match | ≤ 25 ms | ≤ 50 ms | ≤ 80 ms | ≤ 140 ms | Meilisearch + minhash-LSH per ADR-TRANSLATE-0002 |
| QE score (segment) | ≤ 70 ms | ≤ 140 ms | ≤ 200 ms | ≤ 350 ms | COMET-Kiwi-class model on in-house endpoint |
| Real-time caption streaming chunk | ≤ 180 ms | ≤ 300 ms | ≤ 400 ms | ≤ 600 ms | Per chunk; sentence-piece chunking; correction-replay window per ADR-TRANSLATE-0006 |
| Document translate (10-page DOCX) | ≤ 4 s | ≤ 8 s | ≤ 14 s | ≤ 22 s | Pandoc 3.x extract + per-segment MT + Pandoc re-merge |
| Bulk translate (10 k-segment XLIFF) | ≤ 30 s | ≤ 60 s | ≤ 95 s | ≤ 150 s | Bulk-job worker fan-out |
| Translate-router decision latency | ≤ 1 ms | ≤ 3 ms | ≤ 5 ms | ≤ 10 ms | In-process; no upstream RTT |
| Data-residency-correctness | 100.0000 % | 100 % | 100 % | 100 % | Per ADR-TRANSLATE-0004; HARD; CI-validated; any breach is Sev-1 |
| MT engine availability (rolling 30d) | — | — | ≥ 99.95 % | — | Burn-rate alerts at 2 % / 5 % / 14d budget |

### Security

- Every per-vendor credential is referenced as `openbao://<pack>/<tenant>/translate/providers/<vendor>/<credential-name>` and resolved inside the adapter sandbox via `cloud-secrets` µservice. Conformance verified by `oya-translate-credential-isolation` LEAN lane (analogous to foundry-providers').
- Per-tenant Cedar policy gates which engines a tenant may use (e.g., `pack-kr` forbids DeepL US-region without explicit cross-border consent; `pack-cn-stub` permits no external engine — in-house only).
- mTLS via Istio (SPIFFE identity per `cell` µservice) between `translate-router` ↔ adapters ↔ upstream proxy fleet.
- Per-call BLAKE3 content hashing + Ed25519 envelope signing for every `TranslationCompleted` and `BulkJobCompleted` event emitted to `foundry-evidence`.
- Document-translation sandbox runs Pandoc + LibreOffice in **gVisor** with seccomp profile + no-network + read-only-rootfs. The sandbox is hardened against malicious DOCX/PDF/PPTX exploitation per OWASP File Upload Cheat Sheet + CVE-curated LibreOffice security advisories.
- Cedar v4.2 default-deny on `policy/translate-tenant-scope.cedar`.

### Audit + Compliance

- Every translation request emits `TranslationCompleted` to audit-chain (`oya.translate.translation.completed`) with `(tenant_id, principal, source_lang, target_lang, content_class, engine, model_id, jurisdiction_code, segment_hash, translation_hash, qe_score, cost_usd, latency_ms, evidence_ref)`.
- Every TM update emits `TmUpdated` (`oya.translate.tm.updated`) with `(tenant_id, project, segment_hash, new_target_hash, leverage_match)`.
- Every termbase change emits `TermbaseUpdated`.
- Every bulk job emits `BulkJobStarted` + `BulkJobCompleted` + `BulkJobFailed`.
- Every QE call emits `QualityEstimated`.
- EU AI Act Art. 50 (transparency) + Art. 13 (transparency to deployers) disclosure record emitted per call when `jurisdiction = EU`.
- Engine-selection decision emitted as `EngineRouted` for explainability.

### Availability + SLO

- Availability target: 99.95 % monthly for `/translate` and `/detect-language`; failover from one engine to next-best per ADR-TRANSLATE-0001.
- 99.99 % monthly for `data-residency-correctness` (effective: zero cross-region inference events monthly).
- RTO: ≤ 10 min for `translate-router` (stateless restart). RPO ≤ 5 min for TM/termbase (Postgres WAL + 5-min snapshot).
- Per-pack DR pair (Mimir/Loki via observability) per ADR-0117.

### Data residency (ADR-TRANSLATE-0004)

- Per-pack engine whitelist (initial M01):
  - `pack-kr`: in-house (KR-region), Anthropic (KR-region via SCC + ZDR), Google Cloud Translation (KR-region via SCC), DeepL (DE-EU; cross-border permitted only when tenant has PIPA Art. 28 consent on file).
  - `pack-eu`: in-house (EU-region), Anthropic (EU-region), OpenAI (EU-region post-SCC), Google (EU-region), DeepL (DE-EU; native).
  - `pack-us`: any.
  - `pack-us-healthcare`: in-house (HIPAA region) + Anthropic (BAA + ZDR); other vendors per-tenant BAA.
  - `pack-jp`: in-house (JP-region) + Anthropic (JP-region) + DeepL (JP-region) + Google (JP-region).
  - `pack-cn-stub`: in-house ONLY (CN-region); all external vendors forbidden per PIPL Art. 38–43.
  - `pack-sg` / `pack-au` / `pack-in` / `pack-br` / `pack-ae` / `pack-ksa`: per-pack matrix in `policy/data-residency.md`.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106, the µservice exposes layers `kernel`, `domain`, `usecase`, `api`, `adapter` (backend-qualified per vendor + per store), `rest`, `worker`, `sdk`, `app`. Per ADR-0131, all crates live under `microservices/translate/src/crates/`.

| BC | Crate family | Purpose | Key entities |
|---|---|---|---|
| `translate-router` | `oya-translate-router-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}` | Capability-routed engine selection; cost/latency/residency-aware; emits `EngineRouted` events | `TranslationRequest`, `RoutingDecision`, `EngineCandidate`, `ContentClass`, `ResidencyConstraint` |
| `translation-memory` | `oya-translate-tm-{kernel,domain,usecase,api,adapter-postgres,adapter-meilisearch,rest,worker,sdk,app}` | Per-tenant TM; fuzzy/exact/ICE matching; minhash-LSH per ADR-TRANSLATE-0002 | `TmUnit`, `LeverageMatch`, `Segment`, `Project` |
| `termbase-and-glossary` | `oya-translate-termbase-{kernel,domain,usecase,api,adapter-postgres,rest,worker,sdk,app}` | TBX import/export; per-tenant terminology enforcement | `Term`, `TermConcept`, `Translation`, `Termbase` |
| `quality-estimation` | `oya-translate-qe-{kernel,domain,usecase,api,adapter-foundry-runtime,rest,worker,sdk,app}` | COMET-Kiwi-class QE on in-house endpoint; EU AI Act bounds | `QualityScore`, `QeContext`, `QeBound` |
| `language-detection` | `oya-translate-langdetect-{kernel,domain,usecase,api,adapter-foundry-runtime,rest,worker,sdk,app}` | FastText/LangID-class detection | `Language`, `Confidence`, `DetectionResult` |
| `document-translation` | `oya-translate-doc-{kernel,domain,usecase,api,adapter-pandoc,adapter-libreoffice,rest,worker,sdk,app}` | DOCX/PPTX/XLSX/PDF/HTML/Markdown round-trip per ADR-TRANSLATE-0005 | `Document`, `FormatBlock`, `Segment`, `MergePlan` |
| `bulk-translate` | `oya-translate-bulk-{kernel,domain,usecase,api,adapter-postgres,adapter-s3,rest,worker,sdk,app}` | Async bulk jobs; XLIFF/TMX/TBX I/O | `BulkJob`, `JobChunk`, `JobState` |
| `real-time-stream` | `oya-translate-stream-{kernel,domain,usecase,api,adapter-foundry-runtime,rest,worker,sdk,app}` | Sentence-piece streaming + correction-replay per ADR-TRANSLATE-0006 | `StreamSession`, `Chunk`, `Correction` |
| `engine-adapters` | per-vendor: `oya-translate-adapter-anthropic`, `oya-translate-adapter-openai`, `oya-translate-adapter-google-translate`, `oya-translate-adapter-deepl`, `oya-translate-adapter-foundry-runtime` | Transports to engines (in-house + external) | `EngineRequest`, `EngineResponse` |
| `placeholder-and-plural` | sub-module of router-domain | ICU MessageFormat + CLDR plural-rule preservation | `Placeholder`, `PluralRule`, `Formality` |

Naming justifications:

```
NAME: oya-translate-router-<layer>
JUSTIFICATION:
- microservice = translate (microservices/translate/)
- bc-tokens = router (primary BC; capability-routed engine selection)
- layer ∈ {kernel, domain, usecase, api, adapter, rest, worker, sdk, app} per ADR-0105
- exemptions claimed: none
```

```
NAME: oya-translate-adapter-<backend>
JUSTIFICATION:
- microservice = translate
- bc-tokens = adapter-<backend> (postgres | redis | s3 | meilisearch | foundry-runtime | anthropic | openai | google-translate | deepl | pandoc | libreoffice)
- layer = adapter (ADR-0105 13-value enum; backend-qualified per Amendment 3)
- exemptions claimed: none
```

## Cross-µservice Integration

| Producer / consumer | Edge | Contract |
|---|---|---|
| Workflow Studio / mail / messenger / social / docs / sheets / slides / shorts | invokes `TranslateInvoker` | `contracts/proto/translate.proto` + `contracts/openapi/translate.yaml` |
| `meet` | calls `RealTimeStreamInvoker` for caption translate | `contracts/proto/translate-stream.proto` |
| `translate` → `foundry-providers` | external MT engine invocation (Anthropic / OpenAI / Gemini) | foundry-providers `ProviderInvoker` |
| `translate` → `foundry-runtime` | in-house translation/QE/langdetect model inference | foundry-runtime capability-executor |
| `translate` → `cloud-secrets` (OpenBao) | resolves SecretReference for external vendor keys | OpenBao agent socket |
| `translate` → `foundry-evidence` | emits `TranslationCompleted`, `EngineRouted`, `TmUpdated`, etc. | `contracts/asyncapi/translate-events.yaml` |
| `translate` → `observability` | emits SLI metrics | OTel + Prometheus |
| `translate` → `tenancy` | per-tenant policy + residency-pack lookup | tenancy RLS |
| `translate` → `audit-chain` | every translation seal | audit-chain ingest |
| `translate` → `ontology` | binds `Language` / `Project` / `Segment` / `TmUnit` entities | ontology entity store |
| `translate` → `workflow-engine` | human-in-the-loop step orchestration | workflow-engine API |

## Substrate

| Concern | Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|---|
| TM + termbase + project metadata | **PostgreSQL 16** (HA primary+replica per pack) | `oya-translate-tm-adapter-postgres`, `oya-translate-termbase-adapter-postgres` |
| TM leverage search + termbase lookup | **Meilisearch 0.10.0** (per pack) | `oya-translate-tm-adapter-meilisearch` |
| Session + bulk-job state cache | **Redis 7.2** (per pack; sentinel HA) | `oya-translate-bulk-adapter-redis` |
| Document storage + bulk-job artifacts | **S3-compatible (OCI Object Storage)** per pack | `oya-translate-doc-adapter-s3`, `oya-translate-bulk-adapter-s3` |
| In-house MT + QE + LangDetect inference | `foundry-runtime` (vLLM / TGI in-house endpoint) | `oya-translate-adapter-foundry-runtime` |
| External MT vendors (Anthropic / OpenAI / Google / DeepL) | `foundry-providers` adapter substrate | `oya-translate-adapter-{anthropic, openai, google-translate, deepl}` |
| Document format round-trip | **Pandoc 3.x** + **LibreOffice 24.x** in **gVisor** | `oya-translate-doc-adapter-pandoc`, `oya-translate-doc-adapter-libreoffice` |
| Audio source for caption translate | `meet` µservice → Whisper STT (cross-µservice) | `oya-translate-stream-adapter-foundry-runtime` |
| Credentials | OpenBao (`cloud-secrets`) | `oya-translate-adapter-openbao` (via foundry-providers) |
| Telemetry | Mimir + Alertmanager (via `observability`) | recording rules in `iac/helm/translate-router/values.yaml` |
| Evidence | `foundry-evidence` | event emission only here |
| Service mesh | Istio (per `cell`) | mTLS + SPIFFE |

LTS pins per ADR-0133:
- Postgres 16 (LTS through 2028-11).
- Redis 7.2 (LTS through 2027-Q4).
- Meilisearch 0.10.0 LTS line.
- Pandoc 3.x (stable; tracked).
- LibreOffice 24.x community LTS.
- Cedar v4.2 (current major).
- Whisper (large-v3; tracked via foundry-runtime).

## Competitive Benchmark

Per ADR-0133 §"industry-best-practice conformance program" and `competitor-parity-matrix.md`:

| Competitor | Surface | Differentiator |
|---|---|---|
| Google Translate / Cloud Translation API + AutoML | Hosted MT + custom-model training | Wide language coverage; cloud-only; no built-in TMS |
| DeepL + DeepL Pro | Hosted MT (premium quality on EU pairs) | EU-grounded; high quality but no TMS or termbase |
| Microsoft Translator + Custom Translator | Hosted MT + custom-model | Azure-tied; no integrated TMS |
| Amazon Translate | Hosted MT | AWS-tied; minimal customization |
| Lokalise | Hosted TMS | TMS strong; MT via vendor pass-through |
| Smartling | Hosted TMS + LSP marketplace | TMS + agency marketplace; closed |
| Crowdin | Hosted TMS | TMS strong; multi-engine MT |
| Phrase (ex-Memsource) | Hosted TMS (formerly Memsource) | Mature TMS + leverage |
| MateCat | Open-source TMS | TM + MT but no platform stewardship |
| Trados Studio | Desktop CAT tool | Industry-standard desktop; no API; no platform |
| OmegaT | Open-source desktop CAT | Open-source TM + leverage algorithm (cited in ADR-TRANSLATE-0002) |
| Apertium | Open-source rule-based MT | Rule-based; specific low-resource pairs |
| MarianNMT | Research NMT framework | Research-grade; self-host |
| Argos Translate | Open-source self-host MT | Self-host OPUS-MT models |
| Apple Translate | OS-bundled MT | iOS-tied; no API |
| Yandex Translate | Hosted MT | RU-centric |

oyatie differentiators:
- **Per-tenant per-pack residency enforcement at engine selection** (no competitor enforces residency as first-class router constraint with default-deny per pack).
- **In-house frontier-LLM-grade translation through foundry-runtime** with blue/green vs. external engines per ADR-0026.
- **One stable port surface** (`TranslateInvoker`) consumed by 9 sibling oyatie products (mail, messenger, social, docs, sheets, slides, meet, shorts, workflow-studio).
- **EU AI Act Art. 50 / Art. 13 disclosure** baked into every call (no competitor emits this by default).
- **Cedar default-deny + OpenBao credential isolation + audit-chain Ed25519 envelope** on every MT call (no competitor provides this posture).
- **gVisor-isolated document-translation sandbox** (no competitor isolates document parsers at this granularity).
- **Local-pack overlays** (KR, EU, JP, CN-stub) bake regional legal posture into deployment, not just policy.

## Open Questions

1. **In-house MT parity bar (when does router prefer in-house over external).** Default rule: prefer in-house if `(BLEU-on-eval-set ≥ 0.95× of incumbent) AND (cost ≤ 0.5× of incumbent) AND (p99 ≤ 1.2× of incumbent)`. Quarterly council-architecture review; tracked separately under ADR-0026 + ADR-TRANSLATE-0001.
2. **Human-translator marketplace integration (Tier-G).** Future; integrates with Smartling/Lokalise-style LSP marketplaces. Not in M01 scope; tracked under Phase 3.
3. **TMX 2 (proposed).** TMX 1.4 ships M01; TMX 2 (proposed by GALA TAPICC) tracked as it stabilizes; placeholder in import/export.
4. **CN-pack production activation.** `pack-cn-stub` ships with overlay scaffolding in M01 (per ADR-0135), but production activation requires KR-PIPA-equivalent CN regulatory mapping + China-resident-key KMS + CN-resident vLLM cluster; tracked as M03-onward.
5. **Apple Translate parity for iOS / macOS shorts widgets.** Apple Translate is OS-bundled; we do not call it; we cover the same UX surface via in-house MT through `shorts`. No work in M01.
6. **Yandex Translate.** Not in M01 vendor list per geopolitical risk; tracked separately for tenant demand.
7. **Domain-adapted custom models (per-tenant fine-tune).** Tracked under ADR-0026 Phase 4; not in M01 scope.

## Acceptance Criteria (PRD-level)

- **AC-01** — `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice translate` exits 0.
- **AC-02** — `cargo run -p oya-dev-cli -- gate validate authority-cohesion` exits 0.
- **AC-03** — `oya-translate-credential-isolation` LEAN lane present in `.github/branch-protection.yaml` `required_status_checks` for `dev` + `staging`.
- **AC-04** — `oya-translate-data-residency-correctness` LEAN lane present in `.github/branch-protection.yaml` (BLOCKER) on all branches.
- **AC-05** — Every adapter crate's tests verify a zero-occurrence regex sweep for vendor credential bytes against the test fixture set.
- **AC-06** — Translate-router decision p99 (in-process, no upstream) ≤ 5 ms verified by `tests/load/router_decision.rs`.
- **AC-07** — Translation-request p95 (in-house pair) ≤ 250 ms verified by `tests/load/translation_request.rs`.
- **AC-08** — Real-time caption stream p99 ≤ 400 ms per chunk verified by `tests/load/caption_stream.rs`.
- **AC-09** — Document translate 10-page DOCX p95 ≤ 8 s verified by `tests/load/document_translate.rs`.
- **AC-10** — Bulk-translate 10 k-segment XLIFF p95 ≤ 60 s verified by `tests/load/bulk_translate.rs`.
- **AC-11** — Per-pack (residency × engine) matrix in `policy/data-residency.md` matches PRD §"Data residency" launch list.
- **AC-12** — EU AI Act disclosure record emitted per QE / MT call when `jurisdiction = EU` verified by `tests/integration/eu_ai_act_disclosure.rs`.

## References

- ADR-0056 — Rust clean-architecture BNF v4.1.
- ADR-0105 — 13-layer enum + check-family patterns.
- ADR-0106 — `application` → `usecase` rename for new crates.
- ADR-0117 — pack residency model.
- ADR-0135 — connect super-app expansion (parent ADR; translate is per ADR-0135 §"shared substrate µservices").
- ADR-0139 — agentic SLO-gated promotion.
- ADR-0131 — per-microservice flat layout.
- ADR-0132 — product suite and bundle dissolution.
- ADR-0133 — industry-best-practice conformance program.
- ADR-TRANSLATE-0001 — MT engine routing and fallback.
- ADR-TRANSLATE-0002 — Translation memory and leverage model.
- ADR-TRANSLATE-0003 — Quality estimation and EU AI Act bounds.
- ADR-TRANSLATE-0004 — Data-residency-bound inference.
- ADR-TRANSLATE-0005 — Document round-trip fidelity.
- ADR-TRANSLATE-0006 — Real-time translation stream architecture.
- `docs/standards/observability-slo.md`.
- OASIS XLIFF 2.1 — `docs.oasis-open.org/xliff/xliff-core/v2.1/`.
- LISA OSCAR TMX 1.4 — `www.gala-global.org/tmx-14b`.
- ISO 30042:2019 (TBX).
- ICU MessageFormat — `unicode-org.github.io/icu/userguide/format_parse/messages/`.
- CLDR plural rules — `cldr.unicode.org/index/cldr-spec/plural-rules`.
- RFC 5646 BCP 47 language tags.
- ISO 639-3 + ISO 639-5.
- WMT shared-task benchmarks — `statmt.org/wmt24/`.
- COMET — `unbabel.github.io/COMET/`.
- OmegaT TM leverage docs — `omegat.org/`.
- Pandoc 3.x docs — `pandoc.org/`.
- LibreOffice 24.x security advisories — `www.libreoffice.org/about-us/security/advisories/`.
- gVisor — `gvisor.dev/`.
- EU AI Act (Reg. (EU) 2024/1689) Arts. 13 + 50.
- GDPR Arts. 44–50.
- KR PIPA Art. 17 + Art. 22-2 + Art. 23 + Art. 28; PIPC Notice 2020-7.
- HIPAA 45 CFR §164.312.
- APPI Art. 24.
- PDPA SG cross-border transfer guidance.
- AU Privacy Act APP 8.
- DPDPA 2023 §16.
- LGPD Art. 33.
- UAE PDPL + KSA PDPL.
- CN Cybersecurity Law + DSL + PIPL Arts. 38–43.
- ITU-T G.107 (E-model for audio captions).
- WCAG 2.2 AA accessibility.
