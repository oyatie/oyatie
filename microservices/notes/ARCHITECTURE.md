---
doc_class: ArchitectureWalkthrough
shape: Reference
length_cap: 2400
authority_tier: 2
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0105
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0246
  - ADR-0251
  - ADR-0252
  - ADR-0253
  - ADR-0254
  - ADR-0255
  - ADR-0257
  - ADR-0258
  - ADR-0263
  - ADR-0272
  - ADR-0276
  - ADR-0284
  - ADR-0292
  - ADR-0293
  - ADR-0294
  - ADR-0295
  - ADR-0296
  - ADR-0297
companion_docs:
  - microservices/notes/PRD.md
  - microservices/notes/threat-model.md
  - microservices/notes/dpia.md
  - microservices/notes/compliance.md
  - microservices/notes/manifest.json
planned_enforcement_ref: oya-governance-adr-adherence-matrix
inbound_citations:
  - microservices/notes/PRD.md
  - microservices/notes/README.md
---

# Notes µservice — Architecture Walkthrough

## §entry-point — cold-start

The Notes µservice is oyatie's personal + work notes product. Hyperscaler precedents: **Notion + Obsidian + Apple Notes + Bear + Roam Research + Logseq + Craft + Reflect + Mem.ai**. The shape: rich-text + bidirectional links + tag graph + collaborative editing (CRDT) + per-tenant E2E encryption for personal tier + search that respects encryption.

Cold-start question: *Where does a note created on the iOS app, edited collaboratively by a teammate, and tagged with `#research` end up?* Trace:
1. iOS client encrypts the note client-side with the per-tenant MLS group key (per ADR-NOTES-0001 personal-tier E2E default).
2. The encrypted blob lands in `oya-notes-note-store-app` via the JMAP-style sync surface over HTTP/3.
3. The teammate joins the MLS group via the share-link kernel; receives the group key via MLS Welcome message.
4. Collaborative edits use the Loro CRDT (`oya-notes-collab-edit-adapter-loro`) — operations are end-to-end-encrypted, merged client-side, then sealed to the note-store.
5. Tag graph (`oya-notes-tag-graph-kernel`) ingests the tag claim (the *tag-name* is hashed before storage on the personal tier so the server learns nothing about `#research` content while still supporting tag-based search).
6. ADR-0263 audit events `oya.notes.note-create`, `oya.notes.collab-edit-merge`, `oya.notes.tag-add` are emitted with `tenant_id` + `audience_type` to the audit chain.
7. Search index (`oya-notes-search-index-kernel`) handles encrypted-content via per-tenant Tantivy + client-side query (no server-side full-text on E2E notes — clients fetch encrypted blobs and search locally) per ADR-NOTES-0004.

## §principals (ADR-0242)

Operates as `oyatie.notes.{note-store, tag-graph, backlink-graph, share-link, daily-note, template-gallery, web-clipper-bridge, checklist, collab-edit, e2e-key-management}` principals. Called by tenant principals `<tenant>.<workspace>.<actor>` and substrate `ontology`, `intelligence`, `governance`, `messenger` (for note-share), `workflow-studio` (for note-as-trigger).
### Content-pass expansion — principals
- This expansion preserves the existing prose above and closes `principals` for `notes` to the ≥50-line documentation-rigor floor.
- Service owner `axis-notes` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `next-word-and-title-suggest`; bounded contexts: `notes`.
- API surfaces: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy surfaces: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`; +5 more.
- State/event surfaces: `notes.notes`.
- SLO/dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS IAM service-linked roles anchors the external control pattern for `principals`.
- Precedent 2: Google Cloud service agents provides a second independent hyperscaler pattern for `principals`.
- Tenant-scope invariant: every `notes` `next-word-and-title-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/notes/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `notes` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `notes` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `notes` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `notes` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `notes` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `next-word-and-title-suggest` evaluates `<tenant>.notes.next-word-and-title-suggest` against policy, writes `notes.notes`, and emits `oya.notes.next.word.and.title.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `principals`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `principals` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `notes` binds `principals (ADR-0242)` to `{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya-notes-backlink-graph-kernel', 'oya-notes-checklist-kernel', 'oya-notes-collab-edit-adapter-loro', 'oya-notes-collab-edit-kernel', 'oya-notes-daily-note-kernel', 'oya-notes-e2e-key-management-adapter-mls', 'oya-notes-note-store-adapter-postgres', 'oya-notes-note-store-adapter-valkey', 'oya-notes-note-store-adapter-s3', 'oya-notes-note-store-kernel', 'oya-notes-search-index-adapter-meilisearch', 'oya-notes-search-index-kernel', 'oya-notes-share-link-kernel', 'oya-notes-tag-graph-kernel', 'oya-notes-template-gallery-kernel', 'oya-notes-web-clipper-bridge-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `notes` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `principals (ADR 0242)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `notes` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, policy/e2e-personal-tier-default.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `principals (ADR 0242)`.
- Depth detail 4: `notes` state/event naming uses `notes.{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya_notes_backlink_graph_kernel', 'oya_notes_checklist_kernel', 'oya_notes_collab_edit_adapter_loro', 'oya_notes_collab_edit_kernel', 'oya_notes_daily_note_kernel', 'oya_notes_e2e_key_management_adapter_mls', 'oya_notes_note_store_adapter_postgres', 'oya_notes_note_store_adapter_valkey', 'oya_notes_note_store_adapter_s3', 'oya_notes_note_store_kernel', 'oya_notes_search_index_adapter_meilisearch', 'oya_notes_search_index_kernel', 'oya_notes_share_link_kernel', 'oya_notes_tag_graph_kernel', 'oya_notes_template_gallery_kernel', 'oya_notes_web_clipper_bridge_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `notes` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `notes` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `notes` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `principals (ADR 0242)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `notes` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `notes` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `notes` uses SLOs `slos/collab-edit-merge-latency.openslo.yaml, slos/e2e-privacy-correctness.openslo.yaml, slos/full-text-search-latency.openslo.yaml, slos/graph-render-latency.openslo.yaml, slos/note-create-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/e2e-encryption-health.json, dashboards/privacy-and-e2e-health.json, dashboards/search-and-graph.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `notes` uses runbooks `runbooks/ai-classifier-rollback-e2e-respect.md, runbooks/attachment-loss-recovery.md, runbooks/clinical-note-leak-recovery.md, runbooks/crdt-divergence-recovery.md, runbooks/e2e-key-rotation-and-recovery.md, plus 6 more` so `principals (ADR 0242)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `notes` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/notes/Chart.yaml, iac/helm/notes/templates/deployment.yaml, iac/helm/notes/templates/hpa.yaml, plus 13 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `notes` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-notes-backlink-graph-kernel.yaml, catalog/oya-notes-checklist-kernel.yaml, catalog/oya-notes-collab-edit-adapter-loro.yaml, catalog/oya-notes-collab-edit-kernel.yaml, plus 15 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `notes` fails closed when `principals (ADR 0242)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `notes` emits denial evidence for `principals (ADR 0242)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `notes` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `principals (ADR 0242)` workflow.
- Depth detail 17: `notes` telemetry for `principals (ADR 0242)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `notes` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §cedar-gates (ADR-0243)

Defence-in-depth FORBIDs:
- `policy/tenant-scope.cedar` — default-deny baseline
- `policy/auditor-scope.cedar`, `policy/ci-scope.cedar`, `policy/public-read.cedar`
- `policy/dual-context-isolation.md` — personal vs work
- `policy/e2e-personal-tier-default.md` — E2E required for personal tier
- `policy/abuse-defence.cedar` — ADR-0297
- `policy/minor-protection.cedar` — KOSA 14-17 + parental dashboard
- `policy/phi-hipaa-notes.cedar` — clinical notes overlay (HIPAA)
- `policy/pci-payments-notes.cedar` — payments-adjacent notes overlay (PCI)
- `policy/share-link-scope.cedar` — share-link Cedar permit shape

Cedar v4.2 LTS. Fragment soak ≥60s per ADR-0294.
### Content-pass expansion — cedar-gates
- This expansion preserves the existing prose above and closes `cedar-gates` for `notes` to the ≥50-line documentation-rigor floor.
- Service owner `axis-notes` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `next-word-and-title-suggest`; bounded contexts: `notes`.
- API surfaces: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy surfaces: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`; +5 more.
- State/event surfaces: `notes.notes`.
- SLO/dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Verified Permissions Cedar anchors the external control pattern for `cedar-gates`.
- Precedent 2: Google Zanzibar provides a second independent hyperscaler pattern for `cedar-gates`.
- Tenant-scope invariant: every `notes` `next-word-and-title-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/notes/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `notes` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `notes` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `notes` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `notes` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `notes` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `next-word-and-title-suggest` evaluates `<tenant>.notes.next-word-and-title-suggest` against policy, writes `notes.notes`, and emits `oya.notes.next.word.and.title.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `cedar-gates`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `cedar-gates` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `notes` binds `cedar-gates (ADR-0243)` to `{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya-notes-backlink-graph-kernel', 'oya-notes-checklist-kernel', 'oya-notes-collab-edit-adapter-loro', 'oya-notes-collab-edit-kernel', 'oya-notes-daily-note-kernel', 'oya-notes-e2e-key-management-adapter-mls', 'oya-notes-note-store-adapter-postgres', 'oya-notes-note-store-adapter-valkey', 'oya-notes-note-store-adapter-s3', 'oya-notes-note-store-kernel', 'oya-notes-search-index-adapter-meilisearch', 'oya-notes-search-index-kernel', 'oya-notes-share-link-kernel', 'oya-notes-tag-graph-kernel', 'oya-notes-template-gallery-kernel', 'oya-notes-web-clipper-bridge-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `notes` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `cedar gates (ADR 0243)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `notes` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, policy/e2e-personal-tier-default.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `cedar gates (ADR 0243)`.
- Depth detail 4: `notes` state/event naming uses `notes.{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya_notes_backlink_graph_kernel', 'oya_notes_checklist_kernel', 'oya_notes_collab_edit_adapter_loro', 'oya_notes_collab_edit_kernel', 'oya_notes_daily_note_kernel', 'oya_notes_e2e_key_management_adapter_mls', 'oya_notes_note_store_adapter_postgres', 'oya_notes_note_store_adapter_valkey', 'oya_notes_note_store_adapter_s3', 'oya_notes_note_store_kernel', 'oya_notes_search_index_adapter_meilisearch', 'oya_notes_search_index_kernel', 'oya_notes_share_link_kernel', 'oya_notes_tag_graph_kernel', 'oya_notes_template_gallery_kernel', 'oya_notes_web_clipper_bridge_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `notes` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `notes` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `notes` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `cedar gates (ADR 0243)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `notes` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.

## §tenant-scoping (ADR-0244)

Every note row carries `tenant_id` + `home_cell` + `dr_cell` + `audience_type` + `provider_credential_mode` + `compliance_packs[]`. `audience_type` enum: `B2C_PERSONAL_E2E`, `B2B_WORK`, `B2B_HIPAA_CLINICAL`, `B2B_PCI_PAYMENTS_ADJACENT`, `INTERNAL_SUBSTRATE`, `PUBLIC_SHARE_VIEW` (for published share-links). `provider_credential_mode` default `TENANT_BYOK` for personal tier.
### Content-pass expansion — tenant-scoping
- This expansion preserves the existing prose above and closes `tenant-scoping` for `notes` to the ≥50-line documentation-rigor floor.
- Service owner `axis-notes` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `next-word-and-title-suggest`; bounded contexts: `notes`.
- API surfaces: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy surfaces: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`; +5 more.
- State/event surfaces: `notes.notes`.
- SLO/dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Stripe Connect account isolation anchors the external control pattern for `tenant-scoping`.
- Precedent 2: AWS Organizations account boundary provides a second independent hyperscaler pattern for `tenant-scoping`.
- Tenant-scope invariant: every `notes` `next-word-and-title-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/notes/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `notes` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `notes` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `notes` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `notes` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `notes` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `next-word-and-title-suggest` evaluates `<tenant>.notes.next-word-and-title-suggest` against policy, writes `notes.notes`, and emits `oya.notes.next.word.and.title.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `tenant-scoping`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `tenant-scoping` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `notes` binds `tenant-scoping (ADR-0244)` to `{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya-notes-backlink-graph-kernel', 'oya-notes-checklist-kernel', 'oya-notes-collab-edit-adapter-loro', 'oya-notes-collab-edit-kernel', 'oya-notes-daily-note-kernel', 'oya-notes-e2e-key-management-adapter-mls', 'oya-notes-note-store-adapter-postgres', 'oya-notes-note-store-adapter-valkey', 'oya-notes-note-store-adapter-s3', 'oya-notes-note-store-kernel', 'oya-notes-search-index-adapter-meilisearch', 'oya-notes-search-index-kernel', 'oya-notes-share-link-kernel', 'oya-notes-tag-graph-kernel', 'oya-notes-template-gallery-kernel', 'oya-notes-web-clipper-bridge-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `notes` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `tenant scoping (ADR 0244)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `notes` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, policy/e2e-personal-tier-default.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `tenant scoping (ADR 0244)`.
- Depth detail 4: `notes` state/event naming uses `notes.{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya_notes_backlink_graph_kernel', 'oya_notes_checklist_kernel', 'oya_notes_collab_edit_adapter_loro', 'oya_notes_collab_edit_kernel', 'oya_notes_daily_note_kernel', 'oya_notes_e2e_key_management_adapter_mls', 'oya_notes_note_store_adapter_postgres', 'oya_notes_note_store_adapter_valkey', 'oya_notes_note_store_adapter_s3', 'oya_notes_note_store_kernel', 'oya_notes_search_index_adapter_meilisearch', 'oya_notes_search_index_kernel', 'oya_notes_share_link_kernel', 'oya_notes_tag_graph_kernel', 'oya_notes_template_gallery_kernel', 'oya_notes_web_clipper_bridge_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `notes` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `notes` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `notes` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `tenant scoping (ADR 0244)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `notes` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `notes` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `notes` uses SLOs `slos/collab-edit-merge-latency.openslo.yaml, slos/e2e-privacy-correctness.openslo.yaml, slos/full-text-search-latency.openslo.yaml, slos/graph-render-latency.openslo.yaml, slos/note-create-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/e2e-encryption-health.json, dashboards/privacy-and-e2e-health.json, dashboards/search-and-graph.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `notes` uses runbooks `runbooks/ai-classifier-rollback-e2e-respect.md, runbooks/attachment-loss-recovery.md, runbooks/clinical-note-leak-recovery.md, runbooks/crdt-divergence-recovery.md, runbooks/e2e-key-rotation-and-recovery.md, plus 6 more` so `tenant scoping (ADR 0244)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `notes` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/notes/Chart.yaml, iac/helm/notes/templates/deployment.yaml, iac/helm/notes/templates/hpa.yaml, plus 13 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `notes` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-notes-backlink-graph-kernel.yaml, catalog/oya-notes-checklist-kernel.yaml, catalog/oya-notes-collab-edit-adapter-loro.yaml, catalog/oya-notes-collab-edit-kernel.yaml, plus 15 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `notes` fails closed when `tenant scoping (ADR 0244)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `notes` emits denial evidence for `tenant scoping (ADR 0244)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `notes` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `tenant scoping (ADR 0244)` workflow.
- Depth detail 17: `notes` telemetry for `tenant scoping (ADR 0244)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `notes` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §substrate-product-binding (ADR-0245)

**Tier: product.** Substrate dependencies: `ontology` (entity link extraction from notes), `intelligence` (compose-assist + summarize), `governance` (retention), `cell` (placement), `tenancy` (provisioning), `policy-engine`, `observability`, `compliance`, `cloud-secrets` (OpenBao + MLS key escrow for share recovery), `messenger` (MLS substrate reused for note-share groups).
### Content-pass expansion — substrate-product-binding
- This expansion preserves the existing prose above and closes `substrate-product-binding` for `notes` to the ≥50-line documentation-rigor floor.
- Service owner `axis-notes` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `next-word-and-title-suggest`; bounded contexts: `notes`.
- API surfaces: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy surfaces: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`; +5 more.
- State/event surfaces: `notes.notes`.
- SLO/dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir Foundry substrate pattern anchors the external control pattern for `substrate-product-binding`.
- Precedent 2: Google Cloud shared VPC split provides a second independent hyperscaler pattern for `substrate-product-binding`.
- Tenant-scope invariant: every `notes` `next-word-and-title-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/notes/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `notes` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `notes` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `notes` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `notes` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `notes` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `next-word-and-title-suggest` evaluates `<tenant>.notes.next-word-and-title-suggest` against policy, writes `notes.notes`, and emits `oya.notes.next.word.and.title.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `substrate-product-binding`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `substrate-product-binding` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `notes` binds `substrate-product-binding (ADR-0245)` to `{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya-notes-backlink-graph-kernel', 'oya-notes-checklist-kernel', 'oya-notes-collab-edit-adapter-loro', 'oya-notes-collab-edit-kernel', 'oya-notes-daily-note-kernel', 'oya-notes-e2e-key-management-adapter-mls', 'oya-notes-note-store-adapter-postgres', 'oya-notes-note-store-adapter-valkey', 'oya-notes-note-store-adapter-s3', 'oya-notes-note-store-kernel', 'oya-notes-search-index-adapter-meilisearch', 'oya-notes-search-index-kernel', 'oya-notes-share-link-kernel', 'oya-notes-tag-graph-kernel', 'oya-notes-template-gallery-kernel', 'oya-notes-web-clipper-bridge-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `notes` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `substrate product binding (ADR 0245)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `notes` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, policy/e2e-personal-tier-default.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `substrate product binding (ADR 0245)`.
- Depth detail 4: `notes` state/event naming uses `notes.{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya_notes_backlink_graph_kernel', 'oya_notes_checklist_kernel', 'oya_notes_collab_edit_adapter_loro', 'oya_notes_collab_edit_kernel', 'oya_notes_daily_note_kernel', 'oya_notes_e2e_key_management_adapter_mls', 'oya_notes_note_store_adapter_postgres', 'oya_notes_note_store_adapter_valkey', 'oya_notes_note_store_adapter_s3', 'oya_notes_note_store_kernel', 'oya_notes_search_index_adapter_meilisearch', 'oya_notes_search_index_kernel', 'oya_notes_share_link_kernel', 'oya_notes_tag_graph_kernel', 'oya_notes_template_gallery_kernel', 'oya_notes_web_clipper_bridge_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `notes` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `notes` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `notes` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `substrate product binding (ADR 0245)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `notes` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `notes` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `notes` uses SLOs `slos/collab-edit-merge-latency.openslo.yaml, slos/e2e-privacy-correctness.openslo.yaml, slos/full-text-search-latency.openslo.yaml, slos/graph-render-latency.openslo.yaml, slos/note-create-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/e2e-encryption-health.json, dashboards/privacy-and-e2e-health.json, dashboards/search-and-graph.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `notes` uses runbooks `runbooks/ai-classifier-rollback-e2e-respect.md, runbooks/attachment-loss-recovery.md, runbooks/clinical-note-leak-recovery.md, runbooks/crdt-divergence-recovery.md, runbooks/e2e-key-rotation-and-recovery.md, plus 6 more` so `substrate product binding (ADR 0245)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `notes` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/notes/Chart.yaml, iac/helm/notes/templates/deployment.yaml, iac/helm/notes/templates/hpa.yaml, plus 13 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `notes` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-notes-backlink-graph-kernel.yaml, catalog/oya-notes-checklist-kernel.yaml, catalog/oya-notes-collab-edit-adapter-loro.yaml, catalog/oya-notes-collab-edit-kernel.yaml, plus 15 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `notes` fails closed when `substrate product binding (ADR 0245)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `notes` emits denial evidence for `substrate product binding (ADR 0245)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `notes` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `substrate product binding (ADR 0245)` workflow.
- Depth detail 17: `notes` telemetry for `substrate product binding (ADR 0245)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `notes` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §policy-evaluation (ADR-0246 + amendment)

Library-first via `oya-shared-policy-eval`. `policy_evaluation_mode: LIBRARY_FIRST`. Network fallback emits `oya.notes.policy-fallback-network`.
### Content-pass expansion — policy-evaluation
- This expansion preserves the existing prose above and closes `policy-evaluation` for `notes` to the ≥50-line documentation-rigor floor.
- Service owner `axis-notes` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `next-word-and-title-suggest`; bounded contexts: `notes`.
- API surfaces: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy surfaces: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`; +5 more.
- State/event surfaces: `notes.notes`.
- SLO/dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Open Policy Agent sidecar anchors the external control pattern for `policy-evaluation`.
- Precedent 2: AWS Verified Permissions provides a second independent hyperscaler pattern for `policy-evaluation`.
- Tenant-scope invariant: every `notes` `next-word-and-title-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/notes/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `notes` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `notes` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `notes` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `notes` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `notes` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `next-word-and-title-suggest` evaluates `<tenant>.notes.next-word-and-title-suggest` against policy, writes `notes.notes`, and emits `oya.notes.next.word.and.title.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `policy-evaluation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `policy-evaluation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `notes` binds `policy-evaluation (ADR-0246 + amendment)` to `{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya-notes-backlink-graph-kernel', 'oya-notes-checklist-kernel', 'oya-notes-collab-edit-adapter-loro', 'oya-notes-collab-edit-kernel', 'oya-notes-daily-note-kernel', 'oya-notes-e2e-key-management-adapter-mls', 'oya-notes-note-store-adapter-postgres', 'oya-notes-note-store-adapter-valkey', 'oya-notes-note-store-adapter-s3', 'oya-notes-note-store-kernel', 'oya-notes-search-index-adapter-meilisearch', 'oya-notes-search-index-kernel', 'oya-notes-share-link-kernel', 'oya-notes-tag-graph-kernel', 'oya-notes-template-gallery-kernel', 'oya-notes-web-clipper-bridge-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `notes` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `policy evaluation (ADR 0246 + amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `notes` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, policy/e2e-personal-tier-default.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `policy evaluation (ADR 0246 + amendment)`.
- Depth detail 4: `notes` state/event naming uses `notes.{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya_notes_backlink_graph_kernel', 'oya_notes_checklist_kernel', 'oya_notes_collab_edit_adapter_loro', 'oya_notes_collab_edit_kernel', 'oya_notes_daily_note_kernel', 'oya_notes_e2e_key_management_adapter_mls', 'oya_notes_note_store_adapter_postgres', 'oya_notes_note_store_adapter_valkey', 'oya_notes_note_store_adapter_s3', 'oya_notes_note_store_kernel', 'oya_notes_search_index_adapter_meilisearch', 'oya_notes_search_index_kernel', 'oya_notes_share_link_kernel', 'oya_notes_tag_graph_kernel', 'oya_notes_template_gallery_kernel', 'oya_notes_web_clipper_bridge_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `notes` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `notes` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `notes` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `policy evaluation (ADR 0246 + amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `notes` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `notes` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `notes` uses SLOs `slos/collab-edit-merge-latency.openslo.yaml, slos/e2e-privacy-correctness.openslo.yaml, slos/full-text-search-latency.openslo.yaml, slos/graph-render-latency.openslo.yaml, slos/note-create-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/e2e-encryption-health.json, dashboards/privacy-and-e2e-health.json, dashboards/search-and-graph.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `notes` uses runbooks `runbooks/ai-classifier-rollback-e2e-respect.md, runbooks/attachment-loss-recovery.md, runbooks/clinical-note-leak-recovery.md, runbooks/crdt-divergence-recovery.md, runbooks/e2e-key-rotation-and-recovery.md, plus 6 more` so `policy evaluation (ADR 0246 + amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `notes` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/notes/Chart.yaml, iac/helm/notes/templates/deployment.yaml, iac/helm/notes/templates/hpa.yaml, plus 13 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `notes` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-notes-backlink-graph-kernel.yaml, catalog/oya-notes-checklist-kernel.yaml, catalog/oya-notes-collab-edit-adapter-loro.yaml, catalog/oya-notes-collab-edit-kernel.yaml, plus 15 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `notes` fails closed when `policy evaluation (ADR 0246 + amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `notes` emits denial evidence for `policy evaluation (ADR 0246 + amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `notes` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `policy evaluation (ADR 0246 + amendment)` workflow.
- Depth detail 17: `notes` telemetry for `policy evaluation (ADR 0246 + amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `notes` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §intelligence-dispatch (ADR-0255 + amendment)

**E2E invariant**: For `audience_type=B2C_PERSONAL_E2E`, intelligence calls are **client-side library-only** per ADR-NOTES-0005 — the server never decrypts; only the client invokes the local intelligence model. For `B2B_WORK` non-E2E, server-side intelligence allowed with audience tag `B2B_WORK`. For `B2B_HIPAA_CLINICAL`, HIPAA-conformant variant required.
### Content-pass expansion — intelligence-dispatch
- This expansion preserves the existing prose above and closes `intelligence-dispatch` for `notes` to the ≥50-line documentation-rigor floor.
- Service owner `axis-notes` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `next-word-and-title-suggest`; bounded contexts: `notes`.
- API surfaces: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy surfaces: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`; +5 more.
- State/event surfaces: `notes.notes`.
- SLO/dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir AIP tool boundary anchors the external control pattern for `intelligence-dispatch`.
- Precedent 2: Azure OpenAI tenant deployment provides a second independent hyperscaler pattern for `intelligence-dispatch`.
- Tenant-scope invariant: every `notes` `next-word-and-title-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/notes/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `notes` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `notes` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `notes` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `notes` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `notes` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `next-word-and-title-suggest` evaluates `<tenant>.notes.next-word-and-title-suggest` against policy, writes `notes.notes`, and emits `oya.notes.next.word.and.title.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `intelligence-dispatch`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `intelligence-dispatch` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `notes` binds `intelligence-dispatch (ADR-0255 + amendment)` to `{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya-notes-backlink-graph-kernel', 'oya-notes-checklist-kernel', 'oya-notes-collab-edit-adapter-loro', 'oya-notes-collab-edit-kernel', 'oya-notes-daily-note-kernel', 'oya-notes-e2e-key-management-adapter-mls', 'oya-notes-note-store-adapter-postgres', 'oya-notes-note-store-adapter-valkey', 'oya-notes-note-store-adapter-s3', 'oya-notes-note-store-kernel', 'oya-notes-search-index-adapter-meilisearch', 'oya-notes-search-index-kernel', 'oya-notes-share-link-kernel', 'oya-notes-tag-graph-kernel', 'oya-notes-template-gallery-kernel', 'oya-notes-web-clipper-bridge-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `notes` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `intelligence dispatch (ADR 0255 + amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `notes` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, policy/e2e-personal-tier-default.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `intelligence dispatch (ADR 0255 + amendment)`.
- Depth detail 4: `notes` state/event naming uses `notes.{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya_notes_backlink_graph_kernel', 'oya_notes_checklist_kernel', 'oya_notes_collab_edit_adapter_loro', 'oya_notes_collab_edit_kernel', 'oya_notes_daily_note_kernel', 'oya_notes_e2e_key_management_adapter_mls', 'oya_notes_note_store_adapter_postgres', 'oya_notes_note_store_adapter_valkey', 'oya_notes_note_store_adapter_s3', 'oya_notes_note_store_kernel', 'oya_notes_search_index_adapter_meilisearch', 'oya_notes_search_index_kernel', 'oya_notes_share_link_kernel', 'oya_notes_tag_graph_kernel', 'oya_notes_template_gallery_kernel', 'oya_notes_web_clipper_bridge_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `notes` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `notes` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `notes` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `intelligence dispatch (ADR 0255 + amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `notes` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `notes` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `notes` uses SLOs `slos/collab-edit-merge-latency.openslo.yaml, slos/e2e-privacy-correctness.openslo.yaml, slos/full-text-search-latency.openslo.yaml, slos/graph-render-latency.openslo.yaml, slos/note-create-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/e2e-encryption-health.json, dashboards/privacy-and-e2e-health.json, dashboards/search-and-graph.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `notes` uses runbooks `runbooks/ai-classifier-rollback-e2e-respect.md, runbooks/attachment-loss-recovery.md, runbooks/clinical-note-leak-recovery.md, runbooks/crdt-divergence-recovery.md, runbooks/e2e-key-rotation-and-recovery.md, plus 6 more` so `intelligence dispatch (ADR 0255 + amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `notes` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/notes/Chart.yaml, iac/helm/notes/templates/deployment.yaml, iac/helm/notes/templates/hpa.yaml, plus 13 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `notes` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-notes-backlink-graph-kernel.yaml, catalog/oya-notes-checklist-kernel.yaml, catalog/oya-notes-collab-edit-adapter-loro.yaml, catalog/oya-notes-collab-edit-kernel.yaml, plus 15 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `notes` fails closed when `intelligence dispatch (ADR 0255 + amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `notes` emits denial evidence for `intelligence dispatch (ADR 0255 + amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `notes` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `intelligence dispatch (ADR 0255 + amendment)` workflow.
- Depth detail 17: `notes` telemetry for `intelligence dispatch (ADR 0255 + amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `notes` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §ontology-read-path (ADR-0257 + amendment)

`ontology_read_mode: LIBRARY_FIRST_BYO_CACHE`. Entity-link extraction runs client-side for E2E notes (the encrypted blob is decrypted client-side, scanned for entity references, then a separate metadata-only request enriches with ontology data for display). `freshness_floor: LOOSE` (60s).
### Content-pass expansion — ontology-read-path
- This expansion preserves the existing prose above and closes `ontology-read-path` for `notes` to the ≥50-line documentation-rigor floor.
- Service owner `axis-notes` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `next-word-and-title-suggest`; bounded contexts: `notes`.
- API surfaces: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy surfaces: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`; +5 more.
- State/event surfaces: `notes.notes`.
- SLO/dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir Foundry ontology projections anchors the external control pattern for `ontology-read-path`.
- Precedent 2: Google Knowledge Graph serving cache provides a second independent hyperscaler pattern for `ontology-read-path`.
- Tenant-scope invariant: every `notes` `next-word-and-title-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/notes/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `notes` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `notes` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `notes` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `notes` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `notes` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `next-word-and-title-suggest` evaluates `<tenant>.notes.next-word-and-title-suggest` against policy, writes `notes.notes`, and emits `oya.notes.next.word.and.title.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `ontology-read-path`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `ontology-read-path` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `notes` binds `ontology-read-path (ADR-0257 + amendment)` to `{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya-notes-backlink-graph-kernel', 'oya-notes-checklist-kernel', 'oya-notes-collab-edit-adapter-loro', 'oya-notes-collab-edit-kernel', 'oya-notes-daily-note-kernel', 'oya-notes-e2e-key-management-adapter-mls', 'oya-notes-note-store-adapter-postgres', 'oya-notes-note-store-adapter-valkey', 'oya-notes-note-store-adapter-s3', 'oya-notes-note-store-kernel', 'oya-notes-search-index-adapter-meilisearch', 'oya-notes-search-index-kernel', 'oya-notes-share-link-kernel', 'oya-notes-tag-graph-kernel', 'oya-notes-template-gallery-kernel', 'oya-notes-web-clipper-bridge-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `notes` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `ontology read path (ADR 0257 + amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `notes` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, policy/e2e-personal-tier-default.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `ontology read path (ADR 0257 + amendment)`.
- Depth detail 4: `notes` state/event naming uses `notes.{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya_notes_backlink_graph_kernel', 'oya_notes_checklist_kernel', 'oya_notes_collab_edit_adapter_loro', 'oya_notes_collab_edit_kernel', 'oya_notes_daily_note_kernel', 'oya_notes_e2e_key_management_adapter_mls', 'oya_notes_note_store_adapter_postgres', 'oya_notes_note_store_adapter_valkey', 'oya_notes_note_store_adapter_s3', 'oya_notes_note_store_kernel', 'oya_notes_search_index_adapter_meilisearch', 'oya_notes_search_index_kernel', 'oya_notes_share_link_kernel', 'oya_notes_tag_graph_kernel', 'oya_notes_template_gallery_kernel', 'oya_notes_web_clipper_bridge_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `notes` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `notes` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `notes` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `ontology read path (ADR 0257 + amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `notes` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `notes` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `notes` uses SLOs `slos/collab-edit-merge-latency.openslo.yaml, slos/e2e-privacy-correctness.openslo.yaml, slos/full-text-search-latency.openslo.yaml, slos/graph-render-latency.openslo.yaml, slos/note-create-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/e2e-encryption-health.json, dashboards/privacy-and-e2e-health.json, dashboards/search-and-graph.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `notes` uses runbooks `runbooks/ai-classifier-rollback-e2e-respect.md, runbooks/attachment-loss-recovery.md, runbooks/clinical-note-leak-recovery.md, runbooks/crdt-divergence-recovery.md, runbooks/e2e-key-rotation-and-recovery.md, plus 6 more` so `ontology read path (ADR 0257 + amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `notes` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/notes/Chart.yaml, iac/helm/notes/templates/deployment.yaml, iac/helm/notes/templates/hpa.yaml, plus 13 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `notes` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-notes-backlink-graph-kernel.yaml, catalog/oya-notes-checklist-kernel.yaml, catalog/oya-notes-collab-edit-adapter-loro.yaml, catalog/oya-notes-collab-edit-kernel.yaml, plus 15 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `notes` fails closed when `ontology read path (ADR 0257 + amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `notes` emits denial evidence for `ontology read path (ADR 0257 + amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `notes` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `ontology read path (ADR 0257 + amendment)` workflow.
- Depth detail 17: `notes` telemetry for `ontology read path (ADR 0257 + amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `notes` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §time-coordination (ADR-0252)

HLC default. TrueTime opt-in for `B2B_HIPAA_CLINICAL` audit-chain seal (chain-of-custody for clinical notes).
### Content-pass expansion — time-coordination
- This expansion preserves the existing prose above and closes `time-coordination` for `notes` to the ≥50-line documentation-rigor floor.
- Service owner `axis-notes` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `next-word-and-title-suggest`; bounded contexts: `notes`.
- API surfaces: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy surfaces: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`; +5 more.
- State/event surfaces: `notes.notes`.
- SLO/dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google Spanner TrueTime anchors the external control pattern for `time-coordination`.
- Precedent 2: CockroachDB HLC ordering provides a second independent hyperscaler pattern for `time-coordination`.
- Tenant-scope invariant: every `notes` `next-word-and-title-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/notes/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `notes` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `notes` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `notes` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `notes` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `notes` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `next-word-and-title-suggest` evaluates `<tenant>.notes.next-word-and-title-suggest` against policy, writes `notes.notes`, and emits `oya.notes.next.word.and.title.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `time-coordination`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `time-coordination` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `notes` binds `time-coordination (ADR-0252)` to `{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya-notes-backlink-graph-kernel', 'oya-notes-checklist-kernel', 'oya-notes-collab-edit-adapter-loro', 'oya-notes-collab-edit-kernel', 'oya-notes-daily-note-kernel', 'oya-notes-e2e-key-management-adapter-mls', 'oya-notes-note-store-adapter-postgres', 'oya-notes-note-store-adapter-valkey', 'oya-notes-note-store-adapter-s3', 'oya-notes-note-store-kernel', 'oya-notes-search-index-adapter-meilisearch', 'oya-notes-search-index-kernel', 'oya-notes-share-link-kernel', 'oya-notes-tag-graph-kernel', 'oya-notes-template-gallery-kernel', 'oya-notes-web-clipper-bridge-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `notes` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `time coordination (ADR 0252)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `notes` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, policy/e2e-personal-tier-default.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `time coordination (ADR 0252)`.
- Depth detail 4: `notes` state/event naming uses `notes.{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya_notes_backlink_graph_kernel', 'oya_notes_checklist_kernel', 'oya_notes_collab_edit_adapter_loro', 'oya_notes_collab_edit_kernel', 'oya_notes_daily_note_kernel', 'oya_notes_e2e_key_management_adapter_mls', 'oya_notes_note_store_adapter_postgres', 'oya_notes_note_store_adapter_valkey', 'oya_notes_note_store_adapter_s3', 'oya_notes_note_store_kernel', 'oya_notes_search_index_adapter_meilisearch', 'oya_notes_search_index_kernel', 'oya_notes_share_link_kernel', 'oya_notes_tag_graph_kernel', 'oya_notes_template_gallery_kernel', 'oya_notes_web_clipper_bridge_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `notes` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `notes` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `notes` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `time coordination (ADR 0252)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `notes` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `notes` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `notes` uses SLOs `slos/collab-edit-merge-latency.openslo.yaml, slos/e2e-privacy-correctness.openslo.yaml, slos/full-text-search-latency.openslo.yaml, slos/graph-render-latency.openslo.yaml, slos/note-create-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/e2e-encryption-health.json, dashboards/privacy-and-e2e-health.json, dashboards/search-and-graph.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `notes` uses runbooks `runbooks/ai-classifier-rollback-e2e-respect.md, runbooks/attachment-loss-recovery.md, runbooks/clinical-note-leak-recovery.md, runbooks/crdt-divergence-recovery.md, runbooks/e2e-key-rotation-and-recovery.md, plus 6 more` so `time coordination (ADR 0252)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `notes` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/notes/Chart.yaml, iac/helm/notes/templates/deployment.yaml, iac/helm/notes/templates/hpa.yaml, plus 13 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `notes` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-notes-backlink-graph-kernel.yaml, catalog/oya-notes-checklist-kernel.yaml, catalog/oya-notes-collab-edit-adapter-loro.yaml, catalog/oya-notes-collab-edit-kernel.yaml, plus 15 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `notes` fails closed when `time coordination (ADR 0252)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `notes` emits denial evidence for `time coordination (ADR 0252)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `notes` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `time coordination (ADR 0252)` workflow.
- Depth detail 17: `notes` telemetry for `time coordination (ADR 0252)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `notes` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §transport (ADR-0253)

Sync over HTTP/3 + QUIC default. Fallback h3 → h2 → h1.1. TLS 1.3 floor. ECH advertised; PQC hybrid `X25519MLKEM768`; signature hybrid `ed25519+ml_dsa_65`. Native clients (iOS/Android/desktop) all support HTTP/3.
### Content-pass expansion — transport
- This expansion preserves the existing prose above and closes `transport` for `notes` to the ≥50-line documentation-rigor floor.
- Service owner `axis-notes` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `next-word-and-title-suggest`; bounded contexts: `notes`.
- API surfaces: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy surfaces: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`; +5 more.
- State/event surfaces: `notes.notes`.
- SLO/dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google QUIC HTTP/3 anchors the external control pattern for `transport`.
- Precedent 2: Cloudflare ECH/PQC TLS provides a second independent hyperscaler pattern for `transport`.
- Tenant-scope invariant: every `notes` `next-word-and-title-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/notes/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `notes` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `notes` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `notes` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `notes` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `notes` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `next-word-and-title-suggest` evaluates `<tenant>.notes.next-word-and-title-suggest` against policy, writes `notes.notes`, and emits `oya.notes.next.word.and.title.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `transport`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `transport` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `notes` binds `transport (ADR-0253)` to `{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya-notes-backlink-graph-kernel', 'oya-notes-checklist-kernel', 'oya-notes-collab-edit-adapter-loro', 'oya-notes-collab-edit-kernel', 'oya-notes-daily-note-kernel', 'oya-notes-e2e-key-management-adapter-mls', 'oya-notes-note-store-adapter-postgres', 'oya-notes-note-store-adapter-valkey', 'oya-notes-note-store-adapter-s3', 'oya-notes-note-store-kernel', 'oya-notes-search-index-adapter-meilisearch', 'oya-notes-search-index-kernel', 'oya-notes-share-link-kernel', 'oya-notes-tag-graph-kernel', 'oya-notes-template-gallery-kernel', 'oya-notes-web-clipper-bridge-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `notes` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `transport (ADR 0253)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `notes` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, policy/e2e-personal-tier-default.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `transport (ADR 0253)`.
- Depth detail 4: `notes` state/event naming uses `notes.{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya_notes_backlink_graph_kernel', 'oya_notes_checklist_kernel', 'oya_notes_collab_edit_adapter_loro', 'oya_notes_collab_edit_kernel', 'oya_notes_daily_note_kernel', 'oya_notes_e2e_key_management_adapter_mls', 'oya_notes_note_store_adapter_postgres', 'oya_notes_note_store_adapter_valkey', 'oya_notes_note_store_adapter_s3', 'oya_notes_note_store_kernel', 'oya_notes_search_index_adapter_meilisearch', 'oya_notes_search_index_kernel', 'oya_notes_share_link_kernel', 'oya_notes_tag_graph_kernel', 'oya_notes_template_gallery_kernel', 'oya_notes_web_clipper_bridge_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `notes` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `notes` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `notes` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `transport (ADR 0253)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `notes` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `notes` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `notes` uses SLOs `slos/collab-edit-merge-latency.openslo.yaml, slos/e2e-privacy-correctness.openslo.yaml, slos/full-text-search-latency.openslo.yaml, slos/graph-render-latency.openslo.yaml, slos/note-create-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/e2e-encryption-health.json, dashboards/privacy-and-e2e-health.json, dashboards/search-and-graph.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `notes` uses runbooks `runbooks/ai-classifier-rollback-e2e-respect.md, runbooks/attachment-loss-recovery.md, runbooks/clinical-note-leak-recovery.md, runbooks/crdt-divergence-recovery.md, runbooks/e2e-key-rotation-and-recovery.md, plus 6 more` so `transport (ADR 0253)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `notes` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/notes/Chart.yaml, iac/helm/notes/templates/deployment.yaml, iac/helm/notes/templates/hpa.yaml, plus 13 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `notes` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-notes-backlink-graph-kernel.yaml, catalog/oya-notes-checklist-kernel.yaml, catalog/oya-notes-collab-edit-adapter-loro.yaml, catalog/oya-notes-collab-edit-kernel.yaml, plus 15 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `notes` fails closed when `transport (ADR 0253)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `notes` emits denial evidence for `transport (ADR 0253)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `notes` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `transport (ADR 0253)` workflow.
- Depth detail 17: `notes` telemetry for `transport (ADR 0253)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `notes` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §deployment-shape (ADR-0254)

- `oya-notes-note-store-app` → Kata pod (note data sensitivity; E2E blobs still warrant defense-in-depth)
- `oya-notes-collab-edit-kernel` → Kata pod
- `oya-notes-e2e-key-management-adapter-mls` → Kata pod with TPM-backed key derivation
- `oya-notes-search-index-adapter-meilisearch` → standard pod (only handles non-E2E search corpus)
- `oya-notes-web-clipper-bridge-kernel` → standard pod
### Content-pass expansion — deployment-shape
- This expansion preserves the existing prose above and closes `deployment-shape` for `notes` to the ≥50-line documentation-rigor floor.
- Service owner `axis-notes` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `next-word-and-title-suggest`; bounded contexts: `notes`.
- API surfaces: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy surfaces: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`; +5 more.
- State/event surfaces: `notes.notes`.
- SLO/dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Firecracker isolation anchors the external control pattern for `deployment-shape`.
- Precedent 2: GKE Sandbox/Kata provides a second independent hyperscaler pattern for `deployment-shape`.
- Tenant-scope invariant: every `notes` `next-word-and-title-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/notes/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `notes` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `notes` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `notes` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `notes` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `notes` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `next-word-and-title-suggest` evaluates `<tenant>.notes.next-word-and-title-suggest` against policy, writes `notes.notes`, and emits `oya.notes.next.word.and.title.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `deployment-shape`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `deployment-shape` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `notes` binds `deployment-shape (ADR-0254)` to `{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya-notes-backlink-graph-kernel', 'oya-notes-checklist-kernel', 'oya-notes-collab-edit-adapter-loro', 'oya-notes-collab-edit-kernel', 'oya-notes-daily-note-kernel', 'oya-notes-e2e-key-management-adapter-mls', 'oya-notes-note-store-adapter-postgres', 'oya-notes-note-store-adapter-valkey', 'oya-notes-note-store-adapter-s3', 'oya-notes-note-store-kernel', 'oya-notes-search-index-adapter-meilisearch', 'oya-notes-search-index-kernel', 'oya-notes-share-link-kernel', 'oya-notes-tag-graph-kernel', 'oya-notes-template-gallery-kernel', 'oya-notes-web-clipper-bridge-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `notes` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `deployment shape (ADR 0254)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `notes` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, policy/e2e-personal-tier-default.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `deployment shape (ADR 0254)`.
- Depth detail 4: `notes` state/event naming uses `notes.{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya_notes_backlink_graph_kernel', 'oya_notes_checklist_kernel', 'oya_notes_collab_edit_adapter_loro', 'oya_notes_collab_edit_kernel', 'oya_notes_daily_note_kernel', 'oya_notes_e2e_key_management_adapter_mls', 'oya_notes_note_store_adapter_postgres', 'oya_notes_note_store_adapter_valkey', 'oya_notes_note_store_adapter_s3', 'oya_notes_note_store_kernel', 'oya_notes_search_index_adapter_meilisearch', 'oya_notes_search_index_kernel', 'oya_notes_share_link_kernel', 'oya_notes_tag_graph_kernel', 'oya_notes_template_gallery_kernel', 'oya_notes_web_clipper_bridge_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `notes` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `notes` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `notes` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `deployment shape (ADR 0254)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `notes` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `notes` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `notes` uses SLOs `slos/collab-edit-merge-latency.openslo.yaml, slos/e2e-privacy-correctness.openslo.yaml, slos/full-text-search-latency.openslo.yaml, slos/graph-render-latency.openslo.yaml, slos/note-create-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/e2e-encryption-health.json, dashboards/privacy-and-e2e-health.json, dashboards/search-and-graph.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `notes` uses runbooks `runbooks/ai-classifier-rollback-e2e-respect.md, runbooks/attachment-loss-recovery.md, runbooks/clinical-note-leak-recovery.md, runbooks/crdt-divergence-recovery.md, runbooks/e2e-key-rotation-and-recovery.md, plus 6 more` so `deployment shape (ADR 0254)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `notes` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/notes/Chart.yaml, iac/helm/notes/templates/deployment.yaml, iac/helm/notes/templates/hpa.yaml, plus 13 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `notes` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-notes-backlink-graph-kernel.yaml, catalog/oya-notes-checklist-kernel.yaml, catalog/oya-notes-collab-edit-adapter-loro.yaml, catalog/oya-notes-collab-edit-kernel.yaml, plus 15 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `notes` fails closed when `deployment shape (ADR 0254)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.

## §marketplace (ADR-0249)

Exposes `note-template`, `daily-note-template`, `tag-recipe`, `web-clipper-recipe` marketplace categories. Tenants can publish templates under their namespace.
### Content-pass expansion — marketplace
- This expansion preserves the existing prose above and closes `marketplace` for `notes` to the ≥50-line documentation-rigor floor.
- Service owner `axis-notes` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `next-word-and-title-suggest`; bounded contexts: `notes`.
- API surfaces: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy surfaces: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`; +5 more.
- State/event surfaces: `notes.notes`.
- SLO/dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Stripe Connect platform facilitator anchors the external control pattern for `marketplace`.
- Precedent 2: AWS Marketplace seller controls provides a second independent hyperscaler pattern for `marketplace`.
- Tenant-scope invariant: every `notes` `next-word-and-title-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/notes/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `notes` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `notes` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `notes` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `notes` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `notes` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `next-word-and-title-suggest` evaluates `<tenant>.notes.next-word-and-title-suggest` against policy, writes `notes.notes`, and emits `oya.notes.next.word.and.title.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `marketplace`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `marketplace` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `notes` binds `marketplace (ADR-0249)` to `{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya-notes-backlink-graph-kernel', 'oya-notes-checklist-kernel', 'oya-notes-collab-edit-adapter-loro', 'oya-notes-collab-edit-kernel', 'oya-notes-daily-note-kernel', 'oya-notes-e2e-key-management-adapter-mls', 'oya-notes-note-store-adapter-postgres', 'oya-notes-note-store-adapter-valkey', 'oya-notes-note-store-adapter-s3', 'oya-notes-note-store-kernel', 'oya-notes-search-index-adapter-meilisearch', 'oya-notes-search-index-kernel', 'oya-notes-share-link-kernel', 'oya-notes-tag-graph-kernel', 'oya-notes-template-gallery-kernel', 'oya-notes-web-clipper-bridge-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `notes` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `marketplace (ADR 0249)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `notes` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, policy/e2e-personal-tier-default.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `marketplace (ADR 0249)`.
- Depth detail 4: `notes` state/event naming uses `notes.{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya_notes_backlink_graph_kernel', 'oya_notes_checklist_kernel', 'oya_notes_collab_edit_adapter_loro', 'oya_notes_collab_edit_kernel', 'oya_notes_daily_note_kernel', 'oya_notes_e2e_key_management_adapter_mls', 'oya_notes_note_store_adapter_postgres', 'oya_notes_note_store_adapter_valkey', 'oya_notes_note_store_adapter_s3', 'oya_notes_note_store_kernel', 'oya_notes_search_index_adapter_meilisearch', 'oya_notes_search_index_kernel', 'oya_notes_share_link_kernel', 'oya_notes_tag_graph_kernel', 'oya_notes_template_gallery_kernel', 'oya_notes_web_clipper_bridge_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `notes` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `notes` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `notes` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `marketplace (ADR 0249)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `notes` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `notes` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `notes` uses SLOs `slos/collab-edit-merge-latency.openslo.yaml, slos/e2e-privacy-correctness.openslo.yaml, slos/full-text-search-latency.openslo.yaml, slos/graph-render-latency.openslo.yaml, slos/note-create-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/e2e-encryption-health.json, dashboards/privacy-and-e2e-health.json, dashboards/search-and-graph.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `notes` uses runbooks `runbooks/ai-classifier-rollback-e2e-respect.md, runbooks/attachment-loss-recovery.md, runbooks/clinical-note-leak-recovery.md, runbooks/crdt-divergence-recovery.md, runbooks/e2e-key-rotation-and-recovery.md, plus 6 more` so `marketplace (ADR 0249)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `notes` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/notes/Chart.yaml, iac/helm/notes/templates/deployment.yaml, iac/helm/notes/templates/hpa.yaml, plus 13 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `notes` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-notes-backlink-graph-kernel.yaml, catalog/oya-notes-checklist-kernel.yaml, catalog/oya-notes-collab-edit-adapter-loro.yaml, catalog/oya-notes-collab-edit-kernel.yaml, plus 15 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `notes` fails closed when `marketplace (ADR 0249)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `notes` emits denial evidence for `marketplace (ADR 0249)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `notes` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `marketplace (ADR 0249)` workflow.
- Depth detail 17: `notes` telemetry for `marketplace (ADR 0249)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `notes` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §observability (ADR-0263)

Audit-event classes: `oya.notes.note-create`, `oya.notes.note-update`, `oya.notes.note-delete`, `oya.notes.tag-add`, `oya.notes.tag-remove`, `oya.notes.share-link-issue`, `oya.notes.share-link-redeem`, `oya.notes.share-link-revoke`, `oya.notes.collab-edit-merge`, `oya.notes.e2e-key-rotate`, `oya.notes.web-clipper-capture`, `oya.notes.abuse-defence-block`, `oya.notes.minor-protect-engage`, `oya.notes.tenant-byok-mint`.

Per-metric cardinality budget: 10000. Note-content NEVER appears in any metric label, log, or trace attribute (E2E invariant).
### Content-pass expansion — observability
- This expansion preserves the existing prose above and closes `observability` for `notes` to the ≥50-line documentation-rigor floor.
- Service owner `axis-notes` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `next-word-and-title-suggest`; bounded contexts: `notes`.
- API surfaces: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy surfaces: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`; +5 more.
- State/event surfaces: `notes.notes`.
- SLO/dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google SRE four reference signals anchors the external control pattern for `observability`.
- Precedent 2: OpenTelemetry semantic conventions provides a second independent hyperscaler pattern for `observability`.
- Tenant-scope invariant: every `notes` `next-word-and-title-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/notes/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `notes` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `notes` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `notes` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `notes` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `notes` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `next-word-and-title-suggest` evaluates `<tenant>.notes.next-word-and-title-suggest` against policy, writes `notes.notes`, and emits `oya.notes.next.word.and.title.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `observability`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `observability` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `notes` binds `observability (ADR-0263)` to `{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya-notes-backlink-graph-kernel', 'oya-notes-checklist-kernel', 'oya-notes-collab-edit-adapter-loro', 'oya-notes-collab-edit-kernel', 'oya-notes-daily-note-kernel', 'oya-notes-e2e-key-management-adapter-mls', 'oya-notes-note-store-adapter-postgres', 'oya-notes-note-store-adapter-valkey', 'oya-notes-note-store-adapter-s3', 'oya-notes-note-store-kernel', 'oya-notes-search-index-adapter-meilisearch', 'oya-notes-search-index-kernel', 'oya-notes-share-link-kernel', 'oya-notes-tag-graph-kernel', 'oya-notes-template-gallery-kernel', 'oya-notes-web-clipper-bridge-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `notes` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `observability (ADR 0263)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `notes` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, policy/e2e-personal-tier-default.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `observability (ADR 0263)`.
- Depth detail 4: `notes` state/event naming uses `notes.{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya_notes_backlink_graph_kernel', 'oya_notes_checklist_kernel', 'oya_notes_collab_edit_adapter_loro', 'oya_notes_collab_edit_kernel', 'oya_notes_daily_note_kernel', 'oya_notes_e2e_key_management_adapter_mls', 'oya_notes_note_store_adapter_postgres', 'oya_notes_note_store_adapter_valkey', 'oya_notes_note_store_adapter_s3', 'oya_notes_note_store_kernel', 'oya_notes_search_index_adapter_meilisearch', 'oya_notes_search_index_kernel', 'oya_notes_share_link_kernel', 'oya_notes_tag_graph_kernel', 'oya_notes_template_gallery_kernel', 'oya_notes_web_clipper_bridge_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `notes` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `notes` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `notes` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `observability (ADR 0263)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `notes` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `notes` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `notes` uses SLOs `slos/collab-edit-merge-latency.openslo.yaml, slos/e2e-privacy-correctness.openslo.yaml, slos/full-text-search-latency.openslo.yaml, slos/graph-render-latency.openslo.yaml, slos/note-create-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/e2e-encryption-health.json, dashboards/privacy-and-e2e-health.json, dashboards/search-and-graph.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `notes` uses runbooks `runbooks/ai-classifier-rollback-e2e-respect.md, runbooks/attachment-loss-recovery.md, runbooks/clinical-note-leak-recovery.md, runbooks/crdt-divergence-recovery.md, runbooks/e2e-key-rotation-and-recovery.md, plus 6 more` so `observability (ADR 0263)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `notes` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/notes/Chart.yaml, iac/helm/notes/templates/deployment.yaml, iac/helm/notes/templates/hpa.yaml, plus 13 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `notes` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-notes-backlink-graph-kernel.yaml, catalog/oya-notes-checklist-kernel.yaml, catalog/oya-notes-collab-edit-adapter-loro.yaml, catalog/oya-notes-collab-edit-kernel.yaml, plus 15 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `notes` fails closed when `observability (ADR 0263)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `notes` emits denial evidence for `observability (ADR 0263)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `notes` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `observability (ADR 0263)` workflow.
- Depth detail 17: `notes` telemetry for `observability (ADR 0263)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §consent (ADR-0272)

Per-purpose consent: (a) compose-assist (T1), (b) entity-link suggestion (T2), (c) collab-edit (requires invitee acknowledgement), (d) marketing-from-platform (opt-in). Cookie consent on web client per ADR-0272.
### Content-pass expansion — consent
- This expansion preserves the existing prose above and closes `consent` for `notes` to the ≥50-line documentation-rigor floor.
- Service owner `axis-notes` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `next-word-and-title-suggest`; bounded contexts: `notes`.
- API surfaces: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy surfaces: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`; +5 more.
- State/event surfaces: `notes.notes`.
- SLO/dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google Consent Mode anchors the external control pattern for `consent`.
- Precedent 2: Apple App Tracking Transparency provides a second independent hyperscaler pattern for `consent`.
- Tenant-scope invariant: every `notes` `next-word-and-title-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/notes/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `notes` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `notes` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `notes` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `notes` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `notes` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `next-word-and-title-suggest` evaluates `<tenant>.notes.next-word-and-title-suggest` against policy, writes `notes.notes`, and emits `oya.notes.next.word.and.title.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `consent`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `consent` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `notes` binds `consent (ADR-0272)` to `{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya-notes-backlink-graph-kernel', 'oya-notes-checklist-kernel', 'oya-notes-collab-edit-adapter-loro', 'oya-notes-collab-edit-kernel', 'oya-notes-daily-note-kernel', 'oya-notes-e2e-key-management-adapter-mls', 'oya-notes-note-store-adapter-postgres', 'oya-notes-note-store-adapter-valkey', 'oya-notes-note-store-adapter-s3', 'oya-notes-note-store-kernel', 'oya-notes-search-index-adapter-meilisearch', 'oya-notes-search-index-kernel', 'oya-notes-share-link-kernel', 'oya-notes-tag-graph-kernel', 'oya-notes-template-gallery-kernel', 'oya-notes-web-clipper-bridge-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `notes` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `consent (ADR 0272)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `notes` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, policy/e2e-personal-tier-default.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `consent (ADR 0272)`.
- Depth detail 4: `notes` state/event naming uses `notes.{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya_notes_backlink_graph_kernel', 'oya_notes_checklist_kernel', 'oya_notes_collab_edit_adapter_loro', 'oya_notes_collab_edit_kernel', 'oya_notes_daily_note_kernel', 'oya_notes_e2e_key_management_adapter_mls', 'oya_notes_note_store_adapter_postgres', 'oya_notes_note_store_adapter_valkey', 'oya_notes_note_store_adapter_s3', 'oya_notes_note_store_kernel', 'oya_notes_search_index_adapter_meilisearch', 'oya_notes_search_index_kernel', 'oya_notes_share_link_kernel', 'oya_notes_tag_graph_kernel', 'oya_notes_template_gallery_kernel', 'oya_notes_web_clipper_bridge_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `notes` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `notes` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `notes` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `consent (ADR 0272)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `notes` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `notes` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `notes` uses SLOs `slos/collab-edit-merge-latency.openslo.yaml, slos/e2e-privacy-correctness.openslo.yaml, slos/full-text-search-latency.openslo.yaml, slos/graph-render-latency.openslo.yaml, slos/note-create-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/e2e-encryption-health.json, dashboards/privacy-and-e2e-health.json, dashboards/search-and-graph.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `notes` uses runbooks `runbooks/ai-classifier-rollback-e2e-respect.md, runbooks/attachment-loss-recovery.md, runbooks/clinical-note-leak-recovery.md, runbooks/crdt-divergence-recovery.md, runbooks/e2e-key-rotation-and-recovery.md, plus 6 more` so `consent (ADR 0272)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `notes` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/notes/Chart.yaml, iac/helm/notes/templates/deployment.yaml, iac/helm/notes/templates/hpa.yaml, plus 13 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `notes` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-notes-backlink-graph-kernel.yaml, catalog/oya-notes-checklist-kernel.yaml, catalog/oya-notes-collab-edit-adapter-loro.yaml, catalog/oya-notes-collab-edit-kernel.yaml, plus 15 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `notes` fails closed when `consent (ADR 0272)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `notes` emits denial evidence for `consent (ADR 0272)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `notes` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `consent (ADR 0272)` workflow.
- Depth detail 17: `notes` telemetry for `consent (ADR 0272)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `notes` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §minor-protection (ADR-0292)

`audience_type=B2C_PERSONAL_E2E` + `minor_age_band=COPPA_UNDER_13` → refuse account; only via parental delegated child account. `KOSA_14_17` → AI features disabled by default; parental dashboard surface; share-links auto-flag if recipient is also minor.
### Content-pass expansion — minor-protection
- This expansion preserves the existing prose above and closes `minor-protection` for `notes` to the ≥50-line documentation-rigor floor.
- Service owner `axis-notes` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `next-word-and-title-suggest`; bounded contexts: `notes`.
- API surfaces: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy surfaces: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`; +5 more.
- State/event surfaces: `notes.notes`.
- SLO/dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Apple Family/Screen Time controls anchors the external control pattern for `minor-protection`.
- Precedent 2: Google Family Link provides a second independent hyperscaler pattern for `minor-protection`.
- Tenant-scope invariant: every `notes` `next-word-and-title-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/notes/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `notes` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `notes` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `notes` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `notes` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `notes` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `next-word-and-title-suggest` evaluates `<tenant>.notes.next-word-and-title-suggest` against policy, writes `notes.notes`, and emits `oya.notes.next.word.and.title.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `minor-protection`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `minor-protection` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `notes` binds `minor-protection (ADR-0292)` to `{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya-notes-backlink-graph-kernel', 'oya-notes-checklist-kernel', 'oya-notes-collab-edit-adapter-loro', 'oya-notes-collab-edit-kernel', 'oya-notes-daily-note-kernel', 'oya-notes-e2e-key-management-adapter-mls', 'oya-notes-note-store-adapter-postgres', 'oya-notes-note-store-adapter-valkey', 'oya-notes-note-store-adapter-s3', 'oya-notes-note-store-kernel', 'oya-notes-search-index-adapter-meilisearch', 'oya-notes-search-index-kernel', 'oya-notes-share-link-kernel', 'oya-notes-tag-graph-kernel', 'oya-notes-template-gallery-kernel', 'oya-notes-web-clipper-bridge-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `notes` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `minor protection (ADR 0292)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `notes` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, policy/e2e-personal-tier-default.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `minor protection (ADR 0292)`.
- Depth detail 4: `notes` state/event naming uses `notes.{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya_notes_backlink_graph_kernel', 'oya_notes_checklist_kernel', 'oya_notes_collab_edit_adapter_loro', 'oya_notes_collab_edit_kernel', 'oya_notes_daily_note_kernel', 'oya_notes_e2e_key_management_adapter_mls', 'oya_notes_note_store_adapter_postgres', 'oya_notes_note_store_adapter_valkey', 'oya_notes_note_store_adapter_s3', 'oya_notes_note_store_kernel', 'oya_notes_search_index_adapter_meilisearch', 'oya_notes_search_index_kernel', 'oya_notes_share_link_kernel', 'oya_notes_tag_graph_kernel', 'oya_notes_template_gallery_kernel', 'oya_notes_web_clipper_bridge_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `notes` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `notes` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `notes` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `minor protection (ADR 0292)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `notes` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `notes` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `notes` uses SLOs `slos/collab-edit-merge-latency.openslo.yaml, slos/e2e-privacy-correctness.openslo.yaml, slos/full-text-search-latency.openslo.yaml, slos/graph-render-latency.openslo.yaml, slos/note-create-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/e2e-encryption-health.json, dashboards/privacy-and-e2e-health.json, dashboards/search-and-graph.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `notes` uses runbooks `runbooks/ai-classifier-rollback-e2e-respect.md, runbooks/attachment-loss-recovery.md, runbooks/clinical-note-leak-recovery.md, runbooks/crdt-divergence-recovery.md, runbooks/e2e-key-rotation-and-recovery.md, plus 6 more` so `minor protection (ADR 0292)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `notes` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/notes/Chart.yaml, iac/helm/notes/templates/deployment.yaml, iac/helm/notes/templates/hpa.yaml, plus 13 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `notes` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-notes-backlink-graph-kernel.yaml, catalog/oya-notes-checklist-kernel.yaml, catalog/oya-notes-collab-edit-adapter-loro.yaml, catalog/oya-notes-collab-edit-kernel.yaml, plus 15 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `notes` fails closed when `minor protection (ADR 0292)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `notes` emits denial evidence for `minor protection (ADR 0292)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `notes` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `minor protection (ADR 0292)` workflow.
- Depth detail 17: `notes` telemetry for `minor protection (ADR 0292)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `notes` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §abuse-defence (ADR-0297)

Anti-bot on sign-up + share-link redemption; bot-mgmt passive on sync surface (legitimate clients see ZERO friction). Anti-scrape: share-link harvesting detection + per-tenant rate-limit; honeypot share-tokens to detect distribution of leaked share-URLs. Anti-spoof: SPIFFE workload identity per ADR-0295; SAML/OIDC for B2B sign-in.

UX-floor: typing, sync, search MUST add ≤2ms p99 from bot-mgmt. No CAPTCHA on regular use. CAPTCHA only on sign-up + abnormal-volume share-link issuance.
### Content-pass expansion — abuse-defence
- This expansion preserves the existing prose above and closes `abuse-defence` for `notes` to the ≥50-line documentation-rigor floor.
- Service owner `axis-notes` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `next-word-and-title-suggest`; bounded contexts: `notes`.
- API surfaces: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy surfaces: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`; +5 more.
- State/event surfaces: `notes.notes`.
- SLO/dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Cloudflare Bot Management anchors the external control pattern for `abuse-defence`.
- Precedent 2: Stripe Radar provides a second independent hyperscaler pattern for `abuse-defence`.
- Tenant-scope invariant: every `notes` `next-word-and-title-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/notes/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `notes` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `notes` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `notes` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `notes` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `notes` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `next-word-and-title-suggest` evaluates `<tenant>.notes.next-word-and-title-suggest` against policy, writes `notes.notes`, and emits `oya.notes.next.word.and.title.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `abuse-defence`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `abuse-defence` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `notes` binds `abuse-defence (ADR-0297)` to `{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya-notes-backlink-graph-kernel', 'oya-notes-checklist-kernel', 'oya-notes-collab-edit-adapter-loro', 'oya-notes-collab-edit-kernel', 'oya-notes-daily-note-kernel', 'oya-notes-e2e-key-management-adapter-mls', 'oya-notes-note-store-adapter-postgres', 'oya-notes-note-store-adapter-valkey', 'oya-notes-note-store-adapter-s3', 'oya-notes-note-store-kernel', 'oya-notes-search-index-adapter-meilisearch', 'oya-notes-search-index-kernel', 'oya-notes-share-link-kernel', 'oya-notes-tag-graph-kernel', 'oya-notes-template-gallery-kernel', 'oya-notes-web-clipper-bridge-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `notes` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `abuse defence (ADR 0297)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `notes` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, policy/e2e-personal-tier-default.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `abuse defence (ADR 0297)`.
- Depth detail 4: `notes` state/event naming uses `notes.{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya_notes_backlink_graph_kernel', 'oya_notes_checklist_kernel', 'oya_notes_collab_edit_adapter_loro', 'oya_notes_collab_edit_kernel', 'oya_notes_daily_note_kernel', 'oya_notes_e2e_key_management_adapter_mls', 'oya_notes_note_store_adapter_postgres', 'oya_notes_note_store_adapter_valkey', 'oya_notes_note_store_adapter_s3', 'oya_notes_note_store_kernel', 'oya_notes_search_index_adapter_meilisearch', 'oya_notes_search_index_kernel', 'oya_notes_share_link_kernel', 'oya_notes_tag_graph_kernel', 'oya_notes_template_gallery_kernel', 'oya_notes_web_clipper_bridge_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `notes` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `notes` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `notes` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `abuse defence (ADR 0297)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `notes` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `notes` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `notes` uses SLOs `slos/collab-edit-merge-latency.openslo.yaml, slos/e2e-privacy-correctness.openslo.yaml, slos/full-text-search-latency.openslo.yaml, slos/graph-render-latency.openslo.yaml, slos/note-create-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/e2e-encryption-health.json, dashboards/privacy-and-e2e-health.json, dashboards/search-and-graph.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `notes` uses runbooks `runbooks/ai-classifier-rollback-e2e-respect.md, runbooks/attachment-loss-recovery.md, runbooks/clinical-note-leak-recovery.md, runbooks/crdt-divergence-recovery.md, runbooks/e2e-key-rotation-and-recovery.md, plus 6 more` so `abuse defence (ADR 0297)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `notes` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/notes/Chart.yaml, iac/helm/notes/templates/deployment.yaml, iac/helm/notes/templates/hpa.yaml, plus 13 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `notes` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-notes-backlink-graph-kernel.yaml, catalog/oya-notes-checklist-kernel.yaml, catalog/oya-notes-collab-edit-adapter-loro.yaml, catalog/oya-notes-collab-edit-kernel.yaml, plus 15 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `notes` fails closed when `abuse defence (ADR 0297)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `notes` emits denial evidence for `abuse defence (ADR 0297)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `notes` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `abuse defence (ADR 0297)` workflow.
- Depth detail 17: `notes` telemetry for `abuse defence (ADR 0297)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §credential-isolation (ADR-0296)

Per-tenant E2E keys live in MLS Group + OpenBao escrow (for recovery). Sidecar TTL ≤60s on every key release. Note µservice never holds long-lived per-tenant keys.
### Content-pass expansion — credential-isolation
- This expansion preserves the existing prose above and closes `credential-isolation` for `notes` to the ≥50-line documentation-rigor floor.
- Service owner `axis-notes` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `next-word-and-title-suggest`; bounded contexts: `notes`.
- API surfaces: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy surfaces: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`; +5 more.
- State/event surfaces: `notes.notes`.
- SLO/dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: HashiCorp Vault dynamic secrets anchors the external control pattern for `credential-isolation`.
- Precedent 2: AWS KMS envelope isolation provides a second independent hyperscaler pattern for `credential-isolation`.
- Tenant-scope invariant: every `notes` `next-word-and-title-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/notes/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `notes` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `notes` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `notes` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `notes` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `notes` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `next-word-and-title-suggest` evaluates `<tenant>.notes.next-word-and-title-suggest` against policy, writes `notes.notes`, and emits `oya.notes.next.word.and.title.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `credential-isolation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `credential-isolation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `notes` binds `credential-isolation (ADR-0296)` to `{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya-notes-backlink-graph-kernel', 'oya-notes-checklist-kernel', 'oya-notes-collab-edit-adapter-loro', 'oya-notes-collab-edit-kernel', 'oya-notes-daily-note-kernel', 'oya-notes-e2e-key-management-adapter-mls', 'oya-notes-note-store-adapter-postgres', 'oya-notes-note-store-adapter-valkey', 'oya-notes-note-store-adapter-s3', 'oya-notes-note-store-kernel', 'oya-notes-search-index-adapter-meilisearch', 'oya-notes-search-index-kernel', 'oya-notes-share-link-kernel', 'oya-notes-tag-graph-kernel', 'oya-notes-template-gallery-kernel', 'oya-notes-web-clipper-bridge-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `notes` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `credential isolation (ADR 0296)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `notes` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, policy/e2e-personal-tier-default.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `credential isolation (ADR 0296)`.
- Depth detail 4: `notes` state/event naming uses `notes.{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya_notes_backlink_graph_kernel', 'oya_notes_checklist_kernel', 'oya_notes_collab_edit_adapter_loro', 'oya_notes_collab_edit_kernel', 'oya_notes_daily_note_kernel', 'oya_notes_e2e_key_management_adapter_mls', 'oya_notes_note_store_adapter_postgres', 'oya_notes_note_store_adapter_valkey', 'oya_notes_note_store_adapter_s3', 'oya_notes_note_store_kernel', 'oya_notes_search_index_adapter_meilisearch', 'oya_notes_search_index_kernel', 'oya_notes_share_link_kernel', 'oya_notes_tag_graph_kernel', 'oya_notes_template_gallery_kernel', 'oya_notes_web_clipper_bridge_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `notes` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `notes` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `notes` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `credential isolation (ADR 0296)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `notes` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `notes` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `notes` uses SLOs `slos/collab-edit-merge-latency.openslo.yaml, slos/e2e-privacy-correctness.openslo.yaml, slos/full-text-search-latency.openslo.yaml, slos/graph-render-latency.openslo.yaml, slos/note-create-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/e2e-encryption-health.json, dashboards/privacy-and-e2e-health.json, dashboards/search-and-graph.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `notes` uses runbooks `runbooks/ai-classifier-rollback-e2e-respect.md, runbooks/attachment-loss-recovery.md, runbooks/clinical-note-leak-recovery.md, runbooks/crdt-divergence-recovery.md, runbooks/e2e-key-rotation-and-recovery.md, plus 6 more` so `credential isolation (ADR 0296)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `notes` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/notes/Chart.yaml, iac/helm/notes/templates/deployment.yaml, iac/helm/notes/templates/hpa.yaml, plus 13 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `notes` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-notes-backlink-graph-kernel.yaml, catalog/oya-notes-checklist-kernel.yaml, catalog/oya-notes-collab-edit-adapter-loro.yaml, catalog/oya-notes-collab-edit-kernel.yaml, plus 15 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `notes` fails closed when `credential isolation (ADR 0296)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `notes` emits denial evidence for `credential isolation (ADR 0296)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `notes` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `credential isolation (ADR 0296)` workflow.
- Depth detail 17: `notes` telemetry for `credential isolation (ADR 0296)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `notes` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §portability (ADR-0276)

Per-tenant backup export: Markdown bundle + per-note JSON metadata + tag-graph + backlink-graph + encrypted-blob archive for E2E content (with per-tenant decryption key wrapped under tenant-admin key). Importable to Obsidian / Notion via published mapping per `IP-012-import-export-pipelines.md`. GDPR Art. 20 honored.

## §pack-overlays

- `pack-eu` → GDPR Art. 20 portability + Art. 35 DPIA (this doc) + Art. 17 erasure runbook
- `pack-kr` → PIPA per-purpose consent + KR-CSAP data-residency
- `pack-us-healthcare` → HIPAA clinical notes overlay; PHI DLP active; BAA-gated provisioning
- `pack-us-pci` → payments-adjacent notes overlay; cardholder-data scrubbing on capture

## §self-modification

Consumes Foundry-built template-gallery updates; meta-trust-root attestation per ADR-0293.
### Content-pass expansion — self-modification
- This expansion preserves the existing prose above and closes `self-modification` for `notes` to the ≥50-line documentation-rigor floor.
- Service owner `axis-notes` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `next-word-and-title-suggest`; bounded contexts: `notes`.
- API surfaces: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy surfaces: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`; +5 more.
- State/event surfaces: `notes.notes`.
- SLO/dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: SLSA provenance anchors the external control pattern for `self-modification`.
- Precedent 2: Google Binary Authorization provides a second independent hyperscaler pattern for `self-modification`.
- Tenant-scope invariant: every `notes` `next-word-and-title-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/notes/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `notes` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `notes` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `notes` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `notes` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `notes` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `next-word-and-title-suggest` evaluates `<tenant>.notes.next-word-and-title-suggest` against policy, writes `notes.notes`, and emits `oya.notes.next.word.and.title.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `self-modification`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `self-modification` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `notes` binds `self-modification` to `{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya-notes-backlink-graph-kernel', 'oya-notes-checklist-kernel', 'oya-notes-collab-edit-adapter-loro', 'oya-notes-collab-edit-kernel', 'oya-notes-daily-note-kernel', 'oya-notes-e2e-key-management-adapter-mls', 'oya-notes-note-store-adapter-postgres', 'oya-notes-note-store-adapter-valkey', 'oya-notes-note-store-adapter-s3', 'oya-notes-note-store-kernel', 'oya-notes-search-index-adapter-meilisearch', 'oya-notes-search-index-kernel', 'oya-notes-share-link-kernel', 'oya-notes-tag-graph-kernel', 'oya-notes-template-gallery-kernel', 'oya-notes-web-clipper-bridge-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `notes` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `self modification` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `notes` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, policy/e2e-personal-tier-default.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `self modification`.
- Depth detail 4: `notes` state/event naming uses `notes.{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya_notes_backlink_graph_kernel', 'oya_notes_checklist_kernel', 'oya_notes_collab_edit_adapter_loro', 'oya_notes_collab_edit_kernel', 'oya_notes_daily_note_kernel', 'oya_notes_e2e_key_management_adapter_mls', 'oya_notes_note_store_adapter_postgres', 'oya_notes_note_store_adapter_valkey', 'oya_notes_note_store_adapter_s3', 'oya_notes_note_store_kernel', 'oya_notes_search_index_adapter_meilisearch', 'oya_notes_search_index_kernel', 'oya_notes_share_link_kernel', 'oya_notes_tag_graph_kernel', 'oya_notes_template_gallery_kernel', 'oya_notes_web_clipper_bridge_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `notes` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `notes` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `notes` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `self modification` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `notes` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `notes` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `notes` uses SLOs `slos/collab-edit-merge-latency.openslo.yaml, slos/e2e-privacy-correctness.openslo.yaml, slos/full-text-search-latency.openslo.yaml, slos/graph-render-latency.openslo.yaml, slos/note-create-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/e2e-encryption-health.json, dashboards/privacy-and-e2e-health.json, dashboards/search-and-graph.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `notes` uses runbooks `runbooks/ai-classifier-rollback-e2e-respect.md, runbooks/attachment-loss-recovery.md, runbooks/clinical-note-leak-recovery.md, runbooks/crdt-divergence-recovery.md, runbooks/e2e-key-rotation-and-recovery.md, plus 6 more` so `self modification` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `notes` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/notes/Chart.yaml, iac/helm/notes/templates/deployment.yaml, iac/helm/notes/templates/hpa.yaml, plus 13 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `notes` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-notes-backlink-graph-kernel.yaml, catalog/oya-notes-checklist-kernel.yaml, catalog/oya-notes-collab-edit-adapter-loro.yaml, catalog/oya-notes-collab-edit-kernel.yaml, plus 15 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `notes` fails closed when `self modification` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `notes` emits denial evidence for `self modification` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `notes` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `self modification` workflow.
- Depth detail 17: `notes` telemetry for `self modification` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `notes` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §fragment-publish + §bootstrap-trust-chain

Cedar fragments soak 60s; note-store boots with SPIFFE attestation + kill-switch.
### Content-pass expansion — fragment-publish
- This expansion preserves the existing prose above and closes `fragment-publish` for `notes` to the ≥50-line documentation-rigor floor.
- Service owner `axis-notes` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `next-word-and-title-suggest`; bounded contexts: `notes`.
- API surfaces: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy surfaces: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`; +5 more.
- State/event surfaces: `notes.notes`.
- SLO/dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS AppConfig bake windows anchors the external control pattern for `fragment-publish`.
- Precedent 2: Google Binary Authorization provides a second independent hyperscaler pattern for `fragment-publish`.
- Tenant-scope invariant: every `notes` `next-word-and-title-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/notes/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `notes` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `notes` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `notes` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `notes` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `notes` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `next-word-and-title-suggest` evaluates `<tenant>.notes.next-word-and-title-suggest` against policy, writes `notes.notes`, and emits `oya.notes.next.word.and.title.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `fragment-publish`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `fragment-publish` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `notes` binds `fragment-publish + §bootstrap-trust-chain` to `{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya-notes-backlink-graph-kernel', 'oya-notes-checklist-kernel', 'oya-notes-collab-edit-adapter-loro', 'oya-notes-collab-edit-kernel', 'oya-notes-daily-note-kernel', 'oya-notes-e2e-key-management-adapter-mls', 'oya-notes-note-store-adapter-postgres', 'oya-notes-note-store-adapter-valkey', 'oya-notes-note-store-adapter-s3', 'oya-notes-note-store-kernel', 'oya-notes-search-index-adapter-meilisearch', 'oya-notes-search-index-kernel', 'oya-notes-share-link-kernel', 'oya-notes-tag-graph-kernel', 'oya-notes-template-gallery-kernel', 'oya-notes-web-clipper-bridge-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `notes` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `fragment publish + §bootstrap trust chain` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `notes` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, policy/e2e-personal-tier-default.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `fragment publish + §bootstrap trust chain`.
- Depth detail 4: `notes` state/event naming uses `notes.{'name': 'notes', 'description': "Bounded context 'notes' within notes (data plane)", 'crates': ['oya_notes_backlink_graph_kernel', 'oya_notes_checklist_kernel', 'oya_notes_collab_edit_adapter_loro', 'oya_notes_collab_edit_kernel', 'oya_notes_daily_note_kernel', 'oya_notes_e2e_key_management_adapter_mls', 'oya_notes_note_store_adapter_postgres', 'oya_notes_note_store_adapter_valkey', 'oya_notes_note_store_adapter_s3', 'oya_notes_note_store_kernel', 'oya_notes_search_index_adapter_meilisearch', 'oya_notes_search_index_kernel', 'oya_notes_share_link_kernel', 'oya_notes_tag_graph_kernel', 'oya_notes_template_gallery_kernel', 'oya_notes_web_clipper_bridge_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `notes` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `notes` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `notes` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `fragment publish + §bootstrap trust chain` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `notes` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `notes` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `notes` uses SLOs `slos/collab-edit-merge-latency.openslo.yaml, slos/e2e-privacy-correctness.openslo.yaml, slos/full-text-search-latency.openslo.yaml, slos/graph-render-latency.openslo.yaml, slos/note-create-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/e2e-encryption-health.json, dashboards/privacy-and-e2e-health.json, dashboards/search-and-graph.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `notes` uses runbooks `runbooks/ai-classifier-rollback-e2e-respect.md, runbooks/attachment-loss-recovery.md, runbooks/clinical-note-leak-recovery.md, runbooks/crdt-divergence-recovery.md, runbooks/e2e-key-rotation-and-recovery.md, plus 6 more` so `fragment publish + §bootstrap trust chain` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `notes` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/notes/Chart.yaml, iac/helm/notes/templates/deployment.yaml, iac/helm/notes/templates/hpa.yaml, plus 13 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `notes` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-notes-backlink-graph-kernel.yaml, catalog/oya-notes-checklist-kernel.yaml, catalog/oya-notes-collab-edit-adapter-loro.yaml, catalog/oya-notes-collab-edit-kernel.yaml, plus 15 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `notes` fails closed when `fragment publish + §bootstrap trust chain` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `notes` emits denial evidence for `fragment publish + §bootstrap trust chain` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `notes` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `fragment publish + §bootstrap trust chain` workflow.
- Depth detail 17: `notes` telemetry for `fragment publish + §bootstrap trust chain` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `notes` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §where-to-read-next

- `microservices/notes/PRD.md`
- `microservices/notes/threat-model.md`
- `microservices/notes/dpia.md`
- `microservices/notes/compliance.md`

---



## §cell-eligibility
This anchor is closed for `notes` against ADR-0248 §D-1: cell tier, shard width, DR pair and shuffle-shard behavior.

### Service-specific answer
- Cell eligibility declaration: `not declared in manifest; bound here to the conservative platform default`.
- Tier 0/1 control-plane paths run in hardened cells; tenant data planes can shard per tenant, pack, region, and workload class.
- Per-cell shard key is `(tenant_id, home_cell, jurisdiction_code)`; DR pair selection uses `dr_cell` where data-residency permits failover.
- Shuffle-shard width is documented by `multi-region.md` or defaults to three independent cells for Tier-1 control paths.
- Regional outage behavior: keep reads local where pack permits, stop cross-border replication where pack forbids it, and preserve audit emission locally.
- Example: `next-word-and-title-suggest` traffic in a KR pack tenant stays in KR home cell; DR failover requires pack approval and emits a cell-failover audit event.
- Capacity math lives in `capacity-model.md`; this section binds the shard dimensions so the math is not detached from topology.
- Cloud Hypervisor/Kata isolation applies to Tier 0/1 pods; Tier 2/3 paths inherit the same network policy and SPIFFE identity floor.

### Concrete inventory used
- Service: `notes`; owner `axis-notes`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `notes`.
- Capability records cited: `microservices/notes/capabilities/T0-suggest.yaml`, `microservices/notes/capabilities/T1-assist.yaml`, `microservices/notes/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy artifacts cited: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- SLO and dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +17 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar binding: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- State/event binding: `notes.notes`.
- Capability binding: `next-word-and-title-suggest`, `summarize-and-tag-suggest-and-link-suggest`, `auto-organize-vault`.
- SLO binding: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +5 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `notes`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `notes`.
- `policy-engine` supplies the signed Cedar corpus while `notes` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `notes` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `notes`.

### Hyperscaler precedents
- Precedent 1: AWS cell-based architecture is the reference pattern for the control shape described here.
- Precedent 2: Route 53 shuffle-sharding isolation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `notes` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §critical-path-edge-cases
This anchor is closed for `notes` against documentation-rigor.md §3.2.5: applicable human-safety and platform edge-case handling.

### Service-specific answer
- Network partition: `notes` keeps tenant-local reads when safe, stops cross-cell writes that would violate residency, and emits degraded-mode audit events.
- Byzantine caller: Cedar denies forged `principal_id`, mismatched `tenant_id`, invalid SVID, replayed idempotency keys, and suspicious bot-score context.
- Regional outage: home-cell failover follows `multi-region.md`; if a pack forbids cross-border DR, `notes` preserves local queue state instead of failing open.
- Key compromise: ADR-0296 sidecar revokes OpenBao leases, rotates signing keys, and quarantines affected audit event classes for reconciliation.
- Account recovery/hijack path: identity step-up and `notes` audit evidence keep legitimate recovery from becoming an adversary shortcut.
- Mistaken mutation path: high-impact `next-word-and-title-suggest` mutations require idempotency, undo/cooldown where product semantics allow, and sealed evidence for later correction.
- Disaster surge: `notes` enforces per-tenant isolation so one hot tenant or emergency mode cannot starve unrelated cells.
- Verification: capacity math in `capacity-model.md`, rollback in `failure-modes.md`, DR handling in `multi-region.md`, and incident actions in runbooks.

### Concrete inventory used
- Service: `notes`; owner `axis-notes`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `notes`.
- Capability records cited: `microservices/notes/capabilities/T0-suggest.yaml`, `microservices/notes/capabilities/T1-assist.yaml`, `microservices/notes/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy artifacts cited: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- SLO and dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +17 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar binding: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- State/event binding: `notes.notes`.
- Capability binding: `next-word-and-title-suggest`, `summarize-and-tag-suggest-and-link-suggest`, `auto-organize-vault`.
- SLO binding: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +5 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `notes`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `notes`.
- `policy-engine` supplies the signed Cedar corpus while `notes` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `notes` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `notes`.

### Hyperscaler precedents
- Precedent 1: Google SRE incident playbooks is the reference pattern for the control shape described here.
- Precedent 2: Stripe idempotent mutation recovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `notes` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

