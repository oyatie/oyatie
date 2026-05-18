---
contract: sovereign-cloud-overlay
authored: 2026-05-18
canonical_authority: ADR-0179
related_specs:
  - /specs/sovereign-cloud-overlays.json
related_adrs:
  - ADR-0010
  - ADR-0028
  - ADR-0049
  - ADR-0121
  - ADR-0144
  - ADR-0179
status: canonical-base
authorities_cited:
  - GAIA-X EU Sovereign Cloud Catalogue
  - KR MSIT CSAP v3.1 certification guide
  - KSA SDAIA Cloud Computing Framework v1.0 (2023)
---

# Sovereign cloud per regional pack standards

## Overlay shape

Each regional pack declares
`regional-packs/<pack-id>/sovereign-cloud-overlay.yaml` per the schema
in `/specs/sovereign-cloud-overlays.json`. Fields:

| Field | Required | Notes |
| --- | --- | --- |
| `pack_id` | yes | Enum (kr / eu / ksa / us / us-government / jp / in / br / ae / global) |
| `primary_provider` | yes | id + regions + certifications + contract_id |
| `secondary_provider` | yes | same shape |
| `sovereign_data_classes` | yes | List of data classes that MUST stay on `primary_provider` or `secondary_provider` |
| `non_sovereign_data_class_fallbacks` | optional | Per-class allow list for non-sovereign data on other providers |
| `prohibited_egress` | yes | List of (data_class, to_provider) pairs that are denied |
| `audit_cadence` | yes | Frequency of cloud-iac audit |
| `regulator_pack_evidence_cadence` | yes | Frequency of regulator-evidence packet emit |

## Provider catalog

| Provider id | Type | Typical use | Certifications |
| --- | --- | --- | --- |
| `aws` | Global hyperscaler | global pack default | SOC 2, ISO 27001, FedRAMP-Moderate |
| `gcp` | Global hyperscaler | global pack default | SOC 2, ISO 27001, FedRAMP-Moderate |
| `azure` | Global hyperscaler | global pack default | SOC 2, ISO 27001, FedRAMP-Moderate |
| `naver-cloud` | KR sovereign | KR pack primary | CSAP, K-ISMS-P |
| `kt-cloud` | KR sovereign | KR pack secondary | CSAP |
| `stc-cloud` | KSA sovereign | KSA pack primary | NDMO, SDAIA |
| `mobily-cloud` | KSA sovereign | KSA pack secondary | NDMO |
| `ovh` | EU sovereign | EU pack primary | GAIA-X, ISO 27001 |
| `sakura` | JP sovereign | JP pack | METI Cloud Security Mark |
| `aws-gov` | US sovereign | US-Government pack primary | FedRAMP-High, ITAR |
| `azure-government` | US sovereign | US-Government pack secondary | FedRAMP-High, ITAR |
| `openstack-onprem` | On-prem | Per ADR-0121 on-prem stack | per-customer |

## Cross-provider denial

The cloud-iac admission controller (per ADR-0121) refuses any deploy
that:

1. Tags data class X for storage on provider Y when `Y ∉
   sovereign-overlay(pack(X))`.
2. Routes data class X from provider Y to provider Z when `(X, Z) ∈
   prohibited_egress`.

Denial emits to the audit chain (class `SovereignCloudDeny`) with
`{denied_resource, intended_provider, allowed_providers,
data_classes_involved}`.

## Cell mapping

A cell (per ADR-0009) lives on exactly one provider. Cross-cell traffic
across providers traverses the inter-provider mesh tunnel (per
ADR-0044 + ADR-0148). The tunnel is allowed for non-sovereign
data classes only; sovereign data classes are denied at the egress
classifier.

## Per-pack module catalog

`microservices/cloud-iac/iac/opentofu/<provider>/` ships the per-provider
OpenTofu modules. Each module exposes the canonical interface from
ADR-0028 catalog so the µservice runtime is provider-agnostic.

Backlog of new modules (per pack expansion) at
`registry/cloud-iac/per-provider-module-backlog.tsv`.

## Audit cadence

- Cloud-iac audit: quarterly per `audit_cadence`.
- Regulator-evidence packet: per `regulator_pack_evidence_cadence`
  (typically quarterly for sovereign packs, annual for global).

Audit-chain class: `SovereignCloudEvidence`. Contents:

- Per-data-class provider footprint.
- Cross-provider deny counts.
- Per-provider certification expiration calendar.
- Per-cell sustainability metrics.

## Provider failure handling

1. Stateless workloads cut over to secondary per overlay.
2. Stateful workloads follow DR per ADR-0180.
3. Sovereign-bound data classes remain unavailable until primary
   recovers; their µservices emit `outage` per ADR-0176.

## Cost dimension

Per-provider PUE values feed the sustainability tag (ADR-0174). The
FinOps + carbon team biases placement within each pack's allowed
providers.

## Anti-patterns

- Placing sovereign-tagged data on a non-sanctioned provider — denied
  at admission.
- Cross-pack data movement without explicit overlay declaration — denied.
- Routing through a non-sanctioned provider's inter-region link —
  denied at egress classifier.

## Coverage status

Per-pack overlay finalization tracker at
`registry/sovereign-cloud/per-pack-overlay-status.tsv`. Validator lane
`sovereign-cloud-overlay` is advisory until all sovereign packs (kr,
eu, ksa, us-government) have signed-off overlays.
