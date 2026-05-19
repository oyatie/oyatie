# Ops Dashboard / Control Center Cost and FinOps Boundary

## Cost center

- `cc-ops-internal` per `registry/finops/cost-tag-vocabulary.yaml`.
- Workload class: `app`.
- Pack tag: `generic` by default; regional evidence exports add pack dimensions without changing the control-plane cost center.

## Cost drivers

- SSE or gRPC-stream health subscriptions.
- Evidence-pack export assembly and object-store writes.
- Incident and deployment command audit-chain seals.
- Dashboard read amplification during incidents.

## Guardrails

- Per-tenant and per-operator rate limits are mandatory.
- Evidence export requests require scoped windows.
- Long-running exports should be asynchronous tickets, not synchronous downloads.

## Acceptance criteria

- Every command path has a metric usable for attribution.
- Evidence export usage is tagged by tenant, pack, and requesting operator role.
- Cost spikes from incidents are visible without charging tenant-facing product workloads directly.
