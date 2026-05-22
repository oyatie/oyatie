---
doc_class: ImplementationPlan
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0251, ADR-0255, ADR-0244, ADR-0263]
acceptance_status: draft
companion_docs:
  - microservices/mail/policy/phi-dlp.cedar
  - microservices/mail/packs/HIPAA.md
  - microservices/mail/runbooks/phi-leak-recovery.md
  - microservices/mail/catalog/oya-mail-phi-dlp-adapter-kernel.yaml
inbound_citations: [microservices/mail/manifest.json]
---

# IP-018: HIPAA overlay rollout

## A. Problem

The mail PRD allows `audience_type=B2B_HIPAA_PHI`, but mail cannot claim HIPAA-ready behavior merely because a pack file exists. PHI-bearing mail needs BAA-gated tenant provisioning, PHI DLP, HIPAA-conformant intelligence routing, minimum-necessary audit evidence, and breach recovery. The earlier shell did not connect those controls to the actual mail policy, catalog, runbook, or SLO surfaces.

This IP makes the HIPAA overlay a concrete activation path for Work Mail tenants that handle PHI. Personal mail stays outside tenant-admin reach under dual-context isolation.

## B. Approach

Treat HIPAA as a compliance-pack overlay over existing mail primitives: `policy/phi-dlp.cedar` authorizes PHI movement, `oya-mail-phi-dlp-adapter-kernel` classifies PHI, `packs/HIPAA.md` defines pack obligations, `runbooks/phi-leak-recovery.md` handles breach operations, and `contracts/openapi/mail.yaml`/`contracts/proto/mail.proto` carry `DataClass::PHI`.

Provisioning is gated by BAA status and tenant class. Outbound PHI is allowed only to attested partner domains or explicitly approved recipients. Intelligence calls on PHI mailboxes must route to HIPAA-conformant variants declared by the intelligence routing matrix; nonconformant assistants are denied rather than degraded silently.

## C. Deliverables

| Artifact | Change |
|---|---|
| `microservices/mail/packs/HIPAA.md` | make BAA, PHI DLP, breach-notification, and minimum-necessary evidence concrete |
| `microservices/mail/policy/phi-dlp.cedar` | enforce outbound partner attestation, PHI classifier verdicts, and release authority |
| `microservices/mail/catalog/oya-mail-phi-dlp-adapter-kernel.yaml` | bind classifier crate to PHI findings and data-class tags |
| `microservices/mail/contracts/proto/mail.proto` | use `DATA_CLASS_PHI` on `MailMessage` and quarantine/release workflows |
| `microservices/mail/runbooks/phi-leak-recovery.md` | drive suspected PHI leak containment and HHS breach-clock handling |
| `microservices/mail/dashboards/security-dlp.json` | show PHI detections, blocked sends, release approvals, and PHI-leak incident state |

## D. Implementation

1. Add a provisioning check: `B2B_HIPAA_PHI` tenant mailboxes require BAA-on-file before mailbox activation and before PHI DLP policy can enter allow mode.
2. Extend the PHI classifier plan in `oya-mail-phi-dlp-adapter-kernel` to emit `DataClass::PHI`, confidence, detected category, and redacted evidence digest.
3. Update `policy/phi-dlp.cedar` so outbound PHI requires partner-domain attestation, tenant HIPAA pack activation, and a non-expired BAA.
4. Route assistant/summarization requests from PHI mailboxes only to HIPAA-conformant intelligence variants; deny and audit when a nonconformant model is selected.
5. Emit ADR-0263 events for PHI detected, PHI outbound denied, PHI outbound released, PHI access viewed, and PHI leak recovery opened.
6. Bind DLP dashboard panels and `runbooks/phi-leak-recovery.md` to the same event names, not parallel incident vocabulary.
7. Add tests for BAA missing, partner attestation missing, Personal mailbox PHI hold forbidden, nonconformant intelligence denied, and approved partner outbound allowed.
8. Confirm pack overlay does not change retention or discovery behavior for non-HIPAA tenants.

## E. Acceptance

- HIPAA mailbox provisioning fails closed when BAA status is missing or expired.
- `policy/phi-dlp.cedar` blocks outbound PHI to un-attested domains and records a reviewable denial.
- `DATA_CLASS_PHI` is preserved through message read, search result, quarantine, release, and audit evidence paths.
- PHI assistant calls cannot route to non-HIPAA intelligence variants.
- `runbooks/phi-leak-recovery.md` can be executed from emitted incident events without inventing additional state.

## F. Evidence

- `microservices/mail/PRD.md` defines `B2B_HIPAA_PHI` and HIPAA pack behavior.
- `microservices/mail/ARCHITECTURE.md` names `policy/phi-dlp.cedar` in the cold-start route.
- `microservices/mail/contracts/proto/mail.proto` defines `DATA_CLASS_PHI`.
- `microservices/mail/competitor-parity-matrix.md` compares HIPAA/BAA behavior across Gmail Vault, Exchange Purview, Proton, Fastmail, and Naver.
- HHS HIPAA Security and Breach Notification Rules anchor the regulatory obligations.

## G. Counterparts

| Counterpart | Gap closed by this IP |
|---|---|
| Google Workspace Gmail | Matches enterprise HIPAA/BAA expectations while adding explicit PHI DLP and audit-chain evidence. |
| Microsoft Exchange Online / Purview | Narrows PHI governance parity while preserving Oyatie dual-context refusal for Personal mail. |
| Proton / Fastmail | Keeps privacy posture but adds covered-entity operational controls those products do not provide as native enterprise workflow. |

## H. Non-goals and handoff boundaries

- Do not claim universal HIPAA compliance; the overlay is conditional on BAA, pack activation, and tenant workflow.
- Do not give tenant admins any Personal-pillar legal-hold or eDiscovery ability.
- Do not route PHI to generic intelligence variants, even in degraded mode.
- Do not store raw PHI snippets in metrics; dashboards use counts, redacted digests, and audit refs.
- Do not convert DLP denials into generic send failures; the user and auditor need a specific PHI policy outcome.

## I. Fixture set

- `hipaa_missing_baa_provisioning.json` proves activation is blocked.
- `phi_to_unattested_partner.eml` proves outbound denial.
- `phi_to_attested_partner.eml` proves allowed send with audit refs.
- `personal_mailbox_phi_hold_attempt.json` proves dual-context refusal.
- `phi_assistant_nonconformant_model.json` proves intelligence routing denial.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/mail/IP-018-hipaa-overlay-rollout.md` matched `openapi, .proto`; contract files `microservices/mail/contracts/openapi/mail.yaml, microservices/mail/contracts/asyncapi/mail-events.yaml, microservices/mail/contracts/proto/mail.proto`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/mail/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/mail/IP-018-hipaa-overlay-rollout.md` matched `SLO, PHI`; anchors `microservices/mail/runbooks/mailbox-restore-from-backup.md, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.
