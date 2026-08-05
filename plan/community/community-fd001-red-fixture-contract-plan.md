---
plan_id: COMMUNITY-FD001-RED-FIXTURE-CONTRACT-PLAN-001
kanban_task: t_f07b0559
parent_plan_spec_task: t_3321dc87
review_fix_parent_task: t_74fea7bb
title: Community FD-001 RED Fixture and Contract Plan
status: red-check-ready-contract-artifacts-pending
generated_at_utc: 2026-07-01T13:27:25Z
source_commit_at_authoring: c52bdb09ea33
claim_ceiling: metadata-fixture-contract-plan-only; no runtime handlers, storage, generated JSON hand edits, measured SLO, production readiness, GA, customer availability, or hyperscaler maturity claim
---

# Community FD-001 RED Fixture and Contract Plan

## 0. Claim boundary

This artifact is the RED fixture/contract handoff for Kanban task `t_f07b0559`. It converts the approved parent Plan/Spec fixture-family names into a machine-readable fixture manifest and a fail-closed RED checker. It does not create community runtime code, handlers, crates, storage adapters, deployment/IaC, generated faces, measured SLOs, browser evidence, production readiness evidence, hyperscaler maturity evidence, GA evidence, or customer-availability evidence.

The only claim made now is: the exact RED fixture families from `plan/community/community-fd001-service-boundary-plan-spec.md:313-333` are preserved in `specs/fixtures/community-fd001/red-fixtures.json`; the checker validates that fixture plan against source authority and intentionally fails until future Build cards author source-backed API/data/event/schema contract artifacts under `contracts/community-fd001/`.

## 1. Source authority read for this card

Primary source authority:

- `specs/microservices/community.json` — authority lock for community FD-001 planning and RED-contract decomposition only; lines 15-27 set the current authority and non-claim boundary, lines 74-129 normalize modes and pre-build requirements, and lines 150-156 keep Plan/Spec, RED, Build, Review/fix, and Merge/Rollout lifecycle gates distinct.
- `plan/community/community-fd001-service-boundary-plan-spec.md` — approved parent Plan/Spec; lines 313-333 name the exact downstream RED fixture families that this card is allowed to use.
- `docs/decisions/ADR-0217-vertical-slice-rollout-order.md` — lines 24-50 and 54-58 require Tenant/RBAC-packaged core services at full depth, flat microservices, API-first contracts before handlers, Ops Dashboard / Control Center as separate scope, canonical base plus Korea pack, and false-green/silent-regression rejection.
- `docs/decisions/ADR-0234-connect-social-expansion-planning-contract.md` — lines 47-54 constrain social/anonymous/network/shorts expansion as planning contracts, require Workflow/Ontology mediation, and reject monolithic conflation.
- `specs/microservices/social.json` — source for `community-social` and `community-shorts` mode requirements, including feed/story/reel/AR privacy, cross-context denial, regional pack, and frontend/UX mode obligations.
- `specs/microservices/anonymous.json` — source for `community-anonymous` mode requirements, including work-context-only scope, anonymous author token storage, employer aggregate-only posture, legal-hold four-eyes reveal, regional pack, and frontend/UX mode obligations.

Explicit non-authorities for this card:

- `oya/community/manifest.json`
- `registry/catalog/oya-community-*.yaml`
- `tasks/community-*.md`
- `specs/proto/backbone/community/community_post_store.proto`
- `oya/community/contracts/*`

Those paths may be provenance or future reconciliation inputs only. They are not live implementation authority and they are not allowed to determine fixture names for this card.

## 2. Machine-readable fixture manifest

Manifest path:

`specs/fixtures/community-fd001/red-fixtures.json`

It preserves the exact fixture IDs from the parent Plan/Spec:

1. `community_fd001_authority_boundary_fixture`
2. `community_fd001_inventory_provenance_rejection_fixture`
3. `community_fd001_mode_normalization_fixture`
4. `community_fd001_retired_network_shorts_successor_fixture`
5. `community_fd001_messenger_mail_separation_fixture`
6. `community_fd001_workflow_ops_ontology_separation_fixture`
7. `community_fd001_tenant_rbac_context_isolation_fixture`
8. `community_fd001_data_class_retention_legal_hold_fixture`
9. `community_fd001_anonymity_privacy_fixture`
10. `community_fd001_social_media_privacy_fixture`
11. `community_fd001_api_schema_event_contract_fixture`
12. `community_fd001_observability_slo_runbook_fixture`
13. `community_fd001_localization_kr_fixture`
14. `community_fd001_ux_accessibility_evidence_fixture`
15. `community_fd001_generated_face_no_hand_edit_fixture`
16. `community_fd001_build_parentage_fixture`

Every fixture carries `expected_red_status=RED_UNTIL_CONTRACT_ARTIFACT_EXISTS` and future artifact paths under `contracts/community-fd001/`. Those future paths are deliberately absent in this card, so the contract check remains RED until a later Build card creates source-backed contracts and updates/extends the checker with real contract replay.

## 3. Required API/data/event/schema contract families

The future Build chain must satisfy these parent Plan/Spec contract families before handlers:

- REST/OpenAPI: `community-spaces-v1`, `community-posts-v1`, `community-comments-v1`, `community-votes-v1`, `community-moderation-v1`, `community-professional-profile-v1`, `community-import-export-v1`.
- gRPC/proto: `community.post_store.v1`, `community.thread_tree.v1`, `community.vote_tally.v1`, `community.moderation_queue.v1`, `community.professional_profile.v1`.
- AsyncAPI/event topics: `community.post.published.v1`, `community.post.updated.v1`, `community.comment.published.v1`, `community.vote.cast.v1`, `community.vote.retracted.v1`, `community.moderation.case.opened.v1`, `community.moderation.decision.recorded.v1`, `community.boundary.denied.v1`, `audit.community.policy.v1`.
- JSON/schema registry: request/response envelopes, data-class tags, policy-basis refs, idempotency keys, retention/legal-hold fields, boundary-denial reason codes, and compatibility/version metadata.

## 4. Separation, retired-authority, and mode gates

This RED plan catches the specific conflations named by the card:

- Messenger/mail conflation: `community_fd001_messenger_mail_separation_fixture` rejects messenger thread delivery, mailbox records, message delivery ownership, and mail imports as community coverage unless an explicit source-backed integration/export contract exists.
- Workflow/Ops/Ontology conflation: `community_fd001_workflow_ops_ontology_separation_fixture` rejects Workflow execution, Workflow Studio authoring, Ops Dashboard / Control Center operations, Ontology schema administration, Intelligence, and infra/control-plane behavior as community ownership.
- Retired network/shorts authority: `community_fd001_retired_network_shorts_successor_fixture` rejects standalone `network` and `shorts` service fanout and only allows `community-network` through `specs/microservices/community.json` and `community-shorts` through `specs/microservices/social.json`.
- Social mode: `community_fd001_social_media_privacy_fixture` cites `specs/microservices/social.json` and rejects personal/work feed blending, ad-signal cross-context targeting, AR biometric persistence, and standalone shorts claims.
- Anonymous mode: `community_fd001_anonymity_privacy_fixture` cites `specs/microservices/anonymous.json` and rejects real identity in routine anonymous post/query stores, personal anonymous boards, employer individual attribution, moderator identity reveal, and legal reveal without a four-eyes sealed audit package.

## 5. RED/fail-closed check added by this card

Checker path:

`scripts/tests/community_fd001_red_fixture_contract_check.py`

Self-test command:

`python3 scripts/tests/community_fd001_red_fixture_contract_check.py --self-test`

Expected now: zero exit, after validating the live manifest and proving the validator rejects missing fixture IDs, stale inventory/proto live-authority claims, messenger/mail conflation gaps, workflow/ops conflation gaps, missing social/anonymous mode citations, generated JSON future artifacts, and fabricated green status.

Intentional RED command:

`python3 scripts/tests/community_fd001_red_fixture_contract_check.py --manifest specs/fixtures/community-fd001/red-fixtures.json --contract-root contracts/community-fd001`

Expected now: non-zero exit with a missing future contract artifact message. That failure is correct for this RED plan because no source-backed community FD-001 API/data/event/schema contracts exist under `contracts/community-fd001/` yet.

## 6. Future Build card boundaries

Allowed future implementation path class after this RED gate:

- `contracts/community-fd001/`
- future community implementation paths only when the Build card explicitly owns them, is parented behind `t_3321dc87` and `t_f07b0559` or an approved direct successor, and declares allowed path prefixes, forbidden shared roots, generated-face policy, and review/fix lenses.

Forbidden in this RED fixture/contract card:

- runtime handlers, crates, storage adapters, queue/search/moderation-engine code, or deployment/IaC;
- generated JSON edits;
- root-hub pointer edits;
- broad source-authority rewrites to `specs/microservices/community.json`, `specs/microservices/social.json`, or `specs/microservices/anonymous.json`;
- claims that inventory/catalog/task/proto files are live implementation authority;
- messenger/mail/workflow-engine/ops-dashboard/control-center/ontology/intelligence/infra conflation;
- standalone network or shorts service authority;
- production, GA, hyperscaler maturity, customer availability, measured SLO, deployment, or rollout claims.

## 7. Verification commands for this card

Run and record:

1. `python3 -m json.tool specs/fixtures/community-fd001/red-fixtures.json >/tmp/community-fd001-red-fixtures.json`
   - Expected now: pass.
2. `python3 -m json.tool specs/microservices/community.json >/tmp/community-fd001-community.json`
   - Expected now: pass.
3. `python3 -m json.tool specs/microservices/social.json >/tmp/community-fd001-social.json`
   - Expected now: pass.
4. `python3 -m json.tool specs/microservices/anonymous.json >/tmp/community-fd001-anonymous.json`
   - Expected now: pass.
5. `python3 scripts/tests/community_fd001_red_fixture_contract_check.py --self-test`
   - Expected now: pass.
6. `python3 scripts/tests/community_fd001_red_fixture_contract_check.py --manifest specs/fixtures/community-fd001/red-fixtures.json --contract-root contracts/community-fd001`
   - Expected now: fail closed because future contract artifacts are absent.
7. `python3 - <<'PY' ... retired-authority guard ... PY`
   - Expected now: pass, confirming `network`/`shorts` in this plan and manifest are confined to successor/refusal contexts and not standalone live authority.

Closeout condition for `t_f07b0559`: this artifact, the JSON fixture manifest, and the fail-closed checker exist; JSON validation passes; checker self-tests pass; the intentional RED check fails for missing future contract artifacts; and the Kanban closeout explicitly states that future Build work remains blocked behind the Plan/Spec and RED fixture gates.
