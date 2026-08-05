---
plan_id: COMMUNITY-FD001-PLAN-SPEC-001
kanban_task: t_3321dc87
upstream_authority_lock_task: t_28c62d82
title: Community FD-001 Service Boundary and Mode Normalization Plan/Spec
status: ready-for-red-fixture-authoring
date: 2026-07-01
lane: community-fd001/plan-spec
scope: planning-spec-only
owner: communications-service-council + council-architecture + ops-security
conflict_class: product-spec-authority-community-fd001
blast_radius: planning/spec docs only; no runtime implementation; no generated JSON hand edits
allowed_path_prefixes:
  - plan/community/
  - specs/microservices/community.json
  - specs/microservices/manifests-index.json
  - specs/root-hub-pointers.json
  - docs/decisions/
  - evidence/community/
generated_faces_policy: Never hand-edit *.generated.json; materializers own generated faces
source_specs:
  - specs/microservices/community.json
  - specs/microservices/manifests-index.json#microservices[name=community]
  - specs/root-hub-pointers.json#entry_points.prd_community
  - specs/masterplan.json#first_deliverable
  - specs/masterplan.json#communications_service_catalog_2026_05_17
  - docs/decisions/ADR-0217-vertical-slice-rollout-order.md
  - docs/decisions/ADR-0234-connect-social-expansion-planning-contract.md
  - specs/microservices/social.json
  - specs/microservices/anonymous.json
  - specs/microservices/messenger.json
  - specs/microservices/mail.json
---

# Community FD-001 Service Boundary and Mode Normalization Plan/Spec

## 0. Source authority and claim boundary

This Plan/Spec is source-backed by:

- `specs/microservices/community.json`
- `specs/microservices/manifests-index.json#microservices[name=community]`
- `specs/root-hub-pointers.json#entry_points.prd_community`
- `specs/masterplan.json#first_deliverable`
- `specs/masterplan.json#communications_service_catalog_2026_05_17`
- `docs/decisions/ADR-0217-vertical-slice-rollout-order.md`
- `docs/decisions/ADR-0234-connect-social-expansion-planning-contract.md`
- `specs/microservices/social.json`
- `specs/microservices/anonymous.json`
- `specs/microservices/messenger.json`
- `specs/microservices/mail.json`
- upstream Kanban handoff `t_28c62d82`

`t_28c62d82` resolved the community FD-001 authority gate by adding `specs/microservices/community.json`, reconciling `specs/microservices/manifests-index.json`, and wiring `specs/root-hub-pointers.json#entry_points.prd_community`. That handoff is the only reason this Plan/Spec can author downstream RED fixture families. This artifact does not create runtime code, handlers, storage adapters, deployments, generated faces, production readiness evidence, hyperscaler maturity evidence, or GA/customer availability evidence.

`oya/community/manifest.json`, `registry/catalog/oya-community-*.yaml`, `tasks/community-*.md`, `specs/proto/backbone/community/community_post_store.proto`, and `oya/community/contracts/*` are inventory/provenance only until reconciled into the source-backed Plan/Spec + RED fixture chain. They must not be used as live implementation authority, build-readiness evidence, or proof that a handler/API/storage surface already exists.

## 1. FD-001 product boundary

Community is one flat FD-001 microservice inside the Tenant/RBAC-packaged core service set named by `specs/masterplan.json#first_deliverable` and ADR-0217. It must support personal and work contexts while staying separate from messenger, mail, ops-dashboard/control-center, workflow-engine, workflow-studio, ontology, intelligence, and infra.

The FD-001 community boundary contains:

- community-core: tenant-scoped forums, posts, comments/threading, votes/ranking, moderation queues, tags, role scoped groups, and knowledge/share surfaces;
- community-network: retired standalone network successor scope absorbed into community for professional profile facets, connections, jobs/recruiting, endorsements, recommendations, pages/events, and professional outreach facets, excluding LinkedIn-style engagement-feed and sponsored-post promotion patterns;
- community-social: visual/social mode sourced from `specs/microservices/social.json`, including feed, stories, reels/short-form media, AR effects, creator/collaboration facets, and context-isolated social graph contracts;
- community-shorts: retired standalone shorts surface, routed only through `specs/microservices/social.json` as a short-form content mode;
- community-anonymous: verified-anonymous workplace discussion mode sourced from `specs/microservices/anonymous.json`, work-context only, with no real identity in queryable post stores and legal reveal only through a later four-eyes threat-model gate.

Community does not own message delivery, mailbox semantics, workflow execution, control-center operations, graph schema administration, or cloud/infra control-plane behavior. Those surfaces remain under their own source specs and lifecycle lanes.

## 2. Mode normalization matrix

| Mode | Authority | Contexts | FD-001 meaning | Forbidden conflations |
|---|---|---|---|---|
| `community-core` | `specs/microservices/community.json` | personal + work | Tenant-scoped posts, comments, votes, moderation, community roles, knowledge sharing, professional/community groups. | No messenger threads, no mailbox records, no workflow engine execution, no ops-dashboard incident workflow. |
| `community-network` | `specs/microservices/community.json` | work-first, personal only when explicitly context-isolated | Retired network successor facets: profile, connection, recruiting/job, endorsement, recommendation, page/event, and professional outreach records. | No standalone `network` service fanout; no sponsored-post/ad engagement feed; no cross-context profile joins. |
| `community-social` | `specs/microservices/social.json` | personal + work | Visual social/feed/story/reel/AR/collaboration mode within community planning. | No standalone shorts service; no personal/work feed blending; no biometric persistence from AR effects; no ad signal cross-context targeting. |
| `community-shorts` | `specs/microservices/social.json` | personal + work, as allowed by social mode | Short-form media format inside `community-social`. | No `shorts` service, no separate shorts PRD fanout, no direct media-service readiness claim. |
| `community-anonymous` | `specs/microservices/anonymous.json` | work only | Verified-anonymous workplace boards, company channels, salary/compensation aggregates, polls, AMAs, career advice, and legal-hold reveal planning. | No personal anonymous boards; no employer individual attribution; no moderator real-identity reveal; no queryable real-user foreign key. |

## 3. Tenant/RBAC and context-isolation model

Every community artifact must be keyed by tenant and context before any handler or storage implementation is allowed. The first RED fixture card must prove these invariants before Build cards exist.

Required envelope fields for community-core and community-network records:

- `tenant_id`
- `community_id`
- `context_kind` (`personal` or `work`)
- `ownership_pillar` (`person`, `org`, or aggregate-only as declared by the mode)
- `actor_ref`
- `subject_ref` when a record targets another user/entity
- `role_grants_snapshot_ref`
- `policy_basis_ref`
- `retention_policy_id`
- `legal_hold_ids`
- `region_pack_ref`
- `audit_chain_ref`
- `schema_version`
- `idempotency_key` for state-changing commands

Baseline roles:

- `community.reader`
- `community.member`
- `community.contributor`
- `community.moderator`
- `community.admin`
- `community.professional_profile_owner`
- `community.recruiting_operator`
- `community.legal_hold_operator`
- `community.compliance_reviewer`
- `community.export_operator`

Policy rules:

1. Cedar/tenant-RBAC evaluates every read/write. No API path may infer authorization from URL shape alone.
2. Personal and work contexts are hermetically isolated. Any cross-context projection must be aggregate-only, consented, mediated through Workflow/Ontology, and audit-emitting.
3. Work-context community records inherit tenant retention, legal hold, eDiscovery, and audit-chain requirements.
4. Personal-context community records are unavailable to employer admin search/export APIs unless an explicit source spec later creates a lawful, user-consented aggregate export; this Plan/Spec does not grant one.
5. Anonymous mode stores anonymous author tokens, not real identity references, in routine post/query stores. Identity reveal requires a dedicated legal-hold/four-eyes contract and is not Build-ready from this artifact alone.

## 4. Data ownership, classes, and planned entities

Community data classes:

- `community_public_content`: posts, comments, reactions, votes, tags, and topic metadata intentionally shared within a community scope.
- `community_restricted_content`: work-community content, moderated queues, private group discussions, professional/recruiting facets, and admin-only metadata.
- `community_personal_content`: personal-context community/social artifacts under person-pillar ownership.
- `community_professional_profile`: work-context profile, role, endorsement, recommendation, job, and recruiting records inherited from retired network scope.
- `community_moderation_evidence`: reports, classifier signals, moderation decisions, appeals, reviewer actions, and policy snapshots.
- `community_anonymous_content`: anonymous posts, polls, salary/compensation entries, topic trends, and legal-hold reveal request envelopes sourced through `specs/microservices/anonymous.json`.
- `community_media_metadata`: social/story/reel/AR metadata sourced through `specs/microservices/social.json`; no biometric persistence is allowed.
- `community_audit_evidence`: append-only audit-chain rows, export manifests, deletion/retention/legal-hold receipts, and boundary-denial evidence.

Planned community-core/community-network entities for RED fixture naming:

- `CommunitySpace`
- `CommunityMembership`
- `CommunityPost`
- `CommunityComment`
- `CommunityVote`
- `CommunityThreadTree`
- `CommunityModerationCase`
- `CommunityModerationDecision`
- `CommunityTag`
- `CommunityProfessionalProfileFacet`
- `CommunityConnectionFacet`
- `CommunityJobReferralFacet`
- `CommunityImportExportJob`
- `CommunityBoundaryDenial`

Mode-owned entities remain under their mode specs until reconciled:

- `specs/microservices/social.json`: `SocialPost`, `SocialFollowEdge`, `SocialEngagementEvent`, plus social events `SocialContentPublished`, `SocialStoryExpired`, and `SocialBoundaryDenied`.
- `specs/microservices/anonymous.json`: `AnonPost`, `VerificationToken`, `AnonSalaryEntry`, `LegalHoldIdentityRevealRequest`, plus events `AnonPostPublished`, `AnonIdentityRevealed`, and `AnonBoundaryDenied`.
- `specs/microservices/messenger.json`: messenger entities/events stay outside community except for explicit, source-backed integration contracts.
- `specs/microservices/mail.json`: mail entities/events stay outside community except for explicit, source-backed workflow/import/export contracts.

## 5. Imports, exports, and migration-out plan

Imports must be source-labeled and policy-gated:

- legacy community inventory import: only from reconciled `oya/community/*` inventory rows after a RED fixture proves provenance classification and non-claim boundaries;
- retired network import: professional profile/connection/job/recruiting facets map into community-network records only through the community Plan/Spec and retired-successor guard;
- retired shorts import: short-form content metadata maps into community-social only through `specs/microservices/social.json`;
- messenger/mail import: not a community import. A user may link a message or mail artifact through Workflow/Ontology references only when the source service produces an allowed export/event contract;
- external community platform import: source hashes, actor mapping, consent, retention, moderation status, and chain-of-custody metadata must survive import.

Exports must preserve tenant/RBAC, retention, and auditability:

- community content export packages include posts, comments, votes, moderation decisions, media references, policy snapshots, source hashes, schema version, and audit-chain receipt IDs;
- professional profile/network export packages include user-controlled profile facets, endorsements/recommendations, connection graph snapshots, and recruiting/job provenance;
- anonymous exports are aggregate-only unless a four-eyes legal-hold reveal package exists and is separately authorized;
- personal-context exports cannot be queried by work/admin APIs;
- migration-out adapters must emit digest-addressed manifests that downstream services can verify without accepting community as a mailbox/messenger/workflow implementation.

## 6. API, schema, and event contract plan

API-first contract artifacts must exist before handlers. The RED fixture card should create failing contract fixtures for these families; it must not create runtime handlers.

Required contract families:

- REST/OpenAPI: `community-spaces-v1`, `community-posts-v1`, `community-comments-v1`, `community-votes-v1`, `community-moderation-v1`, `community-professional-profile-v1`, `community-import-export-v1`.
- gRPC/proto: `community.post_store.v1`, `community.thread_tree.v1`, `community.vote_tally.v1`, `community.moderation_queue.v1`, `community.professional_profile.v1`.
- AsyncAPI/event topics: `community.post.published.v1`, `community.post.updated.v1`, `community.comment.published.v1`, `community.vote.cast.v1`, `community.vote.retracted.v1`, `community.moderation.case.opened.v1`, `community.moderation.decision.recorded.v1`, `community.boundary.denied.v1`, `audit.community.policy.v1`.
- JSON/schema registry: request/response envelopes, data-class tags, policy-basis references, idempotency keys, retention/legal-hold fields, boundary-denial reason codes, and compatibility/version metadata.

Cross-service integration policy:

1. Workflow and Ontology are the required mediators for cross-product reads/writes.
2. Community must not directly call messenger/mail/workflow-engine/ops-dashboard APIs from domain logic.
3. Direct cross-microservice calls are forbidden except through approved adapter/contract layers backed by a source spec and RED fixture.
4. All event producers must include tenant, context, policy basis, idempotency key, schema version, and audit-chain references.
5. State-changing operations must be idempotent and must emit boundary-denial events when policy rejects a cross-context, cross-service, or retired-authority path.

## 7. Compliance and threat model

Threats that must be represented in RED fixtures and review/fix lenses:

- cross-tenant community access;
- personal/work context bleed;
- employer discovery of personal community/social records;
- professional profile/network data used for advertising or sponsored ranking without a source-backed policy;
- anonymous identity linkage in queryable stores;
- moderator or HR individual-attribution of anonymous content;
- legal-hold identity reveal without four-eyes approval and sealed audit package;
- AR or media processing persisting biometric/special-category data outside an in-session social pipeline;
- harassment, doxxing, abuse, spam, brigading, vote manipulation, sockpuppeting, malicious reporting, and moderator abuse;
- content-safety model false positives/negatives without appeal and audit evidence;
- deleted/edited content disappearing without retention/legal-hold/audit semantics;
- import chain-of-custody gaps;
- cross-region or pack-ineligible data placement;
- silent readiness claims based on inventory/catalog/proto files instead of source-backed contracts and tests.

Compliance control requirements:

- default-deny Cedar/Tenant-RBAC for every read/write/export/moderation action;
- data-class and retention classification at write time;
- legal-hold preservation for work/professional records and moderation evidence;
- audit-chain events for create/update/delete/moderation/export/import/legal-hold/boundary-denial actions;
- anonymity controls sourced from `specs/microservices/anonymous.json`, including work-context-only scope, HSM-backed salt/vault planning, no real identity foreign key in routine stores, and four-eyes legal reveal planning;
- PIPA/GDPR/CCPA-style privacy pack alignment where personal or anonymous content is processed;
- content moderation appeal and abuse-defense evidence suitable for security/privacy/abuse review.

## 8. Localization and Korea pack

FD-001 requires canonical base plus Korea localization pack evidence. Community Build cards remain blocked until RED fixtures prove these requirements are represented in contracts:

- Korean language UI copy, moderation labels, appeal reasons, content category names, and accessibility text;
- PIPA/privacy consent and data-subject export/delete flow copy for personal/social and anonymous surfaces;
- KR workplace/community norms for anonymous boards, compensation discussions, recruiting, and moderation escalation;
- regional retention/legal-hold overlays for work-context community and anonymous records;
- moderation abuse categories and escalation queues that support Korean locale and time-zone operations;
- audit/evidence exports that preserve locale and pack version;
- support/runbook routing for Korea pack incidents.

`specs/microservices/social.json` and `specs/microservices/anonymous.json` both name KR/regional pack readiness gates. Community-core and community-network must not claim Korea pack readiness until comparable contract fixtures, UX evidence, and runbook/observability evidence exist.

## 9. SLO, runbook, rollback, observability, capacity, and cost plan

Planning targets only; no measured SLO exists from this artifact.

SLO families to define before implementation:

- community feed/read p95/p99 latency by tenant cell and context;
- post/comment/vote write p95/p99 latency and idempotency replay correctness;
- moderation queue triage freshness and appeal response objectives;
- boundary-denial correctness at 100% for cross-tenant/cross-context/cross-service rejects;
- audit-chain emission success rate for every write/export/moderation/legal-hold event;
- social mode targets inherited from `specs/microservices/social.json`, including feed load, story purge, AR no-biometric-persistence, regional-pack-KR, and cost envelope planning;
- anonymous mode targets inherited from `specs/microservices/anonymous.json`, including no identity leakage, aggregate-only employer access, legal-hold four-eyes, verification-vault isolation, and performance/cost planning.

Runbooks to author before production-readiness claims:

- moderation backlog saturation;
- vote/ranking abuse or hot controversy surge;
- cross-context boundary-denial spike;
- anonymous identity reveal request handling;
- import/export failure and chain-of-custody repair;
- regional pack policy mismatch;
- social media processing/AR privacy incident;
- retention/legal-hold purge conflict;
- search/index drift versus source-of-record records.

Rollback/containment plan:

- rollback must disable the affected mode, route, topic, or tenant pack through configuration/feature policy without deleting source records;
- idempotency logs and audit-chain receipts must survive rollback;
- moderation/legal-hold queues must fail closed;
- imports must be pauseable by source manifest/digest;
- social/anonymous modes must be independently containable without disabling messenger/mail/workflow-engine/ops-dashboard.

Observability plan:

- golden signals: latency, traffic, errors, saturation, queue depth, policy-denial rate, audit emission success, boundary-denial counts, export/import duration, moderation action latency;
- tracing: tenant/context/mode low-cardinality attributes, matched route template, idempotency key hash, policy decision class, audit-chain receipt status;
- metrics must avoid raw PII, message bodies, anonymous identity material, and content payloads;
- dashboards must separate community-core, community-social, community-anonymous, retired-successor routing, and external integration errors.

Capacity/backpressure/cost plan:

- tenant quotas for posts/comments/votes/media references/moderation cases/import-export jobs;
- hot-thread and vote-spike backpressure with fair queuing per tenant/context;
- independent horizontal scaling of stateless REST/API replicas and workers;
- cell-shard partitioning by tenant/community, not by retired product wrapper;
- FinOps tags for storage, search, moderation, media metadata, anonymous aggregation, and export workloads.

## 10. UX and browser evidence requirements

Any community UI Build card must include browser/user-story evidence; green CI alone is not sufficient.

Required user-story evidence families:

- personal/work context switch reconstructs community feed, roles, search, and composer state with no leaked artifacts;
- create/edit/delete community post and comment with visible policy/retention affordances;
- vote/ranking and retraction behavior is understandable and audit-backed;
- moderation queue triage, appeal, and policy disclosure UX is accessible and localized;
- professional profile/network facets show source/provenance and do not blend personal social data;
- community-social mode proves personal/work feed isolation, story/reel UX, AR no-biometric-persistence disclosure, and creator/collaboration consent affordances before implementation readiness claims;
- community-anonymous mode proves work-context-only entry, verification status, anonymous author display, employer aggregate-only dashboard, and legal-hold reveal gate copy;
- import/export UX shows chain-of-custody, source digest, policy basis, and rollback/repair states;
- Korea pack evidence includes Korean copy, locale formatting, timezone handling, moderation labels, accessibility labels, and support escalation routes.

Accessibility requirements:

- keyboard navigation for feed, thread, composer, moderation, import/export, and anonymous board flows;
- screen-reader labels for policy badges, retention/legal-hold state, mode/context switcher, and anonymous identity guarantees;
- visible focus, color contrast, non-color-only status, and motion-reduction options for social/story/reel surfaces;
- explicit source/provenance text where inventory/proto/catalog data is shown in an admin or operator view.

## 11. Downstream RED fixture families

The child RED card `t_f07b0559` should use these fixture families. These are fixture families only; this Plan/Spec does not create handlers, crates, runtime code, generated JSON, or production evidence.

1. `community_fd001_authority_boundary_fixture`: proves `specs/microservices/community.json`, ADR-0217, ADR-0234, masterplan first_deliverable, and root-hub `prd_community` are cited before Build fanout.
2. `community_fd001_inventory_provenance_rejection_fixture`: rejects `oya/community/manifest.json`, registry catalog rows, task plans, and proto discovery as live implementation authority unless reconciled into source-backed contracts.
3. `community_fd001_mode_normalization_fixture`: routes community-core, community-network, community-social, community-shorts, and community-anonymous to their source authorities.
4. `community_fd001_retired_network_shorts_successor_fixture`: refuses standalone `network` and `shorts` service fanout while allowing successor references to community/social authority.
5. `community_fd001_messenger_mail_separation_fixture`: proves messenger and mail remain separate services with their own PRDs/contracts and are not duplicate community coverage.
6. `community_fd001_workflow_ops_ontology_separation_fixture`: refuses workflow-engine, workflow-studio, ops-dashboard/control-center, ontology, intelligence, and infra conflation.
7. `community_fd001_tenant_rbac_context_isolation_fixture`: requires tenant, context, ownership pillar, role grants, policy basis, retention, legal hold, audit, and idempotency fields on state-changing records.
8. `community_fd001_data_class_retention_legal_hold_fixture`: covers data-class tagging, retention, legal hold, edit/delete audit, and export/import chain-of-custody.
9. `community_fd001_anonymity_privacy_fixture`: imports anonymous-mode guarantees from `specs/microservices/anonymous.json` and refuses real identity in routine anonymous post/query stores.
10. `community_fd001_social_media_privacy_fixture`: imports social-mode guarantees from `specs/microservices/social.json` and refuses cross-context feed joins, ad-signal joins, and biometric persistence.
11. `community_fd001_api_schema_event_contract_fixture`: requires REST/gRPC/AsyncAPI/schema contracts before handlers and checks idempotency, audit, and boundary-denial fields.
12. `community_fd001_observability_slo_runbook_fixture`: requires SLO families, golden signals, runbooks, rollback/containment, capacity/backpressure, and cost-envelope targets before readiness claims.
13. `community_fd001_localization_kr_fixture`: requires canonical base plus Korea pack contract evidence for community-core/network/social/anonymous UX and support flows.
14. `community_fd001_ux_accessibility_evidence_fixture`: requires browser/user-story and accessibility evidence families for any UI Build card.
15. `community_fd001_generated_face_no_hand_edit_fixture`: refuses hand edits to `*.generated.json` and routes materialization to owning scripts/gates.
16. `community_fd001_build_parentage_fixture`: refuses Build cards unless they are parented behind this Plan/Spec card `t_3321dc87` and the RED fixture card `t_f07b0559` or its direct approved successor.

## 12. Single-writer, path, and dependency policy

Conflict class: `product-spec-authority-community-fd001`.

Single-writer allowed paths for this Plan/Spec lane:

- `plan/community/community-fd001-service-boundary-plan-spec.md`

Serialized paths that must not be edited by competing lanes while community FD-001 authority/Plan/Spec work is active:

- `specs/root-hub-pointers.json`
- `specs/microservices/manifests-index.json`
- `specs/microservices/community.json`
- future community Plan/Spec files under `plan/community/`

Future Build cards must be parented behind both:

- Plan/Spec gate: `t_3321dc87`
- RED fixture gate: `t_f07b0559`

Build cards must name allowed path prefixes, forbidden shared roots, generated-face policy, and review/fix lenses before they are spawnable.

Generated-face policy: Never hand-edit any `*.generated.json`; materializers own generated faces, and the diff-policy gate must fail closed on hand edits.

## 13. Non-goals

This Plan/Spec does not authorize:

- runtime implementation;
- crate creation;
- handler/storage/queue/search/moderation-engine code;
- direct edits to generated JSON faces;
- claims that `oya/community/manifest.json`, registry catalog rows, task plans, or proto files are live implementation authority;
- messenger/mail/workflow-engine/ops-dashboard/control-center conflation;
- a standalone network or shorts service;
- production, GA, hyperscaler maturity, customer availability, measured SLO, deployment, or rollout claims.

## 14. Verification checklist for this Plan/Spec lane

Required verification for this card:

- `python3 -m json.tool specs/microservices/community.json specs/microservices/manifests-index.json specs/root-hub-pointers.json`
- readback of this Plan/Spec artifact;
- confirm no `*.generated.json` files were edited by this task;
- fresh Kanban duplicate checks for community, FD-001, network, social, anonymous, messenger, mail, ops-dashboard, and workflow-engine;
- retired-authority guard on any new child candidates; for this lane, no new Kanban child candidates are created, so the guard applies to the named downstream RED fixture families and their successor routing semantics;
- verify `t_f07b0559` remains the RED fixture child after this Plan/Spec gate and that future Build cards are not created from this card alone.
