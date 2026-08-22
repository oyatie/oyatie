# Runbook — TLS cert rotation

**Authority:** ADR-0253 (TLS strict + ECH + PQC) + ADR-0295.
**Owner:** axis-network + ops-security.
**Trigger SEV:** SEV-3 (scheduled) / SEV-1 (emergency).
**Last reviewed:** 2026-05-20.

## A — Trigger conditions

- Scheduled: 30 / 14 / 7 days before cert expiry (cert-manager pre-alerts).
- Emergency: cert compromise (private key leak, CA breach).
- CT log: log distrust event (e.g. Argon log removed from Chrome list).

## B — Pre-checks

1. Confirm cert-manager health: `kubectl get certificate -n api-gateway` — `Ready: True`.
2. Verify cert chain: `openssl s_client -connect api.oyatie.com:443 -servername api.oyatie.com 2>/dev/null | openssl x509 -text | head -20`.
3. Confirm CA reachable: `curl https://acme-v02.api.letsencrypt.org/directory -I` → 200.
4. Confirm OpenBao reachable for sidecar key delivery: `oyatie-openbao-test --path secret/tls`.
5. Confirm CT logs reachable: `curl https://ct.cloudflare.com/logs/nimbus2026/ct/v1/get-sth -I` → 200.

## C — Procedure (scheduled)

1. **Initiate renewal:**
   - `kubectl annotate certificate api-gateway-cert cert-manager.io/renew=true -n api-gateway`.
   - cert-manager issues new cert from CA.
   - Timing budget: ≤10min.
2. **Sidecar reload:**
   - Sidecar polls OpenBao every 60s; picks up new cert within 60s.
   - Audit: `oya.api_gateway.tls.cert.rotated`.
3. **Envoy hot-restart:**
   - SDS push to Envoy; no connection drop (SDS hot-cert-reload).
   - Verify: `kubectl exec -it <envoy-pod> -- curl http://127.0.0.1:9901/certs | jq '.certificates'`.
4. **Verify new cert serving:**
   - `openssl s_client -connect api.oyatie.com:443 -servername api.oyatie.com 2>/dev/null | openssl x509 -enddate -noout` — confirm new expiry.
5. **Update HSTS preload tracker:**
   - `iac/cert-manager-hsts-preload.yaml` updated automatically.
6. **Verify CT inclusion:**
   - `oyatie-ct-check --cert-fingerprint <fp>` → SCTs from ≥2 logs.

## D — Procedure (emergency — cert compromise)

1. **Declare SEV-1** in `#api-gateway-warroom`.
2. **Revoke compromised cert via CA:**
   - `acme.sh --revoke -d api.oyatie.com --reason 1` (1 = keyCompromise).
3. **Push revocation to CRL + OCSP:**
   - Wait ≤5min for OCSP propagation; clients with must-staple will fail-hard.
4. **Issue new cert + new keypair:**
   - `kubectl delete certificate api-gateway-cert -n api-gateway && kubectl apply -f iac/cert-manager.yaml`.
   - Timing budget: ≤15min.
5. **Roll OpenBao TLS private key** (per ADR-0296):
   - `oyatie-openbao secret rotate --path secret/tls`.
6. **Pin alternate CA** if breach is suspected (per `iac/cert-manager.yaml`):
   - Swap `issuerRef` from primary CA to backup CA.
7. **Audit:** `oya.api_gateway.tls.cert.rotated.emergency`.

## E — Verification

- `openssl s_client` shows new cert + new expiry.
- HSTS preload list updated.
- OCSP staple valid.
- ≥2 SCTs from independent CT logs.
- All cells in all regions serve new cert (verify via per-cell `openssl s_client`).

## F — Rollback

If new cert breaks legitimate clients (e.g. cert-pinning app rejected new fingerprint):

1. Pause cert-manager rotation: `kubectl annotate certificate ... cert-manager.io/renew=false`.
2. SDS push reverts to last-known-good cert.
3. Investigate why pinned fingerprint rejected; coordinate with mobile app team.

## G — Post-incident

1. Update `iac/cert-manager-history.yaml` with new cert fingerprint.
2. Postmortem if emergency.
3. Update mobile app cert-pin list per `runbooks/cert-pin-rotation.md` (cross-µservice).

## H — References

- ADR-0253, ADR-0295, ADR-0296
- `iac/cert-manager.yaml`
- `iac/cert-manager-hsts-preload.yaml`
- cert-manager 1.16 documentation
- IETF RFC 8446 (TLS 1.3), RFC 6962 (CT)
