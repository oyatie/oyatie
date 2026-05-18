---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-sites-foundation
impl_plan_id: IP-013-contracts-and-capabilities
status: pending
execution_unit: ChangeSet
owner: axis-sites + foundry-providers + council-privacy
acceptance_lanes: [openapi-lint, asyncapi-lint, proto-lint, oya-governance-capability-tier-lint]
---

# IP-013: OpenAPI + AsyncAPI + Proto contracts + capabilities (T0/T1/T2)

## Intent

Author contracts/openapi/sites.yaml, contracts/asyncapi/sites-events.yaml, contracts/proto/sites.proto, and capabilities/T0-suggest.yaml + T1-assist.yaml + T2-auto.yaml. EU AI Act bounds explicit on T2 per ADR-SITES-0006.

## ChangeSet boundary

3 contract files + 3 capability files.

## Acceptance Gates

```bash
spectral lint microservices/sites/contracts/openapi/sites.yaml --ruleset spectral:oas
spectral lint microservices/sites/contracts/asyncapi/sites-events.yaml --ruleset spectral:asyncapi
buf lint microservices/sites/contracts/proto/sites.proto
cargo run -p oya-dev-cli -- gate validate capability-tier-lint --microservice sites
```

## ChangeSet metadata

```yaml
changeset_id: CS-SITES-IP-013-contracts-and-capabilities
depends_on_changesets: [CS-SITES-IP-003-site-and-page-bcs]
parallel_safe_with_changesets: [CS-SITES-IP-012-policy-dpia-threat-model, CS-SITES-IP-014-dashboards-runbooks-slos]
enables: [CS-SITES-IP-015-hg-sites-maturity-claim]
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | OpenAPI 3.1 spec lints clean against `spectral:oas` ruleset | `spectral lint microservices/sites/contracts/openapi/sites.yaml --ruleset spectral:oas` |
| AC-02 | AsyncAPI 3.0 spec lints clean against `spectral:asyncapi` | `spectral lint microservices/sites/contracts/asyncapi/sites-events.yaml --ruleset spectral:asyncapi` |
| AC-03 | Protobuf lints clean (`buf lint`) and breaking-change check passes | `buf lint microservices/sites/contracts/proto/sites.proto` |
| AC-04 | T0/T1/T2 capability YAMLs declare EU AI Act risk class per ADR-SITES-0006 | `cargo run -p oya-dev-cli -- gate validate capability-tier-lint --microservice sites` |
| AC-05 | T2 ai-page-build capability marked REFUSED for legal/medical/employment overlays | `cargo nextest run --test capability_t2_refusal_overlays` |

## Build Sequence

1. Author `contracts/openapi/sites.yaml` (REST surface mapping to BC usecases).
2. Author `contracts/asyncapi/sites-events.yaml` (publish/page-publish/cms-update events).
3. Author `contracts/proto/sites.proto` (gRPC interop surface).
4. Author `capabilities/T0-suggest.yaml`, `T1-assist.yaml`, `T2-auto.yaml`.
5. Run `spectral lint` + `buf lint` + capability-tier-lint gate.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-sites FR | FR-22 (T2 AI page build), FR-27 (webhooks) |
| PRD-sites AC | AC-15 (contract conformance) |
| ADR | ADR-SITES-0006 (EU AI Act bounds) |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Breaking proto change ships unseen | `buf breaking` lane refuses incompatible diff |
| Capability tier mis-classified (T2 vs T1) | EU AI Act risk class explicit in YAML; gate enforces |
| OpenAPI drift from REST handler | OpenAPI spec hash pinned; CI lane |

## References

- OpenAPI 3.1 specification (`spec.openapis.org/oas/v3.1.0`).
- AsyncAPI 3.0 specification (`asyncapi.com/docs/reference/specification/v3.0.0`).
- Protocol Buffers v3 language guide (`protobuf.dev/programming-guides/proto3`).
- buf CLI documentation (`buf.build/docs`).
- Spectral linter ruleset (`stoplight.io/p/docs/gh/stoplightio/spectral`).
- EU AI Act final text — Regulation (EU) 2024/1689.
- ADR-SITES-0006 (EU AI Act bounds).
- agent-skills api-and-interface-design SKILL.md.
