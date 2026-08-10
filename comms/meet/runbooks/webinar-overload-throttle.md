---
doc_class: Runbook
title: Webinar overload throttle (>10k attendee fan-out + egress allow-list violation)
microservice: meet
severity: "Sev-2 (overload) / Sev-1 (egress allow-list violation)"
status: Accepted
owner_team: ops-sre-reliability + axis-meet + ops-security
date: 2026-05-17
last_drill_date: 2026-05-17
related_artifacts:
  - microservices/meet/failure-modes.md (FM-07, FM-13)
  - comms/meet/dashboards/recording-pipeline.json
  - microservices/meet/slos/webinar-fanout-latency.openslo.yaml
  - comms/meet/policy/meeting-scope.cedar
doc_status: published
---

# Runbook: Webinar overload throttle + egress allow-list violation (meet)

## Trigger

### Overload (Sev-2)
- `oya_meet_webinar_attendees_active` > 10k interactive cap on a single event.
- HLS edge cache miss rate > 50 % sustained.
- WHIP/HLS mesh saturated.
- SRS RTMP egress queue depth > 5 sustained.

### Egress allow-list violation (Sev-1)
- `oya_meet_rtmp_egress_destination` outbound to non-allow-listed host.
- NetworkPolicy denies outbound RTMP/HTTPS to suspicious endpoint.
- DNS allow-list violation event.

## Severity

- Overload: Sev-2.
- Allow-list violation: Sev-1 (potential recording-leak to unauthorized destination).

## Overload Mitigation Procedure (Sev-2)

| Step | Action | Time |
|---|---|---|
| 1 | Inspect `dashboards/recording-pipeline.json` panel "Webinar attendee count by tier" | ≤ 2 min |
| 2 | Force broadcast-mode (transition from interactive tier to broadcast tier per ADR-MEET-0005); new attendees join HLS broadcast tier | ≤ 5 min |
| 3 | Scale SRS RTMP egress + HLS edge cache pods (HPA) | ≤ 5 min |
| 4 | Throttle new interactive-tier joins; existing interactive participants unaffected | ≤ 5 min |
| 5 | Raise per-tenant attendee cap (with FinOps approval) if legitimate growth | ≤ 30 min |
| 6 | Notify host of attendee-cap status | ≤ 5 min |

## Allow-List Violation Procedure (Sev-1)

| Step | Action | Time |
|---|---|---|
| 1 | NetworkPolicy + DNS allow-list refuses outbound RTMP/HTTPS to non-listed destination | ≤ immediate (preventative) |
| 2 | SRS stream terminated; egress worker logs deny event | ≤ 5 min |
| 3 | Audit-chain seal of attempt + destination + tenant + timestamp | ≤ 5 min |
| 4 | Engage ops-security; investigate: misconfiguration vs compromise vs malicious-host swap | ≤ 30 min |
| 5 | If compromise suspected: emergency-rotate per-tenant egress stream key in OpenBao | ≤ 30 min |
| 6 | If misconfiguration: surface to tenant-admin for allow-list correction | ≤ 1 hour |

## Diagnosis (Overload)

| Hypothesis | Signal | Investigation |
|---|---|---|
| Legitimate growth past cap | event registration / promotion drove > 10k | raise cap; FinOps; surface to gtm-customer-success |
| Per-tenant abuse / bot-attendee flood | unusual join rate from sparse IPs | tenant security-admin engaged |
| HLS edge cache misconfiguration | unusual miss rate | review cache-shaping; possibly add CDN warming |
| WHIP fallback unhealthy | WHIP handshake failures | switch broadcast attendees to standard HLS |

## Diagnosis (Allow-list violation)

| Hypothesis | Signal | Investigation |
|---|---|---|
| Tenant added new destination via UI not yet propagated | allow-list cache stale | force cache-refresh; verify Cedar policy current |
| Attacker swapped destination via compromised OpenBao secret | OpenBao audit anomaly | rotate secret; engage ops-security |
| Misconfigured DNS resolves allow-listed host to attacker IP | DNS-tampering pattern | engage cloud-iac for DNS audit |
| Tenant's broadcasting stream is actually being redirected by attacker | RTMP handshake from unexpected source | engage ops-security |

## Recovery Verification

- Webinar attendee count back within cap or broadcast-tier successfully serving overflow.
- HLS edge cache hit rate ≥ 90 %.
- No egress allow-list deny events for 24h.
- All RTMP outbound streams confirmed destination-legitimate.

## Postmortem

- Overload Sev-2: within 5 business days.
- Allow-list violation Sev-1: within 2 business days; council-privacy + ops-security sign-off.

## References

- ADR-MEET-0004 (egress policy).
- ADR-MEET-0005 (large-audience + webinar architecture).
- HLS RFC 8216.
- WHIP IETF draft.
- SRS docs.
- `microservices/meet/threat-model.md` T-I-08, T-E-04.
- `comms/meet/policy/meeting-scope.cedar`.
