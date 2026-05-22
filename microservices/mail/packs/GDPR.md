---
doc_class: CompliancePackOverlay
pack_id: EU-GDPR-2018-baseline
microservice: mail
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# mail GDPR Compliance Pack Overlay

## Pack Identity
- Full pack name: EU GDPR mail privacy and data-subject-rights overlay.
- Citing jurisdiction: European Union and EEA personal-data regime.
- Version: EU-GDPR-2018-baseline-v1.
- Canonical source URL: https://eur-lex.europa.eu/eli/reg/2016/679/oj
- Cited law: Regulation (EU) 2016/679.
- Covered mail surface: mailboxes, aliases, contacts, message bodies, headers, attachments, search index, retention rules, exports, and erasure workflows.
- Pack activation means mail treats mailbox content as personal data unless a narrower data-class proof exists.
- The overlay distinguishes controller tenant mail, processor-hosted mail, and personal mail.
- Data classes added here include `MAIL_PERSONAL_DATA_EU`, `MAIL_SPECIAL_CATEGORY_EU`, and `MAIL_PORTABILITY_EXPORT_EU`.
- GDPR lawful basis is captured per message operation, not only per mailbox.
- The overlay adds DSAR, erasure, restriction, portability, objection, and breach notification behavior.
- ADR-0064 keeps the canonical mail model neutral while this pack injects EU policy and retention deltas.
- ADR-0251 supplies pack activation, cell eligibility, regulator deadlines, and bundle signature rules.
- ADR-0263 requires personal-data scrubbing in telemetry and audit linkage for state changes.
- This overlay excludes PCI-DSS cardholder-data obligations because mail does not own payment acceptance.

## Data Model Deltas
- Add `message.eu_personal_data_signal` as enum `none|personal|special_category`.
- Add `message.lawful_basis` as enum `consent|contract|legal_obligation|vital_interests|public_task|legitimate_interests`.
- Add `message.lawful_basis_evidence_id` as nullable evidence reference.
- Add `message.data_subject_ids_hash` for DSAR fanout without raw identifiers.
- Add `message.erasure_state` as enum `active|restricted|erasure_pending|tombstoned`.
- Add `message.restriction_reason` for Article 18 restriction.
- Add `message.portability_included` as boolean.
- Add `message.processing_purpose_id` linked to tenant register of processing.
- Add `message.transfer_mechanism` as enum `none|scc|adequacy|derogation`.
- Add `message.eu_residency_cell` copied from tenancy placement.
- Add `attachment.eu_special_category_signal` for health, union, biometric, or similar data.
- Add `attachment.portability_format` default `eml`.
- Add `mailbox.controller_processor_role` as enum `controller|processor|joint_controller`.
- Add `mailbox.dpo_contact_ref` for user-facing notices.
- Add `mailbox.default_retention_basis` for tenant retention schedule.
- Add `mailbox.dsar_export_cursor` for resumable exports.
- Add `mailbox.erasure_hold_reason` for legal hold conflicts.
- Add `search_index.eu_personal_data_scrubbed_at` timestamp.
- Add `search_index.rebuild_required_for_erasure` boolean.
- Add `alias.identity_disclosure_level` as enum `opaque|display_name|full_identity`.
- Add `consent_snapshot.mail_processing_consent_id` for consent-based messaging.
- Add `breach_candidate.eu_notification_clock_started_at`.
- Add `audit_shadow.gdpr_event_id` for audit-chain linkage.
- Add `tenant_mail_config.eu_dpa_version` for processor terms proof.

## Cedar Policy Deltas
- Policy `GDPR-mail-read-01`: permit read when `tenant.has_pack("EU-GDPR-2018-baseline") && purpose.allowed == true`.
- Policy `GDPR-mail-read-02`: forbid read when `message.erasure_state == "restricted" && principal.role != "dpo"`.
- Policy `GDPR-mail-send-01`: require lawful basis before outbound processing.
- Policy `GDPR-mail-send-02`: forbid special-category send unless Article 9 condition evidence exists.
- Policy `GDPR-mail-export-01`: permit DSAR export when `principal.subject_id_hash in message.data_subject_ids_hash`.
- Policy `GDPR-mail-export-02`: permit controller export when `principal.role == "dpo" && request.dsar_case_id.exists()`.
- Policy `GDPR-mail-erasure-01`: permit tombstone when no legal hold and no overriding retention basis exists.
- Policy `GDPR-mail-erasure-02`: forbid purge when `message.erasure_hold_reason.exists()`.
- Policy `GDPR-mail-restrict-01`: permit restriction on contested accuracy workflow.
- Policy `GDPR-mail-portability-01`: require machine-readable `.eml` or `.mbox` for Article 20.
- Policy `GDPR-mail-transfer-01`: forbid non-EEA route unless `message.transfer_mechanism in ["scc","adequacy","derogation"]`.
- Policy `GDPR-mail-transfer-02`: forbid fallback SMTP path if target cell lacks EU residency proof.
- Policy `GDPR-mail-consent-01`: require consent snapshot when lawful basis is consent.
- Policy `GDPR-mail-consent-02`: forbid processing after consent withdrawal except retention proof.
- Policy `GDPR-mail-ai-01`: require explicit model-touch consent for smart compose on personal data.
- Policy `GDPR-mail-ai-02`: forbid special-category model touch unless high-assurance approval exists.
- Policy `GDPR-mail-search-01`: permit search only within declared processing purpose.
- Policy `GDPR-mail-index-01`: require index rebuild after erasure tombstone.
- Policy `GDPR-mail-breach-01`: start breach clock when personal-data exfiltration candidate is confirmed.
- Policy `GDPR-mail-admin-01`: require DPO-visible audit for administrator mailbox access.
- Policy `GDPR-mail-alias-01`: permit alias disclosure only at configured identity disclosure level.
- Policy `GDPR-mail-webhook-01`: forbid webhook export without processor DPA.
- Policy `GDPR-mail-retention-01`: require purpose-bounded retention, not blanket indefinite retention.
- Policy `GDPR-mail-objection-01`: forbid direct-marketing mail processing after objection flag.

## API Contract Deltas
- `POST /messages/send` requires `lawful_basis` for EU pack tenants.
- `POST /messages/send` accepts `lawful_basis_evidence_id` when basis needs proof.
- `POST /messages/send` rejects special-category content without Article 9 evidence.
- `GET /messages/{id}` masks restricted messages for non-DPO roles.
- `POST /dsar/export` starts mailbox-scoped Article 15 and 20 export.
- `GET /dsar/export/{id}` returns `.eml` and `.mbox` manifests.
- `POST /dsar/erasure` starts tombstone workflow for subject-linked messages.
- `POST /dsar/restrict` marks messages restricted during accuracy dispute.
- `POST /consent/withdraw` triggers processing stop for consent-based mail.
- `POST /search/rebuild` accepts erasure-driven rebuild reason.
- `POST /rules/retention` requires purpose and lawful basis.
- `POST /webhooks` requires processor DPA reference.
- `POST /ai/draft` requires model-touch consent flag.
- `POST /aliases` requires identity disclosure level.
- `POST /mailboxes/import` records source transfer mechanism.
- `GET /processing-register/mail` exposes mail processing purpose rows.
- `POST /breach-candidates` starts 72-hour GDPR assessment clock.
- `GET /audit/admin-access` exposes DPO-visible admin-read events.
- `DELETE /messages/{id}` returns legal-hold conflict details.
- `PATCH /tenant-mail-config` requires EU DPA version.

## Workflow Deltas
- Outbound send validates lawful basis before SMTP routing.
- Special-category classifier runs before send and index.
- Consent withdrawal triggers mail processing restriction.
- DSAR export enumerates messages by subject hash.
- DSAR export generates `.eml` and `.mbox` with manifest hash.
- Erasure request tombstones bodies and rewrites indexes.
- Legal hold conflict routes to DPO review.
- Restriction workflow hides messages from normal search.
- Direct marketing objection disables mail campaign processing.
- Processor DPA workflow must complete before webhook export.
- EU route planner blocks non-EEA fallbacks without transfer proof.
- AI draft workflow records model-touch consent.
- Admin mailbox access creates DPO-visible review task.
- Alias creation evaluates identity disclosure level.
- Import workflow captures origin transfer mechanism.
- Breach candidate workflow starts Article 33 clock.
- Cross-tenant mail share checks controller and processor roles.
- Retention schedule changes create processing-register update.
- Search index rebuild is mandatory after erasure tombstone.
- Pack deactivation waits for all DSAR cases to settle.

## SLO Deltas
- GDPR breach regulator-notification readiness p99 target is <= 60 hours.
- Breach clock creation p99 must complete <= 5 minutes.
- DSAR export first response target is <= 7 days.
- Full DSAR export completion target is <= 30 days.
- Erasure tombstone p99 target is <= 72 hours after approval.
- Restriction activation p99 must complete <= 15 minutes.
- Consent withdrawal propagation p99 must complete <= 30 minutes.
- Search index erasure rebuild p99 target is <= 24 hours.
- Lawful-basis send preflight p99 must stay <= 500 ms.
- EU route residency validation p99 must stay <= 100 ms.
- Admin access audit emission p99 must complete <= 1 second.
- Direct marketing objection enforcement p99 must complete <= 10 minutes.
- Processor DPA lookup p99 must complete <= 200 ms.
- Portability manifest generation p99 target is <= 4 hours.
- Special-category classifier review cadence is daily.
- DPO dashboard lag target is <= 15 minutes.

## Audit-event class additions
- `MailGdprLawfulBasisChecked` records basis and evidence reference.
- `MailGdprSpecialCategoryBlocked` records classifier and policy id.
- `MailGdprDsarExportStarted` records case id and mailbox scope.
- `MailGdprDsarExportCompleted` records manifest hash.
- `MailGdprErasureRequested` records subject hash.
- `MailGdprMessageTombstoned` records message id and reason.
- `MailGdprRestrictionApplied` records Article 18 reason.
- `MailGdprRestrictionReleased` records reviewer id.
- `MailGdprConsentWithdrawn` records consent id.
- `MailGdprSearchIndexRebuilt` records rebuild reason.
- `MailGdprTransferMechanismChecked` records route and mechanism.
- `MailGdprWebhookDpaRejected` records destination id.
- `MailGdprAiDraftConsentChecked` records model surface.
- `MailGdprAdminAccessReviewed` records DPO review id.
- `MailGdprBreachClockStarted` records candidate id.
- `MailGdprObjectionApplied` records processing purpose.
- `MailGdprRetentionScheduleChanged` records old and new schedule.
- `MailGdprAliasDisclosureChanged` records disclosure level.
- `MailGdprImportTransferRecorded` records origin mechanism.
- `MailGdprPackDeactivationDeferred` records open cases count.

## Failure Modes specific to this pack
- Lawful basis is missing; recovery is reject send and prompt tenant admin for processing purpose.
- Special-category classifier is down; recovery is fail-closed for outbound and queue draft.
- Consent store is stale; recovery is block consent-based processing until snapshot refresh.
- DSAR subject hash fanout is incomplete; recovery is rerun from audit-chain cursor.
- Search index cannot tombstone; recovery is remove index shard from serving and rebuild.
- Legal hold conflicts with erasure; recovery is restrict processing and route DPO decision.
- Transfer mechanism expires; recovery is block non-EEA route and re-evaluate SCC.
- DPA registry lookup fails; recovery is block webhook exports.
- AI draft consent is ambiguous; recovery is block model touch.
- Admin reads mailbox without DPO case; recovery is revoke session and open incident.
- Direct marketing objection races send; recovery is cancel queued campaign mail.
- Portability export exceeds file size; recovery is chunk export with signed manifest.
- Erasure request targets shared mailbox; recovery is split by data-subject participation.
- Imported mailbox lacks origin proof; recovery is quarantine import and request transfer basis.
- Breach candidate clock not started; recovery is create retroactive audit event and page compliance.
- Alias disclosure configuration drifts; recovery is default to opaque alias.
- Retention rule is indefinite; recovery is reject schedule and keep previous lawful schedule.
- Processor role is wrong; recovery is freeze exports until tenant role fixed.
- Pack deactivation is requested with open DSAR cases; recovery is defer deactivation.
- EU cell outage suggests US failover; recovery is queue mail until eligible cell returns.

## Cross-µservice coordination
- `tenancy` provides EU cell placement, tenant role, and active pack roster.
- `identity` provides data-subject identifiers and DPO role claims.
- `compliance` owns processing register, DPA registry, and DSAR case state.
- `audit-chain` seals every DSAR, erasure, restriction, and admin-read event.
- `observability` scrubs personal data before metrics, logs, traces, and exemplars.
- `drive` applies GDPR overlay when mail attachments are exported to drive.
- `workflow-engine` orchestrates DSAR, erasure, breach, and objection workflows.
- `policy-engine` loads all `GDPR-mail-*` fragments under pack scope.
- `search` rebuilds indexes after erasure and restriction changes.
- `notification` avoids personal data in breach or DSAR status previews.
- `data-warehouse` receives only aggregate mail metrics for EU tenants.
- `admin-console` surfaces lawful-basis and retention configuration.
- `dlp-virus-scan` provides special-category verdicts with provenance.
- `incident-response` consumes breach candidates and deadline clocks.
- `legal` reviews erasure conflicts with legal holds.
- `localization` provides EU language notices where tenant requires them.
- `connector` validates external processor destination DPA status.
- `records` receives only opaque subject references from mail.
- `support` access must be DPO-visible and purpose-bounded.
- `pack-registry` signs the EU-GDPR mail overlay bundle.
