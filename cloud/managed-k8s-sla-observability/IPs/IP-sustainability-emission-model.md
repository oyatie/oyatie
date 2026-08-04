# IP: Managed-K8s SLA observability emission fixture

Purpose: service-specific ADR-0344 fixture for `cloud/managed-k8s-sla-observability/manifest.json#sustainability_emission_model`. This is local Plan/Spec evidence for the SLA summary path; it does not introduce Prometheus/Kubernetes live collection, OpenCost, FOCUS, regulator export, chargeback, or measured SLO production evidence.

The workload signal is a read of SLA summaries: managed-cluster availability samples, provisioning-latency snapshots, and error-budget state. The source path is `sla-summary-read` plus `managed-cluster-availability.openslo.yaml` and `provisioning-latency.openslo.yaml`. Live observation ingestion stays deferred behind the API port and in-memory adapter.

Capacity model anchor: Tier-3 runc-edge, 0.01 vCPU, 32 MiB RAM, no service-owned storage, and no persistent Valkey/Postgres/outbound connection pool. That makes this lower energy than the control-plane host and lower storage than generic observability.

Power fixture: CPU 0.16 W/vCPU-second, memory 0.0008 W/GiB-second, storage 0.0 W/GiB-hour, network 0.022 W/GiB. The network term covers status-summary fetches from the sibling control-plane-host seam only after that seam is authorized.

Provider/SKU binding is read-only rate-card authority from cloud-billing (`README.md:56-75`; IP-004 §C.4). Baseline: 7 mWh per p50 summary, 0.0028 g CO2 at 400 gCO2/kWh, with ADR-0344's 20 percent tolerance.
