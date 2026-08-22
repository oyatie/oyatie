---
doc_status: published
---

# Team: Platform — Public API & SDK

## Mission
This team owns the public REST surface, OpenAPI specification generation, webhook signing, and per-language SDK publishing for the Oyatie platform. It exists to ensure that external developers, ISVs, and integration partners have a stable, versioned, well-documented API surface with zero breaking-change surprises. It does **not** own the business logic behind each endpoint (each axis owns its domain); it owns the stability tier, the OpenAPI contract, the webhook signing infrastructure, and the SDK generation pipeline.

## Owned axes / surfaces / contracts
- **Axis(es):** Cross-cutting (SaaS primary consumer; cloud, search, ads publish their own API slices through this team's gateway)
- **Surfaces:**
  - `platform-webhook-kernel` — `WebhookEndpoint`, `SigningKey`, `DeliveryAttempt`, `WebhookSignature`
  - `platform-webhook-app` — delivery, retry, signing, verification
  - Public API gateway (routing, rate limiting, auth token validation)
  - `contracts/openapi/**/*.yaml` — OpenAPI specs for all public-facing surfaces (owner of the file format and stability tier annotations; each axis authors its own slice)
  - Per-language SDK generation pipeline (Rust, TypeScript, Python, Go as first class; others on demand)
  - Developer portal (docs surface; content authored per-axis, portal infrastructure owned here)
- **Cross-axis contracts (DESIGN §10):**
  - `Public REST stability tier` (owner) — all public APIs declare their stability tier via ADR-0040
  - `Webhook delivery + signing` (owner) — external callers and ISVs depend on this contract
- **Catalog records:** `crates/platform-webhook-*`, `contracts/openapi/**/*.yaml`
- **Runbooks:** `runbooks/webhook-delivery-failure.md`, `runbooks/api-gateway-rate-limit-incident.md`, `runbooks/sdk-release.md`
- **ADRs:** ADR-0040 (API stability gate — sole owner)

## In-scope work
- OpenAPI spec validation and stability-tier enforcement (breaking-change CI gate)
- Webhook signing (HMAC-SHA256 per delivery; signing key rotation)
- Webhook delivery: at-least-once, retry with exponential backoff, dead-letter queue
- API gateway: routing, auth token validation (delegates to `platform-tenancy-identity`), rate limiting, per-tenant quota enforcement
- SDK generation: OpenAPI → language SDK; publish to package registries (crates.io, npm, PyPI, pkg.go.dev)
- Developer portal infrastructure: hosting, search, versioned docs
- API versioning policy: deprecation notices, sunset headers, migration guides
- Marketplace listing API surface (co-owned with `axis-saas` for business logic)
- Per-API stability tier classification and promotion (preview → stable → GA)

## Out-of-scope (anti-scope)
- Business logic behind each API endpoint (→ each axis)
- Identity token issuance (→ `platform-tenancy-identity`)
- Per-axis product PRDs (→ per-axis teams)
- Eventing backbone / Kafka (→ `platform-eventing-og`)
- SDK feature completeness beyond what OpenAPI generates (custom SDK extensions are axis-team responsibility)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `platform-tenancy-identity` | Auth token validation in gateway | Per-release |
| All axis teams | OpenAPI spec slices for their surfaces | Per new endpoint |
| `axis-saas` | Marketplace listing API spec | Wave gate |
| `ops-sre-reliability` | API gateway SLO targets, rate-limit runbooks | Quarterly |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| External developers / ISVs | Stable public REST surface, webhook delivery, per-language SDKs | Continuous |
| `axis-saas` | Plugin SDK surface, marketplace listing API | Wave gate |
| `axis-cloud` | Cloud control-plane public API slice publication | Wave gate |
| `axis-search` | Search public API slice | Wave gate |
| `axis-ads-analytics` | Advertiser console API slice | Wave gate |
| `gtm-partnerships` | SDK packaging for partner integrations | Per partner onboard |

## Success metrics
- **Public API breaking changes reaching GA tier without deprecation notice:** 0 (ADR-0040 gate)
- **Webhook delivery success rate:** ≥ 99.5% within 1 hour of trigger
- **SDK release lag after API spec freeze:** ≤ 5 business days
- **Developer portal uptime:** ≥ 99.9%
- **OpenAPI spec coverage of all public endpoints:** 100% (fitness gate)
- **Stability-tier annotations on all public contracts:** 100%

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council (`teams/council-architecture/CHARTER.md`) for stability-tier disputes
- Security: `ops-security` for webhook signing key incidents
- Founder: as last resort

## Communication cadence
- Stand-up: daily async
- Weekly: 30-min sync — API change queue, SDK release pipeline, breaking-change flags
- Cross-team review: participates in monthly cross-axis contract audit for public surface changes

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules; breaking-change PRs require security-reviewer + API stability gate
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch; ADR-0040 amendments on stability-tier policy

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Breaking API change ships without deprecation window | High | ADR-0040 breaking-change CI gate; sunset header enforcement |
| Webhook signing key compromised | High | Key rotation runbook; HMAC verification on consumer side; audit-chain emission on key rotation |
| SDK publish to wrong registry version | Medium | SDK release pipeline has staging + dry-run gate |

## Sources scanned
DESIGN.md §10 (webhook row, public REST stability tier row, marketplace listing row), PRD.md §2 (external developer user class), ADR-0040, DOC-CATALOG.md §2.1 (doc.spec owner = platform-api-sdk).
