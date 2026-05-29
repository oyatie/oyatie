# Dashboard — TLS Health

**Owner:** axis-network + ops-security.
**Source:** `dashboards/tls-health.json`.

## Purpose

TLS 1.3 + ECH + PQC health. Cert expiry. Cipher suite distribution.

## Critical thresholds

| Panel | Alert at |
|---|---|
| TLS handshake success ratio | < 0.9995 |
| Cert expiry days | < 14 days |
| TLS 1.3 ratio | < 1.0 (any non-1.3 is a regression) |
| OCSP staple status | any non-OK |

## Runbooks

- `runbooks/tls-cert-rotation.md`
- `runbooks/ech-key-rotation.md`
- `runbooks/pqc-cert-rotation.md`

## References

- ADR-0253
- `slos/tls-handshake-success.openslo.yaml`
