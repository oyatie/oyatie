# IP: Cloud IaC sustainability emission model deferral

No `sustainability_emission_model` block is declared for `cloud/cloud-iac/manifest.json` in this lane.

Reason: the inventory gate classifies cloud-iac as `current_manifest_conditional_not_claimed_runtime`, and the manifest's capacity model uses `scaling_dimension: not_claimed_runtime`. Its audit chain is disabled, and the manifest explicitly excludes live OpenTofu execution, provider plan/apply, Argo CD API integration, measured capacity, autoscaling, SLOs, and audit-chain persistence for the current local-foundation slice.

What would unblock declaration: a future service-authority card must first promote an implemented runtime path with a valid ADR-0340 capacity model, a pod runtime tier, and an audit-row source for module registry/API or GitOps reconciliation requests. Only then can ADR-0344 coefficients bind to provider-SKU pricing without pretending that metadata-only IaC records are live cloud operations.

Non-claims preserved here: no provider-live call, no OpenTofu plan/apply, no generated JSON hand edit, no cloud-billing mutation, no OpenCost/FOCUS export, and no regulator evidence pack.
