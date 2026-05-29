---
id: ADR-MSG-001
title: MLS RFC 9420 E2EE Key Delivery Architecture
status: Accepted
date: 2026-05-20
microservice: messenger
related_oyatie_adrs:
  - docs/decisions/ADR-0002-tenant-and-identity-kernel.md
  - docs/decisions/ADR-0003-audit-chain-and-evidence-emission.md
  - docs/decisions/ADR-0007-cedar-authorization-policy-and-persona-tier.md
  - docs/decisions/ADR-0008-data-use-boundary.md
  - docs/decisions/ADR-0043-secrets-management-openbao-and-hsm-per-cell.md
decision_owner: axis-messenger
---

# ADR-MSG-001: MLS RFC 9420 E2EE Key Delivery Architecture

## Context

- Messenger owns direct messages, group channels, war rooms, crisis channels, cross-tenant support threads, and regulated export workflows.
- The service already declares `PII_IDENTIFYING`, `AUTHENTICATION`, and `AUDIT` data classes in `microservices/messenger/ARCHITECTURE.md`.
- The existing runbook `microservices/messenger/runbooks/e2e-encryption-key-rotation.md` needs a binding service-level decision for key delivery.
- The workspace standard `docs/standards/messenger-e2e-encryption-mls.md` names MLS as the long-term encryption direction, but this ADR binds the messenger-local architecture.
- Named precedent: WhatsApp Sender Keys solved group fanout, but MLS RFC 9420 is the IETF standard pattern closer to hyperscaler-grade group membership churn.
- Named precedent: Signal's sealed sender informs metadata-minimization, but Oyatie must keep audit-chain evidence for regulated tenants without retaining plaintext.
- Constraint MSG-C1: tenant and principal scope must come from ADR-0002 identity and tenancy primitives, never from caller-supplied channel metadata.
- Constraint MSG-C2: every key-package fetch, welcome publish, commit merge, and recovery denial must emit audit evidence per ADR-0003.
- Constraint MSG-C3: Cedar must authorize membership, export, recovery, and device-add actions before any key material is released per ADR-0007.
- Constraint MSG-C4: plaintext content remains outside server custody; metadata is minimized but still classified per ADR-0008.
- Constraint MSG-C5: any server-held wrapping material uses OpenBao / HSM-backed leases per ADR-0043 with tenant and cell path separation.
- Constraint MSG-C6: emergency channels and survivor-safety channels cannot silently downgrade encryption to preserve availability.
- Constraint MSG-C7: eDiscovery export must never require the messenger service to decrypt message ciphertext.
- Constraint MSG-C8: cross-device onboarding must survive lost devices without introducing tenant-wide escrow.
- Constraint MSG-C9: group membership churn for 100k-member channels must not make send latency exceed the message-send SLO.
- Constraint MSG-C10: sovereign packs may require disabling cross-region delivery of encrypted key packages even when ciphertext is unreadable.
- Current service surfaces include `contracts/openapi/messenger.yaml`, `contracts/asyncapi/messenger-events.yaml`, and `contracts/proto/messenger.proto`.
- Existing policy surfaces include `policy/channel-scope.cedar`, `policy/dual-context-isolation.md`, `policy/tenant-scope.cedar`, and auditor / CI fragments.
- The architecture must support personal DMs, work channels, cross-tenant cohorts, and regulated war rooms with different retention overlays.
- The architecture must make server-side compromise detectably harmful but not content-revealing.
- The architecture must make client compromise containable to the compromised device epoch.
- The architecture must make forced rekey observable, rate-limited, and reversible at the membership graph level.
- The architecture must keep read receipt and presence fanout outside the MLS ciphertext channel because they have different retention and latency shapes.
- The architecture must preserve offline delivery for mobile clients without storing plaintext or unwrapped group secrets.

## Decision

- Use MLS RFC 9420 as the messenger end-to-end encryption protocol for DMs, private channels, regulated war rooms, and cross-tenant confidential threads.
- Implement one MLS group per messenger conversation, identified by `mls_group_id`, not by mutable channel slug.
- Use ciphersuite `MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519` for the default tier.
- Allow pack-gated upgrade to `MLS_256_DHKEMP384_AES256GCM_SHA384_P384` only when the tenant's compliance pack requires it.
- Represent every device as an MLS leaf with a stable `device_id`, a short-lived `key_package_id`, and an identity-bound credential.
- Bind the MLS credential to `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `device_attestation_ref`, and `credential_epoch`.
- Store server-visible MLS artifacts in a `messenger_key_delivery` bounded context.
- Persist KeyPackage objects encrypted at rest, but treat them as public prekeys once uploaded.
- Persist Welcome messages for offline devices with a per-device delivery cursor and a 14-day default TTL.
- Persist Commit messages on the event stream so clients can replay group state deterministically.
- Use the existing Kafka / CloudEvents backbone for key-delivery events, not a separate key-distribution broker.
- Emit `oya.messenger.mls.key_package.uploaded.v1` after a device publishes a usable KeyPackage.
- Emit `oya.messenger.mls.welcome.enqueued.v1` when a new device or member receives a Welcome object.
- Emit `oya.messenger.mls.commit.accepted.v1` when the server validates sequencing and Cedar membership.
- Emit `oya.messenger.mls.epoch.rejected.v1` when a client tries to send under a stale or unauthorized epoch.
- Use Cedar policy `messenger::mls_key_package::read` for KeyPackage fetches.
- Use Cedar policy `messenger::mls_welcome::enqueue` for Welcome publication.
- Use Cedar policy `messenger::mls_commit::append` for Commit merge.
- Use Cedar policy `messenger::mls_recovery::request` for account recovery flows.
- Reject any server-side plaintext recovery path; recovery rotates device membership and replays history only where the client owns prior exported backup secrets.
- Use OpenBao only for server signing keys, delivery-token MAC keys, and abuse-throttle secrets, not for message decryption keys.
- Separate server key paths as `secret/<tenant_id>/messenger/mls-delivery/<cell_id>/<purpose>`.
- Use a rolling 30-day server signing key with overlap window of 48 hours for mobile offline replay.
- Use MLS external commit for device replacement after passkey step-up.
- Use MLS remove proposal for lost or compromised devices.
- Use MLS group-context extensions to carry `tenant_id`, `conversation_id`, `data_class`, `retention_class`, and `pack_set_hash`.
- Treat metadata fields as audit-relevant and classify them under ADR-0008 even though ciphertext is opaque.
- Keep search indexing metadata-only for E2EE channels unless a tenant-controlled client-side index export is explicitly installed.
- Keep eDiscovery export ciphertext-only plus event and membership metadata; actual plaintext export is a client-side or tenant-controlled legal-hold appliance function.
- Use the messenger service to prove delivery, membership, epoch, and retention facts, not to reconstruct content.

## Alternatives Considered

### Signal Double Ratchet Per Pair

- Pros: strong precedent for 1:1 confidentiality.
- Pros: mature mobile client mental model.
- Pros: simple for direct messages with two active devices.
- Cons: group sender-key distribution becomes bespoke once channels exceed a few devices.
- Cons: member removal correctness is hard to prove across offline devices.
- Cons: per-pair fanout cost grows with device count.
- Rejected because messenger's work channels and regulated war rooms need auditable group membership churn at scale.

### Server-Side Envelope Encryption Only

- Pros: easiest eDiscovery and malware scanning.
- Pros: simpler search, preview, and retention workflows.
- Pros: easier incident response when clients are thin.
- Cons: server compromise exposes message content.
- Cons: violates the product posture of encrypted personal and confidential work conversations.
- Cons: creates a privileged plaintext processor in a product service.
- Rejected because it fails the server-compromise threat model and weakens ADR-0008 data-use boundaries.

### Per-Tenant Escrowed Group Key

- Pros: operationally attractive for enterprise recovery.
- Pros: export workflows are simple.
- Pros: device onboarding is fast.
- Cons: one tenant key compromise decrypts all tenant conversations.
- Cons: impossible to express least privilege for personal and work dual-context boundaries.
- Cons: incompatible with survivor-safety and whistleblower channels.
- Rejected because the blast radius is unacceptable and conflicts with identity-scoped authorization.

### MLS With Server-Retained Application Secrets

- Pros: preserves MLS membership semantics while giving server search and export.
- Pros: easier compliance story for some tenants.
- Cons: defeats the point of E2EE.
- Cons: creates permanent secrets that must be rotated and escrowed.
- Cons: makes OpenBao a plaintext recovery system instead of a signing and lease system.
- Rejected because the service must not hold application secrets.

## Consequences

- Positive: group membership state is a first-class, standards-based protocol artifact.
- Positive: device addition and removal can be proven by epoch history instead of inferred from channel ACL rows.
- Positive: server-side breach reveals metadata and pending KeyPackages, not message plaintext.
- Positive: Cedar authorization integrates with membership changes before key material delivery.
- Positive: audit-chain evidence can prove who was authorized at which epoch without storing message bodies.
- Positive: cross-tenant channel access can be represented as scoped membership, not copied shared secrets.
- Positive: mobile offline delivery remains available through queued Welcome objects and Commit replay.
- Positive: recovery can be safe-by-default because it creates a new epoch rather than decrypting old history.
- Negative: full-text server search is unavailable for E2EE channels.
- Negative: malware scanning must happen on attachments before client encryption or in tenant-controlled client scanning.
- Negative: client implementation complexity increases because clients must persist MLS epoch state correctly.
- Negative: eDiscovery depends on tenant-owned legal-hold endpoints for plaintext, not central server decryption.
- Negative: group churn spikes can create large Commit bursts for very large channels.
- Negative: support staff cannot recover user-visible message bodies from server data.
- Neutral: read receipts, typing indicators, presence, and huddle signaling remain separate metadata channels.
- Neutral: public channels may opt out of MLS and use server-side encryption when policy declares them public.
- Neutral: retention rules apply to ciphertext, membership, audit events, and attachment references independently.
- Neutral: a tenant pack may require stricter ciphersuites without changing the service architecture.
- Neutral: MLS protocol upgrades require client/server compatibility windows and feature flags.

## Implementation Notes

- Data shape `MlsKeyPackage`: `{tenant_id, principal_id, device_id, key_package_id, credential_epoch, ciphersuite, key_package_bytes, attestation_ref, expires_at}`.
- Data shape `MlsGroupState`: `{tenant_id, conversation_id, mls_group_id, epoch, tree_hash, confirmed_transcript_hash, pack_set_hash, retention_class}`.
- Data shape `MlsWelcomeQueue`: `{tenant_id, mls_group_id, recipient_device_id, welcome_id, encrypted_welcome_bytes, expires_at, delivery_state}`.
- Data shape `MlsCommitEnvelope`: `{tenant_id, mls_group_id, epoch, sender_device_id, commit_bytes, ratchet_tree_ref, audit_event_id}`.
- Data shape `MlsRecoveryRequest`: `{tenant_id, principal_id, replacement_device_id, passkey_assertion_id, recovery_reason, requested_at}`.
- REST endpoint `POST /v1/messenger/mls/key-packages` uploads a signed KeyPackage.
- REST endpoint `GET /v1/messenger/mls/key-packages/{principal_id}` returns valid KeyPackages after Cedar evaluation.
- REST endpoint `POST /v1/messenger/conversations/{conversation_id}/mls/commits` appends a Commit proposal or Commit.
- REST endpoint `GET /v1/messenger/conversations/{conversation_id}/mls/epoch/{epoch}` returns Commit replay references for a device.
- REST endpoint `POST /v1/messenger/devices/{device_id}/recovery/external-commit` starts passkey-verified device replacement.
- AsyncAPI channel `messenger.mls.key-package.uploaded.v1` publishes KeyPackage availability.
- AsyncAPI channel `messenger.mls.commit.accepted.v1` publishes epoch advancement.
- AsyncAPI channel `messenger.mls.device.removed.v1` publishes revoked leaf state.
- gRPC method `MessengerKeyDelivery.GetPendingWelcomes` supports mobile sync.
- gRPC method `MessengerKeyDelivery.AckWelcome` marks offline Welcome delivery complete.
- Cedar policy `permit(principal, Action::"messenger::mls_key_package::read", resource)` requires same tenant or explicit cross-tenant channel membership.
- Cedar policy `forbid(principal, Action::"messenger::mls_commit::append", resource)` when `principal.device_revoked == true`.
- Cedar policy `permit(principal, Action::"messenger::mls_recovery::request", resource)` requires `context.webauthn_aal >= 3`.
- Cedar policy `forbid(principal, Action::"messenger::mls_export_plaintext", resource)` is unconditional in the service.
- Audit event `EVT-MSG-MLS-KEY-PACKAGE-UPLOADED` includes key package hash, not key material.
- Audit event `EVT-MSG-MLS-COMMIT-ACCEPTED` includes epoch, tree hash, and membership delta.
- Audit event `EVT-MSG-MLS-RECOVERY-DENIED` includes denial reason and Cedar policy id.
- Metric `messenger_mls_commit_accept_latency_ms` is a histogram with dimensions `cell_id`, `pack_family`, and `group_size_bucket`.
- Metric `messenger_mls_epoch_reject_total` counts stale, unauthorized, and malformed epoch submissions.
- Metric `messenger_mls_pending_welcome_age_seconds` tracks offline delivery age.
- Capacity math: for a 100k-member channel with 2.5 devices/member, direct Welcome fanout is 250k objects; commits are chunked into 1k-recipient batches to cap poller memory at 256 MiB.
- Capacity math: if p95 key-package fetch is 20 ms and peak join rate is 500 devices/s, Little's Law gives roughly 10 in-flight fetches per cell before safety factor; provision 100 to absorb burst.
- Rollback path: keep prior epoch server-side accepted until clients converge, but deny new sends under a bad epoch and publish a corrective Commit.
- Rollback path: revert Cedar fragment pointer for key delivery, then replay queued commits through the previous policy bundle.
- Multi-region path: KeyPackage lookup stays in tenant home cell; remote cells receive metadata-only replication for notification fanout.
- Sovereign-cell path: KR, EU, FedRAMP-High, and CN-PIPL overlays can disable cross-cell Welcome queues.
- Versioning: API version `v1` is additive only; MLS ciphersuite changes require a new `credential_epoch`.
- Deprecation: KeyPackage schema fields are supported for at least two mobile LTS releases before removal.

## Verification

- Unit test `mls_key_package_rejects_cross_tenant_fetch` proves Cedar denies unauthorized KeyPackage reads.
- Unit test `mls_commit_requires_current_epoch` proves stale epochs emit `EVT-MSG-MLS-EPOCH-REJECTED`.
- Unit test `mls_recovery_never_returns_plaintext_secret` proves recovery paths produce external commits only.
- Property test `mls_membership_delta_round_trips` generates add, update, remove, and external-commit sequences.
- Property test `mls_pack_set_hash_changes_force_rekey` proves compliance pack changes trigger a new epoch.
- Fuzz test `mls_commit_parser_rejects_malformed_ratchet_tree` covers malformed client bytes.
- Integration test `cross_tenant_dm_requires_explicit_cedar_membership` covers cohort and support-thread cases.
- Integration test `offline_device_receives_welcome_once` verifies exactly-once delivery acknowledgement.
- Integration test `lost_device_remove_blocks_future_commits` covers revoked leaf replay.
- Integration test `ediscovery_export_contains_ciphertext_and_membership_only` prevents plaintext server export.
- Load test `mls_large_channel_join_100k_members` verifies p99 Commit accept latency under 500 ms.
- Load test `mls_key_package_fetch_500_devices_per_second` verifies p95 below 50 ms per cell.
- Soak test `mls_mobile_offline_14_day_replay` keeps Welcome queue expiration semantics stable.
- Chaos test `audit_chain_backpressure_stops_key_release` proves high-risk operations fail closed when evidence cannot be emitted.
- Chaos test `openbao_signing_key_rotation_overlap` verifies the 48-hour overlap accepts old signatures and rejects expired ones.
- Metric SLO: `messenger_mls_commit_accept_latency_ms` p99 below 500 ms for group size bucket `100k`.
- Metric SLO: `messenger_mls_epoch_reject_total` error-budget burn below 1 percent of sends outside active incidents.
- Metric SLO: `messenger_mls_pending_welcome_age_seconds` p95 below 300 seconds for online devices.
- Audit check: every accepted Commit has one `EVT-MSG-MLS-COMMIT-ACCEPTED` chain event.
- Audit check: every recovery denial has one `EVT-MSG-MLS-RECOVERY-DENIED` event.
- Static check: no server crate exposes a `decrypt_message` or `export_plaintext` function for MLS content.
- Static check: OpenBao references use `secret/<tenant_id>/messenger/mls-delivery/<cell_id>/...`.
- Link check: this ADR must remain reachable from `microservices/messenger/README.md` or a decisions index.

