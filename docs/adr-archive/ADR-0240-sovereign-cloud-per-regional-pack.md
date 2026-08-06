---
id: ADR-0240
status: Superseded
date: 2026-05-18
owners:
  - council-architecture
  - council-privacy
  - axis-cloud
  - ops-compliance
supersedes: []
superseded_by: [ADR-708]
related:
  - ADR-0010-regional-pack-architecture.md
  - ADR-0049-cross-region-replication-and-residency.md
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0144-eu-ai-act-graduated-risk-tier-model.md
doc_class: Architecture-Decision-Record
purpose: >
  Each regional pack declares a `sovereign_cloud_overlay` block
  enumerating the substrate providers it MUST use (primary + secondary)
  and the data classes that must remain on those providers.
  Cross-provider traffic for sovereign-tagged data is denied at policy
  time.
enforcement_status: advisory-until-per-pack-overlay-finalized
enforced_by: oya gate validate sovereign-cloud-overlay
---

# ADR-0240: Sovereign cloud per regional pack

## Status

Accepted — 2026-05-18. Enforcement is advisory until each regional
pack has its `sovereign_cloud_overlay.yaml` finalized by the relevant
regulator-facing team (KR ops-compliance, EU ops-compliance, KSA
ops-compliance).

## Context

ADR-0010 establishes the regional-pack concept: KR / EU / KSA / US /
JP packs each carrying jurisdiction-specific overlays (data residency,
regulator evidence cadence, retention).

ADR-0049 (cross-region replication + residency) covers data
replication within a pack. ADR-0009 (cells) covers per-tenant
isolation.

But the portfolio is silent on the **cloud-substrate sovereignty rule**:

- Pack-KR may need Naver Cloud or KT Cloud (CSAP-certified) for
  government-grade tenants.
- Pack-KSA may need STC Cloud or Mobily Cloud (NDMO + SDAIA).
- Pack-EU may need OVH Cloud for GAIA-X-certified workloads.
- Pack-JP may need Sakura Internet for METI Cloud Security Mark.
- Pack-US-Government may need AWS GovCloud or Azure Government.

Without an explicit cloud-substrate overlay, the portfolio defaults to
AWS/GCP/Azure — providers a sovereign-pack regulator can reject. The
deployment of pack-KR to a non-CSAP-certified provider, however
technically excellent, is a regulatory failure.

The hyperscaler-invariant spec (ADR-0128) references multi-cloud as a
property but does not pin the per-pack overlay. ADR-0105 amendment 3
(no vendor lock-in) bans vendor SDK calls in business-logic crates but
does not constrain the cloud-IaC substrate. This ADR closes the gap.

## Decision

### D-1. Per-pack overlay declaration

Each regional pack declares a `sovereign_cloud_overlay.yaml` at
`regional-packs/<pack-id>/sovereign-cloud-overlay.yaml` with the
following shape:

```yaml
pack_id: kr
primary_provider:
  id: naver-cloud
  regions: [kr-seoul, kr-busan]
  certifications: [CSAP-1.0, K-ISMS-P]
  contract_id: NAVER-CLOUD-2026-001
secondary_provider:
  id: kt-cloud
  regions: [kr-seoul-dr]
  certifications: [CSAP-1.0]
  contract_id: KT-CLOUD-2026-001
sovereign_data_classes:
  - PII_KR
  - HEALTH_RECORD_KR_TIER1
  - GOV_DATA_KR
non_sovereign_data_class_fallbacks:
  - provider: aws
    regions: [ap-northeast-2]
    data_classes: [PUBLIC, ANONYMIZED]
prohibited_egress:
  - data_class: PII_KR
    to_provider: aws
  - data_class: GOV_DATA_KR
    to_provider: '*non-kr*'
audit_cadence: quarterly
regulator_pack_evidence_cadence: annual
```

The crate-local SOV-001 fixture/parser slice is the machine-checkable
contract surface for air-gapped regional-pack deployment evidence:
`cell/core/regional-pack/src/sovereign_deployment.rs` parses and
validates the fixture shape, and
`cell/core/regional-pack/tests/fixtures/sovereign-airgap/kr-fsc-deployment-model.json`
is the non-production KR FSC example. These surfaces are evidence-shape
tests only; they do not claim certification, tenant activation,
regulator acceptance, runtime deployment, or a real signed bundle.

### D-2. Per-data-class enforcement

The data-class registry (`registry/data-class/`) carries each class's
sovereign-pack-bind. The Cedar policy layer (per ADR-0099) refuses
storage or transit decisions that would place sovereign-tagged data on
a non-sanctioned provider.

### D-3. Multi-cloud substrate stack

`microservices/cloud-iac/` already supports multi-tool deployment
(OpenTofu + Helm + Kustomize + ArgoCD/Flux per ADR-0121). This ADR
adds the per-provider terraform/opentofu module catalog:

| Provider | Module home | Supported services |
| --- | --- | --- |
| AWS | `microservices/cloud-iac/iac/opentofu/aws/` | EKS, RDS, S3, ElastiCache, ALB |
| GCP | `microservices/cloud-iac/iac/opentofu/gcp/` | GKE, CloudSQL, GCS, Memorystore, GCLB |
| Azure | `microservices/cloud-iac/iac/opentofu/azure/` | AKS, Postgres Flexible, Blob, Redis, Application Gateway |
| Naver Cloud | `microservices/cloud-iac/iac/opentofu/naver/` | NKS, Cloud DB, Object Storage, Cloud Memory DB, Cloud Load Balancer |
| KT Cloud | `microservices/cloud-iac/iac/opentofu/kt/` | K2P, DBaaS, OSS, Redis, LB |
| STC Cloud | `microservices/cloud-iac/iac/opentofu/stc/` | K8s, RDS, OSS, Redis, LB |
| OVH | `microservices/cloud-iac/iac/opentofu/ovh/` | Kubernetes, Managed Postgres, Object Storage, Redis, LB |
| AWS GovCloud | `microservices/cloud-iac/iac/opentofu/aws-gov/` | inherits AWS, FedRAMP-High substrate |

Each module exposes the same canonical interface (per ADR-0028 cloud
microservice catalog) so the µservice runtime doesn't care which
provider it's on.

### D-4. Cross-provider denial

The cloud-iac admission controller refuses any deploy that:

- Tags data class `X` for storage on provider `Y` when `Y ∉
  sovereign-overlay(pack(X))`.
- Routes data class `X` from provider `Y` to provider `Z` when
  `Z ∈ prohibited_egress(X)`.

The denial emits to the audit chain (class `SovereignCloudDeny`).

### D-5. Cell mapping

A cell (per ADR-0009) lives on exactly one provider. Cross-cell traffic
across providers must traverse the inter-provider mesh tunnel (per
ADR-0044 + ADR-0148). The inter-provider tunnel is allowed only for
data classes whose `prohibited_egress` permits the traversal.

### D-6. Audit + regulator evidence

Each pack emits a quarterly evidence packet (class `SovereignCloudEvidence`)
that enumerates:

- The exact provider footprint per data class.
- Cross-provider deny counts (operational anti-leakage proof).
- Per-provider certification expiration dates (so renewals are calendared).
- Per-cell sustainability metrics (per ADR-0174 cost-tag).

Regulators pull the packet via `microservices/cloud-iac/` audit-export.

### D-7. Provider failure handling

If the primary provider in a pack has an outage:

1. Stateless workloads cut over to the secondary provider per the
   per-pack overlay.
2. Stateful workloads (Postgres / Redis cluster) follow the DR plan
   per ADR-0180.
3. Data classes whose `sovereign_data_classes` block forbids fallback
   to the secondary REMAIN unavailable until the primary recovers.
   The brown-out signal (per ADR-0176) emits `outage` for those
   workloads; the rest emit `degraded`.

## Alternatives considered

### Alt-1. Single global cloud substrate (AWS only)

Run everything on AWS globally. **Rejected.** Pack-KR regulators
(CSAP, K-ISMS-P) and pack-KSA (NDMO, SDAIA) deny non-domestic
substrate for sovereign data classes. The portfolio loses sovereign
tenants by definition.

### Alt-2. Per-µservice provider choice

Let each µservice choose its provider independently. **Rejected.**
Defeats the cell-isolation property (a cell crosses providers); makes
sovereign-data egress untrackable; vendor coupling becomes per-µservice
rather than per-pack.

### Alt-3. Hybrid model — pack overlay for storage only, compute is
        global

Pin storage to per-pack providers; let compute float globally.
**Rejected.** CSAP / NDMO / GAIA-X requirements bind compute as well
as storage (the cipher key + computation must be on sovereign
substrate). Splitting storage from compute is a regulatory dead end.

## Consequences

### C-1. Positive

- **Sovereign tenants are addressable.** Pack-KR + pack-KSA + pack-EU
  + pack-US-Government workloads land on certified substrates.
- **Vendor independence is the default.** No single hyperscaler has a
  fatal grip on the portfolio.
- **Regulator evidence is automatic.** Quarterly packets emit to the
  audit chain.
- **Cell-isolation extends to provider-isolation.** A noisy cell on
  Naver doesn't impact a cell on AWS.

### C-2. Negative

- **Per-provider IaC modules increase substrate surface.** Mitigation:
  modules share the canonical interface; the cost is one
  module-per-provider, not one module per service.
- **Per-provider certification cadence must be tracked.** Mitigation:
  per-pack quarterly evidence packet enumerates expiration dates;
  ops-compliance owns renewal calendar.
- **Inter-provider tunnel cost.** Mitigation: cross-provider traffic
  is bounded by the sovereign-data-class denial; remaining traffic is
  routine.
- **Provider feature lag.** A regional provider (e.g. KT Cloud) may
  lag AWS in K8s minor version or managed-Postgres extension support.
  Mitigation: per-provider feature matrix in `microservices/cloud-iac/iac/`;
  µservices declare minimum-feature requirements; the matrix tells
  ops-compliance which packs each µservice is qualified for.

### C-3. Sustainability

- Per-provider PUE varies (Naver Cloud Chuncheon DC: PUE 1.18; AWS
  ap-northeast-2: PUE 1.21; OVH France: PUE 1.09). The
  sustainability tag (ADR-0174) carries per-provider PUE; the
  FinOps + carbon team can bias placement within a pack's allowed
  providers.

## Implementation surface

- `specs/sovereign-cloud-overlays.json` — canonical pack-id → providers
  mapping schema.
- `regional-packs/kr/sovereign-cloud-overlay.yaml` — KR pack overlay
  (full content, not stub).
- `regional-packs/eu/sovereign-cloud-overlay.yaml` — EU pack overlay
  (full content, not stub).
- `regional-packs/ksa/sovereign-cloud-overlay.yaml` — KSA pack overlay
  (full content, not stub).
- `regional-packs/us-government/sovereign-cloud-overlay.yaml` — US-Gov
  pack overlay (full content, not stub).
- `docs/standards/sovereign-cloud-overlay.md` — full standards doc.
- New validator lane `sovereign-cloud-overlay` added to
  `AGGREGATED_VALIDATE_LANES` (advisory).
- Per-provider OpenTofu modules:
  `microservices/cloud-iac/iac/opentofu/<provider>/` — module catalog
  (existing AWS/GCP/Azure modules carry forward; new ones added per
  pack in subsequent IPs tracked in `registry/cloud-iac/per-provider-module-backlog.tsv`).

## References

- AWS Builders Library — *Designing for multi-region and multi-cloud*
  (re:Invent 2023 talks).
- GAIA-X — *EU Sovereign Cloud Catalogue* (gaia-x.eu).
- KR MSIT — *CSAP (Cloud Security Assurance Program) v3.1 certification
  guide*.
- KSA SDAIA — *Cloud Computing Framework v1.0* (sdaia.gov.sa, 2023).
- ADR-0010 (this portfolio) — regional pack architecture.
- ADR-0049 (this portfolio) — cross-region replication + residency.
- ADR-0028 (this portfolio) — cloud microservice architecture (catalog).
- ADR-0121 (this portfolio) — on-prem k8s stack (multi-tool IaC base).
- ADR-0144 (this portfolio) — EU AI Act graduated-risk tier model.
