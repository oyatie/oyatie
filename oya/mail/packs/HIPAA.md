---
doc_class: CompliancePackOverlay
pack_id: HIPAA-2024
microservice: mail
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# mail HIPAA Compliance Pack Overlay

## Pack Identity
- Full pack name: HIPAA Administrative Simplification mail overlay.
- Citing jurisdiction: United States federal health information regime.
- Version: HIPAA-2024-v1.
- Canonical source URL: https://www.ecfr.gov/current/title-45/subtitle-A/subchapter-C
- Cited law: 45 CFR Parts 160, 162, and 164.
- Covered mail surface: work mailboxes, shared mailboxes, aliases, inbound attachments, outbound messages, DLP holds, legal hold exports, and eDiscovery search.
- Pack activation means the tenant may transmit ePHI through mail only when the tenant has a signed BAA and a HIPAA-eligible cell.
- This overlay does not make personal mail HIPAA-covered; personal mail is refused for tenant PHI workflows.
- HIPAA terms map to mail data classes `MAIL_BODY_PHI`, `MAIL_ATTACHMENT_PHI`, `MAIL_HEADER_LIMITED_PHI`, and `MAIL_AUDIT_PHI`.
- Minimum necessary is enforced before send, forward, search, export, and AI-draft creation.
- The overlay composes with ADR-0064 canonical base by adding pack policy fragments, not by forking mail domain models.
- The overlay composes with ADR-0251 by adding cell eligibility, audit retention, breach workflow, and Cedar fragments.
- The overlay composes with ADR-0263 by tightening scrubbing and requiring audit ids on every PHI-touching emission.
- The mail service remains responsible for message-state transitions; compliance owns pack registry and breach workflows.
- The overlay is inactive until `tenant.compliance_packs` contains `HIPAA-2024`.

## Data Model Deltas
- Add `message.phi_signal` as enum `none|possible|confirmed`.
- Add `message.phi_basis` as enum `treatment|payment|operations|patient_request|none`.
- Add `message.minimum_necessary_scope` as array of allowed recipient roles.
- Add `message.baa_required` as boolean derived from tenant pack activation.
- Add `message.baa_verified_at` as nullable timestamp copied from compliance pack admission.
- Add `message.patient_context_id` as nullable opaque reference, never patient name.
- Add `message.break_glass_reason_id` for emergency override linkage.
- Add `message.forwarding_phi_risk` as score stored only in tenant region.
- Add `message.ai_draft_phi_blocked` as boolean for model-touch prevention.
- Add `message.disclosure_accounting_required` as boolean.
- Add `attachment.phi_scan_verdict` as enum `clear|possible_phi|confirmed_phi|scan_failed`.
- Add `attachment.encrypted_phi_blob_ref` for envelope-encrypted attachment body.
- Add `attachment.quarantine_bucket_ref` for blocked inbound PHI.
- Add `attachment.openbao_dek_ref` for tenant-owned data key.
- Add `mailbox.hipaa_role_scope` for covered workforce role mapping.
- Add `mailbox.shared_phi_allowed` as boolean requiring approval.
- Add `mailbox.auto_forward_phi_blocked` as boolean default true.
- Add `mailbox.retention_floor_iso8601` default `P6Y` for HIPAA evidence.
- Add `mail_thread.phi_participant_set_hash` for accounting of disclosures.
- Add `mail_thread.patient_request_marker` for patient-requested communications.
- Add `dlp_verdict.hipaa_rule_id` for classifier decision traceability.
- Add `audit_shadow.mail_phi_event_id` for audit-chain correlation.
- Add `export_job.phi_manifest_hash` for eDiscovery export evidence.
- Add `tenant_mail_config.hipaa_cell_certification` requiring `hipaa-certified`.

## Cedar Policy Deltas
- Policy `HIPAA-mail-send-01`: permit send when `tenant.has_pack("HIPAA-2024") && message.phi_signal != "confirmed"`.
- Policy `HIPAA-mail-send-02`: permit send when `message.phi_signal == "confirmed" && principal.role in message.minimum_necessary_scope`.
- Policy `HIPAA-mail-send-03`: forbid send when `message.phi_signal == "confirmed" && recipient.external == true && recipient.baa_status != "verified"`.
- Policy `HIPAA-mail-forward-01`: forbid forward when `message.phi_signal == "confirmed" && mailbox.auto_forward_phi_blocked == true`.
- Policy `HIPAA-mail-forward-02`: permit manual forward when `principal.acr >= "elevated" && request.reason_id.exists()`.
- Policy `HIPAA-mail-attachment-01`: forbid download when `attachment.phi_scan_verdict == "scan_failed"`.
- Policy `HIPAA-mail-attachment-02`: permit attachment read when `principal.purpose in ["treatment","payment","operations"]`.
- Policy `HIPAA-mail-ai-01`: forbid AI draft when `message.phi_signal == "confirmed" && tenant.provider_credential_mode != "byok_required_by_pack"`.
- Policy `HIPAA-mail-search-01`: permit search when `principal.role in ["privacy_officer","covered_workforce"]`.
- Policy `HIPAA-mail-search-02`: forbid body search across patients unless `request.case_id` exists.
- Policy `HIPAA-mail-export-01`: permit eDiscovery export when `principal.role == "privacy_officer" && approval.count >= 2`.
- Policy `HIPAA-mail-breakglass-01`: permit emergency read when `request.break_glass_reason_id` exists and TTL <= 1h.
- Policy `HIPAA-mail-retention-01`: forbid purge before `mailbox.retention_floor_iso8601`.
- Policy `HIPAA-mail-shared-01`: forbid shared mailbox PHI when `mailbox.shared_phi_allowed == false`.
- Policy `HIPAA-mail-header-01`: forbid subject-line PHI when `message.header_phi_detected == true`.
- Policy `HIPAA-mail-route-01`: permit SMTP route only through `cell.certification.contains("hipaa-certified")`.
- Policy `HIPAA-mail-route-02`: forbid cross-region route when `message.phi_signal == "confirmed"`.
- Policy `HIPAA-mail-disclosure-01`: require disclosure accounting when recipient is outside covered entity.
- Policy `HIPAA-mail-alias-01`: forbid alias creation that hides covered workforce identity.
- Policy `HIPAA-mail-admin-01`: permit admin mailbox inspection only with privacy-office approval.
- Policy `HIPAA-mail-quarantine-01`: permit quarantine release only after DLP override and audit seal.
- Policy `HIPAA-mail-webhook-01`: forbid outbound webhook carrying PHI unless destination BAA verified.
- Policy `HIPAA-mail-legalhold-01`: permit legal hold release only after compliance workflow completion.
- Policy `HIPAA-mail-token-01`: require `principal.acr >= "elevated"` for PHI mailbox access.

## API Contract Deltas
- `POST /messages/send` requires `X-Oyatie-Purpose` for PHI-capable tenants.
- `POST /messages/send` rejects subject PHI detected by DLP.
- `POST /messages/send` accepts `patient_context_id` only as opaque identifier.
- `POST /messages/send` requires `minimum_necessary_scope` when `phi_signal=confirmed`.
- `POST /messages/{id}/forward` requires `reason_id` for confirmed PHI.
- `POST /messages/{id}/forward` rejects auto-forward rules for PHI.
- `GET /messages/{id}` requires `X-Oyatie-Access-Reason` for break-glass reads.
- `GET /attachments/{id}` returns 423 when PHI scan failed.
- `POST /attachments/scan-callback` must include signed scanner verdict.
- `POST /mailboxes/{id}/shared-members` requires privacy-office approval for PHI mailboxes.
- `POST /exports/ediscovery` requires two approvals and `case_id`.
- `GET /exports/{id}` returns PHI manifest hash with the export metadata.
- `POST /ai/draft` rejects confirmed PHI unless provider BYOK mode is active.
- `POST /dlp/override` requires `override_reason`, `approver_id`, and `ttl`.
- `PATCH /tenant-mail-config` refuses HIPAA activation without BAA admission proof.
- `GET /audit/disclosures` becomes available to privacy officers.
- `DELETE /messages/{id}` returns retention conflict before six-year floor.
- `POST /rules/auto-forward` refuses PHI-capable mailbox targets.
- `POST /webhooks` validates destination BAA status.
- `GET /threads/{id}` masks patient context for unauthorized recipients.

## Workflow Deltas
- Mail send preflight runs DLP PHI classifier before routing.
- Confirmed PHI send requires minimum-necessary evaluation.
- External recipient send checks BAA registry before SMTP handoff.
- Attachment ingest waits for PHI scan before delivery.
- PHI scan failure moves the attachment to quarantine.
- Quarantine release requires privacy-office approval.
- Shared mailbox membership change triggers covered-workforce review.
- Break-glass read opens a one-hour workflow with retrospective review.
- Legal hold starts audit-chain lock before message export.
- Legal hold release waits for compliance workflow completion.
- eDiscovery export emits PHI manifest before file creation.
- Patient-request communication records disclosure accounting.
- AI draft workflow blocks platform-default model providers.
- Mailbox deletion launches HIPAA retention review before purge.
- Auto-forward rule creation runs PHI-capability gate.
- Webhook creation checks BAA destination proof.
- Incident workflow tags suspected PHI exfiltration as reportable candidate.
- Failed audit emission blocks high-risk PHI mutations.
- Cell migration workflow requires HIPAA-certified target cell.
- Tenant pack deactivation waits for PHI retention drain.

## SLO Deltas
- PHI send preflight p99 must stay <= 750 ms.
- PHI DLP scan callback p99 must complete <= 10 minutes.
- Audit-chain seal for PHI send must complete <= 1 second.
- Break-glass audit emission p99 must complete <= 500 ms.
- Quarantine release approval routing p99 must start <= 2 minutes.
- External BAA lookup p99 must complete <= 200 ms.
- PHI export manifest generation p99 must complete <= 15 minutes.
- Retention conflict response p99 must complete <= 300 ms.
- Suspected breach candidate creation p99 must complete <= 5 minutes.
- HIPAA breach notification workflow must support 60-day outer deadline.
- Internal privacy-office notification p99 target is <= 24 hours after confirmed incident.
- Audit backpressure fail-closed transition p99 must complete <= 30 seconds.
- PHI route residency validation p99 must complete <= 100 ms.
- PHI attachment quarantine p99 must complete <= 2 minutes after verdict.
- Disclosure-accounting query p99 must complete <= 3 seconds.
- PHI classifier false-negative review cadence is weekly.

## Audit-event class additions
- `MailPhiSendPreflighted` records message id, purpose, and DLP verdict.
- `MailPhiSendBlocked` records recipient class and policy id.
- `MailPhiSent` records audit id and disclosure accounting flag.
- `MailPhiForwardRejected` records auto-forward or recipient reason.
- `MailPhiAttachmentQuarantined` records attachment digest and scanner verdict.
- `MailPhiAttachmentReleased` records approver and TTL.
- `MailPhiBreakGlassReadStarted` records reason id and expiry.
- `MailPhiBreakGlassReviewed` records reviewer decision.
- `MailPhiExternalDisclosureRecorded` records recipient organization hash.
- `MailPhiAiDraftBlocked` records provider credential mode.
- `MailPhiLegalHoldApplied` records hold id.
- `MailPhiLegalHoldReleased` records workflow id.
- `MailPhiExportManifestCreated` records manifest hash.
- `MailPhiRetentionPurgeRefused` records retention floor.
- `MailPhiWebhookRefused` records destination id.
- `MailPhiSharedMailboxMemberAdded` records approval id.
- `MailPhiBaaLookupFailed` records fail-closed routing.
- `MailPhiRouteResidencyBlocked` records source and target cell.
- `MailPhiAuditBackpressureClosed` records queue depth.
- `MailPhiPackDeactivationDeferred` records open retention count.

## Failure Modes specific to this pack
- PHI classifier is unavailable; recovery is fail-closed for external send and queue internal drafts.
- BAA registry lookup times out; recovery is block external route and retry with exponential backoff.
- Attachment scanner cannot decrypt tenant blob; recovery is quarantine and rotate DEK reference.
- Audit-chain backpressure appears; recovery is stop PHI mutations before evidence loss.
- User writes PHI in subject; recovery is reject send and create safe remediation draft.
- Auto-forward legacy rule exists; recovery is disable rule at activation and notify mailbox owner.
- External recipient loses BAA status; recovery is block future mail and open review for prior disclosures.
- Break-glass TTL expires mid-session; recovery is revoke token and preserve read audit.
- eDiscovery export exceeds retention scope; recovery is split export by case and patient context.
- AI provider mode is platform default; recovery is block model touch and request BYOK admission.
- Cross-region SMTP failover would leave HIPAA cell; recovery is queue until eligible route returns.
- Legal hold release races with purge; recovery is hold lock wins and purge retries after release.
- PHI scan false positive blocks urgent care message; recovery is privacy-office override with TTL.
- PHI scan false negative is discovered; recovery is retroactive disclosure accounting and incident review.
- Shared mailbox owner leaves tenant; recovery is freeze membership and require administrator reassignment.
- Patient context id maps to stale patient; recovery is block disclosure accounting until identity refresh.
- DLP override reason missing; recovery is reject override and keep quarantine.
- Webhook destination changes certificate; recovery is suspend route until BAA and TLS proof refresh.
- Pack deactivation requested with PHI retained; recovery is defer deactivation until retention obligations close.
- Mail import contains historical PHI; recovery is batch classify, quarantine risky items, and emit import audit.

## Cross-µservice coordination
- `tenancy` must activate `HIPAA-2024` and place the tenant in a HIPAA-certified cell.
- `identity` must provide elevated ACR claims for PHI mailbox reads.
- `compliance` must expose BAA status, breach workflow, and pack admission proof.
- `audit-chain` must seal every PHI state change within the tightened SLO.
- `observability` must scrub PHI at emission boundaries before Loki, Mimir, Tempo, or ClickHouse ingest.
- `drive` must apply HIPAA overlay when mail attachments are saved to drive.
- `workflow-engine` must run quarantine, legal hold, break-glass review, and breach workflows.
- `policy-engine` must load all `HIPAA-mail-*` Cedar fragments atomically.
- `cloud-kms` or OpenBao must provide tenant-owned key references for PHI attachments.
- `notification` must avoid PHI in push, SMS, and email notification previews.
- `search` must index only PHI-safe tokens and never raw PHI bodies.
- `admin-console` must expose BAA status before mail pack activation.
- `dlp-virus-scan` must return signed verdicts with scanner build provenance.
- `incident-response` must consume suspected breach candidates from mail.
- `records` must own patient context; mail stores only opaque references.
- `data-warehouse` must receive only aggregate PHI-free mail metrics.
- `legal` must register disclosure accounting export templates.
- `billing` must not receive PHI metadata from mail usage.
- `support` must use break-glass workflow for tenant PHI tickets.
- `pack-registry` must version this overlay with the HIPAA bundle signature.
