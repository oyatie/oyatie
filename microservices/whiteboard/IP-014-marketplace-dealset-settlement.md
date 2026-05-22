# IP-014 Whiteboard marketplace-dealset-settlement

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-014-marketplace-dealset-settlement.md
Benchmarks: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo-local references: microservices/whiteboard/PRD.md, microservices/whiteboard/ARCHITECTURE.md, microservices/whiteboard/capabilities/template-marketplace-install.yaml, microservices/whiteboard/capabilities/export-render.yaml, microservices/whiteboard/capabilities/history-snapshot.yaml, microservices/whiteboard/catalog, microservices/whiteboard/compliance.md, microservices/whiteboard/cost-budget.md, microservices/whiteboard/dpia.md, microservices/whiteboard/scorecards

## Objective
- Turn template-marketplace-install from a stamped capability into a settlement-governed commercial path.
- Preserve ADR-0314 DealSet settlement as a first-class obligation inside whiteboard plans.
- Preserve ADR-0321 coverage for benchmark parity, tenant scoping, audit, rollback, and pack overlay.
- Ensure template installs, paid stencil packs, premium export styles, and migration helpers do not bypass marketplace economics.
- Keep the product boundary flat: Whiteboard owns install semantics, not a vendor suite boundary.
- Treat Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard as displaced marketplace expectations.
- Make settlement evidence suitable for finance, auditor, tenant admin, and marketplace operator review.
- Make commercial obligations visible without leaking vendor-specific labels into durable object names.
- Keep board collaboration fast while settlement runs idempotently and asynchronously where possible.
- Avoid template access when the DealSet cannot be proven.

## Non-goals
- Do not build a generic marketplace service in this IP.
- Do not change payment rails, invoice posting, or marketplace ADRs.
- Do not make template-marketplace-install the owner of board-open or canvas-op-append.
- Do not allow a paid template to become available on failed settlement.
- Do not store vendor account identifiers as canonical board identity.
- Do not collapse free first-party templates and paid marketplace packs into one evidence path.
- Do not edit catalog files in this slice.
- Do not modify ADR-0314 or ADR-0321.
- Do not weaken Cedar or data residency controls for commercial convenience.
- Do not rely on post-hoc reconciliation as the normal success path.

## Settlement surface
- The initiating capability is `template-marketplace-install`.
- Settlement may also be triggered by premium export-render styles.
- Settlement may also be triggered by history-snapshot recovery packs when a vendor migration helper is licensed.
- The settlement request includes tenant_id, principal_id, marketplace_pack_id, DealSet id, price basis, jurisdiction_code, and audit event class.
- The install request includes board id only after policy and settlement preflight pass.
- The installed template copy records source marketplace pack, version, license scope, and rollback id.
- The board object never stores payment credentials.
- Marketplace settlement state is separate from canvas operation state.
- Failed settlement blocks new installation but does not corrupt existing boards.
- Revoked entitlement freezes future instantiation and leaves existing audit evidence intact.

## Benchmark displacement notes
- Miro Enterprise sets expectations for enterprise template libraries and admin controls.
- Mural Enterprise sets expectations for facilitation templates and workspace governance.
- FigJam sets expectations for community-style templates with fast install.
- Lucidspark sets expectations for diagrams, voting, and structured workshop artifacts.
- Whiteboard.fi sets expectations for education boards and classroom template speed.
- Microsoft Whiteboard sets expectations for suite-admin template availability.
- Oyatie displaces those expectations with DealSet evidence, tenant policy, and pack-aware install rules.
- Benchmark parity must include fast free templates and governed commercial packs.
- Vendor parity does not justify unmetered cross-tenant distribution.
- Tenant-specific licenses remain tenant-specific even when template content is copied into board state.

## Capability binding
- `template-marketplace-install` validates commercial entitlement before template materialization.
- `board-open` exposes installed template provenance in board metadata.
- `canvas-op-append` treats template-created objects as normal canvas operations after materialization.
- `presence-sync` never carries settlement metadata.
- `history-snapshot` records template provenance for after-the-fact license audit.
- `export-render` checks whether export styles, brand kits, or assets require settlement.
- Capability records under microservices/whiteboard/capabilities remain the naming authority.
- PRD-whiteboard remains the product authority for tenant-scoped and migration-ready collaboration.
- ARCHITECTURE.md remains the bounded-context authority for template and export aggregates.
- cost-budget.md remains the cost dimension reference for tenant, source vendor, and workflow template.

## Settlement state machine
- State `preflight_requested` records the user's intent and idempotency key.
- State `policy_checked` records the Cedar decision id.
- State `dealset_checked` records entitlement lookup and commercial basis.
- State `settlement_reserved` records the reservation id and expiry.
- State `template_materialized` records the immutable template version copied into tenant scope.
- State `audit_sealed` records the audit-chain event id.
- State `installed` marks the template available to the board.
- State `failed_policy` blocks the install without settlement.
- State `failed_settlement` blocks materialization and records retry rules.
- State `failed_materialization` releases the reservation if content was not made available.
- State `revoked` blocks new instantiations but preserves evidence.
- State `rolled_back` removes future availability and preserves historical operations.
- All transitions are idempotent by tenant, board, template, DealSet, and install key.
- No transition can skip audit_sealed for paid packs.
- No transition can mark installed without policy_checked and dealset_checked.

## Policy and authorization
- Cedar action `install_marketplace_template` requires tenant admin, template curator, workflow owner, or delegated facilitator authority.
- Cedar action `install_emergency_template` is routed through IP-013 emergency rules and still checks commercial entitlement.
- Cedar resource includes marketplace_pack_id, template_family, data_class, pack overlay, and jurisdiction.
- Cedar context includes source vendor, DealSet id, settlement mode, and requested board id.
- Deny if template family is not allowed by tenant policy.
- Deny if pack overlay forbids the source region or asset type.
- Deny if principal lacks template install authority.
- Deny if DealSet entitlement cannot be tied to the same tenant.
- Deny if paid template attempts free-template path.
- Deny if a classroom/education template is installed into a tenant pack that forbids education data handling.
- Allow free first-party templates only when catalog provenance and policy pass.
- Emit refusal evidence for every deny.
- Audit includes policy decision id before settlement reservation.
- Policy fragments are future work under microservices/whiteboard/policies.
- ADR-0243 and ADR-0244 remain binding for policy and ontology.

## Data requirements
- `marketplace_install_id` is immutable.
- `marketplace_pack_id` identifies the commercial or free pack.
- `dealset_id` is required for paid packs.
- `settlement_basis` records free, prepaid, metered, enterprise-license, trial, refund, or emergency-entitlement.
- `template_version` identifies the copied source.
- `source_vendor_label` is evidence, not canonical domain identity.
- `tenant_license_scope` records tenant, workspace, board, classroom, or incident scope.
- `install_actor` records principal id and authority basis.
- `board_materialization_hash` proves what was copied.
- `rollback_bundle_id` points to removal and evidence instructions.
- `audit_event_id` seals the transition.
- `cost_tags` include tenant, capability, source vendor, workflow template, and board id.
- `pack_overlay_result` records higher-restriction-wins decisions.
- `export_license_terms` record whether exports may include premium assets.
- `revocation_policy` records freeze, remove, or retain-with-watermark behavior.

## Implementation plan
- Step 1: Define marketplace install DTOs in future OpenAPI and internal proto contract slices.
- Step 2: Add Cedar policy for install authority and template-family constraints.
- Step 3: Add DealSet preflight adapter behind an application-layer port.
- Step 4: Add idempotent settlement reservation before template materialization.
- Step 5: Add materialization logic that copies immutable template content into tenant board scope.
- Step 6: Add audit-chain event after materialization and before user-visible install completion.
- Step 7: Add rollback bundle generation for failed or revoked installs.
- Step 8: Add history-snapshot provenance fields for installed template content.
- Step 9: Add export-render license checks for premium assets.
- Step 10: Add cost-budget emission for settlement reservations and metered usage.
- Step 11: Add DPIA coverage for education and public-sector templates.
- Step 12: Add compliance pack overlays for retention and audit export.
- Step 13: Add scorecard rows for free template, paid template, failed settlement, and revocation.
- Step 14: Add benchmark parity rows for the six displaced vendors.
- Step 15: Add runbook steps for settlement outage and entitlement revocation.
- Step 16: Add dashboard panels for install latency, settlement failures, and revocations.
- Step 17: Add replay worker rules for settlement event rehydration.
- Step 18: Add capacity admission treatment so settlement outages degrade install only, not board editing.
- Step 19: Add threat-model cases for entitlement replay and cross-tenant template leakage.
- Step 20: Add auditor export shape for DealSet proof.

## Operational controls
- Settlement adapter has a circuit breaker that blocks paid installs when degraded.
- Free first-party installs continue only if catalog and policy are healthy.
- Paid template content is not materialized before settlement reservation.
- Reservation expiry releases unmaterialized commercial claims.
- Metered templates emit usage after materialization and export.
- Refund or revocation does not delete board history.
- Tenant admins can disable marketplace templates by family.
- Finance can reconcile by DealSet id, tenant id, and marketplace_install_id.
- Auditors can trace install to policy decision, settlement reservation, materialization hash, and audit event.
- SRE alerts on settlement failure rate and reservation latency.
- Marketplace operators receive no raw board content.
- Export paths check license terms before rendering premium assets.
- Education templates require education pack compatibility.
- Public-sector templates require jurisdiction and regulator export metadata.
- Emergency templates inherit IP-013 expiry and non-bypassable controls.

## Failure modes
- DealSet service unavailable: block paid installs, allow existing installed templates, emit degraded evidence.
- Catalog unavailable: block new installs, allow board-open and canvas operations.
- Policy unavailable: deny installs and emit refusal evidence.
- Materialization hash mismatch: abort install, release reservation, quarantine content.
- Audit-chain outage: do not mark paid install complete.
- Duplicate install submission: return prior state through idempotency key.
- Revoked entitlement: freeze new instantiation, preserve existing history, alert tenant admin.
- Cross-tenant pack id: deny and emit policy/refusal evidence.
- Export asset license conflict: render without premium asset only if license terms allow fallback.
- Cost tag emission failure: mark installed only if audit is sealed and cost event is queued.
- Marketplace timeout after reservation: retry idempotently until reservation expiry.
- Refund after use: preserve audit evidence and defer commercial handling to settlement owner.
- Pack overlay conflict: higher restriction wins.
- Replay drift: quarantine settlement replay and require manual review.
- Source vendor rename: keep immutable marketplace_pack_id and treat label as display evidence.

## Evidence and tests
- Evidence 1: Free first-party template install passes policy and no DealSet reservation.
- Evidence 2: Paid template install requires DealSet id and reservation.
- Evidence 3: Paid template cannot reach installed without materialization hash and audit event.
- Evidence 4: Duplicate install returns same marketplace_install_id.
- Evidence 5: Failed settlement leaves board unchanged.
- Evidence 6: Revocation freezes future instantiation and preserves history.
- Evidence 7: Export-render refuses premium assets without export license.
- Evidence 8: History-snapshot includes template provenance.
- Evidence 9: Cost-budget events include tenant, capability, source vendor, workflow template, and board id.
- Evidence 10: DPIA covers education and public-sector templates.
- Evidence 11: Benchmark parity maps template-library expectations for all six displaced vendors.
- Evidence 12: ADR-0321 matrix retains principal, Cedar, tenant, audit, pack, rollback, and benchmark anchors.
- Evidence 13: Negative tests prove paid templates cannot use free path.
- Evidence 14: Negative tests prove cross-tenant entitlement cannot install.
- Evidence 15: Runbook drill covers DealSet outage and entitlement revocation.

## Marketplace-specific domain and contract deltas
- Domain aggregate: `marketplace_template_install` is separate from `template_document` to keep commerce state out of board content.
- Domain invariant: `marketplace_template_install.dealset_id` is required when `settlement_basis` is paid, prepaid, metered, or enterprise-license.
- Domain invariant: `template_document.materialization_hash` must match the catalog version admitted by settlement.
- Domain invariant: revoked entitlement blocks future materialization but does not rewrite existing canvas operations.
- Domain invariant: a free first-party template cannot later be reclassified as paid for existing boards.
- Domain event `whiteboard.marketplace.install.preflighted` records policy and catalog checks.
- Domain event `whiteboard.marketplace.dealset.reserved` records settlement reservation id.
- Domain event `whiteboard.marketplace.template.materialized` records content hash and board id.
- Domain event `whiteboard.marketplace.entitlement.revoked` records freeze policy.
- OpenAPI delta: install request carries `marketplace_pack_id`, `settlement_basis`, `dealset_id`, and `template_family`.
- OpenAPI delta: install response carries `marketplace_install_id`, `materialization_hash`, `license_scope`, and `rollback_bundle_id`.
- AsyncAPI delta: emit `whiteboard.marketplace.install.completed.v1` only after audit seal.
- AsyncAPI delta: emit `whiteboard.marketplace.install.blocked.v1` for policy, settlement, catalog, capacity, or budget denial.
- Proto delta: internal command `ReserveTemplateSettlement` is called before `MaterializeTemplate`.
- Proto delta: internal event `TemplateMaterialized` carries `dealset_evidence_id` and `pack_overlay_result`.
- Cedar fact: `context.settlement_basis == "free"` requires first-party catalog provenance.
- Cedar fact: `context.dealset_tenant_id` must equal `principal.tenant_id`.
- Cedar fact: `resource.template_family` must be allowed by tenant template policy.
- Cedar fact: `resource.asset_region` must satisfy active pack overlay.
- Workflow decision: paid install uses preflight, reservation, materialization, audit seal, and completion.
- Workflow decision: failed reservation releases no content to the board.
- Workflow decision: revoked entitlement opens tenant-admin review rather than mutating old history.
- Workflow decision: disputed charge links finance review to immutable materialization hash.
- SLO: free first-party template install p95 target is 400 ms after catalog warmup.
- SLO: paid template install p95 target excludes external DealSet latency but records it separately.
- SLO: settlement reservation idempotency conflict resolution target is 1 second.
- SLO: revocation freeze propagation target is 60 seconds.
- Replay case: settlement event replay rehydrates DealSet evidence before template history.
- Replay case: catalog version replay refuses to change materialization hash on existing boards.
- Replay case: export replay validates premium asset license before rendering.
- Rollback: failed materialization releases settlement reservation when no content was exposed.
- Rollback: mistaken paid install disables future instantiation and produces audit packet.
- Rollback: bad catalog rollout restores prior allowed template families.
- Test case: paid template without DealSet id is denied before materialization.
- Test case: cross-tenant DealSet id is denied with refusal evidence.
- Test case: revoked entitlement blocks new boards but preserves existing history snapshots.
- Test case: export with premium asset fails when export license term forbids it.
- Test case: duplicate paid install returns prior marketplace_install_id.
- Evidence field: `settlement_reservation_id` links commerce state to install.
- Evidence field: `materialization_hash` proves copied template payload.
- Evidence field: `license_scope` records tenant, board, classroom, incident, or enterprise scope.

## Acceptance criteria
- AC-001: Marketplace install has a concrete state machine.
- AC-002: DealSet settlement per ADR-0314 remains mandatory for paid packs.
- AC-003: ADR-0321 remains listed and unmodified.
- AC-004: All six benchmark names are present exactly.
- AC-005: Template install, history snapshot, export render, cost budget, DPIA, and compliance are tied to repo-local references.
- AC-006: Failure modes distinguish policy, catalog, settlement, materialization, audit, and revocation.
- AC-007: Rollback preserves evidence and avoids destructive history deletion.
- AC-008: Vendor labels are evidence only, not canonical object identity.
- AC-009: The plan is implementation-ready without editing adjacent files.
- AC-010: Settlement evidence records template license state, DealSet decision id, tenant entitlement, and rollback token before any marketplace template appears on a board.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
