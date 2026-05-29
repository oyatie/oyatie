---
ip_id: IP-024
microservice: finops-portal
bounded_context: commitment-management
layer: api
related_adrs: [ADR-0253, ADR-0258]
---

# IP-024 — commitment-management gRPC

## Goal

proto3 gRPC for commitment-management. mTLS via SPIFFE. Internal-only (substrate caller).

## Acceptance

- proto file extended in `contracts/cost-allocation-policy-internal.proto`.
- SemVer policy in proto file header.
- Per-call audit chain seal.
