# AUDIT-002 retrieval/DSR contract snapshot — 2026-07-01

Task: `t_194d4e1a` — CONTRACT-SNAPSHOT: AUDIT-002 retrieval assumptions for spec descendants.

Status: DRAFT CONTRACT SNAPSHOT ONLY. No production-readiness claim. This artifact is a source-context bridge for downstream contract/spec slices. It does not implement the audit-chain runtime, expose an API, create a CAS/WORM store, modify migrations, materialize generated faces, or assert production readiness / hyperscaler-grade maturity.

## Authority and source boundary

This snapshot is intentionally bounded to `specs/`, `docs/`, `registry/`, and `evidence/` source material inspected for the task. It separates current contract assumptions from runtime requirements because the substrate card `t_c157a3ae` is still blocked for full implementation sequencing, and because ADR-0003 / ADR-0038 are Proposed planning context while ADR-0162 plus `/specs/per-tenant-audit-log-slicing-canonical.json` provide the accepted per-tenant slicing bridge.

Sources inspected:

- `docs/AGENTS.md` — operating contract, non-claim and review discipline.
- `specs/root-hub-pointers.json` — root pointer for audit-chain and per-tenant audit slicing specs.
- `docs/decisions/ADR-0003-audit-chain-and-evidence-emission.md` — proposed append-only audit-chain and daily integrity-check model.
- `docs/decisions/ADR-0038-trust-framework-and-dsr-cascade-and-proof-of-erasure.md` — proposed DSR cascade, proof archive, and trust portal model.
- `docs/decisions/ADR-0162-per-tenant-audit-log-slicing.md` — accepted per-tenant slicing and retrieval API decision.
- `specs/per-tenant-audit-log-slicing-canonical.json` — canonical per-tenant retrieval, sharding, sealing cadence, and dedicated sovereign shard contract.
- `specs/audit-event-schema.json` — audit event envelope fields and tenant-scoping contract.
- `specs/audit-event-class-registry.json` — event-class registry/update constraints.
- `contracts/openapi/platform/platform-dsr-v1.yaml` — current DSR cascade execute contract and proof/completion fields.
- `registry/milestone-audit/index.json` — `F-AUDIT-002` naming collision note; this task uses Kanban AUDIT-002, not milestone finding `F-AUDIT-002`.

## Snapshot assumptions downstream specs may rely on

| ID | Contract assumption | Downstream impact | Reconciliation need |
|---|---|---|---|
| A-01 | `tenant_id` is the audit retrieval boundary. A tenant reads only its own seals unless an explicit Cedar grant authorizes a service/governance cross-tenant read. | Workflow, privacy, tenancy, compliance, ontology, detection, and sustainability slices can model audit evidence as tenant-scoped by default. | Future runtime card must bind this to a concrete Cedar policy, principal shape, and deny/audit event class; this snapshot does not implement the gate. |
| A-02 | Multi-tenant packs use a per-pack Merkle tree with `tenant_id` as the leaf-level partition; any pack marked `dedicated_audit_shard: true` uses dedicated per-tenant Merkle trees (sovereign packs are the named baseline, and the canonical spec also carries explicit dedicated defaults such as `pack-kr-fintech` and `pack-us-gov`). | Contract slices may distinguish shared-pack subtree proof from dedicated-shard proof without changing their local fixture shapes. | Pack overlays and generated/cloud manifests must later prove the `dedicated_audit_shard` value from source-of-truth config, not from hardcoded examples. |
| A-03 | Retrieval endpoint shape is `GET /v1/audit-chain/tenant/{tenant_id}/seals` with `since`, `until`, `event_class`, `proof`, and cursor pagination of 1000 seals/page. | Descendant API/spec fixtures can cite a stable audit retrieval surface when they need evidence lookup or trust-portal linkage. | Runtime API, OpenAPI publication, router/BFF, and generated clients remain separate implementation work. |
| A-04 | `proof=true` returns Merkle inclusion proof for each seal; proof material verifies inclusion in the published root but must not expose cross-tenant leaves or personal payload. | Consumer slices can require proof references / verification vectors in metadata-only fixtures. | Inclusion-proof serialization, root publication, verifier algorithm, and trust-portal UX are runtime/contract follow-ups. |
| A-05 | Retrieved seals are DSR-cascade-safe: hashes, ids, event metadata, data classes, and proof refs may appear; raw personal data, `subject_ref`, request payloads, and unredacted per-store records do not. | Privacy, compliance, DSR, and trust-center slices can treat audit retrieval as evidence-safe if they avoid raw DSR subject data. | DSR API currently carries `subject_ref` as `PII_IDENTIFYING`; any audit retrieval projection must map DSR completion to redacted refs, proof ids, aggregate hashes, and lawful-block reason metadata only. |
| A-06 | The DSR cascade produces per-store proof metadata (`proof_id`, `proof_method`, `evidence_hash`, `witness_ref`, `signer_ref`, `signature_ref`, `rekor_log_index`) and an aggregate completion record. | Consumer slices can reference proof ids/hashes rather than copying per-store personal data. | Event-class registry must add/confirm DSR request/dispatch/proof/completion classes before any emitted runtime claim. |
| A-07 | Audit event envelope assumptions include `tenant_id`, `audit_id`, `source_microservice`, `cell_id`, `jurisdiction_code`, trace/span/event ids, `sub_scope_path`, HLC timestamp, and sustainability/FinOps fields when using envelope v2. | Sustainability/FinOps, detection, workflow, and ontology slices can align on shared envelope dimensions instead of inventing local audit metadata. | Envelope v2 status is Proposed/advisory in `specs/audit-event-schema.json`; downstream slices should mark metadata-only/spec-ready until the registry/gate is accepted and green. |
| A-08 | Daily/hourly integrity model is a draft contract assumption: hot append target, hourly per-tenant subtree seal, daily root anchor, daily fleet/per-pack root publication where applicable, and integrity mismatch alerting. Sovereign/dedicated shards must preserve the ADR-0162 caveat that sovereign-pinned tenants publish cross-shard verification in-region rather than into the global fleet root. | Downstream slices can require freshness and integrity refs in metadata-only evidence fixtures without claiming the scheduler, anchor, or trust-portal runtime exists. | Actual schedulers, anchors, HSM keys, CAS/WORM storage, alerting, runbooks, and any elevation of ADR-0003/ADR-0038 proposed integrity-alert prose stay serialized runtime/spec-promotion work. |

## Runtime requirements explicitly out of scope for this snapshot

This task did not and must not claim completion of:

- audit-chain µservice storage, CAS/WORM implementation, shard lifecycle, or migrations;
- API gateway/router implementation for `GET /v1/audit-chain/tenant/{tenant_id}/seals`;
- Cedar runtime policy evaluation or cross-tenant grant workflows;
- HSM/KMS/sealing-key custody or Ed25519/Cosign/Rekor execution;
- Merkle inclusion-proof generation, root anchoring, trust portal publication, or verifier UX;
- DSR cascade execution or per-store erasure/correction/export integrations;
- audit event class registry expansion for `DataSubjectRequest*`, proof, tenant settings, or integrity-check classes;
- generated JSON face materialization or cloud-ci gate implementation;
- production-readiness, hyperscaler-grade, SOC2/ISO/PCI/CSAP/ISMS-P, or live compliance claims.

## Downstream consumers unblocked for contract/spec work

| Child task | Consumer slice | What it may use from this snapshot | What it must not claim yet |
|---|---|---|---|
| `t_098c2602` | WORKFLOW-001: workflow state-machine/DAG + saga contract slice | Saga audit evidence can cite tenant-scoped seal lookup, inclusion-proof refs, and HLC/idempotency dimensions. | No live workflow replay, audit-chain emission, or production router claim. |
| `t_2fc04777` | PRIVACY-001: Data Use Boundary matrix + override-pack gate slice | DUB fixtures can require zero raw PII in audit retrieval and model DSR-safe redacted proof refs. | No runtime DUB enforcement, override-pack loading, consent-store integration, or raw subject retrieval. |
| `t_68211420` | SUSTAIN-001: sustainability + FinOps audit-envelope slice | Envelope v2 fields can be used as proposed shared audit dimensions for cost/carbon rollups. | No enforced v2 envelope, measured sustainability claim, or production FinOps pipeline. |
| `t_6f562131` | DETECTION-001: detection/ML/fairness/case-management substrate slice | Chain-of-custody fixture can use tenant-scoped audit ids, event metadata, proof refs, and redacted evidence hashes. | No model runtime, case-management runtime, ML fairness gate, or emitted audit class claim. |
| `t_b4cf0774` | TENANT-004: tenant lifecycle saga + HLC/idempotency slice | Tenant offboarding/lifecycle closeout can cite dedicated/shared shard boundaries, HLC audit dimensions, and DSR-safe evidence closeout. | No tenant deletion runtime, migration, shard deletion, HSM key custody, or portability execution. |
| `t_b70d22bd` | COMPLIANCE-001: compliance-pack certification/evidence/CMP slice | Compliance evidence can cite proof ids/hashes, per-tenant retrieval, redacted proof archive semantics, and lawful-block metadata. | No certification, auditor-room publication, CMP runtime, or SOC2/ISO/PCI/CSAP/ISMS-P claim. |
| `t_fc97c058` | ONTOLOGY-001: typed-entity substrate + property-tier/RLS gate slice | Ontology mutation fixtures can cite audit-chain mutation hooks, tenant-scoped RLS assumptions, and proof refs without inventing runtime storage. | No cloud-ci gate packet, property-tier/RLS runtime, generated faces, or live ontology audit emission. |

## Reconciliation queue for later cards

1. Resolve the authority split: ADR-0162 is Accepted, but ADR-0003 / ADR-0038 remain Proposed. Downstream slices should use accepted ADR-0162 and the canonical per-tenant slicing spec for retrieval assumptions while labeling ADR-0003/0038 as planning context unless a card explicitly elevates them.
2. Decide the machine-readable home for the audit retrieval OpenAPI/schema: keep DSR cascade execute under `contracts/openapi/platform/platform-dsr-v1.yaml`, but introduce a separate audit-chain retrieval contract only on a future runtime/API card.
3. Extend or reconcile `specs/audit-event-class-registry.json` before runtime emission: current examples do not yet include the DSR/proof/integrity classes cited by ADR-0162/ADR-0038 prose.
4. Define the redaction projection from DSR completion (`subject_ref` is PII) to audit retrieval seal records (metadata/proof refs only).
5. Connect per-pack `dedicated_audit_shard` configuration to a real source of truth and validator; do not rely on prose paths or examples, and preserve the canonical-spec distinction between the named sovereign baseline and any other pack defaults marked dedicated.
6. Keep generated JSON and production runtime roots serialized under their own cards; this snapshot is not a substitute for implementation, review/fix, merge, rollout, observability, rollback, or browser/user-story evidence.

## Verification status

Because this snapshot is Markdown evidence only, JSON/schema checks are not applicable to the artifact itself; no JSON/schema or generated-face files were touched. Verification performed for this artifact:

- File readback via `read_file`: 80 lines.
- `git diff --check -- evidence/audits/audit-002-retrieval-assumptions-contract-snapshot-2026-07-01.md`: exit 0.
- Marker check for non-claim boundary, runtime/contract split, downstream child IDs, and `subject_ref` redaction caveat: pass.
- `hermes kanban --board oyatie stats`: exit 0.
- `hermes kanban --board oyatie dispatch --dry-run --max 20 --json`: exit 0, `spawnable_count=0`, `warning_count=0`.
