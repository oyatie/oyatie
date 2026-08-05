---
id: ADR-0163
status: Accepted
deciders: council-architecture, axis-tenancy, axis-governance, axis-application, ops-product, ops-sre-reliability
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: []
related: [ADR-0001, ADR-0009, ADR-0028, ADR-0049, ADR-0110, ADR-0114, ADR-0157, ADR-0158, ADR-0159, ADR-0160]
related_specs:
  - /specs/tenant-environment-tiers-canonical.json
  - /specs/hyperscaler-architecture-invariants.json
---

# ADR-0163 — Per-tenant Environment Tiers (test / staging / prod), Cell-Isolated, Stripe `sk_test_` API-key Pattern

## Status

Accepted (2026-05-18). Establishes that every oyatie tenant has three environment tiers — `test`, `staging`, `prod` — cell-isolated from each other; API keys are prefix-tagged per tier (Stripe pattern); production-tier requires explicit admin acknowledgment for destructive operations.

## Context

ADR-0028 named the cloud microservice architecture. ADR-0009 named per-tenant per-region cells. ADR-0110 named the ChangeSet state machine (dev / staging / production = code lifecycle). ADR-0157 named the api-gateway tier. ADR-0158 named multi-region disposition.

None named the **per-tenant environment tier**. The conflation is real:

- ADR-0110's `dev` / `staging` / `production` describes *code* lifecycle (does this PR ship?).
- A *tenant* needs orthogonal `test` / `staging` / `prod` environments to safely develop their integration without their experiments touching production data, billing, customer-facing email, etc.

The hyperscaler precedent is uniform:

- **Stripe** — every Stripe account has a "test mode" toggle; API keys are prefixed `sk_test_` / `pk_test_` (test mode) vs. `sk_live_` / `pk_live_` (prod mode); test-mode and live-mode share account but are isolated databases.
- **Twilio** — test credentials provided alongside production credentials.
- **AWS** — `dev` / `staging` / `prod` are separate AWS accounts (organizations); not API-key prefix but account-level isolation.
- **GitHub** — repository environments (`development`, `staging`, `production`) with separate secrets per environment.
- **Vercel / Netlify** — preview deploys vs. production deploys; per-environment env vars.

Without an explicit per-tenant environment tier:

- Tenants cannot safely test integration without polluting their production data.
- Workflow Studio (ADR-0029 / hero product) cannot do "test this automation before turning it on" without a non-prod tier.
- Billing events emitted during integration testing accidentally bill the tenant.
- Customer-facing emails accidentally send from test code paths.

ADR-0163 makes the environment tier a first-class tenant property + API-key-prefix surface.

## Decision

Every oyatie tenant has three environment tiers. Each tier is a logically isolated dataset within the tenant's cell:

### Tier definitions

- **`test`** — sandbox environment. Tenant integrations land here first. Data is ephemeral (90-day TTL default; per-pack overlay). Outbound side effects (email send, SMS send, webhook dispatch, billing event) are *intercepted and logged* but not delivered to external systems. API keys prefixed `sk_test_` + `pk_test_`.
- **`staging`** — pre-production environment. Data is durable (retained per ADR-0049 residency class). Outbound side effects fire to *test recipients only* (configured per tenant — typically the tenant's QA team email addresses). API keys prefixed `sk_stage_` + `pk_stage_`. Production data NEVER copies into staging without an explicit ChangeSet (ADR-0110) approval gate.
- **`prod`** — production environment. Data is durable + residency-bound. Outbound side effects fire to real recipients. API keys prefixed `sk_live_` + `pk_live_`. Destructive operations (DSR delete, tenant offboarding, bulk delete) require admin acknowledgment via Cedar `prod_destructive_acknowledged: true` condition.

### Tier isolation contract

- **Cell-level isolation.** Each tier is a separate logical database (separate PostgreSQL schema OR separate database within the cell's PostgreSQL cluster). RLS policies (per ADR-0009 + tenancy µservice) enforce.
- **API-gateway prefix routing.** The api-gateway tier (ADR-0157) reads the API-key prefix and routes to the corresponding tier's workload pool. A `sk_test_` request never reaches the `prod` schema; structural enforcement at the edge.
- **Outbound side-effect mode.** Each tier declares its outbound mode in `tenancy` µservice's tenant config: `test_mode_outbound = intercept`, `staging_mode_outbound = test_recipients`, `prod_mode_outbound = live`. Every µservice that performs outbound side effects (mail, calendar, webhooks, billing) checks the mode before dispatching.
- **Audit-chain partition.** Per-tier audit-chain subtree (audit-chain µservice partitions by `(tenant_id, env_tier)`). DSR retrieval distinguishes per tier.
- **Foundry isolation.** Foundry µservice's per-tenant GPU/sandbox pool is per-tier (test-tier sandboxes never use the production-tier model weights; cost-budget enforces).

### API-key prefix scheme

| Tier | API key (server) | API key (browser/public) | Audit-chain tag |
|---|---|---|---|
| test | `sk_test_` | `pk_test_` | `env_tier=test` |
| staging | `sk_stage_` | `pk_stage_` | `env_tier=staging` |
| prod | `sk_live_` | `pk_live_` | `env_tier=prod` |

API-key generation lives in the tenancy µservice's `/v1/tenancy/api-keys` endpoint. Cedar policy gates per-tier key issuance (e.g. a tenant developer can issue `sk_test_` keys; only a tenant admin can issue `sk_live_` keys).

### Destructive-operation acknowledgment

For prod-tier destructive operations:

- **Cedar policy** requires a `prod_destructive_acknowledged: true` condition on the principal context.
- **API-gateway tier** verifies the acknowledgment flag is set on the request header `x-oya-prod-destructive-ack: true`.
- **UI / portal** prompts admin with explicit confirmation dialog before sending the header.
- **Audit-chain seal** captures the acknowledgment (who, when, what).

Destructive operations include: DSR delete, tenant offboarding, bulk delete > 100 rows, cell migration, residency-class change (which is per-ADR-0049 already recreate-only).

### Workflow Studio + Foundry implications

- Workflow Studio (hero product) automatically defaults a new flow to the `test` tier. Promotion to `staging` then `prod` mirrors the ChangeSet promotion model.
- Foundry workflows distinguish per tier; test-tier workflows use cheaper/smaller models by default (cost-budget enforcement); prod-tier workflows use production model selection.

## Alternatives considered

### Alternative A — Single tier; tenants self-manage isolation via separate tenant accounts

- **Pros:** zero infra cost; tenant creates two accounts.
- **Cons:** tenants pay for double; cross-account data movement to "promote a test to prod" requires custom ETL; loses the per-tenant audit story; Stripe-class pattern not delivered.
- **Rejected because:** tenant UX is poor; doesn't match SaaS expectations.

### Alternative B — Two tiers (test + prod only; no staging)

- **Pros:** simpler.
- **Cons:** "test" sandbox is too lossy for pre-production rehearsals; "prod" is too dangerous for QA. Staging fills the middle. Stripe / GitHub / Vercel all have at least three logical environments.
- **Rejected because:** three tiers is the proven precedent.

### Alternative C — Three tiers, API-key-prefix, cell-isolated (this ADR)

- **Pros:** Stripe precedent; structural isolation at api-gateway; audit-chain per-tier; destructive-op acknowledgment; aligns with Workflow Studio promotion model.
- **Cons:** every tenant gets three datasets (3× storage baseline); every µservice must check `env_tier` for outbound effects.
- **Accepted.**

### Alternative D — Per-tenant per-environment SEPARATE tenant_ids (test / staging / prod each get their own tenant_id)

- **Pros:** maximum isolation; no env_tier field anywhere.
- **Cons:** breaks "the tenant is one entity" abstraction; billing / account management / SSO triple; conflicts with ADR-0028 + ADR-0009 cell architecture which keys on tenant_id.
- **Rejected because:** tenant is the access principal; multiplying it breaks the data model.

### Alternative E — Per-environment SEPARATE cells (test cell, staging cell, prod cell)

- **Pros:** strongest isolation; separate failure domains.
- **Cons:** 3× cell capacity at the cell-µservice level; cost increase is fleet-wide; ADR-0009 cell architecture doesn't decompose this way (cells are per-tenant-per-region not per-env).
- **Rejected because:** cell architecture is per-tenant-per-region; env tier is a sub-cell concern.

## Consequences

### Positive

1. **Tenants safely integrate.** Test mode isolates side effects; staging mode rehearses against test recipients; prod mode fires live.
2. **Stripe-class developer experience.** API-key prefix is industry-recognizable.
3. **Workflow Studio promotion clean.** Test → Staging → Prod tier ladder matches the ChangeSet promotion model.
4. **Destructive-operation acknowledgment auditable.** Cedar gate + audit-chain seal; SOC 2 CC6.6 evidence rolls up.
5. **Foundry cost-budget per tier.** Test-tier workflows use cheap models; prod-tier uses production models; cost-budget enforces.
6. **Audit-chain partition aligned.** Per-tier audit-chain subtree (ADR-0162); DSR retrieval per-tier.

### Negative

1. **3× tenant storage baseline.** Every tenant carries test + staging + prod datasets. Mitigated by test-tier TTL (90 days default).
2. **Per-µservice env_tier check.** Every outbound-effect µservice (mail, calendar, webhook, billing) must check the env_tier before dispatching. Adds a field check in every adapter.
3. **API-key prefix migration.** Existing tenants get `sk_live_` prefix added; migration window required.
4. **Cedar policy complexity.** Per-tier key-issuance Cedar policy is N more rules per tenant.

### Operational

1. `tenancy` µservice PRD updated with per-tier contract (Companion).
2. Protected merge gate `cloud-ci-tenant-environment-tier` is aggregated by `oya-ci-required` and enforces (a) every µservice that performs outbound side effects checks env_tier, (b) every API-key issuance flow validates Cedar tier-grant, (c) every prod-tier destructive op carries the acknowledgment header. Legacy `oya gate validate tenant-environment-tier` wording is historical/local-feedback provenance only.
3. Per-tier RLS policies authored in tenancy µservice's `policy/tenant-scope.cedar` fragments.
4. Companion spec `specs/tenant-environment-tiers-canonical.json` declares the prefix scheme + Cedar conditions + outbound mode matrix.
5. api-gateway tier reads prefix; routes to env-tier-specific schema.

## References

- Stripe test mode + `sk_test_` keys — https://stripe.com/docs/keys
- Twilio test credentials — https://www.twilio.com/docs/usage/api/test-credentials
- AWS Organizations multi-account strategy — https://docs.aws.amazon.com/whitepapers/latest/organizing-your-aws-environment/organizing-your-aws-environment.html
- GitHub deployment environments — https://docs.github.com/en/actions/deployment/targeting-different-environments/using-environments-for-deployment
- Vercel preview deploys — https://vercel.com/docs/deployments/preview-deployments
- ADR-0001 — cohesion thesis (single tenant entity, env tier is sub-attribute).
- ADR-0009 — cell architecture (per-tenant-per-region).
- ADR-0028 — cloud microservice architecture.
- ADR-0049 — cross-region residency.
- ADR-0110 — ChangeSet state machine (code lifecycle; this ADR is the tenant lifecycle orthogonal).
- ADR-0114 — canary observability + rollback.
- ADR-0157 — api-gateway tier (prefix routing).
- ADR-0158 — multi-region disposition.
- ADR-0159 — feature-flag substrate (per-tenant-per-env flags).
- ADR-0160 — progressive delivery (env tier ladder parallels canary ladder).
