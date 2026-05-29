---
id: ADR-identity-001
scope: microservice
microservice: identity
status: Accepted
date: 2026-05-18
owner: axis-identity + ops-security
related: [ADR-0187, ADR-0117]
---

# ADR-identity-001 — JWKS rotation cadence

## Context

OIDC signing keys must rotate to bound the blast radius of a compromise. Rotation too frequent imposes JWKS-cache invalidation cost on consumers; too infrequent expands the compromise window.

## Decision

- **Scheduled rotation cadence: 90 days** per pack.
- **Emergency rotation: ≤15 minutes** end-to-end on key-compromise suspicion.
- **JWKS grace window: 24 hours** for scheduled rotations (old kid remains in JWKS for 24h after new kid published).
- **JWKS endpoint cache TTL: 24 hours** (consumer-side).
- **HSM-backed signing in regulated packs** (pack-kr, pack-eu-pii-regulated, pack-us-healthcare, pack-ksa, pack-ae); software keys in sandbox + non-regulated.

## Consequences

- 90d rotation is industry-conservative (Google, AWS rotate every 12-24 weeks).
- 24h grace window aligns with consumer 24h JWKS cache TTL.
- Emergency rotation runbook (`runbooks/jwks-rotation.md`) is sev-1.
