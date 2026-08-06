---
id: ADR-WPI-001
title: Clock In Geofence with Tolerance vs Strict vs Flexible Tier Matrix
status: Proposed
date: 2026-05-20
microservice: workplace-integration
related_oyatie_adrs:
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0708-platform-foundations-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
decision_owner: axis-workplace-integration
---

# ADR-WPI-001: Clock In Geofence with Tolerance vs Strict vs Flexible Tier Matrix

## Context

- Workplace-integration owns offer e-sign, roster binding, informed consent, clock geofence, office barriers, and workplace evidence.
- Journey j37 names clock-in-geofence as the slice for B2B clocking and attendance.
- Clock events affect payroll exports, wage disputes, compliance evidence, and employee privacy.
- The user-facing flow needs mobile geolocation, worksite metadata, attestation, dispute handling, and payroll row export.
- Named pressure WPI-P1: U.S. FLSA requires accurate hours-worked records and compensation for compensable work.
- Named pressure WPI-P2: EU Working Time Directive requires working-time, rest, and night-work controls.
- Named pressure WPI-P3: Korea Labor Standards Act sets working-condition and working-hours floors.
- Named pressure WPI-P4: strict geofence denial can create wage theft risk if an employee actually worked.
- Named pressure WPI-P5: flexible clocking can create payroll fraud and worksite safety risk.
- Named pressure WPI-P6: mobile GPS, Wi-Fi, and cell location accuracy vary by building, device, OS, and weather.
- Named pressure WPI-P7: biometric or continuous tracking would raise privacy risk beyond the needed attendance proof.
- Named pressure WPI-P8: employees need a dispute path when location proof is noisy.
- Named pressure WPI-P9: tenants need policy tiering for fixed site, hybrid, field work, and remote work.
- Named pressure WPI-P10: payroll exports must record both accepted time and evidence quality.
- Constraint WPI-C1: every clock action is tenant-scoped under ADR-0244.
- Constraint WPI-C2: Cedar gates clock, override, dispute, payroll export, and admin policy changes under ADR-0243.
- Constraint WPI-C3: audit-chain evidence for clock events follows ADR-0003.
- Constraint WPI-C4: location data class and retention follow ADR-0034.
- Constraint WPI-C5: pack-pinned storage follows ADR-0049.
- Constraint WPI-C6: clock mutations use idempotency keys per ADR-0149.
- Constraint WPI-C7: metrics, traces, and dashboards follow ADR-0263.
- Constraint WPI-C8: workplace-integration must not become payroll, identity, payments, or HRIS.
- Constraint WPI-C9: off-the-clock work risk must be visible, not hidden by strict location refusal.
- Constraint WPI-C10: policy must support employee notice, consent, and purpose limitation.
- Existing `IP-journey-j37-clock-in-geofence.md` requires OpenAPI, AsyncAPI, proto3, BNF, Cedar, metrics, rollback, and audit id.
- Existing policy `clock-attest.cedar` anchors the action namespace.
- Existing dashboard and SLO folders include clock attestation and replay health surfaces.
- The decision must define strict, tolerance, and flexible policy tiers.
- The decision must make compliance behavior explicit across U.S., EU, and KR contexts.

## Decision

- Adopt `ClockGeofenceTierMatrix v1`.
- Define tier `strict_site` for high-security fixed worksites.
- Define tier `tolerant_site` for normal fixed worksites with location uncertainty.
- Define tier `flexible_work` for field, hybrid, remote, and travel work.
- Make `tolerant_site` the default tier for new tenants.
- Prohibit silent unpaid denial when location evidence is uncertain.
- In `strict_site`, deny automatic clock-in when device position is outside approved geofence and no supervisor exception exists.
- In `tolerant_site`, accept clock-in inside geofence plus accuracy tolerance radius, and route low-confidence cases to dispute review.
- In `flexible_work`, accept clock-in with declared work location, purpose code, and optional supervisor or shift assignment proof.
- Require every clock event to record evidence quality, not only allow or deny.
- Require every denied or disputed clock attempt to create a wage-risk review item.
- Require payroll exports to distinguish `payable_time`, `disputed_time`, and `non_payable_denied_time`.
- Require employee-visible explanation for denial, acceptance with low confidence, or supervisor override.
- Use one-time geolocation sampling at clock event time, not continuous tracking.
- Use device attestation when available but do not require biometrics for v1.
- Use geofence polygons with configured tolerance meters and accuracy ceilings.
- Use location evidence expiration of five minutes for submitted clock events.
- Use supervisor override with mandatory reason code and audit-chain seal.
- Use employee dispute with mandatory employee statement and optional supporting evidence.
- Use tenant policy to select tier by worksite, worker group, shift type, and jurisdiction.
- Use pack overlays for U.S. FLSA, EU WTD, and KR Labor Standards Act behavior.
- Use Cedar to gate policy update, clock acceptance, supervisor override, payroll export, and evidence read.
- Use location retention minimization: keep rounded location after payroll finalization unless legal hold requires more.
- Use raw coordinates only for active dispute, active investigation, or short validation window.
- Publish `workplace.clock.attested.v1` for accepted clock events.
- Publish `workplace.clock.disputed.v1` for disputed or low-confidence events.
- Publish `workplace.clock.denied.v1` for refused events.
- Publish `workplace.clock.override.applied.v1` for supervisor overrides.
- Keep payroll system output as an export, not authoritative payroll computation.
- Keep HRIS roster identity as a dependency, not owned by workplace-integration.
- Make this ADR authoritative for clock-in location policy.

## Alternatives Considered

### Strict Geofence Only

- Pros: simple enforcement.
- Pros: strong worksite presence signal.
- Pros: attractive for secure facilities.
- Cons: can deny compensable work when GPS is noisy.
- Cons: creates wage and dispute risk under FLSA-like regimes.
- Cons: poor fit for field and remote work.
- Rejected as default; retained for high-security worksites.

### Tolerance Geofence Default

- Pros: handles normal GPS uncertainty.
- Pros: reduces false denial while preserving worksite signal.
- Pros: pairs well with dispute review.
- Cons: can allow near-site fraud if tolerance is too wide.
- Cons: requires evidence-quality metrics.
- Cons: tenants must maintain worksite polygons and tolerances.
- Accepted as default because it balances wage compliance and fraud control.

### Fully Flexible Self-Attestation

- Pros: best fit for remote and field work.
- Pros: minimizes precise location collection.
- Pros: avoids wage denial from device failures.
- Cons: weaker fraud prevention.
- Cons: more supervisor and audit review burden.
- Cons: less useful for site-safety requirements.
- Accepted only for configured worker groups and work types.

### Continuous Location Tracking

- Pros: strong evidence of worksite presence over a shift.
- Pros: can detect early departure and offsite work.
- Pros: useful for some field operations.
- Cons: disproportionate privacy impact.
- Cons: increases regulatory and employee trust risk.
- Cons: not necessary for clock event attestation.
- Rejected for v1 because one-time attestation is sufficient and less invasive.

### Payroll System Decides Clock Validity

- Pros: pushes labor-law logic to existing payroll provider.
- Pros: lower implementation burden.
- Pros: aligns with external payroll exports.
- Cons: fragments evidence and Cedar policy.
- Cons: weakens employee dispute and audit-chain posture.
- Cons: external payroll systems may not preserve tenant pack rules.
- Rejected because workplace-integration owns clock evidence and exports typed payroll rows.

## Consequences

- Positive: default tolerance reduces false unpaid denials.
- Positive: strict mode remains available for secure worksites.
- Positive: flexible mode supports field, hybrid, remote, and travel scenarios.
- Positive: wage-risk review exists for denied and disputed attempts.
- Positive: employee privacy is protected by avoiding continuous tracking.
- Positive: payroll exports include evidence quality and dispute state.
- Positive: U.S., EU, and KR compliance concerns are named in policy.
- Negative: tier selection requires tenant administration.
- Negative: dispute queues become operationally important.
- Negative: evidence-quality thresholds need calibration by device and site.
- Negative: flexible work can increase fraud risk if review discipline is weak.
- Neutral: payroll remains outside this service.
- Neutral: identity and roster binding remain dependencies.
- Neutral: raw coordinates have short retention unless dispute or hold extends them.
- Neutral: strict mode is opt-in and policy-controlled.
- Follow-up work WPI-F1: add worksite polygon editor contract.
- Follow-up work WPI-F2: add dispute review runbook.
- Follow-up work WPI-F3: add payroll export fixture with disputed time.
- Follow-up work WPI-F4: add geofence calibration dashboard.
- Follow-up work WPI-F5: add employee notice copy in application shell.

## Implementation Notes

- Data shape `WorksiteGeofence`: `{tenant_id, worksite_id, polygon, tolerance_meters, accuracy_ceiling_meters, tier, jurisdiction}`.
- Data shape `ClockPolicy`: `{tenant_id, policy_id, worker_group_id, tier, payable_denial_rule, dispute_sla_hours, pack_code}`.
- Data shape `ClockAttempt`: `{tenant_id, attempt_id, worker_id, worksite_id, shift_id, observed_at, idempotency_key}`.
- Data shape `LocationEvidence`: `{attempt_id, lat_rounded, lon_rounded, accuracy_meters, provider, collected_at, raw_ref_ttl}`.
- Data shape `ClockDecision`: `{attempt_id, decision, confidence, payable_state, cedar_permit_id, reason_codes}`.
- Data shape `ClockDispute`: `{tenant_id, dispute_id, attempt_id, worker_statement, reviewer_id, status, resolved_at}`.
- Data shape `SupervisorOverride`: `{tenant_id, override_id, attempt_id, supervisor_id, reason_code, audit_event_id}`.
- Data shape `PayrollExportRow`: `{tenant_id, worker_id, work_date, payable_time, disputed_time, denied_time, evidence_quality}`.
- Postgres table `wpi_worksite_geofence` stores polygons and tolerances.
- Postgres table `wpi_clock_policy` stores tier selection rules.
- Postgres table `wpi_clock_attempt` stores idempotent attempts.
- Postgres table `wpi_location_evidence` stores minimized location data.
- Postgres table `wpi_clock_decision` stores allow, dispute, deny, or override state.
- Postgres table `wpi_clock_dispute` stores wage-risk review.
- REST endpoint `POST /v1/workplace/clock-attempts` records an attempt.
- REST endpoint `GET /v1/workplace/clock-attempts/{attempt_id}` returns decision and evidence quality.
- REST endpoint `POST /v1/workplace/clock-attempts/{attempt_id}/disputes` opens employee dispute.
- REST endpoint `POST /v1/workplace/clock-attempts/{attempt_id}/overrides` applies supervisor override.
- REST endpoint `PUT /v1/workplace/worksites/{worksite_id}/geofence` updates polygon and tolerance.
- REST endpoint `POST /v1/workplace/payroll-exports` emits payroll export rows.
- AsyncAPI channel `workplace.clock.attested.v1` publishes accepted clock event.
- AsyncAPI channel `workplace.clock.disputed.v1` publishes dispute and low-confidence review event.
- AsyncAPI channel `workplace.clock.denied.v1` publishes denied event with wage-risk review id.
- AsyncAPI channel `workplace.clock.override.applied.v1` publishes supervisor override.
- Cedar action `workplace::clock::attempt` requires worker roster membership and active shift or flexible policy.
- Cedar action `workplace::clock::override` requires supervisor scope over worker group.
- Cedar action `workplace::clock::dispute_read` requires worker, supervisor, payroll, or auditor role.
- Cedar action `workplace::geofence::update` requires tenant admin and change reason.
- Cedar action `workplace::payroll_export::create` requires payroll operator and finalized period.
- SLO target `workplace_clock_decision_p95_ms` is <=250.
- SLO target `workplace_clock_false_denial_review_ratio` is 1.0 for denied attempts.
- SLO target `workplace_clock_dispute_resolution_p95_hours` is <=72.
- SLO target `workplace_clock_export_accuracy_ratio` is 1.0 after finalization.
- SLO target `workplace_location_raw_retention_violation_total` is 0.

## Verification

- Unit test `tolerant_site_accepts_within_accuracy_radius` proves default tolerance behavior.
- Unit test `strict_site_denies_outside_polygon_without_override` proves strict enforcement.
- Unit test `flexible_work_accepts_declared_location_with_shift` proves remote and field mode.
- Unit test `denied_attempt_creates_wage_risk_review` proves no silent unpaid denial.
- Unit test `raw_location_expires_without_dispute_or_hold` proves privacy minimization.
- Unit test `payroll_export_separates_disputed_time` proves export semantics.
- Unit test `supervisor_override_requires_reason_and_permit` proves Cedar gate.
- Contract test `clock_attempt_response_contains_evidence_quality` proves client status display.
- Contract test `payroll_export_row_contains_payable_disputed_denied` proves payroll integration.
- Property test `same_idempotency_key_returns_same_clock_decision` proves retry safety.
- Integration test `flsa_pack_routes_denied_work_to_review` proves U.S. wage-risk flow.
- Integration test `eu_pack_checks_rest_window_before_acceptance` proves WTD overlay hook.
- Integration test `kr_pack_records_labor_standard_policy_ref` proves KR overlay hook.
- Failure test `location_provider_unavailable_routes_to_dispute_not_silent_deny` proves safe fallback.
- Failure test `audit_chain_unavailable_blocks_success_response` proves evidence-first posture.
- Security test `tenant_admin_cannot_read_raw_worker_location_without_purpose` proves data minimization.
- Security test `worker_cannot_override_own_clock_attempt` proves role boundary.
- Metric `workplace_clock_attempt_total` tracks attempts by tier, outcome, and pack.
- Metric `workplace_clock_decision_latency_ms` tracks decision latency.
- Metric `workplace_clock_low_confidence_total` tracks noisy geolocation cases.
- Metric `workplace_clock_denied_wage_review_total` tracks denied attempts requiring review.
- Metric `workplace_location_raw_retention_seconds` tracks location retention.
- Dashboard `workplace-clock-geofence-tier-matrix` shows tier usage, allow, deny, dispute, and override.
- Dashboard `workplace-geofence-evidence-quality` shows accuracy, provider, and site calibration.
- Dashboard `workplace-wage-risk-review` shows denied attempt review backlog and SLA burn.
- Dashboard `workplace-payroll-export-clock-quality` shows export rows by evidence quality.
- Alert `WorkplaceClockSilentDenialRisk` fires if denied attempt lacks review id.
- Alert `WorkplaceClockDisputeSlaBurn` fires if p95 dispute resolution exceeds 72 hours.
- Alert `WorkplaceRawLocationRetentionViolation` fires on expired raw coordinate retention.
- Alert `WorkplaceClockDecisionLatencyBurn` fires when p95 exceeds 250 ms.

## References

- Internal: oya/workplace-integration/IP-journey-j37-clock-in-geofence.md
- Internal: oya/workplace-integration/policies/clock-attest.cedar
- Internal: docs/decisions/ADR-0700-ci-admission-live-apex.md
- Internal: docs/decisions/ADR-0702-identity-authz-live-apex.md
- Internal: docs/decisions/ADR-0706-observability-live-apex.md
- U.S. DOL FLSA hours worked Fact Sheet 22: https://www.dol.gov/agencies/whd/fact-sheets/22-flsa-hours-worked
- U.S. DOL recordkeeping and reporting: https://www.dol.gov/general/topic/workhours/hoursrecordkeeping
- U.S. DOL Handy Reference Guide to the FLSA: https://www.dol.gov/agencies/whd/compliance-assistance/handy-reference-guide-flsa
- U.S. DOL Field Assistance Bulletin 2024-1: https://www.dol.gov/sites/dolgov/files/WHD/fab/fab2024_1.pdf
- EUR-Lex Working Time Directive 2003/88/EC: https://eur-lex.europa.eu/eli/dir/2003/88/oj
- European Commission Working Time Directive page: https://employment-social-affairs.ec.europa.eu/policies-and-activities/rights-work/labour-law/working-conditions/working-time-directive_en
- Korea Ministry of Employment and Labor labor standards page: https://www.moel.go.kr/english/policy/laborStandards.do
- Korea Personal Information Protection Commission portal: https://www.pipc.go.kr/eng/
- NIST Privacy Framework: https://www.nist.gov/privacy-framework
- OpenTelemetry semantic conventions: https://opentelemetry.io/docs/concepts/semantic-conventions/
- OpenAPI Specification: https://spec.openapis.org/oas/
- AsyncAPI Specification: https://www.asyncapi.com/docs/reference/specification/latest
