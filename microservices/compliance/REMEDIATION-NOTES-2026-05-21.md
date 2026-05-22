## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/compliance/benchmarks/drata-vanta-onetrust-auditboard-vs-oyatie.md
- microservices/compliance/coherence-audit-2026-05-20.md
- microservices/compliance/onboarding/compliance-engineer-first-week.md
- microservices/compliance/performance-benchmark-numbers-2026-05-20.md

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- None.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture (ADR-0343): PRD now sets manifest RTO 1800s/RPO 300s, multi-region collector/auditor metadata, and `runbooks/seaweedfs-evidence-bucket-loss.md` as the primary failover reference with certification/export runbooks as downstream recovery. Alternative considered: SOC2-only RTO 14400s; rejected because EU-AI and HIPAA packs make evidence windows stricter for regulated tenants. Cost: evidence bucket loss and collector replay tests must prove the 300s RPO.
- Capacity model (ADR-0340): PRD now records manifest values: 0.16 vCPU, 256 MiB RAM, 12 GiB evidence/export storage, 2 Valkey, 3 Postgres, 8 outbound collector slots, `per_workflow_run` scaling, Tier-2 cell placement, and collector/DSAR burst boundaries. Alternative considered: `per_capability`; rejected because D-2 manifest doctrine already names workflow runs as the scaling unit. Cost: audit-window surge isolation must be implemented before broad auditor onboarding.
- Sustainability + cost attribution (ADR-0344): PRD now requires evidence, DSAR, pack, auditor, and breach-clock audit rows to carry cost/carbon/energy fields, while statutory deadlines and pack gates ignore carbon routing. Alternative considered: post-hoc monthly carbon allocation; rejected because compliance evidence must be auditable at event time. Cost: compliance portal must link pack costs to FinOps axes.
- API versioning posture (ADR-0342): PRD now requires date carriers for DSAR/evidence/auditor/regulator APIs, SDK semver, 3-version/180-day support, tenant pinning, and internal mesh exemption. Alternative considered: Backstage plugin versioning only; rejected because external auditors and regulator exports need stable public contracts. Cost: engagement setup must record tenant version pins.

## Wave 15-doctrine-propagation-IPs (2026-05-21)

- Bucket: `D4-BUCKET-2`.
- Doctrine source: ADR-0337..0345 selective propagation by trigger match; this section records only matched IPs.
- Manifest gap: `manifest.json#dr` is absent, so DR sections preserve compliance-pack floors without inventing service RTO/RPO targets.

| IP | Trigger(s) | Required sections | Source evidence | Manifest gaps |
| --- | --- | --- | --- | --- |
| `microservices/compliance/IP-002-soc2-control-mapping.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-004-hipaa-min-necessary-log-substrate.md` | B | DR posture (per ADR-0343) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-006-evidence-storage-seaweedfs.md` | C | Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-007-auditor-readonly-portal.md` | D | Pod runtime tier (per ADR-0338) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#pod_runtime_tier missing |
| `microservices/compliance/IP-013-audit-anomaly-detection.md` | B | DR posture (per ADR-0343) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-015-regulatory-pack-evidence-overlay.md` | B | DR posture (per ADR-0343) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-016-pack-registry-kernel.md` | B | DR posture (per ADR-0343) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-017-pack-registry-domain.md` | B | DR posture (per ADR-0343) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-018-dpia-orchestration-usecase.md` | A, C | API Versioning (per ADR-0342); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-019-breach-notification-workflow.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-020-regulator-audit-evidence-rest.md` | A | API Versioning (per ADR-0342) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-021-cell-certification-attestation-worker.md` | B | DR posture (per ADR-0343) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-022-compliance-control-mapping-domain.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-023-pack-registry-grpc.md` | A | API Versioning (per ADR-0342) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-024-dpia-orchestration-adapter-postgres.md` | A, C | API Versioning (per ADR-0342); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-025-breach-notification-async-emit.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-026-control-mapping-rest-and-sdk.md` | A | API Versioning (per ADR-0342) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-journey-j01-emergency-911-dispatch-pack-overlay.md` | B | DR posture (per ADR-0343) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j02-healthcare-code-blue-ehr-break-glass-privacy-officer.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j100-pack-rollout-first-action.md` | C | Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-journey-j101-pack-attestation.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j104-pack-attestation.md` | C | Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-journey-j105-pack-attestation.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j106-pack-attestation.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j118-data-sharing-pack-overlay.md` | A | API Versioning (per ADR-0342) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-journey-j119-kyb-aml-bid-screening.md` | A, B, C, D | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344); Pod runtime tier (per ADR-0338) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing; manifest.json#pod_runtime_tier missing |
| `microservices/compliance/IP-journey-j122-tax-withholding-overlay.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j125-overlay-union-and-pack-delta.md` | A, C | API Versioning (per ADR-0342); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-journey-j126-fedramp-conmon-pack-overlay.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j129-judicial-process-pack.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j13-higher-restriction-policy.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j130-whistleblower-protection-pack.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j131-multi-jurisdiction-pack-overlay.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j132-eu-ai-act-and-multi-jurisdiction-overlays.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j133-rif-compliance-and-litigation-hold.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j135-per-jurisdiction-investigation-overlay.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j137-corporate-internal-audit-sox-controls-test-pack-overlay.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j141-internal-audit-personal-tenant-boundary-pack-overlay.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j143-dlp-scrub-bot-principal.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j20-kr-pipa-notification-clock.md` | A, C | API Versioning (per ADR-0342); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-journey-j43-hipaa-cell-overlay.md` | B | DR posture (per ADR-0343) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j44-hipaa-consult-overlay.md` | B | DR posture (per ADR-0343) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j45-patient-record-overlay.md` | B | DR posture (per ADR-0343) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j46-rx-overlay.md` | B, D | DR posture (per ADR-0343); Pod runtime tier (per ADR-0338) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing; manifest.json#pod_runtime_tier missing |
| `microservices/compliance/IP-journey-j47-healthcare-billing-overlay.md` | B | DR posture (per ADR-0343) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j48-kr-fss-overlay.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j61-hipaa-pack.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j63-trial-pack.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j64-hipaa-boundary.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j65-gdpr-pack.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j66-tax-pack.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j67-warrant-pack.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j68-per-pack-attestation.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j71-kr-fss-report.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j73-publisher-pack.md` | A, B, C, D | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344); Pod runtime tier (per ADR-0338) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing; manifest.json#pod_runtime_tier missing |
| `microservices/compliance/IP-journey-j74-pack-overlay-verification.md` | A, B, C, D | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344); Pod runtime tier (per ADR-0338) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing; manifest.json#pod_runtime_tier missing |
| `microservices/compliance/IP-journey-j75-incident-pack.md` | A, B, C, D | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344); Pod runtime tier (per ADR-0338) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing; manifest.json#pod_runtime_tier missing |
| `microservices/compliance/IP-journey-j76-pack-overlay-regulator.md` | A | API Versioning (per ADR-0342) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-journey-j77-pack-overlay-regulator.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j78-pack-overlay-regulator.md` | A | API Versioning (per ADR-0342) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-journey-j79-pack-overlay-regulator.md` | A | API Versioning (per ADR-0342) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-journey-j80-pack-overlay-regulator.md` | A | API Versioning (per ADR-0342) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-journey-j81-pack-overlay-regulator.md` | A | API Versioning (per ADR-0342) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-journey-j82-pack-overlay-regulator.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j83-pack-overlay-regulator.md` | A | API Versioning (per ADR-0342) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-journey-j84-pack-overlay-regulator.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j85-pack-overlay-regulator.md` | A | API Versioning (per ADR-0342) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-journey-j86-pack-overlay-regulator.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j87-pack-overlay-regulator.md` | A | API Versioning (per ADR-0342) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-journey-j88-pack-overlay-regulator.md` | A | API Versioning (per ADR-0342) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-journey-j89-pack-overlay-regulator.md` | A | API Versioning (per ADR-0342) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-journey-j90-pack-overlay-regulator.md` | A | API Versioning (per ADR-0342) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-journey-j91-us-msb-mtl-overlay.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j92-br-lgpd-us-parent-dsar.md` | C | Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-journey-j93-in-dpdpa-rbi-overlay.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j94-sox404-public-company-controls.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | manifest.json#dr missing |
| `microservices/compliance/IP-journey-j95-iso27001-soc2-annual-audit.md` | C | Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-journey-j96-ksa-uae-mena-onboarding.md` | C | Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-journey-j97-sg-pdpa-mas-tenant.md` | C | Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-journey-j98-au-privacy-apra-cps234.md` | C | Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |
| `microservices/compliance/IP-journey-j99-multi-pack-conflict-resolution.md` | C | Sustainability emission (per ADR-0344) | microservices/compliance/contracts/openapi.yaml, crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact | none |

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: baseline_cpu_per_tenant 0.16 vCPU; baseline_ram_per_tenant 256 MiB; storage_per_tenant 12 GB; connections valkey=2, postgres=3, outbound_http=8; scaling_dimension per_workflow_run; cell_placement_class Tier-2.
- ADR: ADR-0340 capacity_model; ADR-0248 cellular criticality numbering.
- Why: Compliance evidence collection, DSAR export, auditor portal reads, and regulator exports are workflow-run shaped with larger per-tenant evidence storage than hot request services.
- Rejected: scaling_dimension=per_request because evidence collection and DSAR export costs are dominated by workflow runs and scheduled collectors.
- Cost: Allocates materially higher evidence storage and outbound collector capacity per tenant.

### Block 2: dr
- Values: rto_p99_seconds 1800; rpo_p99_seconds 300; multi_region_active_active true; backup_substrate object_storage_versioned, seaweedfs_replicated, postgres_wal_g, audit_chain_merkle_seal; failover_runbook runbooks/seaweedfs-evidence-bucket-loss.md; replication_shape active-active-multi-az-cross-region-warm.
- ADR: ADR-0343 DR RTO/RPO matrix and compliance-pack floors.
- Why: Compliance holds regulator-visible evidence and DSAR exports; EU AI and HIPAA floors require active multi-region recovery with bounded evidence loss.
- Rejected: cold object restore because evidence export and breach timelines continue during infrastructure incidents.
- Cost: Requires replicated evidence storage and audit-sealed restore drills.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier 1; evidence microservices/compliance/PRD.md, microservices/compliance/ARCHITECTURE.md, microservices/compliance/IP-004-hipaa-min-necessary-log-substrate.md, microservices/compliance/IP-011-cross-microservice-evidence-fan-in.md, microservices/compliance/runbooks/seaweedfs-evidence-bucket-loss.md.
- ADR: ADR-0338 pod runtime tiering; ADR-0340 D-6 cell/runtime co-variance.
- Why: Compliance handles regulated tenant evidence, DSAR exports, auditor access, and breach declarations. It is first-party code, but it touches sensitive tenant evidence and compliance data planes, so Tier 1 isolation applies.
- Rejected: pod_runtime_tier=2 because auditor and DSAR evidence payloads are regulated tenant data, not ordinary application telemetry.
- Cost: Tier 1 runtime placement adds overhead to evidence collectors and export workers.

### Block 4: tenant_version_pinning
- Values: declared_versions 2026-05-21, 2026-02-21, 2025-11-21; default_version 2026-05-21; supported_window_size 3; supported_window_minimum_days 180; supports_per_tenant_pinning true.
- ADR: ADR-0342 hybrid date-versioned public API policy.
- Why: Compliance exposes auditor, DSAR, breach, and pack overlay surfaces whose semantics differ by tenant and regulator.
- Rejected: internal-only classification because auditor and subject-access workflows consume public contracts.
- Cost: Maintains contract windows and migration docs for auditor and DSAR consumers.

### Block 5: consumes_upstream_oss and oss_stewardship_class_overrides
- Values: consumes_upstream_oss postgresql, cedar, openbao, opentelemetry, cilium, istio, kyverno; oss_stewardship_class_overrides empty because registry-default stewardship applies.
- ADR: ADR-0345 OSS stewardship class and CVE response policy.
- Why: Compliance depends on registry-governed data, policy, secret, telemetry, mesh, and admission substrates.
- Rejected: service-local stewardship overrides without a registry delta.
- Cost: Evidence collectors must follow registry CVE and pin-response ownership without a local override.

### Block 6: iac_module_invocations
- Values: oci-guest/object-storage-versioned@v1, on-prem/evidence-store@v1, colo/openbao-secret-binding@v1, oyatie-as-cloud-provider/audit-chain-merkle-seal@v1.
- ADR: ADR-0339 shared IaC module library.
- Why: Compliance needs shared evidence, object storage, secret, and audit-seal modules to keep regulator evidence portable.
- Rejected: collector-local storage provisioning because evidence retention semantics must not diverge by context.
- Cost: Evidence infra rollout now waits for shared module pin validation.
