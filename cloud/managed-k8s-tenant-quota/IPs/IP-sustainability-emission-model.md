# IP: Managed-K8s tenant-quota emission fixture

This Plan/Spec IP supports `cloud/managed-k8s-tenant-quota/manifest.json#sustainability_emission_model`. It models the pure quota-decision path only. No quota-store production adapter, provider SDK, Kubernetes API mutation, billing state change, invoice/chargeback, OpenCost, FOCUS, or live FinOps readiness is claimed.

Signal source: `quota-decision-evaluate` over the Cedar-backed tenant quota policy. A unit is one quota check on the cluster-lifecycle admission path. The fixture cites `cloud/managed-k8s-tenant-quota/cedar/quota-rbac.cedar` because deny/permit scope, tenant matching, and platform-operator authority determine the request branch.

Capacity source: Tier-3, 0.01 vCPU, 32 MiB RAM, zero storage, and zero connection-pool footprint in the current manifest. The path is O(1) pure Rust/in-memory for local foundation, so the fixture is the smallest in this batch.

Coefficients: CPU 0.12 W/vCPU-second, memory 0.0006 W/GiB-second, storage 0.0 W/GiB-hour, network 0.012 W/GiB. The network coefficient is non-zero only to reserve accounting space for future API/app boundary hops; it is not a live external request.

Price binding: cloud-billing per-usage/rate-card authority (`README.md:56-75`; IP-004 §C.4), referenced without mutation. Baseline fixture: 4 mWh per p50 quota decision, 0.0016 g CO2 at 400 gCO2/kWh, ±20 percent.
