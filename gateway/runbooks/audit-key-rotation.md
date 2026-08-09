# Runbook — Audit signing key rotation

**Authority:** ADR-0296 (library-first credential sidecar) + ADR-0263.
**Owner:** ops-security.
**Trigger SEV:** SEV-3 (scheduled) / SEV-1 (emergency).
**Last reviewed:** 2026-05-20.

## A — Trigger conditions

- Scheduled: every 90 days per ADR-0296.
- Emergency: signing key compromise.

## B — Procedure (scheduled)

1. `oyatie-openbao secret rotate --path secret/audit/api-gateway/<cell-id>`.
2. Sidecar pulls new key within 60s.
3. New audit events signed by new key.
4. Old key retained in Merkle chain for verification.
5. Audit: `oya.api_gateway.audit.key.rotated`.

## C — Procedure (emergency)

1. Declare SEV-1.
2. Quarantine compromised key: mark in OpenBao `tombstone: true`.
3. Issue new key + push to sidecar.
4. Re-sign any audit events created in the suspected compromise window (gap forensics).

## D — Verification

- New key signing audits.
- Merkle chain validates with new key.

## E — References

- ADR-0263, ADR-0296
- `microservices/audit-chain/runbooks/`
