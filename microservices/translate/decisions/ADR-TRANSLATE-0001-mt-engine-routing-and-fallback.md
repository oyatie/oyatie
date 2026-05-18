---
doc_class: AdrSpec
template_id: TPL-ADR
adr_id: ADR-TRANSLATE-0001
title: MT engine routing and fallback
status: Accepted
deciders: council-architecture, axis-translate, ops-security, council-privacy, gtm-product
date: 2026-05-17
microservice: translate
supersedes: []
superseded_by: []
related_adrs: [ADR-0025, ADR-0026, ADR-0135, ADR-0131, ADR-0133, ADR-TRANSLATE-0003, ADR-TRANSLATE-0004]
related_artifacts:
  - microservices/translate/PRD.md
  - microservices/translate/competitor-parity-matrix.md
  - microservices/translate/IP-003-translate-router-domain.md
doc_status: published
---

# ADR-TRANSLATE-0001 — MT engine routing and fallback

## Context

The `translate` µservice must route per-call to the best machine-translation engine across:

- **In-house** (foundry-runtime served, oyatie-trained models — the margin driver per ADR-0026).
- **External LLM-class** (Anthropic Claude, OpenAI GPT, via foundry-providers) — frontier capability for content classes requiring long-context, contextual nuance, or legal/medical accuracy.
- **External NMT-class** (Google Cloud Translation API + AutoML, DeepL Pro, future Microsoft Translator + Amazon Translate) — high-volume short segments; DeepL particularly strong on EU pairs.

The routing decision must satisfy hyperscaler-grade bars across four orthogonal axes:

1. **Residency** (HARD; default-deny per ADR-TRANSLATE-0004).
2. **Capability** (language-pair × content-class × quality-tier supported by the engine).
3. **Cost** (per-tenant cost ceiling + per-segment cost-per-1K-chars).
4. **Latency + availability** (rolling-window health from observability + foundry-providers).

Industry references:

- WMT shared-task benchmarks (statmt.org/wmt24/) for translation quality baselines (BLEU + chrF + COMET).
- Slator + LISA + LocWorld industry reports for vendor capability matrices.
- DeepL published quality benchmarks vs Google + Microsoft (DeepL whitepapers).
- Smartling / Crowdin / Phrase multi-engine router patterns (closed-source; surface observation).
- LiteLLM open-source provider router (foundry-providers parallel; cited in foundry-providers PRD).

Bominal context: there is **no Bominal antecedent** for translate per ADR-0135; this ADR is net-new.

## Decision

The `oya-translate-router-domain` crate implements a **weighted-score capability-routed engine selector** that operates on this algebra (see also `IP-003-translate-router-domain.md`):

```
score(candidate, request) = w_fit  · capability_fit_score(candidate, request)
                          - w_cost · normalize(cost_per_1k_chars_usd)
                          - w_lat  · normalize(p99_latency_ms)
                          + w_avail · availability_rolling_15m
                          + w_pref · (prefer_in_house && candidate.vendor == InHouse)
```

Subject to **hard filters** (in order):

1. **Residency filter** — `policy.residency.permitted_vendors.contains(candidate.vendor) && policy.residency.permitted_regions.contains(candidate.region)`; if no candidate passes, return `RouterError::NoResidencyCompliantEngine`. Pre-residency-routing pinned via SLI; any cross-region inference is Sev-1.
2. **Capability filter** — `language_pair_supported && quality_tier_supported && content_class_supported`.
3. **Health filter** — engine availability ≥ tenant minimum (default 99 % rolling 15 m); demoted engines excluded.

Weights vary by **quality tier**:

| Quality tier | w_fit | w_cost | w_lat | w_avail | w_pref |
|---|---|---|---|---|---|
| Draft | 0.4 | 0.3 | 0.1 | 0.1 | 0.1 |
| Standard (default) | 0.4 | 0.2 | 0.2 | 0.1 | 0.1 |
| Premium | 0.6 | 0.1 | 0.1 | 0.1 | 0.1 |
| eIDAS (signed-translation tier) | 0.5 | 0.1 | 0.1 | 0.2 | 0.1 |

**In-house parity bar**: the router prefers in-house only when `(BLEU-on-eval-set ≥ 0.95× incumbent) AND (cost ≤ 0.5× incumbent) AND (p99 ≤ 1.2× incumbent)` per quarterly council-architecture review.

**Fallback chain** on engine failure (5xx, 429 rate-limit, timeout, response-shape-anomaly): the router retries the next-best candidate up to 2 alternates. If all candidates exhausted, return upstream error to the caller with `evidence_ref` preserved.

**Per-tenant adapter pin**: a tenant may pin to a specific (vendor, model-id, adapter-version) tuple to prevent silent vendor model swap (per `runbooks/mt-engine-degraded-shed.md`).

## Alternatives Considered

### Alternative A — Single-engine (in-house only)

- **Pros**: simplest; max margin; max residency posture.
- **Cons**: in-house parity bar not met M01 for all pairs (EN↔DE DeepL premium gap; low-resource pairs); tenant choice constrained; vendor differentiation lost.
- **Verdict**: rejected. In-house is the preferred path when parity met; routing layer enables compounding the parity bar via observability.

### Alternative B — Hard-coded per-tenant engine

- **Pros**: predictable cost; predictable behavior; trivial implementation.
- **Cons**: no failover; no engine-level innovation passes through to tenant; per-tenant manual reassignment on vendor outage; doesn't scale to 11 packs × N vendors matrix.
- **Verdict**: rejected. Per-tenant pin remains available as an override (per `runbooks/mt-engine-degraded-shed.md`), but default is dynamic routing.

### Alternative C — Round-robin across engines

- **Pros**: simple; load-balanced.
- **Cons**: ignores capability + cost + residency; degrades quality unpredictably; vendor-specific strengths squandered.
- **Verdict**: rejected.

### Alternative D — ML-learned routing (per-call NN classifier predicting best engine)

- **Pros**: potentially optimal; learns from feedback.
- **Cons**: brittle to vendor capability changes; opaque; harder to audit per EU AI Act Art. 12 record-keeping (decision explainability); meta-ML adds latency budget; OPEN concern for residency invariants under ML drift.
- **Verdict**: scheduled-for-distinct-tracked-work; tracked under "ADR-TRANSLATE-#### (future)". The transparent weighted-score algorithm is preferred for M01 because every decision is explainable and auditable.

### Alternative E — Pure LLM (Anthropic / OpenAI / Gemini) for all translation

- **Pros**: best quality on contextual content; consistent UX.
- **Cons**: 10–100× higher cost than NMT for short segments; latency higher than NMT; rate-limit-bound; residency-bound; not appropriate for ui-string + code-comment content classes.
- **Verdict**: rejected as default. LLM-class engines used per quality-tier + content-class.

### Alternative F — Aggregator (LiteLLM / OpenRouter pass-through)

- **Pros**: no per-vendor adapter work.
- **Cons**: opaque billing; residency invariant impossible to enforce; credential isolation gone; EU AI Act Art. 50 disclosure suppressed.
- **Verdict**: rejected per foundry-providers ADR-style reasoning (residency + credential + audit posture mandatory).

## Consequences

### positive

1. **Substrate-uniform routing** — every workload µservice sees one stable port surface (`TranslateInvoker`) regardless of which engine responds; no per-product translation code.
2. **Per-tenant per-pack residency invariant** — first-class router constraint with default-deny per pack; no competitor enforces this at router layer (per `competitor-parity-matrix.md`).
3. **In-house adoption path** — router demotes-to-external when in-house parity not met; recovers-to-in-house when bar passes; observability-driven; safe blue/green per ADR-0026.
4. **Explainable + auditable** — every decision emits `EngineRouted` with `(candidate_set, selected, reason)`; EU AI Act Art. 12 record-keeping satisfied.

### negative

1. **Per-vendor adapter maintenance burden** — vendor model swaps + response-shape changes require adapter version pins; ongoing vendor-watch toil.
2. **Per-vendor authentication / cookie / API-version drift** — subscription transports (where supported) are fragile (per foundry-providers' adapter-substitution attack model).
3. **Cost-mix variance** — tenant cost-per-day fluctuates as routing demotes/recovers; may surprise budget-sensitive tenants; mitigated by per-tenant ceiling + cost alert (per `cost-budget.md`).

### neutral

1. **Adapter-pin runbook ownership** — ops-security owns per-vendor adapter-pin lifecycle; ops cost.
2. **WMT-eval cadence** — quarterly per-vendor BLEU/chrF/COMET regression detection adds eval-set maintenance work.
3. **In-house parity bar (0.95 × incumbent)** is conservative; intentionally so to prevent quality regression; will tighten over time as in-house gains.

## Validation

- `tests/integration/router_residency_filter_default_deny.rs` — Alternative A residual covered.
- `tests/load/router_decision_p99_5ms.rs` — performance bar.
- Quarterly WMT eval — quality bar per language pair.
- `EngineRouted` event count = decision count (audit).
- Per-tenant adapter pin: tenant flow under `runbooks/mt-engine-degraded-shed.md`.

## References

- ADR-0025 — Foundry as engineering platform.
- ADR-0026 — In-house AI substrate roadmap.
- ADR-0135 — Connect super-app expansion (parent ADR).
- ADR-0131 — Per-microservice flat layout.
- ADR-0133 — Industry-best-practice conformance program.
- ADR-TRANSLATE-0003 — QE + EU AI Act bounds.
- ADR-TRANSLATE-0004 — Data-residency-bound inference.
- WMT shared task — `statmt.org/wmt24/`.
- COMET — `unbabel.github.io/COMET/`.
- chrF — Popovic 2015.
- BLEU — Papineni et al. 2002.
- DeepL benchmarks (DeepL whitepapers).
- Slator + LISA + LocWorld vendor reports.
- foundry-providers PRD (router patterns inherited).
- LiteLLM router pattern (open-source; reference).
- EU AI Act (Reg. (EU) 2024/1689) Art. 12 (record-keeping) + Art. 50 (transparency).
