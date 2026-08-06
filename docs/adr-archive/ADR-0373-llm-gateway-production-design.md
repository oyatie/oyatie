---
id: ADR-0373
status: Superseded
deciders: founder, council-architecture
date: 2026-05-26
owner: council-architecture
supersedes: []
superseded_by: [ADR-0700]
related: [ADR-0090, ADR-0105, ADR-0131, ADR-0007, ADR-0003, ADR-0145, ADR-0092]
planning_impact: true
milestone: M-CLOUD-INTELLIGENCE
depends_on: []
door: two-way
affected_surfaces:
  crates: [oya-cloud-intelligence-kernel, oya-cloud-intelligence-rest]
  microservices: [cloud-intelligence]
  specs: []
deliverables:
  - id: ADR-0373-D1
    description: "Provider-abstraction adapter layer behind a canonical OpenAI-compatible REST surface (OpenAPI 3.2.0: /v1/chat/completions incl. stream:true SSE with the data:[DONE] sentinel, /v1/embeddings, /v1/models, plus a separate admin realm), OpenAI error envelope {error:{message,type,param,code}}, and byte-passthrough SSE — grounded in the hyperscaler brief (Kong AI Proxy, Azure APIM AI gateway, OpenAI error codes)."
    exit_criteria: "the gateway contracts (OpenAPI 3.2.0 + the per-provider adapter design) are authored under microservices/cloud-intelligence and the design-spec-maturity gate is green for the service."
    verified_by: "oya gate validate design-spec-maturity-claims + oya gate validate supply-chain (catalog)"
  - id: ADR-0373-D2
    description: "Key-pool resilience: the failure→blacklist→jittered-cooldown→restore state machine (oya-cloud-intelligence-kernel) extended with a per-provider circuit breaker honoring upstream Retry-After, a bounded idempotent-retry failure ladder (in-key retry → key rotation → provider fallback → graceful 503), per-tenant key pools, and concurrent token budgets — grounded in the brief (LiteLLM cooldown/fallbacks, Azure dynamic circuit breaker)."
    exit_criteria: "the kernel key-pool state machine is implemented + unit-tested and the resilience/failure-modes design docs are authored."
    verified_by: "cargo nextest -p oya-cloud-intelligence-kernel + oya gate validate design-spec-maturity-claims"
  - id: ADR-0373-D3
    description: "Audit + residency: a Bedrock-ModelInvocationLog-shaped immutable per-invocation audit record (extended with tenant + hashed key-pool/ingress refs) appended to the evidence/audit-chain substrate, with default-OFF prompt/completion body logging and residency-pinned body-spill (AsyncAPI 3.1.0 cloud-intelligence.usage.v1 + cloud-intelligence.audit.v1 channels) — grounded in the brief (Bedrock model-invocation-logging, Cloudflare DLP)."
    exit_criteria: "the audit-evidence-emission + data-residency design docs and the AsyncAPI 3.1.0 contract are authored; manifest audit_chain declares the designed posture as a non-runtime claim."
    verified_by: "oya gate validate design-spec-maturity-claims + oya gate validate honest-claims"
  - id: ADR-0373-D4
    description: "OpenSLO SLI set for the gateway: availability, time-to-first-token (TTFT, headline streaming SLI), end-to-end latency (non-stream), error-rate, and stream-completeness — with targets labeled provisional starting hypotheses (no official vendor SLO exists per the brief) to be replaced by measured baselines once live."
    exit_criteria: "one *.openslo.yaml per SLI exists under microservices/cloud-intelligence/slos with provisional targets + non-claim labeling."
    verified_by: "oya gate validate design-spec-maturity-claims"
purpose: Record the production design of the cloud-intelligence agent-dispatch gateway (microservices/cloud-intelligence; the oya-cloud-intelligence-kernel + oya-cloud-intelligence-rest crates) — a multi-provider key-pool reverse proxy — at a hyperscaler-grade design-maturity bar, grounded in cross-vendor best-practice research (Azure APIM AI gateway, AWS Bedrock, Cloudflare AI Gateway, Kong AI Proxy, LiteLLM, OWASP LLM Top 10 2025). This ADR is the decision record the design dossier (microservices/cloud-intelligence/{PRD.md, design/*, contracts/*, slos/*}) implements; it makes no runtime/deployment claim (the service is a code-backed local foundation per its manifest non-claims).
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0373: cloud-intelligence gateway production design (provider-abstraction, key-pool resilience, audit)

## Status
Accepted — 2026-05-26.

## Context
`microservices/cloud-intelligence` (crates `oya-cloud-intelligence-kernel` + `oya-cloud-intelligence-rest`, introduced in PR #196) is a clean-room multi-provider cloud-intelligence AI gateway: a key-pool reverse proxy fronting OpenAI/Anthropic/Gemini with SSE streaming, a failure→blacklist→jittered-cooldown→restore key-rotation state machine, admin + ingress constant-time auth realms, OpenBao-sourced pooled keys, and OTel/Prometheus metrics. PR #196 shipped the code without a design-spec package, failing the `design-spec-maturity-claims` gate. The founder directed bringing it to the production-100 design bar with decisions grounded in current hyperscaler best practice (`/best-practice-research`) and converged via `/idea-refine`, not invented. The full cited evidence is `microservices/cloud-intelligence/design/hyperscaler-best-practice-brief.md`.

## Decision
Adopt the design recorded in the gateway design dossier, summarized as four decisions:
1. **Provider abstraction + canonical OpenAI-compatible surface** (ADR-0373-D1): one canonical OpenAI-shaped request/response with per-provider adapter traits; OpenAPI 3.2.0 contract; byte-passthrough SSE; OpenAI error envelope; 429 + `Retry-After`; two security schemes (ingress vs admin).
2. **Key-pool resilience state machine** (ADR-0373-D2): the kernel failure→blacklist→jittered-cooldown→restore machine + per-provider circuit breaker honoring upstream `Retry-After`, a bounded retry failure ladder, per-tenant key pools, and concurrent token budgets.
3. **Bedrock-shaped immutable audit + default-off body logging** (ADR-0373-D3): per-invocation audit record (+ tenant/hashed-key refs) to the audit-chain; default-OFF prompt/completion logging; residency-pinned body-spill; AsyncAPI 3.1.0 usage + audit channels.
4. **OpenSLO SLI set** (ADR-0373-D4): availability, TTFT (headline), latency, error-rate, completeness — provisional targets pending measured baselines.

Tenancy follows the dogfood-tenancy model (Oyatie is a tenant of its own gateway). This ADR is design-stage; the manifest's `explicit_non_claims` govern what is NOT yet implemented (no live deployment, no measured SLO, no runtime audit persistence).

## Rejected alternatives
- **No provider abstraction (pass provider-native bodies only)** — rejected: loses OpenAI-SDK interop; kept only as an `llm_format`-style passthrough escape hatch (Kong pattern).
- **Whole-shell semantic caching in v1** — deferred: exact-match cache first; semantic caching (Azure/Apigee) is v2.
- **Fail-open on key exhaustion** — rejected: amplifies denial-of-wallet (OWASP LLM10); the design fast-fails with `Retry-After` set to the soonest key-restore time.
- **Reusing repo ADR numbers 0370/0371/0372** — rejected (those govern the Talos substrate / Cloudflare control-plane / SolidJS frontend); the gateway design decisions are recorded here under ADR-0373 to avoid a false ADR citation under the honest-claims gate.

## Consequences
- Positive: a hyperscaler-grounded, contract-first design (OpenAPI 3.2.0 / AsyncAPI 3.1.0 / proto3 / Cedar / OpenSLO) that greens `design-spec-maturity-claims` for the service and gives the kernel/rest crates an auditable decision record.
- Cost: the SLO targets are provisional hypotheses (no official vendor SLO for TTFT); they must be re-grounded with measured baselines before any availability/SLO claim.
- Neutral: this ADR documents an existing code foundation + its forward design; it introduces no new runtime claim.

## Verification
Per-deliverable `verified_by`. The decision is met when the gateway design dossier exists, the `design-spec-maturity-claims` gate is green for `microservices/cloud-intelligence`, the kernel tests pass, and the manifest's design-stage non-claims are intact (honest-claims green).

## References
ADR-0090 (hyper canonical HTTP backbone), ADR-0105 (layer enum — kernel/rest), ADR-0131 (per-microservice flat layout), ADR-0007 (Cedar authorization), ADR-0003 (audit chain + evidence emission), ADR-0145 (inter-microservice communication), ADR-0092 (dependency-seam). Research + sources: `microservices/cloud-intelligence/design/hyperscaler-best-practice-brief.md`. Canonical contracts: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, OpenSLO.
