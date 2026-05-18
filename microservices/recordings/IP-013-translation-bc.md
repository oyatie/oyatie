---
doc_class: ImplementationPlan
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-013-translation-bc
status: pending
owner: axis-recordings + axis-translate
acceptance_lanes: [lean-a2]
---

# IP-013: Translation BC — cross-µservice handoff to `translate` µservice

## Intent

Land the translation BC that calls the `translate` µservice via Workflow
events (no direct call per LEAN-A2). Translation request → translate
µservice → translated transcript JSON stored alongside the source.

## Concrete crates

`oya-recordings-translation-{kernel,domain,usecase,api,adapter,worker,sdk,app}`.

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate lean-a2 --microservice recordings   # cross-product through Workflow
```

## Next IP

[`IP-014-strangler-migration-adapter.md`](IP-014-strangler-migration-adapter.md)
