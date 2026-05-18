# API Gateway Data Residency

The gateway rejects cross-cell routing at the edge. Tenant residency is checked before workload dispatch using `tenant_id`, `cell_id`, and the regional pack attached to the hostname.

This is a design surface only; runtime residency proof remains owned by deployment and audit evidence gates.
