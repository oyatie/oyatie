---
doc_class: CompetitorParityMatrix
title: Competitor parity matrix
microservice: translate
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-translate + gtm-product
related_adrs: [ADR-0126, ADR-0131, ADR-0133, ADR-TRANSLATE-0001, ADR-TRANSLATE-0002, ADR-TRANSLATE-0005, ADR-TRANSLATE-0006]
related_artifacts:
  - microservices/translate/PRD.md
review_cadence: quarterly + on every major competitor product release
doc_status: published
---

# Competitor Parity Matrix — translate µservice

## Selected Competitors

Per `PRD.md` §"Competitive Benchmark" + LISA/LocWorld + Slator industry reports:

| Competitor | Category | Coverage in oyatie |
|---|---|---|
| Google Cloud Translation API + AutoML | MT API + custom MT | Adapter (foundry-providers) |
| DeepL + DeepL Pro | MT API (premium EU) | Adapter (foundry-providers) |
| Microsoft Translator + Custom Translator | MT API + custom MT | Adapter (tracked; M02) |
| Amazon Translate | MT API | Adapter (tracked; M02) |
| Lokalise | Hosted TMS | Surface parity (TMS + bulk-translate + project mgmt) |
| Smartling | Hosted TMS + LSP marketplace | Surface parity (TMS + termbase + QA + workflow); Tier-G marketplace future |
| Crowdin | Hosted TMS | Surface parity (TMS + workflow + integrations) |
| Phrase (ex-Memsource) | Hosted TMS (mature) | Surface parity (TMS + leverage + QA) |
| MateCat | Open-source TMS | Surface parity (TMS + leverage) |
| Trados Studio | Desktop CAT | Out of scope (desktop; we are API + web) |
| OmegaT | Open-source desktop CAT | TM leverage algorithm cited in ADR-TRANSLATE-0002 |
| Apertium | Open-source rule-based MT | Tracked for low-resource pairs (M03+) |
| MarianNMT | Research NMT framework | Underlies in-house models (via foundry-runtime + ADR-0026) |
| Argos Translate | Open-source self-host MT | Open-source path for cost-sensitive tenants (M03+) |
| Apple Translate | iOS-bundled | N/A (OS; we cover via shorts) |
| Yandex Translate | Hosted MT (RU) | Not in M01 (geopolitical risk) |

## Feature Parity (M01 baseline)

| Feature | Google | DeepL | Microsoft | Amazon | Lokalise | Smartling | Crowdin | Phrase | MateCat | **oyatie translate** |
|---|---|---|---|---|---|---|---|---|---|---|
| Raw MT API | ✅ | ✅ | ✅ | ✅ | ⚠ (via vendor pass) | ⚠ (via vendor pass) | ⚠ (via vendor pass) | ⚠ (via vendor pass) | ⚠ (via vendor pass) | ✅ |
| Custom MT model training | ✅ (AutoML) | ❌ | ✅ (Custom) | ❌ | ❌ | ⚠ | ❌ | ❌ | ❌ | ✅ (foundry-runtime + ADR-0026) |
| Translation Memory | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Termbase + glossary | ❌ | ⚠ (limited) | ⚠ (per-term) | ⚠ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ (TBX) |
| Quality Estimation | ❌ | ⚠ (confidence) | ⚠ (confidence) | ⚠ | ⚠ | ✅ | ⚠ | ✅ | ⚠ | ✅ (COMET-Kiwi-class; ADR-TRANSLATE-0003) |
| Document round-trip (DOCX/PPTX/XLSX/PDF) | ⚠ (limited) | ✅ (Pro doc) | ✅ | ⚠ | ⚠ | ✅ | ✅ | ✅ | ⚠ | ✅ (gVisor sandboxed) |
| Real-time caption streaming | ⚠ (Cloud Speech-to-Text-Translation only) | ❌ | ✅ (Translator speech API) | ⚠ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ (ADR-TRANSLATE-0006) |
| Human-in-loop workflow | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ (workflow-engine) |
| XLIFF 2.1 import/export | ❌ | ❌ | ⚠ | ⚠ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| TMX 1.4 import/export | ❌ | ❌ | ⚠ | ⚠ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| TBX import/export | ❌ | ❌ | ⚠ | ⚠ | ⚠ | ✅ | ⚠ | ✅ | ⚠ | ✅ |
| ICU MessageFormat preservation | ⚠ | ⚠ | ⚠ | ⚠ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| CLDR plural rules | ⚠ | ⚠ | ⚠ | ⚠ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| BCP 47 + ISO 639-3 + ISO 639-5 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Per-pack residency (default-deny cross-border) | ⚠ (regional endpoints; not router-enforced) | ⚠ (EU/US) | ⚠ (regional) | ⚠ (regional) | ❌ | ⚠ (regions) | ⚠ (regions) | ⚠ (regions) | ❌ | ✅ (HARD; ADR-TRANSLATE-0004) |
| EU AI Act Art. 50 disclosure | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ (per call; ADR-TRANSLATE-0003) |
| HIPAA BAA support | ⚠ (Healthcare API) | ❌ | ✅ | ✅ | ❌ | ⚠ | ❌ | ❌ | ❌ | ✅ (pack-us-healthcare) |
| KR PIPA / IN DPDPA / BR LGPD / AE PDPL residency | ⚠ | ❌ | ⚠ | ⚠ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ (per-pack overlays) |
| Open-source self-host option | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ (MateCat OS) | ⚠ (in-house adapters; Argos future) |
| Stable port (`TranslateInvoker`) consumed by 9 sibling products | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | ✅ |
| Per-call Ed25519 audit envelope | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| OpenBao-bridged credential isolation | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| gVisor sandboxed document parsers | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| Capability-routed engine selection (cost × residency × quality × health) | ❌ | ❌ | ❌ | ❌ | ⚠ (multi-engine UI) | ⚠ | ✅ (multi-engine) | ⚠ | ⚠ | ✅ (HARD; ADR-TRANSLATE-0001) |

Legend: ✅ first-class · ⚠ partial / limited · ❌ absent · n/a not applicable

## Quality Benchmarks (WMT + COMET reference)

Targets (in-house parity bar per ADR-TRANSLATE-0001):

| Language pair | BLEU target vs DeepL | chrF target vs DeepL | COMET target vs DeepL |
|---|---|---|---|
| EN→KO | ≥ 0.95× | ≥ 0.95× | ≥ 0.95× |
| KO→EN | ≥ 0.95× | ≥ 0.95× | ≥ 0.95× |
| EN→JA | ≥ 0.92× | ≥ 0.92× | ≥ 0.92× |
| JA→EN | ≥ 0.92× | ≥ 0.92× | ≥ 0.92× |
| EN→DE | ≥ 0.90× (DeepL strong) | ≥ 0.90× | ≥ 0.90× |
| DE→EN | ≥ 0.90× | ≥ 0.90× | ≥ 0.90× |
| EN→ZH | ≥ 0.92× | ≥ 0.92× | ≥ 0.92× |
| ZH→EN | ≥ 0.92× | ≥ 0.92× | ≥ 0.92× |
| EN→AR | ≥ 0.88× (lower-resource pair) | ≥ 0.88× | ≥ 0.88× |
| EN→HI | ≥ 0.88× | ≥ 0.88× | ≥ 0.88× |

Reference dataset: WMT-22 + WMT-23 + WMT-24 test sets + per-pack in-tenant eval sets (de-identified per tenant DPA).

## Strategic Differentiators

1. **Substrate-and-product**: oyatie translate is the substrate every sibling product uses (mail/messenger/social/docs/sheets/slides/meet/shorts/workflow-studio) AND a standalone TMS product tenants buy.
2. **Per-pack residency as a router-decision invariant** (no other vendor enforces this default-deny at the router).
3. **EU AI Act Art. 50 disclosure baked in** (no vendor emits this by default).
4. **OpenBao-bridged credential isolation + per-call Ed25519 audit envelope** (no peer provides this posture).
5. **gVisor-sandboxed document parsers** (no peer isolates LibreOffice/Pandoc at this granularity).
6. **In-house frontier-LLM-grade MT** through foundry-runtime when parity bar met (ADR-0026).
7. **Open-format-first** (XLIFF 2.1 + TMX 1.4 + TBX + ICU + CLDR all M01).

## Quarterly Refresh Process

- Pull current per-vendor feature matrix from product pages.
- Re-run WMT eval against latest in-house model + each external vendor.
- Update parity table.
- Council-architecture review for any "newly lost ✅" cell.

## Verification

- `tests/quality/wmt-eval/` directory contains WMT-* test sets + scoring scripts.
- Per-release COMET score regression caught by `oya-translate-quality-regression` LEAN lane (when implemented per ADR-TRANSLATE-0003).

## References

- ADR-TRANSLATE-0001 (engine routing + parity bar).
- ADR-TRANSLATE-0002 (TM leverage; OmegaT reference).
- ADR-TRANSLATE-0006 (real-time stream).
- ADR-0026 (in-house AI substrate roadmap).
- WMT-22 / WMT-23 / WMT-24 — `statmt.org/wmt24/`.
- COMET — `unbabel.github.io/COMET/`.
- BLEU + chrF references — original papers cited in ADR-TRANSLATE-0001.
- Slator + LISA + LocWorld industry reports.
- Per-vendor product pages (live).
