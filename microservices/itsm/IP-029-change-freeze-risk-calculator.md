# IP-029 ITSM change-freeze-risk-calculator

Service: itsm
ChangeSet scope: microservices/itsm/IP-029-change-freeze-risk-calculator.md
Benchmarks displaced: ServiceNow ITSM, Jira Service Management, BMC Helix ITSM, Ivanti Neurons, Freshservice
Binding ADRs: ADR-0105, ADR-0244, ADR-0246, ADR-0258, ADR-0263, ADR-0316, ADR-0321

## Objective
- Objective 001: Build the ITSM change freeze and risk calculator for standard, normal, emergency, and rollback-prone changes.
- Objective 002: Displace ServiceNow change risk, Jira change management, BMC Helix change calendar, Ivanti change automation, and Freshservice change workflows.
- Objective 003: Produce deterministic risk tier, freeze decision, approval path, rollback requirement, and audit evidence for every change request.
- Objective 004: Prevent vendor-specific change status, requester role, or support group from bypassing Oyatie tenant-scoped policy.
- Objective 005: Make the calculator buildable from this IP under docs/standards/documentation-rigor.md section 1.1.
- Objective 006: Keep incident-management paging, ERP approvals, journeys, manifests, ADR-0321 edits, and other B2B leader batches out of scope.

## Inputs
- Input 001: tenant_id is mandatory and comes from the tenant context kernel.
- Input 002: principal_id is mandatory and comes from the authenticated actor or migration worker identity.
- Input 003: source_system_kind distinguishes ServiceNow ITSM, Jira Service Management, BMC Helix ITSM, Ivanti Neurons, Freshservice, and native Oyatie.
- Input 004: change_type is standard, normal, emergency, or imported_historical.
- Input 005: affected_ci_refs lists canonical CMDB objects and must be validated through the CMDB reconciliation graph.
- Input 006: impacted_service_refs lists service components and customer-facing service refs.
- Input 007: requested_window contains start, end, timezone, recurrence, and local holiday metadata.
- Input 008: freeze_window_refs lists tenant, pack, and global freeze windows.
- Input 009: implementation_plan_ref points to workflow template and executable steps.
- Input 010: rollback_plan_ref is mandatory for normal and emergency changes and optional only for low-risk standard changes.
- Input 011: requester_ref records who requested the change but does not grant approval.
- Input 012: approver_refs record eligible approvers after Cedar and separation-of-duty checks.
- Input 013: risk_answers record operator responses for blast radius, reversibility, customer impact, security impact, and data impact.
- Input 014: compliance_pack_set records GDPR, KR-PIPA, SOC-2, ISO-27001, ITIL, FedRAMP-High, or tenant-specific overlays.
- Input 015: historical_failure_rate records local change failure evidence.
- Input 016: major_incident_correlation records whether affected CIs are linked to recent incidents or problems.

## Outputs
- Output 001: risk_tier is low, medium, high, critical, or emergency.
- Output 002: freeze_decision is allowed, blocked_by_freeze, requires_breakglass, requires_reschedule, or requires_approval_escalation.
- Output 003: approval_path lists approver roles, quorum, separation-of-duty constraints, and pack-required approvers.
- Output 004: rollback_requirement states none_allowed, optional, mandatory, mandatory_with_test, or mandatory_with_preapproved_fallback.
- Output 005: implementation_gate states ready, waiting_for_approval, waiting_for_window, blocked_by_policy, blocked_by_cmdb, or blocked_by_missing_rollback.
- Output 006: policy_decision_id records Cedar decision evidence.
- Output 007: audit_event_id records calculator result evidence.
- Output 008: workflow_template_id records the selected standard, normal, emergency, or post-change review template.
- Output 009: remediation_hint_slug maps blocked outcomes to runbooks.
- Output 010: explanation_lines provide deterministic human-readable reasons without leaking sensitive tenant data.

## Vendor displacement behavior
- Vendor behavior 001: ServiceNow change risk fields map to risk answers and do not become canonical risk tiers directly.
- Vendor behavior 002: ServiceNow blackout schedules map to freeze_window_refs and require tenant plus pack validation.
- Vendor behavior 003: ServiceNow emergency change status maps to change_type only after emergency evidence exists.
- Vendor behavior 004: Jira change request fields map to change_type and affected service refs after request-type validation.
- Vendor behavior 005: Jira project role approvers map to candidate approvers and still require Cedar approval entitlement.
- Vendor behavior 006: BMC Helix change calendar maps to freeze windows and implementation windows.
- Vendor behavior 007: BMC support group routing maps to candidate approvers and operators, not authority.
- Vendor behavior 008: Ivanti automation recommendation maps to suggested implementation plan and not automatic execution.
- Vendor behavior 009: Ivanti risk scoring fields map to input evidence and are recomputed canonically.
- Vendor behavior 010: Freshservice change workflow status maps to process stage and cannot skip approval.
- Vendor behavior 011: Freshservice CAB membership maps to approver candidates and still requires separation-of-duty checks.
- Vendor behavior 012: Unknown vendor risk fields are retained as unmapped evidence and trigger review when they affect scoring.

## Risk scoring rules
- Risk rule 001: Critical data_class impact raises minimum risk tier to high.
- Risk rule 002: Security control impact raises minimum risk tier to high.
- Risk rule 003: Multi-cell blast radius raises minimum risk tier to high.
- Risk rule 004: Single tenant but customer-visible outage raises minimum risk tier to medium.
- Risk rule 005: No tested rollback plan raises minimum risk tier to high for normal changes.
- Risk rule 006: Missing rollback plan blocks high and critical changes.
- Risk rule 007: Recent related major incident raises minimum risk tier by one level.
- Risk rule 008: Recent failed change on affected CI raises minimum risk tier by one level.
- Risk rule 009: Unverified CMDB relation on affected CI blocks automatic approval.
- Risk rule 010: Standard change can remain low risk only if template, affected CI class, rollback, and historical failure thresholds pass.
- Risk rule 011: Emergency change can bypass freeze only with breakglass reason, approver, expiry, and post-change review.
- Risk rule 012: Compliance pack can raise approval quorum but cannot lower base risk.

## Freeze decision rules
- Freeze rule 001: Tenant freeze window blocks normal and standard changes unless explicit exception exists.
- Freeze rule 002: Global freeze window blocks all non-emergency changes.
- Freeze rule 003: Compliance pack freeze blocks changes affecting protected data class.
- Freeze rule 004: Holiday freeze window blocks customer-visible changes unless emergency evidence exists.
- Freeze rule 005: Major incident freeze blocks changes affecting related CIs unless incident commander approves.
- Freeze rule 006: Security emergency can require change execution inside freeze but must record breakglass evidence.
- Freeze rule 007: Freeze exception requires approver not equal to requester.
- Freeze rule 008: Freeze exception requires rollback plan and post-change review.
- Freeze rule 009: Freeze exception emits dedicated audit event class.
- Freeze rule 010: Freeze denial returns remediation hint to reschedule, request breakglass, or reduce scope.

## Implementation sequence
- Implementation 001: Add calculator input and output structs in ITSM domain or usecase layer per ADR-0105.
- Implementation 002: Add vendor mapping helper for imported change risk evidence.
- Implementation 003: Add risk scoring engine with explicit minimum-tier and tier-raise rules.
- Implementation 004: Add freeze evaluator with tenant, global, pack, holiday, major-incident, and security emergency windows.
- Implementation 005: Add approval-path builder with separation-of-duty checks.
- Implementation 006: Add rollback requirement builder.
- Implementation 007: Add workflow template selector for standard, normal, emergency, and post-change review paths.
- Implementation 008: Add explanation-line builder with deterministic reason order.
- Implementation 009: Add audit event emission for calculated, blocked, breakglass_required, approved_path_selected, and rolled_back.
- Implementation 010: Add OpenAPI examples for change open, approve, implement, and freeze denial.
- Implementation 011: Add AsyncAPI messages for risk_calculated, freeze_blocked, breakglass_required, and approval_path_selected.
- Implementation 012: Add dashboard panels for change risk tier mix, freeze denials, exception approvals, and rollback missing rate.
- Implementation 013: Add runbook links for change freeze override, local change failure spike, and rollback failure.
- Implementation 014: Add SDK builder method that runs calculation preview before submitting change approval.
- Implementation 015: Add CLI fixture command for benchmark vendor change examples.

## Test matrix
- Test 001: Unit test ServiceNow blackout schedule maps to freeze window ref.
- Test 002: Unit test ServiceNow risk field is recomputed and not trusted directly.
- Test 003: Unit test Jira project role approver still requires Cedar approval.
- Test 004: Unit test BMC change calendar maps to implementation window.
- Test 005: Unit test Ivanti recommendation cannot execute automatically.
- Test 006: Unit test Freshservice CAB membership still requires separation-of-duty.
- Test 007: Unit test no rollback plan blocks high-risk normal change.
- Test 008: Unit test recent major incident raises risk.
- Test 009: Unit test protected data class raises risk under pack overlay.
- Test 010: Unit test tenant freeze blocks normal change.
- Test 011: Unit test global freeze blocks standard change.
- Test 012: Unit test emergency change requires breakglass reason and expiry.
- Test 013: Unit test freeze exception requires approver not equal to requester.
- Test 014: Unit test affected CI with unverified relation blocks automatic approval.
- Test 015: Property test proves risk tier never decreases when adding impact dimensions.
- Test 016: Property test proves compliance pack can only maintain or raise risk, never lower it.
- Test 017: Replay test proves identical imported change evidence produces identical risk output.
- Test 018: Rollback test verifies rollback plan ref remains available after change denial.
- Test 019: Contract test verifies freeze denial response includes remediation hint.
- Test 020: Audit test verifies calculated, denied, and breakglass-required events.

## Failure handling
- Failure 001: Missing tenant context returns validation error before risk scoring.
- Failure 002: Missing affected CI refs blocks normal and emergency change calculation.
- Failure 003: Missing rollback plan blocks high-risk and critical changes.
- Failure 004: Missing approver candidate blocks approval path selection.
- Failure 005: Unknown vendor risk field triggers review when mapped field is required.
- Failure 006: Freeze calendar load failure blocks scheduling and returns remediation hint.
- Failure 007: CMDB relation validation failure blocks automatic approval.
- Failure 008: Policy denial records policy_decision_id and audit evidence.
- Failure 009: Audit-start failure blocks calculator output publication.
- Failure 010: Replay drift blocks imported change replay and emits drift evidence.

## Acceptance criteria
- Acceptance 001: An intern can implement calculator input, output, vendor mapping, risk scoring, freeze evaluation, approval path, rollback requirement, and explanation lines.
- Acceptance 002: An intern can explain how ServiceNow, Jira, BMC, Ivanti, and Freshservice change features are displaced.
- Acceptance 003: An intern can implement minimum-tier, tier-raise, and pack-overlay non-lowering rules.
- Acceptance 004: An intern can implement freeze denial and exception semantics without private notes.
- Acceptance 005: An intern can implement benchmark fixtures, property tests, replay tests, rollback tests, contract tests, and audit tests.
- Acceptance 006: An intern can wire the calculator to REST, AsyncAPI, SDK, CLI, dashboards, and runbooks.
- Acceptance 007: An intern can prove vendor risk scores are evidence and never authority.
- Acceptance 008: An intern can prove emergency changes require breakglass evidence and post-change review.
- Acceptance 009: An intern can avoid touching manifests, journeys, ERP, ADR-0321, and other B2B leader services.
- Acceptance 010: An intern can produce a PR that keeps change execution tenant-scoped, reversible, and audit-sealed.

## Citations and authority trail
- Citation 001: docs/standards/documentation-rigor.md section 1.1 defines the intern-buildability bar.
- Citation 002: microservices/itsm/manifest.json defines ITSM audience, benchmark roster, compliance packs, and layer conformance.
- Citation 003: microservices/itsm/PRD.md defines change as an ITSM bounded context.
- Citation 004: microservices/itsm/runbooks/change-freeze-override.md anchors freeze remediation.
- Citation 005: microservices/itsm/runbooks/local-change-failure-rate-spike.md anchors failure-rate remediation.
- Citation 006: microservices/itsm/slos/local-change-failure-rate.openslo.yaml anchors change quality targets.
- Citation 007: ADR-0105 defines layer boundaries for calculator domain, usecase, REST, worker, adapter, and governance code.
- Citation 008: ADR-0244 defines default-deny policy expectations for approval and exception paths.
- Citation 009: ADR-0246 defines reusable library expectations for risk and freeze evaluation.
- Citation 010: ADR-0258 defines contract versioning for calculator responses.
- Citation 011: ADR-0263 defines audit-chain event discipline for risk, denial, breakglass, and rollback.
- Citation 012: ADR-0316 prevents vendor change management labels from becoming service boundaries.
- Citation 013: ADR-0321 defines B2B leader parity expectations for ITSM change depth.

## Detailed build checklist
- Build checklist 001: Add fixture `servicenow_change_normal_valid.json`.
- Build checklist 002: Add fixture `servicenow_blackout_schedule_blocked.json`.
- Build checklist 003: Add fixture `servicenow_emergency_without_breakglass_denied.json`.
- Build checklist 004: Add fixture `jira_change_request_valid.json`.
- Build checklist 005: Add fixture `jira_project_admin_not_approver_denied.json`.
- Build checklist 006: Add fixture `bmc_change_calendar_valid.json`.
- Build checklist 007: Add fixture `bmc_support_group_not_authority_denied.json`.
- Build checklist 008: Add fixture `ivanti_recommendation_preview_only.json`.
- Build checklist 009: Add fixture `ivanti_risk_score_recomputed.json`.
- Build checklist 010: Add fixture `freshservice_cab_separation_denied.json`.
- Build checklist 011: Add canonical output for low-risk standard change.
- Build checklist 012: Add canonical output for medium-risk customer-visible change.
- Build checklist 013: Add canonical output for high-risk protected-data change.
- Build checklist 014: Add canonical output for critical multi-cell change.
- Build checklist 015: Add canonical output for emergency breakglass change.
- Build checklist 016: Add canonical output for freeze-denied normal change.
- Build checklist 017: Add canonical output for rollback-missing blocked change.
- Build checklist 018: Add canonical output for approval-escalation-required change.
- Build checklist 019: Add OpenAPI example for change risk preview.
- Build checklist 020: Add OpenAPI example for change approval path selection.
- Build checklist 021: Add OpenAPI example for freeze denial.
- Build checklist 022: Add AsyncAPI fixture for `itsm.change.risk_calculated.v1`.
- Build checklist 023: Add AsyncAPI fixture for `itsm.change.freeze_blocked.v1`.
- Build checklist 024: Add AsyncAPI fixture for `itsm.change.breakglass_required.v1`.
- Build checklist 025: Add AsyncAPI fixture for `itsm.change.approval_path_selected.v1`.
- Build checklist 026: Add AsyncAPI fixture for `itsm.change.rollback_required.v1`.
- Build checklist 027: Add Grafana panel query for risk tier distribution.
- Build checklist 028: Add Grafana panel query for freeze denial rate.
- Build checklist 029: Add Grafana panel query for breakglass exception count.
- Build checklist 030: Add Grafana panel query for rollback missing rate.
- Build checklist 031: Add runbook pointer for change freeze override.
- Build checklist 032: Add runbook pointer for local change failure rate spike.
- Build checklist 033: Add runbook pointer for major incident backlog when change links to incident bridge.
- Build checklist 034: Add replay test for imported ServiceNow change history.
- Build checklist 035: Add rollback test for denied change with preserved evidence.
- Build checklist 036: Add final verification command for line count and citation density.
- Build checklist 037: Add PR summary line naming this as net-new IP-029.
- Build checklist 038: Add scope note that this IP does not edit manifests, journeys, ERP, ADR-0321, or other B2B leader services.

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/itsm/IP-029-change-freeze-risk-calculator.md` matched [`emission`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/itsm/IP-029-change-freeze-risk-calculator.md`, `microservices/itsm/manifest.json`, `microservices/itsm/capacity-model.md`, `microservices/itsm/compliance.md`, `microservices/itsm/ARCHITECTURE.md`].
