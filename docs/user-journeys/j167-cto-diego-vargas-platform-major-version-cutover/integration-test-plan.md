---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j167-cto-diego-vargas-platform-major-version-cutover
date: 2026-05-20
authority_tier: 2
status: draft
---

# j167 — Integration test plan: cohort gates + rollback + sunset

## §0 — Test fixture inventory

| Fixture | Description |
|---|---|
| `aurelia-robotics-tenant.json` | Aurelia primary tenant + 6 named principals (diego, yamilet, akira, brian, sofia, renata) with WebAuthn-passkey workload identities |
| `aurelia-cell-topology.json` | 47 Tier-1 cell definitions + 3 Tier-0 + ~218 Tier-4 edge cells (mocked) |
| `oya-governance-tenant.json` | Substrate governance tenant for dual-seal target |
| `pwc-mexico-auditor-tenant.json` | PwC México SOC2 auditor read-only tenant |
| `dekra-eu-ai-act-tenant.json` | DEKRA EU-AI-Act notified-body tenant |
| `customer-tenants-fixture.json` | 11 named customer tenants on AUS-TX + 8 on CDMX + 1 Cotrijal on BRA-SAO + 7 long-tail v3 holdouts |
| `cra-document.pdf` | CRA risk assessment document; Yamilet Solís QES signature |
| `cedar-policy-bundle.cedar` | The full Cedar bundle for cohort transitions + rollback + sunset |
| `terraform-module-aurelia-platform-v4.tar.gz` | Pinned v4.0.0 Terraform module |
| `k8s-image-digests.yaml` | Pinned v4 container image SHA256 digests |
| `mock-truetime-driver.ts` | TrueTime fence mock with configurable uncertainty (default 2.4 ms) |
| `mock-slo-burn-injector.ts` | Injects synthetic SLO regression events for canary-spike + SEV-2 tests |

## §1 — Pre-cutover readiness checklist tests

### TEST-J167-001 — Readiness checklist renders 87/87 green

**Setup**: Load `aurelia-robotics-tenant.json` + `aurelia-cell-topology.json` + `cra-document.pdf` + CAB sign-off quorum already collected.

**Action**: `GET /v1/changes/CHG-V4-CUTOVER-2026-10-20/readiness` as Diego principal.

**Expected**:
- HTTP 200
- `readiness_checklist.total_items == 87`
- `readiness_checklist.green_items == 87`
- All 7 category objects show `total == green`
- `cra_signature.signer == "yamilet.solis@..."`
- `cab_signoff_quorum.length == 4` with all PERMIT
- Response includes audit_seal `EVT-J167-PRE-REVIEW-COMPLETE-001`
- Cedar policy decision latency for the read ≤ 5 ms

**Failure mode probe**: If any category has `total != green`, response carries `readiness_status == "blocked"` and `next_gate == null`.

### TEST-J167-002 — Cedar permit blocks read if principal not in quorum group

**Setup**: Same as TEST-J167-001 but call as an unrelated tenant-admin principal (`unrelated-engineer@aurelia-...`).

**Action**: Same GET.

**Expected**:
- HTTP 403
- Body: `{"error": "cedar_policy_denied", "principal": "unrelated-engineer@...", "action": "governance.readiness_read", "reason": "principal_not_in_group_aurelia-cutover-quorum-members"}`
- No audit seal written

## §2 — Cohort A Cedar permit vote tests

### TEST-J167-010 — Quorum 4-of-4 PERMIT advances state machine

**Setup**: Pre-review sealed. Vote window opens at 07:58:00 CDT.

**Actions** (sequential):
1. Diego POSTs vote at 07:58:18 CDT with `decision=PERMIT` + passkey attestation.
2. Yamilet POSTs vote at 07:58:22 CDT.
3. Brian POSTs vote at 07:58:31 CDT.
4. Akira POSTs vote at 07:58:42 CDT.

**Expected**:
- Each individual response: `quorum_progress.current` increments 1 → 2 → 3 → 4.
- After 4th vote: response includes `quorum_decision: "PERMIT"`, `audit_seal: "EVT-J167-COHORT-A-PERMIT-002"`, `dual_seal_tenants: ["aurelia-robotics-internacional-sa-de-cv-mx", "oya-governance-change-management-system-tenant"]`.
- TrueTime fence value `≤ 10 ms`.
- The `governance` workflow advances `pre_review → cohort_a_initiating`.
- `cloud-iac.apply_terraform_cohort_a` workflow auto-triggers within 6 seconds.

### TEST-J167-011 — Single DENY blocks quorum

**Setup**: Same.

**Actions**: Diego PERMIT, Yamilet PERMIT, Brian DENY (with rationale `"Customer A's beta-customer integration regression report from Oct 19 not yet root-caused"`), Akira waits.

**Expected**:
- After Brian's DENY: `quorum_status: "deny_threshold_reached"`, `audit_seal: "EVT-J167-COHORT-A-DENIED-002b"`, workflow state stays at `pre_review` (does NOT advance).
- Cohort A transition aborted.
- Notification dispatched to all 4 quorum members + the war-room channel.

### TEST-J167-012 — Cedar policy denies vote if outside business-hours

**Setup**: Same but mock clock to 03:42 CDT (outside `business_hours_cdt`).

**Actions**: Diego POSTs vote.

**Expected**:
- HTTP 403
- `{"error": "cedar_policy_denied", "reason": "business_hours_cdt_false", "context.business_hours_cdt": false}`
- No vote recorded.

### TEST-J167-013 — TrueTime uncertainty > 10ms fails fence precondition

**Setup**: Same but mock TrueTime driver to return `uncertainty_ms: 14.2`.

**Actions**: Diego POSTs vote.

**Expected**:
- HTTP 503
- `{"error": "truetime_fence_uncertain", "current_uncertainty_ms": 14.2, "max_allowed_ms": 10}`
- Vote deferred; retry after 30 seconds.

## §3 — Cloud-IaC Terraform v-bump cascade tests

### TEST-J167-020 — Serial cascade applies to 4 cells in order with checkpoints

**Setup**: Cohort A permit sealed at 07:58:42 CDT.

**Action**: `cloud-iac.apply_terraform_cohort_a` workflow runs.

**Expected**:
- `apply_id` issued.
- Per-cell state transitions: `queued → applying → applied`, in order: CDMX → AUS-TX → QRO → GDL.
- Checkpoint every 30 seconds with progress %.
- Total apply duration `≤ 02:17 minutes`.
- Final state: all 4 cells `applied`.
- Per-cell audit seal `EVT-J167-COHORT-A-TF-APPLY-CELL-{cell-id}-003b`.
- Final aggregate seal `EVT-J167-COHORT-A-TF-APPLY-COMPLETE-003c`.

### TEST-J167-021 — Mid-cascade failure halts and reports

**Setup**: Inject Terraform-apply failure on AUS-TX cell (mock: simulated AWS API error on `aws_eks_cluster` resource).

**Action**: Run cascade.

**Expected**:
- CDMX cell completes successfully.
- AUS-TX cell enters `failed` state with error code `AWS_API_ERROR_THROTTLE`.
- QRO + GDL cells stay in `queued` state.
- `apply_status: "failed_after_partial"`, `rollback_on_failure: false` (configured), so the CDMX cell stays applied.
- Notification fires to Sofía + Diego + Yamilet.
- War-room auto-opens: `#cutover-v4-warroom`.

## §4 — Feature-flag canary traffic-split tests

### TEST-J167-030 — Traffic split applies at exact effective_at timestamp

**Setup**: Cohort A Terraform apply complete.

**Action**: `feature-flags.PUT /v1/flags/aurelia-fleet-coordinator-version/rules` with `effective_at: 2026-10-20T08:00:00-05:00`.

**Expected**:
- Rule applied within 18 ms of effective_at.
- First v4-routed request observed at 08:00:00.042 CDT.
- 1.0% ± 0.04% of traffic routes to v4 over the first 5 minutes (statistical assertion with 1.96σ confidence).
- Audit seal `EVT-J167-COHORT-A-LIVE-003`.

### TEST-J167-031 — Per-tenant override pins specific tenant to legacy version

**Setup**: Cohort B active. Cotrijal customer hotfix scenario.

**Action**: `feature-flags.PUT /v1/flags/aurelia-fleet-coordinator-version/rules/tenant-override` with `tenant: cotrijal-coop-rs-br`, `version: 3.x`, `expires_at: 2026-10-28T00:00:00Z`.

**Expected**:
- Cotrijal's traffic routes to v3.x exclusively (regardless of cohort percentage).
- All other tenants in Cohort B continue 10% v4 routing.
- Override expires at expiration timestamp; Cotrijal traffic auto-promotes back to cohort default.

## §5 — Observability SLO regression detection tests

### TEST-J167-040 — Canary-spike alarm fires after 5-min sustained breach

**Setup**: Cohort A active. Inject synthetic latency regression on `dispatch-cell-qro`: 84ms baseline → 312ms starting 13:48:42 CDT.

**Action**: Watch `observability` µservice's SLO regression detector.

**Expected**:
- At 13:53:42 CDT (5 min sustained): regression detected with severity `warning`.
- At 13:58:42 CDT (10 min sustained): regression escalates to `error`.
- At 14:00:18 CDT (~12 min sustained): SEV-2-candidate alarm fires; pages dispatched to Sofía + Yamilet + Diego.
- Audit seal `EVT-J167-CANARY-SPIKE-ALARM-004`.
- Auto-page latency from threshold-breach to page-delivered: ≤ 30 seconds.

### TEST-J167-041 — Bytecode pre-warm mitigation restores latency

**Setup**: Canary-spike alarm active. Pre-warm job kicked off at 14:07 CDT.

**Action**: Run pre-warm job.

**Expected**:
- Job duration: 8–14 minutes (11 min nominal).
- During job: cache miss-rate monitored; expected to drop from 18.4% → < 0.1% over 11 min.
- After job: p99 latency on QRO drops back to ≤ 92ms within 8 minutes of job completion.
- Audit seal `EVT-J167-MITIGATION-APPLIED-005`.
- Alarm auto-resolves when SLO returns to `green` state.

## §6 — Cohort B + C + D progression tests

### TEST-J167-050 — Cohort B vote with Cohort A SLO precondition green

**Setup**: Cohort A stable 24 hours. SLO precondition check returns `green`.

**Action**: Cohort B vote window opens. All 4 quorum members PERMIT.

**Expected**: Same shape as TEST-J167-010 but with `target_cohort: "cohort_b"`, `percentage: 10`, `cell_scope` = 12 cells. Seal `EVT-J167-COHORT-B-PERMIT-006`.

### TEST-J167-051 — Cohort B vote BLOCKED if Cohort A SLO not green

**Setup**: Cohort A active but injected sustained burn rate 1.4× (above 1.0× target).

**Action**: Attempt Cohort B vote.

**Expected**:
- HTTP 412 Precondition Failed
- `{"error": "previous_cohort_slo_not_green", "current_burn_rate": 1.4, "max_allowed": 1.0}`
- Cohort B initiation blocked until previous cohort returns to green.

### TEST-J167-052 — Cohort C 50% rollout with custom-policy compile failure

**Setup**: Cohort C active. Cotrijal's custom Cedar policy bundle uses deprecated v3 syntax (`principal.isInGroup()`).

**Action**: Compile Cotrijal bundle under Cedar v4 evaluator.

**Expected**:
- Compile fails with error `cedar_v4_deprecated_function: principal.isInGroup is removed in v4; use 'principal in Group'`.
- Auto-trigger per-tenant override (TEST-J167-031 scenario).
- Notification routed to Cotrijal's `cotrijal-it-admin` principal + Renata Castro (CPO) + Brian Tate (SVP-CS).
- Cohort C rollout continues for all other tenants.

## §7 — SEV-2 Saturday Oct 24 incident tests

### TEST-J167-060 — CRD-watch lag causes 3 stale pods serving v4 traffic

**Setup**: Cohort C active on AUS-TX cell. Inject CRD-watch lag: 3 pods receive a stale flag-bundle CRD version.

**Action**: Watch for SEV-2 alarm.

**Expected**:
- Error-budget burn rate on AUS-TX climbs to 4.8× over 90 minutes.
- SEV-2 alarm fires at burn-rate ≥ 4.0× sustained 30 min.
- War-room reopens; on-call pages dispatched.
- Audit seal `EVT-J167-SEV2-AUS-TX-008`.

### TEST-J167-061 — `kubectl rollout restart` mitigation works

**Setup**: SEV-2 active.

**Action**: Run `cloud-k8s.POST /v1/deployments/{id}/rollout-restart` on the 3 affected pods.

**Expected**:
- Each pod restarts in 8–15 minutes (graceful drain + reschedule).
- After all 3 pods complete restart: CRD version is current.
- Error-budget burn rate drops from 4.8× peak to 1.8× within 30 min of restart completion.
- Burn rate returns to ≤ 1.0× target within 8 hours.

### TEST-J167-062 — Rollback decision quorum 4-of-4 NO_ROLLBACK

**Setup**: SEV-2 mitigation in progress, burn rate dropping.

**Action**: Open rollback-decision vote. All 4 quorum members vote NO_ROLLBACK with rationale.

**Expected**:
- `governance.cohort_rollback` Cedar policy allows 3-of-4 quorum within 4-hour window.
- 4-of-4 NO_ROLLBACK satisfies; cohort C stays at 50%.
- Audit seal `EVT-J167-SEV2-AUS-TX-008` (single seal with all 4 votes attached).
- Follow-up workflow: post-mortem scheduled Monday 2026-10-26 14:00 CDT.

### TEST-J167-063 — Rollback Cedar BLOCKS vote after 4-hour window

**Setup**: SEV-2 active. Mock clock to 4h 12min after alarm.

**Action**: Diego attempts rollback vote.

**Expected**:
- HTTP 403
- `{"error": "cedar_policy_denied", "reason": "minutes_since_alarm_exceeded_240", "minutes_since_alarm": 252}`
- The decision must be re-classified as a new change-record (CHG-V4-ROLLBACK-...) with its own quorum.

## §8 — Cohort D 100% rollout tests

### TEST-J167-070 — Cohort D vote completes with all 47 cells targeted

**Setup**: Cohort C stable 96 hours.

**Action**: Cohort D vote.

**Expected**:
- 4-of-4 PERMIT.
- Audit seal `EVT-J167-COHORT-D-PERMIT-009`.
- Terraform apply cascades to 23 new cells (24 from Cohort C already done).
- Feature-flag flips to 100% v4 on all 47 cells.
- v3.x residual traffic drops to ≤ 0.4% within 90 min.

## §9 — V3.x hard sunset tests

### TEST-J167-080 — Sunset flips v3_api_enabled to false globally at exact UTC timestamp

**Setup**: Cohort D stable 72 hours. v3 residual traffic at 0.04%.

**Action**: Schedule sunset for `2026-10-30T23:59:00Z`.

**Expected**:
- At 23:59:00 UTC: `v3_api_enabled` flips to `false` globally.
- Legacy systems enter shutdown lifecycle:
  - `aurelia-fleetsync-v3-daemon` shutdown at 23:59:18Z (graceful drain)
  - `aurelia-gatewaybridge-v3-pods` shutdown at 23:59:42Z
  - `aurelia-contractadapter-v3-service` shutdown at 24:00:18Z
- All shutdowns report `graceful: true`.
- Audit seal `EVT-J167-V3-SUNSET-010` under TrueTime fence ≤ 10 ms.
- Merkle root for change record computed and dual-sealed.

### TEST-J167-081 — Late v3 request after sunset receives HTTP 410 Gone

**Setup**: Sunset complete at 23:59:00Z.

**Action**: Customer SDK on stale v3 endpoint POSTs at 24:00:42Z.

**Expected**:
- HTTP 410 Gone
- Body: `{"error": "v3_api_sunset", "sunset_at": "2026-10-30T23:59:00Z", "migration_guide_url": "https://docs.aurelia-robotics.com/v3-to-v4-migration"}`
- Telemetry tagged `v3_post_sunset_attempt`; Customer Success team paged for outreach.

## §10 — Compliance attestation tests

### TEST-J167-090 — ISO-27001-A.12.1.2 attestation packet generated for PwC

**Setup**: Change record CHG-V4-CUTOVER-2026-10-20 closed.

**Action**: Auto-trigger attestation generation.

**Expected**:
- Attestation bundle merkle root matches change-record root.
- Bundle includes: all 4 cohort permit seals + SEV-2 incident seal + sunset seal + CRA document + CAB sign-off.
- Submitted to `pwc-mexico-soc2-auditor-tenant` via cross-tenant attestation API.
- PwC acknowledgement expected within 7 business days (mocked).
- Audit seal `EVT-J167-COMPLIANCE-ISO-27001-ATTEST-011`.

### TEST-J167-091 — EU-AI-Act-Art-17 notification dispatched to DEKRA

**Setup**: Change record closed; AI-module-safety-interface-diff report attached.

**Action**: Auto-trigger EU-AI-Act notification.

**Expected**:
- Notification type: `qms_change_no_re_assessment_required` (because safety-relevant interfaces preserved).
- Submitted to `dekra-eu-ai-act-notified-body-tenant`.
- DEKRA acknowledgement expected within 14 business days (mocked).
- Audit seal `EVT-J167-COMPLIANCE-EU-AI-ACT-NOTIFY-012`.

### TEST-J167-092 — SOC2-CC8.1 evidence bundle assembled

**Setup**: Change record closed.

**Action**: Compose SOC2-CC8.1 evidence bundle.

**Expected**:
- Bundle includes: change-management workflow trace, CAB sign-off, CRA risk assessment, all permit votes, incident records, post-mortem document, rollback decision records, attestation chain.
- Bundle hash: SHA-384 hex.
- Submitted to PwC México.

## §11 — Cross-tenant audit-dual-seal invariant tests

### TEST-J167-100 — Every cohort permit dual-seals in 2 tenants

**Setup**: All 4 cohorts complete.

**Action**: Query both `aurelia-robotics-internacional-sa-de-cv-mx` AND `oya-governance-change-management-system-tenant` for the 4 cohort-permit audit seals.

**Expected**:
- Both tenants return the same 4 seals with byte-identical Merkle hashes.
- TrueTime uncertainty for each seal ≤ 10 ms.
- The seal IDs match: `EVT-J167-COHORT-A-PERMIT-002`, `-B-PERMIT-006`, `-C-PERMIT-007`, `-D-PERMIT-009`.

### TEST-J167-101 — Every SEV-level incident dual-seals

**Setup**: SEV-2 AUS-TX incident closed.

**Action**: Query both tenants for `EVT-J167-SEV2-AUS-TX-008`.

**Expected**: Byte-identical seal in both tenants.

## §12 — Performance + scale tests

### TEST-J167-110 — 47-cell Terraform cascade completes within 90 min for Cohort D

**Setup**: Cohort D apply triggered.

**Action**: Run full cascade.

**Expected**: All 47 cells applied within 90 minutes (~2 min per cell with parallelism = 4 concurrent applies).

### TEST-J167-111 — Cedar policy decision latency stays ≤ 5ms at 100k req/sec

**Setup**: Cohort D active. Load-test at 100,000 req/sec across 47 cells.

**Action**: Measure Cedar policy decision p99 latency.

**Expected**: p99 ≤ 5 ms; p99.9 ≤ 12 ms.

### TEST-J167-112 — Audit-chain dual-seal latency stays ≤ 10ms p99

**Setup**: Cohort A live + canary traffic.

**Action**: Measure dual-seal latency for 10,000 audit events.

**Expected**: p99 ≤ 10 ms.

## §13 — Failure / chaos tests

### TEST-J167-120 — Cell-failover during Cohort C apply

**Setup**: Cohort C apply in progress on cell 18 of 24.

**Action**: Inject cell-controller crash on cell 18.

**Expected**:
- Cell 18 enters `failed` state.
- Cascade halts; cells 19-24 stay `queued`.
- Auto-page to on-call (Sofía).
- Cell controller restarts within 30 seconds (K8s readiness probe re-elects).
- Cascade resumes from cell 18 with retry counter +1.
- Total cascade completes within 130% of nominal time.

### TEST-J167-121 — Cedar bytecode cache invalidation mid-cohort

**Setup**: Cohort B active. Inject cache invalidation event on all 12 cells simultaneously.

**Action**: Watch for SLO regression.

**Expected**:
- p99 latency climbs across all cells.
- SLO regression detector fires within 5 minutes.
- Auto-mitigation: bytecode pre-warm fires on all 12 cells in parallel.
- Latency restored within 12 minutes.
- No SEV-2 alarm (mitigation kicks in before 30-min sustained threshold).

## §14 — Acceptance criteria coverage

| AC | Tests |
|---|---|
| AC-J167-001 | TEST-J167-001 + TEST-J167-002 |
| AC-J167-002 | TEST-J167-010 + TEST-J167-012 + TEST-J167-013 |
| AC-J167-003 | TEST-J167-020 + TEST-J167-030 |
| AC-J167-004 | TEST-J167-040 |
| AC-J167-005 | TEST-J167-041 |
| AC-J167-006 | TEST-J167-050 + TEST-J167-051 |
| AC-J167-007 | TEST-J167-052 |
| AC-J167-008 | TEST-J167-060 + TEST-J167-061 + TEST-J167-062 |
| AC-J167-009 | TEST-J167-070 |
| AC-J167-010 | TEST-J167-080 + TEST-J167-081 |
| AC-J167-011 | TEST-J167-090 + TEST-J167-092 (service-credit MXN total assertion in test report) |
| AC-J167-012 | TEST-J167-100 + TEST-J167-101 |
| AC-J167-013 | TEST-J167-091 |
| AC-J167-014 | Coverage across §9 + §10 + §11 |

## §15 — Pass/fail thresholds for journey-level acceptance

- All TEST-J167-* tests pass.
- Total service-credit MXN-burn from incidents ≤ MXN 200,000 (the journey-level target).
- Cedar policy decision p99 ≤ 5 ms throughout cutover.
- Audit-chain dual-seal latency p99 ≤ 10 ms.
- TrueTime fence uncertainty ≤ 10 ms at every gate decision.
- No SEV-1 incidents.
- ≤ 2 SEV-2 incidents.
- ≤ 5 SEV-3 incidents.
- Total customer-impact-minutes ≤ 240 (cumulative across all incidents).
- 0 rollback events triggered.
- All compliance attestations (ISO-27001 + SOC2 + EU-AI-Act + MX-NOM-151) generated.
