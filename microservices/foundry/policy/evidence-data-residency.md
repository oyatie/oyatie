---
doc_class: PolicyDocument
title: foundry-evidence — data residency
microservice: foundry-evidence
status: Accepted
date: 2026-05-17
owner_team: council-privacy + axis-foundry-evidence
related_adrs: [ADR-0117, ADR-0028, ADR-0131]
related_artifacts:
  - microservices/foundry/PRD.md
  - microservices/foundry/dpia.md
  - microservices/foundry/threat-model.md
  - microservices/foundry/policy/tenant-scope.cedar
doc_status: published
---

# foundry-evidence — data residency policy

## DR-01 — Pack-local processing

Every evidence pack is processed, indexed, and persisted **strictly within the source tenant's `pack` boundary** as set at tenant onboarding by `tenancy` µservice. The `pack` value is carried on every envelope and pinned at recording time by `policy/tenant-scope.cedar` PERMIT 1.

| Pack | Region | Cloud | DR-class | Local-law minimum cited |
|---|---|---|---|---|
| pack-kr | KR (Seoul) | OCI KR | DR-A | KR PIPA Art. 28 cross-border + Art. 29 safety + Arts. 5–7 전자문서법 |
| pack-eu | EU (Frankfurt + Madrid) | OCI EU | DR-A | GDPR Art. 30 + EU AI Act Arts. 12/18/26 |
| pack-us | US (Ashburn + San Jose) | OCI US | DR-A | SOC 2 CC4.x + state privacy where applicable |
| pack-us-healthcare | US (Ashburn HIPAA-eligible) | OCI US | DR-AA | HIPAA §164.312(b) + §164.316(b)(2) + §164.308(a)(1)(ii)(D) |
| pack-jp | JP (Tokyo) | OCI JP | DR-A | APPI cross-border restriction |
| pack-sg | SG | OCI SG | DR-A | Singapore PDPA |
| pack-au | AU (Sydney) | OCI AU | DR-A | Australian Privacy Act |
| pack-in | IN (Mumbai) | OCI IN | DR-A | DPDP Act 2023 |
| pack-br | BR (São Paulo) | OCI BR | DR-A | LGPD Art. 33 international transfers |
| pack-ae | AE (Abu Dhabi) | OCI AE | DR-A | UAE PDPL |
| pack-ksa | KSA (Riyadh) | OCI KSA | DR-A | Saudi PDPL |

## DR-02 — Cross-pack replication FORBIDDEN

Evidence pack data MUST NOT replicate across packs at the data plane.

- Postgres replicas are pack-local only.
- audit-chain Merkle root publication is pack-local; the `tenant:oya-aggregate` series carries hashes only, never tenant-distinguishable data.
- WORM blob (audit-chain substrate) is pack-local; cross-region replication is opt-in per-tenant DPA and uses the substrate's cross-region path which is **disabled by default** per ADR-0028 §"Chain locality".
- LEAN lane `cross-pack-replication-forbidden` blocks any Helm or Terraform change that introduces cross-pack data-plane edges.

## DR-03 — Regulator-export cross-jurisdiction

Regulator export to a non-pack regulator endpoint is permitted **only** when:

1. The tenant has filed a DPA-recorded export plan with `tenancy` µservice.
2. A receiving-bucket SCC (or equivalent legal-transfer mechanism: BCR, Article-49 derogation) is attested on file.
3. 2-person rule sign-off on the export request (Cedar `regulator-export-scope.cedar`).
4. Export delivery uses TLS 1.3 + mTLS where regulator endpoint supports; falls back to tenant-mediated bridge otherwise.
5. The export-issuance event is audit-emitted with `dpa_export_plan_id` + `scc_attestation_id` + `framework`.

## DR-04 — Sensitive-data class handling

| Data class | Permitted in packs | Plaintext-read entitlement source |
|---|---|---|
| INTERNAL_ONLY | all packs | tenant operator (default) |
| BEHAVIORAL_TENANT_PRODUCT | all packs | tenant operator (default) |
| PII_IDENTIFYING | all packs (per-tenant DPA) | tenant operator with `pii_identifying` entitlement |
| PII_QUASI_IDENTIFIER | all packs | tenant operator with `pii_quasi_identifier` entitlement |
| SENSITIVE_PIPA_ART23 | pack-kr (consent required) + pack-jp (APPI sensitive) | tenant operator with explicit consent attestation |
| PHI | pack-us-healthcare ONLY (BAA required) | tenant operator with BAA-bound `phi` entitlement |
| AUDIT | all packs (this is the substrate's own class) | tenant operator (default) — audit-of-audits |
| SECRET | all packs (never plaintext to TenantOperator) | InternalForensic during declared incident only |

`policy/tenant-scope.cedar` PERMITs gate plaintext reads against the principal's `sensitive_data_entitlements` set; the set is populated by `tenancy` µservice at DPA / BAA signature.

## DR-05 — DSR cascade

Data subject right exercise from `tenancy` cascades into foundry-evidence via the audit-chain substrate retention-cascade RPC. foundry-evidence does not own retention authority; it consumes substrate-emitted `RetentionApplied` events to redact its Postgres index rows and refuse subsequent plaintext reads.

## DR-06 — Public-read scope

Per `policy/public-read.cedar`, public-read endpoints expose:

- Schema versions (`oya-foundry-evidence-evidence-pack` schema introspection).
- Claim-matrix per ADR-0133.
- Framework-profile field selectors.
- SLO targets (top-level numbers from PRD.md NFR table).
- Capability catalog metadata.

Public-read does NOT expose tenant identifiers, invocation counts, or per-tenant timings.

## DR-07 — Operator access

Internal operator access (ops-sre-reliability, ops-security, council-privacy) is Cedar-gated, audit-emitted, and never grants plaintext read except during a declared incident with `incident_scope_packs` entitlement (`policy/tenant-scope.cedar` PERMIT 4).

## DR-08 — Cross-microservice import

`foundry-evidence` consumes `audit-chain` strictly via `oya-audit-chain-emission-sdk` re-exports. Direct type-import from `oya-audit-chain-*-domain` or any internal substrate crate is forbidden; LEAN lane `cross-microservice-import-forbidden` blocks.

## DR-09 — Review cadence

- Annual review by council-privacy.
- Out-of-cycle review on: new pack onboarded, change to substrate residency posture in ADR-0117, change to regulator-export channel mechanism.
- Sign-off: council-privacy chair + DPO of affected packs.
