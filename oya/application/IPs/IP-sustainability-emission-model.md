# IP: Application sustainability emission model planning fixture

Status: advisory Plan/Spec fixture for ADR-0344. This file backs `oya/application/manifest.json#sustainability_emission_model`; it is not runtime carbon accounting, billing, OpenCost, FOCUS, invoice, chargeback, or regulator-export evidence.

The application shell's natural workload signal is the interactive request/session path, not a batch job. The manifest binds the emission model to `oya.application.module-load`, `oya.application.session-emit`, and `oya.application.shell-render` audit events because those rows represent module discovery, shell rendering, and tenant-context hydration. Capacity input comes from `capacity_model`: 0.28 vCPU, 512 MiB RAM, 6 GB storage, and 10 outbound HTTP connections per active tenant user at Tier-2 placement.

Power-model rationale: CPU coefficient 0.38 W/vCPU-second reflects UI composition and auth-gateway checks below the ADR example; memory coefficient 0.0019 W/GiB-second is anchored to the 512 MiB shell/session footprint; storage coefficient 0.00042 W/GiB-hour covers tenant shell/cache state; network coefficient 0.045 W/GiB covers module and policy-disclosure payload fanout.

Price binding: `cloud/cloud-billing/README.md:56-75` and `cloud/cloud-billing/implementation-plans/IP-004-composable-billing-components.md:91-107` define usage events, rate-card refs, and per-usage axes. The application block references that provider-SKU binding only; it does not mutate billing state.

Deterministic fixture: one shell-route request at p50 uses 28 mWh, which is 0.0112 g CO2 at 400 gCO2/kWh. Tolerance remains ADR-0344's default 20 percent until measured per-service calibration evidence lands.
