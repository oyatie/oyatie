---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-developer-sdk
microservice: developer-sdk
status: Accepted
sales_segment: hero-product
tier: external-facing
milestone_first_ship: M06-ecosystem-developer-portal
bominal_source:
  - ADR-0037   # Plugin substrate (developer surface absent; ADR-0213 supersedes)
related_adrs: [ADR-0056, ADR-0065, ADR-0105, ADR-0110, ADR-0123, ADR-0131, ADR-0132, ADR-0139, ADR-0170, ADR-0185, ADR-0199, ADR-0211, ADR-0212, ADR-0213, ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345]
related_specs: [/specs/microservices/developer-sdk.json, /specs/per-microservice-flat-layout.json]
related_unbundle_adr: ADR-0213
unbundle_sibling: microservices/plugin-app-store/
date: 2026-05-18
owner_team: axis-ecosystem + council-architecture
doc_status: published
---

# PRD-developer-sdk: Developer SDK — third-party developer onboarding + contracts + sandbox + portal + payout

## Purpose

The `developer-sdk` µservice is oyatie's **developer-facing surface** of the Ecosystem-as-a-Service product per ADR-0213. The sibling `plugin-app-store` µservice owns the tenant + admin surface (catalog + install + vetting). This µservice owns: third-party developer onboarding (Stripe-Connect-style KYC + AML), per-developer ED25519 signing key issuance via OpenBao, the canonical OpenAPI 3.2 + AsyncAPI 3.1 + proto3 API contracts registry, the codegen pipeline producing six SDK family clients (TS/Rust/Swift/Kotlin/C#/Python), per-developer on-demand sandbox tenants, the Backstage-extension developer portal (API browser + try-in-sandbox + plugin metrics), the daily revenue-share payout substrate (ACH/SEPA/KFTC/FedWire), and tax-form generation (1099-MISC/VAT-MOSS/KR-VAT).

This µservice is **the supply-side persona platform** of the ecosystem: developers come here to author, distribute, and monetize plugins on oyatie. Without this µservice, the plugin-app-store has no plugins.

## Tenant Value (developer-as-tenant framing)

- **Developer Outcome 1 — Onboarding in 30 minutes.** Sign up, complete KYC + bank account verification, get signing keys issued, get sandbox tenant provisioned. Stripe-Connect parity.
- **Developer Outcome 2 — Get first plugin to vetting in 1 day.** SDK in their language, OpenAPI contracts published, sandbox tenant for development, dev portal for submission. JetBrains Marketplace + Apple App Store parity.
- **Developer Outcome 3 — Six native SDK families.** No need to use a generic HTTP client; idiomatic SDK in their stack. TS / Rust / Swift / Kotlin / C# / Python at GA; more on roadmap.
- **Developer Outcome 4 — Reset-on-demand sandbox.** Every developer gets a sandbox tenant with synthetic data; reset within 30s to a known clean state. Apple Developer Program parity.
- **Developer Outcome 5 — Transparent revenue share.** Per-plugin install + subscription + revenue ledger; daily payout to bank account in their pack (ACH US, SEPA EU, KFTC KR, FedWire US-wire). Stripe Connect parity.
- **Developer Outcome 6 — Tax forms in-app.** 1099-MISC (US), EU VAT MOSS, KR VAT generated automatically; download in dev portal at tax season.
- **Developer Outcome 7 — Vetting status streamed.** Submitted plugin's vetting progress is visible per stage; rejection reasons structured + actionable; no opacity.
- **Internal Outcome 8 — Stripe-Connect-parity payout substrate.** developer-sdk's payout substrate is reusable by any future µservice paying external creators (AI prompt store, template store, plugin store).

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | new developer | to sign up with email + company info | I start the onboarding flow | developer-onboarding | Must |
| FR-02 | developer | to complete KYC (ID upload + selfie + liveness check) | I'm verified to publish + receive payouts | developer-onboarding | Must |
| FR-03 | developer | to add a bank account (US ACH / EU SEPA / KR KFTC) | I can receive payouts | developer-onboarding + payout | Must |
| FR-04 | developer | to receive an ED25519 signing key issued via OpenBao | I can Cosign my plugin artifacts | signing-key-issuance | Must |
| FR-05 | developer | to rotate or revoke my signing keys | key compromise is recoverable | signing-key-issuance | Must |
| FR-06 | developer | to browse the canonical OpenAPI / AsyncAPI / proto contracts | I know what to integrate against | api-contracts-registry + dev-portal | Must |
| FR-07 | developer | to download SDK in my language (TS / Rust / Swift / Kotlin / C# / Python) | I write code in my stack | sdk-codegen + dev-portal | Must |
| FR-08 | developer | to get a sandbox tenant provisioned within 60s | I can develop + test | sandbox-provisioner | Must |
| FR-09 | developer | to reset my sandbox tenant within 30s | I can restart from a clean state | sandbox-provisioner | Must |
| FR-10 | developer | to use a "try in sandbox" widget for each API endpoint in the dev portal | I learn the API interactively | dev-portal | Should |
| FR-11 | developer | to submit a plugin version with manifest + Wasmtime artifact | I publish to the plugin-app-store catalog | dev-portal (cross-µservice call to plugin-app-store) | Must |
| FR-12 | developer | to see vetting pipeline status streamed per stage | I track my submission | dev-portal | Must |
| FR-13 | developer | to view per-plugin metrics: install count, revenue, ratings, SLO compliance | I run my plugin as a business | dev-portal | Must |
| FR-14 | payout substrate | to settle developer balances daily via ACH / SEPA / KFTC / FedWire | developers receive payouts on schedule | payout | Must |
| FR-15 | developer | to download tax forms (1099-MISC / VAT MOSS / KR VAT) in-app at year end | I file taxes correctly | tax-form | Must |
| FR-16 | dev-portal | to surface per-developer aggregate dashboard | I see overall business health | dev-portal | Should |
| FR-17 | platform admin | to revoke a developer (e.g., for ToS violation) | bad actors are kicked out + plugins revoked | developer-onboarding (cross-µservice signal to plugin-app-store) | Must |

## Non-functional Requirements

### Performance
- Onboarding flow each step ≤ 500ms p95.
- KYC verification result ≤ 60s p99 (async; via in-house KYC pipeline).
- Signing-key issuance ≤ 1s p99 (OpenBao adapter).
- Sandbox provision ≤ 60s p99 (tenancy adapter + synthetic seed).
- Sandbox reset ≤ 30s p99 (tear + rebuild).
- Codegen full SDK family emit ≤ 10 min p99 (nightly batch).
- Portal page load p95 ≤ 500ms (Backstage SSR + caching).
- Payout settlement batch p99 ≤ 4h (daily window).

### Availability
- Onboarding flow: 99.9%.
- Signing-key issuance: 99.99% (load-bearing for plugin publish).
- Dev portal: 99.9% (developers tolerate brief outages; non-tenant-impacting).
- Sandbox provisioner: 99.5% (degrades to retry).
- Payout substrate: 99.95% (regulatory load-bearing).

### Scalability
- 100k registered developers at GA; 1M at hyperscaler tier.
- 10k concurrent sandbox tenants.
- 10k plugin submissions per day at GA.
- 1M daily payout settlements at hyperscaler tier (mostly micro-payouts).

### Security
- KYC: in-house ID + selfie + liveness pipeline; no external SaaS per ADR-0211.
- AML: in-house OFAC / EU sanctions / FATF list lookup; daily refresh.
- Signing keys: ED25519; OpenBao-managed; 30-day expiry default; auto-rotate.
- Payout: dual-signature for amounts > $10k; manual review queue for amounts > $50k.
- Tax forms: PII redacted in transport; encrypted at rest.

### Compliance
- US BSA + 1099-MISC (W-9 / W-8 capture on onboarding).
- EU AML5 + VAT MOSS + GDPR Article 28.
- KR FSS + KR VAT + KR cross-border data transfer compliance.
- FATF sanctions + OFAC SDN list daily check.
- Per-pack regulatory overlay: developer in pack-kr binds to KR-FSS terms; developer in pack-eu to AML5 terms.

### Cost
- Per-developer onboarding cost: ~$5 (KYC + AML + bank verification micropayments).
- Sandbox tenant cost: charged to oyatie developer-bench cost-center per ADR-0199; developers do not pay.
- Codegen pipeline: nightly batch cost amortized across all SDK families.

### DR posture (ADR-0343)

- RTO/RPO target: manifest-declared RTO p99 900s and RPO p99 60s for developer onboarding, signing-key issuance metadata, contracts registry, sandbox metadata, payout ledgers, and tax-form state. Applicable floors are SOC2-T2 14400s/900s, ISO27001-2022 14400s/3600s, and KR-PIPA 14400s/900s; the manifest target is stricter because signing keys and payouts are platform load-bearing.
- Failover reference: manifest `failover_runbook` is `runbooks/dr-failover.md`; supporting drills remain `microservices/developer-sdk/multi-region.md`, `runbooks/signing-key-issuance-timeout.md`, `runbooks/sandbox-provision-slow.md`, and `runbooks/payout-settlement-mismatch.md`.
- Multi-region active-active posture: true per manifest; replication shape is `active-active-multi-az-cross-region-warm` across `postgres_wal_g`, versioned object storage, OpenBao seal/unseal, audit-chain Merkle seals, and Valkey.
- Tenant-visible behavior: developers can keep reading contracts and SDK documentation during a regional event, while onboarding, key issuance, and payout changes pause or retry rather than creating duplicate legal or bank state.

### Capacity model (ADR-0340)

- Per-tenant baseline: manifest-declared 0.30 vCPU, 768 MiB RAM, 5 GB storage, three Postgres connections, two Valkey connections, and ten outbound HTTP connections.
- Scaling dimension: `per_workflow_run` per manifest for codegen, sandbox provisioning, signing-key issuance, payout/tax workflows, and developer portal operations, with `per_user` and `per_batch` as secondary portal and codegen dimensions.
- Cell placement class: Tier-2 per manifest for developer portal, onboarding, contracts, sandbox, payout, and tax-form state; pod runtime tier is 2, with Tier-0 isolation delegated to plugin-app-store for untrusted plugin execution.
- Autoscaling boundaries: portal and onboarding APIs scale 3-80 replicas, sandbox provisioners 2-40, codegen workers 2-30, and payout/tax workers 2-20 with queue admission by pack and settlement deadline.
- Tenant load profile: supports 100k registered developers, 10k concurrent sandbox tenants, 10k submissions per day, and 1M daily payout settlements without letting codegen batches delay signing-key issuance.

### Sustainability and cost attribution (ADR-0344)

- Per-call emission claim: onboarding, KYC decision, signing-key issuance, SDK codegen, sandbox provision/reset, payout settlement, and tax-form audit rows emit `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with developer tenant, provider, cell, capability, and compliance_pack axes.
- Carbon-aware provider routing: yes for nightly codegen, package replication, and tax-form batch work where the jurisdictional deadline allows; no for signing-key issuance, KYC legal checkpoints, payout settlement windows, or sanctions checks.
- Tenant transparency surface: developer portal and finops-portal expose sandbox, codegen, payout, and package-publication cost lines, separating oyatie-funded developer-bench costs from billable ecosystem activity.
- Regulatory driver: CSRD, SB-253, and SEC climate disclosure reporting require developer ecosystem and payout processing emissions to be retained with the financial evidence, not estimated from batch totals later.

### API versioning posture (ADR-0342)

- Public API version model: `YYYY-MM-DD` carrier triplet across version header, URL prefix, and proto3 field for onboarding, contracts registry, SDK download, sandbox, payout, and tax-form APIs.
- SDK semver model: all generated SDK families use `major.minor.patch`; a public date-version break is the only reason to bump major.
- Support window: last 3 public versions are supported for at least 180 days.
- Per-tenant pinning: yes for developer accounts, sandbox tenants, generated SDK families, and plugin submission contracts.
- Internal-mesh exemption: yes; ADR-0145 direct gRPC remains valid for internal codegen, payout, and sandbox coordination.

## Acceptance Criteria (AC)

| ID | Criterion | Test |
|---|---|---|
| AC-01 | Onboarding completion rate ≥ 70% from sign-up to KYC-verified within 30 min | funnel measurement |
| AC-02 | KYC false-positive rate ≤ 2% (legitimate developers rejected) | weekly review |
| AC-03 | Signing-key issuance ≤ 1s p99 | bench test |
| AC-04 | Sandbox provision ≤ 60s p99 | integration test |
| AC-05 | Sandbox reset ≤ 30s p99 | integration test |
| AC-06 | All six SDK families generated from canonical OpenAPI contracts byte-deterministically | replay test (regenerate twice → diff = 0) |
| AC-07 | Dev portal page load p95 ≤ 500ms | k6 load test |
| AC-08 | Payout settlement correctness: 100% match between ledger total and bank-side total | daily reconciliation |
| AC-09 | Tax form 1099-MISC correctness: 100% match with IRS spec | annual audit |
| AC-10 | Vetting status streamed to dev portal with ≤ 1s lag | integration test |
| AC-11 | Per-plugin metrics accurate to plugin-app-store source-of-truth ≤ 60s lag | integration test |
| AC-12 | Developer revocation cascades to plugin-app-store within 30s p99 (all plugins of revoked dev → revoked) | scripted e2e drill |
| AC-13 | Per-pack regulatory overlay applies (developer in pack-kr cannot opt out of KR-FSS terms) | integration test per pack |
| AC-14 | Deterministic-replay: payout settlement reproduces byte-equally on identical input | replay test |

## Bounded Contexts

### developer-onboarding
- KYC + AML state machine; W-9 / W-8 / sanctions list verification; bank account verification.
- Storage: Postgres event-sourced onboarding log.

### signing-key-issuance
- OpenBao-backed ED25519 issuance + rotation + revocation.
- No durable storage in µservice; OpenBao authoritative.

### api-contracts-registry
- Version-locked OpenAPI 3.2 + AsyncAPI 3.1 + proto3 definitions consumed by codegen + portal.
- Storage: Git-managed contracts repository; cached in Postgres.

### sdk-codegen
- Six SDK family generators (TS/Rust/Swift/Kotlin/C#/Python); deterministic codegen.
- Output: oyatie in-house package registry per ADR-0211.

### sandbox-provisioner
- Per-developer sandbox tenant via tenancy µservice; reset within 30s; synthetic data seed.
- Storage: Postgres for sandbox metadata; tenancy authoritative for tenant state.

### dev-portal
- Backstage extension serving API browser + try-in-sandbox + plugin submission + metrics dashboards.
- Storage: Backstage authoritative; cached projections.

### payout
- Daily settlement batch via ACH (US) / SEPA (EU) / KFTC (KR) / FedWire (US-wire).
- Storage: Postgres event-sourced payout ledger; nightly reconciliation against bank statements.

### tax-form
- 1099-MISC (US) / VAT MOSS (EU) / KR VAT generation.
- Storage: PDFs to SeaweedFS; metadata to Postgres.

## Persona Map

| Persona | Surface | Capabilities | Primary BC |
|---|---|---|---|
| new developer | Backstage onboarding wizard | sign up, KYC, bank, ToS | developer-onboarding |
| verified developer | Backstage dev portal | submit plugin, view metrics, manage keys, get payout | dev-portal + signing-key-issuance + payout |
| platform admin | Backstage admin | revoke developer, review KYC queue, review payout exceptions | developer-onboarding + payout |
| tax-compliance reviewer | Backstage admin | export tax forms, review 1099 batch | tax-form |

## Cross-product integration

**Workflow + Ontology routing only.** developer-sdk imports nothing from other product µservices' crates. All cross-product data flow:
- **From / To plugin-app-store**: plugin submission events + vetting status events via event-bus.
- **From tenancy**: sandbox-tenant provisioning.
- **From identity**: developer-principal-id.
- **From cloud-secrets**: OpenBao signing-key issuance.
- **From governance**: Cedar policy evaluation.
- **To audit-chain**: developer-onboarding + payout + signing-key seal events.
- **To finops-portal**: developer-bench cost-center attribution.
- **To ontology**: Developer, DeveloperSigningKey, DeveloperPayoutLedgerEntry, SandboxTenant projections.

## Out of scope

- Plugin runtime execution (owned by plugin-app-store per-installation sandbox).
- Per-tenant plugin install (owned by plugin-app-store plugin-install BC).
- Marketing site (oyatie.dev/developers — owned by content µservice).
- IDE plugin (VS Code extension for oyatie SDK development — future µservice).

## References

- ADR-0213 (Ecosystem-as-a-Service architecture).
- ADR-0170 (Backstage developer portal substrate).
- ADR-0185 (OpenAPI 3.2 codegen).
- ADR-0211 (in-house tech policy).
- Stripe Connect docs — stripe.com/docs/connect.
- Apple Developer Program — developer.apple.com/programs.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `developer-sdk` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `developer-sdk` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 4 module pin(s) across 1 context(s).
- Scaling input: `per_workflow_run` with cell placement `Tier-2` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
