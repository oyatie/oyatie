---
adr_id: ADR-0213
title: Ecosystem-as-a-Service architecture — Plugin/App Store substrate (third-party developer plugins/apps) + Developer SDK
status: Superseded
date: 2026-05-18
owner_team: council-architecture + axis-ecosystem
deciders: [founder, council-architecture, council-security, council-privacy, council-design-system]
template_id: TPL-ADR
doc_status: published
supersedes: []
superseded_by: [ADR-700]
amended_by: [ADR-0249]
related_adrs:
  - ADR-0001  # cohesion thesis (one product, seven axes); plugin-app-store is an axis
  - ADR-0002  # tenant + identity kernel; plugins install against tenant
  - ADR-0003  # audit chain + evidence emission; every plugin-app action seals
  - ADR-0007  # Cedar authorization; per-plugin permission scoping enforced via Cedar
  - ADR-0008  # data-use boundary; per-plugin data access bound to declared scope
  - ADR-0028  # Bominal-inherited audit chain
  - ADR-0037  # Bominal-inherited plugin substrate; ADR-0213 supersedes for new work
  - ADR-0056  # BNF v4.1 crate naming
  - ADR-0065  # Rust-WASM hybrid (developer-sdk web portal substrate)
  - ADR-0105  # 13-layer enum + check family patterns
  - ADR-0110  # ChangeSet state machine (plugin/app version state machine inherits)
  - ADR-0123  # Hyperscaler maturity claim gate (HG-PAS + HG-SDK)
  - ADR-0131  # Per-microservice flat layout (plugin-app-store + developer-sdk ship under this)
  - ADR-0132  # No-grouping policy; EaaS is two single-concern µservices, not one bundle
  - ADR-0139  # Agentic SLO-gated promotion (per-plugin SLO publishing inherits)
  - ADR-0147  # Wasmtime sandbox baseline
  - ADR-0394  # first-party Rust developer portal and composition boundary
  - ADR-0181  # Cosign signing (per-plugin signature verification)
  - ADR-0185  # OpenAPI 3.2 codegen for SDK families
  - ADR-0199  # Per-tenant cost attribution (revenue share computation feeds)
  - ADR-0200  # Wasmtime canonical; plugin execution surface
  - ADR-0211  # In-house tech policy; EaaS itself is in-house from day one
  - ADR-0212  # Buildability bar; EaaS µservices satisfy ADR-0212 at scaffold time
related_specs:
  - /specs/microservices/plugin-app-store.json    # (to be authored)
  - /specs/microservices/developer-sdk.json       # (to be authored)
  - /specs/master-plan-sequencing.json§ecosystem-as-a-service
  - /specs/hyperscaler-gates.json#HG-PAS
  - /specs/hyperscaler-gates.json#HG-SDK
unbundle_descendants:
  - microservices/plugin-app-store/   # discovery + install + lifecycle + per-plugin billing aggregation + vetting (third-party developer plugins/apps)
  - microservices/developer-sdk/      # contracts + SDK families + sandbox + dev portal + payout
inherited_doctrine_from:
  - WeChat Mini-Program platform — single super-app shell, third-party developers as a first-class persona, vetted mini-program ecosystem
  - Apple App Store — vetting pipeline, per-app permissions, signature verification, revenue share, developer onboarding, sandbox runtime, entitlements
  - VS Code Marketplace — extension catalog, per-extension permission scoping, signed VSIX artifacts, in-IDE install flow
  - AWS Marketplace (SaaS + AMI catalog) — vetted catalog of third-party offerings, per-account procurement, billing aggregation, security review
  - Stripe — third-party developer onboarding, KYC + AML, revenue share + payout substrate, developer dashboard
  - Shopify App Store — published / unlisted / private app modes, per-tenant install model, per-install configuration, scoped permissions
  - Salesforce AppExchange — enterprise vetting (security review), per-app subscription billing
  - JetBrains Marketplace — IDE plugin distribution + paid plugins + revenue share + per-plugin signing
forbidden_primitives_for_implementation:
  - "External plugin-store SaaS (e.g., GitHub Marketplace as backing substrate) — plugin-app-store is 100% in-house per ADR-0211"
  - "External developer-portal SaaS or third-party portal runtime — first-party Rust portal per ADR-0394"
  - "External revenue-share SaaS (e.g., Tipalti, Stripe hosted) — payout substrate is in-house per ADR-0211 §revenue-share"
  - "External package-manager surface (e.g., npm registry, crates.io) for first-party SDK distribution — vendored under microservices/developer-sdk/iac/registry per ADR-0211 §package-distribution"
---

<!-- Canonical-base: specs/adr/canonical-frontmatter-schema.json + docs/templates/adr-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# ADR-0213: Ecosystem-as-a-Service architecture — Plugin/App Store substrate

## Status

**Proposed** — 2026-05-18.

Open for council review through 2026-06-01. Acceptance gates:
- council-architecture (chair: founder) — approves the EaaS doctrine + two-µservice unbundle (plugin-app-store + developer-sdk).
- council-security — approves the vetting pipeline + sandbox isolation model.
- council-privacy — approves the per-plugin data-use boundary + per-plugin Cedar scoping.
- council-design-system — approves the discovery + install + permission-grant UX.
- axis-finops (consulted) — approves the revenue-share + payout substrate plug.

## Disambiguation (read first)

**The ecosystem moat is the third-party developer plugin/app ecosystem, NOT a commerce marketplace.** Three distinct concerns, three distinct µservices, three distinct futures. This ADR fixes the architecture for #1 only; #2 and #3 are scoped here only to prevent name collisions.

| # | µservice | Concern | Comparable to | Status |
|---|---|---|---|---|
| 1 | `microservices/plugin-app-store/` | Third-party developer plugins/apps catalog: discovery + install + lifecycle + vetting + per-plugin billing aggregation + per-plugin Cedar permissions. The plugin ecosystem moat. | Apple App Store + VS Code Marketplace + AWS Marketplace (SaaS catalog) + JetBrains Marketplace + Shopify App Store + Salesforce AppExchange | **This ADR — being scaffolded now (M02b)** |
| 2 | `microservices/marketplace/` (FUTURE) | B2C / B2B commerce surface for buying-and-selling physical or digital goods + services across tenants. Cart, checkout, inventory, fulfillment, returns. The commerce-vertical product. | Amazon Marketplace + Shopify storefront + Etsy + eBay | **NOT in scope of ADR-0213**; future ADR + future µservice. The name `marketplace` is reserved for this concern and must NOT be used for plugin distribution. |
| 3 | `microservices/community/` | Social network + job search + recruiter tooling (existing seed; future major expansion). | LinkedIn + Handshake + TeamBlind + Reddit | **Existing µservice**, future major expansion scoped by separate ADR. Out of scope of ADR-0213. |

**Naming rule (enforced by `oya gate validate naming-justification`):** the substring `marketplace` in oyatie µservice names is reserved for concern #2 (B2C commerce). Plugin distribution uses `plugin-app-store`; developer surface uses `developer-sdk`. This ADR registers the naming reservation.

**Why two µservices for concern #1 (per ADR-0132 no-grouping policy):**
- `plugin-app-store` serves tenant-operators + tenant-admins + platform-admins (the consumer + governance personas of the ecosystem).
- `developer-sdk` serves third-party developers (the supply-side persona of the ecosystem).
These are two distinct personas with two distinct rate-limit profiles, two distinct authorization surfaces, and two distinct release cadences. ADR-0132 forbids bundling them.

## Context

Bominal-inherited ADR-0037 ("plugin substrate") shipped a thin plugin-loader concept under a single shared substrate µservice. That model has structural limits:

1. **Plugin loader ≠ plugin ecosystem.** A "plugin loader" implies a load-once code extension. In 2026 the load-bearing competitive surface is a **third-party developer ecosystem** with marketplace-class UX: VS Code Marketplace (~50k extensions), Apple App Store (~1.8M apps), JetBrains Marketplace (~5k plugins), Shopify App Store (~13k apps), AWS Marketplace (~15k catalog items), Salesforce AppExchange (~8k apps), WeChat Mini-Programs (~3.6M). Every hyperscaler-class platform of the 2020s ships a plugin-ecosystem surface.
2. **ADR-0037 has no developer-facing product surface.** No public SDK, no contracts registry, no sandbox env, no developer onboarding, no revenue share. ADR-0037's load-bearing assumption was that third-party developers are an internal concern, not a tenant-facing product persona.
3. **One-bundle-µservice violates ADR-0132 (no-grouping policy).** A single `plugin-substrate` µservice that does discovery + lifecycle + vetting + signing + sandbox + billing + SDK + portal violates the single-concern rule.
4. **VS Code / App Store / Stripe parity requires per-plugin billing aggregation.** Tenant pays one consolidated invoice; revenue share routes per-plugin; per-plugin subscription state machine. ADR-0037 had no billing seam.
5. **Per-plugin permissions are the security load-bearing surface.** Each plugin declares which capabilities it consumes (read PII, write to Workflow Engine, call AI features). Tenant grants per-plugin at install time. Apple App Store has had this since iOS 6 (2012); VS Code added explicit `Permissions` in 2024. ADR-0037 had no per-plugin permission model.

User directive 2026-05-18 made the decision explicit: **Direction C — Explicit Ecosystem-as-a-Service**. Ship two distinct µservices, one for the consumer + governance surface (the **plugin/app store**) and one for the developer surface (the **SDK**), each single-concern per ADR-0132. Treat third-party developers as a first-class tenant persona, with the same audit-grade rigor every other oyatie µservice receives.

A subsequent user directive 2026-05-18 corrected the naming: this is a **plugin/app store** (Apple App Store / VS Code Marketplace shape), not a commerce **marketplace**. The substring `marketplace` is reserved for the future B2C commerce µservice per the §Disambiguation table above.

## Decision

Oyatie ships an **Ecosystem-as-a-Service** product surface, composed of **two single-concern µservices** under the ADR-0131 flat layout, citing the industry inheritances listed in the frontmatter:

### 1. Two µservices, single-concern each (per ADR-0132)

| µservice | Concern | Persona served | Inheritance |
|---|---|---|---|
| `microservices/plugin-app-store/` | Consumer-facing plugin/app discovery + install + per-plugin permission grant + subscription mgmt + audit; admin-facing vetting queue + revocation | tenant-operator, tenant-admin, platform-admin | Apple App Store + VS Code Marketplace + AWS Marketplace + Shopify App Store |
| `microservices/developer-sdk/` | Developer-facing SDK distribution + API contracts (OpenAPI 3.2 + AsyncAPI 3.1 + proto3) + sandbox env + dev portal + signing-key issuance + Stripe-Connect-style onboarding + revenue payout | 3rd-party developer | Stripe (developer onboarding + payout) + Apple Developer Program (signing keys + sandbox) + first-party portal composition (ADR-0394) |

Both µservices share **no code** with each other beyond canonical contracts (event schemas, Cedar policy fragments). They communicate exclusively via the Workflow + Ontology adapter layer per `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145).md`. No `ecosystem` super-bundle; ADR-0132 forbids it.

### 2. Plugin/app lifecycle (state machine)

Every plugin/app has a **version-level state machine** parallel to ADR-0110 ChangeSet:

```
draft → submitted → vetting → (approved | rejected)
                              ↓
                        published → deprecated → retired
                              ↓
                          revoked  ← (kill-switch from admin)
```

States enforced by:
- `microservices/plugin-app-store/src/lifecycle/` (lifecycle kernel + domain + usecase).
- Cedar policy `microservices/plugin-app-store/policy/plugin-lifecycle-scope.cedar`.
- Audit chain seal events: `oya.plugin-app-store.plugin-submitted`, `plugin-vetting-passed`, `plugin-published`, `plugin-revoked`, etc.

### 3. Vetting pipeline (security + privacy + accessibility)

Every submitted plugin version transits a vetting pipeline before publish:

| Stage | What is checked | Tool / authority | Inheritance |
|---|---|---|---|
| signature-verification | Cosign-signed artifact present and trusted | Cosign (per ADR-0181) | Apple notarisation + VS Code VSIX signing |
| vulnerability-scan | Wasmtime artifact + npm/cargo manifest scanned | Trivy + cargo-audit | App Store security review |
| sandbox-isolation-validation | Artifact runs in Wasmtime per ADR-0147 + ADR-0200; no syscall escapes | Wasmtime + seccomp profile | Apple sandbox + iOS entitlements |
| capability-scope-validation | Declared capabilities match Cedar policy fragment | `oya gate validate cedar-policy --microservice plugin-app-store` | Apple capability declarations + Shopify scopes + VS Code Permissions |
| data-use-boundary-check | Declared data classes within tenant data-use boundary (ADR-0008) | `oya gate validate data-use-boundary` | Apple privacy nutrition label |
| accessibility-conformance | WCAG 2.2 AA per ADR-0207 | axe + pa11y on plugin's tenant-facing UI surface | Apple accessibility review |
| ai-act-classification | If plugin uses AI capabilities, EU AI Act risk class declared | `eu_ai_act_risk_class` in plugin manifest | EU AI Act compliance |
| performance-budget | Cold-start ≤ 300ms p99; steady-state CPU ≤ 100m; memory ≤ 128Mi | Wasmtime metrics | App Store perf review |

Pipeline implemented as deterministic stages in `microservices/plugin-app-store/src/vetting/`. A submitted version that fails any stage transitions to `rejected` with a structured rejection reason emitted to the developer-sdk dev portal.

### 4. Sandbox runtime (per-plugin isolation)

Every plugin runs in a per-installation Wasmtime sandbox per ADR-0147 + ADR-0200:
- **One Wasmtime engine per tenant-plugin installation.** No shared state between tenants. No shared state between plugins within a tenant beyond explicit declared scopes.
- **Per-plugin rate limits.** Default 100 req/s; per-plugin override via plugin manifest with admin approval at install time.
- **Per-plugin OpenSLO.** Plugin declares its self-SLOs at submission; rendered into a per-installation OpenSLO manifest under `microservices/plugin-app-store/slos/installations/<tenant_id>/<plugin_id>.openslo.yaml`.
- **Per-plugin Cedar permissions.** Declared capabilities are translated to Cedar policy at install time; runtime authorization is via the central Cedar evaluator (per ADR-0007).
- **Per-plugin audit trail.** Every action the plugin takes against tenant data emits a seal event consumed by the audit-chain µservice (ADR-0003).

### 5. Revenue model — Stripe parity, in-house

Three monetisation paths per plugin:
- **Free (open-source).** Developer ships free plugin; oyatie covers infra cost from tenant base subscription. No revenue share.
- **One-time purchase.** Tenant pays once per install; oyatie deducts platform fee (30% standard, 15% after $1M lifetime per Apple model) and routes the remainder to developer payout balance.
- **Recurring subscription.** Per-seat or flat monthly; identical fee structure to one-time; renewals managed by plugin-app-store subscription-billing aggregator.

Revenue substrate is **100% in-house** per ADR-0211 §revenue-share:
- KYC + AML on developer onboarding implemented under `microservices/developer-sdk/src/onboarding/` (citing FATF + EU AML5 + US BSA — no third-party KYC SaaS).
- Payout ledger implemented under `microservices/developer-sdk/src/payout/`; daily settlement to developer-declared bank account via ACH (US) + SEPA (EU) + KFTC (KR) + FedWire (US) per pack.
- Tax-form generation (1099-MISC for US, EU VAT MOSS, KR VAT) emitted under `microservices/developer-sdk/src/tax/`.
- Forex / settlement currency conversion via Oya-internal rate-lock per ADR-0199.

### 6. SDK families (codegen from OpenAPI 3.2)

Per ADR-0185 OpenAPI 3.2 codegen, developer-sdk emits client SDKs for six stacks at GA:
- TypeScript / JavaScript — Node 22 LTS + browser ESM.
- Rust — `cargo` crate; targets stable + WASM.
- Swift — SPM package; iOS 17+ / macOS 14+.
- Kotlin — Gradle module; JVM + Android.
- C# — NuGet package; .NET 8 LTS.
- Python — PyPI package; 3.12+.

Each SDK is generated from the canonical OpenAPI 3.2 spec under `microservices/developer-sdk/contracts/openapi/oya-ecosystem.yaml` plus AsyncAPI 3.1 contracts for event subscriptions. Codegen pipeline lives under `microservices/developer-sdk/src/codegen/`; nightly CI publishes new versions to Oya-internal package registry under `microservices/developer-sdk/iac/registry/`.

### 7. Sandbox environment (developer-facing)

Every developer with a verified account gets an isolated sandbox tenant:
- Provisioned via `microservices/developer-sdk/src/sandbox-provisioner/`; calls `microservices/tenancy/` to create a sandbox-class tenant per pack.
- Reset-on-demand via dev portal button (`POST /v1/sandbox/reset`); reset is a full tear-and-rebuild within ≤ 30s.
- Synthetic data seeded from the public-data pack templates (`docs/templates/sandbox-fixtures.json`).
- Sandbox tenants billed to a shared developer-bench cost-center per ADR-0199; developers do not pay.

### 8. Developer portal (first-party module)

Per ADR-0394, developer workflows are modules in the first-party Rust portal composition under
`app/ops-console/developer-portal/`. The developer-sdk capability owns the APIs and domain rules;
the portal owns presentation and composition only:
- API contracts browseable through the first-party documentation module.
- "Try in sandbox" interactive widget for every endpoint.
- Per-plugin metrics: install count, revenue-to-date, vetting-queue position, SLO compliance.
- Submission workflow: developer authors plugin manifest → uploads Wasmtime artifact → portal initiates vetting pipeline → status streamed.
- API key + signing key management via OpenBao (per ADR-0211).

Backstage may be used as a feature reference or bounded one-way import source. It is not a runtime,
plugin host, deployment substrate, or catalog authority.

### 9. Per-plugin SLO publishing

Each published plugin version ships an OpenSLO manifest validating its declared capabilities:
- Latency SLO per declared API surface (p95 + p99 targets).
- Error-rate SLO (max 1% errors per declared capability).
- Availability SLO (min 99.5% for free plugins; 99.9% for paid plugins; verified by plugin-app-store's promotion gate per ADR-0139).
- SLO breach → automatic install suspension on the offending plugin version; tenant operators notified.

### 10. Discovery + install UX

Per `feedback_workflow_studio_scope.md` design language consistency, the plugin-app-store tenant-facing UI is built on the same Leptos + design-system primitives as Workflow Studio:
- Category-tree browse + full-text search.
- Plugin detail page: screenshots, permissions requested, data-use boundary, pricing, ratings, vetting badge.
- One-click install with explicit permission grant UI (Apple-style modal listing every declared capability).
- Post-install configuration panel.
- Subscription management: pause, resume, cancel, change tier.

## Alternatives considered

### Alternative A — Extend ADR-0037 plugin-substrate (REJECTED)

Reject because:
- ADR-0037 has no developer-facing product surface; bolting it on creates a bundle µservice forbidden by ADR-0132.
- "Plugin loader" framing under-sells the ambition; tenant operators in 2026 expect App-Store-class experience, not VSCode-extension-1.0-era experience.
- No revenue share design; would need a separate µservice anyway.

### Alternative B — Use AWS Marketplace SaaS Contracts as backing substrate (REJECTED)

Reject because:
- ADR-0211 (in-house tech policy) forbids external marketplace SaaS as backing substrate.
- AWS Marketplace optimises for AWS-billed customers; oyatie tenants pay oyatie, not AWS.
- Revenue share mechanics on AWS Marketplace SaaS Contracts route through AWS — gives AWS visibility into per-tenant revenue.

### Alternative C — Direction C: Explicit Ecosystem-as-a-Service (CHOSEN)

The user directive 2026-05-18 selected this option. Decision-record path: founder explicitly confirmed "Similar to WeChat super-app + AWS service catalog + Apple App Store + Stripe Connect."

A follow-up user directive 2026-05-18 corrected the naming: `marketplace` is reserved for B2C commerce (concern #2 in §Disambiguation); plugin distribution uses `plugin-app-store`.

### Alternative D — Three µservices: plugin-app-store + sdk + signing-substrate (REJECTED)

Reject because:
- Signing-key issuance is part of the developer surface; splitting it creates an extra inter-service hop with no benefit.
- ADR-0132 disfavors fragmentation; two single-concern µservices is the minimum coherent split.

### Alternative E — One µservice named `ecosystem-marketplace` (REJECTED)

Reject because:
- `marketplace` carries B2C commerce semantics in oyatie naming (Amazon/Shopify class); using it for plugin distribution conflates two distinct concerns and pre-empts a future µservice's name.
- The chosen name `plugin-app-store` aligns with Apple App Store + VS Code Marketplace shape and is unambiguous.
- A single µservice violates ADR-0132 no-grouping policy.

## Consequences

### Positive

- **Two µservices ship under ADR-0131 flat layout** with audit-grade rigor; no exception fences for ecosystem code.
- **Hero-product axis confirmed.** EaaS becomes the seventh axis under ADR-0001 cohesion thesis (cf. "third-party-extensibility" axis previously implicit under "platform").
- **Revenue-share substrate available to other µservices.** developer-sdk's payout substrate is reusable by any future µservice that needs to pay external creators (e.g., AI-prompt store, template store).
- **Per-plugin permission model unblocks tenant-trust messaging.** Tenants see per-plugin capability declarations identical to Apple App Store privacy nutrition label.
- **Vetting pipeline is auditable.** Every transition is sealed; council-security has a forensic trail of every vetting decision.
- **Naming reservation prevents future conflict.** The `marketplace` name remains available for the future B2C commerce µservice (concern #2).

### Negative

- **Two more µservices to maintain** (currently 41; this brings the count to 43). Mitigated by ADR-0131 flat layout requiring zero per-µservice CI bespoke wiring.
- **Vetting pipeline introduces submission latency.** Trade-off accepted: security ≫ developer convenience; vetting SLA target ≤ 5 business days (faster than Apple's typical ≤ 14 days).
- **Revenue-share + payout substrate is high regulatory burden.** Inheriting FATF + EU AML5 + US BSA + KR FSS + KFTC ACH compliance. Mitigated by phased pack rollout: pack-us first (BSA + 1099-MISC), pack-eu second (AML5 + VAT MOSS), pack-kr third (FSS + KFTC), others follow.
- **Per-plugin sandbox cost.** Wasmtime per-installation engine consumes resources. Mitigated by cold-start optimisation per ADR-0147 + opportunistic engine teardown after 60s idle.

### Neutral

- Existing `oya-foundry-*` plugin code remains; ADR-0037 is **superseded** for new work, but existing in-flight plugin work continues until each crate is migrated under one of the two µservices in its own follow-up ADR.

## Phased rollout (master-plan-sequencing-aligned)

| Phase | Milestone | What ships | Gate |
|---|---|---|---|
| Phase 0 (this ADR) | M02b | ADR accepted; µservice scaffolds + manifests + IPs authored | scaffold-pr merged |
| Phase 1 | M04-ecosystem-substrate | discovery + install + revoke; pack-us only; single capability tier; OSS-only plugins | preview |
| Phase 2 | M05-ecosystem-paid | one-time + recurring billing; revenue share; pack-eu + pack-us packs | stable |
| Phase 3 | M06-ecosystem-developer-portal | first-party portal modules; SDK families published; sandbox env | GA |
| Phase 4 | M07-ecosystem-vetting-acceleration | parallel vetting workers; SLA ≤ 1 business day; automated AI-Act class detection | GA-evolved |

Each phase's gate is in `/specs/master-plan-sequencing.json§ecosystem-as-a-service` (to be added by a follow-up CR).

## Hyperscaler-invariant coverage

Per `ADR-0123 hyperscaler-maturity-claim-gate`, both µservices register hyperscaler-gate evidence at scaffold time:

- **HG-PAS (plugin-app-store)** — INV-CIRCUIT-BREAKER (per-plugin circuit on failing plugin), INV-SHUFFLE-SHARDING (per-tenant plugin-app-store shard), INV-FOUR-GOLDEN-SIGNALS (per-plugin latency / errors / saturation / traffic), INV-SLO-ERROR-BUDGET (vetting-pipeline + install-flow burn-rate).
- **HG-SDK (developer-sdk)** — INV-CIRCUIT-BREAKER (per-developer rate-limit), INV-SHUFFLE-SHARDING (per-developer sandbox cell), INV-FOUR-GOLDEN-SIGNALS (codegen + portal + payout planes), INV-SLO-ERROR-BUDGET (payout settlement burn-rate, sandbox cold-start burn-rate).

Both µservices ship the canonical PrometheusRule wired to the observability µservice substrate per ADR-0210 (OTEL tail sampling).

## In-house roadmap (per ADR-0211)

EaaS is **100% in-house from day one** per ADR-0211 in-house tech policy. The roadmap forbids reliance on:
- External developer-portal SaaS and third-party portal runtimes (readme.io, stoplight, gitbook,
  Backstage) — replaced by the first-party Rust portal per ADR-0394.
- External package-registry SaaS (npmjs.com, crates.io) for first-party SDK distribution — vendored in-house under `microservices/developer-sdk/iac/registry/`.
- External KYC / AML SaaS (Onfido, Stripe Identity) — implemented in-house under `microservices/developer-sdk/src/onboarding/kyc/`.
- External payout SaaS (Tipalti, Stripe hosted, Wise hosted) — implemented in-house with direct ACH / SEPA / KFTC integration under `microservices/developer-sdk/src/payout/`.
- External vetting SaaS (Veracode, Snyk hosted) — Trivy + cargo-audit + cosign-verify chained in-house under `microservices/plugin-app-store/src/vetting/`.
- External plugin-store SaaS (e.g., GitHub Marketplace) — `microservices/plugin-app-store/` is the plugin store; no external dependency.

This is the WeChat / Apple posture: the platform owner runs the platform; third-parties produce plugins **on the platform**; the platform never delegates platform mechanics to a vendor.

## Out-of-scope (explicit non-decisions for this ADR)

- **Specific developer revenue-share percentages.** Deferred to founder + axis-finops council in a follow-up CR; ADR-0213 fixes only the architecture, not the commercial terms.
- **Specific vetting SLA targets.** Deferred to council-security in a follow-up runbook; current ADR commits only to a ≤ 5-business-day Phase-1 target.
- **AI plugin subcategory taxonomy.** Owned by axis-ai-features after the AI capability registry hardens; tracked as a post-registry taxonomy decision.
- **Tenant-developer accounts (tenant operator who also publishes plugins).** Both µservices anticipate this persona but the dual-role onboarding flow is scoped to a follow-up CR after Phase 3 GA.
- **The `marketplace` µservice (B2C commerce, concern #2 in §Disambiguation).** Out of scope; future ADR.
- **`community` µservice major expansion (LinkedIn + Handshake + TeamBlind + Reddit, concern #3 in §Disambiguation).** Out of scope; future ADR.

## Verification

This ADR is verified by:
- `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice plugin-app-store` exits 0.
- `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice developer-sdk` exits 0.
- `cargo run -p oya-dev-cli -- gate validate authority-cohesion` exits 0 with both µservices registered.
- `cargo run -p oya-dev-cli -- gate validate naming-justification` exits 0 confirming `marketplace` name reservation.
- Both µservices' manifest.json present, parseable, schema-valid against `/specs/per-microservice-flat-layout.json` substrate schema.
- ADR registered in `docs/decisions/README.md` next index slot.

## References

- WeChat ecosystem reference — Tencent platform blog 2023 disclosures (Mini-Program platform architecture).
- Apple App Store Review Guidelines — developer.apple.com/app-store/review/guidelines (vetting + sandbox + entitlements).
- VS Code Marketplace publisher docs — code.visualstudio.com/api/working-with-extensions/publishing-extension (VSIX signing + permissions).
- Stripe platform documentation — stripe.com/docs/connect (KYC + payout + revenue split).
- Shopify App Store partner program — shopify.dev/docs/apps (per-install model + scopes).
- AWS Marketplace seller guide — docs.aws.amazon.com/marketplace (vetted catalog + per-account procurement).
- Salesforce AppExchange Security Review — partners.salesforce.com/partnerresource/security (enterprise vetting).
- JetBrains Marketplace plugin distribution — plugins.jetbrains.com/docs/marketplace (signed plugins + paid plugins + revenue share).
- Bominal ADR-0037 (plugin substrate) — inherited verbatim, superseded for new work by ADR-0213.
- ADR-0131 (per-microservice flat layout) — substrate authority.
- ADR-0132 (no-grouping policy) — two single-concern µservices, not one bundle.
- Memory: `feedback_quality_performance_scalability_bar.md` (hyperscaler-grade); `feedback_workflow_objectgraph_adapter_layer.md` (Workflow + Ontology adapter routing); `feedback_canonical_base_localization.md` (per-pack overlays).
