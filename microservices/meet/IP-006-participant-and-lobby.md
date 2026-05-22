---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-meet-foundation
impl_plan_id: IP-006-participant-and-lobby
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-meet + ops-security
acceptance_lanes: [cargo-nextest, lobby-gate-test, oya-governance-cedar-coverage]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: participant BC + lobby/waiting-room with Cedar gate

## Intent

Participant BC tracks per-participant state with role discrimination (`Host`, `CoHost`, `Presenter`, `Attendee`, `Guest`, `Interpreter`). Lobby + waiting-room are server-side Cedar-evaluated gates: non-members enter lobby pre-meeting; host approves to graduate to meeting; LiveKit refuses publish/subscribe without `lobby_approved` bit in access token.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-meet-participant-{kernel,domain,usecase}/src/...` | create |
| `src/crates/oya-meet-participant-adapter-valkey/src/lobby_queue.rs` | create — Valkey-backed lobby queue per instance |
| `src/crates/oya-meet-participant-adapter-postgres/src/log.rs` | create — append-only participant log |
| `src/crates/oya-meet-participant-rest/src/handlers.rs` | create — REST handlers (lobby approve/deny, role-change, list) |
| `src/crates/oya-meet-participant-worker/src/lobby_evictor.rs` | create — TTL-evict lobby members after host inactivity |
| `policy/meeting-scope.cedar` | edit — add Action::"approve_lobby", "join_meeting", "change_role" |
| `tests/lobby_gate_e2e.rs` | create |

## Code Shape

```rust
// usecase: lobby approval
pub struct ApproveLobbyMember;

impl ApproveLobbyMember {
    pub async fn execute(
        &self,
        ctx: &Ctx,
        principal: &Principal,
        instance_id: &InstanceId,
        user_ref: &UserRef,
    ) -> Result<()> {
        // Cedar gate
        let decision = ctx.cedar.evaluate(
            Action::ApproveLobby,
            principal,
            Resource::MeetingInstance(instance_id),
        ).await?;
        if !decision.is_allow() { return Err(Error::Forbidden); }
        // Dequeue from lobby queue
        ctx.lobby.remove(instance_id, user_ref).await?;
        // Issue LiveKit token with lobby_approved=true
        let token = ctx.sfu.issue_participant_token(instance_id, user_ref, ParticipantRole::Attendee).await?;
        // Push token to user via WebSocket signaling
        ctx.signaling.notify_approved(user_ref, token).await?;
        // Audit
        ctx.audit.seal(ParticipantApproved { instance_id, user_ref, approver: principal.id() }).await?;
        Ok(())
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-meet-participant-adapter-valkey
cargo nextest run -p oya-meet-participant-rest
cargo nextest run --test lobby_gate_e2e
cargo run -p oya-dev-cli -- gate validate cedar-coverage --microservice meet
```

## Test Plan

- Lobby gate: guest token attempts to publish without approval → LiveKit refuses; meet-rest emits `oya_meet_lobby_bypass_attempt_total`.
- Host approval: dequeue + token issue + signal arrival in < 500ms.
- TTL evict: idle lobby member after 10min (configurable) is auto-evicted.
- Cedar deny on cross-tenant approve: refused at policy layer.

## Halt Conditions

- Any lobby code path that bypasses Cedar — refuse.
- Token-issued-before-approval pattern — refuse.

## Next IP

[`IP-007-screen-share-and-tracks.md`](IP-007-screen-share-and-tracks.md)

## References

- ADR-0008; ADR-MEET-0001; ADR-MEET-0003.
- `microservices/meet/policy/meeting-scope.cedar`.
- `microservices/meet/threat-model.md` T-I-04 (lobby bypass).
