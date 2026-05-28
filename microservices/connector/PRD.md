---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-connector-integration-substrate
microservice: connector
status: Accepted
sales_segment: shared-substrate
tier: substrate
milestone_first_ship: M01-foundation
related_adrs: [ADR-0056, ADR-0105, ADR-0106, ADR-0145, ADR-0242, ADR-0243, ADR-0244, ADR-0245, ADR-0246, ADR-0248, ADR-0253, ADR-0255, ADR-0258, ADR-0263, ADR-0273, ADR-0294, ADR-0295, ADR-0296, ADR-0297, ADR-0338, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345]
related_specs: [/specs/connector-catalog.json, /specs/inter-microservice-communication-reform.json]
companion_docs:
  - microservices/connector/ARCHITECTURE.md
  - microservices/connector/threat-model.md
  - microservices/connector/dpia.md
  - microservices/connector/compliance.md
  - microservices/connector/multi-region.md
  - microservices/connector/policy/abuse-defence.cedar
  - docs/standards/documentation-rigor.md
date: 2026-05-20
owner_team: axis-integration
doc_status: published
inbound_citations:
  - docs/DOC-CATALOG.md
  - docs/AGENTS.md
  - microservices/workflow-engine/PRD.md
---

# PRD-connect: Integration Substrate (Connector Directory + OAuth Broker + Webhook Receiver)

## A. Problem

Every product surface — workflow-engine, marketplace, ops-dashboard-control-center, intelligence, foundry — needs to call hundreds of third-party SaaS APIs (Slack, Salesforce, Shopify, Stripe, Notion, Linear, GitHub, AWS Lambda, Twilio, Toss Payments, etc.) on behalf of tenants. Without a shared substrate, each product re-implements OAuth dances, webhook receivers, payload signature verification, retry-and-DLQ machinery, and connector adapters. The result is duplicated brittle code, inconsistent abuse-defence posture, and per-product credential sprawl that violates ADR-0255 §D-4 (provider-BYOK SecretReference model) and ADR-0296 (credential sidecar isolation).

The hyperscaler precedent: **Zapier** + **n8n** + **Workato** + **Boomi** + **MuleSoft** + **Tray.io** + **Pipedream** + **AWS EventBridge**. Each provides a connector directory, OAuth broker, and webhook receiver as a unified primitive. oyatie ships the same shape as a tenant-scoped substrate.

This µservice is **substrate, not product**. It is consumed by workflow-engine (executes workflows that call connectors), marketplace (lists connectors as marketplace items), foundry (Foundry runs use connectors), ops-dashboard-control-center (admin actions to provision/revoke OAuth), and every product surface that needs to talk to an external SaaS.

Distinct from `workflow-engine` (which orchestrates *execution* of workflows): `connector` provides the **integration directory + connector adapters + OAuth dance + webhook receiver substrate**. Workflow engine consumes connect via library-first dispatch per ADR-0246.

## B. Target Users

### B2B Personas

- **Persona TIE — Tenant Integration Engineer.** Goals: wire their tenant's Salesforce, Slack, GitHub, and internal Postgres to oyatie workflows in <30min. Frustrations: each SaaS uses a different OAuth flow; webhooks fail silently; rate limits get exhausted by parallel runs.
- **Persona TAB — Tenant Admin (provider-BYOK).** Goals: provision per-tenant API keys (Toss Payments merchant key, AWS IAM role ARN, Slack bot token) via OpenBao SecretReference so substrate never holds raw credentials. Frustrations: vendors that demand long-lived static tokens; rotation is manual; revocation propagation is unclear.
- **Persona MPO — Marketplace Publisher (Operator).** Goals: list a connector adapter as a paid marketplace item; collect royalties; ship updates without breaking tenants. Frustrations: no shared substrate; each marketplace item re-implements OAuth.
- **Persona FE — Foundry Engineer.** Goals: a Foundry job that fetches from Shopify, transforms via Intelligence, and writes to BigQuery — using ≥4 connectors composed in one job. Frustrations: per-connector failure modes leak into job logic.

### B2C Personas (downstream)

- **Persona EU — End User (consumer of connector-backed workflow).** Goals: experience the workflow as a single coherent product, not a chain of vendor logos. Frustrations: a flaky third-party shows up as "oyatie is broken" instead of "Stripe is timing out".

## C. User Stories (≥40 across surfaces)

### Connector Catalog (browsing)
1. **TIE-001**: As a TIE, I browse the connector catalog by category (CRM, Comms, Payments, Cloud, Analytics) so I can discover which integrations exist.
2. **TIE-002**: As a TIE, I search the catalog by keyword "send slack message" so I find every connector that offers that capability.
3. **TIE-003**: As a TIE, I view a connector's detail page: triggers, actions, auth model, rate-limit profile, data classes touched, deprecation status.
4. **TIE-004**: As a TIE, I filter the catalog by tenant's active compliance packs so I see only connectors with appropriate jurisdictional posture.
5. **TIE-005**: As a TIE, I view a connector's hyperscaler-equivalent rate-limit budget (Zapier "tasks per month") so I can plan capacity.
6. **MPO-001**: As an MPO, I publish a new connector adapter via marketplace; it lands in the catalog under my publisher namespace.
7. **MPO-002**: As an MPO, I version-bump my connector and the catalog shows both 1.x and 2.x with deprecation banner on 1.x per ADR-0258.
8. **MPO-003**: As an MPO, I retire a connector and existing wirings get a 90-day sunset window with email notifications per ADR-0273.

### OAuth Broker (auth dance)
9. **TIE-010**: As a TIE, I click "Salesforce" and complete OAuth in <90s without ever pasting a raw token into oyatie.
10. **TIE-011**: As a TIE, I revoke a connection and the substrate purges the refresh token + invalidates downstream wirings within 60s.
11. **TIE-012**: As a TIE, I rotate a connection (force re-OAuth) without downtime to in-flight workflow runs.
12. **TAB-001**: As a TAB, I provision a per-tenant OAuth client (my Slack app's client_id + client_secret in OpenBao) so substrate uses *my* tenant's client identity, not oyatie's shared one (provider-BYOK per ADR-0255 §D-4).
13. **TAB-002**: As a TAB, I review every active OAuth grant — what scopes, which user authorized, which workflow consumes — in one dashboard.
14. **TAB-003**: As a TAB, I configure SCIM-based OAuth-grant lifecycle so departing employees' grants auto-revoke.
15. **TIE-013**: As a TIE, I see "Salesforce requires re-auth" notifications when refresh tokens approach expiry, with 7-day-ahead lead time.

### Webhook Receiver (inbound)
16. **TIE-020**: As a TIE, I expose a webhook URL `https://hooks.<tenant>.oyatie.app/<connector>/<wiring-id>` per ADR-0273 per-tenant DNS, and Shopify sends order-created events to it.
17. **TIE-021**: As a TIE, every inbound webhook is HMAC-verified against the connector's per-tenant signing secret; spoofed payloads return 401 + audit event.
18. **TIE-022**: As a TIE, replay attacks are blocked: any payload with `timestamp < now - 5min` returns 401; `idempotency-key` dedupes within a 24h window.
19. **TIE-023**: As a TIE, if my downstream workflow is slow, webhooks queue with backpressure rather than dropping — DLQ kicks in after 3 retries with 1m/5m/30m backoff.
20. **TIE-024**: As a TIE, I view a webhook's recent activity (success/4xx/5xx breakdown, p99 latency, last-failed payload digest) without seeing raw payload contents (PII-redacted).

### Connector Adapter (outbound calls)
21. **TIE-030**: As a TIE, my workflow invokes a connector action (e.g., `slack.chat.postMessage`) and the substrate handles auth, rate-limiting, retry, and observability — my workflow code is one line.
22. **TIE-031**: As a TIE, when a connector returns 429, the substrate retries with exponential backoff respecting `Retry-After` headers.
23. **TIE-032**: As a TIE, when a connector returns 5xx, the substrate retries 3× with jitter, then DLQs the call with full context.
24. **TIE-033**: As a TIE, I configure per-connector circuit-breakers per ADR-0145 §invariant-1; a Salesforce outage doesn't cascade into other connector latency.
25. **TIE-034**: As a TIE, the connector adapter emits OTel spans, metrics (`oya_connector_action_total{connector,action,status}`), and audit events per ADR-0263.
26. **FE-001**: As an FE, my Foundry job's connector calls inherit the job's tenant context, not the substrate's; cross-tenant data isolation holds across the connector boundary.

### Data Mapping (payload canonicalization)
27. **TIE-040**: As a TIE, I map a Salesforce contact's `Email__c` field to oyatie's canonical `email_address` via a visual mapper; the mapping persists and is versioned.
28. **TIE-041**: As a TIE, the mapper detects when a vendor changes their schema (e.g., Salesforce renames a field) and flags affected wirings.
29. **TIE-042**: As a TIE, PII-bearing fields (email, phone, address) get automatic data-class tagging that downstream policy enforces.

### Retry + DLQ
30. **TIE-050**: As a TIE, I view the DLQ for any wiring; failed calls show error class, last-tried-at, retry count, and a "replay" button.
31. **TIE-051**: As a TIE, replay from DLQ honors idempotency-keys; duplicate replays don't double-charge or double-message.
32. **TAB-004**: As a TAB, DLQ retention is per-tenant configurable (default 7d; max 30d per compliance pack).

### Abuse Defence (§3.2.3 baseline)
33. **TAB-005**: As a TAB, my webhook endpoint sees zero impact from default-path traffic; abuse-defence only intercepts confirmed-bot-score traffic per UX-floor invariant.
34. **TAB-006**: As a TAB, a sudden burst of webhook traffic from an unknown IP triggers adaptive challenge — my legitimate vendors (with known JA4 fingerprints) flow through unmodified.
35. **TAB-007**: As a TAB, accredited search-engine and accessibility-tool crawlers are allow-listed via `audience_type = FRIENDLY_CRAWLER_PARTNER` and never see a challenge.

### Cross-pack / Sovereign cells
36. **TAB-008**: As a TAB in pack-kr, my Toss Payments connector is allow-listed; pack-eu tenants don't see it in their catalog.
37. **TAB-009**: As a TAB in pack-us-healthcare, only HIPAA-eligible connectors (BAA signed by oyatie + the vendor) are listed.
38. **TAB-010**: As a TAB, when my tenant's `jurisdiction_code` changes, the catalog re-renders per the new pack overlay without me re-configuring.

### Cell-tier awareness
39. **TIE-060**: As a TIE in a Tier-0 edge cell (per ADR-0248), my connector calls inherit the cell's bot-mgmt + WAF posture automatically.
40. **TIE-061**: As a TIE in a Tier-3 data cell (no internet egress), only internal-protocol connectors (Postgres, Mongo, Snowflake via PrivateLink) are available; SaaS connectors are filtered out by Cedar policy.

### Observability + audit
41. **TAB-011**: As a TAB, every OAuth grant + revoke + connector call + webhook receive emits an audit event per ADR-0263 registry; auditors can reconstruct any tenant's integration activity from the audit chain.
42. **TIE-070**: As a TIE, when a connector P99 latency exceeds budget, my tenant's SLO error budget burns per ADR-0130, surfacing on the tenant dashboard.

### Anti-stories (what we refuse)
- **AS-01**: We do NOT execute workflows — that's workflow-engine. provides the substrate; orchestration belongs to the consumer.
- **AS-02**: We do NOT hold tenant-owned credentials in cleartext. All credentials are OpenBao SecretReferences per ADR-0296.
- **AS-03**: We do NOT charge per-call for connectors. Pricing lives in marketplace per the multi-category marketplace doctrine (ADR-0249).
- **AS-04**: We do NOT impose default-path friction (CAPTCHA, JS-PoW) on legitimate traffic. UX-floor per §3.2.3 is non-negotiable.

## D. Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | TIE | a searchable catalog of ≥500 connectors at GA | I can wire integrations without writing adapter code | connector-catalog | Must |
| FR-02 | TIE | OAuth-broker that handles authorization-code, client-credentials, JWT-bearer, refresh-token flows | every common SaaS works without per-tenant code | oauth-broker | Must |
| FR-03 | TIE | webhook-receiver with HMAC verification, replay-window ≤5min, idempotency-key | inbound vendor events are trustworthy | webhook-receiver, signature-verification | Must |
| FR-04 | TIE | connector adapters that handle auth + retry + rate-limit + circuit-break + observability | my code calls actions as a one-liner | connector-adapter | Must |
| FR-05 | TIE | data-mapper visual editor with schema-drift detection | vendor schema changes don't silently break my wirings | data-mapping | Must |
| FR-06 | TIE | retry + DLQ with per-wiring policy + replay UI | transient failures self-heal; persistent ones surface | retry-and-DLQ | Must |
| FR-07 | TAB | per-tenant provider-BYOK provisioning per ADR-0255 §D-4 | substrate never holds my raw credentials | oauth-broker | Must |
| FR-08 | TAB | abuse-defence baseline per §3.2.3 (anti-bot + anti-spoof + anti-scrape) wired at the webhook ingress | my endpoint isn't a soft target | webhook-receiver | Must |
| FR-09 | MPO | a marketplace publishing flow for third-party connectors | I can ship + monetize my adapter | connector-catalog | Must |
| FR-10 | FE | library-first dispatch per ADR-0246 amendment | Foundry jobs call connectors without a network hop when collocated | connector-adapter | Must |

## E. Non-Functional Requirements (six-dimension matrix per documentation-rigor §1.2)

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Connector action latency (overhead only) | ≤5ms | ≤20ms | ≤50ms | Excludes vendor RTT |
| OAuth flow completion (user-click → token-stored) | ≤2s | ≤8s | ≤15s | Single redirect |
| Webhook ingest → ack to vendor | ≤30ms | ≤100ms | ≤200ms | HMAC verify + queue accept |
| Webhook → workflow trigger latency | ≤200ms | ≤1s | ≤2s | Includes queue + dispatch |
| Catalog search query | ≤50ms | ≤150ms | ≤300ms | Indexed by ElasticSearch |
| DLQ replay latency | ≤1s | ≤5s | ≤10s | From replay-click to vendor call |

Evidence: modeling note at `docs/performance-budgets/connector-integration-substrate.md` derives each via Little's Law from observed peer-rate (Zapier handles ~3M tasks/min at peak; our 100k tasks/min target requires per-worker queue depth ≤30 with 100-worker pool — see §F-3 capacity model).

### Scalability

MUST scale to: ≥500 connectors in catalog; ≥10M webhook receives/day per tenant at p99; ≥100k concurrent OAuth grants in flight across all tenants; ≥1M outbound connector actions/min platform-wide.

Bottleneck: per-tenant rate-limit token-bucket (Valkey-backed). Resolution: shuffle-sharding per ADR-0248 §cellular shuffle-sharding — each tenant pins to N=4 of M=64 Valkey shards, P(any-two-tenants-share-all-shards) = C(M-N,N)/C(M,N) ≈ 6.4e-7. Horizontal scale-out: add shards; rebalance via consistent-hash ring.

### Observability (per ADR-0263 emission contract)

- Metrics: `oya_connector_action_total{connector,action,status}` (cardinality budget: 500 connectors × 50 actions × 5 statuses = 125k series); `oya_connector_oauth_grant_total{connector,outcome}`; `oya_connector_webhook_receive_total{connector,verify_outcome}`; `oya_connector_dlq_depth{wiring_id}`.
- Traces: every connector call is a span with attributes `oya.connector.name`, `oya.connector.action`, `oya.connector.tenant_id`, `oya.connector.vendor_request_id`.
- Audit events (ADR-0263 registry): `OAuthGrantIssued`, `OAuthGrantRevoked`, `ConnectorActionInvoked`, `WebhookReceived`, `WebhookSignatureVerifyFailed`, `DLQEntryAdded`, `DLQEntryReplayed`.

### Performance — tail-latency mitigation

Hedged requests for read-only connector calls (per Tail at Scale, Dean & Barroso 2013) — duplicate request fires after p95 budget if no response; first response wins. Circuit-breakers per ADR-0145 isolate failed vendors.

### Optimization (cost-performance frontier)

Lazy connector-instance creation (per-tenant adapter loaded on first use, not at startup) reduces cold-start RAM by ~80%. Adapter binaries are downloaded from marketplace registry on demand with content-addressed caching (per ADR-0254 Cloud Hypervisor / Kata pods isolation). Cache-invalidation: marketplace version bump emits invalidation event.

### Code quality

Required test classes: unit (per crate), property (HMAC verify roundtrip; OAuth state-machine), fuzz (webhook payload deserializer; OAuth callback parser), load (1M actions/min sustained), e2e (real vendor sandbox for top-50 connectors). Coverage floor: 85% line, 75% branch. Lint passes: `oya-check-bnf-naming-conformance`, `oya-check-13-layer-enum`, `oya-check-cedar-baseline`. Type strictness: Rust `#![deny(warnings)]`. SemVer policy per ADR-0258; ABI compat: minor versions backward-compatible.

### Maintainability

Module boundaries enforced by BNF v4.1 + ADR-0105 13-layer. Versioning policy: SemVer + deprecation per ADR-0258 (90-day notice; 180-day sunset). Configuration: every per-connector tunable in a `connectors/<name>/config.yaml` with schema validation. Reverse dependencies enumerated in `manifest.json:substrate_dependencies`.

### Security

Default-deny Cedar baseline; defence-in-depth FORBID per §3.2.3. provider-BYOK per ADR-0255 §D-4 (`provider_credential_mode: byok`). Credential sidecar isolation per ADR-0296 (≤60s OpenBao TTL). HTTP/3 + strict TLS 1.3 per ADR-0253 (ECH advertised; PQC hybrid `X25519MLKEM768` offered).

### Audit + Compliance

Every audit event is signed by the per-µservice signing key (ADR-0296 sidecar); audit chain Merkle-sealed (ADR-0028). Audit-event-class registry entries declared in `manifest.json:audit_chain.seal_events`.

### Availability + SLO

Availability target: 99.95% monthly for webhook-receive path; 99.9% for OAuth broker; 99.95% for connector-action dispatch. RTO ≤15min. RPO ≤60s (one DLQ flush cycle).

### Data residency

Connector credentials, webhook payloads, audit records inherit tenant's `jurisdiction_code` per ADR-0117. Cross-pack movement forbidden by default.

### DR posture (per ADR-0343)

- Manifest target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_active_active=true`, `replication_shape=active-active-multi-az-cross-region-warm`. The older §E 15-minute/60-second objective remains a stretch SLO, not the ADR-0343 manifest contract.
- Applicable pack floors from `specs/compliance-pack-floors.json`: EU-AI-ACT-2024-HIGH-RISK `1800s/300s` with multi-region required; HIPAA-2024 `3600s/300s` with multi-region required; SOC2-T2 `14400s/900s`; ISO27001-2022 `14400s/3600s`; PCI-DSS-L1-v4 `86400s/3600s`. The effective maximum pack floor is PCI-DSS `86400s/3600s`; connect keeps the stricter substrate target because webhooks and credentials are tenant-critical.
- `failover_runbook=runbooks/dr-failover.md`, resolved at `microservices/connector/runbooks/dr-failover.md`; backup substrates are `postgres_wal_g`, `object_storage_versioned`, `openbao_seal_unseal`, and `audit_chain_merkle_seal`.
- `multi_region_active_active=true` for webhook admission, OAuth callback handling, and audit event sealing; outbound connector calls respect vendor idempotency and rate-limit replay rules.
- Why: connector is being retired as a product surface but remains a connector substrate; tenants need accepted webhooks, OAuth grants, and DLQ evidence to survive regional movement without duplicating third-party side effects.

### Capacity model (per ADR-0340)

- Per-tenant baseline: `0.06 vCPU`, `128 MiB RAM`, `1 GiB storage`, `connections_per_tenant={valkey:1, postgres:2, outbound_http:8}`.
- Scaling dimension: `per_request` for webhooks and connector actions, `per_capability` for OAuth broker, mapping, and DLQ replay.
- Cell placement class: `Tier-1` with manifest `pod_runtime_tier=1`; retiring still owns OAuth broker, webhook receiver, signature verification, and connector-adapter retirement evidence.
- Autoscaling boundaries: min `2` webhook/api replicas per tenant-cell, max `40` before shard/queue split; connector action workers scale by per-vendor token-bucket pressure rather than raw CPU alone.
- Why: the dominant load is third-party webhook bursts and outbound API fan-out; capacity must protect admission and audit evidence while throttling vendor-specific replay safely.

### Sustainability + cost attribution (per ADR-0344)

- Every OAuth grant, webhook receive, connector action, DLQ entry/replay, mapping transform, and audit event emits `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with tenant, product, capability, provider, cell, and compliance-pack dimensions.
- Provider routing is carbon-aware for catalog syncs, non-urgent backfills, and replay batches; it is not carbon-routed for payment/fraud connectors, HIPAA-emergency connectors, OAuth callbacks, or real-time webhook admission.
- Tenant cost transparency surface: connector catalog/admin shows action volume, webhook ingress, DLQ replay, per-connector provider spend, and marketplace-publisher cost; finops-portal rolls up tenant and compliance-pack dimensions.
- Why: integrations can make external-provider cost opaque, so CSRD, SB-253, and SEC climate-disclosure exports must attribute connector emissions and spend to the tenant wiring that caused them.

### API versioning posture (per ADR-0342)

- Public API version model: `YYYY-MM-DD` carrier triplet using `Oyatie-Version` header, `/v/YYYY-MM-DD/` REST/webhook URL prefix, and proto3 field `string oyatie_version = 8001` for public connector events/contracts.
- SDK semver model: connector SDKs publish `major.minor.patch`; individual connector adapters may advance independently while the public broker contract is pinned by date.
- Support window: last `N=3` public versions for at least `180` days after deprecation.
- Per-tenant pinning: yes for connector tenants and marketplace publishers, especially where vendor certification windows require a frozen callback or action contract.
- Internal-mesh exemption: yes; ADR-0145 direct gRPC over HTTP/3 remains tag-compatible and exempt from public carrier routing.

## F. UX Flows (≥6)

### Flow 1: First connector wiring (Salesforce → Slack)

```
TIE clicks "+ Add Connector" in workflow-studio
  → catalog modal opens (search "salesforce")
  → TIE picks "Salesforce" connector v2.3
  → modal shows scopes: ["read_contacts", "read_leads"]
  → TIE clicks "Authorize"
  → redirect to Salesforce OAuth dialog
  → user grants → callback to https://oauth.<tenant>.oyatie.app/callback
  → broker stores refresh_token in OpenBao at secret/<tenant>/connect/oauth/salesforce/<grant-id>
  → workflow-studio shows "Connected ✓"
  → TIE picks an action (e.g., "On new Lead")
  → repeats for Slack (post message)
  → wiring saved; first event flows in <30s
```

### Flow 2: Webhook receive (Shopify order-created)

```
TIE configures Shopify connector → "Order created" trigger
  → broker generates webhook URL https://hooks.<tenant>.oyatie.app/shopify/<wiring-id>
  → broker generates HMAC signing-secret; stored in OpenBao at secret/<tenant>/connect/webhook-secrets/<wiring-id>
  → TIE pastes URL + signing-secret into Shopify admin
  → first order placed → Shopify POSTs payload + X-Shopify-Hmac-Sha256 header
  → ingress: HMAC verify (constant-time) → 30ms p99
  → if verified: enqueue → ack 200 to Shopify (<100ms p99)
  → if not: 401 + audit event WebhookSignatureVerifyFailed
  → if replay (timestamp < now - 5min OR idempotency-key seen): 401 + audit event
  → downstream worker dequeues → invokes workflow-engine
```

### Flow 3: OAuth revoke + propagation

```
TAB clicks "Revoke Salesforce grant <grant-id>" in tenant-admin-surface
  → step-up auth required (WebAuthn passkey per docs/standards/step-up-auth-classes.md)
  → broker: revoke refresh token at Salesforce → mark grant as REVOKED in PG
  → broker: emit OAuthGrantRevoked audit event
  → broker: enumerate downstream wirings via workflow-engine cross-ref
  → workflow-engine: pause wirings; emit notification to wiring-owner
  → propagation complete <60s p99
```

### Flow 4: DLQ replay

```
TIE notices 23 failed Stripe charges in DLQ over last hour
  → opens DLQ panel → filters by wiring-id
  → sees error class "stripe_idempotency_conflict"
  → clicks "Replay all" → confirm dialog (idempotency-key reuse → no double-charge)
  → broker: dequeue 23 entries → invoke vendor with original idempotency-keys
  → Stripe returns 200 (charge already exists) → audit event DLQEntryReplayed × 23
  → DLQ count drops to 0
```

### Flow 5: provider-BYOK provision (TAB onboarding)

```
TAB navigates to "Provider Credentials" in tenant-admin
  → "Provision Salesforce OAuth client" wizard
  → TAB enters their Salesforce Connected App's client_id + client_secret
  → broker writes to OpenBao at secret/<tenant>/connect/oauth-clients/salesforce
  → all subsequent Salesforce OAuth flows for this tenant use TAB's client identity (not oyatie shared)
  → audit event ProviderCredentialProvisioned
```

### Flow 6: Schema-drift detection

```
Salesforce renames Lead.Email__c → Lead.PrimaryEmail__c
  → next mapper validation cycle: catalog-sync worker fetches updated schema
  → diff detected against wiring's saved mapping
  → emit SchemaDriftDetected audit event
  → notify TIE via email + in-app banner
  → wiring remains active (renames are non-breaking) with banner "Field renamed: Email__c → PrimaryEmail__c (auto-mapped)"
  → if breaking (field deleted): wiring auto-pauses; TIE prompted to remap
```

## G. Success Metrics

- Time-to-first-wiring (median): ≤5min from tenant signup → first webhook event flowing.
- Connector catalog coverage: ≥500 at GA; ≥1000 by 12mo post-GA.
- OAuth flow success rate: ≥99.5% (excludes user-cancel).
- Webhook signature verify success rate: ≥99.99% (failures are spoofs).
- DLQ replay success rate: ≥95%.
- Per-tenant abuse-defence false-positive rate: ≤0.1% (legitimate vendor calls challenged).
- Default-path latency overhead: ≤2ms p99 (UX-floor invariant from §3.2.3).

## H. Compliance Impact

Pack overlays activate per `microservices/connector/compliance.md`:
- pack-kr: K-CSAP overlay; Toss Payments + KakaoPay + LINE Pay allow-listed; PIPC notification on credential incidents.
- pack-eu: GDPR Art. 28 sub-processor terms baked into connector vendor matrix; Schrems II transfer mechanisms enforced.
- pack-us-healthcare: HIPAA-eligible-only catalog filter; BAA required per vendor.
- pack-cn: CN-PIPL-2021 overlay; only domestic-licensed payment connectors visible.

Compliance packs are CI-enforced via `oya-governance-compliance-pack-coverage`.

## I. Open Questions

1. Marketplace-listed connectors developed by third-party MPOs — what's the security review SLA? (Currently: 5 business days; targeting 2 days post-GA via automated CI.)
2. Schema-drift detection cadence — currently hourly; do we need real-time (vendor webhook on schema-change)? (Tracking: ICP-CONNECTOR-SCHEMA-DRIFT-RT)
3. Per-connector cost attribution — how do we split vendor-API-cost from oyatie-cost in tenant invoices? (Tracking: FinOps-portal joint design.)

## J. Out of Scope

- Workflow execution (workflow-engine's job).
- Workflow visual editor (workflow-studio's job).
- Marketplace billing (marketplace + finops-portal's job).
- Vendor SLA negotiation (vendor-relations team's responsibility, not the substrate).
- Reverse-proxy for outbound HTTP traffic (api-gateway's job).

## References

- ADR-0145 inter-microservice communication reform
- ADR-0242..0258 keystone bundle 2026-05-20
- ADR-0273 per-tenant DKIM/SPF/DMARC (DNS shape for webhook URLs)
- ADR-0296 library-first credential sidecar
- ADR-0297 abuse-defence baseline
- docs/standards/documentation-rigor.md (the bar this PRD targets)
- Hyperscaler precedents: Zapier, n8n, Workato, Boomi, MuleSoft, Tray.io, Pipedream, AWS EventBridge

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `connector` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `connector` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 3 module pin(s) across 1 context(s).
- Scaling input: `per_request` with cell placement `Tier-1` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
