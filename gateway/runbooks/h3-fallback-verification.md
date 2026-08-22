# Runbook — HTTP/3 fallback verification

**Authority:** ADR-0253 (HTTP/3 + QUIC default; fallback chain).
**Owner:** axis-network.
**Trigger SEV:** SEV-3.
**Last reviewed:** 2026-05-20.

## A — Trigger conditions

- SLO `h3-negotiation-rate` < 0.8 for >30min.
- Customer report: "My corp VPN blocks UDP/443; the app is slow."
- Routine quarterly verification.

## B — Pre-checks

1. Confirm h3 advertised: `curl -I https://api.oyatie.com/` → `Alt-Svc: h3=":443"; ma=86400, h3-29=":443"; ma=86400`.
2. Confirm UDP/443 reachable from common networks (synthetic probes from us-east, eu-frankfurt, ap-tokyo).
3. Verify cell QUIC listener: `kubectl logs -n api-gateway -l app=envoy --tail 100 | grep "QUIC listener"`.

## C — Procedure

1. **Synthetic probe — pure HTTP/3:**
   - `curl --http3 https://api.oyatie.com/healthz -v 2>&1 | grep "HTTP/3"`.
2. **Synthetic probe — h3 → h2 fallback** (QUIC blocked):
   - `iptables -A OUTPUT -p udp --dport 443 -j DROP` (in test env only!).
   - `curl https://api.oyatie.com/healthz -v 2>&1 | grep "HTTP/2"`.
   - `iptables -D OUTPUT -p udp --dport 443 -j DROP`.
3. **Synthetic probe — h2 → h1.1 fallback** (h2 disabled via ALPN restriction):
   - `curl --http1.1 https://api.oyatie.com/healthz -v 2>&1 | grep "HTTP/1.1"`.
4. **NEVER fall to HTTP/1.0** (forbidden by ADR-0253):
   - `curl --http1.0 https://api.oyatie.com/healthz -v 2>&1 | grep -E "^HTTP/"` → expect connection error or upgrade.
5. **Per-region h3 negotiation rate:**
   - Pull metric `api_gateway_h3_negotiation_ratio` per cell; expect ≥0.8 on non-restrictive networks, ≥0.5 globally.
6. **Per-tenant h3 negotiation rate:**
   - `api_gateway_h3_negotiation_ratio{tenant_id=<id>}`; if a tenant drops below 0.5 unexpectedly, investigate their network.

## D — Verification

- h3 succeeds where UDP/443 reachable.
- h2 fallback succeeds where UDP/443 blocked.
- h1.1 fallback succeeds where ALPN restricts.
- HTTP/1.0 refused.

## E — Rollback

N/A — verification-only runbook.

## F — Post-incident (if h3 broken)

1. Investigate Envoy QUIC listener config.
2. Investigate kernel QUIC UDP socket support (`sysctl net.core.somaxconn`, `sysctl net.ipv4.udp_mem`).
3. Investigate Anycast/BGP path MTU < 1280 (QUIC min).
4. Investigate firewall / DDoS provider blocking UDP/443.

## G — References

- ADR-0253
- IETF RFC 9000 (QUIC), RFC 9114 (HTTP/3)
- `iac/envoy-config.yaml` (h3 listener block)
- Cloudflare HTTP/3 implementation 2024
