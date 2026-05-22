---
doc_class: Implementation-Plan
ip_id: IP-journey-j113-cross-tenant-dm-boundary
journey_ref: docs/user-journeys/j113-cross-tenant-internship-from-handshake/
microservice: messenger
status: draft
date: 2026-05-20
authority_tier: 3
related_adrs:
  - ADR-0242-oyatie-is-a-tenant-doctrine
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0249-multi-category-marketplace-doctrine
  - ADR-0263-observability-emission-contract
  - ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0313-conglomerate-tenant-hierarchy
  - ADR-0314-marketplace-universal-deal-settlement-substrate
planned_enforcement_ref: oya-governance-doc-rigor
---

# IP - messenger role in j113: Cross-tenant internship from Handshake

Role: cross-tenant-dm-boundary.

Journey purpose: Aiyana, a student, interns at KrampusCorp through Community Handshake-mode with student and employer
tenant bindings, weekly timesheets, stipend, and mentor DM channel.

## Scope

messenger owns only the cross-tenant-dm-boundary slice for j113. It does not absorb another service responsibility, does
not bypass Cedar, and does not write into another tenant-owned store without an explicit grant.

## Acceptance criteria

1. messenger exposes or consumes the typed j113 contract without ad hoc string parsing.
2. Every state-changing path evaluates Cedar and records the permit id.
3. Every mutation emits an ADR-0263 observability event with audit_id linkage.
4. Rollback exists for each reversible state and pause exists for irreversible state.
5. Cross-tenant reads require explicit tenant pair and purpose.
6. Personal-tenant data is default-deny unless the personal tenant owner consents.
7. The implementation maps to one of the ADR-0105 canonical layers.
8. The test plan includes success, expired-permit, outage, and residency-hold cases.

## Substantive journey rows

The previous 68-row deliverable loop stamped the same path-numbered generic increment wording across employer and
university tenants. This table keeps the messenger-owned internship communication actions that map to current
OpenAPI, AsyncAPI, proto3, Cedar, SLO, and journey artifacts. Removed rows named payment, ontology, DRMP, and generic
marketplace events that messenger does not own.

| Row | Specific messenger action | Trigger and actor | State effect and evidence | Counterpart equivalence |
|---:|---|---|---|---|
| 001 | Create the mentor/student DM or small GroupDM after Community Handshake confirms the internship match. | `community` emits the accepted internship handoff from `docs/user-journeys/j113-cross-tenant-internship-from-handshake/handshake.md`; actor is `ChannelAdmin` or `TenantOperator` calling `POST /channels` (`createChannel`) with `Channel.kind = DM` or `GroupDM`, `context_kind = Professional`, and employer tenant scope. | Persists `Channel.channel_id`, `tenant_id`, `kind`, `initial_members`, and `retention_policy_id`; emits AsyncAPI `channel-created` and records `X-Request-Id` for audit-chain sealing. | Matches Teams cross-org chat creation and Slack Connect DM/channel setup, with explicit tenant and context headers instead of workspace-only visibility. |
| 002 | Add the university career-center observer as a scoped member only when the internship agreement permits it. | Career-center approval arrives from the j113 workflow; actor is `ChannelAdmin` invoking `POST /channels/{channel_id}/members` (`addChannelMember`) and Cedar `Action::"add_channel_member"` in `policy/channel-scope.cedar`. | Emits AsyncAPI `channel-member-changed` with `action = granted`; evidence includes `member_ref`, `actor_ref`, `changed_at`, and the Cedar principal `admin_channels` membership. | Matches Teams guest access and Slack Connect external member invitations; Oyatie avoids tenant-wide guest elevation. |
| 003 | Post the weekly timesheet-reminder card into the mentor DM without owning the timesheet record. | `workflow-engine` schedules the reminder after workplace-integration advances the weekly timesheet milestone; actor is a service principal posting through `MessageStream.PostMessage` / `POST /channels/{channel_id}/messages` under channel membership. | Persists `Message.message_id`, `thread_id` when linked to a prior week, `author_ref`, `data_class`, and `content_hash`; emits AsyncAPI `message-posted` and carries audit envelope headers. | Matches Teams Adaptive Card reminders and Slack Workflow Builder messages, but keeps workflow state outside messenger. |
| 004 | Let Aiyana reply to mentor feedback in the same thread while preventing unrelated employer channels from seeing it. | Student calls `POST /threads/{thread_id}/replies` (`postThreadReply`); actor is `EndUser` with `member_of_channels` under `policy/tenant-scope.cedar` permit 2. | Updates `Thread.reply_count` and emits `message-posted` with `thread_id`; non-member reads are denied by `policy/channel-scope.cedar` and `policy/tenant-scope.cedar`. | Matches Slack threaded internship check-ins and Teams reply chains, with Cedar-backed channel membership evidence. |
| 005 | Hide student personal-context DMs from employer admin disclosure and eDiscovery paths. | Any employer admin or compliance actor attempts `POST /disclosures` or `POST /holds` against a Personal `DirectConversation`; actor is rejected by `policy/personal-dm-scope.cedar`. | Cedar forbids `disclose_dm_body`, `admin_decrypt_dm`, `read_dm_plaintext`, `apply_ediscovery_hold`, and `export_for_ediscovery`; evidence is a deny decision plus absence from disclosure events. | Contrasts with Slack/Teams enterprise exports, where vendor/admin disclosure can include work chat; Oyatie personal DM ciphertext stays out of tenant-admin reach. |
| 006 | Surface mentor presence to Aiyana only inside the shared internship channel membership graph. | Client calls `GET /presence/{user_ref}` (`getPresence`); actor is an `EndUser` whose `member_of_channels` includes the DM or GroupDM. | Returns `Presence.user_ref`, `state`, `updated_at`; emits `presence-changed` / `ws-presence-frame`; evidence touches `presence-propagation.openslo.yaml` and Cedar `read_presence` permit. | Matches Teams presence and Slack status, with membership-bound visibility instead of global directory presence. |
| 007 | Attach signed onboarding or internship documents to the channel while leaving document authority with Drive/workplace-integration. | Mentor initiates `POST /attachments` then posts the returned `attachment_id` in `attachment_refs`; actor is a channel member with `upload_attachment` permission. | Persists `Attachment.digest_sha256`, `size_bytes`, `mime_type`, `scan_status`; emits AsyncAPI `file-attached` and uses `attachment-scan-freshness.openslo.yaml` as evidence touch. | Matches Slack/Teams file share UX, but messenger stores scanned attachment metadata and signed blob references rather than becoming the HR document system. |
| 008 | Search internship messages for weekly feedback during an approved review window. | Mentor or career-center observer calls `GET /search` with `channel_id`, `after`, and `before`; actor is `EndUser` or `TenantOperator` scoped by channel/tenant. | Returns Cedar-filtered `SearchResult.results`; evidence is `search-latency.openslo.yaml`, `request_id`, and no results for channels outside membership. | Matches Slack and Teams channel search, with server-side Cedar filtering per `IP-013-search-and-cedar-filter.md`. |
| 009 | Revoke the mentor DM membership when the internship ends or the employer tenant demotes the work binding. | `workplace-integration` or tenancy closeout triggers `DELETE /channels/{channel_id}/members/{user_ref}` (`removeChannelMember`); actor is `ChannelAdmin` with `Action::"remove_channel_member"`. | Emits `channel-member-changed` with `action = revoked`; subsequent `read_message`, `read_thread`, and `read_presence` attempts fail the membership checks. | Matches Slack external member removal and Teams guest revocation, with revocation evidence on the event bus. |
| 010 | Archive the internship channel and preserve only Professional-context records covered by the employer retention policy. | Internship completion triggers `DELETE /channels/{channel_id}` (`archiveChannel`) after any hold decision; actor is `ChannelAdmin`. | Sets `Channel.archived_at`; archived-channel forbid rules block new posts, while retention/legal-hold flows remain under `POST /holds` and audit-chain sealing. | Matches Teams channel archive and Slack channel archive, with the Personal/Professional boundary kept explicit for student privacy. |

## Dependencies and non-goals

- Depends on community through a typed contract only; no shared table or hidden callback is allowed.
- Depends on identity through a typed contract only; no shared table or hidden callback is allowed.
- Depends on workplace-integration through a typed contract only; no shared table or hidden callback is allowed.
- Depends on payments through a typed contract only; no shared table or hidden callback is allowed.
- Depends on calendar through a typed contract only; no shared table or hidden callback is allowed.
- messenger does not own stipend payment, interview scheduling, SCIM provisioning, ontology projection, or marketplace settlement rows.

## Done evidence

- Journey README links this IP from docs/user-journeys/j113-cross-tenant-internship-from-handshake/README.md.
- Integration test plan names messenger in at least one positive and one failure-injection case.
- Schema docs include the fields this service owns for j113.
- Multispectrum evidence records the doc-only change class.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/messenger/IP-journey-j113-cross-tenant-dm-boundary.md` matched `SLO, payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), KR-PIPA-2023-amendment(14400s/900s), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/messenger/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/messenger/slos/attachment-scan-freshness.openslo.yaml`, `microservices/messenger/slos/mention-fanout.openslo.yaml`, `microservices/messenger/slos/message-send-availability.openslo.yaml`, `microservices/messenger/slos/message-send-latency.openslo.yaml`, `microservices/messenger/policy/auditor-scope.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/messenger/IP-journey-j113-cross-tenant-dm-boundary.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/messenger/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
