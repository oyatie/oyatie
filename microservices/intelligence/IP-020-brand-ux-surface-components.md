---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P02-consumer-brand-surface
impl_plan_id: IP-020-brand-ux-surface-components
status: pending
owner: axis-intelligence
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, vitest, playwright]
related_adrs: [ADR-0255, ADR-0253]
---

# IP-020: Brand UX surface — Layer-B components

## Intent

Ship `oya-intelligence-brand-ux-surface-sdk-ts`: TypeScript/React component library for the
consumer brand chrome (Layer-B per ADR-0255). Components: SparkleIcon, StreamingText,
TierBadge, RefusalBanner, CostFloorDisclosure, CitationCard. Renders against Layer-A
DispatchOutcome over SSE or WebSocket. UX-floor: high-fidelity streaming with zero challenge
on default path.

## Concrete file targets

| Path | Action |
|---|---|
| `packages/oya-intelligence-brand-ux-surface-sdk-ts/src/SparkleIcon.tsx` | create |
| `packages/oya-intelligence-brand-ux-surface-sdk-ts/src/StreamingText.tsx` | create |
| `packages/oya-intelligence-brand-ux-surface-sdk-ts/src/TierBadge.tsx` | create |
| `packages/oya-intelligence-brand-ux-surface-sdk-ts/src/RefusalBanner.tsx` | create |
| `packages/oya-intelligence-brand-ux-surface-sdk-ts/src/CostFloorDisclosure.tsx` | create |
| `packages/oya-intelligence-brand-ux-surface-sdk-ts/src/CitationCard.tsx` | create |
| `packages/oya-intelligence-brand-ux-surface-sdk-ts/src/useDispatchStream.ts` | create |

## Key implementation notes

- `StreamingText`: progressive token rendering; no flicker on re-render; accessibility — ARIA live region `aria-live="polite"`.
- `RefusalBanner`: renders EU AI Act Art. 13 transparency text when `refusal_reason == AnnexIiiHighRisk`; accessible, localised.
- `CostFloorDisclosure`: renders per-call cost estimate (B2C transparency obligation per ADR-0255).
- `useDispatchStream`: SSE / WebSocket abstraction; auto-detects transport capability; falls back gracefully.
- On-device (Apple Foundation Models): SDK detects `window.AppleIntelligence` and routes locally; no network dispatch.
- WCAG 2.2 AA: every component passes axe-core in CI.

## Acceptance gates

```bash
vitest run packages/oya-intelligence-brand-ux-surface-sdk-ts
playwright test --project=a11y packages/oya-intelligence-brand-ux-surface-sdk-ts
cargo run -p oya-dev-cli -- gate validate brand-ux-surface-wcag-aa
```

## References

- `microservices/intelligence/ARCHITECTURE.md §2` (Layer-B).
- ADR-0255 §5 (brand-ux-surface BC).
- `docs/standards/a11y-canonical.md`.
- IP-016 (SSE transport), IP-017 (WebSocket transport).

## Wave 15 substance conversion — consumer AI chrome

### §A Problem

Layer-B is promised in `ARCHITECTURE.md` as a consumer brand UX surface, but the old slice only lists component
names and generic frontend checks.
The real gap is that consumer-facing AI output must expose streaming state, refusal reasons, citations, model tier,
and cost-floor disclosure without letting UI code call providers directly.
If this IP is weak, product teams will recreate AI chrome in each app and bypass the central dispatch/refusal/audit
contract.

### §B Approach

Ship a TypeScript SDK that renders `DispatchOutcome`, `DispatchChunk`, `RefusalDecision`, and `AttributionGraph`
from Layer-A contracts.
The components are thin presentation adapters over the dispatch SDK, not provider clients.
`useDispatchStream` selects SSE or WebSocket using IP-016/IP-017 transport capabilities and reports every terminal
state back to audit and observability.

### §C Deliverables

- Create `packages/oya-intelligence-brand-ux-surface-sdk-ts/src/StreamingText.tsx`.
- Create `RefusalBanner.tsx`, `CostFloorDisclosure.tsx`, `CitationCard.tsx`, `TierBadge.tsx`, and `SparkleIcon.tsx`.
- Create `useDispatchStream.ts` with transport fallback and abort handling.
- Add fixtures derived from `contracts/openapi/intelligence-v1.yaml` response shapes.
- Add Playwright a11y coverage for refusal, citation, loading, and disconnected stream states.

### §D Implementation

1. Type `DispatchOutcome` and `DispatchChunk` from the OpenAPI schema rather than local ad hoc interfaces.
2. Render partial chunks in order and tolerate duplicate chunk ids without visual duplication.
3. Map `RefusalReason` into localized refusal copy while preserving machine-readable reason codes.
4. Render `CitationCard` from attribution spans and source URIs without embedding raw RAG documents.
5. Surface cost-floor disclosure for B2C platform-default calls and hide it for tenant-BYOK calls where policy says so.
6. Expose component events for audit correlation ids but never expose provider credentials or prompts.
7. Verify keyboard, screen-reader, and reduced-motion states in the Playwright project.

### §E Acceptance

The vitest and Playwright gates must cover happy streaming, refusal, citation, cost disclosure, transport reconnect,
and abort paths.
The final proof references `ARCHITECTURE.md` §2, `contracts/openapi/intelligence-v1.yaml`, and the WCAG gate named
in the original acceptance block.

### §F Evidence

Local evidence: `capabilities/assist-draft.yaml`, `capabilities/attribution.yaml`, `slos/first-token-latency.openslo.yaml`,
`slos/streaming-throughput.openslo.yaml`, and `runbooks/assist-draft-policy-refusal.md`.
Doctrine evidence: ADR-0255 Layer-B, ADR-0253 transport profile, ADR-0263 audit evidence.

### §G Counterparts

| Counterpart | Relevant behaviour | Oyatie closure |
|---|---|---|
| OpenAI ChatGPT UI/API widgets | Streaming answer, citations, refusal copy | Match UX primitives while binding to tenant-scoped dispatch and audit |
| Anthropic Console | Model tier and safety feedback | Keep transparent refusal/cost state in reusable product components |
| Google AI Studio | Multimodal previews and grounding display | Preserve grounding display without provider-specific UI coupling |

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-020-brand-ux-surface-components.md` matched `attribution, cost`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
