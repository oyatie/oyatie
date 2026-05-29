---
doc_class: PackOverlayCatalog
microservice: contract-lifecycle-management
related_adrs:
  - ADR-0251
  - ADR-0329
  - ADR-0330
  - ADR-0331
date: 2026-05-21
---

# Contract Lifecycle Management — Pack Overlay Catalog

This directory replaces the retired `capability-tiers/tier-matrix.md` (deleted per ADR-0329 + Wave-4-Rolling remediation 2026-05-21). The legally-substantive content that previously lived in the retired named capability levels stratification (AES vs QES e-signature, OOXML diff, Llama-3.1 redlining, Loro CRDT collaboration, Thales Luna 7 A790 HSM, KISA/DSS TSA bindings) is now expressed as **pack overlays** keyed by the tuple `(deployment_context, tenant_class, jurisdiction_pack)`.

## Doctrine

Per ADR-0331 tenant-class-doctrine + ADR-0330 deployment-context-doctrine + ADR-0251 compliance-pack-primitive (KS#8):

- `tenant_class ∈ {demo_trial, paid}` with `paid.billing_components ⊆ {per_seat, per_usage, revenue_share}`.
- `deployment_context ∈ {oyatie-public-cloud, aws-guest, oci-guest, on-prem, colo, oyatie-as-cloud-provider}`.
- `jurisdiction_pack` is a per-jurisdiction overlay (e.g. `eu-eidas-qes`, `kr-pipa-sovereign`, `us-state-ca`).
- Compliance pack (e.g. `gdpr`, `sox-404`, `hipaa-baa`) and jurisdiction pack compose multiplicatively; higher-restriction-wins on every dimension.

The microservice does **not** stratify capability by tier. Every deployed instance ships the full canonical capability surface (contract draft + clause library + negotiation + obligation + renewal). Pack overlays gate **affordance** (e.g. QES allowed only when `jurisdiction_pack ∈ eidas-qes-set` AND `deployment_context` provides HSM custody), **retention** (e.g. SOX seven-year), **residency** (e.g. KR-PIPA in-country), and **evidence formats** (e.g. CAdES vs PAdES envelope choice).

## Pack inventory

| Pack | Family | File | Authoritative source |
|---|---|---|---|
| `gdpr` | compliance | `gdpr/README.md` | GDPR Articles 5-7, 17, 25, 32, 35 |
| `eidas` | compliance + jurisdiction | `eidas/README.md` | Regulation (EU) 910/2014 Articles 25-26, 28 |
| `esign` | compliance | `esign/README.md` | ESIGN Act 15 USC § 7001 |
| `sox-404` | compliance | `sox-404/README.md` | SOX §404 + §802 (18 USC §1520) |
| `hipaa-baa` | compliance | `hipaa-baa/README.md` | HIPAA Security Rule §164.308(b)(3) |
| `kr-pipa` | compliance + jurisdiction | `kr-pipa/README.md` | KR-PIPA Articles 15, 17, 32, 39 |
| `soc-2` | compliance | `soc-2/README.md` | AICPA SOC-2 TSP CC + Confidentiality |
| `iso-27001` | compliance | `iso-27001/README.md` | ISO/IEC 27001:2022 Annex A |
| `sec-17a-4` | compliance | `sec-17a-4/README.md` | SEC 17a-4(f) |

## Composition rule

For a given `(tenant, deployment_context, jurisdiction_pack)` tuple, the **active pack set** is the union of:

1. Tenant-default packs declared in the tenant manifest.
2. Jurisdiction-implied packs (e.g. a tenant in EU implies `gdpr` + `eidas`).
3. Industry-implied packs (e.g. a tenant declared as `healthcare_provider` implies `hipaa-baa`; `broker_dealer` implies `sec-17a-4`; `public_company` implies `sox-404`).

The active pack set is then resolved via **higher-restriction-wins**: any field where two packs disagree resolves to the stricter rule. The resolved pack is sealed into the tenant's audit chain at activation time.

## Mapping from retired tier matrix

For evidence preservation and migration tracing, the deleted `capability-tiers/tier-matrix.md` content maps onto the new model as follows. (See `legal-dimensions/tier-to-pack-migration-trace.md` for the full per-row trace.)

| Retired tier content | New shape |
|---|---|
| retired-basic hardware envelope (2 pods, AES-only, ≤10k contracts/year, 30d hot) | `tenant_class=demo_trial` overlay under `deployment_context=oci-guest/always-free` or `on-prem-light` |
| retired-standard hardware envelope (5 pods, AES+QES, AI redlining, ≤100k contracts/year) | `tenant_class=paid + billing_components=[per_seat]` default for mid-market B2B |
| retired-advanced hardware envelope (9 pods active-active, ≤1M contracts/year, Loro CRDT, obligation-AI) | `tenant_class=paid + billing_components=[per_seat, per_usage]` with AI capability flag enabled |
| retired-sovereign (sovereign-pack, in-pack HSM, FIPS 140-3 L3) | `tenant_class=paid + jurisdiction_pack ∈ {eu-eidas-qes, kr-pipa-sovereign, hipaa-baa}` + `provider_credential_modes.hsm_qes=byok_required_by_pack` |

The legally substantive vendor and standard references that anchored the retired tier matrix are preserved verbatim in the appropriate pack file (Thales Luna 7 A790 in `eidas/README.md`; KISA TSA in `kr-pipa/README.md`; DSS-list TSA in `eidas/README.md`; SeaweedFS WORM in `sec-17a-4/README.md`; Llama-3.1 / Claude in `legal-dimensions/ai-redlining-prompt-template.md`; Loro CRDT in `legal-dimensions/redline-collaboration-crdt.md`).
