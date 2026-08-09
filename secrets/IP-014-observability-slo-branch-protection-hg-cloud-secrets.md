---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-openbao-secretreference-substrate
impl_plan_id: IP-014-observability-slo-branch-protection-hg-cloud-secrets
status: pending
owner: axis-cloud-secrets + axis-observability + axis-governance
acceptance_lanes: [promotion-readiness, authority-cohesion]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-014: observability SLO + branch-protection + HG-CLOUD-SECRETS register

## Intent

Author OpenSLO manifests for cloud-secrets hot-path; register HG-CLOUD-SECRETS in authority-cohesion ledger per ADR-0123; wire branch-protection for `microservices/cloud-secrets/**` paths.

## ChangeSet boundary

Pure governance + observability wiring; no Rust code.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cloud-secrets/slos/secret-resolution.openslo.yaml` | create |
| `microservices/cloud-secrets/slos/rotation-completeness.openslo.yaml` | create |
| `microservices/cloud-secrets/slos/audit-emission-completeness.openslo.yaml` | create |
| `microservices/cloud-secrets/slos/hsm-attestation.openslo.yaml` | create |
| `registry/authority-cohesion/HG-CLOUD-SECRETS.yaml` | create |
| `.github/branch-protection.yaml` | update — require cloud-secrets gates on PRs touching the µservice |

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate openslo-manifest --microservice cloud-secrets
cargo run -p oya-dev-cli -- gate validate authority-cohesion
cargo run -p oya-dev-cli -- gate validate promotion-readiness --microservice cloud-secrets
```

## Test Plan

- OpenSLO manifests validate against v1.0 schema.
- HG-CLOUD-SECRETS registered + linked to PRD + threat-model + compliance.
- branch-protection refuses PR if cloud-secrets gates not green.

## Halt Conditions

- HG registration missing post-merge — fail authority-cohesion lane.

## Next IP

`IP-015-lean-a11-raw-secret-emission-lane-wiring.md`

## Wave 15-IP-substance A-G

### A. Problem
Cloud-secrets cannot promote on paper-only SLOs or unregistered authority. Its gates must prove the service is observable, tied to HG-CLOUD-SECRETS, and protected by branch rules whenever the microservice changes.

### B. Approach
Bind the service's OpenSLO manifests, dashboards, authority-cohesion registration, and branch-protection checks into the promotion path. This IP turns PRD availability/security claims into machine-checkable gates.

### C. Deliverables
- OpenSLO files under `microservices/cloud-secrets/slos/`.
- Dashboards `secret-resolution-rate.json`, `rotation-compliance.json`, and `audit-emission-completeness.json`.
- `registry/authority-cohesion/HG-CLOUD-SECRETS.yaml`.
- Branch protection requiring cloud-secrets gates on touched PRs.
- Promotion-readiness evidence from `manifest.json`, `PRD.md`, `threat-model.md`, and `compliance.md`.

### D. Ordered Implementation Steps
1. Validate every SLO listed in `manifest.json` has an OpenSLO file.
2. Link SLO metrics to dashboards and alert rules.
3. Register HG-CLOUD-SECRETS with PRD, architecture, threat model, compliance, and manifest references.
4. Add branch-protection selectors for `microservices/cloud-secrets/**`.
5. Wire promotion readiness to require SLO, authority, LEAN-A11, and layout gates.
6. Add doc-link checks for all referenced service evidence.
7. Capture final promotion evidence in remediation notes.

### E. Acceptance
- `cargo run -p oya-dev-cli -- gate validate openslo-manifest --microservice cloud-secrets`.
- `cargo run -p oya-dev-cli -- gate validate authority-cohesion`.
- `cargo run -p oya-dev-cli -- gate validate promotion-readiness --microservice cloud-secrets`.
- Branch protection fails PRs that touch cloud-secrets while required gates are red.

### F. Evidence
Evidence anchors are `manifest.json` SLO registry, `slos/*.openslo.yaml`, `dashboards/*.json`, `PRD.md`, `ARCHITECTURE.md`, `threat-model.md`, `compliance.md`, and `coherence-audit-2026-05-20.md`.

### G. Counterpart Comparison
AWS, GCP, Azure, Vault, and Akeyless expose operational metrics, but Oyatie's counterpart standard requires service-local SLO manifests plus promotion and authority gates. This IP makes observability a delivery blocker, not an after-the-fact dashboard.

Grep-recognized counterpart anchor: GitHub Actions Secrets is directly relevant to branch-protection and CI gate wiring because workflow-distributed credentials must be detected, referenced, and audited by cloud-secrets checks. The observability comparator remains vendor metric and promotion-control surfaces.

## DR posture (per ADR-0343)

- Target source: `microservices/cloud-secrets/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`openbao_seal_unseal`, `postgres_wal_g`, `audit_chain_merkle_seal`].
- Surface evidence: `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/manifest.json`, `microservices/cloud-secrets/IP-014-observability-slo-branch-protection-hg-cloud-secrets.md`.

## Sustainability emission (per ADR-0344)

- Per-call audit row emission MUST include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on the same metering/audit event.
- Carbon-aware scheduling eligibility: eligible only when the workload is not Tier 0/Tier 1 and not one of `eu-ai-act-annex-iii`, `hipaa-em-incident-response`, or `pci-dss-realtime-fraud-detection`; excluded calls emit `defer_rejected`.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Cost source: `microservices/cloud-secrets/manifest.json#paid_billing_components_emitted` is absent; this section is triggered by IP text and must be reconciled with the manifest billing model.
- Surface evidence: `microservices/cloud-secrets/manifest.json`, `microservices/cloud-secrets/IP-014-observability-slo-branch-protection-hg-cloud-secrets.md`.
