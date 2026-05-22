# IP-030 Whiteboard cross-board template marketplace review

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-030-cross-board-template-marketplace-review.md
Capability focus: template-marketplace-install, board-open, canvas-op-append, export-render
Benchmarks: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0253-amendment, ADR-0257, ADR-0263, ADR-0297, ADR-0314, ADR-0316, ADR-0321
Repo-local references: microservices/whiteboard/PRD.md; microservices/whiteboard/ARCHITECTURE.md; microservices/whiteboard/capabilities/template-marketplace-install.yaml; microservices/whiteboard/capabilities/board-open.yaml; microservices/whiteboard/capabilities/export-render.yaml; microservices/whiteboard/runbooks/template-import-rollback.md; microservices/whiteboard/runbooks/dealset-template-license-hold.md; microservices/whiteboard/policy/canvas-collaboration-authorization.cedar; microservices/whiteboard/competitor-parity-matrix.md; microservices/whiteboard/compliance.md

## Objective
- Define review controls for templates reused across boards.
- Bind marketplace template installs to DealSet settlement and tenant policy.
- Prevent cross-board template reuse from leaking private board content.
- Preserve template provenance during install, modification, export, and rollback.
- Keep template marketplace behavior inside whiteboard capability boundaries.
- Match Miro Enterprise template library depth with governed install controls.
- Match Mural Enterprise workshop template reuse with commercial evidence.
- Match FigJam community template expectations without public leakage.
- Match Lucidspark diagram template expectations with connector-safe provenance.
- Match Whiteboard.fi classroom template reuse with education privacy.
- Match Microsoft Whiteboard enterprise template sharing with tenant identity controls.

## Current repo anchors
- anchor 001: PRD-whiteboard lists template as a bounded object family.
- anchor 002: PRD-whiteboard excludes bypassing marketplace DealSet settlement.
- anchor 003: ARCHITECTURE.md names template as a bounded context.
- anchor 004: template-marketplace-install capability binds marketplaceSettlement to DealSet.
- anchor 005: board-open capability controls installation target eligibility.
- anchor 006: export-render capability controls template package export.
- anchor 007: template-import-rollback runbook covers failed imports and rollback.
- anchor 008: dealset-template-license-hold runbook covers unresolved template licensing.
- anchor 009: canvas-collaboration-authorization.cedar is the default policy hook.
- anchor 010: competitor-parity-matrix.md names template marketplace install as a whiteboard primitive.
- anchor 011: ADR-0321 authorizes B2B SaaS leader coverage for whiteboard.
- anchor 012: ADR-0314 requires marketplace and DealSet settlement discipline.

## Domain vocabulary
- vocabulary 001: `template_id` identifies a reusable whiteboard template.
- vocabulary 002: `template_version_id` identifies a reviewed template revision.
- vocabulary 003: `template_origin_ref` identifies source board, vendor, or marketplace package.
- vocabulary 004: `template_install_id` identifies a specific installation into a board.
- vocabulary 005: `review_case_id` identifies approval workflow for marketplace publication or tenant install.
- vocabulary 006: `license_hold_id` identifies DealSet settlement or entitlement hold.
- vocabulary 007: `source_board_scope` records visibility and data-class constraints copied from source.
- vocabulary 008: `target_board_scope` records destination tenant, board, pack, and audience.
- vocabulary 009: `template_sanitization_report` records removed content and reasons.
- vocabulary 010: `template_policy_profile` records allowed install and export contexts.
- vocabulary 011: `template_dependency_ref` records fonts, assets, widgets, and connector dependencies.
- vocabulary 012: `template_review_epoch` increments on approval, rejection, revocation, or policy change.

## Review classes
- review 001: `tenant-private` templates stay inside a single tenant.
- review 002: `org-shared` templates are reusable across tenant departments.
- review 003: `marketplace-paid` templates require DealSet settlement.
- review 004: `marketplace-free` templates require publisher provenance and abuse review.
- review 005: `education-classroom` templates require student privacy defaults.
- review 006: `migration-imported` templates preserve source vendor ids and transform evidence.
- review 007: `regulated-pack` templates require data-class and retention review.
- review 008: `external-client` templates require client boundary and export review.
- review 009: `diagram-structured` templates require connector and object-schema validation.
- review 010: `facilitation-workshop` templates require timer and vote governance defaults.
- review 011: `ai-assisted` templates require generated-content provenance when present.
- review 012: `revoked` templates cannot be installed but existing boards keep audit evidence.

## Marketplace workflow
- workflow 001: Publisher submits template package with template_origin_ref.
- workflow 002: API validates package manifest and declared review class.
- workflow 003: Usecase evaluates publisher authority and tenant scope.
- workflow 004: Sanitizer scans source board for private objects, comments, votes, and student data.
- workflow 005: Sanitizer emits template_sanitization_report.
- workflow 006: Dependency scanner records fonts, images, widgets, and connector refs.
- workflow 007: Policy evaluator checks template_policy_profile.
- workflow 008: DealSet evaluator checks commercial settlement requirements.
- workflow 009: Human or automated review creates review_case_id.
- workflow 010: Approved template_version_id becomes installable.
- workflow 011: Rejected template records reasons and remediation path.
- workflow 012: Revoked template blocks new installs and emits tenant notifications.

## Install workflow
- install 001: Tenant admin or board owner requests template install.
- install 002: Request includes target_board_scope and purpose.
- install 003: Policy checks principal authority for the target board.
- install 004: DealSet checks entitlement for paid templates.
- install 005: Education templates apply teacher-visible and peer-hidden defaults.
- install 006: Facilitated workshop templates apply timer and voting governance defaults.
- install 007: Diagram templates validate connector endpoints before object creation.
- install 008: Install creates template_install_id and object mapping.
- install 009: Installed objects carry template_origin_ref and template_version_id.
- install 010: Install emits canvas-op-append operations, not direct storage writes.
- install 011: Install creates history snapshot checkpoint when configured.
- install 012: Install evidence links to review_case_id and license_hold_id when present.

## Cross-board privacy requirements
- privacy 001: Source board private content is excluded unless review policy explicitly permits.
- privacy 002: Comments are removed by default from templates.
- privacy 003: Vote results are removed unless facilitation-summary template mode permits aggregate data.
- privacy 004: Timer state is converted to default configuration, not active countdown.
- privacy 005: Student board content is excluded from marketplace templates.
- privacy 006: Teacher-authored classroom scaffolds may be included when sanitized.
- privacy 007: User names are removed or replaced with role placeholders.
- privacy 008: External images are copied only when license and privacy policy permit.
- privacy 009: Connector endpoints to private objects are removed or repaired.
- privacy 010: Embedded files use drive policy and are referenced, not copied, unless allowed.
- privacy 011: Template preview never exposes source board ids to unapproved viewers.
- privacy 012: Template export includes redaction and sanitization reports.

## Benchmark displacement map
- benchmark 001: Miro Enterprise displaced behavior is enterprise template library and team templates.
- benchmark 002: Miro Enterprise gap is closed by tenant-private and org-shared review classes.
- benchmark 003: Mural Enterprise displaced behavior is workshop template publication and reuse.
- benchmark 004: Mural Enterprise gap is closed by facilitation-workshop defaults and governance review.
- benchmark 005: FigJam displaced behavior is community template duplication.
- benchmark 006: FigJam gap is closed by marketplace-free review with source sanitization.
- benchmark 007: Lucidspark displaced behavior is diagram and flowchart templates.
- benchmark 008: Lucidspark gap is closed by diagram-structured validation.
- benchmark 009: Whiteboard.fi displaced behavior is classroom activity reuse.
- benchmark 010: Whiteboard.fi gap is closed by education-classroom review and privacy defaults.
- benchmark 011: Microsoft Whiteboard displaced behavior is enterprise sharing through Microsoft identity.
- benchmark 012: Microsoft Whiteboard gap is closed by tenant identity, target board scope, and audit evidence.

## Policy hooks
- policy 001: Publish requires template publisher authority.
- policy 002: Tenant-private install requires same tenant and board owner authority.
- policy 003: Org-shared install requires department or workspace scope compatibility.
- policy 004: Marketplace-paid install requires DealSet settlement success.
- policy 005: Marketplace-free install requires publisher and abuse review approval.
- policy 006: Education-classroom install requires teacher or curriculum owner authority.
- policy 007: External-client template install requires client boundary approval.
- policy 008: Template export requires export-render egress policy.
- policy 009: Revoked template cannot be installed.
- policy 010: Existing revoked-template instances remain readable with warning and audit evidence.
- policy 011: Sanitization report must be accepted before publication.
- policy 012: Cross-region install must obey target board residency and pack overlays.

## Data and event model
- event 001: `whiteboard.template.submitted` records publisher and template_origin_ref.
- event 002: `whiteboard.template.sanitized` records sanitization report digest.
- event 003: `whiteboard.template.review_opened` records review_case_id.
- event 004: `whiteboard.template.approved` records template_version_id.
- event 005: `whiteboard.template.rejected` records rejection reasons.
- event 006: `whiteboard.template.revoked` records revocation reason and epoch.
- event 007: `whiteboard.template.license_hold_created` records license_hold_id.
- event 008: `whiteboard.template.license_hold_released` records DealSet evidence.
- event 009: `whiteboard.template.install_requested` records target_board_scope.
- event 010: `whiteboard.template.installed` records object mapping and install id.
- event 011: `whiteboard.template.install_rejected` records policy or license reason.
- event 012: `whiteboard.template.export_attested` records export provenance.

## Compliance and commercial evidence
- compliance 001: SOC-2 evidence includes review_case_id and approver.
- compliance 002: ISO-27001 evidence includes dependency and asset inventory.
- compliance 003: GDPR evidence includes personal data sanitization report.
- compliance 004: KR-PIPA evidence includes privacy removal and retention labels.
- compliance 005: Education pack evidence includes student-content exclusion.
- compliance 006: Public-sector pack evidence includes data residency and procurement label.
- compliance 007: DealSet evidence includes entitlement, price, publisher, and revocation terms.
- compliance 008: Export evidence includes artifact digest and redaction manifest.
- compliance 009: Audit evidence includes source board scope and target board scope.
- compliance 010: Marketplace abuse review includes template preview and dependency scan.
- compliance 011: Revocation evidence includes affected tenants and notification state.
- compliance 012: Rollback evidence includes template-import-rollback runbook outcome.

## SLO and telemetry
- telemetry 001: Measure template submission validation latency.
- telemetry 002: Measure sanitization duration by object count bucket.
- telemetry 003: Measure review queue age.
- telemetry 004: Measure DealSet license hold duration.
- telemetry 005: Measure install latency by template size.
- telemetry 006: Measure install rejection rate by policy reason.
- telemetry 007: Measure revoked template install attempts.
- telemetry 008: Measure marketplace abuse rejection count.
- telemetry 009: Measure education template privacy denial count.
- telemetry 010: Measure export attestation latency.
- telemetry 011: Trace template_install_id through publish, review, install, export, and rollback.
- telemetry 012: Keep raw tenant_id out of metrics while preserving signed audit evidence.

## Acceptance criteria
- acceptance 001: Every template publication has review_case_id.
- acceptance 002: Every template version has sanitization report evidence.
- acceptance 003: Every paid install has DealSet settlement or license hold.
- acceptance 004: Every installed object carries template_origin_ref.
- acceptance 005: Cross-board template install uses canvas-op-append operations.
- acceptance 006: Student content is excluded from marketplace templates.
- acceptance 007: Revoked templates block new installs.
- acceptance 008: Existing revoked-template instances keep audit evidence.
- acceptance 009: Template export includes redaction and marketplace manifests.
- acceptance 010: Benchmark evidence names all six required displaced products.
- acceptance 011: ADR-0321, ADR-0316, and ADR-0314 are included in the evidence packet.
- acceptance 012: License holds route to dealset-template-license-hold runbook.

## Test plan
- test 001: Unit-test template package manifest validation.
- test 002: Unit-test sanitization report generation.
- test 003: Unit-test template dependency scan.
- test 004: Unit-test review state transitions.
- test 005: Unit-test revoked template install denial.
- test 006: Unit-test installed object template_origin_ref.
- test 007: Cedar-fixture-test publisher authority denial.
- test 008: Cedar-fixture-test paid template install without DealSet denial.
- test 009: Cedar-fixture-test education classroom template privacy denial.
- test 010: Cedar-fixture-test external-client boundary denial.
- test 011: Contract-test template-marketplace-install request shape.
- test 012: AsyncAPI-test template publication and install events.
- test 013: Migration-fixture-test Miro Enterprise template import.
- test 014: Migration-fixture-test Mural Enterprise workshop template import.
- test 015: Migration-fixture-test FigJam community template import.
- test 016: Migration-fixture-test Lucidspark diagram template import.
- test 017: Migration-fixture-test Whiteboard.fi classroom template import.
- test 018: Migration-fixture-test Microsoft Whiteboard enterprise template import.

## Rollback and recovery
- rollback 001: Put template_version_id into license hold without deleting existing board objects.
- rollback 002: Block new installs for revoked or quarantined templates.
- rollback 003: Use template-import-rollback for failed install object mapping.
- rollback 004: Use dealset-template-license-hold for commercial settlement failures.
- rollback 005: Preserve source template package and sanitization evidence.
- rollback 006: Notify affected tenant admins when template revocation impacts existing boards.
- rollback 007: Remove template marketplace listing while preserving audit records.
- rollback 008: Re-run sanitization after policy revision before republishing.
- rollback 009: Prevent export of quarantined templates.
- rollback 010: Never delete source board history to repair template publication mistakes.

## Command and proto deltas
- proto 001: Add `TemplateSubmitRequest.template_id`, `template_origin_ref`, `review_class`, and `publisher_principal_id`.
- proto 002: Add `TemplatePackageManifest.source_board_scope`, `declared_data_classes`, and `dependency_refs`.
- proto 003: Add `TemplateSanitizationReport.removed_object_count`, `removed_comment_count`, `removed_identity_count`, and `student_content_removed`.
- proto 004: Add `TemplateReviewDecision.review_case_id`, `decision`, `reviewer_ref`, and `template_review_epoch`.
- proto 005: Add `TemplateLicenseHold.license_hold_id`, `dealset_ref`, `hold_reason`, and `release_condition`.
- proto 006: Add `TemplateInstallRequest.template_version_id`, `target_board_scope`, and `install_purpose`.
- proto 007: Add `TemplateInstallResult.template_install_id` and `installed_object_mapping`.
- proto 008: Add `InstalledTemplateObject.template_origin_ref` and `template_version_id`.
- proto 009: Add `TemplateRevocation.revocation_reason`, `effective_at_epoch`, and `affected_install_count`.
- proto 010: Add `TemplateExportAttestation.redaction_manifest_id` and `marketplace_manifest_id`.
- proto 011: Add `TemplateDependencyRef.dependency_type` for font, image, widget, connector, and embedded file.
- proto 012: Add `TemplatePolicyProfile.allowed_install_contexts` and `allowed_export_contexts`.

## Cedar facts
- cedar-fact 001: `principal_can_publish_template` gates template submission.
- cedar-fact 002: `review_class` selects the required review policy.
- cedar-fact 003: `sanitization_report_accepted` must be true before marketplace approval.
- cedar-fact 004: `dealset_state=settled` gates marketplace-paid install.
- cedar-fact 005: `license_hold_active` blocks install and export.
- cedar-fact 006: `target_board_scope_compatible` gates cross-board install.
- cedar-fact 007: `education_classroom_template=true` requires student-content removal.
- cedar-fact 008: `external_client_template=true` requires client boundary approval.
- cedar-fact 009: `template_revoked=true` blocks new install.
- cedar-fact 010: `dependency_scan_passed=true` is required before template publication.

## Workflow decisions
- workflow 001: Submission, sanitization, review, publication, install, export, and revocation are separate states.
- workflow 002: Sanitization is mandatory before any human approval can publish a marketplace template.
- workflow 003: DealSet license hold pauses paid install without deleting review evidence.
- workflow 004: Cross-board install creates canvas operations so merge, audit, and rollback stay normal.
- workflow 005: Revocation blocks future installs but does not destructively remove installed board objects.
- workflow 006: Education templates default to peer-hidden privacy and teacher-visible scaffolds.
- workflow 007: Diagram templates validate connector dependencies before approval.
- workflow 008: Export of template package re-runs redaction against the latest policy profile.

## Failure and replay cases
- failure 001: Sanitizer crash resumes from template package manifest and does not publish partial results.
- failure 002: DealSet outage creates license_hold_id and queues tenant notification.
- failure 003: Template install retry must not duplicate installed objects because template_install_id is idempotent.
- failure 004: Revoked template install attempt is denied and linked to revocation evidence.
- failure 005: Miro Enterprise template import must remove private board comments before publication.
- failure 006: Mural Enterprise workshop template import must convert active timers into defaults.
- failure 007: FigJam community template import must remove user identities and source board ids.
- failure 008: Lucidspark diagram template import must validate connector endpoints and dependency refs.
- failure 009: Whiteboard.fi classroom template import must remove student content.
- failure 010: Microsoft Whiteboard enterprise template import must map sharing scope to Oyatie target_board_scope.

## Evidence fields
- evidence 001: `template_id` proves template identity.
- evidence 002: `template_version_id` proves approved revision.
- evidence 003: `template_origin_ref` proves source board or vendor provenance.
- evidence 004: `review_case_id` proves review workflow lineage.
- evidence 005: `template_sanitization_report` proves privacy cleanup.
- evidence 006: `license_hold_id` proves commercial hold state.
- evidence 007: `dealset_decision_id` proves settlement outcome.
- evidence 008: `template_install_id` proves destination install lineage.
- evidence 009: `installed_object_mapping` proves source-to-target object mapping.
- evidence 010: `template_review_epoch` proves approval, revocation, or policy-version ordering.

## Done definition
- done 001: IP defines cross-board template marketplace review.
- done 002: IP references whiteboard PRD, architecture, capabilities, policies, competitor matrix, compliance, and runbooks.
- done 003: IP names Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
- done 004: IP includes workflow, privacy, policy, events, compliance, telemetry, tests, and rollback substance.
- done 005: IP stays inside microservices/whiteboard and does not edit ADR-0321.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
