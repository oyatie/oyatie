---
kanban_task: t_9b68c9a5
source_parent: t_cf995f91
title: QK-07 Abuse/Fraud/DDoS Dogfood Harness Evidence Producer Plan/Spec/RED
status: red-check-ready-runtime-evidence-pending
generated_at_utc: 2026-07-01T13:20:54Z
source_commit_at_authoring: c52bdb09ea33
claim_ceiling: target-backlog-schema-and-red-check-only; no measured abuse/fraud/DDoS result or green production_100 claim
---

# QK-07 Abuse/Fraud/DDoS Dogfood Harness Evidence Producer Plan/Spec/RED

## 0. Claim boundary

This artifact is the Plan/Spec/RED handoff for Kanban task `t_9b68c9a5`. It specifies the missing QK-07 dogfood harness/evidence producer and adds a fail-closed RED check. It does not implement abuse/fraud/DDoS runtime drills, does not emit a dogfood run receipt, and does not claim green security-control effectiveness, production readiness, public SLA/SLO, tenant workload readiness, hyperscaler maturity, or an external SaaS/public-cloud fallback.

The only claim this artifact makes now is: QK-07 has a source-cited future harness contract, scenario-to-output map, evidence record path, non-claim boundary, and a RED check that fails until a real dogfood receipt with abuse/fraud/DDoS drill results, ingress threshold provenance, and suspension/appeals round-trip evidence exists.

## 1. Source authority read for this card

- Target definition: `specs/cloud-production-quality-kits-target.json:133-151` defines `QK-07-abuse-fraud-ddos`, source `ADR-0297 abuse defence + ingress protection`, harness `abuse/fraud/DDoS scenario drills + ingress protection test`, scenarios `signup abuse`, `payment fraud`, `L3/4 + L7 DDoS`, and `resource-exhaustion abuse`, controls `signup/payment risk hooks`, `suspension + appeals workflow`, `ingress rate/protection (anycast + edge)`, and `customer-safe comms templates`, evidence `drill results; ingress protection thresholds; suspension/appeals round-trip`, and gate `production_100_bar.security_exit`.
- Backlog/schema row: `specs/cloud-production-quality-kit-evidence-backlog.json:1477-1687` keeps QK-07 `status=pending_implementation`, `runtime_status=not_implemented`, and `evidence_status=evidence_required`; it requires fields `kit_id`, `scenario_id`, `run_id`, `dogfood_environment`, `command`, `status`, `artifact_digest`, `reviewer`, `created_at`, `source_commit`, `evidence_window`, and `result_summary`.
- Official source evidence: `specs/cloud-production-quality-kit-evidence-backlog.json:1739-1745` binds objective domain `abuse_threat_scenarios` to `https://cheatsheetseries.owasp.org/cheatsheets/Bot_Management_and_Anti-Automation_Cheat_Sheet.html` with `claim_status=backlog_only_evidence_required`.
- Parent matrix: `/Users/jasonlee/.hermes/kanban/boards/oyatie/workspaces/t_0ca79b25/cloud-quality-slo-gap-matrix.md:39` records QK-07 as target/backlog only and missing implemented dogfood harness, dated dogfood run receipt, artifact digest/source commit/reviewer, measured outputs, and green production_100 evidence.
- De-dupe source: `/Users/jasonlee/.hermes/kanban/boards/oyatie/workspaces/t_cf995f91/cloud-quality-kit-dedupe-plan-spec-red-map.md:110-121` records this child, command, future evidence path, fields, digest fields, outputs, and non-claim language.
- Validator baseline: `python3 scripts/tests/cloud_production_quality_kit_evidence_backlog_check.py` remains the backlog/schema validator for the quality-kit target/backlog rows.

## 2. Future harness command and evidence path

Exact future command to preserve:

`python3 scripts/tests/qk_07_abuse_fraud_ddos_future_harness_check.py --dogfood-environment oyatie-dogfood-cell --emit-evidence evidence/cloud/quality-kits/qk-07-abuse-fraud-ddos/runs/<run_id>.json`

Future evidence path:

`evidence/cloud/quality-kits/qk-07-abuse-fraud-ddos/runs/<run_id>.json`

Dogfood environment record value:

`oyatie-dogfood-cell`

Forbidden fallbacks:

- no external SaaS runner;
- no GitHub Actions runner as runtime fallback;
- no public-cloud-provider runtime fallback;
- no static source text, Markdown plan, backlog row, TrustCenter publication, incident/status workflow, or target spec may stand in for a run receipt.

## 3. Scenario-to-evidence binding

| Scenario ID | Source scenario | Required future dogfood input/provenance | Required output families | RED/non-claim posture |
| --- | --- | --- | --- | --- |
| `QK-07-abuse-fraud-ddos-S01` | `signup abuse` | Dogfood-cell signup abuse drill receipt tied to signup/payment risk hook configuration, request corpus digest, risk decision trace, suspension decision receipt, and evidence window. | `abuse_drill_results`, `suspension_appeals_round_trip` | Current artifact defines the required run shape only. No signup-abuse control effectiveness or suspension correctness exists until a real dogfood receipt is emitted. |
| `QK-07-abuse-fraud-ddos-S02` | `payment fraud` | Dogfood-cell payment fraud drill receipt tied to payment risk hook inputs, fraud-decision trace, resulting suspension/hold action, appeals path receipt, and artifact digest. | `abuse_drill_results`, `suspension_appeals_round_trip` | Current artifact does not prove fraud detection or appeals safety. Static source text or TrustCenter prose is rejected. |
| `QK-07-abuse-fraud-ddos-S03` | `L3/4 + L7 DDoS` | Dogfood-cell ingress threshold report and DDoS drill receipt covering L3/4 and L7 scenarios, threshold configuration digest, observed ingress behavior, and evidence window. | `ingress_threshold_report`, `abuse_drill_results` | Current artifact does not prove ingress protection thresholds, anycast/edge behavior, or public-cloud maturity. |
| `QK-07-abuse-fraud-ddos-S04` | `resource-exhaustion abuse` | Dogfood-cell resource-exhaustion drill receipt tied to abuse workload profile, rate/protection decision trace, suspension action, appeals round-trip receipt, and artifact digest. | `abuse_drill_results`, `ingress_threshold_report`, `suspension_appeals_round_trip` where applicable | Current artifact does not prove resource-exhaustion containment. Fabricated `passed_after_future_runtime_evidence` is rejected. |

A future single run receipt may cover all four scenarios if it includes a `scenario_results` map for every scenario ID above. If future implementers choose one receipt per scenario, each receipt must still preserve the command/evidence path family and must link to a rollup receipt before QK-07 can feed `production_100_bar.security_exit`.

## 4. Evidence record contract

The future JSON receipt must include all backlog-required fields:

- `kit_id`
- `scenario_id`
- `run_id`
- `dogfood_environment`
- `command`
- `status`
- `artifact_digest`
- `reviewer`
- `created_at`
- `source_commit`
- `evidence_window`
- `result_summary`

Digest fields that must be present and non-placeholder:

- `source_commit`
- `command`
- `dogfood_environment`
- `artifact_digest`

QK-07 output keys that must be present in `result_summary` before any future implementation may attempt a positive status:

- `abuse_drill_results`
- `ingress_threshold_report`
- `suspension_appeals_round_trip`

Additional QK-07 provenance requirement: the receipt must include an `abuse_fraud_ddos_provenance` object with concrete evidence references for signup abuse, payment fraud, L3/4 + L7 DDoS, resource-exhaustion, ingress thresholds, suspension decision, and appeals round-trip. These references must point under `evidence/cloud/quality-kits/qk-07-abuse-fraud-ddos/`; static source files under `specs/`, `plan/`, `docs/`, `templates/`, or `registry/` do not satisfy this evidence gate.

## 5. RED/fail-closed check added by this card

Added check path:

`scripts/tests/qk_07_abuse_fraud_ddos_future_harness_check.py`

RED behavior:

1. Rejects the wrong dogfood environment; only `oyatie-dogfood-cell` is valid for this card.
2. Rejects `--emit-evidence` paths outside `evidence/cloud/quality-kits/qk-07-abuse-fraud-ddos/runs/` or placeholder run IDs.
3. Rejects missing dogfood run receipts.
4. Rejects static source text or source-only artifacts as evidence, including `specs/`, `plan/`, `docs/`, `templates/`, and `registry/` references in receipt artifact refs.
5. Rejects missing required evidence fields and missing digest fields.
6. Rejects missing abuse/fraud/DDoS scenario drill receipts and missing `scenario_results` coverage for the four QK-07 scenarios.
7. Rejects missing ingress threshold, suspension decision, or appeals round-trip provenance.
8. Rejects missing QK-07 output keys.
9. Rejects `status=passed_after_future_runtime_evidence` as fabricated while this RED-only checker is in place. A future Build card must replace/extend the checker with actual dogfood runtime verification before any positive status can pass.

Intentional RED command for this Plan/Spec card:

`python3 scripts/tests/qk_07_abuse_fraud_ddos_future_harness_check.py --dogfood-environment oyatie-dogfood-cell --emit-evidence evidence/cloud/quality-kits/qk-07-abuse-fraud-ddos/runs/red-check-missing-receipt.json`

Expected result now: non-zero exit with a missing dogfood run receipt message. That failure is the correct Plan/Spec/RED proof because no runtime dogfood receipt exists yet.

Self-test command for the checker shape:

`python3 scripts/tests/qk_07_abuse_fraud_ddos_future_harness_check.py --self-test`

Expected result now: zero exit after exercising rejection of static source text, missing ingress/suspension/appeals provenance, missing digest fields, missing scenario drill receipts, forbidden fallback markers, and fabricated `passed_after_future_runtime_evidence`.

## 6. No-action overlap rationale

- TrustCenter (`t_157e833c`, `t_3af64a26`, `t_3a144f8c`, `t_c9fba41f`, `t_e615a913`): no action here. Those surfaces may later publish or ingest customer-safe summaries; they do not produce QK-07 abuse/fraud/DDoS dogfood drill receipts.
- SREOPS/status/incident workflow (`t_9d403ad1`, `t_cc1cb9cd`): no action here. Those cards cover operations/status/incident contracts and communications, not ingress threshold drills or suspension/appeals round-trip evidence.
- Security/compliance authority gates (`t_77bc98f3`): no action here. That lane preserves vulnerability/SBOM/SRE/root-of-trust/PCI/sovereign gates; it does not own abuse/fraud/DDoS runtime drills for QK-07.
- Managed-k8s/ingress/root-trust lanes (`t_e14c021c` and related root-of-trust/security cards): no action here. Those lanes may own lifecycle/control-plane/authority/ingress or trust surfaces; this card owns only the QK-07 Plan/Spec/RED evidence-producer wrapper and fail-closed receipt contract.
- Broad security controls: no action here. This card must not roll out WAF/ingress/provider controls, fraud services, suspension systems, or product/control-plane surfaces.

## 7. Future Build card boundaries

Allowed future implementation path class:

- `scripts/tests/qk_07_abuse_fraud_ddos_future_harness_check.py`
- `evidence/cloud/quality-kits/qk-07-abuse-fraud-ddos/`
- ingress/security/fraud dogfood fixtures only when a Build card explicitly owns them and serializes with competing lanes
- common incident/status evidence paths only when explicitly serialized with SREOPS/status owners

Forbidden in this Plan/Spec/RED card:

- generated JSON edits;
- root-hub pointer edits;
- live ingress/provider/network state edits;
- secrets or root-trust material edits;
- broad product/control-plane surface edits;
- public-cloud or external-SaaS fallback implementation;
- green runtime evidence, security-control effectiveness, production readiness, public SLA/SLO, tenant workload readiness, or hyperscaler maturity claims.

## 8. Verification commands for this card

Run and record:

1. `python3 scripts/tests/cloud_production_quality_kit_evidence_backlog_check.py`
   - Expected now: pass for the existing backlog/schema.
2. `python3 scripts/tests/qk_07_abuse_fraud_ddos_future_harness_check.py --self-test`
   - Expected now: pass the validator self-tests that prove fail-closed rejection cases.
3. `python3 scripts/tests/qk_07_abuse_fraud_ddos_future_harness_check.py --dogfood-environment oyatie-dogfood-cell --emit-evidence evidence/cloud/quality-kits/qk-07-abuse-fraud-ddos/runs/red-check-missing-receipt.json`
   - Expected now: fail closed because the dogfood run receipt is absent.

Closeout condition for `t_9b68c9a5`: this artifact plus the RED check exist, the backlog validator still passes, the QK-07 check self-tests pass, the QK-07 command fails closed for the absent receipt, and the Kanban closeout explicitly states that the future evidence path remains pending until a real dogfood run receipt exists.
