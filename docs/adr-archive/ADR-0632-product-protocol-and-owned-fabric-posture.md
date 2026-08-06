---
id: ADR-0632
title: "Public product protocols, internal RPC, transport security, serialization, telemetry, and provider-owned fabric posture"
status: Superseded
doc_status: published
planning_impact: true
deciders: founder
date: 2026-08-01
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-0704]
depends_on: [ADR-0515, ADR-0562, ADR-0565]
amends: [ADR-0051, ADR-0056, ADR-0105, ADR-0157, ADR-0159, ADR-0167, ADR-0169, ADR-0176, ADR-0182, ADR-0185, ADR-0258, ADR-0478, ADR-0479, ADR-0480, ADR-0481, ADR-0565]
related: [ADR-0203, ADR-0211, ADR-0213, ADR-0246, ADR-0358, ADR-0394, ADR-0506, ADR-0561]
milestone: W0-B
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0632: Product protocol and owned-fabric posture

## Status

**Accepted — 2026-08-01.** This records the founder decisions required to close the
capability-first reorganization planning HOLD. It is normative for new and reorganized product
surfaces. Existing surfaces become migration inventory; they do not gain an exemption merely by
predating this decision.

The machine-readable authority is `specs/product-protocol-contract.json`. The fail-closed
`ci/facade/product-protocol-policy` gate keeps that contract aligned with the API-contract SSOT,
the endpoint transport profile, and the root hub.

## Context

The repository mixed public and internal transport choices, described gRPC without a stable
exposure boundary, and carried ambitious fabric technologies as if they were immediate product
requirements. A capability-first layout needs the opposite: product contracts must be stable and
customer-shaped; internal transports must remain replaceable behind ports; provider-owned physical
networking must not leak into application architecture; future performance technologies need an
evidence gate rather than speculative adoption.

## Decision

### D1 — Public product contract: REST, versioned webhooks, events, and deliberate streaming

Public synchronous APIs are HTTPS REST documented by OpenAPI 3.2.0. Public asynchronous delivery
uses versioned, authenticated, signed, idempotent, replay-protected webhooks described with
AsyncAPI 3.1.0; CloudEvents 1.0.2 is the event envelope where its stable HTTP binding applies.

SSE is the default for server-to-browser one-way streams. WebSocket is allowed only for a real
bidirectional session. AsyncAPI describes WebSocket messages, but Oyatie does **not** claim a stable
CloudEvents WebSocket binding.

GraphQL remains forbidden by ADR-0565. Public gRPC, gRPC-Web, and Connect are also forbidden. A
future public RPC surface requires a separate Accepted ADR and compatibility profile; internal use
does not silently create a public contract.

### D2 — Internal service RPC: Protobuf and gRPC over HTTP/2

Internal typed RPC uses Protocol Buffers with the current proto3 language profile and gRPC's
canonical HTTP/2 transport. Workload identity is SPIFFE-shaped mTLS with TLS 1.3. Removed Protobuf
fields reserve both names and numbers, field numbers are never reused, and wire compatibility is
tested separately from semantic compatibility.

Protobuf Editions 2024 is the official forward evolution path, but it is not activated by this
ADR. The current Rust and cross-language generator contract remains proto3 until a separate
qualification proves prost/tonic and every required SDK profile. That preserves the founder's
chosen canonical format without freezing a future migration door shut.

### D3 — TLS 1.3 and HTTP/3 edge preference

Normative language says **TLS**, never SSL. TLS 1.3 is the floor for controlled public and internal
network endpoints. Internal workload and control-plane communication requires mTLS plus issuance,
rotation, revocation, and authorization; encryption without an identity lifecycle is insufficient.
Public client certificates are profile-gated rather than a universal browser/API requirement.

HTTP/3 is preferred at capable public edges and advertised through the endpoint transport profile.
HTTP/2 is the mandatory fallback, and application correctness may not depend on QUIC availability.
Internal gRPC stays on HTTP/2 until a separately qualified transport profile exists.

### D4 — One canonical schema system

Public JSON/OpenAPI and internal Protobuf projections come from the Rust-native contract SSOT.
FlatBuffers is not an independent source of truth. It may exist only as a derived adapter for an
isolated hot path after a reproducible benchmark proves material latency or zero-copy benefit and a
schema-evolution review proves that the second wire format will not drift.

### D5 — Telemetry: OpenTelemetry plus Cilium/Hubble, with eBPF behind evidence

OpenTelemetry remains the application-semantic baseline. Cilium/Hubble provides network-flow and
policy visibility where Cilium is the Kubernetes dataplane. Neither kernel telemetry nor flow logs
replace tenant, workflow, authorization, and business-level spans, metrics, logs, and events.

OpenTelemetry eBPF instrumentation is pilot-only while its relevant surfaces remain pre-1.0.
Custom eBPF programs require a measured observability or operations gap, security review, kernel and
privilege compatibility evidence, bounded cardinality, rollback, and a proven inability of
Hubble/OpenTelemetry/provider telemetry to meet the acceptance criterion.

### D6 — Provider-owned physical fabric; advanced network operations stay opt-in

Physical Clos, underlay BGP, EVPN/VXLAN, switch ASICs, and switch NOS operation are provider
responsibilities until Oyatie demonstrably owns physical or bare-metal fabric. RFC 7938 and RFC
8365 are design references, not application dependencies.

SONiC and SAI are deferred because Oyatie is not currently a switch operator or vendor. gNMI and
gNOI are deferred until an owned-device profile exists; mutating operations always require a
separate Accepted profile with RBAC, change control, rollback, and device-fleet evidence. UEC is a
future AI/HPC interoperability candidate, not a W0 dependency.

Cilium BGP may advertise Kubernetes routes to an external router, but it is not treated as proof
that Oyatie owns or programs the physical fabric.

### D7 — Managed storage first; NVMe/TCP before RDMA

Provider-managed block and object storage remain the default. If storage disaggregation becomes a
measured requirement, NVMe/TCP is the first profile to benchmark because it runs over ordinary IP
networks. RoCEv2, NVMe/RDMA, Fibre Channel, and UEC fabrics are not product requirements. They need
workload evidence plus NIC, congestion control, loss management, telemetry, interoperability,
operator, and rollback qualification.

## Consequences

- Public compatibility is browser, webhook, and HTTP ecosystem shaped; internal transports remain
  behind capability ports.
- There is one canonical internal schema system rather than parallel Protobuf and FlatBuffers
  authorship.
- TLS/mTLS identity and fallback behavior are contract fields, not implementation folklore.
- Advanced networking cannot become reorganization scope merely because it is hyperscaler-adjacent.
- Performance exceptions are activated by measurements and separate Accepted profiles.

## Rejected alternatives

- **Public gRPC/gRPC-Web/Connect by default:** rejected; it expands customer compatibility and
  browser tooling without a product need.
- **GraphQL federation/BFF:** rejected by ADR-0565.
- **FlatBuffers as a second canonical schema:** rejected absent benchmark evidence.
- **HTTP/3-only edge:** rejected because UDP/middlebox/provider variability makes HTTP/2 fallback
  operationally necessary.
- **Owning SONiC/SAI/gNMI/gNOI now:** rejected because the current physical fabric is provider-owned.
- **RoCEv2/NVMe-RDMA as a baseline:** rejected because ordinary managed storage or NVMe/TCP must
  first fail a measured objective.

## Official references

- OpenAPI 3.2.0: <https://spec.openapis.org/oas/v3.2.0.html>
- AsyncAPI 3.1.0: <https://www.asyncapi.com/docs/reference/specification/v3.1.0>
- CloudEvents: <https://github.com/cloudevents/spec>
- SSE: <https://html.spec.whatwg.org/dev/server-sent-events.html>
- WebSocket: <https://www.rfc-editor.org/rfc/rfc6455.html>
- gRPC core concepts: <https://grpc.io/docs/what-is-grpc/core-concepts/>
- Protobuf proto3 and Editions: <https://protobuf.dev/programming-guides/proto3/> and
  <https://protobuf.dev/editions/overview/>
- TLS 1.3 and HTTP/3: <https://www.rfc-editor.org/rfc/rfc8446.html> and
  <https://www.rfc-editor.org/rfc/rfc9114.html>
- FlatBuffers evolution: <https://flatbuffers.dev/evolution/>
- BGP Clos and EVPN: <https://www.rfc-editor.org/rfc/rfc7938.html> and
  <https://www.rfc-editor.org/rfc/rfc8365.html>
- SONiC and SAI: <https://github.com/sonic-net/SONiC/wiki/Architecture> and
  <https://www.opencompute.org/community/sai>
- UEC specification history: <https://ultraethernet.org/specification-history/>
- gNMI and gNOI: <https://openconfig.net/docs/gnmi/gnmi-specification/> and
  <https://github.com/openconfig/gnoi>
- Hubble and Cilium BGP: <https://docs.cilium.io/en/stable/observability/hubble/index.html> and
  <https://docs.cilium.io/en/latest/network/bgp-control-plane/bgp-control-plane/>
- OpenTelemetry and eBPF instrumentation: <https://opentelemetry.io/docs/> and
  <https://opentelemetry.io/docs/zero-code/obi/>
- NVMe specifications: <https://nvmexpress.org/specifications/>

## Artifact accounting

ADR-0632 is the justification anchor for:

- `specs/product-protocol-contract.json`
- `ci/facade/product-protocol-policy/`
- `registry/catalog/ci-product-protocol-policy.yaml`

The existing API-contract SSOT, endpoint transport profile, root hub, artifact-capabilities
registry, reachability registry, required workflow, and gate-self-conformance policy are amended
in place and retain their existing ownership boundaries.
