---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-015-hg-social-registration-and-branch-protection
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social + council-architecture + ops-platform
acceptance_lanes: [hyperscaler-gate-registration, branch-protection, governance-required-checks]
---

# IP-015: HG-SOCIAL registration and branch protection

## A. Problem
HG-SOCIAL evidence can regress unless manifest, contracts, policy, SLO, parity, and runbook checks are required at branch-protection time.

## B. Approach
Register the social hyperscaler gate as a required check over repo-local evidence. Promotion blocks on missing contracts, policy parse failures, OpenSLO drift, unresolved catalog paths, or unsupported counterpart claims.

## C. Deliverables
| Artifact | Role |
|---|---|
| `manifest.json` | Service identity and inventory source. |
| `scorecards/overrides.json` | Local scorecard and exception surface. |
| `coherence-audit-2026-05-20.md` | Known-gaps register. |
| `competitor-parity-matrix.md` and `feature-parity-matrix-2026-05-20.md` | Claim-boundary sources. |
| `REMEDIATION-NOTES-2026-05-21-tier-scrub.md` | Updated by this wave. |

## D. Ordered implementation steps
1. Parse manifest and scorecard JSON.
2. Resolve every catalog, contract, policy, SLO, dashboard, and runbook path.
3. Register contract, policy, SLO, doc-link, and parity checks as required.
4. Add failure messages that point to exact social files.
5. Test one missing-path fixture and one green fixture.
6. Verify forbidden claims in competitor matrix stay blocked.
7. Record gate evidence in the remediation note and PR.

## E. Acceptance
- `jq -e . microservices/social/manifest.json` passes.
- `jq -e . microservices/social/scorecards/overrides.json` passes.
- `buck2 build //:quality-lane-registry-authority-check # lane=hyperscaler-maturity --microservice social` passes.
- Required checks cover contracts, policies, SLOs, dashboards, parity, and doc links.
- Branch-protection dry run blocks a missing social policy path.

## F. Evidence
- Manifest: `manifest.json`.
- Scorecard: `scorecards/overrides.json`.
- Claim sources: `competitor-parity-matrix.md`, `feature-parity-matrix-2026-05-20.md`.
- Audit: `coherence-audit-2026-05-20.md`.

## G. Counterpart comparison
Counterparts do not publish their release gates. Oyatie can only claim a higher-governance posture if HG-SOCIAL is enforced by branch protection and catches drift in the same evidence used to compare against X, Instagram, TikTok, Snapchat, Bluesky, Mastodon, Threads, and LinkedIn.

## H. Foundation delivery expansion
- Deliverable detail: HG-SOCIAL required checks cover manifest parse, contract validation, Cedar policy tests, OpenSLO validation, dashboard validation, doc links, and parity trace.
- Deliverable detail: check names include service and evidence family for clear branch-protection failures.
- Deliverable detail: failure output names the missing social file and the impacted IP.
- Deliverable detail: scorecard overrides require justification, owner, expiry, and linked risk.
- Deliverable detail: counterpart claims resolve through competitor and feature matrices before promotion.
- Deliverable detail: remediation note entry includes the foundation IP count and verification commands.
- Deliverable detail: missing policy, dashboard, SLO, contract, catalog, or runbook paths block by default.
- Deliverable detail: Slack community moderation pressure is named explicitly in the counterpart coverage gate.

## I. Acceptance expansion
- Acceptance detail: `jq` must parse manifest and scorecard override files.
- Acceptance detail: required check registration must include contract, policy, SLO, dashboard, parity, and doc-link lanes.
- Acceptance detail: missing-path fixture must fail with the exact social path.
- Acceptance detail: passing fixture must cover all IP-001 through IP-018 foundation files.
- Acceptance detail: scorecard override scan must reject silent passes.
- Acceptance detail: line-count verification must show no social foundation IP in the 31-79 line band.
- Acceptance detail: counterpart grep must find an approved counterpart name in every foundation IP.
- Acceptance detail: Slack, X, Instagram, TikTok, and Reddit comparisons must fail closed if evidence paths drift.

## J. Evidence expansion
- Evidence detail: capture `jq -e . microservices/social/manifest.json`.
- Evidence detail: capture `jq -e . microservices/social/scorecards/overrides.json`.
- Evidence detail: capture HG-SOCIAL gate or dry-run output.
- Evidence detail: capture line-count verification for social foundation IPs.
- Evidence detail: capture approved-counterpart grep verification for social foundation IPs.
- Evidence detail: cite `coherence-audit-2026-05-20.md` for known gaps.
- Evidence detail: cite Slack as the approved community/channel moderation comparison name required by this repair.
