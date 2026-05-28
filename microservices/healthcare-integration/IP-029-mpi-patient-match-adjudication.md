# IP-029 Healthcare Integration MPI patient match adjudication

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-029-mpi-patient-match-adjudication.md
Doc class: Implementation Plan
Batch: C healthcare-integration IP deepening
Status: authoring-ready
Owner: axis-healthcare-integration
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo-local references: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/ARCHITECTURE.md; microservices/healthcare-integration/capabilities/patient-match-review.yaml; microservices/healthcare-integration/runbooks/local-patient-match-duplicate.md; microservices/healthcare-integration/runbooks/patient-match-duplicate.md; microservices/healthcare-integration/contracts/openapi-v1.yaml; microservices/healthcare-integration/contracts/asyncapi-v1.yaml; microservices/healthcare-integration/contracts/healthcare-integration-v1.proto; microservices/healthcare-integration/dashboards/local-operator-remediation.json; microservices/healthcare-integration/slos/local-fhir-bundle-success.openslo.yaml
Benchmarks displaced: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, Health Catalyst

## Objective
- Define an atomic implementation plan for MPI patient match review and adjudication.
- Keep probabilistic matching separate from consent, authorization, and provenance.
- Require human review for ambiguous, duplicate, conflicting, or high-impact patient match decisions.
- Bind all adjudication decisions to tenant scope, Cedar permit, audit-chain evidence, and ontology projection.
- Prevent NextGate-style identity confidence from becoming unconditional clinical data access.
- Preserve replayability for HL7 ACK custody and FHIR bundle segmentation.
- Keep this IP documentation-only and limited to the assigned file.

## Match candidate fields
- Field 001: tenant_id scopes the candidate set.
- Field 002: principal_id identifies the actor or worker requesting adjudication.
- Field 003: audience_type identifies operator, reviewer, auditor, or approved worker.
- Field 004: purpose identifies treatment, operations, import, replay, or remediation.
- Field 005: data_class identifies hl7_message, fhir_resource, or patient_match_review.
- Field 006: source_system_id identifies the origin system.
- Field 007: source_patient_key stores source-local patient reference.
- Field 008: candidate_patient_id identifies local patient projection candidate.
- Field 009: match_score stores deterministic score version output.
- Field 010: match_score_version stores algorithm and threshold version.
- Field 011: match_reason_codes store exact contributing signals.
- Field 012: conflict_reason_codes store exact blocking signals.
- Field 013: demographic_hash stores privacy-preserving comparison evidence.
- Field 014: identifier_hash stores privacy-preserving identifier comparison evidence.
- Field 015: encounter_context_id stores context when available.
- Field 016: consent_record_id stores relevant consent linkage without granting access.
- Field 017: route_custody_id stores HL7 custody linkage when adjudication blocks ACK AA.
- Field 018: fhir_bundle_job_id stores bundle linkage when adjudication blocks export.
- Field 019: reviewer_queue_id stores review assignment.
- Field 020: audit_event_class stores ADR-0263 event family.

## Adjudication states
- State 001: candidate-generated means the match engine proposed one or more candidates.
- State 002: auto-rejected means policy or threshold rejected the candidate.
- State 003: auto-accepted-low-risk means deterministic threshold allowed acceptance for low-risk scope.
- State 004: review-required means ambiguity, conflict, high-risk data class, or pack rule requires human review.
- State 005: reviewer-assigned means a qualified reviewer owns the case.
- State 006: evidence-collected means source, demographic, identifier, consent, and route evidence are attached.
- State 007: accepted means reviewer selected a candidate.
- State 008: rejected means reviewer rejected all candidates.
- State 009: split-required means one source record maps to multiple tenant records.
- State 010: merge-required means multiple tenant records map to one patient.
- State 011: conflict-escalated means reviewer escalated to tenant admin or compliance.
- State 012: correction-requested means source system must correct upstream data.
- State 013: replay-ready means blocked HL7 or FHIR work can replay after decision.
- State 014: replayed means downstream work replayed with adjudication reference.
- State 015: superseded means a newer source message or FHIR resource replaces the case.
- State 016: appealed means tenant or patient representative challenges the decision.
- State 017: archived means retention controls apply.
- State 018: final means all required evidence and downstream work are complete.
- State 019: regulator-exported means export packet was produced.
- State 020: deleted is forbidden for adjudication history.

## Review decision rules
- Rule 001: deny adjudication when tenant_id is absent.
- Rule 002: deny adjudication when principal_id is absent.
- Rule 003: deny adjudication when audience_type lacks reviewer authority.
- Rule 004: deny adjudication when purpose is not import, treatment, operations, replay, or remediation.
- Rule 005: deny adjudication when data_class is outside approved classes.
- Rule 006: deny auto-accept when match_score_version is deprecated.
- Rule 007: deny auto-accept when candidate crosses tenant boundary.
- Rule 008: deny auto-accept when pack requires human review.
- Rule 009: deny auto-accept when consent conflict exists.
- Rule 010: deny auto-accept when demographic and identifier signals disagree materially.
- Rule 011: deny auto-accept when source_system_id is untrusted or quarantined.
- Rule 012: deny auto-accept when route custody is under dispute.
- Rule 013: deny reviewer decision without attached evidence.
- Rule 014: deny reviewer decision without audit-chain write.
- Rule 015: deny reviewer decision if credential sidecar cannot prove source context.
- Rule 016: permit low-risk auto-accept only under tenant-approved thresholds.
- Rule 017: permit replay only after decision is final or explicitly replay-ready.
- Rule 018: permit split or merge only with tenant-admin notification.
- Rule 019: permit regulator export only with redacted evidence packet.
- Rule 020: emit refusal evidence for every deny.

## Candidate scoring requirements
- Score 001: deterministic score version is required.
- Score 002: thresholds are tenant and pack aware.
- Score 003: demographic features are privacy-preserving and explainable.
- Score 004: identifier features are hashed or tokenized in evidence.
- Score 005: source-system trust tier contributes to score.
- Score 006: route context can raise review requirement but not grant access.
- Score 007: consent context can block acceptance but not grant identity.
- Score 008: prior adjudication can contribute only if not superseded.
- Score 009: exact identifier match can still require review under pack rules.
- Score 010: fuzzy demographic match cannot auto-accept high-risk cases.
- Score 011: score output includes reason codes.
- Score 012: score output includes conflict codes.
- Score 013: score output includes version and threshold ids.
- Score 014: score output includes replay determinism hash.
- Score 015: score output includes audit event id.
- Score 016: score output excludes raw patient identifiers from metrics.
- Score 017: score model changes follow ADR-0258 deprecation.
- Score 018: score projection changes follow ADR-0257.
- Score 019: score threshold changes follow ADR-0294 soak.
- Score 020: score evidence is retained for review and regulator export.

## MPI Adjudication Benchmark Displacement
- Displacement claim: this IP measures competitors against patient-match adjudication, so every comparison must cover candidate scoring, reviewer authority, consent separation, and replay prerequisites.
- Non-generic rule: identity confidence alone is never enough; the displacement proof must include adjudication state and downstream custody impact.
- Redox displacement: Redox patient identity routing is displaced by adjudication evidence attached to each blocked exchange.
- Redox proof: route custody and FHIR jobs carry adjudication references.
- Rhapsody displacement: interface-engine patient routing is displaced by tenant-owned MPI decision states.
- Rhapsody proof: reviewer decision ids become replay prerequisites.
- InterSystems IRIS for Health displacement: platform MPI is displaced by flat service adjudication and explicit ADR bindings.
- InterSystems proof: match scoring, review, and replay stay under healthcare-integration.
- Lyniate/Corepoint displacement: channel duplicate handling is displaced by structured split, merge, reject, and replay states.
- Lyniate/Corepoint proof: local patient-match runbooks own remediation.
- Mirth displacement: script-based patient routing is displaced by deterministic scoring and Cedar-gated review.
- Mirth proof: score rules are versioned and testable outside scripts.
- NextGate displacement: NextGate identity confidence is displaced by consent-separated, reviewer-audited adjudication.
- NextGate proof: match confidence never authorizes FHIR delivery by itself.
- Health Catalyst displacement: analytics patient stitching is displaced by operational patient-match evidence and replay custody.
- Health Catalyst proof: every downstream analytic or export path cites adjudication event ids.

## Failure modes
- Failure 001: no candidates opens correction-requested state.
- Failure 002: too many candidates opens review-required state.
- Failure 003: conflicting identifiers opens conflict-escalated state.
- Failure 004: deprecated score version blocks auto-accept.
- Failure 005: tenant boundary conflict blocks match.
- Failure 006: source trust quarantine blocks match.
- Failure 007: consent conflict blocks downstream access.
- Failure 008: route custody dispute blocks ACK AA.
- Failure 009: FHIR bundle job blocks export.
- Failure 010: reviewer unavailable escalates queue.
- Failure 011: audit-chain outage blocks final decision.
- Failure 012: credential sidecar outage blocks source proof.
- Failure 013: replay hash mismatch blocks replay.
- Failure 014: regulator export redaction failure blocks export.
- Failure 015: attempted deletion of adjudication history is forbidden.

## Capacity and performance
- Capacity 001: candidate generation partitions by tenant, source_system_id, and home_cell.
- Capacity 002: reviewer queues partition by severity, pack, tenant, and data class.
- Capacity 003: match score metrics use score bucket, not patient identifiers.
- Capacity 004: duplicate volume metrics use source_type and reason_code.
- Capacity 005: auto-accept thresholds are low-risk only.
- Capacity 006: high-risk queues have shorter escalation targets.
- Capacity 007: replay backlog is separate from review backlog.
- Capacity 008: regulator export jobs are idempotent.
- Capacity 009: large imports use batch candidate generation with stable ordering.
- Capacity 010: Little's Law review queue math uses arrival rate, reviewer service time, and active backlog.
- Capacity 011: reviewer overload alerts before HL7 ACK latency burns.
- Capacity 012: FHIR bundle jobs wait without spinning.
- Capacity 013: MPI decisions do not bypass consent cache budgets.
- Capacity 014: cross-cell match requires metadata-only lookup unless pack allows.
- Capacity 015: audit event writes are part of final-decision latency.

## Observability
- Event `oya.healthcare.integration.patient_match.candidate_generated` records candidates.
- Event `oya.healthcare.integration.patient_match.review_required` records review trigger.
- Event `oya.healthcare.integration.patient_match.accepted` records acceptance.
- Event `oya.healthcare.integration.patient_match.rejected` records rejection.
- Event `oya.healthcare.integration.patient_match.replayed` records downstream replay.
- Metric `healthcare_integration_patient_match_candidates_total` dimensions: score_bucket, reason_code, cell, pack.
- Metric `healthcare_integration_patient_match_review_latency_seconds` dimensions: status, reviewer_role, pack.
- Metric `healthcare_integration_patient_match_replay_total` dimensions: downstream_type, status, cell.
- Trace span `healthcare.patient_match.adjudicate` wraps score, policy, review, audit, and replay.
- Log schema includes match_case_id, score_version, reviewer_id, decision_id, audit_event_id, and workflow_run_id.
- Dashboard reference: dashboards/local-operator-remediation.json.
- Runbook reference: runbooks/local-patient-match-duplicate.md.
- Runbook reference: runbooks/patient-match-duplicate.md.
- Capability reference: capabilities/patient-match-review.yaml.
- Contract references: openapi-v1.yaml, asyncapi-v1.yaml, and healthcare-integration-v1.proto.

## Implementation steps
- Step 001: Add patient match candidate value object.
- Step 002: Add adjudication aggregate in domain.
- Step 003: Add candidate generation usecase.
- Step 004: Add reviewer decision usecase.
- Step 005: Add split and merge remediation states.
- Step 006: Add route custody replay hook.
- Step 007: Add FHIR bundle replay hook.
- Step 008: Add Cedar tests for reviewer authority.
- Step 009: Add score-version deprecation tests.
- Step 010: Add audit-chain events for candidate, review, decision, replay, and export.
- Step 011: Add dashboard rows for queue and replay health.
- Step 012: Add contract examples for accepted, rejected, split, merge, and replay.
- Step 013: Add property tests for deterministic scoring.
- Step 014: Add replay tests for downstream blocked work.
- Step 015: Add benchmark displacement evidence to review packet.

## Tests and evidence
- Test 001: line count for this IP is at least 200.
- Test 002: ADR scan finds the full binding ADR list.
- Test 003: benchmark scan finds all seven named competitors.
- Test 004: local reference scan finds patient-match-review.yaml.
- Test 005: local reference scan finds both patient-match runbooks.
- Test 006: local reference scan finds contract references.
- Test 007: local reference scan finds local-operator-remediation.json.
- Test 008: review confirms consent and identity are separate gates.
- Test 009: review confirms ADR-0321 was not edited.
- Test 010: review confirms no oya vcs verify, done, or promote was run.

## Rollback
- Rollback 001: disable new score_version for affected tenant only.
- Rollback 002: force ambiguous cases to manual review.
- Rollback 003: retain prior candidate and decision evidence.
- Rollback 004: do not delete adjudication history.
- Rollback 005: replay blocked downstream work only after valid decision.
- Rollback 006: restore prior Cedar fragment only after soak-window rules permit.
- Rollback 007: notify tenant admin for split or merge rollback.
- Rollback 008: keep DealSet holds when identity decision affects billable exchange.
- Rollback 009: export regulator packet for affected decisions.
- Rollback 010: open remediation for score drift.

## Acceptance criteria
- AC01: Every match case carries tenant, principal, audience, purpose, data class, source, candidate, score version, and audit class.
- AC02: Identity confidence never grants consent or authorization.
- AC03: Ambiguous, conflicting, deprecated, or high-risk cases require review.
- AC04: Reviewer decisions are signed, replayable, and retained.
- AC05: Split and merge actions require tenant-admin notification.
- AC06: HL7 ACK and FHIR bundle work can wait on adjudication without losing custody.
- AC07: Score changes follow versioning and soak rules.
- AC08: Regulator exports redact patient identifiers.
- AC09: All seven named benchmarks are explicitly displaced.
- AC10: This plan remains scoped to the assigned IP file.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/healthcare-integration/IP-029-mpi-patient-match-adjudication.md:10` - Repo-local references: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/ARCHITECTURE.md; microservices/healthcare-integration/capabili...; `microservices/healthcare-integration/IP-029-mpi-patient-match-adjudication.md:177` - - Contract references: openapi-v1.yaml, asyncapi-v1.yaml, and healthcare-integration-v1.proto..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.
