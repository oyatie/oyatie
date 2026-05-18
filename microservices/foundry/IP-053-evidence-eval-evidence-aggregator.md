---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-foundry-evidence-frontend
impl_plan_id: IP-008-eval-evidence-aggregator
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-evidence + axis-foundry-eval
acceptance_lanes: [cargo-clippy, property-based-tests, eval-temporal-correctness-drill]
---

# IP-008: Eval-evidence aggregator

## Intent

`oya-foundry-evidence-eval-evidence-aggregator-{kernel,domain,usecase,adapter,worker}`: subscribe to `foundry.eval.verdict.published.v1` events; build the verdict-at-invocation-time index; expose lookup to pack-builder. Per ADR-0024.

## ChangeSet boundary

5 Rust crates.

## Concrete File Targets

| Crate | Layer |
|---|---|
| `oya-foundry-evidence-eval-evidence-aggregator-kernel` | kernel |
| `oya-foundry-evidence-eval-evidence-aggregator-domain` | domain — verdict-history indexing logic |
| `oya-foundry-evidence-eval-evidence-aggregator-usecase` | usecase — join `(invocation_ts, eval_set_id)` to current verdict |
| `oya-foundry-evidence-eval-evidence-aggregator-adapter` | adapter — Postgres verdict-history table + Workflow consumer |
| `oya-foundry-evidence-eval-evidence-aggregator-worker` | worker — leader-elected event consumer |

## Acceptance Gates

```bash
cargo check -p oya-foundry-evidence-eval-evidence-aggregator-kernel
cargo check -p oya-foundry-evidence-eval-evidence-aggregator-domain
cargo check -p oya-foundry-evidence-eval-evidence-aggregator-usecase
cargo check -p oya-foundry-evidence-eval-evidence-aggregator-adapter
cargo check -p oya-foundry-evidence-eval-evidence-aggregator-worker
cargo nextest run -p oya-foundry-evidence-eval-evidence-aggregator-usecase --test eval_join_temporal_correctness
oya gate validate eval-temporal-correctness-drill --microservice foundry-evidence
```

## Halt Conditions

- Drill detects eval verdict that postdates invocation_ts being joined — block (ADR-0024 violation).
- Worker fails to handle out-of-order event delivery — block (Workflow bus does not guarantee in-order delivery).

## Next IP

[`IP-009-evidence-query-stack.md`](IP-009-evidence-query-stack.md)

## References

- ADR-0024 (eval-evidence integration).
- `policy/evidence-pack-integrity.md` EPI-07.
