<!-- WAVE 15J SCRUB COMPLETION REPORT
  µservice: mail
  capability_tiers_directory_deleted: yes
  manifest_tier_fields_removed: 2
  prd_md_tier_references_scrubbed: 2
  architecture_md_tier_references_scrubbed: 25
  compliance_md_pack_tier_references_scrubbed: 0 (no Gold/Platinum pack strings found; 19 adjacent metadata/age-band tier strings scrubbed)
  total_files_modified: 18
  total_lines_changed: 186 retained-file lines plus deleted retired tier-matrix file
  ADR_0316_citations_replaced_with_0329_0330_0331: 2
  cellular_tier_references_preserved: 7 (per ADR-0248)
  halt_cleanly: yes
-->

## Wave 15-IP-substance scrub (2026-05-21)

Bucket: IP-BUCKET-E.

Scope: `mail`.

Rewritten in place:

- `IP-016-jmap-rfc-8620-frontend.md` — replaced the short RFC/JMAP shell with a bespoke plan for `oya-mail-jmap-frontend-rest`, `/jmap/session`, `/jmap/api`, upload/download, state-token invalidation, HTTP/3 fallback, `ContextKind` isolation, `contracts/openapi/mail.yaml`, `contracts/proto/mail.proto`, and `slos/jmap-mailbox-fetch-latency.openslo.yaml`.
- `IP-017-anti-phishing-edge-wiring.md` — replaced the short anti-phishing shell with a concrete inbound SMTP edge plan covering DKIM/SPF/DMARC/ARC facts, `oya-mail-anti-phishing-kernel`, `policy/anti-phishing.cedar`, URL reputation, quarantine release, `dashboards/abuse-defence-outcomes.json`, and `runbooks/account-compromise-recovery.md`.
- `IP-018-hipaa-overlay-rollout.md` — replaced the short HIPAA shell with BAA-gated provisioning, `policy/phi-dlp.cedar`, `DATA_CLASS_PHI`, `oya-mail-phi-dlp-adapter-kernel`, HIPAA-conformant intelligence routing, `packs/HIPAA.md`, and `runbooks/phi-leak-recovery.md`.

Preserved as already substantive:

- `IP-001` through `IP-015` already referenced concrete mail files, crates, contracts, policies, SLOs, test gates, and counterpart/product doctrine.
- Journey IPs were long-form, domain-specific, and outside the short stamped-shell cluster.

Deleted as duplicative: none.

Counterpart anchors added across rewritten IPs: Fastmail, Stalwart, Gmail, Microsoft Exchange, Proton, and Google Workspace.

Verification notes:

- The three rewritten IPs now include bespoke A-G sections, concrete local file references, implementation steps, acceptance checks, evidence, and counterpart rows.
- Remaining 30-80 line output after the scrub is expected to include preserved substantive IPs, not just stamped shells.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:

- `microservices/mail/IP-001-iac-bootstrap.md`
- `microservices/mail/iac/helm/values.yaml`

Counterpart-fact preservations:

None.

Files renamed (git mv):

None.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: set PRD target to manifest `rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_active_active=true`, and `runbooks/dr-failover.md` per ADR-0343, with EU-AI/HIPAA/KR-PIPA/SOC2/ISO27001/PCI floors cited. Rejected mailbox-restore-only wording because accepted SMTP delivery and legal notice evidence must fail over first. Cost: queue replication, WAL retention, restore drills, and active-active ingress.
- Capacity model: declared manifest `0.18 vCPU`, `384 MiB RAM`, `25 GiB storage`, Valkey/Postgres/outbound baselines, `per_message` scaling, Tier-3 placement, `pod_runtime_tier=2`, and `3..64` replica bounds per ADR-0340. Rejected a single mailbox-size scalar because DLP, search, eDiscovery, and SMTP bursts have different bottlenecks. Cost: reserved edge capacity and separate export worker quotas.
- Sustainability + cost attribution: required `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on send, receive, DLP, search, restore, eDiscovery, and deliverability audit rows per ADR-0344. Rejected provider-bill-only accounting because climate and tenant cost disclosures must distinguish retention, scanning, and delivery. Cost: metering on storage/search/provider paths and finops reconciliation.
- API versioning posture: adopted the `YYYY-MM-DD` carrier triplet for oyatie mail APIs, SDK semver, N=3/180-day support, tenant pinning, and ADR-0145 internal-mesh exemption per ADR-0342. Rejected RFC-protocol-only versioning because JMAP/REST extensions and proto events still need governed public compatibility. Cost: versioned extension contracts and migration support.
- Frontmatter: added ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345. ADR-0337 was not added because mail does not document an Iceberg data-warehouse write path.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.18 vCPU, 384 MiB RAM, 25 GB storage per active tenant; connections valkey=3, postgres=4, outbound_http=8; scaling_dimension=per_message; cell_placement_class=Tier-3.
- ADR: ADR-0340 capacity declaration plus ADR-0248 cellular class.
- Rejected: template-stamped values copied from another service; mail remains Tier-2/Tier-3 because its unit of isolation is the first-party mailbox application; the cost is larger storage and WAL-G/object-store DR rather than Kata placement.
- Cost: cell sizing and autoscaler budgets must reserve this per-tenant baseline before admitting more tenants.

### Block 2: dr
- Values: rto_p99_seconds=3600, rpo_p99_seconds=300, multi_region_active_active=true, backup_substrate=postgres_wal_g, object_storage_versioned, valkey, openbao_seal_unseal, audit_chain_merkle_seal, failover_runbook=runbooks/dr-failover.md.
- ADR: ADR-0343 plus compliance-pack floors; HIPAA/us-healthcare floors drive the 1h/5m baseline where applicable.
- Rejected: looser 24h PCI-only recovery because this service can serve healthcare or sensitive tenant workflows.
- Cost: warm cross-region replication and quarterly drill evidence are required for the declared runbook.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=2; evidence=microservices/mail/PRD.md, microservices/mail/IP-004-mailbox-store-s3-adapter.md, microservices/mail/IP-008-imap-frontend.md, microservices/mail/runbooks/mailbox-restore-from-backup.md.
- ADR: ADR-0338 runtime tiering; ADR-0340/ADR-0248 co-variance with cell_placement_class=Tier-3.
- Rejected: weaker runtime class that would contradict the documented tenant-data or first-party-app surface.
- Cost: runtime placement, nodepool capacity, and incident severity inherit this tier.

### Block 4: tenant_version_pinning
- Values: declared_versions=2026-05-21; default_version=2026-05-21; supported_window_size=3; supported_window_minimum_days=180; supports_per_tenant_pinning=true.
- ADR: ADR-0342 date-versioned public APIs with per-tenant pinning.
- Rejected: internal-only exemption because this service has public OpenAPI, AsyncAPI, and proto surfaces.
- Cost: at least three supported public API windows and migration docs for any future breaking change.

### Block 5: consumes_upstream_oss + oss_stewardship_class_overrides
- Values: consumes_upstream_oss=postgresql, valkey, cedar, openbao, kafka, opentelemetry; oss_stewardship_class_overrides=[].
- ADR: ADR-0345 and /specs/oss-stewardship-registry.json registry authority.
- Rejected: local stewardship overrides because the registry default class is sufficient for each declared upstream.
- Cost: SBOM and CVE-response evidence must trace this service to each upstream owner team.

### Block 6: iac_module_invocations
- Values: oyatie-as-cloud-provider/k8s-namespace-bootstrap@v1, oyatie-as-cloud-provider/secrets-bootstrap@v1, oyatie-as-cloud-provider/dns@v1.
- ADR: ADR-0339 shared IaC module library.
- Rejected: unpinned local wrapper-only IaC because module reuse and pinning are the admission surface.
- Cost: module pins must be advanced deliberately when cloud-iac publishes new primitives.

## Wave 15-doctrine-propagation-IPs (2026-05-21)

- Bucket: `D4-BUCKET-2`.
- Doctrine source: ADR-0337..0345 selective propagation by trigger match; this section records only matched IPs.
- Manifest gap: `manifest.json#dr` is absent, so DR sections preserve compliance-pack floors without inventing service RTO/RPO targets.

| IP | Trigger(s) | Required sections | Source evidence | Manifest gaps |
| --- | --- | --- | --- | --- |
| `microservices/mail/IP-001-iac-bootstrap.md` | B | DR posture (per ADR-0343) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-005-dual-context-isolation.md` | C | Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
| `microservices/mail/IP-006-inbound-smtp.md` | B | DR posture (per ADR-0343) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-007-outbound-smtp.md` | B | DR posture (per ADR-0343) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-008-imap-frontend.md` | B | DR posture (per ADR-0343) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-009-search-index.md` | B | DR posture (per ADR-0343) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-010-retention-policy.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-011-legal-hold-engine.md` | B | DR posture (per ADR-0343) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-014-hg-mail-authority-cohesion.md` | B | DR posture (per ADR-0343) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-015-pack-kr-overlay.md` | B | DR posture (per ADR-0343) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-016-jmap-rfc-8620-frontend.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-017-anti-phishing-edge-wiring.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-018-hipaa-overlay-rollout.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j01-emergency-family-mail-fallback.md` | A, C | API Versioning (per ADR-0342); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
| `microservices/mail/IP-journey-j04-safe-inbox-routing.md` | A, C | API Versioning (per ADR-0342); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
| `microservices/mail/IP-journey-j07-inheritance-mail-digest.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j09-recovery-notice-delivery.md` | A, C | API Versioning (per ADR-0342); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
| `microservices/mail/IP-journey-j100-pack-rollout-first-action.md` | C | Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
| `microservices/mail/IP-journey-j101-tenant-notification.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j105-tenant-notification.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j107-tenant-notification.md` | C | Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
| `microservices/mail/IP-journey-j117-customer-notification-trail.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j122-vendor-remittance-notices.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j124-supplier-and-employee-alerts.md` | A | API Versioning (per ADR-0342) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
| `microservices/mail/IP-journey-j127-mail-archive-on-leaver.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j132-hiring-mail-cascade.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j133-rif-mail-templates.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j136-enrollment-mail-cascade.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j137-corporate-internal-audit-sox-controls-test-archive-reader.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j138-corporate-audit-targeted-correspondence-pull.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j142-work-mail-demotion-and-cross-tenant-packet.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j144-auto-reply-and-digest-delivery.md` | B, C, D | DR posture (per ADR-0343); Sustainability emission (per ADR-0344); Pod runtime tier (per ADR-0338) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing; manifest.json#pod_runtime_tier missing |
| `microservices/mail/IP-journey-j145-cross-tenant-offer-letter-delivery.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j146-marketplace-notifications.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j18-authority-notice-delivery.md` | A, C | API Versioning (per ADR-0342); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
| `microservices/mail/IP-journey-j22-first-week-inbox.md` | C | Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
| `microservices/mail/IP-journey-j23-sale-receipt.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j24-shipping-notices.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j27-imip-invite-bridge.md` | C | Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
| `microservices/mail/IP-journey-j35-workplace-deliverability.md` | C | Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
| `microservices/mail/IP-journey-j36-approval-notification-thread.md` | B, D | DR posture (per ADR-0343); Pod runtime tier (per ADR-0338) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing; manifest.json#pod_runtime_tier missing |
| `microservices/mail/IP-journey-j38-counterparty-envelope.md` | B | DR posture (per ADR-0343) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j40-billing-receipts.md` | B, D | DR posture (per ADR-0343); Pod runtime tier (per ADR-0338) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing; manifest.json#pod_runtime_tier missing |
| `microservices/mail/IP-journey-j45-lab-result-notice.md` | B | DR posture (per ADR-0343) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j46-rx-status-messaging.md` | B, D | DR posture (per ADR-0343); Pod runtime tier (per ADR-0338) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing; manifest.json#pod_runtime_tier missing |
| `microservices/mail/IP-journey-j47-bill-and-eob-thread.md` | B | DR posture (per ADR-0343) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j48-tax-notice-delivery.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j49-support-email-bridge.md` | B, D | DR posture (per ADR-0343); Pod runtime tier (per ADR-0338) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing; manifest.json#pod_runtime_tier missing |
| `microservices/mail/IP-journey-j51-po-ingest-sender.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j52-tracking-notification.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j53-invoice-and-reminder.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j54-quote-delivery.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j55-formal-notice.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j56-offer-letter.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j57-welcome-sequence.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j58-review-summary.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j59-forward-and-retention.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j60-promotion-notice.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j61-patient-summary.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j62-receipt-and-instructions.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j64-hipaa-referral.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j65-mail-export.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j66-regulator-notifications.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j67-user-and-court-notice.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j69-mail-triage.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j70-counterparty-thread.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j71-receipt-and-appeal.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j72-auto-translate-thread.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j73-subscriber-notice.md` | A, B, C, D | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344); Pod runtime tier (per ADR-0338) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing; manifest.json#pod_runtime_tier missing |
| `microservices/mail/IP-journey-j74-plugin-mail-actions.md` | A, B, C, D | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344); Pod runtime tier (per ADR-0338) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing; manifest.json#pod_runtime_tier missing |
| `microservices/mail/IP-journey-j75-admin-notice.md` | A, B, C, D | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344); Pod runtime tier (per ADR-0338) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing; manifest.json#pod_runtime_tier missing |
| `microservices/mail/IP-journey-j76-notice-delivery.md` | A | API Versioning (per ADR-0342) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
| `microservices/mail/IP-journey-j78-notice-delivery.md` | A | API Versioning (per ADR-0342) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
| `microservices/mail/IP-journey-j79-notice-delivery.md` | A | API Versioning (per ADR-0342) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
| `microservices/mail/IP-journey-j80-notice-delivery.md` | A | API Versioning (per ADR-0342) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
| `microservices/mail/IP-journey-j82-notice-delivery.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j84-notice-delivery.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j85-notice-delivery.md` | A | API Versioning (per ADR-0342) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
| `microservices/mail/IP-journey-j89-notice-delivery.md` | A | API Versioning (per ADR-0342) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
| `microservices/mail/IP-journey-j91-us-msb-mtl-overlay.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j92-br-lgpd-us-parent-dsar.md` | C | Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
| `microservices/mail/IP-journey-j93-in-dpdpa-rbi-overlay.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j94-sox404-public-company-controls.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | manifest.json#dr missing |
| `microservices/mail/IP-journey-j95-iso27001-soc2-annual-audit.md` | C | Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
| `microservices/mail/IP-journey-j96-ksa-uae-mena-onboarding.md` | C | Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
| `microservices/mail/IP-journey-j97-sg-pdpa-mas-tenant.md` | C | Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
| `microservices/mail/IP-journey-j98-au-privacy-apra-cps234.md` | C | Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
| `microservices/mail/IP-journey-j99-multi-pack-conflict-resolution.md` | C | Sustainability emission (per ADR-0344) | microservices/mail/contracts/openapi/mail.yaml, crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage | none |
