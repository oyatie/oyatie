---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-foundry-evidence-frontend
impl_plan_id: IP-009-evidence-query-stack
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-evidence
acceptance_lanes: [cargo-clippy, lean-layer-correctness, lean-cross-tenant-leak-prevention, load-drill-evidence-query]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: Evidence-query stack

## Intent

`oya-foundry-evidence-evidence-query-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,sdk}`: Cedar-gated, tenant-scoped per-invocation query API + dashboards data path. Every read is itself audit-emitted (audit-of-audits). p99 ≤ 100 ms.

## ChangeSet boundary

8 Rust crates.

## Concrete File Targets

| Crate | Layer | Purpose |
|---|---|---|
| `oya-foundry-evidence-evidence-query-kernel` | kernel | port traits + types |
| `oya-foundry-evidence-evidence-query-domain` | domain | query construction; data-class filtering rules |
| `oya-foundry-evidence-evidence-query-usecase` | usecase | QueryUsecase: Cedar evaluate + Postgres query + audit-of-audits emit |
| `oya-foundry-evidence-evidence-query-api` | api | re-exports |
| `oya-foundry-evidence-evidence-query-adapter` | adapter | generic glue |
| `oya-foundry-evidence-evidence-query-adapter-postgres` | adapter | B-tree-indexed query against evidence_pack table; read-replica routing |
| `oya-foundry-evidence-evidence-query-rest` | rest | axum router; OpenAPI conformance |
| `oya-foundry-evidence-evidence-query-sdk` | sdk | streaming client per `sdk-plan.md` |

## Acceptance Gates

```bash
cargo check -p oya-foundry-evidence-evidence-query-kernel
cargo check -p oya-foundry-evidence-evidence-query-domain
cargo check -p oya-foundry-evidence-evidence-query-usecase
cargo check -p oya-foundry-evidence-evidence-query-api
cargo check -p oya-foundry-evidence-evidence-query-adapter
cargo check -p oya-foundry-evidence-evidence-query-adapter-postgres
cargo check -p oya-foundry-evidence-evidence-query-rest
cargo check -p oya-foundry-evidence-evidence-query-sdk
cargo nextest run -p oya-foundry-evidence-evidence-query-rest --test cross_pack_refusal
cargo nextest run -p oya-foundry-evidence-evidence-query-usecase --test audit_of_audits_emit
oya gate validate cross-tenant-leak-prevention --microservice foundry-evidence
oya gate validate load-drill-evidence-query --microservice foundry-evidence
```

## Halt Conditions

- p99 evidence-query drill exceeds 100 ms — block.
- Cross-tenant leak drill returns non-zero — block.
- audit-of-audits emit missing for any read — block (Bominal ADR-0028 §"Self-observability").
- Plaintext returned without Cedar PERMIT 3 evaluated true — block (data-class gating).

## Next IP

[`IP-010-regulator-export-stack.md`](IP-010-regulator-export-stack.md)

## References

- `policy/tenant-scope.cedar`, `policy/auditor-scope.cedar`.
- ADR-0028 (audit-of-audits).
