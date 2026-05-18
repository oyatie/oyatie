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

## References

- OpenAPI 3.1.
- AsyncAPI 3.0.
- ADR-SITES-0006 (EU AI Act bounds).
- agent-skills api-and-interface-design SKILL.md.
