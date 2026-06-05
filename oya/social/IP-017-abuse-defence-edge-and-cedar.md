---
doc_class: ImplementationPlan
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0293]
acceptance_status: draft
companion_docs: [microservices/social/policy/abuse-defence.cedar, microservices/social/iac/edge-waf.yaml]
inbound_citations: [microservices/social/manifest.json]
---

# IP-017: Abuse defence edge and Cedar

## A. Problem
Social abuse controls must stop bot, spam, impersonation, scraping, coordinated inauthentic behavior, and harassment without adding friction to clean users or bypassing Cedar.

## B. Approach
Combine edge WAF signals, Cedar decisions, sock-puppet detection, moderation queues, and audit-chain evidence. Edge controls shape traffic; application Cedar remains the source of authorization truth.

## C. Deliverables
| Artifact | Role |
|---|---|
| `iac/edge-waf.yaml` | Edge abuse filtering anchor. |
| `policy/abuse-defence.cedar` | Application abuse policy. |
| `catalog/oya-community-social-sock-puppet-detector-kernel.yaml` | Existing detector catalog anchor. |
| `dashboards/abuse-defence-outcomes.json` | Abuse outcomes dashboard. |
| `runbooks/coordinated-inauthentic-behavior-response.md` and `runbooks/sock-puppet-cluster-takedown.md` | Incident closure. |

## D. Ordered implementation steps
1. Define edge signal taxonomy and normalization into application risk signals.
2. Ensure WAF allow/deny outcomes are audited and explainable.
3. Evaluate Cedar before sensitive actions even when edge allows traffic.
4. Add sock-puppet cluster detection and appeal-safe evidence.
5. Test false positives, rate-limit bypass, bot storms, and clean-user low-friction paths.
6. Wire dashboards and incident runbooks.
7. Add rollback for overbroad WAF or Cedar fragments.

## E. Acceptance
- `policy/abuse-defence.cedar` tests pass.
- `iac/edge-waf.yaml` parses and references documented signal names.
- `dashboards/abuse-defence-outcomes.json` validates as JSON.
- `buck2 build //:quality-lane-registry-authority-check # lane=abuse-defence --microservice social` passes.
- Runbooks include triggers, rollback, and evidence export.

## F. Evidence
- Policy/IaC: `policy/abuse-defence.cedar`, `iac/edge-waf.yaml`.
- Catalog: `catalog/oya-community-social-sock-puppet-detector-kernel.yaml`.
- Runbooks: `runbooks/coordinated-inauthentic-behavior-response.md`, `runbooks/sock-puppet-cluster-takedown.md`, `runbooks/abuse-report-backlog-drain.md`.

## G. Counterpart comparison
X, TikTok, Instagram, Snapchat, Reddit, Mastodon, and Bluesky all fight abuse and automated manipulation. Oyatie's counterpart distinction is Cedar-audited enforcement and reversible edge controls rather than opaque trust-and-safety actions.

## H. Foundation delivery expansion
- Deliverable detail: edge signals include rate, reputation, device, ASN, challenge, bot, and federation risk inputs.
- Deliverable detail: WAF outcomes normalize into allow, challenge, throttle, deny, and observe states.
- Deliverable detail: Cedar evaluates sensitive actions after edge allow decisions.
- Deliverable detail: sock-puppet detector records cluster id, evidence basis, confidence, and appeal state.
- Deliverable detail: rollback can disable overbroad WAF fragments or Cedar rules independently.
- Deliverable detail: false-positive metrics feed recalibration and operator review.
- Deliverable detail: dashboards separate edge blocks, app denials, appeals, and confirmed abuse.
- Deliverable detail: Slack workspace spam and community abuse controls are direct counterpart pressure.

## I. Acceptance expansion
- Acceptance detail: `edge-waf.yaml` must parse and reference documented signal names.
- Acceptance detail: Cedar abuse fixtures must cover bot storm, spam, harassment, sock-puppet, and clean-user paths.
- Acceptance detail: bypass tests must prove edge allow does not skip app policy.
- Acceptance detail: false-positive tests must preserve appeal and rollback evidence.
- Acceptance detail: dashboard JSON must validate for abuse outcome metrics.
- Acceptance detail: runbooks must include trigger, containment, rollback, and evidence export.
- Acceptance detail: federation egress policy must remain compatible with abuse controls.
- Acceptance detail: Slack, Reddit, X, TikTok, and Mastodon comparisons must map to abuse-defense evidence.

## J. Evidence expansion
- Evidence detail: capture parser output for `iac/edge-waf.yaml`.
- Evidence detail: capture Cedar tests for `policy/abuse-defence.cedar`.
- Evidence detail: capture abuse-defence gate output for social.
- Evidence detail: cite `catalog/oya-community-social-sock-puppet-detector-kernel.yaml`.
- Evidence detail: cite `dashboards/abuse-defence-outcomes.json`.
- Evidence detail: cite coordinated-behavior and sock-puppet runbooks if present.
- Evidence detail: cite Slack as community/channel abuse-defense pressure alongside Reddit and X.
- Evidence detail: include the abuse-signal taxonomy revision and WAF fragment hash in the promotion evidence bundle.
