# Runbook — PQC cert rotation

**Authority:** ADR-0253 (PQC hybrid X25519MLKEM768 + ed25519+ml_dsa_65).
**Owner:** axis-network + ops-security.
**Trigger SEV:** SEV-3 (scheduled) / SEV-0 (algorithm break).
**Last reviewed:** 2026-05-20.

## A — Trigger conditions

- Scheduled: per cert validity period (default 90 days for Let's Encrypt).
- Emergency: PQC algorithm break (e.g. NIST decertifies MLKEM-768).
- Crypto-agility test: quarterly.

## B — Pre-checks

1. Confirm PQC-enabled CA reachable (Let's Encrypt PQC pilot, or oyatie-rooted sigstore CA).
2. Confirm cert-manager has PQC chain support (`cert-manager >= 1.16`).
3. Confirm Envoy QUIC + TLS stack has PQC support (BoringSSL or Rustls with ml-kem).
4. Verify OpenBao PQC key storage: `oyatie-openbao secret read secret/pqc`.

## C — Procedure (scheduled)

1. **Generate hybrid keypair** (X25519 + MLKEM-768):
   - `oyatie-pqc-gen --hybrid x25519-mlkem768 --output pqc-key.bin`.
2. **Sign with hybrid CA** (ed25519 + ml_dsa_65):
   - `oyatie-pqc-sign --cert-template api.oyatie.com --sig-alg ed25519+ml_dsa_65`.
3. **Stage in OpenBao:**
   - `oyatie-openbao secret write secret/pqc/<cell-id>/staged @pqc-key.bin`.
4. **Push to Envoy via SDS:**
   - cert-manager sees new cert; SDS publish to Envoy; hot-reload.
5. **Audit:** `oya.api_gateway.pqc.cert.rotated`.
6. **Verify hybrid handshake:**
   - `openssl s_client -connect api.oyatie.com:443 -groups X25519MLKEM768` → expects `Server Temp Key: X25519MLKEM768`.

## D — Procedure (emergency — algorithm break)

1. **Declare SEV-0.**
2. **Switch to FrodoKEM fallback** (NIST round-2 alternate):
   - Per `iac/pqc-cert.yaml`: change `kem_algorithm: X25519FrodoKEM640`.
3. **Issue new hybrid certs** with FrodoKEM.
4. **Coordinate global rollout** within 24h.
5. **Customer comms:** sov-cell tenants notified; SLA hold per contract.
6. **Audit:** `oya.api_gateway.pqc.algorithm.swapped`.

## E — Verification

- New PQC hybrid cert served.
- Client handshake negotiates new KEM.
- Non-PQ clients still fall through to classical TLS 1.3 (per ADR-0253 graceful degradation).
- `oya_api_gateway_pqc_handshake_negotiated_ratio` recovers to baseline.

## F — Rollback

If new PQC chain breaks clients (highly unlikely; should be graceful degradation):

1. Revert cert chain via cert-manager.
2. Investigate client-side PQC stack compatibility.

## G — Post-incident

1. Document new PQC algorithm in `iac/pqc-cert-history.yaml`.
2. Update `docs/standards/crypto-agility.md`.
3. Postmortem if emergency.

## H — References

- ADR-0253
- IETF draft-kwiatkowski-tls-ecdhe-mlkem-02 (PQC TLS hybrid)
- NIST FIPS 203 (ML-KEM)
- NIST FIPS 204 (ML-DSA)
- Cloudflare PQC implementation 2024
- `iac/pqc-cert.yaml`
- `iac/pqc-cert-history.yaml`
