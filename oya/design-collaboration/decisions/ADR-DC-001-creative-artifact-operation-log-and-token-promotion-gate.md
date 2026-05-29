---
id: ADR-DC-001
title: Creative artifact operation log and token promotion gate
status: Proposed
date: 2026-05-20
microservice: design-collaboration
related_oyatie_adrs:
  - ADR-0003-audit-chain-and-evidence-emission
  - ADR-0007
  - ADR-0008-data-use-boundary
  - ADR-0037
  - ADR-0105-thirteen-layer-canonical-enum
  - ADR-0131-per-microservice-flat-layout
  - ADR-0145-inter-microservice-communication-reform
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0245-substrate-vs-product-layering
  - ADR-0329
  - ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components
  - ADR-0331-per-microservice-tenant-class-adoption
  - ADR-0263-observability-emission-contract
decision_owner: axis-design-collaboration
---

# ADR-DC-001: Creative artifact operation log and token promotion gate

## Context

- Architectural pressure name: multiplayer creative provenance pressure.
- Design Collaboration owns design systems, prototype links, asset versioning, and design review beyond generic drive.
- The PRD benchmark set includes Figma, Adobe Creative Cloud, Canva, InVision, and Sketch.
- The service has bounded contexts for design-file, component-library, review-comment, prototype, and brand-kit.
- Existing local policies include file open entitlement, version save control, comment thread scope, handoff export approval, token promotion, and asset preview egress.
- Existing SLOs include file load time, version save latency, comment sync latency, permission check latency, asset preview render, and design handoff export.
- Existing dashboards include local policy decisions, local domain throughput, local audit completeness, SLO burn, and compliance pack health.
- Constraint DC-C1: design files are high-churn collaborative artifacts, not static drive documents.
- Constraint DC-C2: design tokens can affect production UI and need promotion gates.
- Constraint DC-C3: review comments need anchor stability across artifact versions.
- Constraint DC-C4: asset preview generation must not leak restricted source assets across tenant or pack boundaries.
- Constraint DC-C5: brand kits can be tenant-wide and need explicit publish authority.
- Constraint DC-C6: handoff export can expose source assets, tokens, and implementation detail.
- Constraint DC-C7: source vendor imports must preserve provenance but cannot own canonical ids.
- Constraint DC-C8: Cedar must authorize every open, save, comment, token promotion, and export.
- Constraint DC-C9: operation log replay must reconstruct file state without external SaaS access.
- Constraint DC-C10: metrics must avoid raw file id and tenant id label cardinality.
- The service must support concurrent edits while preserving audit evidence.
- The service must keep design-token promotion separate from ordinary file save.
- The service must support component library review and rollback.
- The service must keep prototype publication subject to pack overlays and egress policy.
- The service must let implementation handoff export only reviewed and permitted content.
- The service must avoid a suite boundary while still owning creative artifact semantics.
- The service must let external imports from Figma or Adobe become migration evidence, not runtime authority.
- The service must preserve review comments when canvas nodes move or get renamed.
- The service must provide deterministic replay for audit and migration.

## Decision

- Decision name: CreativeArtifactLog v1.
- Adopt a per-design-file operation log with checkpointed snapshots and deterministic replay.
- Treat `CreativeArtifact` as the stable identity for design files, prototypes, component libraries, brand kits, and asset bundles.
- Treat `ArtifactOperation` as the canonical unit of collaborative change.
- Treat binary assets as content-addressed object references outside the operation log.
- Treat rendered previews as derived cache, never authority.
- Use operation sequence numbers scoped to artifact id.
- Require every operation to carry actor, tenant, principal, role, policy decision id, trace id, and prior artifact digest.
- Require operation payloads to be schema-versioned and replayable without provider calls.
- Create checkpoints every 500 operations or 5 minutes, whichever comes first.
- Require replay from latest checkpoint to current head to complete under 2 seconds p95 for normal files.
- Cap one operation payload at 256 KiB before storing payload by object reference.
- Require save p95 below 250 ms and p99 below 750 ms for operation append.
- Require file open p95 below 1500 ms for files under 50 MiB rendered state.
- Require comment sync p95 below 500 ms.
- Require preview render p95 below 2 seconds for standard frames.
- Require token promotion to be a distinct workflow state: proposed, reviewed, approved, promoted, rolled_back, or rejected.
- Require token promotion to compare semantic token diff, affected component set, and implementation handoff impact.
- Require promotion of production-bound tokens to have two approvals when pack or tenant policy demands segregation.
- Require design handoff export to bind artifact digest, token set digest, asset digest manifest, and review status.
- Require asset preview egress to deny source asset export unless the principal has handoff export approval.
- Store operation log and metadata in Postgres partitioned by tenant, cell, and artifact class.
- Store large assets in tenant-scoped object storage with content digest and retention policy.
- Publish `design.artifact.operation_appended.v1`, `design.artifact.checkpoint_created.v1`, `design.review.comment_resolved.v1`, `design.token.promotion_requested.v1`, `design.token.promoted.v1`, and `design.handoff.exported.v1`.
- Use workflow-engine for review, approval, and promotion tasks.
- Use ontology projection for CreativeArtifact, DesignToken, Component, PrototypeLink, ReviewComment, and HandoffExport.
- Make this ADR authoritative for creative artifact versioning, replay, token promotion, comment anchoring, and handoff export evidence.

## Alternatives Considered

### Alternative 1: Store one mutable JSON document per design file

- Pros: easy to read and write.
- Pros: simple import path.
- Pros: mirrors early file-storage systems.
- Cons: concurrent edits overwrite each other.
- Cons: audit replay cannot explain intermediate states.
- Cons: comment anchor stability is weak.
- Rejected because creative collaboration needs operation provenance.

### Alternative 2: Make external Figma or Adobe file ids canonical

- Pros: simpler migration from incumbent tools.
- Pros: fewer mapping layers for imported files.
- Pros: external previews can remain the initial source.
- Cons: vendor id changes break canonical identity.
- Cons: provider outage blocks replay.
- Cons: tenant residency and audit custody leave Oyatie control.
- Rejected because source vendors are import sources, not authority.

### Alternative 3: Promote design tokens by ordinary file save

- Pros: minimal workflow surface.
- Pros: fewer review states.
- Pros: faster designer iteration.
- Cons: production UI changes can bypass approval.
- Cons: rollback cannot isolate token changes.
- Cons: affected component evidence is missing.
- Rejected because tokens are production-impacting configuration.

### Alternative 4: Put comments in messenger threads only

- Pros: reuses existing conversation features.
- Pros: easier notification integration.
- Pros: fewer design-specific tables.
- Cons: comments lose anchor stability across canvas versions.
- Cons: review resolution cannot be tied to artifact digest.
- Cons: handoff export cannot prove comment closure.
- Rejected because design review comments are artifact state.

### Alternative 5: Render previews by direct source asset egress

- Pros: preview workers are simple.
- Pros: lower local transform complexity.
- Pros: fast first implementation.
- Cons: source assets may leak across packs.
- Cons: previews are not redaction-aware.
- Cons: source egress cannot be audited as derived content.
- Rejected because preview egress must be policy-bound.

## Consequences

### Positive

- Concurrent design edits become append-only evidence.
- Audit replay can reconstruct artifact state at any operation sequence.
- Comment anchors can track canvas node history.
- Token promotion is isolated from ordinary creative iteration.
- Handoff export can prove exactly which version and token set shipped.
- Provider imports can be normalized without accepting provider authority.
- Asset preview egress becomes separately controllable.
- Designers get rollback semantics at artifact, token, and export levels.
- Component library changes can be reviewed before production use.
- Pack overlays can change export rules without changing artifact identity.

### Negative

- Operation log replay requires careful schema evolution.
- Very large artifacts need checkpoint and asset-reference tuning.
- Review UIs must distinguish file save, token promotion, and handoff export.
- Import adapters must map vendor operations into Oyatie operation schema.
- Preview workers need separate redaction and egress policy checks.
- Component impact analysis needs ontology projection freshness.
- Metrics cardinality needs strict bucketing by artifact class and pack.

### Neutral

- Drive may still store exported files and attachments.
- Messenger may still notify comment events.
- Workflow-engine still owns approval task orchestration.
- Sites, slides, and application may consume promoted tokens by contract.
- External design tools may remain migration or import targets.

### Follow-up work

- Add `CreativeArtifactOperation` schema and replay fixture corpus.
- Add source-vendor import mappers for first two providers.
- Add token semantic diff fixture set.
- Add comment anchor relocation tests.
- Add preview redaction worker conformance tests.
- Add handoff export auditor dashboard.
- Add component impact graph projection in ontology.

## Implementation Notes

### Data Shapes

- `CreativeArtifact`: `artifact_id`, `tenant_id_hash`, `artifact_type`, `current_head_seq`, `current_checkpoint_id`, `status`, `data_class`, `pack_code`, `created_by`.
- `ArtifactOperation`: `artifact_id`, `operation_seq`, `operation_type`, `schema_version`, `actor_principal_id`, `payload_ref_or_inline`, `prior_digest`, `next_digest`, `policy_decision_id`.
- `ArtifactCheckpoint`: `checkpoint_id`, `artifact_id`, `operation_seq`, `state_ref`, `state_digest`, `created_at`, `replay_schema_version`.
- `ReviewComment`: `comment_id`, `artifact_id`, `anchor_path`, `anchor_fallback`, `operation_seq_opened`, `operation_seq_resolved`, `state`, `audit_event_id`.
- `DesignTokenSet`: `token_set_id`, `artifact_id`, `semantic_version`, `token_digest`, `source_operation_seq`, `promotion_state`, `approved_by`.
- `TokenPromotion`: `promotion_id`, `token_set_id`, `diff_ref`, `affected_component_refs`, `workflow_run_id`, `state`, `rollback_ref`.
- `AssetReference`: `asset_id`, `tenant_id_hash`, `artifact_id`, `content_digest`, `object_ref`, `media_type`, `egress_class`, `retention_pack`.
- `HandoffExport`: `export_id`, `artifact_id`, `token_set_id`, `asset_manifest_digest`, `review_state`, `policy_decision_id`, `evidence_id`.

### API Endpoints

- `POST /v1/design/artifacts` creates a creative artifact.
- `GET /v1/design/artifacts/{artifact_id}` returns current metadata and head digest.
- `POST /v1/design/artifacts/{artifact_id}/operations` appends one operation.
- `GET /v1/design/artifacts/{artifact_id}/replay` returns state at operation sequence.
- `POST /v1/design/artifacts/{artifact_id}/comments` opens a review comment.
- `POST /v1/design/comments/{comment_id}/resolve` resolves a review comment.
- `POST /v1/design/token-sets/{token_set_id}/promotion` requests token promotion.
- `POST /v1/design/promotions/{promotion_id}/approve` approves token promotion.
- `POST /v1/design/promotions/{promotion_id}/rollback` rolls back promoted tokens.
- `POST /v1/design/artifacts/{artifact_id}/handoff-exports` creates governed handoff export.
- `GET /v1/design/assets/{asset_id}/preview` renders policy-filtered preview.

### Cedar Policies

- `design::artifact::open` requires file entitlement, tenant match, and pack compatibility.
- `design::operation::append` requires version-save permission and matching artifact head.
- `design::comment::write` requires review-comment scope and artifact read permission.
- `design::comment::resolve` requires comment owner, reviewer, or tenant admin.
- `design::token::promotion_request` requires component-library maintainer role.
- `design::token::promote` requires approver role and segregation rule when enabled.
- `design::asset::preview` checks asset egress class before rendering.
- `design::handoff::export` requires reviewed artifact, approved token set, and export approval.
- `design::brand_kit::publish` requires tenant-wide brand-kit authority.

### SLO Targets

- `design_version_save_p95_ms` target is 250.
- `design_version_save_p99_ms` target is 750.
- `design_file_open_p95_ms` target is 1500 for normal files.
- `design_comment_sync_p95_ms` target is 500.
- `design_asset_preview_render_p95_ms` target is 2000.
- `design_handoff_export_success` target is 0.99.
- `design_policy_decision_p95_ms` target is 50.
- `design_audit_emission_lag_p95_seconds` target is 1.

## Verification

- Unit test `operation_append_requires_matching_head_digest`.
- Unit test `checkpoint_created_after_500_operations`.
- Unit test `large_operation_payload_stores_object_reference`.
- Unit test `comment_anchor_survives_node_rename`.
- Unit test `token_promotion_not_created_by_file_save`.
- Unit test `handoff_export_requires_approved_token_set`.
- Property test `operation_log_replay_matches_checkpoint_plus_tail`.
- Property test `token_semantic_diff_is_stable_for_same_inputs`.
- Property test `comment_anchor_fallback_resolves_after_canvas_move`.
- Fuzz test `vendor_operation_import_rejects_unknown_node_type`.
- Cedar test `file_open_denies_missing_entitlement`.
- Cedar test `token_promotion_requires_component_library_maintainer`.
- Cedar test `handoff_export_denies_unreviewed_artifact`.
- Cedar test `asset_preview_denies_source_egress_without_policy`.
- Cedar test `brand_kit_publish_requires_tenant_wide_authority`.
- Contract test `design_openapi_operation_append_matches_router`.
- Contract test `design_asyncapi_token_events_include_digest`.
- Contract test `design_proto_handoff_export_matches_rest_shape`.
- Integration test `concurrent_operations_append_in_sequence_order`.
- Integration test `preview_render_uses_redacted_asset_manifest`.
- Integration test `token_promotion_approves_and_publishes_event`.
- Integration test `handoff_export_contains_artifact_and_token_digest`.
- Replay test `artifact_replay_rebuilds_state_at_each_checkpoint`.
- Load test `one_hundred_concurrent_editors_single_artifact`.
- Load test `ten_thousand_comment_syncs_under_p95_budget`.
- Chaos test `object_storage_unavailable_blocks_large_asset_append`.
- Chaos test `workflow_unavailable_pauses_token_promotion`.
- Metric `oya_design_operation_append_total`.
- Metric `oya_design_operation_append_duration_ms`.
- Metric `oya_design_replay_tail_operation_count`.
- Metric `oya_design_token_promotion_state_total`.
- Metric `oya_design_handoff_export_total`.
- Dashboard `design-local-domain-throughput`.
- Dashboard `design-local-policy-decisions`.
- Dashboard `design-local-audit-completeness`.
- Dashboard `design-slo-burn`.
- Alert `DesignVersionSaveLatencyBurn`.
- Alert `DesignTokenPromotionStalled`.
- Alert `DesignHandoffExportPolicyDenySpike`.

## References

- Internal: microservices/design-collaboration/PRD.md.
- Internal: microservices/design-collaboration/ARCHITECTURE.md.
- Internal: microservices/design-collaboration/policy/creative-artifact-authorization.cedar.
- Internal: microservices/design-collaboration/policies/local-version-save-control.cedar.
- Internal: microservices/design-collaboration/policies/local-handoff-export-approval.cedar.
- Internal: microservices/design-collaboration/policies/local-asset-preview-egress.cedar.
- Internal: microservices/design-collaboration/slos/local-version-save-latency.openslo.yaml.
- Internal: microservices/design-collaboration/slos/local-design-handoff-export.openslo.yaml.
- Internal: microservices/design-collaboration/IP-028-design-token-promotion-gate.md.
- Internal: microservices/design-collaboration/IP-030-review-comment-resolution-ledger.md.
- Figma Plugin API documentation.
- Figma REST API documentation.
- Adobe Creative Cloud Libraries documentation.
- W3C Design Tokens Community Group Format Module draft.
- Martin Kleppmann, A comprehensive study of Convergent and Commutative Replicated Data Types.
- Shapiro et al., Conflict-free Replicated Data Types.
- Automerge documentation.
- Yjs documentation.
- OpenAPI Specification.
- AsyncAPI Specification.
- CloudEvents Specification.
- W3C Trace Context.
- RFC 9110: HTTP Semantics.
