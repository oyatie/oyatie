---
doc_class: Runbook
title: Ingress DDoS throttle (Envoy gateway)
microservice: cloud-k8s
severity: "Sev-1 (external availability)"
status: Accepted
owner_team: ops-sre-reliability + ops-security + axis-cloud
date: 2026-05-17
related_artifacts:
  - k8s/failure-modes.md (FM-08)
  - k8s/threat-model.md (T-D-06)
doc_status: published
---

# Runbook: Ingress DDoS throttle

## Trigger

- `envoy_cluster_upstream_rq_pending_overflow_total > 0` (queue overflow)
- Ingress `5xx` rate > 1% over 1m
- WAF rate-limit-trigger spike
- Provider DDoS alert (Cloudflare / OCI shield)
- Tenant-reported external connectivity outage

## Severity

Sev-1 — external tenant availability impacted; reputational + SLA risk.

## Immediate triage (≤ 5 min)

| Step | Action | Time |
|---|---|---|
| 1 | Verify DDoS pattern (not legit traffic burst): check source-IP diversity, request shape, geographic distribution | ≤ 2 min |
| 2 | Identify target: which tenant host(s)? `kubectl -n istio-system logs <ingress-pod> | head -200 | grep -oP 'host: \K.*'` | ≤ 3 min |
| 3 | Verify Cloudflare DDoS mitigation engaged (if Cloudflare-fronted) | ≤ 2 min |
| 4 | Engage Cloudflare on-call if not auto-mitigated | ≤ 5 min |

## Containment (≤ 15 min)

| Step | Action | Time |
|---|---|---|
| 1 | Enable Envoy rate-limit filter (if not already): `kubectl -n istio-system apply -f rate-limit-emergency.yaml` (RL policy 100 req/sec/IP, 1000 req/sec/tenant) | ≤ 2 min |
| 2 | Engage OCI shield (provider-edge DDoS) | ≤ 5 min |
| 3 | Scale ingress gateway via HPA: `kubectl -n istio-system patch hpa istio-ingressgateway -p '{"spec":{"minReplicas":10,"maxReplicas":50}}'` | ≤ 2 min |
| 4 | If targeted attack on one tenant: temporarily block source IPs via WAF rule | ≤ 5 min |
| 5 | If broad attack: enable Cloudflare "Under Attack" mode for the tenant host(s) (challenge interstitial) | ≤ 2 min |

## Mitigation tactics (≤ 30 min)

| Issue | Mitigation |
|---|---|
| Volumetric L3/L4 attack | Provider-edge shield + Cloudflare BGP-level absorption; on-prem-side can do little |
| L7 HTTP flood | Envoy rate-limit filter + WAF (OWASP CRS rule 920_PROTOCOL_ATTACK) |
| Slow-loris / slow-read | Envoy `connection_idle_timeout` short (30s); `request_timeout` short (30s) |
| Application-layer (login flooding) | Per-route rate-limit + CAPTCHA challenge at application layer (workload owner's responsibility) |
| Reflection / amplification | Block known reflector source IPs at provider edge |

## Communication

- Status page: update ≤ 5 min of declaration
- Tenants affected: email per `incident-response.md` Sev-1 template
- If attack persists > 1h: leadership engagement; consider extended Cloudflare "Under Attack" mode
- Public comms: blameless transparency; do not disclose source-IP details (avoid signaling)

## Verification (after attack ends)

- `envoy_cluster_upstream_rq_pending_overflow_total` rate at baseline (= 0)
- Ingress `5xx` rate < 0.1%
- HPA-scaled replicas can step back down
- Cloudflare "Under Attack" mode can be disabled
- Tenants verified reachable

## Post-incident

- Postmortem ≤ 5 business days
- Re-evaluate rate-limit defaults (was the emergency RL too aggressive? too lax?)
- Tune WAF rules based on attack signature
- Capture attack source ASNs for ongoing block-list

## References

- `k8s/failure-modes.md` FM-08.
- `k8s/threat-model.md` T-D-06.
- Envoy rate-limit — `envoyproxy.io/docs/envoy/latest/configuration/http/http_filters/rate_limit_filter`.
- Istio ingress security — `istio.io/latest/docs/tasks/security/`.
- Cloudflare DDoS — `developers.cloudflare.com/ddos-protection/`.
- OCI shield — `oracle.com/cloud/security/cloud-services/web-application-firewall/`.
- OWASP CRS — `coreruleset.org`.
