---
doc_class: Runbook
runbook_id: identity-idp-failover-drill
microservice: identity
sev: planned-quarterly
owner_team: ops-sre-reliability + axis-identity
date: 2026-05-18
---

# Runbook: IdP failover drill (quarterly DR exercise)

## Purpose

Exercise the per-pack failover path per ADR-0179 + multi-region.md §"Scenario 2 — Primary region partial outage". Validates RTO ≤5min target, RPO ≤30s target, and post-failover JWKS continuity.

## Pre-flight

- Schedule with stakeholders 7 days in advance.
- Pick the pack (rotate quarterly: pack-eu Q1, pack-us Q2, pack-kr Q3, pack-us-healthcare Q4).
- Notify in change-mgmt channel.
- Maintenance window: 2h tolerance.

## Procedure

### Pre-drill snapshot

1. Capture current state:
   - `oya identity health --pack <pack>` — all green expected.
   - `oya identity slo snapshot --pack <pack>` — record budget burn.
   - Postgres LSN snapshot for replication lag baseline.
   - JWKS endpoint response snapshot.

### Step 1 — Promote warm-standby to read-write

```
oya identity dr promote-standby --pack <pack> --to <failover-region> --dry-run
oya identity dr promote-standby --pack <pack> --to <failover-region>
```

- Patroni promotes the warm-standby Postgres to leader.
- Zitadel pods in failover region begin processing reads + writes.
- Measure: time from promote command to first successful write = T1.

### Step 2 — DNS update

```
oya dns update --record identity-<pack>.oyatie.dev --weight failover:100 --primary:0
```

- Per-pack DNS weighted to failover region.
- Propagation: ≤2min (TTL=60).
- Measure: time from DNS change to first successful sign-in from failover region = T2.

### Step 3 — Verify

- `curl https://identity-<pack>.oyatie.dev/oauth/v2/discovery` — issuer + endpoints match.
- `curl https://identity-<pack>.oyatie.dev/oauth/v2/keys` — JWKS contains all expected kids.
- Initiate test sign-in; verify Passkey ceremony completes.
- Initiate test SCIM POST; verify creation succeeds.
- Initiate step-up grant; verify Cedar policy enforces.
- `oya identity audit replay --pack <pack> --since 5m` — events flowing.

### Step 4 — Measure RTO + RPO

- **RTO**: T2 - drill-start = total time-to-recovery.
- **RPO**: max Postgres LSN delta between primary and replica at promote-time.
- **Acceptable**: RTO ≤ 5min, RPO ≤ 30s.

### Step 5 — Restore

```
oya identity dr restore-primary --pack <pack>
```

- Old primary returns to service as the leader.
- DNS update reverses (failover region back to standby).
- Postgres re-syncs.

### Post-drill

- Snapshot post-state; compare to pre-drill.
- Validate audit-chain continuity (no events lost).
- Validate user sessions: any signed-in user during the drill should have stayed signed-in (JWT survives DNS change).
- Validate SLO budget burn: did the drill exceed the budget? If yes, document.

## Drill report

Emit `evidence/identity/dr-drill-<pack>-<date>.json`:
- RTO measured
- RPO measured
- Errors observed
- Session impact (count of forced re-sign-ins)
- SLO budget impact
- Action items

## Drill failure modes (and remediation)

| Failure | Cause | Mitigation |
|---|---|---|
| Patroni promote fails | Quorum loss | escalate to dba-on-call; manual leader pin |
| Zitadel pods fail to start in failover region | Image pull fail | pre-pull images on failover region nodes |
| JWKS not synced | Postgres replication lag exceeded | extend wait time; OR force replication catch-up |
| DNS does not propagate | TTL too high | lower TTL pre-drill |
| Audit-chain continuity broken | Network split during drill | replay DLQ |

## Quarterly review

After 4 drills, review trend: RTO improving? RPO stable? If RTO degrading, file IP for the architectural fix.

## Cross-references

- multi-region.md §"Scenario 2"
- ADR-0152 RPO/RTO canonical
- ADR-0179 sovereign-cloud-per-regional-pack
