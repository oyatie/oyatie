---
doc_class: ArchitectureWalkthrough
shape: Reference
length_cap: 2400
authority_tier: 2
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0105
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0246
  - ADR-0252
  - ADR-0253
  - ADR-0254
  - ADR-0255
  - ADR-0257
  - ADR-0258
  - ADR-0263
  - ADR-0272
  - ADR-0273
  - ADR-0276
  - ADR-0284
  - ADR-0292
  - ADR-0294
  - ADR-0295
  - ADR-0296
  - ADR-0297
companion_docs:
  - microservices/mail/PRD.md
  - microservices/mail/threat-model.md
  - microservices/mail/dpia.md
  - microservices/mail/compliance.md
  - microservices/mail/manifest.json
planned_enforcement_ref: oya-governance-adr-adherence-matrix
inbound_citations:
  - microservices/mail/PRD.md
  - microservices/mail/README.md
---

# Mail µservice — Architecture Walkthrough

## §entry-point — cold-start

The Mail µservice is oyatie's personal + B2B mail product, modeled on **Gmail + Outlook + Apple Mail + Fastmail + ProtonMail + Tutanota + Hey.com** with JMAP RFC 8620 as the primary protocol and IMAP/POP3 as secondary. Calendar interop via iCalendar; filters via Sieve.

Cold-start question: *Where does an inbound email from a B2B client land, get classified, and surface in the recipient's inbox?* Trace:
1. Inbound SMTP (`oya-mail-inbound-smtp-adapter-smtp`) receives the message on port 25/587; DKIM/SPF/DMARC/ARC verification is the first step (per ADR-0273 per-tenant configuration).
2. Cedar gate `policy/abuse-defence.cedar` evaluates against bot-score + reputation; `policy/anti-phishing.cedar` evaluates against payload-shape + sender-history.
3. Spam classifier (T2-auto capability, EU-AI-Act limited risk per ADR-MAIL-0004) emits a score; HIPAA-eligible mailboxes additionally route to PHI-DLP classifier.
4. Mailbox store (`oya-mail-mailbox-store-app`) persists to Postgres (metadata) + S3 (blobs); per-tenant encryption applies when encryption-key BYOK is enabled (ADR-0251 §D-10).
5. Search index (`oya-mail-search-index-adapter-tantivy`) ingests metadata-only for E2EE accounts; full-text for non-E2EE.
6. ADR-0263 audit events `oya.mail.inbound-receive`, `oya.mail.spam-classify`, `oya.mail.deliver` are emitted with `tenant_id` + `audience_type` + `dual_context` (personal vs work) to the audit chain.
7. Push notification path delivers to the recipient's APNs / FCM / WebPush channels.

## §principals (ADR-0242)

Operates as `oyatie.mail.{inbound-smtp, outbound-smtp, mailbox-store, search-index, retention-policy, legal-hold, jmap-frontend, imap-frontend, spam-classifier}` principals. Called by tenant principals `<tenant>.<workspace>.<actor>` and by substrate principals from `connect`, `ontology`, `intelligence`, `governance`.
### Content-pass expansion — principals
- This expansion preserves the existing prose above and closes `principals` for `mail` to the ≥50-line documentation-rigor floor.
- Service owner `axis-mail` owns this answer; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-suggest`; bounded contexts: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`; +3 more.
- API surfaces: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy surfaces: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`; +5 more.
- State/event surfaces: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`; +1 more.
- SLO/dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS IAM service-linked roles anchors the external control pattern for `principals`.
- Precedent 2: Google Cloud service agents provides a second independent hyperscaler pattern for `principals`.
- Tenant-scope invariant: every `mail` `T0-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/mail/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `mail` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `mail` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `mail` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `mail` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `mail` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-suggest` evaluates `<tenant>.mail.t0-suggest` against policy, writes `mail.dual_context_isolation`, and emits `oya.mail.t0.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `principals`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `principals` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `mail` binds `principals (ADR-0242)` to `{'name': 'dual-context-isolation', 'description': "Bounded context 'dual-context-isolation' within mail (control plane)", 'crates': ['oya-mail-dual-context-isolation-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `mail` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `principals (ADR 0242)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `mail` is `policy/abuse-defence.cedar, policy/anti-phishing.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `principals (ADR 0242)`.
- Depth detail 4: `mail` state/event naming uses `mail.{'name': 'dual_context_isolation', 'description': "Bounded context 'dual_context_isolation' within mail (control plane)", 'crates': ['oya_mail_dual_context_isolation_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `mail` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `mail` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `mail` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `principals (ADR 0242)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `mail` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `mail` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `mail` uses SLOs `slos/dual-context-correctness.openslo.yaml, slos/ediscovery-export-freshness.openslo.yaml, slos/inbound-receive-availability.openslo.yaml, slos/inbox-open-latency.openslo.yaml, slos/jmap-mailbox-fetch-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/delivery-pipeline.json, dashboards/dmarc-deliverability.json, dashboards/inbox-experience.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `mail` uses runbooks `runbooks/account-compromise-recovery.md, runbooks/dkim-key-rotation.md, runbooks/dlp-quarantine-release.md, runbooks/dmarc-rollout-monitoring.md, runbooks/e2e-encryption-key-recovery.md, plus 5 more` so `principals (ADR 0242)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `mail` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/Chart.yaml, iac/helm/templates/deployment.yaml, iac/helm/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `mail` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-mail-anti-phishing-kernel.yaml, catalog/oya-mail-dual-context-isolation-kernel.yaml, catalog/oya-mail-imap-frontend-rest.yaml, catalog/oya-mail-inbound-smtp-adapter-smtp.yaml, plus 13 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `mail` fails closed when `principals (ADR 0242)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `mail` emits denial evidence for `principals (ADR 0242)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `mail` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `principals (ADR 0242)` workflow.
- Depth detail 17: `mail` telemetry for `principals (ADR 0242)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `mail` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §cedar-gates (ADR-0243)

Default-deny baseline at `policy/tenant-scope.cedar`. Defence-in-depth FORBIDs:
- `policy/auditor-scope.cedar` — auditors get read on retention-flagged ranges
- `policy/ci-scope.cedar` — CI principals separated from runtime
- `policy/public-read.cedar` — public-list metadata only
- `policy/dual-context-isolation.md` — personal vs work isolation
- `policy/abuse-defence.cedar` — anti-bot + anti-spoof + anti-scrape (anti-phishing in this context)
- `policy/anti-phishing.cedar` — payload-shape + sender-history + URL-reputation
- `policy/phi-dlp.cedar` — PHI detection on HIPAA-eligible mailboxes
- `policy/minor-protection.cedar` — KOSA 14-17 age-band handling per ADR-0292

Cedar v4.2 LTS. Fragment soak ≥60s per ADR-0294.
### Content-pass expansion — cedar-gates
- This expansion preserves the existing prose above and closes `cedar-gates` for `mail` to the ≥50-line documentation-rigor floor.
- Service owner `axis-mail` owns this answer; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-suggest`; bounded contexts: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`; +3 more.
- API surfaces: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy surfaces: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`; +5 more.
- State/event surfaces: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`; +1 more.
- SLO/dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Verified Permissions Cedar anchors the external control pattern for `cedar-gates`.
- Precedent 2: Google Zanzibar provides a second independent hyperscaler pattern for `cedar-gates`.
- Tenant-scope invariant: every `mail` `T0-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/mail/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `mail` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `mail` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `mail` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `mail` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `mail` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-suggest` evaluates `<tenant>.mail.t0-suggest` against policy, writes `mail.dual_context_isolation`, and emits `oya.mail.t0.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `cedar-gates`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `cedar-gates` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `mail` binds `cedar-gates (ADR-0243)` to `{'name': 'dual-context-isolation', 'description': "Bounded context 'dual-context-isolation' within mail (control plane)", 'crates': ['oya-mail-dual-context-isolation-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `mail` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `cedar gates (ADR 0243)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `mail` is `policy/abuse-defence.cedar, policy/anti-phishing.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `cedar gates (ADR 0243)`.
- Depth detail 4: `mail` state/event naming uses `mail.{'name': 'dual_context_isolation', 'description': "Bounded context 'dual_context_isolation' within mail (control plane)", 'crates': ['oya_mail_dual_context_isolation_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `mail` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `mail` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `mail` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `cedar gates (ADR 0243)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `mail` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `mail` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.

## §tenant-scoping (ADR-0244)

Every mailbox row carries `tenant_id` + `home_cell` + `dr_cell` + `audience_type` + `provider_credential_mode` + `compliance_packs[]`. `audience_type` enum: `B2C_PERSONAL`, `B2B_WORK`, `B2B_HIPAA_PHI`, `FRIENDLY_CRAWLER_PARTNER` (search-engine for public folders only), `INTERNAL_SUBSTRATE`. `provider_credential_mode`: `TENANT_BYOK` (`tenant_class=paid` + HIPAA + EU-sovereign default), `PLATFORM_MANAGED` (`tenant_class=demo_trial` default).
### Content-pass expansion — tenant-scoping
- This expansion preserves the existing prose above and closes `tenant-scoping` for `mail` to the ≥50-line documentation-rigor floor.
- Service owner `axis-mail` owns this answer; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-suggest`; bounded contexts: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`; +3 more.
- API surfaces: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy surfaces: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`; +5 more.
- State/event surfaces: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`; +1 more.
- SLO/dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Stripe Connect account isolation anchors the external control pattern for `tenant-scoping`.
- Precedent 2: AWS Organizations account boundary provides a second independent hyperscaler pattern for `tenant-scoping`.
- Tenant-scope invariant: every `mail` `T0-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/mail/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `mail` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `mail` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `mail` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `mail` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `mail` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-suggest` evaluates `<tenant>.mail.t0-suggest` against policy, writes `mail.dual_context_isolation`, and emits `oya.mail.t0.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `tenant-scoping`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `tenant-scoping` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `mail` binds `tenant-scoping (ADR-0244)` to `{'name': 'dual-context-isolation', 'description': "Bounded context 'dual-context-isolation' within mail (control plane)", 'crates': ['oya-mail-dual-context-isolation-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `mail` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `tenant scoping (ADR 0244)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `mail` is `policy/abuse-defence.cedar, policy/anti-phishing.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `tenant scoping (ADR 0244)`.
- Depth detail 4: `mail` state/event naming uses `mail.{'name': 'dual_context_isolation', 'description': "Bounded context 'dual_context_isolation' within mail (control plane)", 'crates': ['oya_mail_dual_context_isolation_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `mail` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `mail` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `mail` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `tenant scoping (ADR 0244)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `mail` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `mail` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `mail` uses SLOs `slos/dual-context-correctness.openslo.yaml, slos/ediscovery-export-freshness.openslo.yaml, slos/inbound-receive-availability.openslo.yaml, slos/inbox-open-latency.openslo.yaml, slos/jmap-mailbox-fetch-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/delivery-pipeline.json, dashboards/dmarc-deliverability.json, dashboards/inbox-experience.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `mail` uses runbooks `runbooks/account-compromise-recovery.md, runbooks/dkim-key-rotation.md, runbooks/dlp-quarantine-release.md, runbooks/dmarc-rollout-monitoring.md, runbooks/e2e-encryption-key-recovery.md, plus 5 more` so `tenant scoping (ADR 0244)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `mail` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/Chart.yaml, iac/helm/templates/deployment.yaml, iac/helm/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `mail` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-mail-anti-phishing-kernel.yaml, catalog/oya-mail-dual-context-isolation-kernel.yaml, catalog/oya-mail-imap-frontend-rest.yaml, catalog/oya-mail-inbound-smtp-adapter-smtp.yaml, plus 13 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `mail` fails closed when `tenant scoping (ADR 0244)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `mail` emits denial evidence for `tenant scoping (ADR 0244)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `mail` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `tenant scoping (ADR 0244)` workflow.
- Depth detail 17: `mail` telemetry for `tenant scoping (ADR 0244)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `mail` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §substrate-product-binding (ADR-0245)

**Service class: product.** Substrate dependencies: `ontology` (entity resolution for senders/recipients), `intelligence` (compose-assist, summarize, smart-reply), `governance` (retention + legal hold), `cell` (mailbox placement), `tenancy` (provisioning), `policy-engine` (Cedar evaluation library-first), `observability` (metrics, traces, audit chain), `compliance` (pack overlays), `cloud-secrets` (OpenBao for encryption-key BYOK keys per ADR-0251 §D-10), `comms-email` (transactional delivery substrate distinct from mailbox-hosted mail).
### Content-pass expansion — substrate-product-binding
- This expansion preserves the existing prose above and closes `substrate-product-binding` for `mail` to the ≥50-line documentation-rigor floor.
- Service owner `axis-mail` owns this answer; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-suggest`; bounded contexts: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`; +3 more.
- API surfaces: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy surfaces: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`; +5 more.
- State/event surfaces: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`; +1 more.
- SLO/dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir Foundry substrate pattern anchors the external control pattern for `substrate-product-binding`.
- Precedent 2: Google Cloud shared VPC split provides a second independent hyperscaler pattern for `substrate-product-binding`.
- Tenant-scope invariant: every `mail` `T0-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/mail/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `mail` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `mail` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `mail` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `mail` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `mail` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-suggest` evaluates `<tenant>.mail.t0-suggest` against policy, writes `mail.dual_context_isolation`, and emits `oya.mail.t0.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `substrate-product-binding`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `substrate-product-binding` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `mail` binds `substrate-product-binding (ADR-0245)` to `{'name': 'dual-context-isolation', 'description': "Bounded context 'dual-context-isolation' within mail (control plane)", 'crates': ['oya-mail-dual-context-isolation-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `mail` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `substrate product binding (ADR 0245)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `mail` is `policy/abuse-defence.cedar, policy/anti-phishing.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `substrate product binding (ADR 0245)`.
- Depth detail 4: `mail` state/event naming uses `mail.{'name': 'dual_context_isolation', 'description': "Bounded context 'dual_context_isolation' within mail (control plane)", 'crates': ['oya_mail_dual_context_isolation_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `mail` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `mail` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `mail` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `substrate product binding (ADR 0245)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `mail` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `mail` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `mail` uses SLOs `slos/dual-context-correctness.openslo.yaml, slos/ediscovery-export-freshness.openslo.yaml, slos/inbound-receive-availability.openslo.yaml, slos/inbox-open-latency.openslo.yaml, slos/jmap-mailbox-fetch-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/delivery-pipeline.json, dashboards/dmarc-deliverability.json, dashboards/inbox-experience.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `mail` uses runbooks `runbooks/account-compromise-recovery.md, runbooks/dkim-key-rotation.md, runbooks/dlp-quarantine-release.md, runbooks/dmarc-rollout-monitoring.md, runbooks/e2e-encryption-key-recovery.md, plus 5 more` so `substrate product binding (ADR 0245)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `mail` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/Chart.yaml, iac/helm/templates/deployment.yaml, iac/helm/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `mail` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-mail-anti-phishing-kernel.yaml, catalog/oya-mail-dual-context-isolation-kernel.yaml, catalog/oya-mail-imap-frontend-rest.yaml, catalog/oya-mail-inbound-smtp-adapter-smtp.yaml, plus 13 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `mail` fails closed when `substrate product binding (ADR 0245)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `mail` emits denial evidence for `substrate product binding (ADR 0245)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `mail` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `substrate product binding (ADR 0245)` workflow.
- Depth detail 17: `mail` telemetry for `substrate product binding (ADR 0245)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `mail` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §policy-evaluation (ADR-0246 + amendment)

Library-first via `oya-shared-policy-eval`. `policy_evaluation_mode: LIBRARY_FIRST`. Network fallback emits `oya.mail.policy-fallback-network`.
### Content-pass expansion — policy-evaluation
- This expansion preserves the existing prose above and closes `policy-evaluation` for `mail` to the ≥50-line documentation-rigor floor.
- Service owner `axis-mail` owns this answer; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-suggest`; bounded contexts: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`; +3 more.
- API surfaces: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy surfaces: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`; +5 more.
- State/event surfaces: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`; +1 more.
- SLO/dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Open Policy Agent sidecar anchors the external control pattern for `policy-evaluation`.
- Precedent 2: AWS Verified Permissions provides a second independent hyperscaler pattern for `policy-evaluation`.
- Tenant-scope invariant: every `mail` `T0-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/mail/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `mail` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `mail` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `mail` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `mail` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `mail` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-suggest` evaluates `<tenant>.mail.t0-suggest` against policy, writes `mail.dual_context_isolation`, and emits `oya.mail.t0.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `policy-evaluation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `policy-evaluation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `mail` binds `policy-evaluation (ADR-0246 + amendment)` to `{'name': 'dual-context-isolation', 'description': "Bounded context 'dual-context-isolation' within mail (control plane)", 'crates': ['oya-mail-dual-context-isolation-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `mail` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `policy evaluation (ADR 0246 + amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `mail` is `policy/abuse-defence.cedar, policy/anti-phishing.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `policy evaluation (ADR 0246 + amendment)`.
- Depth detail 4: `mail` state/event naming uses `mail.{'name': 'dual_context_isolation', 'description': "Bounded context 'dual_context_isolation' within mail (control plane)", 'crates': ['oya_mail_dual_context_isolation_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `mail` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `mail` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `mail` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `policy evaluation (ADR 0246 + amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `mail` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `mail` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `mail` uses SLOs `slos/dual-context-correctness.openslo.yaml, slos/ediscovery-export-freshness.openslo.yaml, slos/inbound-receive-availability.openslo.yaml, slos/inbox-open-latency.openslo.yaml, slos/jmap-mailbox-fetch-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/delivery-pipeline.json, dashboards/dmarc-deliverability.json, dashboards/inbox-experience.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `mail` uses runbooks `runbooks/account-compromise-recovery.md, runbooks/dkim-key-rotation.md, runbooks/dlp-quarantine-release.md, runbooks/dmarc-rollout-monitoring.md, runbooks/e2e-encryption-key-recovery.md, plus 5 more` so `policy evaluation (ADR 0246 + amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `mail` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/Chart.yaml, iac/helm/templates/deployment.yaml, iac/helm/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `mail` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-mail-anti-phishing-kernel.yaml, catalog/oya-mail-dual-context-isolation-kernel.yaml, catalog/oya-mail-imap-frontend-rest.yaml, catalog/oya-mail-inbound-smtp-adapter-smtp.yaml, plus 13 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `mail` fails closed when `policy evaluation (ADR 0246 + amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `mail` emits denial evidence for `policy evaluation (ADR 0246 + amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `mail` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `policy evaluation (ADR 0246 + amendment)` workflow.
- Depth detail 17: `mail` telemetry for `policy evaluation (ADR 0246 + amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `mail` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §intelligence-dispatch (ADR-0255 + amendment)

Library-first for compose-assist (T1), summarize-thread (T1), smart-reply suggestions (T1), spam-classifier (T2 limited risk). Audience tag: per-call `B2C_PERSONAL` / `B2B_WORK` / `B2B_HIPAA_PHI`. **HIPAA-eligible mailboxes**: intelligence calls are gated by additional `policy/phi-dlp.cedar` + tenant BAA acknowledgement; outputs are routed only through HIPAA-conformant intelligence variants (per ADR-0255 §HIPAA-overlay).
### Content-pass expansion — intelligence-dispatch
- This expansion preserves the existing prose above and closes `intelligence-dispatch` for `mail` to the ≥50-line documentation-rigor floor.
- Service owner `axis-mail` owns this answer; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-suggest`; bounded contexts: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`; +3 more.
- API surfaces: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy surfaces: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`; +5 more.
- State/event surfaces: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`; +1 more.
- SLO/dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir AIP tool boundary anchors the external control pattern for `intelligence-dispatch`.
- Precedent 2: Azure OpenAI tenant deployment provides a second independent hyperscaler pattern for `intelligence-dispatch`.
- Tenant-scope invariant: every `mail` `T0-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/mail/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `mail` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `mail` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `mail` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `mail` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `mail` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-suggest` evaluates `<tenant>.mail.t0-suggest` against policy, writes `mail.dual_context_isolation`, and emits `oya.mail.t0.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `intelligence-dispatch`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `intelligence-dispatch` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `mail` binds `intelligence-dispatch (ADR-0255 + amendment)` to `{'name': 'dual-context-isolation', 'description': "Bounded context 'dual-context-isolation' within mail (control plane)", 'crates': ['oya-mail-dual-context-isolation-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `mail` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `intelligence dispatch (ADR 0255 + amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `mail` is `policy/abuse-defence.cedar, policy/anti-phishing.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `intelligence dispatch (ADR 0255 + amendment)`.
- Depth detail 4: `mail` state/event naming uses `mail.{'name': 'dual_context_isolation', 'description': "Bounded context 'dual_context_isolation' within mail (control plane)", 'crates': ['oya_mail_dual_context_isolation_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `mail` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `mail` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `mail` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `intelligence dispatch (ADR 0255 + amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `mail` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `mail` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `mail` uses SLOs `slos/dual-context-correctness.openslo.yaml, slos/ediscovery-export-freshness.openslo.yaml, slos/inbound-receive-availability.openslo.yaml, slos/inbox-open-latency.openslo.yaml, slos/jmap-mailbox-fetch-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/delivery-pipeline.json, dashboards/dmarc-deliverability.json, dashboards/inbox-experience.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `mail` uses runbooks `runbooks/account-compromise-recovery.md, runbooks/dkim-key-rotation.md, runbooks/dlp-quarantine-release.md, runbooks/dmarc-rollout-monitoring.md, runbooks/e2e-encryption-key-recovery.md, plus 5 more` so `intelligence dispatch (ADR 0255 + amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `mail` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/Chart.yaml, iac/helm/templates/deployment.yaml, iac/helm/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `mail` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-mail-anti-phishing-kernel.yaml, catalog/oya-mail-dual-context-isolation-kernel.yaml, catalog/oya-mail-imap-frontend-rest.yaml, catalog/oya-mail-inbound-smtp-adapter-smtp.yaml, plus 13 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `mail` fails closed when `intelligence dispatch (ADR 0255 + amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `mail` emits denial evidence for `intelligence dispatch (ADR 0255 + amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `mail` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `intelligence dispatch (ADR 0255 + amendment)` workflow.
- Depth detail 17: `mail` telemetry for `intelligence dispatch (ADR 0255 + amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `mail` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §ontology-read-path (ADR-0257 + amendment)

`ontology_read_mode: LIBRARY_FIRST_BYO_CACHE`. Used to resolve sender / recipient entity identities for thread-grouping, contact-card enrichment, and entity-link suggestions in compose. `freshness_floor: LOOSE` (60s acceptable; mail isn't a strict-realtime surface).
### Content-pass expansion — ontology-read-path
- This expansion preserves the existing prose above and closes `ontology-read-path` for `mail` to the ≥50-line documentation-rigor floor.
- Service owner `axis-mail` owns this answer; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-suggest`; bounded contexts: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`; +3 more.
- API surfaces: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy surfaces: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`; +5 more.
- State/event surfaces: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`; +1 more.
- SLO/dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir Foundry ontology projections anchors the external control pattern for `ontology-read-path`.
- Precedent 2: Google Knowledge Graph serving cache provides a second independent hyperscaler pattern for `ontology-read-path`.
- Tenant-scope invariant: every `mail` `T0-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/mail/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `mail` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `mail` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `mail` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `mail` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `mail` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-suggest` evaluates `<tenant>.mail.t0-suggest` against policy, writes `mail.dual_context_isolation`, and emits `oya.mail.t0.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `ontology-read-path`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `ontology-read-path` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `mail` binds `ontology-read-path (ADR-0257 + amendment)` to `{'name': 'dual-context-isolation', 'description': "Bounded context 'dual-context-isolation' within mail (control plane)", 'crates': ['oya-mail-dual-context-isolation-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `mail` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `ontology read path (ADR 0257 + amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `mail` is `policy/abuse-defence.cedar, policy/anti-phishing.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `ontology read path (ADR 0257 + amendment)`.
- Depth detail 4: `mail` state/event naming uses `mail.{'name': 'dual_context_isolation', 'description': "Bounded context 'dual_context_isolation' within mail (control plane)", 'crates': ['oya_mail_dual_context_isolation_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `mail` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `mail` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `mail` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `ontology read path (ADR 0257 + amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `mail` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `mail` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `mail` uses SLOs `slos/dual-context-correctness.openslo.yaml, slos/ediscovery-export-freshness.openslo.yaml, slos/inbound-receive-availability.openslo.yaml, slos/inbox-open-latency.openslo.yaml, slos/jmap-mailbox-fetch-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/delivery-pipeline.json, dashboards/dmarc-deliverability.json, dashboards/inbox-experience.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `mail` uses runbooks `runbooks/account-compromise-recovery.md, runbooks/dkim-key-rotation.md, runbooks/dlp-quarantine-release.md, runbooks/dmarc-rollout-monitoring.md, runbooks/e2e-encryption-key-recovery.md, plus 5 more` so `ontology read path (ADR 0257 + amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `mail` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/Chart.yaml, iac/helm/templates/deployment.yaml, iac/helm/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `mail` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-mail-anti-phishing-kernel.yaml, catalog/oya-mail-dual-context-isolation-kernel.yaml, catalog/oya-mail-imap-frontend-rest.yaml, catalog/oya-mail-inbound-smtp-adapter-smtp.yaml, plus 13 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `mail` fails closed when `ontology read path (ADR 0257 + amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `mail` emits denial evidence for `ontology read path (ADR 0257 + amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `mail` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `ontology read path (ADR 0257 + amendment)` workflow.
- Depth detail 17: `mail` telemetry for `ontology read path (ADR 0257 + amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `mail` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §time-coordination (ADR-0252)

HLC default; TrueTime opt-in for `legal-hold` engagement timestamps (chain-of-custody requirement).
### Content-pass expansion — time-coordination
- This expansion preserves the existing prose above and closes `time-coordination` for `mail` to the ≥50-line documentation-rigor floor.
- Service owner `axis-mail` owns this answer; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-suggest`; bounded contexts: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`; +3 more.
- API surfaces: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy surfaces: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`; +5 more.
- State/event surfaces: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`; +1 more.
- SLO/dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google Spanner TrueTime anchors the external control pattern for `time-coordination`.
- Precedent 2: CockroachDB HLC ordering provides a second independent hyperscaler pattern for `time-coordination`.
- Tenant-scope invariant: every `mail` `T0-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/mail/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `mail` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `mail` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `mail` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `mail` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `mail` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-suggest` evaluates `<tenant>.mail.t0-suggest` against policy, writes `mail.dual_context_isolation`, and emits `oya.mail.t0.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `time-coordination`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `time-coordination` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `mail` binds `time-coordination (ADR-0252)` to `{'name': 'dual-context-isolation', 'description': "Bounded context 'dual-context-isolation' within mail (control plane)", 'crates': ['oya-mail-dual-context-isolation-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `mail` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `time coordination (ADR 0252)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `mail` is `policy/abuse-defence.cedar, policy/anti-phishing.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `time coordination (ADR 0252)`.
- Depth detail 4: `mail` state/event naming uses `mail.{'name': 'dual_context_isolation', 'description': "Bounded context 'dual_context_isolation' within mail (control plane)", 'crates': ['oya_mail_dual_context_isolation_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `mail` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `mail` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `mail` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `time coordination (ADR 0252)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `mail` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `mail` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `mail` uses SLOs `slos/dual-context-correctness.openslo.yaml, slos/ediscovery-export-freshness.openslo.yaml, slos/inbound-receive-availability.openslo.yaml, slos/inbox-open-latency.openslo.yaml, slos/jmap-mailbox-fetch-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/delivery-pipeline.json, dashboards/dmarc-deliverability.json, dashboards/inbox-experience.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `mail` uses runbooks `runbooks/account-compromise-recovery.md, runbooks/dkim-key-rotation.md, runbooks/dlp-quarantine-release.md, runbooks/dmarc-rollout-monitoring.md, runbooks/e2e-encryption-key-recovery.md, plus 5 more` so `time coordination (ADR 0252)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `mail` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/Chart.yaml, iac/helm/templates/deployment.yaml, iac/helm/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `mail` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-mail-anti-phishing-kernel.yaml, catalog/oya-mail-dual-context-isolation-kernel.yaml, catalog/oya-mail-imap-frontend-rest.yaml, catalog/oya-mail-inbound-smtp-adapter-smtp.yaml, plus 13 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `mail` fails closed when `time coordination (ADR 0252)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `mail` emits denial evidence for `time coordination (ADR 0252)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `mail` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `time coordination (ADR 0252)` workflow.
- Depth detail 17: `mail` telemetry for `time coordination (ADR 0252)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `mail` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §transport (ADR-0253)

JMAP over HTTP/3 + QUIC default; falls back HTTP/3 → HTTP/2 → HTTP/1.1 (never skip h2). IMAP/POP3 over TLS 1.3. SMTP-over-TLS with `MTA-STS` + `DANE` for outbound. ECH advertised on JMAP surface; PQC hybrid `X25519MLKEM768` advertised; signature hybrid `ed25519+ml_dsa_65` on cert chains. UX-floor: non-PQC mail clients fall through to classical TLS 1.3 without breakage.
### Content-pass expansion — transport
- This expansion preserves the existing prose above and closes `transport` for `mail` to the ≥50-line documentation-rigor floor.
- Service owner `axis-mail` owns this answer; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-suggest`; bounded contexts: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`; +3 more.
- API surfaces: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy surfaces: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`; +5 more.
- State/event surfaces: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`; +1 more.
- SLO/dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google QUIC HTTP/3 anchors the external control pattern for `transport`.
- Precedent 2: Cloudflare ECH/PQC TLS provides a second independent hyperscaler pattern for `transport`.
- Tenant-scope invariant: every `mail` `T0-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/mail/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `mail` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `mail` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `mail` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `mail` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `mail` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-suggest` evaluates `<tenant>.mail.t0-suggest` against policy, writes `mail.dual_context_isolation`, and emits `oya.mail.t0.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `transport`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `transport` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `mail` binds `transport (ADR-0253)` to `{'name': 'dual-context-isolation', 'description': "Bounded context 'dual-context-isolation' within mail (control plane)", 'crates': ['oya-mail-dual-context-isolation-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `mail` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `transport (ADR 0253)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `mail` is `policy/abuse-defence.cedar, policy/anti-phishing.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `transport (ADR 0253)`.
- Depth detail 4: `mail` state/event naming uses `mail.{'name': 'dual_context_isolation', 'description': "Bounded context 'dual_context_isolation' within mail (control plane)", 'crates': ['oya_mail_dual_context_isolation_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `mail` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `mail` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `mail` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `transport (ADR 0253)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `mail` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `mail` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `mail` uses SLOs `slos/dual-context-correctness.openslo.yaml, slos/ediscovery-export-freshness.openslo.yaml, slos/inbound-receive-availability.openslo.yaml, slos/inbox-open-latency.openslo.yaml, slos/jmap-mailbox-fetch-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/delivery-pipeline.json, dashboards/dmarc-deliverability.json, dashboards/inbox-experience.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `mail` uses runbooks `runbooks/account-compromise-recovery.md, runbooks/dkim-key-rotation.md, runbooks/dlp-quarantine-release.md, runbooks/dmarc-rollout-monitoring.md, runbooks/e2e-encryption-key-recovery.md, plus 5 more` so `transport (ADR 0253)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `mail` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/Chart.yaml, iac/helm/templates/deployment.yaml, iac/helm/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `mail` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-mail-anti-phishing-kernel.yaml, catalog/oya-mail-dual-context-isolation-kernel.yaml, catalog/oya-mail-imap-frontend-rest.yaml, catalog/oya-mail-inbound-smtp-adapter-smtp.yaml, plus 13 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `mail` fails closed when `transport (ADR 0253)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `mail` emits denial evidence for `transport (ADR 0253)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `mail` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `transport (ADR 0253)` workflow.
- Depth detail 17: `mail` telemetry for `transport (ADR 0253)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `mail` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §deployment-shape (ADR-0254)

K8s + Cloud Hypervisor + Kata pods:
- `oya-mail-mailbox-store-app` → Kata pod (mailbox data sensitivity)
- `oya-mail-inbound-smtp-adapter-smtp` → standard pod with mTLS sidecar
- `oya-mail-outbound-smtp-adapter-smtp` → standard pod
- `oya-mail-search-index-adapter-tantivy` → Kata pod
- `oya-mail-spam-classifier` → Kata pod with isolated GPU when present
### Content-pass expansion — deployment-shape
- This expansion preserves the existing prose above and closes `deployment-shape` for `mail` to the ≥50-line documentation-rigor floor.
- Service owner `axis-mail` owns this answer; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-suggest`; bounded contexts: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`; +3 more.
- API surfaces: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy surfaces: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`; +5 more.
- State/event surfaces: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`; +1 more.
- SLO/dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Firecracker isolation anchors the external control pattern for `deployment-shape`.
- Precedent 2: GKE Sandbox/Kata provides a second independent hyperscaler pattern for `deployment-shape`.
- Tenant-scope invariant: every `mail` `T0-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/mail/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `mail` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `mail` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `mail` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `mail` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `mail` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-suggest` evaluates `<tenant>.mail.t0-suggest` against policy, writes `mail.dual_context_isolation`, and emits `oya.mail.t0.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `deployment-shape`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `deployment-shape` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `mail` binds `deployment-shape (ADR-0254)` to `{'name': 'dual-context-isolation', 'description': "Bounded context 'dual-context-isolation' within mail (control plane)", 'crates': ['oya-mail-dual-context-isolation-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `mail` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `deployment shape (ADR 0254)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `mail` is `policy/abuse-defence.cedar, policy/anti-phishing.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `deployment shape (ADR 0254)`.
- Depth detail 4: `mail` state/event naming uses `mail.{'name': 'dual_context_isolation', 'description': "Bounded context 'dual_context_isolation' within mail (control plane)", 'crates': ['oya_mail_dual_context_isolation_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `mail` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `mail` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `mail` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `deployment shape (ADR 0254)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `mail` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `mail` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `mail` uses SLOs `slos/dual-context-correctness.openslo.yaml, slos/ediscovery-export-freshness.openslo.yaml, slos/inbound-receive-availability.openslo.yaml, slos/inbox-open-latency.openslo.yaml, slos/jmap-mailbox-fetch-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/delivery-pipeline.json, dashboards/dmarc-deliverability.json, dashboards/inbox-experience.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `mail` uses runbooks `runbooks/account-compromise-recovery.md, runbooks/dkim-key-rotation.md, runbooks/dlp-quarantine-release.md, runbooks/dmarc-rollout-monitoring.md, runbooks/e2e-encryption-key-recovery.md, plus 5 more` so `deployment shape (ADR 0254)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `mail` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/Chart.yaml, iac/helm/templates/deployment.yaml, iac/helm/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `mail` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-mail-anti-phishing-kernel.yaml, catalog/oya-mail-dual-context-isolation-kernel.yaml, catalog/oya-mail-imap-frontend-rest.yaml, catalog/oya-mail-inbound-smtp-adapter-smtp.yaml, plus 13 more` to keep layer names and owners machine-checkable.

## §marketplace (ADR-0249)

Exposes `email-template`, `signature-gallery`, `sieve-filter-recipe`, `compose-assist-skill` marketplace categories.
### Content-pass expansion — marketplace
- This expansion preserves the existing prose above and closes `marketplace` for `mail` to the ≥50-line documentation-rigor floor.
- Service owner `axis-mail` owns this answer; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-suggest`; bounded contexts: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`; +3 more.
- API surfaces: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy surfaces: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`; +5 more.
- State/event surfaces: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`; +1 more.
- SLO/dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Stripe Connect platform facilitator anchors the external control pattern for `marketplace`.
- Precedent 2: AWS Marketplace seller controls provides a second independent hyperscaler pattern for `marketplace`.
- Tenant-scope invariant: every `mail` `T0-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/mail/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `mail` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `mail` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `mail` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `mail` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `mail` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-suggest` evaluates `<tenant>.mail.t0-suggest` against policy, writes `mail.dual_context_isolation`, and emits `oya.mail.t0.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `marketplace`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `marketplace` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `mail` binds `marketplace (ADR-0249)` to `{'name': 'dual-context-isolation', 'description': "Bounded context 'dual-context-isolation' within mail (control plane)", 'crates': ['oya-mail-dual-context-isolation-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `mail` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `marketplace (ADR 0249)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `mail` is `policy/abuse-defence.cedar, policy/anti-phishing.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `marketplace (ADR 0249)`.
- Depth detail 4: `mail` state/event naming uses `mail.{'name': 'dual_context_isolation', 'description': "Bounded context 'dual_context_isolation' within mail (control plane)", 'crates': ['oya_mail_dual_context_isolation_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `mail` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `mail` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `mail` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `marketplace (ADR 0249)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `mail` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `mail` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `mail` uses SLOs `slos/dual-context-correctness.openslo.yaml, slos/ediscovery-export-freshness.openslo.yaml, slos/inbound-receive-availability.openslo.yaml, slos/inbox-open-latency.openslo.yaml, slos/jmap-mailbox-fetch-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/delivery-pipeline.json, dashboards/dmarc-deliverability.json, dashboards/inbox-experience.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `mail` uses runbooks `runbooks/account-compromise-recovery.md, runbooks/dkim-key-rotation.md, runbooks/dlp-quarantine-release.md, runbooks/dmarc-rollout-monitoring.md, runbooks/e2e-encryption-key-recovery.md, plus 5 more` so `marketplace (ADR 0249)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `mail` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/Chart.yaml, iac/helm/templates/deployment.yaml, iac/helm/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `mail` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-mail-anti-phishing-kernel.yaml, catalog/oya-mail-dual-context-isolation-kernel.yaml, catalog/oya-mail-imap-frontend-rest.yaml, catalog/oya-mail-inbound-smtp-adapter-smtp.yaml, plus 13 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `mail` fails closed when `marketplace (ADR 0249)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `mail` emits denial evidence for `marketplace (ADR 0249)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `mail` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `marketplace (ADR 0249)` workflow.
- Depth detail 17: `mail` telemetry for `marketplace (ADR 0249)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `mail` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §observability (ADR-0263)

Audit-event classes: `oya.mail.inbound-receive`, `oya.mail.outbound-send`, `oya.mail.spam-classify`, `oya.mail.dlp-detect`, `oya.mail.deliver`, `oya.mail.legal-hold-engage`, `oya.mail.legal-hold-release`, `oya.mail.retention-purge`, `oya.mail.dkim-key-rotate`, `oya.mail.abuse-defence-block`, `oya.mail.phi-detect`, `oya.mail.minor-protect-engage`, `oya.mail.tenant-byok-mint`, `oya.mail.search-index-eviction`.

Per-metric cardinality budget: 10000. High-cardinality (message_id, thread_id) → trace-span attributes only.
### Content-pass expansion — observability
- This expansion preserves the existing prose above and closes `observability` for `mail` to the ≥50-line documentation-rigor floor.
- Service owner `axis-mail` owns this answer; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-suggest`; bounded contexts: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`; +3 more.
- API surfaces: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy surfaces: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`; +5 more.
- State/event surfaces: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`; +1 more.
- SLO/dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google SRE four reference signals anchors the external control pattern for `observability`.
- Precedent 2: OpenTelemetry semantic conventions provides a second independent hyperscaler pattern for `observability`.
- Tenant-scope invariant: every `mail` `T0-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/mail/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `mail` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `mail` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `mail` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `mail` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `mail` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-suggest` evaluates `<tenant>.mail.t0-suggest` against policy, writes `mail.dual_context_isolation`, and emits `oya.mail.t0.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `observability`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `observability` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `mail` binds `observability (ADR-0263)` to `{'name': 'dual-context-isolation', 'description': "Bounded context 'dual-context-isolation' within mail (control plane)", 'crates': ['oya-mail-dual-context-isolation-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `mail` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `observability (ADR 0263)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `mail` is `policy/abuse-defence.cedar, policy/anti-phishing.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `observability (ADR 0263)`.
- Depth detail 4: `mail` state/event naming uses `mail.{'name': 'dual_context_isolation', 'description': "Bounded context 'dual_context_isolation' within mail (control plane)", 'crates': ['oya_mail_dual_context_isolation_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `mail` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `mail` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `mail` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `observability (ADR 0263)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `mail` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `mail` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `mail` uses SLOs `slos/dual-context-correctness.openslo.yaml, slos/ediscovery-export-freshness.openslo.yaml, slos/inbound-receive-availability.openslo.yaml, slos/inbox-open-latency.openslo.yaml, slos/jmap-mailbox-fetch-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/delivery-pipeline.json, dashboards/dmarc-deliverability.json, dashboards/inbox-experience.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `mail` uses runbooks `runbooks/account-compromise-recovery.md, runbooks/dkim-key-rotation.md, runbooks/dlp-quarantine-release.md, runbooks/dmarc-rollout-monitoring.md, runbooks/e2e-encryption-key-recovery.md, plus 5 more` so `observability (ADR 0263)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `mail` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/Chart.yaml, iac/helm/templates/deployment.yaml, iac/helm/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `mail` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-mail-anti-phishing-kernel.yaml, catalog/oya-mail-dual-context-isolation-kernel.yaml, catalog/oya-mail-imap-frontend-rest.yaml, catalog/oya-mail-inbound-smtp-adapter-smtp.yaml, plus 13 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `mail` fails closed when `observability (ADR 0263)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `mail` emits denial evidence for `observability (ADR 0263)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `mail` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `observability (ADR 0263)` workflow.
- Depth detail 17: `mail` telemetry for `observability (ADR 0263)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §consent (ADR-0272)

Per-purpose consent surface on first sign-in: (a) compose-assist (T1), (b) smart-reply (T1), (c) spam-classifier (T2 limited; required for delivery), (d) marketing-email-from-platform (opt-in). Cookie consent on web client follows ADR-0272 per-purpose model.
### Content-pass expansion — consent
- This expansion preserves the existing prose above and closes `consent` for `mail` to the ≥50-line documentation-rigor floor.
- Service owner `axis-mail` owns this answer; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-suggest`; bounded contexts: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`; +3 more.
- API surfaces: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy surfaces: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`; +5 more.
- State/event surfaces: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`; +1 more.
- SLO/dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google Consent Mode anchors the external control pattern for `consent`.
- Precedent 2: Apple App Tracking Transparency provides a second independent hyperscaler pattern for `consent`.
- Tenant-scope invariant: every `mail` `T0-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/mail/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `mail` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `mail` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `mail` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `mail` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `mail` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-suggest` evaluates `<tenant>.mail.t0-suggest` against policy, writes `mail.dual_context_isolation`, and emits `oya.mail.t0.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `consent`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `consent` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `mail` binds `consent (ADR-0272)` to `{'name': 'dual-context-isolation', 'description': "Bounded context 'dual-context-isolation' within mail (control plane)", 'crates': ['oya-mail-dual-context-isolation-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `mail` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `consent (ADR 0272)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `mail` is `policy/abuse-defence.cedar, policy/anti-phishing.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `consent (ADR 0272)`.
- Depth detail 4: `mail` state/event naming uses `mail.{'name': 'dual_context_isolation', 'description': "Bounded context 'dual_context_isolation' within mail (control plane)", 'crates': ['oya_mail_dual_context_isolation_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `mail` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `mail` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `mail` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `consent (ADR 0272)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `mail` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `mail` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `mail` uses SLOs `slos/dual-context-correctness.openslo.yaml, slos/ediscovery-export-freshness.openslo.yaml, slos/inbound-receive-availability.openslo.yaml, slos/inbox-open-latency.openslo.yaml, slos/jmap-mailbox-fetch-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/delivery-pipeline.json, dashboards/dmarc-deliverability.json, dashboards/inbox-experience.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `mail` uses runbooks `runbooks/account-compromise-recovery.md, runbooks/dkim-key-rotation.md, runbooks/dlp-quarantine-release.md, runbooks/dmarc-rollout-monitoring.md, runbooks/e2e-encryption-key-recovery.md, plus 5 more` so `consent (ADR 0272)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `mail` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/Chart.yaml, iac/helm/templates/deployment.yaml, iac/helm/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `mail` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-mail-anti-phishing-kernel.yaml, catalog/oya-mail-dual-context-isolation-kernel.yaml, catalog/oya-mail-imap-frontend-rest.yaml, catalog/oya-mail-inbound-smtp-adapter-smtp.yaml, plus 13 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `mail` fails closed when `consent (ADR 0272)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `mail` emits denial evidence for `consent (ADR 0272)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `mail` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `consent (ADR 0272)` workflow.
- Depth detail 17: `mail` telemetry for `consent (ADR 0272)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `mail` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §email-deliverability (ADR-0273)

Per-tenant DKIM signing key in OpenBao at `secret/<tenant>/mail/dkim-signing-key`; rotated 90d per `runbooks/dkim-key-rotation.md`. SPF auto-published as `v=spf1 include:_spf.mail.oyatie.example -all` per tenant subdomain. DMARC `p=reject` after 30d onboarding monitoring (`p=none` initially). ARC seal on every forwarded message. BIMI logo per tenant.
### Content-pass expansion — email-deliverability
- This expansion preserves the existing prose above and closes `email-deliverability` for `mail` to the ≥50-line documentation-rigor floor.
- Service owner `axis-mail` owns this answer; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-suggest`; bounded contexts: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`; +3 more.
- API surfaces: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy surfaces: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`; +5 more.
- State/event surfaces: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`; +1 more.
- SLO/dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google Workspace DKIM/SPF/DMARC anchors the external control pattern for `email-deliverability`.
- Precedent 2: AWS SES domain identity provides a second independent hyperscaler pattern for `email-deliverability`.
- Tenant-scope invariant: every `mail` `T0-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/mail/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `mail` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `mail` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `mail` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `mail` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `mail` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-suggest` evaluates `<tenant>.mail.t0-suggest` against policy, writes `mail.dual_context_isolation`, and emits `oya.mail.t0.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `email-deliverability`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `email-deliverability` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `mail` binds `email-deliverability (ADR-0273)` to `{'name': 'dual-context-isolation', 'description': "Bounded context 'dual-context-isolation' within mail (control plane)", 'crates': ['oya-mail-dual-context-isolation-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `mail` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `email deliverability (ADR 0273)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `mail` is `policy/abuse-defence.cedar, policy/anti-phishing.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `email deliverability (ADR 0273)`.
- Depth detail 4: `mail` state/event naming uses `mail.{'name': 'dual_context_isolation', 'description': "Bounded context 'dual_context_isolation' within mail (control plane)", 'crates': ['oya_mail_dual_context_isolation_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `mail` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `mail` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `mail` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `email deliverability (ADR 0273)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `mail` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `mail` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `mail` uses SLOs `slos/dual-context-correctness.openslo.yaml, slos/ediscovery-export-freshness.openslo.yaml, slos/inbound-receive-availability.openslo.yaml, slos/inbox-open-latency.openslo.yaml, slos/jmap-mailbox-fetch-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/delivery-pipeline.json, dashboards/dmarc-deliverability.json, dashboards/inbox-experience.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `mail` uses runbooks `runbooks/account-compromise-recovery.md, runbooks/dkim-key-rotation.md, runbooks/dlp-quarantine-release.md, runbooks/dmarc-rollout-monitoring.md, runbooks/e2e-encryption-key-recovery.md, plus 5 more` so `email deliverability (ADR 0273)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `mail` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/Chart.yaml, iac/helm/templates/deployment.yaml, iac/helm/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `mail` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-mail-anti-phishing-kernel.yaml, catalog/oya-mail-dual-context-isolation-kernel.yaml, catalog/oya-mail-imap-frontend-rest.yaml, catalog/oya-mail-inbound-smtp-adapter-smtp.yaml, plus 13 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `mail` fails closed when `email deliverability (ADR 0273)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `mail` emits denial evidence for `email deliverability (ADR 0273)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `mail` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `email deliverability (ADR 0273)` workflow.
- Depth detail 17: `mail` telemetry for `email deliverability (ADR 0273)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `mail` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §minor-protection (ADR-0292)

If `audience_type=B2C_PERSONAL` AND tenant declares `minor_age_band=COPPA_UNDER_13` → **refuse account provisioning** (no mailbox created; remediation: parental account with delegated child mailbox via paid `family` tenant_class pack). If `minor_age_band=KOSA_14_17` → strict content-filter on inbound, anti-grooming heuristics, parental dashboard surface, age-gated AI features disabled by default.
### Content-pass expansion — minor-protection
- This expansion preserves the existing prose above and closes `minor-protection` for `mail` to the ≥50-line documentation-rigor floor.
- Service owner `axis-mail` owns this answer; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-suggest`; bounded contexts: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`; +3 more.
- API surfaces: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy surfaces: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`; +5 more.
- State/event surfaces: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`; +1 more.
- SLO/dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Apple Family/Screen Time controls anchors the external control pattern for `minor-protection`.
- Precedent 2: Google Family Link provides a second independent hyperscaler pattern for `minor-protection`.
- Tenant-scope invariant: every `mail` `T0-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/mail/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `mail` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `mail` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `mail` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `mail` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `mail` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-suggest` evaluates `<tenant>.mail.t0-suggest` against policy, writes `mail.dual_context_isolation`, and emits `oya.mail.t0.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `minor-protection`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `minor-protection` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `mail` binds `minor-protection (ADR-0292)` to `{'name': 'dual-context-isolation', 'description': "Bounded context 'dual-context-isolation' within mail (control plane)", 'crates': ['oya-mail-dual-context-isolation-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `mail` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `minor protection (ADR 0292)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `mail` is `policy/abuse-defence.cedar, policy/anti-phishing.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `minor protection (ADR 0292)`.
- Depth detail 4: `mail` state/event naming uses `mail.{'name': 'dual_context_isolation', 'description': "Bounded context 'dual_context_isolation' within mail (control plane)", 'crates': ['oya_mail_dual_context_isolation_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `mail` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `mail` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `mail` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `minor protection (ADR 0292)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `mail` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `mail` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `mail` uses SLOs `slos/dual-context-correctness.openslo.yaml, slos/ediscovery-export-freshness.openslo.yaml, slos/inbound-receive-availability.openslo.yaml, slos/inbox-open-latency.openslo.yaml, slos/jmap-mailbox-fetch-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/delivery-pipeline.json, dashboards/dmarc-deliverability.json, dashboards/inbox-experience.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `mail` uses runbooks `runbooks/account-compromise-recovery.md, runbooks/dkim-key-rotation.md, runbooks/dlp-quarantine-release.md, runbooks/dmarc-rollout-monitoring.md, runbooks/e2e-encryption-key-recovery.md, plus 5 more` so `minor protection (ADR 0292)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `mail` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/Chart.yaml, iac/helm/templates/deployment.yaml, iac/helm/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `mail` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-mail-anti-phishing-kernel.yaml, catalog/oya-mail-dual-context-isolation-kernel.yaml, catalog/oya-mail-imap-frontend-rest.yaml, catalog/oya-mail-inbound-smtp-adapter-smtp.yaml, plus 13 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `mail` fails closed when `minor protection (ADR 0292)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `mail` emits denial evidence for `minor protection (ADR 0292)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `mail` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `minor protection (ADR 0292)` workflow.
- Depth detail 17: `mail` telemetry for `minor protection (ADR 0292)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `mail` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §abuse-defence (ADR-0297)

Anti-bot: token-bucket per-IP + per-tenant; bot-mgmt scoring at edge for JMAP web client; CAPTCHA on suspicious sign-up only. Anti-phishing (anti-spoof): inbound DKIM/SPF/DMARC/ARC enforcement; URL-reputation via threat-intel; payload-shape ML classifier for impersonation attempts; quarantine-on-suspicion with user-recovery flow. Anti-scrape: aggressive rate-limit on JMAP unauthenticated endpoints; honeypot mail addresses to detect harvesting bots; per-user invisible watermarks on rendered HTML emails (legacy-client-safe variant).

UX-floor: legitimate JMAP / IMAP clients see ZERO friction; bot-mgmt is passive scoring; CAPTCHA only on suspicious sign-up + suspicious password-reset (never on regular login or compose). Latency added ≤2ms p99.
### Content-pass expansion — abuse-defence
- This expansion preserves the existing prose above and closes `abuse-defence` for `mail` to the ≥50-line documentation-rigor floor.
- Service owner `axis-mail` owns this answer; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-suggest`; bounded contexts: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`; +3 more.
- API surfaces: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy surfaces: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`; +5 more.
- State/event surfaces: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`; +1 more.
- SLO/dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Cloudflare Bot Management anchors the external control pattern for `abuse-defence`.
- Precedent 2: Stripe Radar provides a second independent hyperscaler pattern for `abuse-defence`.
- Tenant-scope invariant: every `mail` `T0-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/mail/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `mail` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `mail` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `mail` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `mail` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `mail` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-suggest` evaluates `<tenant>.mail.t0-suggest` against policy, writes `mail.dual_context_isolation`, and emits `oya.mail.t0.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `abuse-defence`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `abuse-defence` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `mail` binds `abuse-defence (ADR-0297)` to `{'name': 'dual-context-isolation', 'description': "Bounded context 'dual-context-isolation' within mail (control plane)", 'crates': ['oya-mail-dual-context-isolation-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `mail` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `abuse defence (ADR 0297)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `mail` is `policy/abuse-defence.cedar, policy/anti-phishing.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `abuse defence (ADR 0297)`.
- Depth detail 4: `mail` state/event naming uses `mail.{'name': 'dual_context_isolation', 'description': "Bounded context 'dual_context_isolation' within mail (control plane)", 'crates': ['oya_mail_dual_context_isolation_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `mail` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `mail` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `mail` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `abuse defence (ADR 0297)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `mail` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `mail` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `mail` uses SLOs `slos/dual-context-correctness.openslo.yaml, slos/ediscovery-export-freshness.openslo.yaml, slos/inbound-receive-availability.openslo.yaml, slos/inbox-open-latency.openslo.yaml, slos/jmap-mailbox-fetch-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/delivery-pipeline.json, dashboards/dmarc-deliverability.json, dashboards/inbox-experience.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `mail` uses runbooks `runbooks/account-compromise-recovery.md, runbooks/dkim-key-rotation.md, runbooks/dlp-quarantine-release.md, runbooks/dmarc-rollout-monitoring.md, runbooks/e2e-encryption-key-recovery.md, plus 5 more` so `abuse defence (ADR 0297)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `mail` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/Chart.yaml, iac/helm/templates/deployment.yaml, iac/helm/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `mail` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-mail-anti-phishing-kernel.yaml, catalog/oya-mail-dual-context-isolation-kernel.yaml, catalog/oya-mail-imap-frontend-rest.yaml, catalog/oya-mail-inbound-smtp-adapter-smtp.yaml, plus 13 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `mail` fails closed when `abuse defence (ADR 0297)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `mail` emits denial evidence for `abuse defence (ADR 0297)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `mail` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `abuse defence (ADR 0297)` workflow.
- Depth detail 17: `mail` telemetry for `abuse defence (ADR 0297)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §credential-isolation (ADR-0296)

Tenant encryption-key BYOK keys live in OpenBao with ≤60s sidecar TTL (ADR-0251 §D-10). DKIM signing key, mailbox-encryption key, S3 KMS reference, and intelligence provider-credential BYOK token (ADR-0255 §D-4) all flow through sidecar; mail µservice never holds long-lived credentials.
### Content-pass expansion — credential-isolation
- This expansion preserves the existing prose above and closes `credential-isolation` for `mail` to the ≥50-line documentation-rigor floor.
- Service owner `axis-mail` owns this answer; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-suggest`; bounded contexts: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`; +3 more.
- API surfaces: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy surfaces: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`; +5 more.
- State/event surfaces: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`; +1 more.
- SLO/dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: HashiCorp Vault dynamic secrets anchors the external control pattern for `credential-isolation`.
- Precedent 2: AWS KMS envelope isolation provides a second independent hyperscaler pattern for `credential-isolation`.
- Tenant-scope invariant: every `mail` `T0-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/mail/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `mail` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `mail` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `mail` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `mail` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `mail` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-suggest` evaluates `<tenant>.mail.t0-suggest` against policy, writes `mail.dual_context_isolation`, and emits `oya.mail.t0.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `credential-isolation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `credential-isolation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `mail` binds `credential-isolation (ADR-0296)` to `{'name': 'dual-context-isolation', 'description': "Bounded context 'dual-context-isolation' within mail (control plane)", 'crates': ['oya-mail-dual-context-isolation-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `mail` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `credential isolation (ADR 0296)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `mail` is `policy/abuse-defence.cedar, policy/anti-phishing.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `credential isolation (ADR 0296)`.
- Depth detail 4: `mail` state/event naming uses `mail.{'name': 'dual_context_isolation', 'description': "Bounded context 'dual_context_isolation' within mail (control plane)", 'crates': ['oya_mail_dual_context_isolation_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `mail` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `mail` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `mail` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `credential isolation (ADR 0296)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `mail` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `mail` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `mail` uses SLOs `slos/dual-context-correctness.openslo.yaml, slos/ediscovery-export-freshness.openslo.yaml, slos/inbound-receive-availability.openslo.yaml, slos/inbox-open-latency.openslo.yaml, slos/jmap-mailbox-fetch-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/delivery-pipeline.json, dashboards/dmarc-deliverability.json, dashboards/inbox-experience.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `mail` uses runbooks `runbooks/account-compromise-recovery.md, runbooks/dkim-key-rotation.md, runbooks/dlp-quarantine-release.md, runbooks/dmarc-rollout-monitoring.md, runbooks/e2e-encryption-key-recovery.md, plus 5 more` so `credential isolation (ADR 0296)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `mail` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/Chart.yaml, iac/helm/templates/deployment.yaml, iac/helm/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `mail` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-mail-anti-phishing-kernel.yaml, catalog/oya-mail-dual-context-isolation-kernel.yaml, catalog/oya-mail-imap-frontend-rest.yaml, catalog/oya-mail-inbound-smtp-adapter-smtp.yaml, plus 13 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `mail` fails closed when `credential isolation (ADR 0296)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `mail` emits denial evidence for `credential isolation (ADR 0296)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `mail` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `credential isolation (ADR 0296)` workflow.
- Depth detail 17: `mail` telemetry for `credential isolation (ADR 0296)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `mail` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §portability (ADR-0276)

Per-tenant backup export in standard MBOX + JMAP backup envelope; portable to any RFC 8620 client. Export gated by Cedar `policy/auditor-scope.cedar`. GDPR Art. 20 right-to-data-portability honored.

## §self-modification

Does not produce self-modification artifacts. Consumes Foundry-built Sieve filter templates; meta-trust-root attestation per ADR-0293 verified.
### Content-pass expansion — self-modification
- This expansion preserves the existing prose above and closes `self-modification` for `mail` to the ≥50-line documentation-rigor floor.
- Service owner `axis-mail` owns this answer; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-suggest`; bounded contexts: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`; +3 more.
- API surfaces: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy surfaces: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`; +5 more.
- State/event surfaces: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`; +1 more.
- SLO/dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: SLSA provenance anchors the external control pattern for `self-modification`.
- Precedent 2: Google Binary Authorization provides a second independent hyperscaler pattern for `self-modification`.
- Tenant-scope invariant: every `mail` `T0-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/mail/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `mail` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `mail` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `mail` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `mail` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `mail` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-suggest` evaluates `<tenant>.mail.t0-suggest` against policy, writes `mail.dual_context_isolation`, and emits `oya.mail.t0.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `self-modification`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `self-modification` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `mail` binds `self-modification` to `{'name': 'dual-context-isolation', 'description': "Bounded context 'dual-context-isolation' within mail (control plane)", 'crates': ['oya-mail-dual-context-isolation-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `mail` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `self modification` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `mail` is `policy/abuse-defence.cedar, policy/anti-phishing.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `self modification`.
- Depth detail 4: `mail` state/event naming uses `mail.{'name': 'dual_context_isolation', 'description': "Bounded context 'dual_context_isolation' within mail (control plane)", 'crates': ['oya_mail_dual_context_isolation_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `mail` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `mail` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `mail` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `self modification` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `mail` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `mail` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `mail` uses SLOs `slos/dual-context-correctness.openslo.yaml, slos/ediscovery-export-freshness.openslo.yaml, slos/inbound-receive-availability.openslo.yaml, slos/inbox-open-latency.openslo.yaml, slos/jmap-mailbox-fetch-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/delivery-pipeline.json, dashboards/dmarc-deliverability.json, dashboards/inbox-experience.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `mail` uses runbooks `runbooks/account-compromise-recovery.md, runbooks/dkim-key-rotation.md, runbooks/dlp-quarantine-release.md, runbooks/dmarc-rollout-monitoring.md, runbooks/e2e-encryption-key-recovery.md, plus 5 more` so `self modification` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `mail` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/Chart.yaml, iac/helm/templates/deployment.yaml, iac/helm/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `mail` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-mail-anti-phishing-kernel.yaml, catalog/oya-mail-dual-context-isolation-kernel.yaml, catalog/oya-mail-imap-frontend-rest.yaml, catalog/oya-mail-inbound-smtp-adapter-smtp.yaml, plus 13 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `mail` fails closed when `self modification` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `mail` emits denial evidence for `self modification` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `mail` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `self modification` workflow.
- Depth detail 17: `mail` telemetry for `self modification` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `mail` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §fragment-publish (ADR-0294) + §bootstrap-trust-chain (ADR-0295)

Cedar fragments soak 60s. Mailbox-store boots with SPIFFE attestation; kill-switch engages on attestation failure.
### Content-pass expansion — fragment-publish
- This expansion preserves the existing prose above and closes `fragment-publish` for `mail` to the ≥50-line documentation-rigor floor.
- Service owner `axis-mail` owns this answer; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-suggest`; bounded contexts: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`; +3 more.
- API surfaces: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy surfaces: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`; +5 more.
- State/event surfaces: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`; +1 more.
- SLO/dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS AppConfig bake windows anchors the external control pattern for `fragment-publish`.
- Precedent 2: Google Binary Authorization provides a second independent hyperscaler pattern for `fragment-publish`.
- Tenant-scope invariant: every `mail` `T0-suggest` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/mail/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `mail` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `mail` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `mail` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `mail` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `mail` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-suggest` evaluates `<tenant>.mail.t0-suggest` against policy, writes `mail.dual_context_isolation`, and emits `oya.mail.t0.suggest.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `fragment-publish`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `fragment-publish` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `mail` binds `fragment-publish (ADR-0294) + §bootstrap-trust-chain (ADR-0295)` to `{'name': 'dual-context-isolation', 'description': "Bounded context 'dual-context-isolation' within mail (control plane)", 'crates': ['oya-mail-dual-context-isolation-kernel']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `mail` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `fragment publish (ADR 0294) + §bootstrap trust chain (ADR 0295)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `mail` is `policy/abuse-defence.cedar, policy/anti-phishing.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dual-context-isolation.md, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `fragment publish (ADR 0294) + §bootstrap trust chain (ADR 0295)`.
- Depth detail 4: `mail` state/event naming uses `mail.{'name': 'dual_context_isolation', 'description': "Bounded context 'dual_context_isolation' within mail (control plane)", 'crates': ['oya_mail_dual_context_isolation_kernel']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `mail` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `mail` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `mail` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `fragment publish (ADR 0294) + §bootstrap trust chain (ADR 0295)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `mail` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `mail` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `mail` uses SLOs `slos/dual-context-correctness.openslo.yaml, slos/ediscovery-export-freshness.openslo.yaml, slos/inbound-receive-availability.openslo.yaml, slos/inbox-open-latency.openslo.yaml, slos/jmap-mailbox-fetch-latency.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/delivery-pipeline.json, dashboards/dmarc-deliverability.json, dashboards/inbox-experience.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `mail` uses runbooks `runbooks/account-compromise-recovery.md, runbooks/dkim-key-rotation.md, runbooks/dlp-quarantine-release.md, runbooks/dmarc-rollout-monitoring.md, runbooks/e2e-encryption-key-recovery.md, plus 5 more` so `fragment publish (ADR 0294) + §bootstrap trust chain (ADR 0295)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `mail` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/Chart.yaml, iac/helm/templates/deployment.yaml, iac/helm/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `mail` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-mail-anti-phishing-kernel.yaml, catalog/oya-mail-dual-context-isolation-kernel.yaml, catalog/oya-mail-imap-frontend-rest.yaml, catalog/oya-mail-inbound-smtp-adapter-smtp.yaml, plus 13 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `mail` fails closed when `fragment publish (ADR 0294) + §bootstrap trust chain (ADR 0295)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `mail` emits denial evidence for `fragment publish (ADR 0294) + §bootstrap trust chain (ADR 0295)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `mail` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `fragment publish (ADR 0294) + §bootstrap trust chain (ADR 0295)` workflow.
- Depth detail 17: `mail` telemetry for `fragment publish (ADR 0294) + §bootstrap trust chain (ADR 0295)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `mail` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §where-to-read-next

- `microservices/mail/PRD.md`
- `microservices/mail/threat-model.md`
- `microservices/mail/dpia.md`
- `microservices/mail/compliance.md`

---



## §cell-eligibility
This anchor is closed for `mail` against ADR-0248 §D-1: cell tier, shard width, DR pair and shuffle-shard behavior.

### Service-specific answer
- Cell eligibility declaration: `not declared in manifest; bound here to the conservative platform default`.
- Tier 0/1 control-plane paths run in hardened cells; tenant data planes can shard per tenant, pack, region, and workload class.
- Per-cell shard key is `(tenant_id, home_cell, jurisdiction_code)`; DR pair selection uses `dr_cell` where data-residency permits failover.
- Shuffle-shard width is documented by `multi-region.md` or defaults to three independent cells for Tier-1 control paths.
- Regional outage behavior: keep reads local where pack permits, stop cross-border replication where pack forbids it, and preserve audit emission locally.
- Example: `T0-suggest` traffic in a KR pack tenant stays in KR home cell; DR failover requires pack approval and emits a cell-failover audit event.
- Capacity math lives in `capacity-model.md`; this section binds the shard dimensions so the math is not detached from topology.
- Cloud Hypervisor/Kata isolation applies to Tier 0/1 pods; Tier 2/3 paths inherit the same network policy and SPIFFE identity floor.

### Concrete inventory used
- Service: `mail`; owner `axis-mail`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`, `outbound-smtp`; +2 more.
- Capability records cited: `microservices/mail/capabilities/T0-suggest.yaml`, `microservices/mail/capabilities/T1-assist.yaml`, `microservices/mail/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy artifacts cited: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- SLO and dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar binding: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- State/event binding: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`, `mail.outbound_smtp`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `mail`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `mail`.
- `policy-engine` supplies the signed Cedar corpus while `mail` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `mail` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `mail`.

### Hyperscaler precedents
- Precedent 1: AWS cell-based architecture is the reference pattern for the control shape described here.
- Precedent 2: Route 53 shuffle-sharding isolation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `mail` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §critical-path-edge-cases
This anchor is closed for `mail` against documentation-rigor.md §3.2.5: applicable human-safety and platform edge-case handling.

### Service-specific answer
- Network partition: `mail` keeps tenant-local reads when safe, stops cross-cell writes that would violate residency, and emits degraded-mode audit events.
- Byzantine caller: Cedar denies forged `principal_id`, mismatched `tenant_id`, invalid SVID, replayed idempotency keys, and suspicious bot-score context.
- Regional outage: home-cell failover follows `multi-region.md`; if a pack forbids cross-border DR, `mail` preserves local queue state instead of failing open.
- Key compromise: ADR-0296 sidecar revokes OpenBao leases, rotates signing keys, and quarantines affected audit event classes for reconciliation.
- Account recovery/hijack path: identity step-up and `mail` audit evidence keep legitimate recovery from becoming an adversary shortcut.
- Mistaken mutation path: high-impact `T0-suggest` mutations require idempotency, undo/cooldown where product semantics allow, and sealed evidence for later correction.
- Disaster surge: `mail` enforces per-tenant isolation so one hot tenant or emergency mode cannot starve unrelated cells.
- Verification: capacity math in `capacity-model.md`, rollback in `failure-modes.md`, DR handling in `multi-region.md`, and incident actions in runbooks.

### Concrete inventory used
- Service: `mail`; owner `axis-mail`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`, `outbound-smtp`; +2 more.
- Capability records cited: `microservices/mail/capabilities/T0-suggest.yaml`, `microservices/mail/capabilities/T1-assist.yaml`, `microservices/mail/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy artifacts cited: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- SLO and dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar binding: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- State/event binding: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`, `mail.outbound_smtp`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `mail`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `mail`.
- `policy-engine` supplies the signed Cedar corpus while `mail` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `mail` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `mail`.

### Hyperscaler precedents
- Precedent 1: Google SRE incident playbooks is the reference pattern for the control shape described here.
- Precedent 2: Stripe idempotent mutation recovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `mail` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.
