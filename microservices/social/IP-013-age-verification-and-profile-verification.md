---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-013-age-verification-and-profile-verification
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social + trust-safety
acceptance_lanes: [cargo-nextest, age-gate-test, profile-verification-policy-test]
---

# IP-013: Age verification and profile verification

## A. Problem
Social needs authentic profiles and age-aware controls before federation, ranking, DMs, recommendations, and minor-facing surfaces can safely ship.

## B. Approach
Implement age-verification and profile-verification bounded contexts using only crate paths named by the PRD/IP plus existing catalog adapter `oya-social-profile-verification-adapter-idv`. Cedar controls verification state, minor defaults, and disclosure.

## C. Deliverables
| Artifact | Role |
|---|---|
| `catalog/oya-social-profile-verification-adapter-idv.yaml` | Existing IDV adapter anchor. |
| `src/crates/oya-social-age-verification-{kernel,domain,usecase,api,adapter-postgres,sdk}/` | Planned family named by PRD/IP. |
| `src/crates/oya-social-profile-verification-{kernel,domain,usecase,api,adapter-postgres,adapter-idv,sdk}/` | Planned family named by PRD/IP/catalog. |
| `policy/profile-verification.cedar` and `policy/minor-protection.cedar` | Policy anchors. |
| `slos/minor-protection-engagement-correctness.openslo.yaml` | Minor-protection SLO. |

## D. Ordered implementation steps
1. Define age attestation, age bracket, guardian state, verification request, badge, and revocation types.
2. Implement COPPA/KOSA/EU age gate rules in domain/usecase tests.
3. Add IDV adapter boundary and explicit provider-result normalization.
4. Add profile verification issuance, revocation, and audit events.
5. Test minor defaults, false-positive review, badge revocation, and provider timeout.
6. Connect DMs, feed ranking, recommendations, and profile visibility to verification state.
7. Wire SLO and dashboard evidence.

## E. Acceptance
- `cargo nextest run -p oya-social-age-verification-kernel` passes.
- `cargo nextest run -p oya-social-profile-verification-adapter-idv` passes.
- `policy/minor-protection.cedar` and `policy/profile-verification.cedar` tests pass.
- `slos/minor-protection-engagement-correctness.openslo.yaml` resolves.
- Verification events validate against `contracts/asyncapi/social-events.yaml`.

## F. Evidence
- PRD FR-24 and FR-28: `PRD.md`.
- Policies: `policy/minor-protection.cedar`, `policy/profile-verification.cedar`.
- Catalog: `catalog/oya-social-profile-verification-adapter-idv.yaml`.
- Dashboard: `dashboards/minor-protection-health.json`.

## G. Counterpart comparison
Instagram Teen Accounts, TikTok Restricted Mode, Snapchat Family Center, X verification, and LinkedIn professional identity set the counterpart pressure. Oyatie must combine authenticity and age controls with pack-aware Cedar policy and audit evidence.

## H. Foundation delivery expansion
- Deliverable detail: age records include attestation source, age bracket, jurisdiction, guardian state, expiry, and audit correlation.
- Deliverable detail: profile verification records include request, provider result, badge state, revocation, and appeal metadata.
- Deliverable detail: IDV adapter normalizes provider statuses without storing unnecessary raw identity documents.
- Deliverable detail: minor-protection outputs feed DMs, feed ranking, notifications, search, and profile visibility.
- Deliverable detail: Cedar policies decide disclosure of verification state and age bracket.
- Deliverable detail: dashboards track pending, failed, revoked, appealed, and false-positive cases.
- Deliverable detail: data-residency rules bind provider artifacts to the tenant pack.
- Deliverable detail: Slack verified workspace identity is counterpart pressure for trustable community/user identity.

## I. Acceptance expansion
- Acceptance detail: under-13, 14-17, adult, unknown-age, and guardian cases must all have fixtures.
- Acceptance detail: provider timeout and mismatch tests must fail closed according to policy.
- Acceptance detail: verification issuance and revocation tests must update profile visibility.
- Acceptance detail: false-positive review tests must preserve appeal evidence.
- Acceptance detail: Cedar tests must distinguish disclosure to public, tenant admin, auditor, and guardian.
- Acceptance detail: AsyncAPI verification events must validate.
- Acceptance detail: SLO resolution must include minor-protection correctness when defined.
- Acceptance detail: Slack, LinkedIn, X, Instagram, TikTok, and Snapchat comparisons must map to identity and age-safety evidence.

## J. Evidence expansion
- Evidence detail: capture nextest output for age-verification and profile-verification crates.
- Evidence detail: capture policy tests for profile verification and minor protection.
- Evidence detail: capture AsyncAPI validation for verification events.
- Evidence detail: cite `catalog/oya-social-profile-verification-adapter-idv.yaml`.
- Evidence detail: cite `policy/profile-verification.cedar` and `policy/minor-protection.cedar`.
- Evidence detail: cite `dashboards/minor-protection-health.json`.
- Evidence detail: cite Slack as verified-community identity pressure alongside LinkedIn and X.
