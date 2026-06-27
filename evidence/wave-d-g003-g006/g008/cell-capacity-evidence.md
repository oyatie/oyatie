# G008-F cell/capacity foundation evidence

Task 17 asked whether the cell/capacity substrate is present and what the smallest truthful slice is.

## Verified findings
- `cloud/cell-lifecycle/` exists and is documented as the logical Cell aggregate state machine.
- `cloud/cell-rebalancer/` exists and is documented as tenant migration across cells.
- `cloud/cloud-capacity/` does **not** exist as a top-level directory in this checkout.
- `specs/masterplan.json` names `cloud-capacity` as part of the cloud substrate list.
- The owned cell manifests already point at the shared capacity guardrail surface via `crates/oya-cloud-capacity-domain/src/lib.rs` and `crates/oya-cloud-capacity-domain/tests/cloud_ops_foundation.rs`.

## Conclusion
No owned-path source change was needed for this lane. The truthful completion artifact is evidence that the cell/capacity substrate is represented by the existing cell-lifecycle and cell-rebalancer foundations, while a separate `cloud/cloud-capacity/` tree is absent in this checkout.

## Notes
- This is an evidence-only slice.
- No generated files were edited.
- No files outside the owned cell/capacity evidence slot were touched.
