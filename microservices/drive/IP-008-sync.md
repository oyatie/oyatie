---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-drive-foundation
impl_plan_id: IP-008-sync
status: pending
execution_unit: ChangeSet
owner: axis-drive
acceptance_lanes: [cargo-build, cargo-nextest, oya-check-cdc-parameters-pinned]
---

# IP-008: sync BC — FastCDC + LBFS delta-sync + deterministic conflict tie-break

## Intent

Stand up `oya-drive-sync-*` BC per ADR-DRIVE-0002. FastCDC implementation + chunk-manifest exchange + delta-set computation + deterministic conflict tie-break.

## Crates

`oya-drive-sync-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,sdk,app}` (10 crates).

## Acceptance Gates

```bash
cargo nextest run -p oya-drive-sync-domain -- fastcdc_parameters_pinned
cargo nextest run -p oya-drive-sync-domain -- delta_minimum_bytes
cargo nextest run -p oya-drive-sync-domain -- tie_break_determinism
cargo nextest run -p oya-drive-sync-domain -- fastcdc_adversarial
```

## References

- ADR-DRIVE-0002.
- PRD-drive §FR-06; AC-04.
