# API Gateway Cost and FinOps Model

Cost drivers are Envoy replicas, WAF rule evaluation, rate-limit storage, and telemetry emission volume. The design budget treats edge admission as shared substrate and attributes incremental cost by tenant request count, route family, and cell.

The FinOps boundary is design-only here. Actual spend proof requires runtime metering and chargeback evidence outside this claim gate.
