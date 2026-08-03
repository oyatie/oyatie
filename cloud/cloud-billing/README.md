# cloud-billing

The `cloud-billing` microservice is Oyatie's canonical source-of-truth for **commercial state**. It owns the `tenant_class ∈ {demo_trial, paid}` enum, the `billing_components ⊆ {revenue_share, per_seat, per_usage}` composable subset, the metering ledger, the multi-currency invoice ledger, the rate-card lifecycle, the reservation lifecycle, the credit memo ledger, the FX lock service, the revenue-share settlement engine, the per-seat counter, the per-usage aggregator, the FOCUS 1.1 export adapter, the ERP export adapter, the dunning policy, the subscription primitive, and the proration engine.

cloud-billing is the **keystone microservice** for ADR-0330 (tenant class + composable billing components). It is Phase-0 substrate per ADR-0328 §D-1.

## Tenant-class model

The retired capability ladder is not part of this service. `cloud-billing`
uses ADR-0330's canonical `tenant_class` model:

- `demo_trial` tenants run with $0 commercial state, OCI Always Free defaults,
  and explicit time/usage caps.
- `paid` tenants use composable `billing_components`: `revenue_share`,
  `per_seat`, and `per_usage`.
- Capability availability is uniform across tenant classes except where
  demo_trial caps, compliance-pack activation, BYOK, marketplace listing, or
  contractual SLO posture require a Cedar gate.

## Status

| Field | Value |
|---|---|
| Microservice | `cloud-billing` |
| Phase | Phase-0 substrate |
| Owner | axis-cloud-billing + council-finance |
| Sales segment | Shared substrate (consumed by every product) |
| Authority | ADR-0330 keystone |
| Quality bar | Hyperscaler-grade (Stripe + AWS B&CM + Recurly UNION) |
| Tenant classes supported | `demo_trial`, `paid` |
| Deployment contexts | All 6 canonical + OCI Always Free |
| Language | Rust strict (per memory directive) |

## Top-of-mind facts

- The 1,030-line Rust kernel at `crates/oya-cloud-billing-domain/src/lib.rs` is the substance truth.
- The `tenant_class` enum has exactly 2 values: `demo_trial` and `paid`. No third value is permitted without superseding ADR-0330.
- `billing_components` is meaningful only when `tenant_class == paid`. The 8 valid subsets are documented in §7 of `PRD.md`.
- `demo_trial → paid` is a one-way transition. `paid → demo_trial` is forbidden.
- Quality, performance, scalability, security, observability, and accessibility posture are uniform across tenant classes. The only acceptable differences are SLO commitment posture (paid is contractual), cap-hit semantics (demo_trial has caps), and gates on compliance pack / BYOK / marketplace listing.
- cloud-billing is the source-of-truth; cloud-iam reads via the tenant-class-API at principal issuance time.
- Every state mutation is Cedar-gated (per ADR-0243). No inline `if tenant_class == "demo_trial"` guards.

## Quickstart

### Read tenant class

```rust
use oya_cloud_billing_sdk::Client;

let client = Client::new("https://cloud-billing.internal.oyatie.dev:50051")?;
let status = client.get_tenant_class("ten_acme").await?;
println!("tenant_class={:?}, billing_components={:?}", status.tenant_class, status.billing_components);
```

### Emit a usage event

```rust
use oya_cloud_billing_domain::{CloudBillingEventCreate, CloudBillingEventKind};
use data_boundary_kernel::DataClass;

let event = CloudBillingEventCreate {
    id: "cbill_pod_minute_42".to_string(),
    tenant_id: "ten_acme".to_string(),
    resource_id: "oya:cloud:us-east-1:ten_acme:pod:webapp-7d4f".to_string(),
    region: "us-east-1".to_string(),
    metering_tag: "oya:metering:ten_acme:pod".to_string(),
    kind: CloudBillingEventKind::Usage,
    units: vec![/* MeterUnit */],
    rate_card_ref: "rate/us-east-1/cloud-compute-k8s/v1".to_string(),
    occurred_at_epoch_seconds: now_epoch(),
    idempotency_key: "idem_ten_acme_pod_minute_42_2026-05-21T12:00:00Z".to_string(),
    data_class: DataClass::Public,
};
let result = client.emit_usage_event(event).await?;
```

### Convert demo_trial → paid

```rust
let contract_id = "contract_2026_acme_q2";
let billing_components = vec![BillingComponent::PerSeat, BillingComponent::PerUsage];
let result = client.convert_tenant("demo_acme", contract_id, billing_components).await?;
```

### Generate invoice

```rust
let invoice = client.issue_invoice("ten_acme", "2026-04").await?;
```

### Compute revenue-share settlement

```rust
let statement = client.compute_settlement("ten_marketplace_seller", "2026-04").await?;
```

## Architecture at a glance

```
┌─────────────────────────────────────────────────────────────────┐
│                          cloud-billing                          │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Cluster A — Tenant-class state machine                  │  │
│  │   tenant-class-state, billing-components-set,            │  │
│  │   conversion-engine, cap-breach-monitor, grace-window    │  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Cluster B — Metering + ingest                           │  │
│  │   metering-bus, cloud-billing-event-ledger,              │  │
│  │   meter-aggregator, idempotency-dedup                    │  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Cluster C — Invoicing + settlement                      │  │
│  │   invoice-worker, seat-counter, settlement-engine,       │  │
│  │   proration-engine, dunning-policy, credit-memo-issuer,  │  │
│  │   subscription-lifecycle                                  │  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Cluster D — Cross-cutting                               │  │
│  │   fx-lock-service, rate-card-lifecycle,                  │  │
│  │   reservation-lifecycle, focus-export-adapter,           │  │
│  │   erp-export-adapter, audit-chain-emission,              │  │
│  │   attribution-engine, anomaly-detection                  │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
        ↑              ↑              ↑              ↑
        │              │              │              │
   cloud-iam      cloud-billing-tax  payments    audit-chain
```

See `ARCHITECTURE.md` for the full architectural specification.

## Directory layout

```
microservices/cloud-billing/
├── PRD.md                              # Product Requirements
├── ARCHITECTURE.md                     # Architecture specification
├── README.md                           # This file
├── REMEDIATION-NOTES-2026-05-21.md     # Wave 15B remediation notes
├── coherence-audit-2026-05-20.md       # Wave 4-rolling audit (12 P0 findings)
├── feature-parity-matrix-2026-05-20.md # Counterpart parity (existing)
├── performance-benchmark-numbers-2026-05-20.md
├── competitor-parity-matrix.md         # Stripe + AWS B&CM + Recurly UNION
├── supported-oses.json                 # 13 primary OSes + test-only + out-of-scope
│
├── benchmarks/                         # Existing
├── faqs/                               # Existing
├── migration-playbooks/                # Existing
├── onboarding/                         # Existing
├── reference-implementations/          # Existing
├── runbooks/                           # Existing + Wave 15B additions
├── tutorials/                          # Existing
│
├── contracts/                          # NEW — Wave 15B
│   ├── openapi.yaml                    # REST surface (OpenAPI 3.2.0)
│   ├── asyncapi.yaml                   # Event surface (AsyncAPI 3.1.0)
│   └── proto/
│       └── cloud-billing.proto         # gRPC (proto3)
│
├── slos/                               # NEW — Wave 15B (OpenSLO 1.0)
│   ├── invoice-generation-time.openslo.yaml
│   ├── usage-aggregation-time.openslo.yaml
│   ├── seat-counting-availability.openslo.yaml
│   ├── rev-share-settlement-time.openslo.yaml
│   ├── fx-lock-freshness.openslo.yaml
│   ├── tenant-class-read-api-latency.openslo.yaml
│   ├── metering-event-ingest-latency.openslo.yaml
│   ├── audit-chain-seal-latency.openslo.yaml
│   ├── focus-export-completion-time.openslo.yaml
│   └── cap-breach-detection-latency.openslo.yaml
│
├── policies/                           # NEW — Wave 15B (Cedar)
│   ├── cloud-billing.cedar             # Master permits
│   ├── tenant-class-binding.cedar      # tenant_class attribute binding
│   ├── billing-components-gates.cedar  # Per-component gates
│   ├── demo-trial-gates.cedar          # Compliance + BYOK + marketplace
│   ├── settlement-gates.cedar          # Revenue-share settlement
│   └── conversion-gates.cedar          # demo_trial → paid conversion
│
├── decisions/                          # NEW — Wave 15B (per-µservice ADRs)
│   ├── ADR-MS-001-billing-components-composability.md
│   └── ADR-MS-002-revenue-share-settlement-pipeline.md
│
├── implementation-plans/               # NEW — Wave 15B (IPs)
│   ├── IP-001-tenant-class-enum-kernel-extension.md
│   ├── IP-002-billing-components-set-kernel-extension.md
│   ├── IP-003-conversion-engine.md
│   ├── IP-004-cap-breach-monitor.md
│   ├── IP-005-grace-window-state-machine.md
│   ├── IP-006-revenue-share-settlement-engine.md
│   ├── IP-007-per-seat-counter.md
│   ├── IP-008-per-usage-meter-aggregator.md
│   ├── IP-009-subscription-primitive.md
│   ├── IP-010-proration-engine.md
│   ├── IP-011-fx-lock-service.md
│   ├── IP-012-audit-chain-emission-realignment.md
│   ├── IP-013-cedar-policy-authoring.md
│   ├── IP-014-iac-six-contexts.md
│   └── IP-015-os-support-matrix.md
│
└── iac/                                # NEW — Wave 15B (OpenTofu)
    ├── oyatie-public-cloud/
    ├── guest-on-aws/
    ├── guest-on-oci/
    ├── on-prem/
    ├── colo/
    ├── oyatie-as-cloud-provider/
    └── oci-guest/
        └── always-free/                # demo_trial default
```

## Tenant class semantics (TLDR)

### demo_trial

- $0 paid to Oyatie for the trial window.
- Default deployment: OCI Always Free (`iac/oci-guest/always-free/`).
- Time cap: 30 days default.
- Usage caps: per-µservice limits (5 agents, 100 workflows/day, 10 seats, 5 GB store, etc.).
- Same product surface, UX shell, agent dispatch, audit-chain semantics, observability stack as paid tenants.
- Cannot activate compliance packs.
- Cannot opt into BYOK.
- Cannot list on marketplace.
- Cap-breach → 80% warning → 100% Cedar-deny on writes + 7-day grace → suspension → 90-day retention → purge.

### paid

- Executes a commercial contract with Oyatie or an authorized reseller.
- Chooses one of 6 deployment contexts.
- `billing_components ⊆ {revenue_share, per_seat, per_usage}` — 8 valid subsets.
- No default caps; soft-cap configurable.
- Contractual SLO posture.
- May activate any applicable compliance pack.
- May opt into BYOK.
- May list / sell / purchase on marketplace.
- Sub-tenancies supported.

## Billing components (TLDR)

### revenue_share

Marketplace sellers + B2C consumer-product operators + embedded SaaS resellers + affiliate / channel partners.

- Independently activatable; does not require per_seat or per_usage.
- Commission rate per category (per-marketplace-category ADRs pending Wave 15K).
- Monthly settlement.
- FX accounting via fx_lock + settlement-FX-adjustment line item.
- Clawback / chargeback netted in next settlement.
- Direction: `oyatie_pays` (oyatie owes tenant) or `oyatie_collects` (tenant owes oyatie).

### per_seat

B2B enterprise named-user model.

- Independently activatable.
- Seat = 1 named human or 1 named non-human principal.
- Counted by cloud-iam at monthly close.
- 7-day deactivation grace before drop from count.
- Over-seat principals fail-closed via Cedar.
- Monthly cadence; annual prepay supported.
- Multi-tenant users = 1 seat per tenant (no pooling).

### per_usage

Pay-as-you-go developer / metered consumption.

- Independently activatable.
- Per-µservice meter shape declared in each µservice's PRD.
- Continuous metering; hourly/daily/weekly visibility in finops-portal.
- Monthly invoice grouped by (meter_unit, pricing_dimension).
- Soft-cap + optional hard-cap configurable.
- Idempotency-keyed; 7-day dedup window.
- Correction handling via `correction_for` field.

## Composability examples

| Configuration | Use case |
|---|---|
| `{}` (paid, empty) | Contract setup transient state |
| `{revenue_share}` | Pure marketplace seller |
| `{per_seat}` | Pure B2B enterprise |
| `{per_usage}` | Pure pay-as-you-go developer |
| `{revenue_share, per_seat}` | Reseller with internal team |
| `{revenue_share, per_usage}` | Marketplace seller with metered ops |
| `{per_seat, per_usage}` | Enterprise with consumption workload |
| `{revenue_share, per_seat, per_usage}` | Complex enterprise reseller |

## Counterparts (industry parity)

cloud-billing maintains feature parity with the UNION of:

- **Stripe Billing** (subscription lifecycle, invoicing, tax, marketplace via Connect, revenue recognition, dunning, sigma analytics)
- **AWS Billing & Cost Management** (CUR, Cost Allocation Tags, Cost Categories, Billing Conductor, Savings Plans, Reservations, Cost Anomaly Detection, Budgets, Free Tier Alerts, FOCUS 1.1)
- **Recurly** (subscription, dunning, ASC 606 / IFRS 15 revenue recognition, B2B net-terms invoicing, webhooks)

See `competitor-parity-matrix.md` for the ≥100-capability row-by-row coverage matrix.

## Cross-microservice handoffs

| Direction | Counterparty | Purpose |
|---|---|---|
| read | cloud-iam | Principal claim emission (tenant_class, billing_components, cap_breached) |
| ingest | every Phase-0/1/2 µservice | Usage event emission |
| call | cloud-billing-tax | Per-jurisdiction tax computation |
| emit | payments | Settlement statement → payout / invoice |
| emit | audit-chain | Event seal per ADR-0263 |
| write | cloud-storage | FOCUS export + invoice PDF + settlement PDF |
| call | cloud-kms | Invoice + statement signing |
| emit | notifications | Trial expiry / cap breach / conversion / payout alerts |
| emit | observability | Cost metrics + SLO metrics |
| publish | every µservice | tenant-class-mutated + billing-components-mutated events |

## Authority chain

cloud-billing inherits authority from:

1. **ADR-0330** — Tenant Class + Composable Billing Components (keystone)
2. **ADR-0329** — Tier System Retired
3. **ADR-0331** — Per-Microservice Tenant-Class Adoption Template
4. **ADR-0328** — Substance Bar as Canonical Sequence + Batch Discipline
5. **ADR-0244** — Tenant as Universal Scoping Primitive
6. **ADR-0243** — Cedar as Universal Gate
7. **ADR-0251** — Compliance Pack Primitive
8. **ADR-0255** §D-4 — BYOK Credentials Gating
9. **ADR-0249** — Multi-Category Marketplace
10. **ADR-0131** — Per-Microservice Flat Layout
11. **ADR-0132** — No-Grouping Policy
12. **ADR-0145** — Inter-Microservice Communication Reform (Direct gRPC)
13. **ADR-0263** — Audit Emission Contract
14. **ADR-0130** — Agentic SLO-Gated Promotion
15. **ADR-0253** — HTTP/3 + QUIC Default
16. **ADR-0252** — HLC + TrueTime Tier
17. **ADR-0248** — Amazon-Shape Cellular Architecture
18. **ADR-0218** — Per-Tenant Deployment Context

## Local ADRs

- `decisions/ADR-MS-001-billing-components-composability.md` — composability rules + 8 valid subsets + mutation flow.
- `decisions/ADR-MS-002-revenue-share-settlement-pipeline.md` — monthly settlement + clawback + FX + payments handoff.

## Implementation plans

See `implementation-plans/IP-001` through `implementation-plans/IP-015` for the per-slice plans.

## Running tests

```bash
# Kernel tests (existing 1,030-line kernel)
cargo test -p oya-cloud-billing-domain

# Integration tests (cross-µservice tax handoff)
cargo test -p oya-cloud-billing-tax-app

# Wave 15B new tests
cargo test -p oya-cloud-billing-app
cargo test -p oya-cloud-billing-worker
```

Dual-fixture tests are required per ADR-0330 §B.9.3. The CI lane `ci-tenant-class-adoption-check` rejects single-fixture tests.

## Deployment

OpenTofu modules at `iac/<deployment-context>/`. Each module is sigstore + cosign signed per ADR-0039.

```bash
# OCI Always Free (demo_trial default)
cd iac/oci-guest/always-free/
tofu init
tofu plan -var="tenant_id=demo_acme" -var="region=us-ashburn-1" -var="cell_id=cell-oci-ashburn-1"
tofu apply

# Paid tenant on AWS
cd iac/guest-on-aws/
tofu init
tofu plan -var="tenant_id=ten_acme" -var="region=us-east-1" -var="cell_id=cell-aws-use1-1"
tofu apply
```

## Operations

Runbooks at `runbooks/`. On-call rotation owned by axis-cloud-billing SRE.

Incidents page links from `runbooks/invoice-generation-timeout.md`, `runbooks/per-tenant-cost-attribution-mismatch.md`, `runbooks/reservation-recommendation-engine-stall.md`.

Coordination with downstream consumers (finops-portal, payments, audit-chain) for any cap-class incident.

## OS support

See `supported-oses.json`:

- Tier-1: Talos, RHEL 9+, Oracle Linux 9+, SLES 15 SP6+, Ubuntu 24.04 LTS+, Debian 13+, Rocky 9+, AlmaLinux 9+, CentOS Stream 10+, Amazon Linux 2023+, Flatcar, Photon 5+, macOS Apple Silicon M5+ (developer-only).
- Tier-2 (test only): linux/ppc64le, linux/s390x.
- Out-of-scope: Intel macOS, M1-M4, FreeBSD, OpenBSD, Windows Server, Solaris.
- Architecture: linux/amd64, linux/arm64, darwin/arm64-m5+.
- Package formats: RPM, DEB, container image, Talos extension, Flatcar ignition, macOS .pkg (dev-cell only), Homebrew.

Production deployment is always container-image-based; .pkg / Homebrew are developer-tooling only (cloud-billing is a backend µservice).

## Substance bar

This µservice is authored under ADR-0322 substance-bar requirement. The 1,030-line Rust kernel at `crates/oya-cloud-billing-domain/src/lib.rs` is the substance truth; the Wave 15B spec authoring (PRD + ARCHITECTURE + contracts + SLOs + Cedar + IaC + OS manifest) documents the kernel + extensions per ADR-0330.

A cold-start intern engineer should be able to build cloud-billing using only:

1. PRD.md (mission + outcomes + personas + FR + NFR + tenant_class + billing_components + handoffs)
2. ARCHITECTURE.md (layers + bounded contexts + data plane + control plane + deployment topology)
3. contracts/* (REST + AsyncAPI + gRPC contracts)
4. slos/* (SLO definitions)
5. policies/* (Cedar permits)
6. iac/<context>/* (OpenTofu modules per deployment context)
7. supported-oses.json (OS support matrix)
8. decisions/ADR-MS-* (local ADRs)
9. implementation-plans/IP-* (per-slice plans)
10. competitor-parity-matrix.md (industry-leader UNION coverage)

The 1,030-line existing kernel is hyperscaler-grade; the Wave 15B spec authoring closes the kernel-ahead-of-spec inversion identified in `coherence-audit-2026-05-20.md`.

## Contact

- Owner team: axis-cloud-billing
- Council: council-finance, council-architecture
- Pager: axis-cloud-billing SRE rotation
- Slack: #cloud-billing-substrate
- Quarterly review: industry-best-practice-conformance refresh

## License

Inherited from oyatie root.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0347-governance-fitness-bulk-rename.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
