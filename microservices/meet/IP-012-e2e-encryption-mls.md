---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-meet-foundation
impl_plan_id: IP-012-e2e-encryption-mls
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-meet + council-privacy
acceptance_lanes: [cargo-nextest, e2e-handshake-smoke, oya-governance-cedar-coverage]
---

# IP-012: opt-in E2E encryption (MLS RFC 9420 + W3C Insertable Streams)

## Intent

Author the e2e-encryption BC: per-tenant opt-in MLS RFC 9420 group key agreement; per-frame encryption via W3C Insertable Streams (SFrame). Server sees Insertable-Streams ciphertext only; oyatie servers + LiveKit SFU + Whisper transcription + recording pipeline all see ciphertext only. Recording + transcription + AI summary refused by Cedar deny when E2E mode active.

MLS handshake p95 ≤ 1.0s for up to 12-participant group. Epoch rotation client-driven (monthly recommended per RFC 9420 §11.6).

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-meet-e2e-encryption-{kernel,domain,usecase}/src/...` | create |
| `src/crates/oya-meet-e2e-encryption-adapter-mls/src/group.rs` | create — mls-rs binding |
| `src/crates/oya-meet-e2e-encryption-sdk/src/insertable_streams.rs` | create — client-side SFrame helper |
| `policy/meeting-scope.cedar` | edit — add forbids on recording/transcription when `e2e=true` |
| `tests/e2e_handshake_e2e.rs` | create |

## Code Shape

```rust
// adapter-mls/src/group.rs
pub struct MlsGroup { /* mls-rs handle */ }

impl MlsGroup {
    pub fn create(creator_key_package: &KeyPackage, member_key_packages: &[KeyPackage]) -> Result<Self> {
        // Bootstrap MLS group with all members
        // ...
    }
    pub fn advance_epoch(&mut self, removed: &[Member], added: &[Member]) -> Result<Commit> {
        // Per RFC 9420 §12
        // ...
    }
}
```

```cedar
// meeting-scope.cedar additions:
forbid (
  principal,
  action in [
    Action::"start_recording",
    Action::"start_transcription",
    Action::"start_ai_summary"
  ],
  resource in MeetingInstance::?i
)
when {
  resource has e2e_mode &&
  resource.e2e_mode == true
};
```

## Acceptance Gates

```bash
cargo nextest run -p oya-meet-e2e-encryption-adapter-mls
cargo nextest run --test e2e_handshake_e2e
cargo run -p oya-dev-cli -- gate validate cedar-coverage --microservice meet
```

## Test Plan

- MLS group bootstrap: 2-, 6-, 12-participant groups; handshake p95 ≤ 1.0s.
- Epoch rotation: client-driven advance; old epoch ciphertext unreadable in new epoch by removed members.
- Cedar deny: with `e2e_mode=true`, recording start returns 403 + audit-chain `oya_meet_e2e_recording_attempt_denied_total` increment.
- Server-decrypt-attempt: server-side decrypt attempt returns `oya_meet_e2e_admin_decrypt_attempt_total++` (target = 0 in production).
- Insertable Streams: per-frame encryption verified in WebRTC peer-conn integration test (Chrome + Firefox).

## Halt Conditions

- E2E mode allows recording — refuse.
- MLS implementation diverges from RFC 9420 — refuse.

## Next IP

[`IP-013-contracts-openapi-asyncapi-proto.md`](IP-013-contracts-openapi-asyncapi-proto.md)

## References

- ADR-MEET-0003 (E2E for meetings).
- ADR-MSGR-0002 (messenger E2E tier-split; aligned posture).
- RFC 9420 (MLS).
- W3C Insertable Streams `w3c.github.io/webrtc-encoded-transform/`.
- IETF MLS WG `datatracker.ietf.org/wg/mls/`.
- mls-rs `github.com/awslabs/mls-rs`.
