---
doc_class: ImplementationPlan
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0292]
acceptance_status: draft
companion_docs: [microservices/social/policy/minor-protection.cedar]
inbound_citations: [microservices/social/manifest.json]
---

# IP-016: Minor-protection strict defaults

## A. Problem
Minor-facing social defaults must be strict before feed ranking, DMs, recommendations, federation, and ads-like surfaces can safely operate.

## B. Approach
Bind ADR-0292 behavior to `policy/minor-protection.cedar`, age verification, notification muting, DM restrictions, feed/ranking controls, guardian visibility, and abuse heuristics. Fail closed when age or guardian state is ambiguous.

## C. Deliverables
| Artifact | Role |
|---|---|
| `policy/minor-protection.cedar` | Strict-default policy. |
| `catalog/oya-social-sock-puppet-detector-kernel.yaml` | Existing abuse/minor-adjacent detector anchor. |
| `slos/minor-protection-engagement-correctness.openslo.yaml` | Minor-protection correctness SLO. |
| `dashboards/minor-protection-health.json` | Operating dashboard. |
| `runbooks/social-bot-score-recalibration.md` | False-positive tuning runbook. |

## D. Ordered implementation steps
1. Enforce COPPA refusal for under-13 standalone account provisioning.
2. Apply KOSA 14-17 defaults for DMs, ranking, ads, notifications, discoverability, livestream, and purchase gates.
3. Bind age-verification state from IP-013.
4. Add guardian dashboard read model with jurisdiction-aware limits.
5. Emit anti-grooming and suspicious-contact audit events.
6. Add precision/false-positive tests and appeal routing.
7. Wire SLO, dashboard, and runbook evidence.

## E. Acceptance
- `policy/minor-protection.cedar` tests pass for under-13, 14-17, adult, guardian, and unknown-age cases.
- `slos/minor-protection-engagement-correctness.openslo.yaml` resolves.
- `dashboards/minor-protection-health.json` validates as JSON.
- `cargo run -p oya-dev-cli -- gate validate minor-protection --microservice social` passes.
- Age and DM controls interlock with `policy/dm-scope.cedar`.

## F. Evidence
- Policy: `policy/minor-protection.cedar`, `policy/dm-scope.cedar`.
- SLO/dashboard: `slos/minor-protection-engagement-correctness.openslo.yaml`, `dashboards/minor-protection-health.json`.
- Runbooks: `runbooks/social-bot-score-recalibration.md`, `runbooks/sock-puppet-cluster-takedown.md`.

## G. Counterpart comparison
Instagram Teen Accounts, TikTok Restricted Mode, Snapchat Family Center, and LinkedIn youth/professional identity controls are the real counterparts. Oyatie's target is stricter default behavior with Cedar evidence and auditable appeal paths.

## H. Foundation delivery expansion
- Deliverable detail: strict defaults cover DMs, ranking, ads, notifications, discoverability, livestream, purchases, and contact suggestions.
- Deliverable detail: age state from IP-013 is consumed as a policy input, not duplicated.
- Deliverable detail: guardian read model exposes allowed settings and audit history without private post leakage.
- Deliverable detail: anti-grooming signals emit audit events with appeal-safe evidence.
- Deliverable detail: false-positive handling routes through documented review and rollback paths.
- Deliverable detail: dashboards separate protected-user safety, false positives, appeals, and enforcement lag.
- Deliverable detail: jurisdiction overlays distinguish COPPA, KOSA, EU, KR, and default behavior.
- Deliverable detail: Slack youth/community workspace moderation is counterpart pressure for conservative contact defaults.

## I. Acceptance expansion
- Acceptance detail: under-13 tests must refuse standalone account provisioning where policy requires it.
- Acceptance detail: 14-17 tests must apply strict defaults without adult-level discovery.
- Acceptance detail: unknown-age tests must choose conservative defaults.
- Acceptance detail: guardian tests must expose only policy-allowed controls.
- Acceptance detail: DM-scope tests must prove protected users cannot receive unsafe contact.
- Acceptance detail: dashboard JSON must validate for minor-protection health.
- Acceptance detail: SLO resolution must include engagement correctness or document missing measured evidence.
- Acceptance detail: Slack, Instagram, TikTok, Snapchat, and LinkedIn comparisons must map to safety defaults and auditability.

## J. Evidence expansion
- Evidence detail: capture Cedar tests for `policy/minor-protection.cedar`.
- Evidence detail: capture minor-protection gate output for social.
- Evidence detail: capture dashboard JSON validation for `minor-protection-health.json`.
- Evidence detail: cite `policy/dm-scope.cedar` for contact controls.
- Evidence detail: cite `slos/minor-protection-engagement-correctness.openslo.yaml`.
- Evidence detail: cite recalibration and sock-puppet runbooks if present, or record them as required gaps.
- Evidence detail: cite Slack as community moderation pressure for strict contact defaults.
- Evidence detail: include the fixture matrix count for under-13, 14-17, adult, guardian, and unknown-age cases in the evidence bundle.
