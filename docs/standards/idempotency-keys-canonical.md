---
doc_class: Standard
title: Idempotency Keys (Canonical)
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-18
owner_team: council-architecture
deciders: council-architecture, axis-foundry, axis-all-microservices
related_adrs: [ADR-0149, ADR-0145]
review_cadence: annually
doc_status: published
---

# Idempotency Keys (Canonical)

## Authority

ADR-0149-idempotency-keys-canonical landed this contract. Stripe's
`Idempotency-Key` HTTP header pattern and AWS's "ClientToken" /
`X-Amz-Idempotency-Token` pattern are the industry references. Every
state-changing operation in every oyatie microservice MUST honor the
canonical header below.

## Contract

### 1. Required on every state-changing operation

Every `POST`, `PUT`, `PATCH`, and `DELETE` request to any oyatie
REST surface MUST accept the canonical header:

```
Idempotency-Key: <opaque-256-bit-value>
```

- Format: opaque ASCII string; recommended ULID or UUIDv7.
- Length: 16-256 bytes after URL-decoding.
- Lifetime: 24 hours minimum at the server; 7 days for billing-class
  surfaces (matching Stripe).

### 2. Server contract

On receipt:

1. Compute or accept the client-supplied key.
2. Look up `(tenant_id, capability_id, idempotency_key)` in the
   idempotency store.
3. If found and the recorded request fingerprint matches: return the
   recorded response verbatim with HTTP status `200/201/204` plus
   `Idempotency-Replay: true`.
4. If found and the request fingerprint differs: return `409 Conflict`
   with `Idempotency-Conflict: fingerprint-mismatch`.
5. If absent: process the request, atomically record
   `(key, request_fingerprint, response, response_timestamp)` BEFORE
   committing side effects, then return the response.

### 3. Trait surface

Every microservice integrates the `IdempotencyKeyStore` trait from
`shared-idempotency-key-kernel`:

```rust
pub trait IdempotencyKeyStore: Send + Sync {
    fn get_or_compute<F>(&self, key: &IdempotencyKey, compute: F)
        -> Result<IdempotentResponse, IdempotencyStoreError>
        where F: FnOnce() -> IdempotentResponse;
    fn consume(&self, key: &IdempotencyKey) -> Result<(), IdempotencyStoreError>;
    fn peek(&self, key: &IdempotencyKey) -> Result<Option<IdempotentResponse>, IdempotencyStoreError>;
}
```

### 4. OpenAPI declaration

Every state-changing path operation MUST declare the parameter via
`$ref: '#/components/parameters/IdempotencyKey'`, and every
microservice OpenAPI document MUST publish that component:

```yaml
components:
  parameters:
    IdempotencyKey:
      in: header
      name: Idempotency-Key
      required: true
      schema:
        type: string
        minLength: 16
        maxLength: 256
      description: |
        Opaque idempotency key per docs/standards/idempotency-keys-canonical.md.
```

### 5. Read-path exemption

`GET` / `HEAD` / `OPTIONS` are read-only and naturally idempotent; the
header MAY be supplied but the server MUST NOT enforce uniqueness on
read paths.

### 6. Distributed-tx interaction

Idempotency keys interact with the outbox pattern (ADR-0153):
the outbox record key is keyed by `(idempotency_key, capability_id)`
so the publisher emits at most one event per state change.

### 7. Validation

The `check-idempotency-key-coverage` gate enforces that every
state-changing operation in every microservice OpenAPI document
declares the canonical `Idempotency-Key` parameter.

## References

- Stripe API — idempotent requests: https://stripe.com/docs/api/idempotent_requests
- AWS API design — idempotency tokens (ClientToken pattern).
- ADR-0149-idempotency-keys-canonical.
- ADR-0145-inter-microservice-communication-reform.
- ADR-0153-outbox-pattern.
