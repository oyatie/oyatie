---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M03-P01-IP-004
title: Cloud IAM Cedar + SSO + STS API
status: partial (cedar-bind-app-composition-green; federated-oidc-sts-api-runtime-green; provider-managed-idp-crud/live-smoke pending)
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Bring cloud.iam.* to stable with Cedar policy substrate + SSO + STS.
---

# M03-P01-IP-004 — Cloud IAM Cedar + SSO + STS API

## Purpose
Bring cloud.iam.* to stable with Cedar policy substrate + SSO + STS.

## Symbols-to-grit-claim
```
crates/oya-cloud-iam-api/src/lib.rs::create_role
crates/oya-cloud-iam-api/src/lib.rs::issue_sts_token
crates/oya-cloud-iam-app/src/lib.rs::bind_cedar_policy
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M01-P08 ≥ P5 merged.

## Acceptance-test-commands
```
cargo test -p <owning-crate> --all-features
cargo run -p oya-foundry-fitness-cohesion -- <owning-crate-glob>
scripts/check.sh
```

## Done-criteria
- All acceptance-test commands return 0.
- Distroless image built (if IP ships a deployed binary); size < per-binary budget per `docs/standards/image-size-budgets.md`.
- No provider-specific deps outside adapter crates (Directive 4).
- All direct deps current LTS or have ADR-tracked exception (Directive 8).
- PR "good-taste audit" section non-empty (Directive 7).
- Distroless + provider-coupling + LTS-dependency lanes green on PR.

## Rollback-procedure
`grit done` is atomic per-symbol; if a subsequent IP regresses, revert the merge commit. For crates that other IPs depend on, follow per-crate split unwind per ADR-0015 §7.

## Next-IP-pointer
Next IP in this phase's INDEX list (or first IP of next phase if phase complete).

## Icm-store-payload
```
icm store -t context-oyatie -c 'M03-P01-IP-004 Cloud IAM Cedar + SSO + STS API shipped; acceptance commands green' -i high -k 'M03-P01-IP-004,complete'
```

## Progress

### 2026-05-21 — Cedar policy bind app composition green
- ChangeSet: `cs-m03-p01-cloud-iam-cedar-bind-app-2026-05-21`.
- Added `oya-cloud-iam-app::bind_cedar_policy` as a domain-only app seam that publishes a Cedar policy version and creates the matching IAM role transactionally.
- Kept app dependencies inward to `oya-cloud-iam-domain` and `oya-policy-cedar-domain`; architecture-boundaries rejects app-to-api coupling.
- Verification: targeted app fmt/test/clippy passed; existing Cloud IAM API and Cedar API tests passed; catalog/cohesion/planning-closure/architecture-boundaries passed; `oya verify --ci-required` passed 89/89 lanes.
- Remaining: SSO/federation runtime and STS expansion beyond existing API tests are follow-up slices before this IP is complete.

### 2026-05-21 — Federated OIDC STS API runtime green
- ChangeSet: `cs-m03-p01-cloud-iam-sso-sts-api-2026-05-21`.
- Added Cloud IAM API regression coverage proving an OIDC-backed external principal can receive a scoped STS token through `issue_cloud_iam_sts_token_from_api` when it presents an external id.
- Added denial coverage proving external principals without `external_id` map to Cloud IAM policy denial (`403`) instead of a generic request failure.
- Kept this slice provider-agnostic and in-memory: no cloud SDK, network call, credential, or provider-specific adapter was introduced.
- Verification: targeted federated STS tests passed; full `oya-cloud-iam-api` test suite passed with 14 tests after formatting.
- Remaining: provider-managed IdP CRUD/runtime adapter and live-provider smoke are separate follow-up slices before this IP is complete.

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: Cedar policy publication and IAM role creation now share one rollback-friendly app seam instead of letting handlers duplicate cross-kernel partial-commit logic; federated STS issuance now reuses the existing STS API path instead of growing a separate SSO token path. Provider-managed IdP CRUD/live smoke remains a separate follow-up boundary.
