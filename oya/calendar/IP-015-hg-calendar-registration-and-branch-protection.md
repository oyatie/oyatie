---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-015-hg-calendar-registration-and-branch-protection
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar + council-architecture + ops-platform
acceptance_lanes: [hyperscaler-gate-registration, branch-protection, governance-required-checks]
---

# IP-015: HG-CALENDAR registration and branch protection

## A. Problem
Calendar maturity gates are ineffective unless they are registered as required checks before PR promotion to `dev`.

## B. Approach
Register HG-CALENDAR as a required governance check backed by manifest, contract, policy, SLO, parity, and doc-link validation. Branch protection should block promotion when calendar evidence regresses or local paths disappear.

## C. Deliverables
| Artifact | Role |
|---|---|
| `manifest.json` | Service registration source. |
| `scorecards/overrides.json` | Local override and evidence scoring surface. |
| `coherence-audit-2026-05-20.md` | Current known-gaps register. |
| `contracts/`, `policy/`, `slos/`, `catalog/` | Required path families for the check. |
| `REMEDIATION-NOTES-2026-05-21-tier-scrub.md` | Existing remediation note file updated by this wave. |

## D. Ordered implementation steps
1. Verify manifest identity, owner, bounded contexts, contracts, SLOs, and IP entries.
2. Register the HG-CALENDAR gate with required check names.
3. Add branch protection requirements for contract validation, policy validation, SLO resolution, and doc-link resolution.
4. Add failure messages that point to exact calendar files.
5. Test with one intentional missing-path fixture and one passing fixture.
6. Update scorecard overrides only for documented exceptions.
7. Record gate evidence in remediation notes and PR body.

## E. Acceptance
- `jq -e . microservices/calendar/manifest.json` passes.
- `buck2 build //:quality-lane-registry-authority-check # lane=hyperscaler-maturity --microservice calendar` passes.
- Required check registration includes contract, policy, SLO, and doc-link lanes.
- `scorecards/overrides.json` parses and contains no silent pass for missing IP evidence.
- Branch-protection dry run blocks a missing calendar contract path.

## F. Evidence
- Manifest: `microservices/calendar/manifest.json`.
- Scorecard: `microservices/calendar/scorecards/overrides.json`.
- Audit: `microservices/calendar/coherence-audit-2026-05-20.md`.
- Contract and policy families: `contracts/`, `policy/`, `slos/`, `catalog/`.

## G. Counterpart comparison
Counterparts do not expose their internal promotion gates to tenants. Oyatie's claim is stronger only if HG-CALENDAR is enforceable in branch protection, making Google/Outlook/Cal.com parity claims fail closed when evidence files or governance checks drift.

## H. Foundation delivery expansion
- Deliverable detail: required checks include manifest parse, contract validation, Cedar policy validation, SLO resolution, doc links, and parity trace.
- Deliverable detail: check names use the HG-CALENDAR prefix consistently in branch protection.
- Deliverable detail: failure messages include exact file path, missing field, and responsible evidence family.
- Deliverable detail: scorecard overrides must include justification and expiry for any exception.
- Deliverable detail: remediation notes record the number of IP files expanded and the verification commands.
- Deliverable detail: missing contract/policy/SLO/catalog files block promotion by default.
- Deliverable detail: branch-protection dry run includes both passing and intentionally broken fixture paths.
- Deliverable detail: Slack collaboration-calendar pressure is represented in claim checks alongside Google/Outlook/Cal.com.

## I. Acceptance expansion
- Acceptance detail: `jq` must parse manifest and scorecard override files.
- Acceptance detail: branch-protection registration must include every required HG-CALENDAR lane.
- Acceptance detail: dry-run failure fixture must block on a missing calendar contract path.
- Acceptance detail: dry-run passing fixture must include all foundation IP files.
- Acceptance detail: scorecard override scan must reject silent pass-throughs.
- Acceptance detail: remediation note entry must include calendar foundation IP count.
- Acceptance detail: counterpart grep must find Slack or another approved counterpart in every foundation IP.
- Acceptance detail: Google, Outlook, Cal.com, and Slack comparisons must fail closed if evidence paths drift.

## J. Evidence expansion
- Evidence detail: capture `jq -e . microservices/calendar/manifest.json`.
- Evidence detail: capture `jq -e . microservices/calendar/scorecards/overrides.json`.
- Evidence detail: capture HG-CALENDAR gate or dry-run output.
- Evidence detail: capture line-count verification proving no foundation IP remains in the 31-79 line band.
- Evidence detail: capture counterpart grep verification over calendar foundation IPs.
- Evidence detail: cite `coherence-audit-2026-05-20.md` for known gaps.
- Evidence detail: cite Slack as the explicit collaboration-calendar comparison name required by this repair.
