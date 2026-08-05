# IP: Managed-K8s cluster-lifecycle emission fixture

This IP backs `cloud/managed-k8s-cluster-lifecycle/manifest.json#sustainability_emission_model` for advisory sustainability planning. It covers admission/orchestration work only; it does not provision a cluster, call a provider SDK, run OpenTofu, write Kubernetes objects, emit live billing, or claim production FinOps readiness.

Signal map: the unit is a `cluster-lifecycle-create` admission request. The request validates tenant id, cluster name, tier, and requested resources, then calls `QuotaDecisionPort` before the downstream `ControlPlaneProvisioning` port. Because `audit_chain.enabled=false` in this manifest today, the source row is a planned ADR-0263 admission row, not a live seal event.

Capacity basis: Tier-3, 0.01 vCPU, 32 MiB RAM, zero storage, and zero persistent/shared connections. The service is intentionally a narrow composition over ports, so the deterministic fixture is small and request-bound.

Coefficient choices: CPU 0.18 W/vCPU-second, memory 0.0007 W/GiB-second, storage 0.0 W/GiB-hour, network 0.028 W/GiB. The non-zero network coefficient represents the in-process/port boundary accounting for quota and provisioning requests, not a provider-live call.

Pricing reference remains cloud-billing's provider-SKU/per-usage authority (`README.md:56-75` and IP-004 §C.4). Deterministic fixture: 9 mWh per p50 admission, 0.0036 g CO2 at 400 gCO2/kWh, ±20 percent until measured evidence replaces it.
