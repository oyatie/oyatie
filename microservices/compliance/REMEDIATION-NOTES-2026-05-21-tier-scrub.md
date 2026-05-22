# Wave 15J-batch-4 tier scrub remediation notes: compliance

## Files Modified

- ARCHITECTURE.md: 1281 lines
- README.md: 50 lines
- benchmarks/drata-vanta-onetrust-auditboard-vs-oyatie.md: 126 lines
- capability-tier-deltas-vs-counterparts-2026-05-20.md: 26 lines
- coherence-audit-2026-05-20.md: 806 lines
- faqs/compliance-engineer-faq.md: 197 lines
- feature-parity-matrix-2026-05-20.md: 405 lines
- manifest.json: 188 lines
- migration-playbooks/from-onetrust.md: 205 lines
- onboarding/compliance-engineer-first-week.md: 298 lines
- performance-benchmark-numbers-2026-05-20.md: 306 lines
- reference-implementations/pack-publish-and-conflict-rust-sdk.md: 317 lines
- tutorials/resolve-multi-pack-erasure-conflict.md: 245 lines

## Directory Deletion

- capability-tiers/ dir deleted: Y

## Vocabulary Replacement Count

- Rough replacement count: ~280 matches, including deleted capability-tiers/ content.

## Design Decisions

- Replaced capability-level compliance language with `tenant_class`, `billing_components`, `compliance_pack`, and `cell_topology`.
- Converted manifest `capability_tiers` to `tenant_class_model`.
- Collapsed the old counterpart delta memo to a supersession note that points to ADR-0329/0330/0331 and the active service docs.
- Reframed sovereign residency, regulator attestations, air-gap custody, and cross-jurisdiction transfer evidence as compliance-pack and placement requirements rather than customer ladder steps.

## Outstanding Follow-ups

- none

## Wave 15-IP-substance scrub (2026-05-21)

Bucket: IP-BUCKET-E.

Scope: `compliance`.

Rewritten in place:

- `IP-016-pack-registry-kernel.md` — expanded into a typed no-I/O pack lifecycle kernel plan with `CompliancePackManifest`, lifecycle states, pack docs, `policy/pack-overlay-authorization.cedar`, and `slos/pack-publish-soak-respected.openslo.yaml`.
- `IP-017-pack-registry-domain.md` — expanded into a side-effecting pack registry domain plan with storage ports, coverage expectation writes, Cedar subscription gates, pack conflict handling, dashboards, and ADR-COMP-001 tie-in.
- `IP-018-dpia-orchestration-usecase.md` — expanded into a GDPR Article 35 DPIA workflow with ontology inventory, LINDDUN categories, DPO review, mitigation links, pack activation block, and sealed finalization.
- `IP-019-breach-notification-workflow.md` — expanded into a breach declaration, risk assessment, authority/subject notification, US-state/KR-PIPA cascade, dashboard, SLO, and runbook plan.
- `IP-020-regulator-audit-evidence-rest.md` — expanded into an engagement-scoped auditor/regulator REST surface over `contracts/openapi.yaml`, `policy/auditor-scope.cedar`, export bundles, dashboards, and HTTP/3/ECH/PQC requirements.
- `IP-021-cell-certification-attestation-worker.md` — expanded into a per-cell/per-pack certification worker using SLOs, evidence coverage, pack registry, cloud-secrets, audit-chain, and conflict-preserving status rows.
- `IP-022-compliance-control-mapping-domain.md` — expanded into a control mapping domain for frameworks, collector bindings, satisfaction status, attestation history, and dashboard/SLO rollup.
- `IP-023-pack-registry-grpc.md` — expanded into a typed internal gRPC surface for pack manifests, requirements, and tenant subscriptions in `contracts/compliance.proto`.
- `IP-024-dpia-orchestration-adapter-postgres.md` — expanded into a Postgres/RLS adapter for DPIA records, risks, mitigations, DPO signatures, audit refs, migrations, and backup portability.
- `IP-025-breach-notification-async-emit.md` — expanded into signed AsyncAPI breach channels in `contracts/asyncapi.yaml`, replay protection, deadline metadata, and no-raw-PII payload constraints.
- `IP-026-control-mapping-rest-and-sdk.md` — expanded into REST and Rust/TypeScript SDK surfaces for frameworks, controls, collector bindings, attestation history, and tenant/validator-scoped reads.

Preserved as already substantive:

- `IP-001` through `IP-015` already contained specific compliance evidence paths, kernel/usecase/API details, DSAR and HIPAA schemas, SeaweedFS/audit-chain details, Cedar examples, acceptance evidence, and cross-references.
- Journey IPs were long-form, domain-specific, and outside the short stamped-shell cluster.

Deleted as duplicative: none.

Counterpart anchors added across rewritten IPs: Drata, Vanta, OneTrust/Tugboat Logic, AuditBoard, ServiceNow GRC, AWS Audit Manager, AWS Artifact, and Google Cloud service agents.

Verification notes:

- The eleven rewritten IPs now include bespoke A-G sections, concrete local file references, implementation steps, acceptance checks, evidence, and counterpart rows.
- Remaining 30-80 line output after the scrub is expected to include preserved substantive IPs, not just stamped shells.
