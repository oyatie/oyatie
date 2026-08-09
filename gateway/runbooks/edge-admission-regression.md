# Edge Admission Regression Runbook

## Trigger

Admission denials or latency deviate from the API gateway design envelope after a route, WAF, JWT, or rate-limit policy change.

## Operator Boundary

The gateway owner rolls back edge admission configuration. Workload authorization and business behavior remain outside this runbook.

## Checks

1. Compare denial events against `oya.api_gateway.request.denied`.
2. Verify the current WAF, JWT, and Cedar coarse-scope policy versions.
3. Reapply the previous route bundle if denial rate exceeds the design threshold.
4. Emit an audit-chain note linking the route bundle, cell, and tenant cohort.
