---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-sites-foundation
impl_plan_id: IP-012-policy-dpia-threat-model
status: pending
execution_unit: ChangeSet
owner: ops-security + council-privacy + axis-sites
acceptance_lanes: [cedar-policy-lint, oya-governance-cedar-enforcement]
---

# IP-012: Cedar policy + DPIA + threat-model sign-off

## Intent

Finalise Cedar policies (`tenant-scope.cedar`, `ci-scope.cedar`, `auditor-scope.cedar`, `public-read.cedar`), DPIA, threat-model, compliance.md, multi-region.md, incident-response.md, failure-modes.md, backfill-replay.md. Council-privacy + ops-security sign-off recorded in audit-chain.

## ChangeSet boundary

6 Cedar policy files + 9 markdown policy/governance files. No Rust code.

## Acceptance Gates

```bash
cedar validate microservices/sites/policy/tenant-scope.cedar
cedar validate microservices/sites/policy/ci-scope.cedar
cedar validate microservices/sites/policy/auditor-scope.cedar
cedar validate microservices/sites/policy/public-read.cedar
cargo run -p oya-dev-cli -- gate validate cedar-enforcement --microservice sites
cargo run -p oya-dev-cli -- gate validate dpia-completeness --microservice sites
cargo run -p oya-dev-cli -- gate validate threat-model-completeness --microservice sites
```

## References

- ADR-0140 (Cedar policy).
- agent-skills documentation-and-adrs SKILL.md.
- agent-skills security-and-hardening SKILL.md.
