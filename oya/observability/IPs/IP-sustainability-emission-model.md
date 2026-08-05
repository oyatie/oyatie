# IP: Observability sustainability emission model planning fixture

This advisory IP supplies the service-specific calibration notes for `oya/observability/manifest.json#sustainability_emission_model`. It is a planning fixture only: no electricityMaps client, Valkey carbon cache, OpenCost ingestion, FOCUS export, billing mutation, or production FinOps readiness is asserted here.

Observability is a Tier-1 shared telemetry substrate. The workload signal is `per_query` because the model follows eligibility queries, OpenSLO validation, and SLO evaluation reads rather than tenant UI sessions. The source rows are the manifest audit events `oya.observability.eligibility-query`, `oya.observability.openslo-validate`, and `oya.observability.slo-evaluate`, with SLO context from the log, metric, trace, and query-latency OpenSLO files.

Capacity anchor: 0.26 vCPU, 768 MiB RAM, 20 GB storage, 3 Valkey, 3 Postgres, and 8 outbound HTTP connections per tenant query stream. The 20 GB storage footprint makes this model storage-heavier than the application shell.

Coefficients are deliberately not the ADR illustrative tuple: 0.52 CPU W/vCPU-second, 0.0022 memory W/GiB-second, 0.0009 storage W/GiB-hour, and 0.07 network W/GiB. The higher storage/network weights reflect query fanout over metrics/logs/traces and ClickHouse/Iceberg-backed retention pressure.

Pricing anchor is the cloud-billing usage/rate-card authority (`README.md:56-75`, IP-004 §C.4), cited as provider-SKU binding only. The deterministic fixture expects 64 mWh per p50 query bundle, equal to 0.0256 g CO2 at a 400 gCO2/kWh grid factor, with the ADR-0344 default 20 percent tolerance.
