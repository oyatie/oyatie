---
doc_class: Runbook
template_id: TPL-RUNBOOK
title: Affinity-Attestation Issuer-Key Rotation
microservice: anonymous
severity: "Sev-2 (planned rotation) / Sev-1 (compromise-driven)"
status: Accepted
owner_team: axis-anonymous + ops-security
date: 2026-05-17
related_adrs: [ADR-ANON-0002]
related_artifacts:
  - microservices/anonymous/policy/affinity-attestation-verification.md
doc_status: published
---

# Runbook: Affinity-Attestation Issuer-Key Rotation

## Trigger

| Scenario | Severity | Cadence |
|---|---|---|
| Planned 12-month rotation per ADR-ANON-0002 rotation policy | Sev-2 | annual per issuer |
| Suspected issuer-private-key compromise | Sev-1 | on-detection |
| Issuer migration to new IdP / domain (e.g., employer rebrand) | Sev-2 | on-request |
| FIPS module re-certification (vendor update) | Sev-3 | per re-cert |

## Pre-checks

1. Confirm the rotation kind (planned vs compromise-driven).
2. For compromise: confirm with ops-security; capture the compromise vector for post-mortem.
3. List affected issuers: `cargo run -p oya-dev-cli -- anonymous affinity-attestation list-issuers --rotation-window`
4. List currently-bound affinity attestations under the old key: `cargo run -p oya-dev-cli -- anonymous affinity-attestation list-bindings --issuer-key-id <old-id>`
5. Plan migration window (planned rotation; not compromise): typically a 30-day overlap during which both keys verify, then old key is retired.

## Steps — Planned rotation (Sev-2)

| Step | Action | Time budget |
|---|---|---|
| 1 | Coordinate with the issuer to generate a new signing key (issuer-side; not platform-side) | 1-3 days |
| 2 | Issuer registers new public key via `POST /v1/affinity-issuers/<issuer-id>/keys` | ≤ 1h |
| 3 | Platform records new public key in `AffinityIssuerRegistry` with audit-chain seal; both keys are now `active` | ≤ 5 min |
| 4 | Issuer re-issues credentials to its constituency using the new key (users receive a refreshed BBS+ credential) | 30 days |
| 5 | After 30 days: list residual bindings under old key (`cargo run -p oya-dev-cli -- anonymous affinity-attestation list-bindings --issuer-key-id <old-id>`); affected users receive a forced-rebind notification | ≤ 1 day |
| 6 | Retire old key: `cargo run -p oya-dev-cli -- anonymous affinity-attestation retire-key --issuer-id <id> --key-id <old-id>` | ≤ 5 min |
| 7 | Audit-chain seal records `AffinityIssuerKeyRetired` event | ≤ 5 min |

## Steps — Compromise-driven rotation (Sev-1)

| Step | Action | Time budget |
|---|---|---|
| 1 | Declare Sev-1 in `#inc-<id>` Slack channel; assign IC | ≤ 5 min |
| 2 | Immediately mark old key as `revoked` in `AffinityIssuerRegistry`: `cargo run -p oya-dev-cli -- anonymous affinity-attestation revoke-key --issuer-id <id> --key-id <compromised>` | ≤ 1 min |
| 3 | Invalidate all existing bindings under the compromised key: `cargo run -p oya-dev-cli -- anonymous affinity-attestation invalidate-bindings --issuer-key-id <compromised>` | ≤ 5 min |
| 4 | Notify all affected users: bindings invalidated; re-bind required | ≤ 30 min |
| 5 | Coordinate with issuer to generate a new key under controlled key-ceremony | 24-48h |
| 6 | Issuer registers new key | ≤ 1h |
| 7 | Issuer re-issues credentials | per issuer-side scale |
| 8 | Post-mortem within 5 business days (compromise vector, MTTR, mitigation) | – |

## Failure modes

| Failure | Mitigation | Severity escalation |
|---|---|---|
| Issuer cannot generate a new key in time | extend overlap window; mark old key `expiring_soon`; notify users to re-bind | Sev-2 → Sev-1 if compromise |
| Audit-chain seal fails during retirement | abort retirement; investigate audit-chain incident; retry | Sev-1 |
| User cannot re-bind (issuer-side outage) | issuer's responsibility; user's binding remains invalidated until issuer recovers | per-issuer |

## Cross-µservice coordination

- `tenancy`: notify of issuer-status change so tenant operators can re-bind
- `audit-chain`: every key registration, retirement, revocation is sealed
- `observability`: rotation events emit a metric `oya_anonymous_issuer_key_rotation_total{kind, status}`

## References

- ADR-ANON-0002 — affinity-attestation verification (§"Issuer Registration", §"Key Rotation")
- W3C Verifiable Credentials 2.0 — `keyRotation` schema
- NIST SP 800-57 (key management) Part 1 — recommended rotation cadence
