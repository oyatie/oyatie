---
doc_class: Evidence
status: contract-snapshot
source_task: t_3076cb13
generated_at_utc: 2026-07-01T09:42:00Z
claim_ceiling: accepted-authority/spec and migration-gate plan only; no Kubernetes manifest apply, live cluster mutation, runtime LoadBalancer migration, production-readiness, rollout, SLO, or rollback exercise claim
---

# Mail non-HTTP protocol edge authority and LoadBalancer gate plan

## Disposition

Accepted authority now distinguishes mail non-HTTP protocol edge handling from ADR-0157's HTTP/gRPC api-gateway route contract without weakening NETWORK-001:

- `docs/decisions/ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation.md` adds a `Non-HTTP protocol edge extension` for SMTP MX, SMTP submission, IMAP/IMAPS, and ManageSieve.
- `docs/decisions/ADR-0157-api-gateway-tier.md` keeps the HTTP/gRPC api-gateway route contract and updates the gate taxonomy so unclassified tenant-facing `LoadBalancer` resources fail closed.
- `specs/microservices/mail.json` records the mail product edge target, authorized protocol/port set, owner, required `ClusterIP` workload posture, and compensating controls.
- `oya/api-gateway/contracts/api-gateway.openapi.yaml` carries the mechanical LoadBalancer inventory taxonomy token set for the NETWORK-001 contract surface.

The chosen model is **edge-owned non-HTTP protocol routes**, not a mail workload-owned LoadBalancer exception. Current `oya/mail/iac/helm/templates/service.yaml` direct `Service.type=LoadBalancer` resources remain migration debt until a Build card moves public listener ownership to the edge-owned protocol surface or a later accepted ADR explicitly deprecates or retires those protocols.

## Gate taxonomy

The inventory/gate token classes are:

1. `api_gateway_http_grpc_load_balancer` — ADR-0157 api-gateway / Envoy Gateway public HTTP/gRPC edge.
2. `authorized_non_http_protocol_edge` — ADR-0182 edge-owned SMTP/submission/IMAP/Sieve protocol listeners with accepted authority refs and compensating controls.
3. `unclassified_tenant_facing_load_balancer` — any other tenant-facing `LoadBalancer`; the gate/admission path must fail closed.

The gate must also assert `direct_mail_workload_load_balancer_allowed: false` and `required_workload_service_type: ClusterIP` for mail workloads behind the authorized protocol edge.

## Required compensating controls

The Build/migration card must preserve these controls before runtime promotion:

- Edge-owned Gateway API `TCPRoute`/`TLSRoute` or a dedicated `mail-protocol-edge` adapter owned by api-gateway/edge platform.
- Mail workload Services remain `ClusterIP`/mesh-internal.
- mTLS/SPIFFE from edge listener to mail workloads.
- CiliumNetworkPolicy allowlists for edge-to-mail traffic only.
- L4 DDoS, connection-rate limiting, and per-tenant/IP rate limits.
- STARTTLS/MTA-STS/TLS-RPT posture where protocol-applicable.
- SASL/OIDC or equivalent tenant identity binding for authenticated submission, IMAP, and ManageSieve flows.
- open-relay refusal.
- DMARC/SPF/DKIM/ARC/Rspamd processing for mail ingress.
- OpenTelemetry trace and audit correlation.
- Rendered manifest evidence, Kubernetes schema/dry-run validation, protocol smoke checks, rollout plan, and rollback notes.

## Exact next Build scope

Suggested next-card title: `Build: migrate mail protocol LoadBalancers to edge-owned non-HTTP protocol routes and gate inventory`.

Files likely touched:

- `oya/mail/iac/helm/templates/service.yaml` — convert `inbound-smtp-mx`, `outbound-submission`, and `imap` public Services from `LoadBalancer` ownership to internal `ClusterIP` workload targets or remove them once edge routes exist.
- `oya/mail/iac/helm/templates/networkpolicy.yaml` — replace `0.0.0.0/0` mail protocol ingress with edge-to-mail CiliumNetworkPolicy allowlists.
- `oya/api-gateway/iac/**` or a new edge-owned `mail-protocol-edge` chart surface — add Gateway API `TCPRoute`/`TLSRoute` or adapter manifests for TCP 25/465/587/143/993/4190.
- `oya/api-gateway/contracts/api-gateway.openapi.yaml` or a companion gate fixture — preserve the taxonomy tokens and route classification.
- Cloud-ci/Rust gate files for the eventual `LoadBalancer` inventory check once the gate owner lane is selected.

Required verification commands for that Build card:

```bash
ruby -e 'require "yaml"; %w[oya/api-gateway/contracts/api-gateway.openapi.yaml].each { |p| YAML.load_file(p); puts "YAML OK #{p}" }'
python3 -m json.tool specs/microservices/mail.json >/dev/null
python3 <load-balancer-inventory-token-check>.py
git diff --check -- docs/decisions/ADR-0157-api-gateway-tier.md docs/decisions/ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation.md specs/microservices/mail.json oya/api-gateway/contracts/api-gateway.openapi.yaml oya/mail/iac/helm/templates/service.yaml oya/mail/iac/helm/templates/networkpolicy.yaml
helm template <mail chart args> # once values are known
kubectl apply --dry-run=server -f <rendered manifests> # against an appropriate validation cluster/context, not during this spec-only card
```

## Non-claims

- No Kubernetes manifests were applied.
- No live cluster, cloud LoadBalancer, DNS, or certificate state was mutated.
- No `*.generated.json` file was hand-edited.
- This card does not claim runtime protocol reachability, deliverability readiness, production readiness, SLO conformance, rollout completion, or rollback exercise.
- The current direct mail LoadBalancers are not accepted as workload-owned exceptions; they are now classified migration debt under accepted edge-owned protocol authority.
