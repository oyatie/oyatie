# Performance Management

| Field | Value |
|---|---|
| Microservice | `performance-management` |
| Status | wave-4-rolling-remediation-2026-05-21 |
| Big-8 family | HR/Payroll (P0) |
| Audience | tenant-b2b-hr, segment b2b-leader |
| Counterparts | Lattice, 15Five, Workday Performance (primary); Culture Amp, Glint (engagement adjacency) |
| Binding ADRs | ADR-0105, ADR-0131, ADR-0132, ADR-0244, ADR-0245, ADR-0248, ADR-0314, ADR-0315, ADR-0316, ADR-0321, ADR-0328, ADR-0329, ADR-0330, ADR-0331 |
| Tenant classes | `demo_trial` (read-only redacted), `paid` (full surface) |
| Billing component | `bc-performance-management` |
| Cell tiers | T1, T2 (per ADR-0248) |

## 1. Scope and non-goals

The `performance-management` microservice is the **operational concern owner** for employee
goal management, performance review evidence, talent calibration, engagement signal, manager
tooling, and recognition. It is one flat microservice under `microservices/performance-management/`
with `src/` as the canonical code root per ADR-0131.

**In scope** (12 bounded contexts; each owns its own command/event/projection surface):

1. `goal-cycle` — OKR authoring, cascade across the org tree, status updates, period roll-forward.
2. `review-cycle` — annual, semi-annual, project-anytime, and probationary review forms with
   evidence sealing per IP-027.
3. `feedback` — request/give, anytime, 360-degree, manager note, peer praise.
4. `engagement-survey` — pulse and full-cycle surveys, eNPS, anonymity guard per IP-029.
5. `calibration` — talent calibration sessions, force-distribution checks, nine-box grid,
   calibration ledger.
6. `one-on-one-cadence` — manager/direct-report agendas, action items, talking points.
7. `succession-planning` — talent cards, successor lists, readiness rating, ready-now/in-N-years.
8. `recognition` — public praise, kudos, peer recognition, recognition wall.
9. `weekly-check-in` — weekly priorities, blockers, mood, manager rollup.
10. `talent-management` — high-potential identification, performance-potential matrix.
11. `analytics-reporting` — manager dashboards, HRBP analytics, sentiment, trend lines.
12. `manager-tooling` — review-form drafting, performance-summary generation, 1:1 prep packets.

**Out of scope** (forwarded to siblings):

- Compensation merit-increase math → `compensation` µservice consumes `RatingFinalizedEvent`.
- Payroll posting → `payroll` µservice (no direct edge; mediated by `compensation`).
- HRIS system-of-record → `people-records` µservice (org tree, employment status).
- Learning course completion → `learning-management` µservice (we consume completion events).
- Time-off / leave accrual → `time-tracking` µservice (we consume for cycle proration).
- Recruiting requisitions → `recruiting` µservice (we consume new-hire events for 30/60/90).
- Workforce planning headcount → `workforce-planning` µservice (we produce talent-card events).

## 2. Principals and tenant scope

Every request, event, and projection carries a `tenant_id` (UUID v7) and `principal_id`. ADR-0244
makes tenant scoping the universal primitive: no row, audit record, or cost report is allowed
to exist without tenant context.

Principal classes recognized by Cedar gates (`policies/*.cedar`):

- `User::"employee"` — self-reads goals, self-submits reviews/feedback.
- `User::"manager"` — reads reports' goals, drafts review-cycle forms, leads 1:1s.
- `User::"hrbp"` — runs calibration, releases engagement summaries, exports redacted evidence.
- `User::"executive"` — reads org-rolled-up calibration outputs, succession boards.
- `User::"auditor"` — read-only across audit trail (requires `ticket_id`).
- `User::"talent-reviewer"` — calibration-only attendance.
- `ServiceAccount::"foundry.oyatie.<role>"` — substrate-side principals per ADR-0247.
- `External::"compensation-microservice"`, `External::"workforce-planning-microservice"`,
  `External::"learning-management-microservice"`, `External::"time-tracking-microservice"`,
  `External::"recruiting-microservice"`, `External::"people-records-microservice"` —
  sibling-microservice service accounts authorized for specific cross-handoff actions only.

## 3. Cedar gates

Per ADR-0243 every authorization decision is a Cedar evaluation; no policy in code.

| Policy file | Gate intent |
|---|---|
| `policies/local-review-cycle-scope.cedar` | review form access by org-tree position |
| `policies/local-goal-alignment-approval.cedar` | manager approval of cascade |
| `policies/local-rating-change-guard.cedar` | who may change a finalized rating + breakglass |
| `policies/local-calibration-lock-control.cedar` | calibration-session lock owner |
| `policies/local-feedback-visibility.cedar` | feedback visibility by domain + data class |
| `policies/local-hr-export-egress.cedar` | redacted export egress (HR Business Partner only) |
| `policies/local-engagement-pulse-anonymity.cedar` | engagement anonymity floor (k≥8 default) |

All policies share the default-deny posture documented in IP-002 and branch on
`context.tenant_class ∈ {demo_trial, paid}` per ADR-0331. Demo tenants are denied real-PII
egress; paid tenants under packs `hipaa` or `eu-worker-council` get additional restrictions
(see `compliance.md` for the pack-by-pack matrix).

## 4. Data model walkthrough

The five legacy bounded contexts (goal-cycle, review-cycle, feedback, engagement-survey,
calibration) are joined by seven new contexts in this remediation wave. The full data model
lives in `ARCHITECTURE.md` §3 and the per-table column lists are in `IP-001..IP-037` and the
`capabilities/*.yaml` records. Headline tables:

- `performance_goal` (goal-cycle), `performance_goal_alignment` (cascade edges).
- `performance_review_cycle`, `performance_review_form`, `performance_review_evidence_seal`.
- `performance_feedback_entry`, `performance_feedback_request`, `performance_feedback_360`.
- `performance_engagement_pulse`, `performance_engagement_release`.
- `performance_calibration_session`, `performance_calibration_bucket`, `performance_nine_box_cell`.
- `performance_one_on_one` (agenda, action items), `performance_check_in_weekly`.
- `performance_succession_talent_card`, `performance_succession_readiness`.
- `performance_recognition_post` (kudos), `performance_recognition_reaction`.

Every row carries `tenant_id`, `principal_id_owner`, `created_at`, `updated_at`, and an
append-only audit ref `audit_event_id`. Soft-delete via `deleted_at` column; hard-delete only
during regulator-driven erasure window per `compliance.md` §10.

## 5. Workflow and replay semantics

ADR-0263 declares the workflow-engine as the canonical substrate. Performance Management
publishes the following long-lived workflows (templates live in `IP-004`):

- `goal-cycle.open → cascade → quarterly-check-in → close → roll-forward`.
- `review-cycle.kickoff → draft → peer-feedback → manager-review → calibration → seal → publish`.
- `engagement-pulse.send → collect → anonymity-check → release-aggregate`.
- `calibration.schedule → load-cohort → session → lock → emit-outcomes → publish`.
- `succession.identify → talent-card-author → executive-review → publish-to-workforce-planning`.
- `weekly-check-in.send-reminder → collect → manager-rollup`.

Every workflow step emits an AsyncAPI 3.1.0 envelope (see `contracts/asyncapi-v1.yaml`). Replay
is supported via `IP-016-backfill-replay-worker.md` deterministic event ID generation.

## 6. Contracts and versioning

Three contract surfaces; all under `contracts/`:

- `contracts/openapi-v1.yaml` — REST commands (OpenAPI 3.2.0).
- `contracts/asyncapi-v1.yaml` — outbound events (AsyncAPI 3.1.0).
- `contracts/performance-management-v1.proto` — internal gRPC (proto3, HTTP/3 transport).

Cross-microservice handoff contracts ship as separate AsyncAPI documents (one per sibling edge):

- `contracts/hr-handoff-compensation.asyncapi.yaml` (B-1 rating-finalized → merit-increase).
- `contracts/hr-handoff-people-records.asyncapi.yaml` (B-2/B-3 calibration outcome + org tree).
- `contracts/hr-handoff-learning-management.asyncapi.yaml` (B-5 learning completion consumption).
- `contracts/hr-handoff-time-tracking.asyncapi.yaml` (B-6 time-off proration).
- `contracts/hr-handoff-workforce-planning.asyncapi.yaml` (B-7 talent-card → headcount cost).
- `contracts/hr-handoff-recruiting.asyncapi.yaml` (B-9 new-hire 30/60/90 trigger).

Versioning policy per ADR-0322: backwards-compatible additions bump minor; breaking changes
bump major + 90-day deprecation per ADR-0330.

## 7. Transport and cryptography

HTTP/3 + QUIC is the default per ADR-0253-amendment. The transport profile string is
`h3-h2-h1-strict-tls13-ech-pqc`:

- HTTP/3 over QUIC primary.
- HTTP/2 over TLS 1.3 fallback.
- HTTP/1.1 strict-TLS-1.3 emergency fallback.
- TLS 1.3 with `ECH` (Encrypted Client Hello) per `iac/ech-config.yaml`.
- Post-quantum cert hybrid per `iac/pqc-cert.yaml` (X25519+Kyber768 KEM).

gRPC runs over HTTP/3 per ADR-0253. mTLS between microservices via OpenBao-issued certs
(`iac/openbao-policy.yaml`).

## 8. Abuse defence and emergency bypass

`iac/edge-waf.yaml` declares the WAF posture: ratelimits at the cell ingress (per ADR-0248);
goal-cycle write at 30/min/principal, calibration at 5 sessions/min/cell, engagement-pulse
release at 60/min/cell.

Emergency bypass:

- `rating.change.breakglass` (Cedar action) requires `breakglass_ticket_id` +
  `approval_chain = security-plus-service-owner`. Emits an audit event of class
  `PerformanceManagementFeedbackVisibilityBreakglass`. See `runbooks/local-cycle-reopen-breakglass.md`.
- `engagement-pulse.release-with-suppression` requires `aggregate_only=true` and hrbp role.

## 9. Marketplace settlement

ADR-0314 ties paid actions to DealSet settlement. The `engagement-pulse-summary-released`,
`succession-talent-card-published`, `review-cycle-sealed`, and `calibration-session-locked`
events carry `deal_set_id` for billing-component `bc-performance-management`. Demo-trial
tenants never trigger settlement (Cedar gate `local-hr-export-egress.cedar` short-circuits).

## 10. Observability

Per ADR-0130 every promotion beyond `dev` requires SLOs to be authored. See `slos/*.openslo.yaml`
for the twelve OpenSLO objects (one per bounded context + engagement-pulse anonymity gate).
Dashboards under `dashboards/*.json` (Grafana 11.x layout). Traces and metrics emit to the
observability substrate via OTLP per IP-011.

Key SLOs (paid tenant):

- Review form open: p99 ≤ 300ms.
- Goal cascade apply: p99 ≤ 800ms.
- Engagement pulse release: p99 ≤ 1.5s.
- Calibration outcome publish: p99 ≤ 2s.

Demo-trial tenants accept a 5x looser p99.

## 11. Capacity

`capacity-model.md` projects the worst-case load for a 50,000-employee tenant during annual
review cycle close (peak): 8 million review-form opens over a 14-day window, 200,000
calibration-bucket writes, 1.2 million feedback entries. Cell sizing per ADR-0248 places the
service in cell tier T1 with shuffle-sharded capacity.

## 12. Failure modes

`failure-modes.md` documents the eleven scenario classes. Headline operations recovery:

- Review-form latency burn → `runbooks/local-review-form-latency-burn.md`.
- Calibration deadlock → `runbooks/calibration-deadlock.md` (lock arbitration).
- Engagement-pulse anonymity floor breach → `runbooks/engagement-pulse-privacy-hold.md`.
- Manager-feedback abuse → `runbooks/manager-feedback-abuse-report.md`.
- Review evidence seal failure → `runbooks/review-evidence-seal-failure.md`.

## 13. Regional packs

Localization is a pack overlay per the canonical-base+localization doctrine. Packs registered
in `compliance.md`:

- `soc2`, `iso27001` — global control mapping.
- `gdpr` — EU data minimization, right-to-erasure, DSAR support.
- `kr-pipa` — Korea PIPA controls (data-subject consent, kept apart from PII).
- `hipaa` — applies only if tenant enables healthcare workforce mode.
- `eu-worker-council` — works-council notification before review-cycle launch; calibration
  outcomes require council consultation.
- `us-labor` — Title VII fairness check on calibration distributions, EEOC reporting hooks.

## 14. Multi-context deployment

Per ADR-0329 + audit Finding 6.1.A the service ships in six deployment contexts. Each context
has its own OpenTofu module under `iac/<context>/`:

- `iac/oyatie-public-cloud/` — Oyatie-hosted multi-tenant.
- `iac/guest-on-aws/` — customer AWS VPC, Oyatie operates.
- `iac/oci-guest/` — customer OCI tenancy; sub-module `always-free/` for demo_trial workloads.
- `iac/on-prem/` — customer datacenter (Talos/RHEL/SUSE).
- `iac/colo/` — colo-hosted single-tenant.
- `iac/oyatie-iaas/` — Oyatie as cloud provider; built on Cloud Hypervisor + Kata pods.

## 15. Acceptance evidence

Promotion past `dev` requires:

1. Zero P0 findings against `coherence-audit-2026-05-20.md` (audit re-run mandatory).
2. SLO objects authored at `slos/*.openslo.yaml` (twelve objects required).
3. ≥85% counterpart coverage per `feature-parity-matrix-2026-05-20.md`.
4. All seven HR-family cross-handoff contracts under `contracts/hr-handoff-*.asyncapi.yaml`.
5. Cedar policy `local-engagement-pulse-anonymity.cedar` present and tested.
6. `supported_oses.json` covering all thirteen OS targets.
7. CI lane per-OS green; arch matrix (linux/amd64, linux/arm64, darwin/arm64) green.

## 16. References

- Audit: `coherence-audit-2026-05-20.md`
- Parity: `feature-parity-matrix-2026-05-20.md`
- Remediation log: `REMEDIATION-NOTES-2026-05-21.md`
- PRD: `PRD.md`
- Architecture: `ARCHITECTURE.md`
- Compliance: `compliance.md`
- Implementation Plans: `IP-001` … `IP-037`
- Sibling audits to consume this service's outbound edges: `compensation`, `people-records`,
  `learning-management`, `time-tracking`, `workforce-planning`, `recruiting`.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
