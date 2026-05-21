# Capability Catalog: cell-lifecycle

| Capability | Tier | Risk | Description | Evidence |
| --- | --- | --- | --- | --- |
| register-cell | T1 | limited | Register logical cell after cloud-iac readiness. | cloud_iac_receipt_id + audit_chain_event_id |
| activate-cell | T1 | limited | Activate registered cell after telemetry bootstrap. | observability_window_id + evidence_pack_id |
| promote-cell | T0 | high | Promote through ADR-0204 tiers with ADR-0266 gates. | gate_snapshot_sha256 + Cedar permit |
| drain-cell | T0 | high | Enter Draining and trigger cell-rebalancer. | drain evidence pack + rebalancer receipt |
| decommission-cell | T0 | high | Finalize Draining cell after resident_count zero. | tenancy snapshot + audit-chain seal |
| list-cells | T2 | minimal | List filtered lifecycle summaries for operators. | Cedar ReadLifecycle decision |
| read-lifecycle-history | T1 | limited | Return immutable history references. | audit-chain event references |
