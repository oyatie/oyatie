---
doc_class: Standard
title: MLS RFC 9420 Conformance Standard
status: Accepted
date: 2026-05-20
owner: axis-messenger + council-security + council-privacy
related_oyatie_adrs:
  - ADR-0188
  - ADR-0240
  - ADR-0242
  - ADR-0251
  - ADR-0253
enforced_by:
  - governance-mls-rfc-9420-conformance
  - governance-dual-context-isolation
  - governance-crypto-test-vectors
canonical_paths:
  - docs/standards/messenger-e2e-encryption-mls.md
  - microservices/messenger/
  - microservices/messenger/decisions/ADR-MSG-001-mls-e2ee-key-delivery-architecture.md
  - microservices/messenger/decisions/ADR-MSGR-0002-e2e-personal-dm-key-escrow.md
external_reference:
  - https://www.rfc-editor.org/rfc/rfc9420.html
---

# MLS RFC 9420 Conformance Standard

This standard converts the Messenger MLS design into an enforceable RFC 9420
conformance bar. RFC 9420 defines Messaging Layer Security as a protocol for
asynchronous group keying with forward secrecy and post-compromise security. In
Oyatie, MLS applies to personal-context messenger encryption and related Meet
signaling surfaces; professional-context channels remain governed by tenant
envelope encryption, DLP, eDiscovery, and Cedar policy.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT,
RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are interpreted
as described in RFC 2119 and RFC 8174 when they appear in all capitals.

## Scope

This standard applies to personal-context messenger MLS groups.

It applies to one-to-one MLS groups.

It applies to personal group chats.

It applies to personal device groups.

It applies to personal attachment key wrapping.

It applies to Meet signaling when MLS derives session keys.

It applies to server-side delivery services that store MLS ciphertext.

It applies to KeyPackage storage and validation.

It applies to Welcome, Commit, Proposal, and application-message handling.

It does not apply to professional-context server-readable channels.

It does not apply to audit-chain plaintext visibility.

It does not define federation beyond documented posture.

## Normative Requirements

M-001. Personal messenger conversations MUST use MLS for E2EE.

M-002. Professional channels MUST NOT be silently upgraded to MLS when compliance visibility is required.

M-003. Every MLS group MUST declare protocol version.

M-004. Every MLS group MUST declare cipher suite.

M-005. Every client MUST support the default suite selected by `messenger-e2e-encryption-mls.md`.

M-006. Every client SHOULD support the declared fallback suite.

M-007. Every client MUST reject unsupported cipher suites.

M-008. Every group MUST advance epoch on membership change.

M-009. Every group MUST advance epoch on key update.

M-010. Every group MUST process at most one Commit per epoch.

M-011. Clients MUST verify tree hash when group membership changes.

M-012. Clients MUST verify transcript hash when processing commits.

M-013. Clients MUST verify sender authentication.

M-014. Clients MUST reject malformed PublicMessage frames.

M-015. Clients MUST reject malformed PrivateMessage frames.

M-016. Clients MUST reject stale KeyPackages.

M-017. Clients MUST reject reused one-time KeyPackages.

M-018. KeyPackages MUST expire.

M-019. KeyPackages MUST bind to a device identity.

M-020. KeyPackages MUST bind to credential identity.

M-021. KeyPackage publication MUST be rate-limited.

M-022. KeyPackage publication MUST be auditable.

M-023. Welcome messages MUST be encrypted for intended recipients only.

M-024. Welcome messages MUST not expose plaintext group secrets to the server.

M-025. Commit messages MUST be persisted before dependent application messages are accepted.

M-026. Application messages MUST be rejected when the local epoch is missing.

M-027. Application messages MUST be associated with a conversation id.

M-028. Application messages MUST carry replay protection.

M-029. Attachment keys MUST be generated per attachment.

M-030. Attachment keys MUST be wrapped inside MLS application messages.

M-031. Attachment ciphertext MUST be stored separately from plaintext.

M-032. The server MUST NOT receive message plaintext.

M-033. The server MUST NOT receive MLS group secrets.

M-034. The server MUST NOT forge KeyPackages.

M-035. The server MUST validate envelope metadata.

M-036. The server MUST preserve ordering semantics required by the client.

M-037. The server MUST expose delivery receipts without plaintext.

M-038. The server MUST emit audit events for key package publish, consume, and revoke.

M-039. The server MUST emit audit events for suspicious replay attempts.

M-040. The server MUST emit metrics for MLS processing failures.

M-041. Clients MUST support device removal.

M-042. Device removal MUST advance epoch.

M-043. Lost-device recovery MUST not expose previous plaintext to the server.

M-044. Backup recovery MUST follow the documented key escrow posture.

M-045. Key escrow MUST never grant platform plaintext access for personal content.

M-046. Federation MUST remain disabled unless a federation ADR lands.

M-047. MLS test vectors MUST be run in CI.

M-048. Negative test vectors MUST be run in CI.

M-049. Cross-implementation vectors SHOULD be run when the implementation library changes.

M-050. Library upgrades MUST include security review.

M-051. PQ hybrid suites MUST remain opt-in until IETF and local ADR approval.

M-052. PQ hybrid suites MUST include performance benchmarks.

M-053. MLS state storage MUST be encrypted at rest on clients.

M-054. MLS state storage MUST be scoped per user and device.

M-055. MLS state corruption MUST have a recovery path.

M-056. Push notifications MUST not include personal plaintext.

M-057. Search indexes MUST be client-side or encrypted per documented design.

M-058. Safety-number or verification UX MUST bind to device identity and tree hash.

M-059. Dual-context isolation MUST prevent personal MLS messages entering professional retention stores.

M-060. Dual-context isolation MUST prevent professional admin disclosure paths from reading personal MLS ciphertext.

## Worked Examples

### Example 1: KeyPackage record

```sql
CREATE TABLE messenger_mls_key_packages (
  key_package_id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  device_id TEXT NOT NULL,
  cipher_suite TEXT NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  consumed_at TIMESTAMPTZ,
  key_package BYTEA NOT NULL
);
```

This passes only when `key_package` is MLS-encoded and never decrypted server-side.

### Example 2: Welcome handling

```text
Client receives Welcome.
Client verifies credential.
Client verifies group info signature.
Client stores epoch state.
Client emits local telemetry.
Server sees only delivery metadata.
```

This passes because group secret handling stays client-side.

### Example 3: Invalid server plaintext feature

```text
Server indexes message body for global search.
```

This fails for personal context because MLS plaintext is unavailable to the server.

### Example 4: Device removal

```yaml
action: remove_device
group: personal-device-group
commit_required: true
epoch_advances: true
audit_event: EVT-MESSENGER-MLS-DEVICE-REMOVED-V1
```

This passes because removal advances epoch and emits audit.

### Example 5: Attachment encryption

```yaml
attachment:
  object_store_key: messenger/personal/ciphertext/...
  content_key_wrapped_in: MLS application message
  server_plaintext_access: false
```

This passes because only ciphertext is stored.

## Verification

Primary command:

```bash
presubmit (retired CLI gate validate) mls-rfc-9420-conformance --microservice messenger
```

The checker MUST run RFC 9420 positive vectors.

The checker MUST run malformed frame negative vectors.

The checker MUST run stale epoch tests.

The checker MUST run stale KeyPackage tests.

The checker MUST run one-time KeyPackage reuse tests.

The checker MUST run unsupported cipher suite tests.

The checker MUST run device removal tests.

The checker MUST run server plaintext absence tests.

The checker MUST run personal/professional isolation tests.

The checker MUST run attachment key wrapping tests.

The checker MUST run recovery tests.

The checker MUST run telemetry schema tests.

The checker MUST verify audit event names.

The checker MUST verify storage columns do not include plaintext body for personal content.

The checker MUST verify KeyPackage expiry.

The checker MUST verify KeyPackage consume-once behavior.

The checker SHOULD verify cross-implementation compatibility after library upgrades.

The checker SHOULD measure commit processing latency by group size.

The checker SHOULD report group size envelopes.

## Common Anti-Patterns

Server-side personal plaintext search is an anti-pattern.

Reusing KeyPackages is an anti-pattern.

Accepting multiple commits for one epoch is an anti-pattern.

Treating device identity as user identity is an anti-pattern.

Skipping tree hash verification is an anti-pattern.

Skipping transcript hash verification is an anti-pattern.

Putting professional eDiscovery channels on personal MLS is an anti-pattern.

Putting personal MLS ciphertext into professional retention paths is an anti-pattern.

Sending plaintext push notification bodies is an anti-pattern.

Treating post-quantum draft suites as default is an anti-pattern.

Treating the delivery service as trusted for confidentiality is an anti-pattern.

Treating MLS as a backup protocol is an anti-pattern.

Treating federation as implied by RFC 9420 is an anti-pattern.

Treating client state corruption as data-loss-only without runbook is an anti-pattern.

Treating cipher suite negotiation as a server preference is an anti-pattern.

## Cross-References

External authority: `https://www.rfc-editor.org/rfc/rfc9420.html`.

`docs/standards/messenger-e2e-encryption-mls.md` gives the full design.

`microservices/messenger/decisions/ADR-MSG-001-mls-e2ee-key-delivery-architecture.md` binds key delivery.

`microservices/messenger/decisions/ADR-MSGR-0002-e2e-personal-dm-key-escrow.md` binds escrow posture.

`docs/decisions/ADR-0701-monorepo-capability-live-apex.md` binds device identity.

`docs/decisions/ADR-0708-platform-foundations-live-apex.md` binds sovereign cells.

`docs/decisions/ADR-0708-platform-foundations-live-apex.md` binds compliance pack behavior.

`docs/standards/cedar-policy-authoring.md` binds policy gates for professional surfaces.

## Substance Bar Compliance Checklist

MLS-SB-001. Verify personal context uses MLS.

MLS-SB-002. Verify professional context does not silently use personal MLS posture.

MLS-SB-003. Verify protocol version.

MLS-SB-004. Verify cipher suite.

MLS-SB-005. Verify default suite support.

MLS-SB-006. Verify fallback suite support.

MLS-SB-007. Verify unsupported suite rejection.

MLS-SB-008. Verify epoch advance on add.

MLS-SB-009. Verify epoch advance on remove.

MLS-SB-010. Verify epoch advance on update.

MLS-SB-011. Verify at-most-one commit per epoch.

MLS-SB-012. Verify tree hash.

MLS-SB-013. Verify transcript hash.

MLS-SB-014. Verify sender authentication.

MLS-SB-015. Verify PublicMessage parse errors.

MLS-SB-016. Verify PrivateMessage parse errors.

MLS-SB-017. Verify stale KeyPackage rejection.

MLS-SB-018. Verify reused KeyPackage rejection.

MLS-SB-019. Verify KeyPackage expiry.

MLS-SB-020. Verify device identity binding.

MLS-SB-021. Verify credential binding.

MLS-SB-022. Verify Welcome encryption.

MLS-SB-023. Verify Commit persistence.

MLS-SB-024. Verify application message epoch.

MLS-SB-025. Verify replay protection.

MLS-SB-026. Verify attachment key generation.

MLS-SB-027. Verify attachment key wrapping.

MLS-SB-028. Verify server stores ciphertext only.

MLS-SB-029. Verify server lacks group secrets.

MLS-SB-030. Verify delivery service metadata validation.

MLS-SB-031. Verify delivery receipt plaintext absence.

MLS-SB-032. Verify key package publish audit.

MLS-SB-033. Verify key package consume audit.

MLS-SB-034. Verify suspicious replay audit.

MLS-SB-035. Verify MLS failure metrics.

MLS-SB-036. Verify device removal.

MLS-SB-037. Verify lost-device recovery.

MLS-SB-038. Verify escrow posture.

MLS-SB-039. Verify federation disabled posture.

MLS-SB-040. Verify crypto test vectors.

MLS-SB-041. Check `messenger` personal DM.

MLS-SB-042. Check `messenger` personal group chat.

MLS-SB-043. Check `messenger` personal device group.

MLS-SB-044. Check `messenger` attachment path.

MLS-SB-045. Check `messenger` push notification path.

MLS-SB-046. Check `messenger` search index path.

MLS-SB-047. Check `meet` MLS signaling path.

MLS-SB-048. Check KeyPackage storage.

MLS-SB-049. Check Welcome handling.

MLS-SB-050. Check Commit handling.

MLS-SB-051. Reject server plaintext search.

MLS-SB-052. Reject plaintext push body.

MLS-SB-053. Reject KeyPackage reuse.

MLS-SB-054. Reject stale epoch message.

MLS-SB-055. Reject unsupported cipher suite.

MLS-SB-056. Reject professional eDiscovery on personal MLS.

MLS-SB-057. Reject personal ciphertext in professional retention.

MLS-SB-058. Reject federation without ADR.

MLS-SB-059. Reject PQ draft as default.

MLS-SB-060. Reject server group-secret access.

MLS-SB-061. Emit MLS group count.

MLS-SB-062. Emit KeyPackage count.

MLS-SB-063. Emit consumed KeyPackage count.

MLS-SB-064. Emit epoch advance count.

MLS-SB-065. Emit commit conflict count.

MLS-SB-066. Emit replay rejection count.

MLS-SB-067. Emit malformed frame count.

MLS-SB-068. Emit device removal count.

MLS-SB-069. Emit ciphertext storage assertion.

MLS-SB-070. Emit vector test count.

MLS-SB-071. Preserve forward secrecy.

MLS-SB-072. Preserve post-compromise security.

MLS-SB-073. Preserve device-scoped identity.

MLS-SB-074. Preserve personal-professional isolation.

MLS-SB-075. Preserve ciphertext-only server role.

MLS-SB-076. Preserve attachment key separation.

MLS-SB-077. Preserve safety-number binding.

MLS-SB-078. Preserve backup privacy posture.

MLS-SB-079. Preserve RFC 9420 test-vector discipline.

MLS-SB-080. Preserve security review on library upgrade.

## Extended Worked Example: Messenger Group Creation Transcript

The transcript below is a documentation fixture. It is not a real secret or
test vector; it shows which artifacts a conformant implementation MUST record
without exposing plaintext message content to the server.

```yaml
transcript_id: messenger-mls-group-create-v1
owning_microservice: messenger
related_adrs:
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
rfc: RFC 9420
group:
  group_id: mls-group-01HZZZZZZZZZZZZZZZZZZZZZZZ
  protocol_version: mls10
  cipher_suite: MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519
  extension_set:
    - application_id
    - ratchet_tree
    - required_capabilities
members:
  - client_id: alice.device.1
    credential_type: basic
    signature_scheme: ed25519
  - client_id: bob.device.1
    credential_type: basic
    signature_scheme: ed25519
server_visible:
  - group_id
  - epoch
  - sender_client_id
  - ciphertext_size
  - delivery_timestamp
server_forbidden:
  - application_plaintext
  - exporter_secret
  - path_secret
  - joiner_secret
audit_events:
  - EVT-MLS-GROUP-CREATED
  - EVT-MLS-COMMIT-ACCEPTED
  - EVT-MLS-WELCOME-DELIVERED
verification:
  - cargo test -p messenger-mls-rfc9420 -- test_group_create_transcript
  - cargo run -p check-mls-rfc9420-conformance --quiet
```

## Extended MLS Conformance Matrix

| ID | RFC 9420 concern | Oyatie requirement | Test fixture | Checker |
|---|---|---|---|---|
| MLS-MAT-001 | Protocol version | MLS 1.0 only until upgrade ADR | `mls/version` | `check-mls-version` |
| MLS-MAT-002 | Cipher suite | Approved suite registry | `mls/cipher-suite` | `check-mls-cipher-suite` |
| MLS-MAT-003 | Credential | Credential type declared | `mls/credential` | `check-mls-credential` |
| MLS-MAT-004 | Key package | One-time use enforced | `mls/key-package` | `check-mls-key-package` |
| MLS-MAT-005 | Welcome | Welcome encrypted to joiner | `mls/welcome` | `check-mls-welcome` |
| MLS-MAT-006 | Commit | Commit validates tree hash | `mls/commit` | `check-mls-commit` |
| MLS-MAT-007 | Proposal | Proposal type allowlisted | `mls/proposal` | `check-mls-proposal` |
| MLS-MAT-008 | Ratchet tree | Tree hash stored as opaque | `mls/ratchet-tree` | `check-mls-ratchet-tree` |
| MLS-MAT-009 | Epoch | Epoch monotonic per group | `mls/epoch` | `check-mls-epoch` |
| MLS-MAT-010 | Exporter | Exporter secret never server-visible | `mls/exporter` | `check-mls-secret-boundary` |
| MLS-MAT-011 | Resumption | PSK use declared | `mls/psk` | `check-mls-psk` |
| MLS-MAT-012 | Application message | Server stores ciphertext only | `mls/application` | `check-mls-ciphertext-only` |
| MLS-MAT-013 | Framing | Sender data protected | `mls/framing` | `check-mls-framing` |
| MLS-MAT-014 | Authentication | Signature verified | `mls/signature` | `check-mls-signature` |
| MLS-MAT-015 | Membership | Remove commits revoke sender | `mls/remove` | `check-mls-removal` |
| MLS-MAT-016 | External sender | External sender policy explicit | `mls/external-sender` | `check-mls-external-sender` |
| MLS-MAT-017 | Extensions | Required capabilities enforced | `mls/extensions` | `check-mls-extensions` |
| MLS-MAT-018 | Delivery | Duplicate commit rejected | `mls/delivery` | `check-mls-replay` |
| MLS-MAT-019 | Backup | Backup payload encrypted client-side | `mls/backup` | `check-mls-backup` |
| MLS-MAT-020 | Upgrade | Library upgrade requires security review | `mls/upgrade` | `check-mls-upgrade` |

## Extended Security Review Questions

MLS-REV-001. Does the server ever see plaintext?

MLS-REV-002. Does the server ever see exporter secrets?

MLS-REV-003. Does the server enforce key-package one-time use?

MLS-REV-004. Does every commit verify membership state?

MLS-REV-005. Does every remove commit revoke future send ability?

MLS-REV-006. Does every welcome encrypt only to intended joiners?

MLS-REV-007. Does every epoch transition have replay protection?

MLS-REV-008. Does every attachment use a separate content-encryption key?

MLS-REV-009. Does every backup remain client-encrypted?

MLS-REV-010. Does every library upgrade cite RFC 9420 compatibility notes?

MLS-REV-011. Does every test fixture avoid real user metadata?

MLS-REV-012. Does every audit event avoid plaintext content?

MLS-REV-013. Does every abuse-defense scan operate on metadata only?

MLS-REV-014. Does every legal hold path preserve ciphertext-only posture?

MLS-REV-015. Does every cross-device recovery path require user-held key material?

MLS-REV-016. Does every group export path preserve member consent rules?

MLS-REV-017. Does every safety-number change notify clients?

MLS-REV-018. Does every failed commit produce a typed error?

MLS-REV-019. Does every unsupported extension fail closed?

MLS-REV-020. Does promote evidence cite `check-mls-rfc9420-conformance`?

## Extended MLS Evidence Ledger

MLS-EVID-001. Record MLS library name and version.

MLS-EVID-002. Record RFC 9420 compatibility claim.

MLS-EVID-003. Record protocol version.

MLS-EVID-004. Record cipher-suite registry row.

MLS-EVID-005. Record credential type.

MLS-EVID-006. Record signature scheme.

MLS-EVID-007. Record key-package fixture path.

MLS-EVID-008. Record welcome fixture path.

MLS-EVID-009. Record commit fixture path.

MLS-EVID-010. Record proposal fixture path.

MLS-EVID-011. Record ratchet-tree fixture path.

MLS-EVID-012. Record replay-protection test id.

MLS-EVID-013. Record duplicate-commit test id.

MLS-EVID-014. Record remove-member test id.

MLS-EVID-015. Record add-member test id.

MLS-EVID-016. Record external-sender policy id.

MLS-EVID-017. Record required-capabilities extension set.

MLS-EVID-018. Record ciphertext-only storage test id.

MLS-EVID-019. Record attachment-key separation test id.

MLS-EVID-020. Record backup encryption test id.

MLS-EVID-021. Record safety-number change test id.

MLS-EVID-022. Record unsupported-extension failure test id.

MLS-EVID-023. Record audit-event redaction test id.

MLS-EVID-024. Record abuse-defense metadata-only test id.

MLS-EVID-025. Record legal-hold ciphertext-only review.

MLS-EVID-026. Record library upgrade security-review id.

MLS-EVID-027. Record checker crate version.

MLS-EVID-028. Record VCS changeset id.

MLS-EVID-029. Record promote bundle id.

MLS-EVID-030. Record residual security-review risks.

## Extended MLS Failure Modes

MLS-FAIL-001. Server observes plaintext.

MLS-FAIL-002. Server stores exporter secret.

MLS-FAIL-003. Key package is reused.

MLS-FAIL-004. Commit skips tree-hash validation.

MLS-FAIL-005. Remove commit fails to revoke sender.

MLS-FAIL-006. Welcome is encrypted to wrong client.

MLS-FAIL-007. Epoch rollback is accepted.

MLS-FAIL-008. Attachment reuses MLS application secret directly.

MLS-FAIL-009. Backup stores plaintext.

MLS-FAIL-010. Library upgrade ships without security review.

## Extended Promotion Review Checklist

MLS-PROMOTE-001. MLS library name is recorded.

MLS-PROMOTE-002. MLS library version is recorded.

MLS-PROMOTE-003. RFC 9420 compatibility claim is recorded.

MLS-PROMOTE-004. Protocol version is pinned.

MLS-PROMOTE-005. Cipher suite is allowlisted.

MLS-PROMOTE-006. Credential type is declared.

MLS-PROMOTE-007. Signature scheme is declared.

MLS-PROMOTE-008. Key-package fixture exists.

MLS-PROMOTE-009. Welcome fixture exists.

MLS-PROMOTE-010. Commit fixture exists.

MLS-PROMOTE-011. Proposal fixture exists.

MLS-PROMOTE-012. Ratchet-tree fixture exists.

MLS-PROMOTE-013. Replay-protection test passes.

MLS-PROMOTE-014. Duplicate-commit test passes.

MLS-PROMOTE-015. Remove-member test passes.

MLS-PROMOTE-016. Add-member test passes.

MLS-PROMOTE-017. External-sender policy is explicit.

MLS-PROMOTE-018. Required-capabilities extension set is explicit.

MLS-PROMOTE-019. Ciphertext-only storage test passes.

MLS-PROMOTE-020. Attachment-key separation test passes.

MLS-PROMOTE-021. Backup encryption test passes.

MLS-PROMOTE-022. Safety-number change test passes.

MLS-PROMOTE-023. Unsupported-extension test fails closed.

MLS-PROMOTE-024. Audit-event redaction test passes.

MLS-PROMOTE-025. Abuse-defense metadata-only test passes.

MLS-PROMOTE-026. Legal-hold ciphertext-only review is complete.

MLS-PROMOTE-027. Library upgrade security review is attached.

MLS-PROMOTE-028. Server plaintext exposure count is zero.

MLS-PROMOTE-029. Exporter-secret exposure count is zero.

MLS-PROMOTE-030. Key-package reuse count is zero.

MLS-PROMOTE-031. Epoch rollback acceptance count is zero.

MLS-PROMOTE-032. Backup plaintext finding count is zero.

MLS-PROMOTE-033. Checker crate version is recorded.

MLS-PROMOTE-034. VCS changeset id is recorded.

MLS-PROMOTE-035. Promote bundle id is recorded.

MLS-PROMOTE-036. Residual security risks are recorded.

MLS-PROMOTE-037. Messenger service owner is recorded.

MLS-PROMOTE-038. E2E encryption runbook is linked.

MLS-PROMOTE-039. Security reviewer signoff is attached.

MLS-PROMOTE-040. RFC vector compatibility evidence is attached.

MLS-PROMOTE-041. Client recovery path is tested.

MLS-PROMOTE-042. Cross-device add path is tested.

MLS-PROMOTE-043. Cross-device remove path is tested.

MLS-PROMOTE-044. Group export path is tested.

MLS-PROMOTE-045. Pack overlay restrictions are tested.

MLS-PROMOTE-046. Data-class labels are safe.

MLS-PROMOTE-047. Audit events omit plaintext.

MLS-PROMOTE-048. Trace labels omit plaintext.

MLS-PROMOTE-049. Attachments use separate keys.

MLS-PROMOTE-050. Promotion evidence includes MLS checker output.
