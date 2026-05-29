---
ip_id: IP-018
microservice: compliance
bounded_context: dpia-orchestration
layer: usecase
status: planned
related_adrs: [ADR-0251, ADR-0292, ADR-0263, ADR-0244]
---

# IP-018 — DPIA orchestration usecase

## A. Problem

GDPR Article 35 DPIA obligations cannot be satisfied by a static pack flag. A tenant subscribing to GDPR, EU AI Act, or high-risk processing overlays needs a workflow that gathers data-flow inventory, assesses privacy risk, records DPO review, and blocks pack activation when mitigation is incomplete. The existing IP did not connect this workflow to local contracts, DSR/DSAR evidence, or the pack registry.

## B. Approach

Implement `oya-compliance-dpia-orchestration-usecase` as a use-case layer over pack registry, ontology projection, control mapping, and audit evidence. It creates a DPIA record when pack subscription or new data processing requires assessment, pulls data-flow inventory from ontology, applies LINDDUN-style threat categories, links mitigations to control ids, and seals the final record through compliance evidence.

## C. Deliverables

| Artifact | Change |
|---|---|
| `microservices/compliance/catalog/oya-compliance-dpia-orchestration-usecase.yaml` | usecase catalog row |
| `microservices/compliance/contracts/openapi.yaml` | add or reserve DPIA request/status endpoints when REST surface lands |
| `microservices/compliance/contracts/dsar-export-format.json` | reference DPIA data-flow terms where subject data exports intersect processing records |
| `microservices/compliance/runbooks/dsar-backlog-overflow.md` | separate DSAR backlog from DPIA review backlog |
| `microservices/compliance/slos/evidence-emission-lag.openslo.yaml` | evidence emission freshness after DPIA finalization |

## D. Implementation

1. Define `DpiaCreateRequest { tenant_id, pack_id, bounded_context, processing_purpose, data_classes }`.
2. Resolve pack obligations through IP-017 and decide whether DPIA is mandatory.
3. Pull data-flow inventory from ontology projection by tenant and bounded context; fail with `OntologyStale` when projection freshness is below floor.
4. Generate a risk register using LINDDUN categories: linkability, identifiability, non-repudiation, detectability, disclosure, unawareness, and non-compliance.
5. Link each mitigation to an IP-022 control id and an evidence collector id.
6. Require DPO review signature before `finalize`; emit `oya.compliance.dpia-finalized` with audit seal.
7. Block pack subscription in IP-017 when risk is above threshold and mitigations are unsigned.
8. Add tests for mandatory/not-mandatory DPIA, stale ontology, high-risk block, DPO signature missing, and sealed finalization.

## E. Acceptance

- GDPR/EU AI Act high-risk pack subscription opens a DPIA record before activation.
- DPIA finalization emits an evidence artifact and a control-mapping link.
- High-risk unmitigated findings block pack activation.
- Tenant A's data-flow inventory never appears in tenant B's DPIA.

## F. Evidence

- `microservices/compliance/PRD.md` names GDPR DSAR automation and cross-tenant isolation as goals.
- `microservices/compliance/packs/GDPR.md` and `EU-AI-Act.md` are the local pack anchors.
- `microservices/compliance/competitor-parity-matrix.md` lists Drata, Vanta, OneTrust/Tugboat, AuditBoard, and ServiceNow GRC as GRC counterparts; OneTrust is the strongest DPIA counterpart.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| OneTrust | Provides DPIA workflow parity without sending data-flow inventory to a third-party SaaS. |
| Vanta / Drata | Goes deeper than generic privacy evidence by blocking pack activation on unreviewed risk. |
| ServiceNow GRC | Narrows workflow orchestration parity while preserving Oyatie audit-chain sealing. |

## H. Non-goals and handoff boundaries

- Do not author legal DPIA text automatically; the usecase orchestrates evidence and review.
- Do not open DPIA records for packs that do not require them; mandatory logic comes from pack registry.
- Do not accept stale ontology inventory when the DPIA controls high-risk processing.
- Do not finalize without DPO signature and mitigation links.
- Do not mix DSAR exports with DPIA records; they share data-flow vocabulary but remain separate artifacts.

## I. Fixture set

- `gdpr_pack_requires_dpia.json` proves mandatory creation.
- `soc2_pack_no_dpia_required.json` proves no unnecessary record.
- `ontology_stale_blocks_review.json` proves freshness gate.
- `high_risk_without_mitigation_blocks_pack.json` proves activation block.
- `dpo_signature_missing.json` proves finalization denial.

## J. Launch blockers

- DPIA finalizes without DPO signature metadata.
- High-risk processing activates a pack before mitigation is linked to a control id.
- Ontology freshness failure is downgraded to warning.
- Data-flow inventory contains records from another tenant.
- Final evidence lacks an audit-chain seal reference.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/compliance/IP-018-dpia-orchestration-usecase.md` matched `openapi`; contract files `microservices/compliance/contracts/openapi.yaml, microservices/compliance/contracts/asyncapi.yaml, microservices/compliance/contracts/compliance.proto`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/compliance/IP-018-dpia-orchestration-usecase.md` matched `emission`; anchors `microservices/compliance/manifest.json, crates/oya-shared-compliance-evidence-kernel/src/lib.rs`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.
