---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j137-corporate-internal-audit-sox-controls-test
status: draft
date: 2026-05-20
related_adrs: [ADR-0311, ADR-0313, ADR-0307, ADR-0310, ADR-0243, ADR-0244, ADR-0028, ADR-0145, ADR-0263]
test_tier: e2e-internal-audit
ci_lane: oya-test-internal-audit-sox-404
---

# j137 — Integration test plan: Sam's Q2 SOX 404 audit

This plan defines the end-to-end tests verifying the j137 journey. Tests
are organized by phase + by invariant. Each test has fixtures,
acceptance criteria, audit-chain evidence expected, observability
emissions expected, and a regression-check on the personal-tenant
default-deny invariant (which MUST NOT be bypassed under any error
condition).

## 1. Test environment

| Component | Setup |
|---|---|
| Test work tenant | `test.marcus-corp.tenant` synthetic; packs `pack-us-sox-404-test`, `pack-pcaob-as5-test`, `pack-eu-whistleblower-2019-1937-test` |
| Test personal tenant (employee) | `test.tobi.adeyemi@oyatie.me` synthetic personal tenant |
| Test personal tenant (counterparty) | `test.klaus.fischer@oyatie.me` synthetic |
| Test conglomerate sibling | `test.marcus-corp-subsidiary-b.tenant` (used to test ADR-0313 cross-subsidiary deny) |
| Sam test principal | `sam.okafor.test@marcus-corp.com`; passkey simulated; audience_type=B2B_INTERNAL_AUDIT |
| Audrey test principal | `audrey.chen.test@marcus-corp.com`; role=audit_committee_chair |
| Test fixtures: 210 invoices spread across Q2-test-window with stratified amounts; 47 of them have correlated personal-tenant principals |
| Cedar policy set | loaded from `policies/internal-audit/q2-sox-404-test.cedar` |
| audit-chain | local test instance with Merkle root reset per suite |
| Observability | local Mimir + Tempo + Loki; metrics asserted per test |
| Time | synthetic clock starting `2026-07-12T22:00:00Z` |

## 2. Phase 1 tests — Permit request and dual-control

### 2.1 `test_permit_request_creates_audit_case`

**Setup:** Sam principal exists; charter active.

**Action:** POST `/api/v1/internal-audit/cases` with Q2 scope.

**Assert:**
- HTTP 202 within 1000ms.
- Response body contains `audit_case_id`.
- Audit event `InternalAuditCaseCreated` sealed.
- ops-dashboard pane reflects new case.
- Cedar permit batch in DRAFT state (not yet active).

### 2.2 `test_dual_control_required_for_activation`

**Setup:** Case created in 2.1.

**Action:** Sam attempts to use the permit BEFORE Audrey co-signs.

**Assert:**
- All read attempts return Cedar DENY.
- `oya_cedar_evaluate_total{decision="deny",reason="missing_dual_control"}`
  increments.
- No data exposed.

### 2.3 `test_co_sign_activates_permit`

**Setup:** Case created.

**Action:** Audrey co-signs via passkey ceremony (simulated).

**Assert:**
- `CedarPermitCoSigned` audit event sealed.
- Permit batch ACTIVE.
- `context.dual_control_approval_at` populated.
- Sam's next read attempt returns PERMIT.

### 2.4 `test_co_sign_by_unauthorized_principal_fails`

**Setup:** Case created; attempt co-sign by a non-audit-committee-chair
principal.

**Assert:**
- HTTP 403 with reason "not audit_committee_chair".
- No permit activation.
- `CedarPermitCoSignAttemptDenied` audit event sealed.

### 2.5 `test_co_sign_timeout_expires_case`

**Setup:** Case created; advance clock 25h with no co-sign.

**Assert:**
- Case auto-transitions to EXPIRED.
- Permit batch revoked.
- `AuditCaseExpired` event sealed.

## 3. Phase 2 tests — Sample pull happy path

### 3.1 `test_single_sample_pull_happy_path`

**Setup:** Permit active. Test invoice #247811-test ($712,400) with
4-node approval chain, 18 messenger messages, 12 mail messages, no
personal-tenant correlation.

**Action:** Pull sample.

**Assert:**
- HTTP 200 within 60s p95.
- Sample evidence bundle assembled with 5 sub-bundles (payments,
  messenger, mail, workflow logs, audit-chain seals).
- Audit event `SamplePullEvidenceSealed` sealed.
- Merkle proof verifies against the audit-chain root.
- `oya_internal_audit_sample_pull_total{outcome="ok"}` increments.

### 3.2 `test_sample_pull_p95_under_60s_at_concurrency_5`

**Setup:** Permit active; 5 concurrent sample pulls in flight.

**Action:** Pull 25 samples in 5-wide concurrency.

**Assert:**
- p95 ≤ 60s.
- p99 ≤ 90s.
- No data leakage across concurrent pulls (each sample isolated).
- All 25 evidence bundles sealed.

### 3.3 `test_sample_pull_with_audit_chain_brownout`

**Setup:** Permit active; induce audit-chain brownout mid-pull.

**Action:** Pull sample.

**Assert:**
- workflow-engine pauses at seal-point.
- Pane shows "audit-chain in brownout".
- When brownout clears, pull resumes from last sealed step.
- Final evidence bundle is intact and Merkle-verified.

## 4. Phase 3 tests — Personal-tenant deny (the boundary test)

### 4.1 `test_personal_tenant_principal_correlation_returns_deny`

**Setup:** Test invoice with correlated personal-tenant principal
(Tobi-test's personal tenant has Messenger thread on date in scope).

**Action:** Pull sample.

**Assert:**
- Work-tenant content returned in full.
- Personal-tenant content NEVER appears in the response body.
- Response includes `personal_tenant_deny_count: 1`.
- Response includes principal-class label `"personal_tenant_owned"` but
  NO principal content or metadata.
- Audit event `MessengerPersonalTenantReadDenied` sealed.
- `oya_personal_tenant_deny_total` increments by exactly 1.

### 4.2 `test_3645_denies_across_full_audit_match_expected`

**Setup:** Full Q2 sample (210 samples) with the standard test fixture
(47 invoices with personal-tenant correlation; total 3,645 denies
expected per fixture).

**Action:** Run full audit.

**Assert:**
- Exactly 3,645 deny events sealed.
- Zero personal-tenant content in any response.
- `oya_personal_tenant_deny_total` at exactly 3,645.

### 4.3 `test_personal_tenant_deny_cannot_be_circumvented_by_principal_overlap`

**Setup:** Sam's personal-tenant principal is `sam.okafor@oyatie.me`
(the SAME human). Test injects a sample whose personal-tenant
principal happens to be Sam's own personal tenant.

**Assert:**
- Even for Sam's OWN personal tenant, the audit permit denies.
- (Sam can read his personal-tenant content via his B2C_CONSUMER
  permit; not via the audit permit.)
- This is the strongest version of the boundary: it holds even
  for the auditor's own personal tenant.

### 4.4 `test_personal_tenant_deny_under_cedar_gate_timeout`

**Setup:** Induce Cedar gate timeout during a personal-tenant
correlated read.

**Assert:**
- Fail-closed: deny is preserved.
- No personal-tenant content exposed even on timeout.
- Pane shows degraded-state notice; sample pull retries.

### 4.5 `test_personal_tenant_deny_event_does_not_expose_principal_identity_in_body`

**Setup:** Single sample with one personal-tenant deny.

**Assert:**
- The audit event payload contains a principal-class label only.
- The principal-id (e.g., the actual email) is reconstructable
  from the work-tenant CORRELATION (where Sam has permit) but is
  NOT present in the deny event itself.
- This is intentional: the deny event MUST be auditable without
  exposing the personal-tenant principal's identity.

## 5. Phase 4 tests — Evidence-pack assembly

### 5.1 `test_evidence_pack_assembly_happy_path`

**Setup:** All 210 samples pulled.

**Action:** Assemble evidence pack.

**Assert:**
- Pack manifest created with 1,247 leaves.
- Merkle root computed.
- Pack signature recorded (Sam's passkey).
- Co-signature recorded (Audrey's passkey).
- `EvidencePackRootSealed` event sealed.
- `oya_audit_pack_assembly_ms` recorded.

### 5.2 `test_evidence_pack_signature_verification`

**Setup:** Assembled pack from 5.1.

**Action:** External verifier (mock PwC) fetches pack and verifies.

**Assert:**
- Merkle root re-computed externally matches stored root.
- Both signatures verify against the registered passkey public keys.
- No leaf is missing.

### 5.3 `test_evidence_pack_immutability`

**Setup:** Assembled pack from 5.1.

**Action:** Attempt to modify a leaf after seal.

**Assert:**
- Modification rejected at audit-chain layer.
- `audit-chain.mutability_violation_attempt_total` increments.

## 6. Phase 5 tests — External-auditor handoff

### 6.1 `test_external_auditor_handoff_signed_url`

**Setup:** Pack assembled.

**Action:** Sam triggers handoff.

**Assert:**
- Signed URL generated with TTL of 24h.
- PwC mock fetches URL successfully.
- PwC mock verifies Merkle root.
- `ExternalAuditorHandoffRequested` and `ExternalAuditorPackFetched`
  events sealed.

### 6.2 `test_external_auditor_handoff_url_expiry`

**Setup:** Handoff URL generated; advance clock 25h.

**Action:** PwC mock attempts to fetch.

**Assert:**
- HTTP 410 Gone.
- Sam can re-generate a new URL.

### 6.3 `test_external_auditor_handoff_to_unregistered_party_fails`

**Setup:** Attempt handoff to non-pre-registered external party.

**Assert:**
- Cedar DENY at handoff request.
- `ExternalAuditorHandoffAttemptDenied` event sealed.

## 7. Phase 6 tests — Audit case closure

### 7.1 `test_audit_case_close_revokes_permit`

**Setup:** Case in CLOSED-READY state.

**Action:** Sam closes case.

**Assert:**
- Cedar permit batch REVOKED.
- Sam's subsequent reads fail with DENY.
- `AuditCaseClosureSealed` and `CedarPermitBatchRevoked` events sealed.

### 7.2 `test_audit_case_close_with_unresolved_findings_blocks`

**Setup:** Case with `F-...-001` finding still in OPEN state.

**Action:** Sam closes case.

**Assert:**
- HTTP 409 Conflict; "unresolved findings prevent close".
- Case remains open.

## 8. ADR-0313 conglomerate-hierarchy invariants

### 8.1 `test_sam_permit_does_not_extend_to_sibling_subsidiary`

**Setup:** Marcus's conglomerate has another subsidiary tenant
`test.marcus-corp-subsidiary-b.tenant`. Sam's permit is for
`test.marcus-corp.tenant` only.

**Action:** Sam attempts to read from sibling subsidiary tenant.

**Assert:**
- Cedar DENY.
- No content exposed.
- `oya_cross_subsidiary_deny_total` increments.
- Event `ConglomerateSiblingTenantReadDenied` sealed.

### 8.2 `test_separate_permit_required_per_subsidiary`

**Setup:** Sam needs to audit sibling B; separate audit-charter
exists for sibling B.

**Action:** Sam requests a separate permit batch for sibling B.

**Assert:**
- Sibling B's audit-committee-chair must co-sign (different
  principal).
- After co-sign, permit B is active in addition to permit A; they
  do not cross-extend.

## 9. ADR-0307 detection-substrate cross-references

### 9.1 `test_detection_substrate_flag_appears_in_audit_pane`

**Setup:** detection µservice emits a `ChannelStuffingRiskKeyword`
flag for an invoice in scope.

**Action:** Sam's audit case loads.

**Assert:**
- Flag appears in audit pane as `auto_flag`.
- Sam can mark for walkthrough interview.
- Flag is sealed into the audit-case event log.

(Note: deeper detection-substrate integration is tested in j138.)

## 10. ADR-0310 case-management cross-references

### 10.1 `test_audit_case_lifecycle_states`

**Setup:** Empty.

**Action:** Create case → activate → pull samples → flag findings →
resolve findings → close.

**Assert:**
- State transitions match ADR-0310 lifecycle.
- Each transition is sealed.
- Cannot skip states (e.g., cannot close with unresolved finding).

## 11. Performance and load tests

### 11.1 `test_p95_pull_under_60s_at_realistic_concurrency`

Run 60 sample pulls concurrent at realistic system load (with other
internal-audit cases in flight for other tenants).

**Assert:**
- p95 ≤ 60s; p99 ≤ 90s.
- No cross-tenant data leakage.

### 11.2 `test_audit_chain_seal_latency_at_load`

**Assert:** p95 seal latency ≤ 200ms even at 100 seal/s sustained.

## 12. Observability assertion suite

### 12.1 `test_all_observability_metrics_emitted`

After a complete audit-case run, verify the metric catalog from
handshake.md §"Observability emissions summary" — every metric listed
must have non-zero values matching the expected cardinality.

### 12.2 `test_trace_continuity_end_to_end`

Single sample pull traces with a single root span; every gRPC hop
preserves the trace_id; the trace renders coherently in Tempo.

## 13. Negative tests — security regression

### 13.1 `test_principal_with_only_b2c_consumer_audience_type_cannot_internal_audit`

**Setup:** Test principal with `audience_type=B2C_CONSUMER`.

**Action:** Attempt to create an internal-audit case.

**Assert:**
- Cedar DENY.
- `InternalAuditCaseCreationAttemptDenied` sealed.

### 13.2 `test_revoked_audit_charter_blocks_new_cases`

**Setup:** Charter revoked.

**Action:** Sam attempts to create a case.

**Assert:**
- Cedar DENY (context.audit_charter_active=false).

### 13.3 `test_expired_permit_batch_blocks_new_reads`

**Setup:** Permit batch expired (clock advanced past expiry).

**Action:** Sam attempts to pull a sample.

**Assert:**
- Cedar DENY.
- No data exposed.

### 13.4 `test_personal_tenant_resource_cannot_be_targeted_directly`

**Setup:** Permit active.

**Action:** Sam attempts to directly URL-construct a personal-tenant
resource read.

**Assert:**
- Cedar DENY at api-gateway (URL parse path).
- No data exposed.
- `DirectPersonalTenantReadAttemptDenied` sealed.

### 13.5 `test_sql_injection_in_sample_filter_blocked`

**Setup:** Sam attempts to inject SQL via sample filter parameters.

**Assert:**
- api-gateway input validation rejects.
- No SQL reaches workflow-engine.
- `MalformedAuditFilterRejected` sealed.

## 14. Locale and i18n tests

### 14.1 `test_audit_pane_renders_all_8_locales`

Render the audit pane in each of the 8 supported locales; assert
no missing translations; assert correct date/time format per locale.

### 14.2 `test_personal_tenant_deny_panel_aria_live_announces_per_locale`

For each locale, the `aria-live=assertive` text matches the locale's
translation of "Personal-tenant boundary — N denies".

## 15. Cross-jurisdiction overlay tests

### 15.1 `test_eu_counterparty_triggers_eu_wb_pack_overlay`

**Setup:** Sample includes a German distributor as counterparty.

**Assert:**
- compliance µservice activates `pack-eu-whistleblower-2019-1937`
  overlay.
- Audit event tagged with active pack set including EU-WB.

### 15.2 `test_nigeria_employee_residency_triggers_ndpr_overlay`

**Setup:** Sales rep is Nigerian resident.

**Assert:**
- `pack-ng-data-protection-2023` active.

## 16. Stress and chaos tests

### 16.1 `test_audit_chain_partial_failure_does_not_corrupt_pack`

**Setup:** Inject 5% seal failures during sample pull.

**Assert:**
- workflow-engine retries failed seals (max 3) — succeeds.
- Final pack has no missing leaves.
- No personal-tenant data leaked due to retry path.

### 16.2 `test_messenger_brownout_degrades_partially`

**Setup:** Messenger archive returns 503 on 10% of reads.

**Assert:**
- Pane flags partial-evidence on affected samples.
- Sam can retry just the failed samples.
- No personal-tenant data exposed even during degradation.

## 17. Test acceptance gates

The integration suite must:

- All 80+ test cases PASS.
- No flake rate >1% across 100 runs.
- Total wall-clock ≤ 12 minutes in CI lane `oya-test-internal-audit-sox-404`.
- Zero personal-tenant content exposed in any test run.
- All audit-chain seals verified Merkle-clean.
- All Cedar evaluations have matching audit events.

## 18. CI lane wiring

The test plan runs in CI lane `oya-test-internal-audit-sox-404`,
gated by:

- `oya-lint-cedar` (Cedar policy lint clean)
- `oya-schema-validate` (all schemas validate JSON Schema 2020-12)
- `oya-fixture-generate` (test fixtures generated deterministically)
- `oya-test-internal-audit-sox-404` (this suite)
- `oya-test-internal-audit-personal-tenant-boundary` (cross-suite
  regression — shared across j137/j138/j139/j140/j141)

## 19. Test fixture generation

Fixtures are generated by `tools/internal-audit-fixture-generator`:

```
$ tools/internal-audit-fixture-generator \
    --tenant test.marcus-corp.tenant \
    --quarter Q2 \
    --year 2026 \
    --sample-size 60 \
    --personal-tenant-correlation-pct 22 \
    --output test/fixtures/j137/
```

The generator produces:
- 60 invoices (stratified)
- ~3,645 personal-tenant correlations (deterministic seed)
- ~1,247 work-tenant-message threads
- Pre-sealed audit-chain history for the test window
- Cedar permit batch in draft state

## 20. Closing test invariants

This test suite enforces:

- The personal-tenant default-deny holds 100% of the time, including
  under all failure modes.
- Sam's audit work is itself audited.
- Cross-subsidiary deny holds 100% of the time.
- The dual-control gate cannot be bypassed.
- All observability metrics emit per the handshake catalog.
- Cedar policy timeout is fail-closed (deny).
- The evidence pack is immutable post-seal.
- Locale rendering is complete in all 8 supported locales.
- The audit-pane assembly time is within SLA at realistic load.

Without all 80+ tests passing, the j137 surface is not promotable to
production.
