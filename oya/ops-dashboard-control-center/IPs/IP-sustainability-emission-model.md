# IP: Ops dashboard control-center emission model non-claim

No emission-model declaration is added to `oya/ops-dashboard-control-center/manifest.json` for this batch.

The manifest is explicitly `implemented-local-foundation-truth-down`: it records DCOps/domain guardrail metadata and non-claims for live REST/gRPC/SDK/worker runtime, measured SLO/SLI evidence, OpenTofu or provider-live operation, audit-chain writer persistence, autoscaling/rebalance execution, incident-command runtime, and dashboard service crates. It also lacks the ADR-0340 capacity model and pod runtime tier that ADR-0344 requires before coefficients are meaningful.

The correct next step is source-authority work, not coefficient authoring. A future card should decide whether the emission model belongs to a real ops-dashboard runtime, the DCOps domain library, or a separate cloud-capacity surface. Until then, the only safe mapping is a negative one: DCOps evidence prefixes (`evidence/cloud-ops/finops/`, `evidence/cloud-ops/capacity/`, and `audit-chain/cloud-ops/`) are references, not live emission rows.

Non-claims preserved: no OpenCost or FOCUS export, no cloud-billing mutation, no provider SDK action, no Kubernetes runtime, no generated output edit, and no production cost/emission readiness.
