---
doc_class: Runbook
title: Envoy SNI / TLS debug
microservice: cloud-k8s
severity: "Sev-2 (tenant TLS handshakes fail)"
status: Accepted
owner_team: ops-sre-reliability + axis-cloud
date: 2026-05-17
related_artifacts:
  - k8s/failure-modes.md (FM-09)
  - k8s/threat-model.md (T-S-03, T-I-03)
doc_status: published
---

# Runbook: Envoy SNI / TLS debug

## Trigger

- `envoy_listener_https_downstream_cx_ssl_handshake_errors_total > 0` for ≥ 5 min
- Tenant-reported TLS handshake failure on `https://<tenant-host>`
- cert-manager renewal failure
- Suspected SNI spoofing attempt (threat T-S-03)

## Severity

Sev-2 (tenant external TLS path degraded). Sev-1 if SNI spoofing suspected.

## Procedure

| Step | Action | Time |
|---|---|---|
| 1 | Identify symptom: which tenant host? `kubectl -n istio-system logs <ingress-gateway-pod> | grep -i 'ssl_error\|sni\|handshake'` | ≤ 5 min |
| 2 | Check cert-manager status: `kubectl get certificate -A | grep -i 'false\|expired'` | ≤ 5 min |
| 3 | Inspect cert detail: `kubectl -n <ns> describe certificate <tenant-cert>` — expiry, renewal, last error | ≤ 5 min |
| 4 | If renewal failed: verify OpenBao reachability + cert-manager ServiceAccount perms | ≤ 5 min |
| 5 | Force cert-manager reconcile: `kubectl -n <ns> annotate certificate <name> cert-manager.io/issue-temporary-certificate=true --overwrite` | ≤ 2 min |
| 6 | Wait for new cert: `kubectl -n <ns> get certificate <name> -w` until READY=True | ≤ 5 min |
| 7 | Restart ingress gateway to pick up new cert: `kubectl -n istio-system rollout restart deployment istio-ingressgateway` | ≤ 5 min |
| 8 | Verify TLS handshake from external probe: `curl -vI https://<tenant-host>` returns cert with correct SAN + expiry | ≤ 5 min |

## SNI spoofing investigation (T-S-03)

| Step | Action | Time |
|---|---|---|
| 1 | Engage ops-security; declare Sev-1 + `#inc-sec-<id>` | immediate |
| 2 | Identify the suspected spoofing flows: `kubectl -n istio-system logs <ingress-pod> | grep 'sni=<unexpected-host>'` | ≤ 10 min |
| 3 | Check VirtualService routing rules: `kubectl get virtualservice -A -o yaml | grep -A5 host:` | ≤ 5 min |
| 4 | Verify SNI value validated against cert SAN list (defense): `kubectl -n istio-system get gateway -o yaml | grep -A5 hosts:` | ≤ 5 min |
| 5 | If a spoofed SNI was routed: forensic trace via audit-chain `IstioPolicyChanged` events | ≤ 30 min |
| 6 | Mitigation: tighten Gateway `hosts:` allowlist; refuse wildcard hosts unless explicitly authorised | ≤ 30 min |
| 7 | Verify: re-test the spoofed SNI value returns 421 (Misdirected Request) | ≤ 5 min |

## cert-manager renewal failure

| Issue | Diagnosis | Fix |
|---|---|---|
| ACME challenge failed | DNS-01: TXT record propagation slow | wait + retry; verify `_acme-challenge.<host>` TXT record visible via `dig` |
| ACME challenge failed | HTTP-01: ingress unreachable | verify Envoy ingress listening on :80 for /.well-known/acme-challenge |
| OpenBao issuer failed | OpenBao token expired | rotate token in cert-manager ClusterIssuer |
| Rate-limited by Let's Encrypt | Too many renewals | use staging issuer for testing; verify production rate-limit window |
| KMS-backed key gen failed | KMS API unreachable | engage cloud-iac; verify KMS endpoint |

## ECH (Encrypted Client Hello) status

ECH is rolling out across the ecosystem; oyatie tracks Envoy + Cloudflare ECH support. Once enabled (subsequent-to-M03-completion), SNI sniffing residual (T-I-03) drops to L.

## Verification

- `curl -vI https://<tenant-host>` returns cert with correct SAN, expiry > 30d
- `envoy_listener_https_downstream_cx_ssl_handshake_errors_total` rate returns to baseline (= 0)
- cert-manager `kubectl get certificate -A` all READY=True
- audit-chain has no recent anomalous Gateway / VirtualService mutations

## References

- `k8s/failure-modes.md` FM-09.
- `k8s/threat-model.md` T-S-03, T-I-03.
- Envoy TLS — `envoyproxy.io/docs/envoy/latest/intro/arch_overview/security/tls`.
- Istio Gateway / VirtualService — `istio.io/latest/docs/concepts/traffic-management/`.
- cert-manager — `cert-manager.io/docs/troubleshooting/`.
- ECH — `tools.ietf.org/html/draft-ietf-tls-esni`.
