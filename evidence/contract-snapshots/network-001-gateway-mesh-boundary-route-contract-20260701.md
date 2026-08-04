---
doc_class: Evidence
status: contract-snapshot
source_task: t_9e4e1495
generated_at_utc: 2026-07-01T09:12:16Z
claim_ceiling: spec/contract encoding only; no runtime gateway, mesh, collector, LoadBalancer migration, live trace/audit emission, or production-readiness claim
---

# NETWORK-001 gateway/mesh boundary route contract snapshot

This snapshot records the bounded authority and verification basis for the one-route contract encoded in `oya/api-gateway/contracts/api-gateway.openapi.yaml` for `POST /edge/admission` (`route_id=edge.admission.v1`). The repo mutation intentionally stays at the gateway/mesh contract layer and does not apply Kubernetes resources, alter cloud provider load balancers, migrate mail/SMTP ingress, wire runtime Envoy/Istio/Cilium/Cedar paths, or claim live observability.

## Accepted authority used for this slice

- `docs/decisions/ADR-0145-inter-microservice-communication-reform.md:36-46` requires decentralized audit-chain emission at the caller and OpenTelemetry trace propagation for inter-microservice calls.
- `docs/decisions/ADR-0145-inter-microservice-communication-reform.md:80-89` permits direct sibling egress only under mTLS authentication, Cedar policy authorization, and audit/tracing invariants.
- `docs/decisions/ADR-0148-service-mesh-cilium-ambient-layered.md:49-57` defines the zero-overlap layered mesh: Cilium owns L3/L4, Istio Ambient owns SPIFFE mTLS/L7 policy/waypoint `ext_authz`.
- `docs/decisions/ADR-0148-service-mesh-cilium-ambient-layered.md:114-123` binds waypoint `ext_authz` to the Cedar PDP and keeps CNP plus AuthorizationPolicy generated from one Cedar source of truth.
- `docs/decisions/ADR-0148-service-mesh-cilium-ambient-layered.md:197-204` requires CiliumNetworkPolicy, tenant-scope Cedar, waypoint resources for enrolled services, ambient labels, ClusterMesh, and OTel forwarding.
- `docs/decisions/ADR-0157-api-gateway-tier.md:42-57` establishes `api-gateway` as the north-south entry tier and requires TLS/AuthN/AuthZ/WAF/rate-limit/schema enforcement before workload routing.
- `docs/decisions/ADR-0157-api-gateway-tier.md:66-79` says api-gateway owns edge concerns and must not contain domain logic or per-resource workload authorization.
- `docs/decisions/ADR-0157-api-gateway-tier.md:132-137` calls for a gate that every external endpoint routes under api-gateway and no other microservice declares a tenant-facing LoadBalancer.
- `docs/decisions/ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation.md:40-60` defines Envoy Gateway for north-south and Cilium+Istio Ambient for east-west with zero overlap.
- `docs/decisions/ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation.md:127-132` places north-south HTTPRoute resources under api-gateway and keeps Cedar `ext_authz` inside the mesh waypoint.
- `docs/standards/observability.md:69-78` requires W3C `traceparent`/`tracestate` propagation on HTTP, gRPC, and message-queue boundaries.
- `docs/standards/logging-tracing.md:24-33` names mandatory trace/span/service/tenant/cell/data-class span fields.

Context-only source:

- `docs/decisions/ADR-0044-service-mesh-istio-ambient-and-envoy-gateway.md:1-9` is Proposed. Its mTLS/SPIFFE/Envoy/Ambient shape is consistent with the accepted ADRs above, but this slice does not rely on ADR-0044 as binding authority.

## Route contract encoded

The OpenAPI operation `POST /edge/admission` now carries `x-oyatie-gateway-mesh-boundary` with these contract fields:

- `classification`: public north-south ingress owned by `api-gateway`, with east-west mesh handoff and no gateway domain logic.
- `load_balancer_boundary`: tenant-facing LoadBalancer allowlist limited to the api-gateway/Envoy Gateway public edge, while workload upstreams for this route must remain `ClusterIP`/mesh-internal and must not expose direct `LoadBalancer`, `NodePort`, or public ingress.
- `mesh_boundary`: Cilium L4 identity, Istio Ambient ztunnel, mTLS, gateway and upstream SPIFFE identities, Cedar `ext_authz` before workload handoff, and fail-closed `failure_mode_allow=false`.
- `route_schema`: `#/components/schemas/EdgeAdmissionRequest` plus required gateway headers for trace, tenant, cell, request, and audit correlation.
- `trace_audit_requirements`: required `traceparent`, preserved `tracestate`, OTel HTTP/service/tenant/cell fields, and `api_gateway.edge_admission.decision` audit correlation fields.

## LoadBalancer inventory note

The slice encodes the required negative boundary but does not remediate all existing inventory:

- Allowed gateway edge inventory for this route:
  - `oya/api-gateway/iac/k8s-deployment.yaml:86-103` exposes `Service/api-gateway-envoy` as the public gateway LoadBalancer.
  - `cloud/cloud-k8s/iac/helm/envoy-gateway/values.yaml:14-20` configures the `envoy-gateway` Helm release gateway service (`gateway.name: istio-ingressgateway`) as a LoadBalancer.
  - `oya/api-gateway/iac/k8s/helm/values.yaml:26-28` keeps the api-gateway application Helm service itself as `ClusterIP`.
- Non-gateway direct LoadBalancer inventory found during the task:
  - `oya/mail/iac/helm/templates/service.yaml:25-75` declares SMTP/submission/IMAP LoadBalancer services. This is outside the gateway/mesh contract blast radius and should be reconciled by a separate network/mail ingress classification or migration card before any repo-wide "no tenant-facing LoadBalancer elsewhere" runtime claim.

## Non-claims and required follow-up before runtime promotion

- No Kubernetes resource was applied, generated, or validated against a live cluster.
- No direct mail/SMTP/IMAP LoadBalancer was changed by this slice.
- No runtime Envoy Gateway, Istio Ambient, Cilium, SPIRE/SPIFFE, Cedar PDP, or OTel collector evidence is claimed.
- No production readiness, hyperscaler maturity, SLO, rollout, or rollback evidence is claimed.
- Any future runtime/promotion lane must add a gate that proves external HTTP/gRPC routes traverse api-gateway, workload upstreams are cluster-internal, direct non-gateway LoadBalancers are classified or migrated, and trace/audit/mTLS/SPIFFE/Cedar evidence is emitted from a real request path.

## Verification expectations

- Parse `oya/api-gateway/contracts/api-gateway.openapi.yaml` as YAML.
- Assert the one operation contains `x-oyatie-gateway-mesh-boundary` with accepted ADRs, contextual ADR-0044, public/internal classification, LoadBalancer boundary, mTLS/SPIFFE, Cedar `ext_authz`, route schema refs, and trace/audit fields.
- Assert no `*.generated.json` file is touched by this slice.
- Assert direct LoadBalancer inventory is reported as evidence/non-claim, not silently hidden.
