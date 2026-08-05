# IP: Cloud Intelligence sustainability emission model planning fixture

Scope: `cloud/cloud-intelligence/manifest.json#sustainability_emission_model` for the OAuth subscription-pool gateway. The values below are deterministic design fixtures, not measured gateway telemetry and not a provider invoice, FOCUS export, OpenCost row, chargeback, or live cost-readiness claim.

Workload path: every unit is a proxied provider request through the kernel seat-pool state machine and axum reverse proxy. The manifest ties the model to `llm_invocation_audit`, `key_pool_refresh`, `key_blacklisted`, `provider_breaker_open`, and `budget_exceeded` rows because those events capture tenant-scoped provider dispatch, pool health, and budget failure modes.

Capacity source: Tier-2 runc, 0.1 vCPU, 128 MiB RAM, 0 GB persistent storage, and 4 outbound HTTP connections per active tenant. The gateway is intentionally stateless; persistent storage coefficient is therefore zero while network coefficient is the dominant term.

Fixture coefficients: CPU 0.44 W/vCPU-second, memory 0.0016 W/GiB-second, storage 0.0 W/GiB-hour, network 0.095 W/GiB. The network value is higher than application/observability because SSE/non-stream provider payloads dominate the control-plane cost envelope.

Provider/SKU binding: cloud-billing's usage-event and per-usage axes (`cloud/cloud-billing/README.md:56-75`; IP-004 §C.4). The model only points at rate-card authority and never changes pooled-key budgets or billing state.

Baseline check: a p50 provider-proxy request fixture is 42 mWh, which converts to 0.0168 g CO2 at 400 gCO2/kWh. Future measured calibration must replace this fixture before any production emission claim.
