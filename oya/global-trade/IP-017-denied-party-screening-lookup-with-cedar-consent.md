---
doc_class: ImplementationPlan
ip_id: IP-017
microservice: global-trade
related_adrs:
  - ADR-0105
  - ADR-0243
  - ADR-0244
  - ADR-0253
  - ADR-0263
  - ADR-0304
  - ADR-0315
  - ADR-0329
  - ADR-0330
  - ADR-0331
journey_id: j104-supplier-vendor-onboarding-kyb-cascade
journey_link: docs/user-journeys/j104-supplier-vendor-onboarding-kyb-cascade/story.md
status: Accepted
date: 2026-05-20
owner: axis-global-trade
tenant_class_eligibility: [demo_trial, paid]
sap_submodule_equivalents:
  - SAP GTS-CC sanctioned-party-list-screening
  - SAP GTS-EM screening-result-distribution
---

# IP-017: Denied party screening lookup with Cedar consent

## Context With Why
- This IP builds a denied party screening lookup that checks parties against sanctioned, denied, and restricted-party lists.
- The why is regulatory: a tenant must stop risky transactions before shipment, payment, or broker filing proceeds.
- The feature must be consent-aware because screening can expose sensitive party attributes and watchlist match details.
- The journey leg starts when a party is introduced by order capture, supplier onboarding, customs declaration, or broker filing.
- The journey leg ends when the party has a screening result, match disposition, consent record, and audit-chain evidence.
- Named persona: Julian, a compliance operations reviewer, needs to clear false positives without seeing unrelated tenant data.
- Julian can review match evidence for his assigned region, but he cannot export full watchlist payloads without auditor consent.
- This IP maps SAP GTS-CC sanctioned party list screening into a standalone Oyatie service boundary.
- SAP GTS-EM is relevant because screening results must move to ERPs, brokers, workflow, and hold-release channels.
- The implementation must not own customer, supplier, or employee master data.
- The implementation stores screening snapshots, match decisions, consent references, and list version references.
- Screening must run before customs declaration approval and before broker EDI submission when configured by tenant policy.
- Cedar consent is not a UI checkbox; it is a policy-enforced decision around who can view, resolve, export, or suppress match evidence.
- ADR-0105 keeps list adapters separate from domain match decisions.
- ADR-0243 requires list-source provenance and import checksums.
- ADR-0244 requires tenant and sub-scope isolation on every screening row.
- ADR-0253 requires Cedar default-deny for lookup, evidence read, disposition, and export actions.
- ADR-0263 requires every match and disposition to create chainable audit events.
- ADR-0304 requires ontology projection of ScreeningSubject, DeniedPartyHit, and ConsentGrant.
- ADR-0315 sets the SAP parity target for sanctioned-party screening.
- Intern build target: one lookup aggregate, one match scorer, one consent policy hook, one disposition workflow, and one result export worker.

## Scope Boundaries
- In scope: party lookup, fuzzy match scoring, list version pinning, hit creation, false-positive disposition, consent-gated evidence access.
- In scope: party snapshots from customers, suppliers, brokers, employees, banks, carriers, and consignee records.
- In scope: list imports from OFAC, EU, UN, BIS, local customs sources, and tenant private denied lists.
- In scope: workflow handoff for review and hold-release.
- Out of scope: customer master editing, supplier master editing, payment release, and final export license determination.
- Out of scope: global identity resolution across tenants.
- Out of scope: watchlist scraping that violates source license terms.
- Boundary rule: source party records are immutable snapshots in this service.
- Boundary rule: a disposition can clear a transaction, but it cannot delete the original hit.
- Boundary rule: consent grants are evaluated through Cedar and recorded as resources, not local boolean flags.
- Boundary rule: list import failures must not erase the prior active list version.

## Data Model Deltas
- Table: `global_trade_screening_subject_snapshot`.
- Column: `tenant_id uuid not null`.
- Column: `subject_id uuid primary key`.
- Column: `source_system_ref text not null`.
- Column: `source_party_ref text not null`.
- Column: `subject_type text not null check subject_type in ('customer','supplier','carrier','broker','bank','employee','consignee','ship_to','bill_to')`.
- Column: `legal_name text not null`.
- Column: `alternate_names jsonb not null default '[]'`.
- Column: `country_code text null`.
- Column: `address_fingerprint text null`.
- Column: `date_of_birth date null`.
- Column: `registration_number text null`.
- Column: `snapshot_hash text not null`.
- Column: `idempotency_key text not null`.
- Unique: `gt_screening_subject_source_uq` on `(tenant_id, source_system_ref, source_party_ref, snapshot_hash)`.
- Table: `global_trade_denied_party_screening_result`.
- Column: `tenant_id uuid not null`.
- Column: `screening_id uuid primary key`.
- Column: `subject_id uuid not null references global_trade_screening_subject_snapshot(subject_id)`.
- Column: `list_version_ref text not null`.
- Column: `screening_state text not null check screening_state in ('clear','potential_hit','confirmed_hit','false_positive','expired','error')`.
- Column: `highest_score numeric(5,4) not null`.
- Column: `policy_bundle_version text not null`.
- Column: `consent_resource_ref text not null`.
- Column: `audit_chain_ref text not null`.
- Column: `screened_at timestamptz not null`.
- Index: `gt_screening_result_subject_idx` on `(tenant_id, subject_id, screened_at desc)`.
- Table: `global_trade_denied_party_hit_candidate`.
- Column: `tenant_id uuid not null`.
- Column: `hit_candidate_id uuid primary key`.
- Column: `screening_id uuid not null references global_trade_denied_party_screening_result(screening_id)`.
- Column: `watchlist_entry_ref text not null`.
- Column: `watchlist_program text not null`.
- Column: `match_score numeric(5,4) not null`.
- Column: `match_reasons jsonb not null`.
- Column: `evidence_redaction_level text not null check evidence_redaction_level in ('summary','reviewer','auditor','source_full')`.
- Column: `disposition_state text not null check disposition_state in ('open','false_positive','confirmed','suppressed','escalated')`.
- Table: `global_trade_screening_consent_grant`.
- Column: `tenant_id uuid not null`.
- Column: `consent_id uuid primary key`.
- Column: `principal_id text not null`.
- Column: `resource_scope text not null`.
- Column: `allowed_action text not null`.
- Column: `expires_at timestamptz not null`.
- Column: `granted_by_principal text not null`.
- Column: `grant_reason text not null`.
- Retention: subject snapshots and hits are immutable; dispositions append status events.
- Retention: consent grants expire and are never silently extended.

## API Endpoints
- REST: `POST /v1/global-trade/denied-party-screenings:lookup`.
- REST request example:
```json
{
  "tenant_id": "ten_usa_001",
  "principal_id": "usr_order_ops",
  "idempotency_key": "dps-lookup-2026-05-20-44",
  "source_system_ref": "order:na:checkout",
  "source_party_ref": "customer:88401",
  "subject": {
    "subject_type": "customer",
    "legal_name": "North Harbor Industrial LLC",
    "alternate_names": ["N Harbor Industrial"],
    "country_code": "US",
    "registration_number": "US-DE-4410"
  },
  "purpose": "shipment_release"
}
```
- REST response example:
```json
{
  "screening_id": "scr_01jytg_dps_017",
  "screening_state": "potential_hit",
  "highest_score": 0.9142,
  "consent_resource_ref": "consent_scope:screening:scr_01jytg_dps_017",
  "redacted_hits": [
    {
      "hit_candidate_id": "hit_017_a",
      "watchlist_program": "OFAC-SDN",
      "match_score": 0.9142,
      "evidence_redaction_level": "summary"
    }
  ],
  "audit_event_class": "EVT-GLOBAL_TRADE-DENIED_PARTY_SCREENING-POTENTIAL_HIT"
}
```
- REST: `POST /v1/global-trade/denied-party-screenings/{screening_id}:grant-consent`.
- REST: `POST /v1/global-trade/denied-party-screenings/{screening_id}:dispose-hit`.
- REST: `GET /v1/global-trade/denied-party-screenings/{screening_id}`.
- REST: `GET /v1/global-trade/denied-party-screenings/{screening_id}/evidence`.
- gRPC: `LookupDeniedParty(LookupDeniedPartyRequest) returns (LookupDeniedPartyResult)`.
- gRPC: `GrantScreeningConsent(GrantScreeningConsentRequest) returns (GrantScreeningConsentResult)`.
- gRPC: `DisposeDeniedPartyHit(DisposeDeniedPartyHitRequest) returns (DisposeDeniedPartyHitResult)`.
- Worker command: `global-trade.denied-party.import-watchlist-version`.
- Worker command: `global-trade.denied-party.expire-consent-grants`.
- Error envelope: `POLICY_DENIED`, `CONSENT_REQUIRED`, `LIST_VERSION_UNAVAILABLE`, `MATCH_ENGINE_TIMEOUT`, `AUDIT_CHAIN_SEAL_FAILED`.

## Cedar Policy Hooks
- Principal: `GlobalTrade::Principal::"usr_order_ops"`.
- Action: `GlobalTrade::Action::"LookupDeniedParty"`.
- Resource: `GlobalTrade::ScreeningSubject::"customer:88401"`.
- Context field: `tenant_id`.
- Context field: `purpose`.
- Context field: `consent_resource_ref`.
- Context field: `redaction_level_requested`.
- Context field: `list_version_ref`.
- Context field: `source_system_ref`.
- Consent action: `GlobalTrade::Action::"ReadScreeningEvidence"`.
- Consent resource: `GlobalTrade::ConsentGrant::"consent_scope:screening:scr_01jytg_dps_017"`.
- Allow rule intent: order operations can request lookup and see clear or summary result.
- Allow rule intent: compliance reviewers can see reviewer evidence for assigned tenant and region.
- Allow rule intent: auditors can see full source evidence with active consent grant.
- Deny rule intent: no principal can suppress a hit without `DisposeDeniedPartyHit` and an unexpired consent grant.
- Deny rule intent: source watchlist payload export is denied unless auditor role and source license allow it.
- Deny rule intent: CI principals can read fixtures only when `data_class` is synthetic.
- Audit on allow: include consent id, grant expiration, and redaction level.
- Audit on deny: include action and resource but redact watchlist entry details.

## Ontology Projection Field Mapping
- Ontology node: `ScreeningSubject`.
- `subject_id` maps to `ScreeningSubject.id`.
- `subject_type` maps to `ScreeningSubject.type`.
- `legal_name` maps to `ScreeningSubject.legalName`.
- `alternate_names` maps to `ScreeningSubject.aliases`.
- `source_party_ref` maps to `SourceLineage.sourceEntityRef`.
- `snapshot_hash` maps to `EvidenceSnapshot.hash`.
- Ontology node: `DeniedPartyHit`.
- `screening_id` maps to `DeniedPartyHit.screeningId`.
- `watchlist_entry_ref` maps to `DeniedPartyHit.watchlistEntryRef`.
- `watchlist_program` maps to `DeniedPartyHit.program`.
- `match_score` maps to `DeniedPartyHit.score`.
- `match_reasons` maps to `DeniedPartyHit.reasons`.
- `disposition_state` maps to `DeniedPartyHit.disposition`.
- Ontology node: `ConsentGrant`.
- `consent_id` maps to `ConsentGrant.id`.
- `principal_id` maps to `ConsentGrant.grantee`.
- `allowed_action` maps to `ConsentGrant.allowedAction`.
- `expires_at` maps to `ConsentGrant.expiresAt`.
- Projection mode: write ScreeningSubject first, then DeniedPartyHit, then ConsentGrant edge.
- Projection guard: do not project source-full watchlist evidence to ontology.

## Workflow Steps
- Node `ReceiveScreeningSubject`: validate tenant, source, purpose, idempotency, and subject fields.
- Node `SnapshotSubject`: canonicalize names and create immutable snapshot hash.
- Node `LoadActiveWatchlists`: load tenant-enabled list versions.
- Branch `NoActiveList`: return configuration error and emit event.
- Node `RunMatchEngine`: score exact, phonetic, transliteration, address, and registration matches.
- Branch `ClearBelowThreshold`: create clear result and release downstream hold.
- Branch `PotentialHit`: create hit candidates and workflow review task.
- Branch `ConfirmedHighRisk`: create confirmed hit and block shipment or filing handoff.
- Node `EvaluateCedarConsent`: decide visible evidence level for requester.
- Node `PersistScreeningResult`: write subject, result, candidates, and outbox events.
- Node `SealAuditEvent`: append ADR-0263 event class with prior hash.
- Node `ProjectOntology`: project subject and redacted hit state.
- Node `NotifyWorkflowEngine`: assign review to compliance queue.
- Node `ExportResult`: publish summary result to customs-declaration, broker-filing, or supplier workflow.
- Branch `ConsentExpired`: downgrade evidence response and require new grant.
- Branch `ListImportInProgress`: use prior active list and flag freshness warning.
- Branch `MatchEngineTimeout`: create error result and retry if purpose allows asynchronous release.

## Audit Events
- `EVT-GLOBAL_TRADE-DENIED_PARTY_SCREENING-LOOKUP_REQUESTED`.
- `EVT-GLOBAL_TRADE-DENIED_PARTY_SCREENING-SUBJECT_SNAPSHOTTED`.
- `EVT-GLOBAL_TRADE-DENIED_PARTY_SCREENING-CLEAR_RESULT`.
- `EVT-GLOBAL_TRADE-DENIED_PARTY_SCREENING-POTENTIAL_HIT`.
- `EVT-GLOBAL_TRADE-DENIED_PARTY_SCREENING-CONFIRMED_HIT`.
- `EVT-GLOBAL_TRADE-DENIED_PARTY_SCREENING-FALSE_POSITIVE_DISPOSED`.
- `EVT-GLOBAL_TRADE-DENIED_PARTY_SCREENING-CONSENT_GRANTED`.
- `EVT-GLOBAL_TRADE-DENIED_PARTY_SCREENING-CONSENT_EXPIRED`.
- `EVT-GLOBAL_TRADE-DENIED_PARTY_SCREENING-EVIDENCE_READ`.
- `EVT-GLOBAL_TRADE-DENIED_PARTY_SCREENING-POLICY_DENIED`.
- `EVT-GLOBAL_TRADE-DENIED_PARTY_SCREENING-AUDIT_SEALED`.
- Event fields: `event_id`, `event_class`, `tenant_id`, `principal_id`, `resource_id`, `consent_id`, `redaction_level`, `occurred_at`, `prev_event_hash`, `event_hash`.
- Event rule: evidence-read events record redaction level, not raw watchlist payload.
- Event rule: false-positive disposition includes reviewer id, reason code, and evidence hash.

## SLO Targets
- Availability target: 99.97 percent monthly for lookup endpoint.
- Throughput target: 600 screening lookups per second per region.
- p50 latency target: 60 ms for clear lookup against cached active lists.
- p95 latency target: 300 ms for fuzzy matching with five watchlist programs.
- p99 latency target: 900 ms for transliteration and address-expanded matching.
- Watchlist promotion target: active list version promoted within 15 minutes of validated import.
- Consent expiration target: expired grants removed from positive authorization path within 60 seconds.
- Audit seal target: 99.99 percent first-attempt seal rate.
- Rationale: order checkout and shipment release need fast clear results, while potential hits can move to workflow.
- Burn alert: page when p99 exceeds 1200 ms or clear-result error rate exceeds 0.5 percent for 10 minutes.

## Failure Modes And Recovery
- Failure: watchlist import checksum mismatch; recovery: reject new version and keep prior active version.
- Failure: match engine timeout; recovery: create retryable error and block downstream release for high-risk purposes.
- Failure: consent grant expired during evidence read; recovery: return consent required and emit expired event.
- Failure: reviewer tries to export source-full evidence; recovery: Cedar deny and redacted audit event.
- Failure: duplicate source party snapshot; recovery: reuse prior subject snapshot and create new screening result if list version changed.
- Failure: false-positive disposition conflicts with confirmed hit; recovery: require supervisor override workflow.
- Failure: ontology projection fails; recovery: retry projection and keep screening result queryable.
- Failure: audit chain seal fails; recovery: rollback result write and retry lookup from source event.
- Failure: watchlist source license blocks raw export; recovery: expose summary reason only.
- Failure: tenant threshold is malformed; recovery: fail closed with configuration error.

## Migration Notes With Source Vendor Surfaces
- SAP source: SAP GTS sanctioned party list screening hits and partner screening history.
- SAP source: SAP GTS electronic messaging outputs for screening holds and releases.
- Oracle source: GTM restricted party screening records.
- Descartes source: denied-party screening match export.
- Amber Road source: restricted party list screening and disposition history.
- Dow Jones or Refinitiv source: watchlist entry references when tenant has licensed data.
- Migration step: import historical subjects as immutable snapshots.
- Migration step: import historical dispositions with reviewer and date provenance when available.
- Migration step: map legacy false positive codes to tenant reason codes.
- Migration step: create synthetic consent grants only for audit migration jobs, never for human users.
- Migration step: hash raw watchlist payloads and store only licensed payload fields.
- Migration step: replay ten high-risk historical hits before enabling automatic release.

## Cross-Microservice Handoffs
- To customs-declaration: clear, potential hit, or confirmed hit status by party role.
- To broker-filing: filing block or release summary with audit reference.
- To workflow-engine: potential hit review, supervisor override, and consent grant tasks.
- To ontology: ScreeningSubject, DeniedPartyHit, and ConsentGrant projections.
- To audit-chain: ADR-0263 event append and chain verification.
- To notification: reviewer queue assignment and expired consent notices.
- To data-residency: watchlist payload storage and evidence redaction policy.
- To marketplace: entitlement check for premium watchlist programs only.
- To product or supplier master services: read-only release result by source party ref.
- To observability: lookup latency, match count, deny spikes, consent expirations, and list freshness.

## Implementation Checklist
- Add domain aggregate `DeniedPartyScreeningResult`.
- Add domain entity `ScreeningSubjectSnapshot`.
- Add domain entity `DeniedPartyHitCandidate`.
- Add domain entity `ScreeningConsentGrant`.
- Add value object `WatchlistVersionRef`.
- Add value object `MatchReason`.
- Add match scorer interface with exact, fuzzy, phonetic, transliteration, and address signals.
- Add repository for subject snapshots.
- Add repository for screening results.
- Add repository for consent grants.
- Add transaction boundary for subject, result, candidates, consent outbox, and audit event.
- Add command handler for lookup.
- Add command handler for grant consent.
- Add command handler for dispose hit.
- Add Cedar authorization calls for lookup, evidence read, disposition, and export.
- Add REST route for lookup.
- Add REST route for evidence read.
- Add REST route for consent grant.
- Add REST route for hit disposition.
- Add gRPC method for internal lookup.
- Add worker for watchlist import.
- Add worker for consent expiration.
- Add ontology projection writer with redaction guard.
- Add audit event appender using ADR-0263 class names.
- Add fixture for clear screening result.
- Add fixture for potential hit with redacted evidence.
- Add fixture for confirmed hit.
- Add fixture for expired consent.
- Add policy tests for operator, reviewer, auditor, CI, and broker principals.
- Add unit tests for match scoring thresholds.
- Add unit tests for consent grant expiration.
- Add contract tests for lookup request and response.
- Add replay tests for duplicate subject snapshots.
- Add migration tests for SAP GTS hit history.
- Add negative test proving source-full export is denied without auditor consent.
- Add negative test proving raw watchlist payload is not projected to ontology.
- Add performance test for 1 million watchlist entries in cached lookup mode.
- Add dashboard panels for p50, p95, p99, throughput, active list age, and consent denies.
- Add runbook link for watchlist import checksum mismatch.
- Add evidence bundle for auditor replay.
- Add acceptance evidence referencing this IP id.
