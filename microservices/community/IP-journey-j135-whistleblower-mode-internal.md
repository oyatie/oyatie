---
doc_class: Implementation-Plan
ip_id: IP-journey-j135-whistleblower-mode-internal
journey_ref: docs/user-journeys/j135-hr-handles-harassment-complaint-with-dual-tenant-boundary/
status: draft
date: 2026-05-20
microservice: community
related_adrs: [ADR-0311, ADR-0244, ADR-0263, ADR-0292]
wave_15_journey_ip_substance: rewritten-2026-05-21
---

# IP - j135 - community - whistleblower-mode-internal

## A. Intent

This implementation plan binds the `community` slice of `j135` to real community contracts, Cedar fragments, event channels, proto3 services, SLOs, and counterpart behavior. The previous generated row loop has been consolidated: only rows with a grounded community action remain, and speculative rows without a backing artifact are removed rather than expanded.

## B. Scope

`community` owns posts, replies, votes, flags, moderation queue state, KB promotion, search reindex requests, public-read boundaries, anonymous-mode boundaries, identity/persona/pseudonymous-mode policy checks, and auditor-visible evidence for this journey. It does not own marketplace settlement, payment movement, identity proofing, workflow orchestration, mail delivery, or compliance-pack authority except through typed references and audit evidence.

## C. Substantive journey rows

| Row | Concrete journey action | Trigger and actor | State effect | Evidence touch | Counterpart equivalence |
|---:|---|---|---|---|---|
| 1 | Accept a whistleblower-mode-internal evidence bundle only through the fully anonymous dropbox flow. | Trigger: encrypted submission; actor: Cedar `FullyAnonymousSubmitter::"public"` using `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`. | Stores ciphertext hash and channel metadata without stable user id; rejects rate-limited resources. | AsyncAPI `community.flag.raised` in `microservices/community/contracts/asyncapi/community-events.yaml` plus capability `microservices/community/capabilities/whistleblower-submission.yaml` when whistleblower-specific. | Matches SecureDrop source submission: ciphertext routed, platform does not deanonymize. |
| 2 | Verify eligibility or channel routing without binding the submitter identity. | Trigger: nonbinding proof or journalist mailbox selection; actor: public submitter or verified triage principal. | Persists only proof hash, mailbox id, and audit id; no author_id enters Post/Thread records. | proto3 `ModerationQueueService.RaiseFlag` in `microservices/community/contracts/proto/community.proto` for safety escalation and `microservices/community/policy/anonymity-mode-fully-anonymous.cedar` permits. | Matches GlobaLeaks/SecureDrop non-attribution intake rather than account login. |
| 3 | Escalate illegal, child-safety, or vulnerability payloads to a tenant moderator queue. | Trigger: abuse classifier or report category; actor: `TenantModerator` after default-deny re-evaluation. | Creates moderation queue item with `target_type=attachment` or `post` and sealed reason. | OpenAPI `raiseFlag/applyModerationAction` in `microservices/community/contracts/openapi/community.yaml` and `microservices/community/capabilities/moderate-action.yaml`. | Matches HackerOne triage handoff: reporter privacy preserved, triager gets actionable case. |
| 4 | Publish only redacted case status back to the anonymous channel. | Trigger: moderation action resolved; actor: moderator with two-eyes approval where destructive. | Emits status token and audit hash, not user identity or raw evidence. | AsyncAPI `community.moderation.actioned` in `microservices/community/contracts/asyncapi/community-events.yaml` and `microservices/community/slos/moderation-action-latency.openslo.yaml`. | Matches ethics hotline update receipt: submitter sees case state, not investigator notes. |
| 5 | Deny cross-tenant or CI reads of raw anonymous evidence. | Trigger: auditor, CI, or tenant member attempts raw read; actor: `Auditor`, `CiAgent`, or `TenantMember`. | No state mutation; emits deny metric and leaves evidence encrypted. | `microservices/community/policy/auditor-scope.cedar` and `microservices/community/policy/ci-scope.cedar` default-deny raw tenant data. | Matches SOC2 evidence-room scoping: auditors see attestations, not source identity. |
| 6 | Reindex only public, approved redacted summaries. | Trigger: publish-redacted-summary command; actor: authorized tenant moderator. | Creates search document with `body_sha256`, no ciphertext, no IP/user-agent. | AsyncAPI `community.search.reindex.requested` in `microservices/community/contracts/asyncapi/community-events.yaml` and `microservices/community/slos/search-query-latency.openslo.yaml`. | Matches public disclosure portals that publish sanitized advisories only. |

## D. Contract and policy bindings

- REST: `microservices/community/contracts/openapi/community.yaml` (`listSpaces`, `createPost`, `postReply`, `castVote`, `acceptAnswer`, `raiseFlag`, `applyModerationAction`, `createKbArticle`, `readKbArticle`).
- Events: `microservices/community/contracts/asyncapi/community-events.yaml` (`community.post.created`, `community.reply.posted`, `community.vote.cast`, `community.answer.accepted`, `community.flag.raised`, `community.moderation.actioned`, `community.kb.article.published`, `community.search.reindex.requested`).
- Proto: `microservices/community/contracts/proto/community.proto` (`PostStoreService`, `ThreadTreeService`, `VotingEngineService`, `ModerationQueueService`, `KbArticleStoreService`, `SearchIndexService`).
- Cedar: `microservices/community/policy/tenant-scope.cedar`, `microservices/community/policy/public-read.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`.

## E. Deleted ungrounded row families

The generated loop rows that varied only by counter, layer label, event id, or repeated acceptance phrase were removed. Where this journey needs a role-specific endpoint or policy not present in the real community artifacts above, the row now names it as a planned journey extension and binds it to this IP plus the matching capability file instead of pretending that an implemented endpoint already exists.

## F. Evidence bindings

| Evidence | Backing artifact | Why it grounds the row |
|---|---|---|
| REST contract | `microservices/community/contracts/openapi/community.yaml` | real community operations referenced by row actions |
| Event contract | `microservices/community/contracts/asyncapi/community-events.yaml` | real CloudEvents channels used for audit and reindex triggers |
| Internal RPC | `microservices/community/contracts/proto/community.proto` | real proto3 services and messages used by worker/internal rows |
| Tenant policy | `microservices/community/policy/tenant-scope.cedar` | default-deny tenant-scoped mutation and moderation checks |
| Fully anonymous policy | `microservices/community/policy/anonymity-mode-fully-anonymous.cedar` | specific Cedar principals/actions for dropbox intake |
| Anonymous capability | `microservices/community/capabilities/whistleblower-submission.yaml` | planned capability binding for whistleblower submission |

## G. Verification

- Re-run the Wave 15 journey-IP loop scan on this file; there should be no 30+ repeated row labels.
- Resolve every referenced artifact path before implementation.
- Add contract tests against OpenAPI/AsyncAPI/proto before promoting any planned journey-specific extension.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/community/IP-journey-j135-whistleblower-mode-internal.md` matched `openapi, asyncapi, .proto`; contract files `microservices/community/contracts/openapi/community.yaml, microservices/community/contracts/asyncapi/community-events.yaml, microservices/community/contracts/proto/community.proto`; type anchor `microservices/community/manifest.json`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/community/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/community/IP-journey-j135-whistleblower-mode-internal.md` matched `SLO, payment`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/community/IP-journey-j135-whistleblower-mode-internal.md` matched `attribution`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.
