---
doc_class: CompetitorParityMatrix
microservice: feature-flags
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0159
  - ADR-0248
  - ADR-0250
companion_docs:
  - microservices/feature-flags/PRD.md
  - microservices/feature-flags/ARCHITECTURE.md
  - microservices/feature-flags/sdk-plan.md
planned_enforcement_ref: oya-governance-adr-adherence-matrix
---

# Competitor Parity Matrix — Feature Flags

Hyperscaler precedents compared: LaunchDarkly, Split.io, Statsig, Optimizely, GrowthBook, Unleash, OpenFeature.

## Capability parity

| Capability | LaunchDarkly | Split.io | Statsig | Optimizely | GrowthBook | Unleash | OpenFeature | **oyatie feature-flags** |
|---|---|---|---|---|---|---|---|---|
| Boolean flags | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | spec | ✓ |
| String flags | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | spec | ✓ |
| Number flags | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | spec | ✓ |
| JSON object flags | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | spec | ✓ |
| Percentage rollout | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ |
| User/cohort targeting | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ (Cedar predicates) |
| A/B experiment | ✓ | ✓ | ✓ | ✓ | ✓ | partial | — | ✓ |
| Multivariate experiment | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ |
| Bayesian stats | — | — | ✓ | partial | ✓ | — | — | ✓ |
| Frequentist stats | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ |
| Sequential testing | — | — | ✓ | ✓ (Stats Accelerator) | — | — | — | ✓ (mSPRT) |
| Kill-switch | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ (life-safety bypass) |
| Audit log | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ (ADR-0028 sealed) |
| Multi-tenant isolation | partial | partial | partial | — | partial | ✓ | — | ✓ (per ADR-0244) |
| Per-pack compliance overlay | — | — | — | — | — | — | — | ✓ (HIPAA/PCI/GDPR/KR-FSS) |
| OpenFeature provider | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | spec | ✓ |
| SDK: Rust | — | — | — | — | — | — | ✓ | ✓ |
| SDK: TypeScript | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| SDK: Python | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| SDK: Go | ✓ | ✓ | ✓ | — | ✓ | ✓ | ✓ | roadmap |
| SDK: Java | ✓ | ✓ | ✓ | ✓ | — | ✓ | ✓ | roadmap |
| Streaming flag updates | ✓ (SSE) | ✓ | ✓ | ✓ | ✓ | ✓ (SSE) | — | ✓ (SSE + WebSocket) |
| Local evaluation (relay proxy) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ (SDK in-process cache) |
| gRPC API | partial | — | partial | — | — | — | ✓ | ✓ (HTTP/3 + proto3) |
| Sovereign cloud / data-residency | partial | — | — | partial | — | partial | — | ✓ (per ADR-0248) |
| Emergency services bypass | — | — | — | — | — | — | — | ✓ (life-safety hard rule) |
| Cedar authorization | — | — | — | — | — | — | — | ✓ (ADR-0243) |

## Areas where oyatie exceeds competitors

1. **Per-pack compliance overlays**: No competitor offers pack-mandated flag overrides (HIPAA forces `phi-exposure = off` etc.). This is a unique differentiator for regulated industries.
2. **Life-safety emergency bypass**: Emergency-services principals never see a challenge or rate limit. No competitor documents this invariant.
3. **Cedar policy targeting**: Targeting rules are Cedar predicates (composable, auditable, version-controlled). Competitors use bespoke DSLs.
4. **ADR-0028 sealed audit chain**: Audit events are cryptographically sealed and Merkle-chained. Competitors offer append-only logs but not sealed chains.
5. **Sovereign-cell data-residency**: True per-cell data residency with pack-level enforcement. Competitors offer region selection but not pack-enforced residency.
6. **Bayesian + sequential testing**: Only Statsig and Optimizely offer comparable statistical rigor. GrowthBook (open source) is the closest open-source comparator.

## Areas where competitors lead (roadmap gaps)

| Gap | Competitor leader | Oyatie roadmap |
|---|---|---|
| SDK breadth (Go, Java, .NET, Swift) | LaunchDarkly (10+ SDKs) | Phase 2 (Q4 2026) |
| Metric pre-computation pipeline | Statsig (pre-aggregated metrics warehouse) | Phase 2 |
| No-code experiment setup (visual editor) | Optimizely (visual editor) | Phase 3 (Workflow Studio integration) |
| Holdout groups / global holdouts | LaunchDarkly, Statsig | Phase 2 |
| Funnel experiments (multi-step) | Optimizely | Phase 2 |
| ML-powered auto-targeting | Statsig | Phase 3 (Intelligence substrate) |

## Marketplace exposure

Feature-flags is substrate; it does not expose marketplace surfaces directly. Future:
- `marketplace-experiments` pack: third-party analytics integrations (Amplitude, Mixpanel) as experiment-results consumers via AsyncAPI `flag-state-changed` channel.
- Marketplace flag `marketplace_eligible: true` in manifest.
