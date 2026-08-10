---
doc_class: Runbook
title: coturn key rotation + capacity expansion
microservice: meet
severity: "Sev-3 (planned) / Sev-2 (saturation) / Sev-1 (compromise-driven)"
status: Accepted
owner_team: ops-sre-reliability + ops-security + axis-meet
date: 2026-05-17
last_drill_date: 2026-05-17
related_artifacts:
  - microservices/meet/failure-modes.md (FM-02)
  - comms/meet/dashboards/meeting-quality-mos.json
  - microservices/meet/iac/helm/meet/templates/coturn-deployment.yaml
doc_status: published
---

# Runbook: coturn key rotation + capacity expansion (meet)

## When

Three triggers:

1. **Planned rotation** — scheduled coturn shared-secret rotation per OpenBao 30d cadence (per `policy/data-residency.md`).
2. **Capacity saturation** — `coturn_traffic_bytes_total` bandwidth > 70 % provisioned; ICE relay-candidate selection > 50 % (high TURN dependence).
3. **Compromise-driven rotation** — coturn auth-secret suspected compromised; immediate rotation + audit forensics.

## Severity

- Planned: Sev-3.
- Saturation: Sev-2.
- Compromise-driven: Sev-1.

## Planned Rotation Procedure (Sev-3)

| Step | Action | Owner | Time |
|---|---|---|---|
| 1 | OpenBao schedules secret rotation (`secret/meet/<pack>/coturn/auth-secret`) every 30d | ops-security | n/a |
| 2 | New shared secret generated; old secret kept valid during a 24h grace window | ops-security | ≤ 1 min |
| 3 | Helm release picks up rotated secret via SecretReference; coturn pods reload | axis-meet | ≤ 5 min |
| 4 | Verify active TURN allocations continue working under both secrets during grace | ops-sre-reliability | ≤ 30 min observation |
| 5 | After grace window: old secret revoked; metrics confirm no failed TURN allocations | ops-security | ≤ 24h |

## Saturation Mitigation Procedure (Sev-2)

| Step | Action | Owner | Time |
|---|---|---|---|
| 1 | Confirm trigger via `dashboards/meeting-quality-mos.json` panel "ICE candidate pair selection" + "TURN bytes served/sec" | ops-sre-reliability | ≤ 2 min |
| 2 | Identify source pack + tenants contributing most TURN traffic | ops-sre-reliability | ≤ 5 min |
| 3 | HPA scale-up coturn pods; bandwidth doubles within 5 min | ops-sre-reliability | ≤ 5 min |
| 4 | Verify external_ip reachable + TURN allocations succeed | ops-sre-reliability | ≤ 5 min |
| 5 | If saturation continues: investigate per-tenant abuse (excessive TURN dependence may indicate misconfigured client or bad-actor); engage tenant + ops-security | ops-sre-reliability + ops-security | ≤ 30 min |

## Compromise-Driven Rotation Procedure (Sev-1)

| Step | Action | Owner | Time |
|---|---|---|---|
| 1 | ops-security confirms compromise; identifies pack + scope | ops-security | ≤ 30 min |
| 2 | Force-rotate coturn shared-secret (no 24h grace; immediate revoke of old) | ops-security | ≤ 5 min |
| 3 | All in-flight TURN allocations using old secret will fail; clients re-establish via new secret on reconnect | n/a | ≤ 5 min |
| 4 | Surface in-meeting host banner: "Service reset; please rejoin if media drops" | active-meeting comms lead | ≤ 5 min |
| 5 | Audit-chain seal of rotation event with reason code | ops-security | ≤ 5 min |
| 6 | Forensics: examine coturn access log for allocation attempts using old secret post-rotation (indicator of attacker-still-active) | ops-security | ≤ 24h |
| 7 | Post-mortem within 5 business days | council-privacy + ops-security | |

## Capacity Expansion Procedure (Sev-2)

For sustained saturation (not key-related):

1. Add coturn pods (HPA already auto-scales; manual override for emergency capacity).
2. Verify per-pack regional egress bandwidth quota with cloud-iac team.
3. If multi-region cascade: engage cloud-iac for additional regional coturn cluster activation.
4. Update `capacity-model.md` if peak exceeds forecast envelope.

## Verification

- `coturn_traffic_bytes_total` rate stays below 70 % of cluster bandwidth cap.
- TURN allocation success rate ≥ 99.9 %.
- ICE relay-candidate selection percentage stable (a sustained > 50 % indicates underlying NAT issue, not coturn issue).
- No `coturn_auth_failure_total` spikes after rotation grace window.

## Pack Overlays

| Pack | Variation |
|---|---|
| pack-eu | per GDPR Art. 32 — rotation evidence retained per audit-chain |
| pack-us-healthcare | HIPAA 45 CFR §164.312(a)(2)(iv) — key management documented |
| pack-kr | KR PIPA Art. 29-2 + KR-ISMS-P §2.7 — rotation cadence in DPIA |
| pack-us-financial | SEC 17a-4(f) — TURN access logs retained alongside meeting archive |

## References

- RFC 5766 (TURN); RFC 5389 (STUN); RFC 8489 (STUN updated).
- coturn `github.com/coturn/coturn/wiki`.
- OpenBao secret-rotation docs.
- `microservices/meet/threat-model.md` T-S-03.
- `comms/messenger/runbooks/e2e-encryption-key-rotation.md` (related key-rotation pattern).
