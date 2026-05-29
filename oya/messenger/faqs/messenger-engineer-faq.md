---
doc_class: FAQ
microservice: messenger
persona: messenger-engineer + e2ee-platform-engineer + trust-and-safety-engineer
related_adrs: [ADR-MSG-001, ADR-MSGR-0001, ADR-MSGR-0002, ADR-MSGR-0003, ADR-MSGR-0004, ADR-0254]
date: 2026-05-20
doc_status: published
---

# Messenger Engineer FAQ — messenger

## Why MLS RFC 9420 instead of Signal Double Ratchet, Sender Keys, or server-side envelope?

Per ADR-MSG-001 § Alternatives Considered. Three reasons:

1. **Signal Double Ratchet** is excellent for 1:1 conversations but group sender-key distribution becomes bespoke once channels exceed a handful of devices. Member-removal correctness is hard to prove across offline devices. Per-pair fanout cost grows linearly with device count.
2. **Server-side envelope encryption only** (Slack/Teams default) exposes message content on server compromise. That is a non-starter for the product posture of encrypted personal + confidential work conversations.
3. **Per-tenant escrowed group key** (some enterprise vendors) — one tenant-key compromise decrypts all tenant conversations. Impossible to model least-privilege for personal/work dual-context. Incompatible with survivor-safety and whistleblower channels.

MLS is IETF-standardized (RFC 9420, published July 2023), gives O(log N) group operations for arbitrary-size channels, has formal security analysis (TreeKEM), and is implemented across multiple interoperable libraries (openmls, mlspp, mls-rs).

## What's the difference between demo_trial ciphersuite vs paid advanced/paid compliance-pack ciphersuite?

Per ADR-MSG-001 § Decision:

- **Default (demo_trial + paid)**: `MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519` — X25519 KEM, AES-128-GCM symmetric, SHA-256 hash, Ed25519 signatures. Fast, widely supported, FIPS-140-3 L2 compatible.
- **High-assurance (paid advanced pack tenants + paid compliance-pack mandatory)**: `MLS_256_DHKEMP384_AES256GCM_SHA384_P384` — P-384 KEM, AES-256-GCM symmetric, SHA-384, P-384 signatures. Required for FIPS-140-3 L3, KR-PIPA Art 23 (high-sensitivity), US-HIPAA PHI conversations, FedRAMP-High.

Tenant pack governs which is required; pack-overlay precedence per `ADR-COMP-001` enforces higher-restriction-wins.

## How does the server NEVER see plaintext, yet support eDiscovery?

Per ADR-MSG-001 Constraint MSG-C7 + § Decision. The server holds:

- **KeyPackages** (public prekeys; encrypted at rest but treated as public).
- **Welcome messages** (encrypted to recipient devices' KEM public keys; opaque to server).
- **Commit messages** (signed by sender; carry epoch advancement; encrypted application secrets).
- **Membership metadata** (audit-chain-sealed; who joined which group at which epoch).
- **Audit-chain events** (`EVT-MSG-MLS-*` series; cryptographically sealed).

The server does NOT hold:

- Application secrets (the keys derived by MLS that actually encrypt message content).
- Decrypted message text.
- Per-device unwrapping keys.

For eDiscovery, the server exports ciphertext + membership + audit-chain. The **tenant-controlled legal-hold appliance** (deployed in the tenant's compliance boundary) holds the keys to decrypt for legal review. This satisfies "we cannot read your messages even under subpoena" while allowing tenants to honor lawful eDiscovery requests.

## What is the difference between channel slug and `mls_group_id`?

Per ADR-MSG-001 § Decision: "Implement one MLS group per messenger conversation, identified by `mls_group_id`, not by mutable channel slug."

- Channel slug (e.g., `#sec-engineering`) is **mutable** for UX.
- `mls_group_id` (e.g., `mg_e5a4b3c2d1...`) is **immutable** and binds the cryptographic group identity.

Why: if slug were the identity, renaming `#general` to `#general-archive` and creating a new `#general` could let a malicious admin redirect messages to a different cryptographic group while users believe they're still in the original conversation. Immutable `mls_group_id` makes that attack impossible.

## How does presence sync work without leaking metadata patterns?

Per ADR-MSG-001 § Decision: "Keep read receipt and presence fanout outside the MLS ciphertext channel because they have different retention and latency shapes."

Presence + typing + read receipts go through a **separate Pulsar topic** (`messenger.presence.v1`) with:

- Short retention (60 s).
- Per-tenant scope (no cross-tenant presence leakage even if cross-tenant DM is permitted).
- Aggregation only — server publishes `u-alice is online in tenant drill-acme`, not `u-alice is online and last seen typing in conversation c_001`.
- Cedar `messenger::presence::read` per requester-tenant boundary.

Presence is opt-in per tenant policy. High-risk packs (whistleblower, healthcare break-glass) can disable presence entirely.

## What's the thread model? Are threads E2EE too?

Yes. Each thread is a child conversation with its own `mls_group_id` derived from the parent group's epoch + thread-id (per IP-thread-derivation-005). Thread membership is a subset of parent membership; Cedar enforces "thread member ⊆ parent member".

Threads share the parent's ciphersuite. Thread Commits advance the thread's epoch independently from the parent.

## Why do reactions work on E2EE messages?

Per IP-reactions-engine. Reactions are encrypted under the MLS group epoch (same as messages) but are addressed differently:

- A reaction `{message_id: m_001, emoji: "👍"}` is encrypted as an MLS Application Message with type `reaction`.
- Server stores ciphertext + targeting metadata (which message_id it reacts to).
- Server aggregates counts per `(message_id, emoji)` for the UI without decrypting (clients decrypt the reaction payloads to verify the emoji and the actor).

This means the server CAN count "5 reactions on m_001" but CANNOT see which emoji or which user reacted without client-side decrypt.

## How is per-tenant cross-tenant disclosure policy enforced?

Per ADR-MSGR-0004 + ADR-MSG-001 Cedar gates. Three federation modes:

1. **Isolated** (default demo_trial): no cross-tenant DMs. Cedar denies `messenger::dm::create` when `principal.tenant_id != target.tenant_id`.
2. **Opt-in tenant-pair allowlist** (paid): both tenants must explicitly grant `messenger::federation::tenant_pair_grant`. Grant scope is one of: `dm_only`, `channel_membership`, `huddle_invite`. Time-bounded; default 90 d.
3. **Cohort channels** (paid advanced): multi-tenant channels with `member_eligibility=verified-corp-email`. Each tenant maintains its own Cedar control over which members can join.

Cross-tenant disclosure NEVER allows reading another tenant's conversations the user is not a member of. Federation grants only enable joining shared conversations; existing per-tenant conversations remain isolated.

## What happens to messages when a member is removed?

Per RFC 9420 § 12 + ADR-MSG-001. Removal:

1. Remaining member sends MLS Remove proposal.
2. Anyone with `messenger::mls_commit::append` permission (typically tenant admin or the removed member's manager via Cedar) merges a Commit advancing the epoch.
3. New epoch's application secrets are derived without the removed member's leaf.
4. Removed member's device can no longer decrypt new messages (forward-secrecy guarantee).
5. Messages encrypted BEFORE the removal remain readable to the removed member (history is not retroactively re-encrypted; this matches RFC 9420 + WhatsApp + Signal semantics).

If the tenant requires "burn-after-leave" semantics (rare), per-conversation ephemeral mode can re-derive the entire history under a new root key, but this requires every active device to be online and confirm; the default is "history preserved at the leaver's last epoch".

## How does the personal-DM key escrow toggle work (ADR-MSGR-0002)?

Per ADR-MSGR-0002. Two modes:

- **Mode A: No escrow (default)**: keys live only on user devices + their KeyPackages. If user loses ALL devices + recovery envelope, their old messages are unrecoverable. Maximum confidentiality.
- **Mode B: Tenant escrow (B2B opt-in)**: tenant's KMS (under tenant CMK) wraps a recovery secret per personal-DM group. Tenant admin can recover messages with Cedar `messenger::dm::tenant_recovery` + court-order evidence + dual-approval.

Personal tenants (non-B2B; B2C users in their own personal-tenant) ALWAYS get Mode A. No exceptions per `feedback_oyatie_is_a_tenant_doctrine` + dual-context isolation.

## What's the AAGUID policy for KeyPackages?

Per IP-006-aaguid-refresh-worker. Each KeyPackage upload includes the device's WebAuthn AAGUID. Server consults the AAGUID trust catalog:

- **Allowed**: Apple platform passkeys, Google passkeys, YubiKey 5C NFC, Feitian K9, hardware-backed Android passkeys.
- **Allowed for low-risk only**: synced passkeys from less-attested vendors.
- **Denied**: revoked AAGUIDs (compromised vendor batches; e.g., a hypothetical Yubico recall).

Pack-bound tenants (paid compliance-pack) may require `attestation_class >= hardware_bound` for ALL KeyPackages.

## How are spam DMs prevented in cross-tenant federation?

Per IP-spam-cross-tenant-rate-limit. Layers:

1. Per-user rate limit on cross-tenant DM initiation (default 5/h).
2. Verified-corporate-email requirement (paid) — must have completed identity verification per the community-µservice flow.
3. Tenant-pair federation grant explicit (Cedar `messenger::federation::tenant_pair_grant`).
4. LLM-classifier on cross-tenant DM content (post-decrypt on the recipient client; server cannot see plaintext).
5. Per-recipient block-list (client-side; recipient can block sender across all conversations).

## What's the FedRAMP/CN-PIPL/DORA story for paid compliance-pack?

Per ADR-MSG-001 sovereign-cell path:

- **FedRAMP-High**: all signing keys + KeyPackages + Welcome queues stay in US-Gov cells (us-gov-east-1 + us-gov-west-1). No cross-region replication. `MLS_256_DHKEMP384_AES256GCM_SHA384_P384` mandatory.
- **CN-PIPL**: keys + ciphertext stay in CN cells (cn-north-1 + cn-northwest-1). PIPL Art 38 cross-border transfer assessment required for any export.
- **DORA** (EU financial sector): operational resilience evidence (chaos-test reports, RPO/RTO drills) audit-chain-sealed; regulator can request audit-chain replay per ADR-0303.
- **KSA-SDAIA**: keys + ciphertext stay in me-south-1; SDAIA regulator export gateway available per IP-regulator-response-shaper.

## What if MLS RFC 9420 gets a critical vulnerability?

Per ADR-MSG-001 § Implementation Notes versioning. The protocol upgrade path:

1. New ciphersuite published as a feature flag (e.g., `MLS_PROFILE_V2_HYBRID_PQ_KYBER768`).
2. New tenants opt-in; existing tenants gated by tenant admin.
3. Migration ceremony: each conversation runs an MLS commit transitioning to the new ciphersuite. Old + new ciphersuites accepted in parallel for 30 d (per `feedback_no_silent_regression`).
4. After migration window, old ciphersuite denied for new commits.

Critical CVE accelerates the window to 7 d with regulator notification + audit-chain seal.

## How does this differ from `community`?

- `messenger`: one-to-one or group conversations; default E2EE; ephemeral by default; presence + typing + read receipts.
- `community`: many-to-many publication (forums, posts, comments); public or board-scoped; not E2EE (content is broadcast); reputation + moderation surface.

Boundary: a "DM in response to a forum post" is messenger; a "comment on a forum post" is community. The cross-µservice bridge is `oya-bridge-community-messenger-dm` (per IP-bridge-community-messenger-001).
