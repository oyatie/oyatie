---
doc_class: Standard
template_id: TPL-STANDARD
standard_id: step-up-auth-classes
status: Accepted
date: 2026-05-18
owner_team: axis-identity + council-architecture
related_adrs: [ADR-0189]
related_lanes: [lean-a15-step-up-acr-coverage]
---

# Step-Up Authentication ACR Classes — Standard

Per ADR-0189, every API path that mutates state declares `x-acr-required: <class>` in its OpenAPI spec. This document is the canonical mapping per operation class. New µservices MUST consult this table when authoring contracts; deviations require an ADR.

## ACR enum (canonical)

| Class | Factor minimum | Session age max | Idle max |
|---|---|---|---|
| `routine` | 1 (Passkey OR password+TOTP) | 24h | 4h |
| `elevated` | 1 (Passkey ONLY) | 4h | 1h |
| `sensitive` | 2 (Passkey + step-up Passkey OR hardware key) | 1h | 15min |
| `critical` | 2 + IT-approval (hardware key + JIT IT-approval) | 15min | 5min |

## Mapping operations → ACR

### Reads

| Operation class | ACR |
|---|---|
| Read own profile / list own resources | routine |
| List tenant users (admin) | routine |
| Read aggregated metrics | routine |
| Read PII export | elevated |
| Read all-tenant view (super-admin) | sensitive |
| Read full audit-chain export | sensitive |

### Mutations

| Operation class | ACR |
|---|---|
| Update own profile | elevated |
| Create / update normal resources | elevated |
| Invoke workflow | elevated |
| Delete normal resources | elevated |
| Suspend user | elevated |
| Add WebAuthn credential | elevated |
| Delete WebAuthn credential | sensitive |
| Rotate secret | sensitive |
| Bind external IdP | sensitive |
| Unbind external IdP | critical |
| Delete tenant | critical |
| Rotate JWKS signing key | critical |
| Export all-pack audit | critical |
| Operator-recovery actions | critical |
| Billing currency change | critical |

## OpenAPI extension format

```yaml
paths:
  /secrets/{id}/rotate:
    post:
      x-acr-required: sensitive
```

Optional escape hatch for explicitly pre-auth endpoints:

```yaml
paths:
  /webhooks/inbound:
    post:
      x-acr-exempt: true       # documented reason in surrounding doc
```

## Per-pack overrides

Regulated packs MAY tighten (never loosen) requirements:

- pack-us-healthcare: any operation touching PHI requires at minimum `elevated`.
- pack-kr (KR-FSS sector): financial-data operations require minimum `sensitive`.
- pack-ksa sovereign: critical-class operations require dual-operator approval.

Tightening expressed in per-pack Cedar policy overlay; OpenAPI base still declares the global minimum.

## Verification

CI lane `lean-a15-step-up-acr-coverage` runs `check-step-up-auth-coverage` against every µservice's OpenAPI spec; reports findings advisory-mode for 60 days, then blocker.

## Cross-references

- ADR-0189 (step-up ACR classes)
- ADR-0183 (Cedar policy engine separation)
- ADR-0145 (inter-µservice OIDC bearer)
- `crates/check-step-up-auth-coverage`
