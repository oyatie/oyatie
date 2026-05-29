---
doc_class: IaCModuleManifest
microservice: contract-lifecycle-management
deployment_context: oyatie-public-cloud
related_memories: [feedback_zero_handroll_opentofu_only_2026_05_20, feedback_multi_context_provider_agnostic_2026_05_20]
date: 2026-05-21
---

# OpenTofu Module — Oyatie Public Cloud

Deployment context: `oyatie-public-cloud` (Oyatie's hosted multi-tenant SaaS surface).

This is the default deployment context for tenants who do not bring their own cloud account. Oyatie operates the underlying infrastructure (on top of AWS / OCI / Azure / GCP per ADR-0254 K8s everywhere); tenants pay per `tenant_class=paid + billing_components`.

## Usage (tenant onboarding)

```bash
oya tenant-provision \
  --tenant-id=<tenant_id> \
  --jurisdiction-packs='["gdpr", "esign"]' \
  --tenant-class=paid \
  --billing-components='["per_seat"]' \
  --home-cell=<cell_id> \
  --byok-mode=platform_default
```

Tenant onboarding does not require the customer to run `tofu apply`. Oyatie operates the underlying OpenTofu modules; tenant onboarding allocates the customer's tenant slot in an existing cell.

## Cell topology

Cells are pre-provisioned per ADR-0248 Amazon-shape cellular architecture. CLM cells eligible at cell-tier-1, cell-tier-2.

- Cell-tier-1: standard B2B customer pool.
- Cell-tier-2: high-throughput / large-enterprise pool.
- Cell-tier-0 (sovereign): KR-PIPA sovereign tenants, eu-eidas-qes tenants, HIPAA-Provider, FedRAMP.

## Sovereign cells

Sovereign cells are operated under additional residency + compliance overlays:

- **EU sovereign cell** (Frankfurt + Paris + Dublin): eIDAS QES native; LOTL ingestion; D-Trust + GlobalSign + Certigna TSA.
- **KR sovereign cell** (Seoul + Busan): KR-PIPA + KISA TSA + Yessign.
- **JP sovereign cell** (Tokyo + Osaka): APPI + 認定認証業務.
- **US-Gov sovereign cell** (us-east-gov + us-west-gov): FedRAMP High + IL5 + ITAR.
- **US-HIPAA sovereign cell** (us-east + us-west): HIPAA-Provider + BAA mandatory.
- **AU sovereign cell** (Sydney + Melbourne): IRAP PROTECTED + Privacy Act 1988.

## Module structure (stub)

```
iac/oyatie-public-cloud/
  README.md
  main.tf
  variables.tf
  outputs.tf
  versions.tf
  cells/
    standard-us/
    standard-eu/
    standard-apac/
    sovereign-eu-eidas-qes/
    sovereign-kr-pipa/
    sovereign-jp-appi/
    sovereign-us-gov/
    sovereign-us-hipaa/
    sovereign-au-irap/
  multi-region/
    main.tf
  tenant-provisioning/
    main.tf                  # tenant slot allocation
```

## Cost-sharing model

Per `cost-budget.md`, costs are amortized across all tenants sharing a cell. Tenant cost contribution:

- Per_seat: tenant's per-user fee covers their share.
- Per_usage: tenant pays for envelope counts + AI inference counts.
- Revenue_share: marketplace template sales settled per ADR-0314.

Demo_trial tenants run on the Always Free OCI cell (when applicable) or on a shared standard cell with caps; their cost contribution is zero.

## Operational responsibility

Oyatie operates:

- Cell-level Kubernetes upgrades.
- Cell-level OS patches.
- Cell-level HSM key rotation (for platform_default credential mode).
- Cell-level TSA certificate renewal.
- Cell-level WORM compliance attestation.
- Cell-level multi-region failover drills.
- Cell-level SOC-2 + ISO 27001 audit reports.

Customer (tenant) operates:

- Tenant-level Cedar policy authoring.
- Tenant-level pack activation.
- Tenant-level user provisioning.
- Tenant-level contract authoring.
- Tenant-level BYOK credential management (when applicable).
