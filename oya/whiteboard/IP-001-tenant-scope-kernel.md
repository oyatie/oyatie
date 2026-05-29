# IP-001 Whiteboard Tenant Scope Kernel

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-001-tenant-scope-kernel.md
Planning lane: B2B-leader IP substance deepening pass
Primary concern: tenant-scoped collaboration kernel for board, operation, presence, snapshot, export, and template install capabilities
Capability records: microservices/whiteboard/capabilities/board-open.yaml; microservices/whiteboard/capabilities/canvas-op-append.yaml; microservices/whiteboard/capabilities/presence-sync.yaml; microservices/whiteboard/capabilities/history-snapshot.yaml; microservices/whiteboard/capabilities/export-render.yaml; microservices/whiteboard/capabilities/template-marketplace-install.yaml
Local contracts: microservices/whiteboard/contracts/openapi-v1.yaml; microservices/whiteboard/contracts/local-openapi-v1.yaml; microservices/whiteboard/contracts/asyncapi-v1.yaml; microservices/whiteboard/contracts/local-asyncapi-v1.yaml; microservices/whiteboard/contracts/whiteboard-v1.proto; microservices/whiteboard/contracts/local-operations-v1.proto
Local policy references: microservices/whiteboard/policy/canvas-collaboration-authorization.cedar; microservices/whiteboard/policy/data-residency.md; microservices/whiteboard/policies/local-board-open-scope.cedar; microservices/whiteboard/policies/local-stroke-persistence-guard.cedar; microservices/whiteboard/policies/local-shape-update-acl.cedar; microservices/whiteboard/policies/local-cursor-broadcast-rate.cedar
Local SLO references: microservices/whiteboard/slos/local-board-load-time.openslo.yaml; microservices/whiteboard/slos/local-stroke-persistence-latency.openslo.yaml; microservices/whiteboard/slos/local-presence-freshness.openslo.yaml; microservices/whiteboard/slos/local-crdt-merge-success.openslo.yaml; microservices/whiteboard/slos/local-export-render-latency.openslo.yaml
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Benchmark displacement set: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard

## Executive Intent
- The tenant scope kernel is the first safety boundary for whiteboard because every downstream operation becomes unsafe if board identity, principal identity, tenant identity, and cell identity are inferred later.
- This IP replaces vendor-suite trust with an Oyatie-owned kernel that can displace Miro Enterprise and Mural Enterprise for enterprise collaboration without inheriting their workspace boundary assumptions.
- The same kernel must handle FigJam-style multiplayer immediacy, Lucidspark-style diagramming workflows, Whiteboard.fi-style classroom sessions, and Microsoft Whiteboard-style tenant administration without splitting the service into vendor-shaped product islands.
- The kernel owns canonical scope facts; contracts, policies, workers, dashboards, and runbooks consume those facts rather than recreating them.
- ADR-0321 remains the coverage benchmark authority; this IP translates its B2B leader requirement into whiteboard-specific tenancy rules.
- ADR-0316 remains the boundary guard; whiteboard is a flat operational microservice, not a suite folder or generic collaboration monolith.
- ADR-0314 remains the commercial guard; template and marketplace-originated board material cannot bypass DealSet settlement.
- ADR-0253-amendment remains the transport guard; scope-bearing requests must survive HTTP/3, ECH, and PQC rollout without weakening identity provenance.
- ADR-0105 remains the layer guard; kernel responsibilities stay in the kernel/domain/usecase lanes while REST, AsyncAPI, proto, adapters, and workers project those decisions.
- The outcome is a tenant-scope contract that junior implementers can build from without guessing which identifiers are mandatory, which checks are default-deny, or which evidence must be emitted.

## Local Evidence Read Before Implementation
- microservices/whiteboard/PRD.md defines the product problem as low-latency multi-user canvas operations, board history, and export semantics that are not document-file semantics.
- microservices/whiteboard/manifest.json declares bounded contexts: canvas, board-session, sticky-note, template, and export.
- microservices/whiteboard/manifest.json declares substrate dependencies: drive, messenger, meet, workflow-engine, identity, and ontology.
- microservices/whiteboard/manifest.json requires tenant_home_cell_required for eligible tier-1 and tier-2 cells.
- microservices/whiteboard/manifest.json allows sovereign pack overrides while limiting cross-cell replication to metadata unless a pack allows more.
- microservices/whiteboard/capabilities/board-open.yaml requires tenant_id, principal_id, audience_type, purpose, and data_class.
- microservices/whiteboard/capabilities/canvas-op-append.yaml repeats the same tenant scope fields for operational mutations.
- microservices/whiteboard/capabilities/presence-sync.yaml keeps presence in the same governance family rather than treating cursors as anonymous ephemera.
- microservices/whiteboard/capabilities/history-snapshot.yaml binds history evidence to tenant scope and library-first ontology reads.
- microservices/whiteboard/capabilities/export-render.yaml keeps export rendering inside the same settlement and policy-evaluation posture.
- microservices/whiteboard/capabilities/template-marketplace-install.yaml binds template installation to DealSet settlement and policy evaluation.
- microservices/whiteboard/policy/data-residency.md is the local reference for residency overlays that may constrain scope propagation.
- microservices/whiteboard/policy/canvas-collaboration-authorization.cedar is the global collaboration authorization policy reference.
- microservices/whiteboard/policies/local-board-open-scope.cedar is the local board-open guard this kernel must make evaluable.
- microservices/whiteboard/slos/local-board-load-time.openslo.yaml turns scope checks into a latency-sensitive budget, not an afterthought.
- microservices/whiteboard/slos/local-presence-freshness.openslo.yaml constrains how much scope validation may delay live cursor state.
- microservices/whiteboard/dashboards/local-policy-decisions.json is the intended evidence surface for allow/deny outcomes.
- microservices/whiteboard/dashboards/local-audit-completeness.json is the intended evidence surface for scope-to-audit completeness.
- microservices/whiteboard/runbooks/local-collaboration-acl-mismatch.md is the likely incident route when scope and ACL views disagree.
- microservices/whiteboard/runbooks/region-affinity-mismatch.md is the likely incident route when tenant home cell and request cell disagree.

## Kernel Responsibilities
- The kernel must create a BoardScope value for every board-open, canvas-op-append, presence-sync, history-snapshot, export-render, and template-marketplace-install operation.
- BoardScope must include tenant_id.
- BoardScope must include tenant_home_cell.
- BoardScope must include request_cell.
- BoardScope must include principal_id.
- BoardScope must include audience_type.
- BoardScope must include purpose.
- BoardScope must include data_class.
- BoardScope must include board_id when the operation addresses an existing board.
- BoardScope must include session_id when the operation participates in a live board session.
- BoardScope must include operation_id when the operation mutates canvas state.
- BoardScope must include idempotency_key for commands that can be retried.
- BoardScope must include traceparent and tracestate when present at the edge.
- BoardScope must include policy_pack_set for SOC-2, ISO-27001, GDPR, KR-PIPA, education, public-sector, and HIPAA overlays declared in the manifest.
- BoardScope must include marketplace_dealset_id when imported templates or marketplace assets enter the board.
- BoardScope must include source_vendor when the board or template is migrated from Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, or Microsoft Whiteboard.
- BoardScope must include ontology_projection_version when shape or board metadata is projected into ontology.
- BoardScope must include audit_chain_target for every command accepted by the service.
- BoardScope must include retention_class when history snapshots or exports are requested.
- BoardScope must include residency_zone when data-residency overlays apply.
- BoardScope must include emergency_bypass_claim only when a separate emergency-services policy authorizes it.

## Scope Construction Flow
- The REST layer receives raw request identifiers from microservices/whiteboard/contracts/openapi-v1.yaml and local-openapi-v1.yaml.
- The REST layer does not decide scope; it passes candidate fields into the usecase layer.
- The usecase layer resolves tenant_home_cell from identity and tenancy services declared as manifest dependencies.
- The usecase layer resolves board ownership from the board store, not from user-supplied request paths alone.
- The usecase layer resolves session membership before allowing presence or cursor fanout.
- The usecase layer resolves template source and marketplace settlement status before installation.
- The domain layer constructs BoardScope only after required fields are present.
- The kernel rejects missing tenant_id before policy evaluation to avoid meaningless Cedar decisions.
- The kernel rejects missing principal_id before policy evaluation to avoid anonymous collaboration writes.
- The kernel rejects missing data_class before policy evaluation to avoid pack bypass.
- The kernel rejects missing purpose before policy evaluation to preserve audit meaning.
- The kernel rejects missing tenant_home_cell when the manifest requires tenant_home_cell_required.
- The kernel rejects mismatched request_cell and tenant_home_cell unless a pack or residency rule explicitly allows the route.
- The kernel rejects cross-tenant board references even when board_id and principal_id are individually valid.
- The kernel rejects stale session_id values when a board session has been closed or transferred.
- The kernel normalizes source_vendor into the displaced benchmark vocabulary, not free text.
- The kernel records scope_hash so async workers can prove they used the same scope facts as the original command.
- The kernel stores scope_version so future migrations can replay old decisions without reinterpreting historical requests.
- The kernel emits scope_accepted or scope_rejected audit events before downstream side effects.
- The kernel returns deterministic error codes so REST and AsyncAPI projections can remain stable.
- The kernel never silently substitutes tenant, cell, purpose, or data_class defaults.

## Capability Binding Matrix
- board-open uses BoardScope to prove tenant membership, board visibility, policy pack, and board-load SLO eligibility.
- board-open displaces Miro Enterprise workspace opening by requiring tenant evidence before the board document is materialized.
- board-open displaces Mural Enterprise room access by treating room-like membership as a policy input, not the storage boundary.
- board-open displaces FigJam quick-entry behavior by preserving low-latency open while still requiring purpose and data_class.
- board-open displaces Lucidspark diagram board access by separating diagram object projection from tenant ownership.
- board-open displaces Whiteboard.fi class-board access by making education pack rules explicit.
- board-open displaces Microsoft Whiteboard tenant administration by exposing audit-ready tenant scope rather than tenant-global shortcuts.
- canvas-op-append uses BoardScope to bind every stroke, shape move, sticky note update, and connector mutation to tenant and board.
- canvas-op-append must carry operation_id and idempotency_key so retries cannot duplicate CRDT-visible state.
- canvas-op-append must preserve data_class=canvas_operation for policy and audit correlation.
- presence-sync uses BoardScope to limit cursor and presence fanout to active collaborators.
- presence-sync must treat cursor data as governed presence_cursor data, not unclassified transient traffic.
- history-snapshot uses BoardScope to bind snapshots to tenant retention, residency, and audit-chain targets.
- history-snapshot must prevent source-vendor imports from erasing original source identity.
- export-render uses BoardScope to determine export permissions, residency egress, and rendering latency class.
- export-render must prevent Microsoft Whiteboard-style tenant exports from escaping pack-specific export controls.
- template-marketplace-install uses BoardScope to bind template provenance, DealSet status, and tenant install rights.
- template-marketplace-install must distinguish first-party templates from Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, or Microsoft Whiteboard imports.
- Every capability must preserve the capability record's policyEvaluationMode=caller_side_library_first.
- Every capability must preserve the capability record's ontologyReadMode=library_first.

## Data Model Requirements
- BoardScope is immutable after construction.
- BoardScope is serializable for audit, outbox, and replay evidence.
- BoardScope is versioned independently from REST route versions.
- BoardScope stores tenant_id as an opaque tenant identifier, not a display name.
- BoardScope stores principal_id as an opaque identity principal, not email.
- BoardScope stores audience_type as an enum with COLLABORATION_USER as the default collaboration audience declared by existing IPs.
- BoardScope stores purpose as a policy-relevant enum or canonical slug.
- BoardScope stores data_class as board_object, canvas_operation, presence_cursor, export_snapshot, or template_asset.
- BoardScope stores request_cell and tenant_home_cell separately.
- BoardScope stores region_affinity_result so runbooks can diagnose mismatches.
- BoardScope stores policy_pack_set as a deterministic sorted list.
- BoardScope stores source_vendor as none or one of the displaced benchmark names.
- BoardScope stores marketplace_dealset_id as optional but required for marketplace-originated assets.
- BoardScope stores ontology_projection_version as optional but required when projection data is emitted.
- BoardScope stores audit_chain_target as required for accepted commands.
- BoardScope stores trace context fields without allowing them to override scope fields.
- BoardScope stores idempotency_key only for commands, not passive reads.
- BoardScope stores scope_hash derived from canonical fields.
- BoardScope stores created_at from service time, not client time.
- BoardScope stores created_by_usecase so audit can find the originating operation.
- BoardScope stores rejection_reason for rejected construction attempts without retaining unrelated request bodies.

## Policy Interface
- The kernel passes BoardScope into microservices/whiteboard/policy/canvas-collaboration-authorization.cedar.
- The kernel passes board-open scope into microservices/whiteboard/policies/local-board-open-scope.cedar.
- The kernel passes canvas operation scope into microservices/whiteboard/policies/local-stroke-persistence-guard.cedar and local-shape-update-acl.cedar.
- The kernel passes presence scope into microservices/whiteboard/policies/local-cursor-broadcast-rate.cedar.
- The kernel passes export scope into microservices/whiteboard/policies/local-board-export-egress.cedar.
- Policy input must include tenant_id, principal_id, audience_type, purpose, data_class, board_id, request_cell, and tenant_home_cell.
- Policy input must include policy_pack_set when compliance overlays are active.
- Policy input must include source_vendor when import or migration context exists.
- Policy input must include marketplace_dealset_id when template assets are commercially governed.
- Policy input must include emergency_bypass_claim only from the emergency-services bypass flow.
- Policy output must be allow, deny, or explicit bypass-approved.
- Policy output must include policy_id, policy_version, and evaluation_trace_id.
- Deny output must map to a stable REST problem code.
- Deny output must map to a stable async event reason.
- Deny output must increment local-policy-decisions dashboard dimensions.
- Deny output must emit audit-chain evidence when the request reached policy evaluation.
- Scope construction rejection before policy must emit a separate scope_rejected event.
- Policy evaluation must not mutate BoardScope.
- Policy evaluation must not perform ontology reads.
- Policy evaluation must not perform DealSet settlement itself.

## Persistence And Replay
- Accepted BoardScope is persisted with the command envelope.
- Accepted BoardScope is copied into the outbox envelope as scope_hash plus canonical scope fields.
- Async workers verify scope_hash before applying canvas operations.
- Replay workers verify scope_version before rebuilding board history.
- Snapshot workers verify tenant_home_cell before rendering or materializing exports.
- Backfill workers use BoardScope rather than inferring scope from legacy source ids.
- Migration import records preserve source_vendor and source_object_id for Miro Enterprise.
- Migration import records preserve source_vendor and source_object_id for Mural Enterprise.
- Migration import records preserve source_vendor and source_object_id for FigJam.
- Migration import records preserve source_vendor and source_object_id for Lucidspark.
- Migration import records preserve source_vendor and source_object_id for Whiteboard.fi.
- Migration import records preserve source_vendor and source_object_id for Microsoft Whiteboard.
- Replay never crosses tenant_id boundaries.
- Replay never crosses tenant_home_cell boundaries unless a residency pack allows metadata-only replication.
- Replay never downgrades data_class.
- Replay preserves marketplace_dealset_id for template-derived objects.
- Replay preserves audit_chain_target so compliance evidence remains linked.
- Replay records scope_version mismatches as operator-visible failures.
- Replay failures route to microservices/whiteboard/runbooks/local-regional-board-replay.md or board-history-corruption.md.
- Replay success updates audit completeness dimensions.

## Observability
- Emit metric whiteboard_scope_construct_total with result, capability, tenant, cell, data_class, source_vendor, and pack dimensions.
- Emit metric whiteboard_scope_reject_total with reason, capability, data_class, and source_vendor dimensions.
- Emit metric whiteboard_scope_policy_eval_seconds for policy call latency.
- Emit metric whiteboard_scope_hash_mismatch_total for worker replay failures.
- Emit trace span whiteboard.scope.construct around BoardScope creation.
- Emit trace span whiteboard.scope.policy_input around Cedar input construction.
- Emit structured log whiteboard.scope.accepted without raw board payloads.
- Emit structured log whiteboard.scope.rejected without raw board payloads.
- Emit audit event whiteboard.scope.accepted for accepted commands.
- Emit audit event whiteboard.scope.rejected for construction failures.
- Emit audit event whiteboard.scope.policy_denied for Cedar deny outcomes.
- Emit dashboard dimensions matching microservices/whiteboard/dashboards/local-policy-decisions.json.
- Emit audit completeness dimensions matching microservices/whiteboard/dashboards/local-audit-completeness.json.
- Attach benchmark_name when source_vendor is one of the displaced set.
- Attach capability_record_name such as whiteboard-board-open.
- Attach contract_family as REST, AsyncAPI, proto, worker, or replay.
- Attach board_session_state for presence and live-operation diagnostics.
- Attach tenant_home_cell and request_cell for region-affinity runbooks.
- Attach pack_overlay_count for compliance dashboards.
- Attach scope_version for migration and replay triage.

## Failure Modes
- Missing tenant_id fails closed with scope_missing_tenant.
- Missing principal_id fails closed with scope_missing_principal.
- Missing purpose fails closed with scope_missing_purpose.
- Missing data_class fails closed with scope_missing_data_class.
- Missing tenant_home_cell fails closed with scope_missing_home_cell.
- Board not owned by tenant fails closed with scope_board_tenant_mismatch.
- Principal not authorized for board session fails closed with scope_session_membership_mismatch.
- Request cell not allowed by tenant_home_cell fails closed with scope_cell_mismatch.
- Source vendor not recognized fails closed with scope_source_vendor_unknown.
- DealSet required but absent fails closed with scope_dealset_missing.
- Policy pack list cannot be resolved fails closed with scope_pack_resolution_failed.
- Ontology projection version missing for projected data fails closed with scope_ontology_version_missing.
- Audit chain target missing fails closed with scope_audit_target_missing.
- Idempotency key missing for mutation command fails closed with scope_idempotency_missing.
- Scope hash mismatch in worker fails closed with scope_hash_mismatch.
- Scope version unsupported in replay fails closed with scope_version_unsupported.
- Emergency bypass claim without policy approval fails closed with scope_bypass_denied.
- Residency overlay conflict fails closed with scope_residency_conflict.
- Retention class conflict fails closed with scope_retention_conflict.
- Unknown capability fails closed with scope_capability_unknown.

## Implementation Steps
- Define BoardScope in the kernel layer named by ADR-0105.
- Add BoardScope construction usecases for board-open and canvas-op-append first.
- Add BoardScope construction usecases for presence-sync after session membership lookup exists.
- Add BoardScope construction usecases for history-snapshot and export-render after retention and residency inputs exist.
- Add BoardScope construction usecase for template-marketplace-install after DealSet lookup exists.
- Wire REST command handlers to pass raw identifiers into the usecase layer.
- Wire AsyncAPI command envelopes to carry scope_hash and scope_version.
- Wire proto internal calls to carry BoardScope or scope_hash according to call direction.
- Add policy input mappers for each local Cedar policy file.
- Add source_vendor normalization for Miro Enterprise.
- Add source_vendor normalization for Mural Enterprise.
- Add source_vendor normalization for FigJam.
- Add source_vendor normalization for Lucidspark.
- Add source_vendor normalization for Whiteboard.fi.
- Add source_vendor normalization for Microsoft Whiteboard.
- Add audit event builders for scope accept, reject, and policy deny.
- Add metrics dimensions for capability, data_class, source_vendor, tenant_home_cell, and request_cell.
- Add replay verifier for scope_hash.
- Add migration verifier for source_object_id and source_vendor.
- Add operator-facing error mapping in REST and async worker failure events.
- Add documentation links from implementation comments to this IP only where non-obvious.

## Test Plan
- Unit test BoardScope rejects missing tenant_id.
- Unit test BoardScope rejects missing principal_id.
- Unit test BoardScope rejects missing purpose.
- Unit test BoardScope rejects missing data_class.
- Unit test BoardScope rejects tenant_home_cell absence.
- Unit test BoardScope rejects board tenant mismatch.
- Unit test BoardScope rejects session membership mismatch.
- Unit test BoardScope rejects unsupported source_vendor.
- Unit test BoardScope accepts Miro Enterprise source_vendor.
- Unit test BoardScope accepts Mural Enterprise source_vendor.
- Unit test BoardScope accepts FigJam source_vendor.
- Unit test BoardScope accepts Lucidspark source_vendor.
- Unit test BoardScope accepts Whiteboard.fi source_vendor.
- Unit test BoardScope accepts Microsoft Whiteboard source_vendor.
- Property test scope_hash remains stable under canonical field ordering.
- Property test scope_hash changes when tenant_id changes.
- Property test scope_hash changes when data_class changes.
- Contract test REST board-open requires tenant scope fields.
- Contract test AsyncAPI canvas-op-append carries scope_hash.
- Proto test internal operations preserve scope_version.
- Cedar integration test local-board-open-scope receives complete input.
- Cedar integration test local-stroke-persistence-guard receives operation_id.
- Cedar integration test local-cursor-broadcast-rate receives session_id.
- Replay test rejects scope_hash mismatch.
- Migration test preserves source_vendor and source_object_id.
- Audit test emits scope_accepted for accepted command.
- Audit test emits scope_rejected for construction failure.
- Audit test emits scope_policy_denied for Cedar deny.
- Dashboard test dimensions match local-policy-decisions expectations.
- SLO smoke test scope construction stays inside board-open latency budget.

## Acceptance Criteria
- Every listed capability record can construct BoardScope from required fields.
- Every accepted command has tenant_id, principal_id, audience_type, purpose, data_class, tenant_home_cell, request_cell, audit_chain_target, and scope_hash.
- Every mutation command has idempotency_key.
- Every marketplace template install has marketplace_dealset_id.
- Every migrated source from Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, or Microsoft Whiteboard has source_vendor and source_object_id.
- Every policy evaluation receives BoardScope-derived input.
- Every construction rejection emits deterministic error and audit evidence.
- Every policy denial emits deterministic error and audit evidence.
- Every outbox event carries scope_hash and scope_version.
- Every replay worker verifies scope_hash before side effects.
- Every export worker verifies residency and retention scope before rendering.
- Every local dashboard receives capability, data_class, source_vendor, tenant_home_cell, and result dimensions.
- No implementation infers tenant scope from board_id alone.
- No implementation treats presence as unclassified transient traffic.
- No implementation bypasses DealSet settlement for marketplace template material.
- No implementation edits ADR-0321 or changes the service boundary doctrine.

## Title-Specific Command, Event, And Proto Deltas
- BoardScopeCommandEnvelope is required for every command that enters the kernel.
- BoardScopeCommandEnvelope carries tenant_id before board_id so command parsing cannot use board lookup as tenancy proof.
- BoardScopeCommandEnvelope carries request_cell and tenant_home_cell separately so region-affinity failures are explainable.
- BoardScopeCommandEnvelope carries source_vendor only after normalization to the displaced benchmark set.
- BoardScopeCommandEnvelope carries marketplace_dealset_id only for template or marketplace-derived board material.
- BoardScopeCommandEnvelope carries operation_id for canvas-op-append.
- BoardScopeCommandEnvelope carries session_id for presence-sync and live board-open joins.
- BoardScopeCommandEnvelope carries snapshot_id for history-snapshot replay reads.
- BoardScopeCommandEnvelope carries export_id for export-render status and egress decisions.
- BoardScopeAccepted event must be emitted before any downstream operation event.
- BoardScopeRejected event must be emitted before Cedar evaluation when required fields are absent.
- BoardScopePolicyReady event must be emitted after a complete Cedar input is built.
- BoardScopeWorkerVerified event must be emitted by async workers before side effects.
- BoardScopeReplayVerified event must be emitted by replay workers before rebuilding board state.
- whiteboard-v1.proto must expose BoardScope or a BoardScopeRef message for internal calls.
- local-operations-v1.proto must include scope_hash and scope_version on worker commands.
- Proto messages must preserve source_vendor and source_object_id for benchmark imports.
- Proto messages must preserve marketplace_dealset_id for template installs.
- Proto messages must preserve residency_zone and retention_class for snapshots and exports.
- Proto messages must not allow board_id-only commands.

## Title-Specific Canvas, CRDT, And Session Facts
- The tenant scope kernel treats CRDT clocks as scoped board facts, never tenant facts.
- The tenant scope kernel treats operation_sequence as scoped to board_id and tenant_id.
- The tenant scope kernel treats active session membership as a scope prerequisite for live mutations.
- The tenant scope kernel treats facilitator role as session-scoped evidence, not tenant-wide authority.
- The tenant scope kernel treats cursor fanout as session-scoped presence, not global pubsub.
- The tenant scope kernel treats history snapshots as board-scoped immutable evidence.
- The tenant scope kernel treats export renders as board-scoped artifacts with residency and retention overlays.
- The tenant scope kernel treats template installs as board-scoped materialization plus commercial provenance.
- The tenant scope kernel treats imported Miro Enterprise frames as board-scoped objects.
- The tenant scope kernel treats imported Mural Enterprise rooms as source provenance, not scope.
- The tenant scope kernel treats imported FigJam widgets as object provenance needing template/app review.
- The tenant scope kernel treats imported Lucidspark connectors as operation graph facts.
- The tenant scope kernel treats Whiteboard.fi rosters as education-pack session facts.
- The tenant scope kernel treats Microsoft Whiteboard meeting links as session binding facts.
- The tenant scope kernel rejects CRDT merge attempts that lack matching scope_hash.

## Title-Specific SLO And Evidence Gates
- Scope construction p95 must be budgeted under local-board-load-time for board-open.
- Scope construction p95 must be budgeted under local-stroke-persistence-latency for canvas-op-append.
- Scope construction p95 must be budgeted under local-presence-freshness for presence-sync.
- Scope hash verification must be included in local-crdt-merge-success evidence.
- Scope verification for export-render must be included in local-export-render-latency evidence.
- Audit emission for scope accepted, rejected, and policy-ready events must stay inside audit-emission-lag.
- Evidence fields must include scope_version, scope_hash, tenant_home_cell, request_cell, source_vendor, and capability_record_name.
- Evidence fields must include workflow_template_id when IP-004 templates call the kernel.
- Evidence fields must include event_id when IP-006 async events carry scope across workers.
- Evidence fields must include proto_message_name when internal calls are the source of scope material.

## Rollback
- Disable capability routing at the usecase layer rather than loosening BoardScope rules.
- Preserve persisted BoardScope records during rollback so replay and audit remain explainable.
- Roll back REST route exposure before rolling back kernel construction.
- Roll back AsyncAPI publication before rolling back outbox scope verification.
- Roll back worker consumers before relaxing scope_hash checks.
- Keep source_vendor normalization in place once any migration evidence has been emitted.
- Keep audit event schemas stable once compliance evidence has been generated.
- Route rollback incidents to local-collaboration-acl-mismatch, local-regional-board-replay, or region-affinity-mismatch runbooks based on failure type.
- Treat any request to bypass tenant_id, principal_id, purpose, data_class, or audit_chain_target as a new ADR-level decision, not an implementation tweak.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
