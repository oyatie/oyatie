# Runbook — ECH key rotation

**Authority:** ADR-0253 (ECH per draft-ietf-tls-esni-22 + RFC 9460).
**Owner:** axis-network + ops-security.
**Trigger SEV:** SEV-3 (scheduled) / SEV-1 (emergency).
**Last reviewed:** 2026-05-20.

## A — Trigger conditions

- Scheduled: every 90 days per ADR-0253.
- Emergency: ECH private key compromise.
- DNS provider: HTTPS RR config-id mismatch detected.

## B — Pre-checks

1. Confirm OpenBao reachable: `oyatie-openbao-test --path secret/ech`.
2. Confirm DNS provider API reachable (Cloudflare / NS1 / sov-cell DNS).
3. Verify current ECH key in service: `oyatie-ech-status --cell <cell-id>`.

## C — Procedure (scheduled)

1. **Generate new ECH config:**
   - `oyatie-ech-gen --cell <cell-id> --output new-ech.bin`.
   - Internal: ED25519 keypair + ECHConfig (config-id, version, public-name).
2. **Stage new ECH config in OpenBao:**
   - `oyatie-openbao secret write secret/ech/<cell-id>/staged @new-ech.bin`.
3. **Update HTTPS RR in DNS** (publish dual config-ids during transition):
   - Cloudflare: `curl -X POST .../zones/$ZONE/dns_records -d '{"type":"HTTPS","name":"@","content":"1 . alpn=h3,h2 ipv6hint=::1 ech=<base64(new-ech)+base64(old-ech)>"}'`.
   - Soak ≥1h to let DNS cache update globally.
4. **Activate new config on gateway:**
   - `oyatie-ech-activate --cell <cell-id> --staged`.
   - SDS push to Envoy; hot-reload.
5. **Audit:** `oya.api_gateway.ech.config.rotated`.
6. **Verify client ECH handshake** with new config:
   - `openssl s_client -connect api.oyatie.com:443 -ech_config_list_from_dns`.
7. **After 7 days soak, remove old config from DNS:**
   - Update HTTPS RR to single new config-id.

## D — Procedure (emergency)

1. **Declare SEV-1.**
2. **Skip dual-publish** (no time for 1h DNS soak):
   - Push new ECH config to gateway immediately.
   - Update DNS to new config-id only.
3. **Accept short window** where some clients fall through to standard TLS 1.3 (graceful degradation per ADR-0253).
4. **Audit:** `oya.api_gateway.ech.config.rotated.emergency`.
5. **Rotate ECH private key in OpenBao:**
   - `oyatie-openbao secret rotate --path secret/ech/<cell-id>`.

## E — Verification

- New ECH config-id served in HTTPS RR.
- Client ECH handshake succeeds with new config.
- Audit event Merkle-sealed.

## F — Rollback

If new ECH config breaks clients:

1. Revert DNS HTTPS RR to old config-id.
2. Pause new config activation: `oyatie-ech-activate --cell <cell-id> --revert`.
3. Note: ECH is graceful-degradation; broken ECH means clients fall through to standard TLS 1.3, NOT broken connections.

## G — Post-incident

1. Document new ECH key in `iac/ech-config-history.yaml`.
2. Postmortem if emergency.
3. CI lane: `governance-ech-readiness` should remain green.

## H — References

- ADR-0253
- IETF draft-ietf-tls-esni-22 (ECH)
- IETF RFC 9460 (HTTPS RR)
- Cloudflare ECH implementation 2024
- `iac/ech-config.yaml`
- `iac/ech-config-history.yaml`
