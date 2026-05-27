# Managed Kubernetes Cluster Lifecycle PRD

## Purpose
Gateway-internal cluster create admission under `microservices/managed/`: bind the request to a trusted gateway-injected tenant principal header, validate the request, check `QuotaDecisionPort`, then call `ControlPlaneProvisioning`. The app is not directly tenant-facing until shared auth middleware lands.

## Scope
- Default hosted tier, optional dedicated tier.
- Require upstream gateway/header-stripping before exposure; direct tenant authentication is not claimed in this packet.
- Fail closed on malformed requests, quota denial, missing quota, or quota failure.
- Preserve tenant-quota and control-plane-host as independent bounded contexts consumed through API ports only.
- Current binary fails closed unless explicitly started in in-memory dev/test mode; production adapters are not claimed in this packet.
- Quota reservation/accounting after create is a follow-up seam; this packet only consumes the settled quota-decision port required for admission.

## Out of scope
SLA observability, live CAPI reconciliation beyond the existing control-plane-host port, and worker-node reconciliation.
