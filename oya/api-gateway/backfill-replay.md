# api-gateway — Backfill + replay

**Authority:** ADR-0276 (backup portability GDPR Art. 20) + ADR-0263 (observability emission).

## A — What can be replayed

The gateway is *stateless* on the request data-plane; it does not store request bodies. What IS retained and replayable:

| Data | Retention | Replay surface |
|---|---|---|
| Audit events (admit/deny/WAF/rate-limit/TLS/bot/cedar/upstream/canary/blue-green/tls.cert.rotated/ech.config.rotated/pqc.handshake.completed) | 7y (per ADR-0028 audit-chain doctrine) | JSON-LD export via `oyatie-audit-export` |
| Rate-limit counters (per-tenant, per-fingerprint) | 30d | Per-tenant CSV export |
| Bot-score history (per-fingerprint) | 30d | Per-tenant CSV export |
| Cedar fragment history | indefinite (per ADR-0294) | Git ledger at policy-engine µservice |
| TLS cert history | indefinite | `iac/cert-manager-history.yaml` |
| ECH config history | indefinite | `iac/ech-config-history.yaml` |

## B — Per-tenant portable export (GDPR Art. 20)

The gateway emits audit events that may relate to a data subject. On a portability request:

1. Identify principal context hash for the data subject (via identity µservice).
2. Query audit-chain µservice for all events where `principal_context_hash` matches.
3. Filter to events with `tenant_id == requested-tenant`.
4. Export to JSON-LD per ADR-0276 schema.
5. Deliver via secure download link (signed URL, expiry 7 days).

CLI:

```bash
oyatie-audit-export \
  --tenant <tenant-id> \
  --principal-hash <hash> \
  --since 2020-01-01 \
  --until $(date -u +%FT%T.000Z) \
  --format jsonld \
  --output portable-export-<tenant>-<ts>.jsonld
```

## C — Cedar fragment replay

For audit reconstruction (e.g. "was this request denied by the right Cedar fragment at the time?"):

```bash
oyatie-cedar-replay \
  --request-id <id> \
  --fragment-snapshot-at <timestamp> \
  --output replay.json
```

Cedar fragments are versioned and tagged with their soak-end timestamp per ADR-0294. Replay uses the snapshot active at request time.

## D — Rate-limit replay

If a tenant disputes a 429:

```bash
oyatie-rate-limit-replay \
  --tenant <tenant-id> \
  --window 2026-05-20T12:00:00Z/2026-05-20T13:00:00Z \
  --output bucket-history.json
```

Returns the bucket fills/drains over the requested window.

## E — TLS / ECH / PQC replay

For audit on protocol-level events:

```bash
oyatie-tls-replay \
  --connection-id <quic-conn-id> \
  --output handshake-trace.json
```

Returns the TLS 1.3 + ECH + PQC handshake parameters (without leaking session keys, which are not retained).

## F — Erasure-aware replay

If a data subject has exercised the right to erasure (GDPR Art. 17), the audit chain retains the event (per Art. 17(3)(b) legal-obligation exemption) BUT replaces PII-content fields with `[redacted-2026-05-20]`. Replay reflects this redaction.

## G — References

- ADR-0276, ADR-0263, ADR-0028, ADR-0294
- `microservices/audit-chain/` (downstream µservice)
- `microservices/api-gateway/dpia.md`
