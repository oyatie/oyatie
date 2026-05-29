---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: slides
runbook_id: broadcast-mode-degraded
status: Accepted
severity: Sev-2
date: 2026-05-17
owner_team: axis-workspace + axis-realtime + ops-sre-reliability
related_artifacts:
  - microservices/slides/decisions/ADR-SLIDES-0005-broadcast-mode-and-livekit-reuse.md
  - microservices/slides/slos/broadcast-mode-availability.openslo.yaml
  - microservices/slides/failure-modes.md FM-06 / FM-07
doc_status: published
---

# Runbook — Broadcast-mode degraded (LiveKit signaling drop)

## When to use

- `oya_slides_broadcast_signal_health < 0.95` over 5m.
- Tenant reports broadcast viewers disconnecting mid-present.
- Messenger LiveKit cluster health alarm propagated.

## Symptom triage

| Symptom | Likely cause | Run |
|---|---|---|
| Broadcast signal health drops cluster-wide | Messenger LiveKit cluster issue | step 1 |
| Single broadcast session degraded | Per-deck SFU saturation OR presenter token issue | step 2 |
| Per-pack broadcast unavailable | Pack-specific LiveKit node failure | step 3 |
| Audience reactions/polls not flowing | embed-bridge to forms degraded | step 4 |

## Step 1 — Cluster-wide LiveKit health

```bash
# Check messenger LiveKit cluster health
oya vcs --service messenger --action livekit-cluster-status

# If LiveKit cluster degraded, propagate degradation banner
oya vcs --service slides --action announce-broadcast-degraded --reason "messenger-livekit-degraded"
```

Graceful degradation:
- Active broadcast sessions: notify presenter via banner; viewers see "broadcast paused" overlay.
- New broadcast attempts: refuse with retry-after; tenant-facing error.
- Present-mode (no broadcast) unaffected; presenters can continue presenting locally with audience disconnected from broadcast layer.

Wait for messenger LiveKit recovery; reconnect viewers automatically.

## Step 2 — Single-session degradation

```bash
SESSION_ID=<broadcast_session_id>

# Inspect SFU saturation
oya vcs --service messenger --action livekit-session-health --session-id $SESSION_ID

# If saturation > 80%, trigger SFU cascade
oya vcs --service messenger --action livekit-sfu-cascade --session-id $SESSION_ID

# If presenter token issue (token expired / revoked)
oya vcs --service slides --action issue-presenter-token --session-id $SESSION_ID --presenter <oidc-sub>
```

## Step 3 — Per-pack LiveKit failure

Per `multi-region.md`, LiveKit is pack-pinned via messenger.

```bash
PACK=<pack>

# Confirm pack LiveKit cluster down
oya vcs --service messenger --action livekit-cluster-status --pack $PACK

# Failover to pack secondary region
oya vcs --service messenger --action livekit-region-failover --pack $PACK --target secondary

# Slides broadcast sessions in pack will reconnect post-failover
oya vcs --service slides --action broadcast-region-failover --pack $PACK
```

## Step 4 — Reactions/polls/Q&A bridge degraded

```bash
# Confirm forms µservice up
oya vcs --service forms --action health

# If forms degraded, embed-bridge hides poll widgets; reactions continue
oya vcs --service slides --action announce-poll-bridge-degraded
```

## Re-enable

After fix:

```bash
# Health verify
oya vcs --service slides --action broadcast-health
oya vcs --service messenger --action livekit-cluster-status

# Lift banner
oya vcs --service slides --action announce-broadcast-restored
```

## Verification

- `oya_slides_broadcast_signal_health > 0.99` over 5m.
- `oya_slides_broadcast_session_active_count` matches expected.
- Audit-chain seal of incident emitted.

## Escalation

- Sev-2: on-call.
- Sev-1 (extended pack outage or cross-pack failure): leadership + tenant comms.

## References

- ADR-SLIDES-0005 (LiveKit reuse).
- messenger LiveKit-degraded runbook.
- failure-modes.md FM-06, FM-07.
