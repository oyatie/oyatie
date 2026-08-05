---
doc_class: Evidence
status: contract-snapshot
source_task: t_e9393c8e
generated_at_utc: 2026-07-01T09:20:43Z
claim_ceiling: classification/spec-planning evidence only; no Kubernetes manifest apply, live cluster mutation, direct LoadBalancer migration, accepted mail protocol-edge ADR, production-readiness, rollout, SLO, or rollback claim
---

# NETWORK-FOLLOWUP mail non-HTTP LoadBalancer ingress classification

## Classification

The direct `oya/mail` SMTP/submission/IMAP/Sieve `Service.type=LoadBalancer` resources are classified as an unresolved non-HTTP protocol-edge architecture gap, not as an accepted exception.

Current accepted gateway/mesh authority is strong enough to preserve NETWORK-001's gateway-only HTTP/gRPC route boundary, but it is not strong enough to bless mail-owned tenant-facing LoadBalancers. Mail has accepted product need for SMTP/JMAP/IMAP/REST edge surfaces, and direct SMTP/IMAP ports may require non-HTTP edge handling, but the repo does not currently contain accepted ADR authority that classifies these specific non-HTTP protocols as allowed non-gateway LoadBalancer exceptions with compensating controls.

Therefore:

- `POST /edge/admission` and other external HTTP/gRPC surfaces remain under the canonical `api-gateway` / Envoy Gateway north-south boundary.
- The mail Helm LoadBalancers remain inventory-only architecture drift until a dedicated mail protocol-edge ADR/spec/gate either accepts an explicit non-HTTP exception or migrates their public listeners under the canonical edge/gateway ownership model.
- No runtime resource was changed by this card.

## Source authority inspected

Accepted north-south / east-west authority:

- `docs/decisions/ADR-0157-api-gateway-tier.md:20` scopes the accepted api-gateway decision to every external tenant-facing, partner-facing, public-internet HTTP/gRPC call.
- `docs/decisions/ADR-0157-api-gateway-tier.md:42-57` makes `api-gateway` the canonical north-south entry tier for external HTTP/gRPC requests and places TLS, AuthN/AuthZ, WAF, DDoS/rate-limit, trace-context injection, schema enforcement, and per-cell rejection there before workload routing.
- `docs/decisions/ADR-0157-api-gateway-tier.md:132-137` requires a gate proving all external endpoints resolve under `api-gateway` and no other microservice declares a tenant-facing `LoadBalancer` Service.
- `docs/decisions/ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation.md:40-60` assigns north-south public ingress to Envoy Gateway and east-west traffic to Cilium + Istio Ambient; public traffic must traverse Envoy Gateway first and the mesh must not terminate public TLS.
- `docs/decisions/ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation.md:127-132` keeps north-south Gateway resources under the api-gateway surface while Cedar `ext_authz` begins at the mesh waypoint inside the cluster.
- `docs/decisions/ADR-0148-service-mesh-cilium-ambient-layered.md:49-58` keeps Cilium at L3/L4 and Istio Ambient at L7 with zero overlap; Cilium does not terminate application TLS and Istio does not own CNI.
- `docs/decisions/ADR-0148-service-mesh-cilium-ambient-layered.md:197-204` requires mesh/network policy, ambient labels, and observability plumbing for microservices, but does not authorize direct public LoadBalancers owned by workload microservices.
- `docs/decisions/ADR-0145-inter-microservice-communication-reform.md:80-89` permits direct sibling service egress only for east-west gRPC under mTLS, Cedar authorization, audit, and tracing; it is not north-south mail ingress authority.

Mail product / protocol context:

- `specs/microservices/mail.json:5-14` marks the Mail PRD as an Accepted machine-readable spec.
- `specs/microservices/mail.json:80-82` names an external sender/recipient persona using an `SMTP/JMAP/IMAP/REST edge`.
- `specs/microservices/mail.json:127-131` requires inbound SMTP DMARC/SPF/DKIM stamping, quarantine/reject policy action, and logging.
- `specs/microservices/mail.json:171-189` places SMTP/JMAP/IMAP runtime in stable scope and load-tested multi-region mail cells in GA scope.
- `oya/mail/README.md:46-51` says web/mobile clients use JMAP over HTTP/3 while legacy clients use IMAP/POP3 over TLS 1.3 and Sieve remains a server-side filter protocol.
- `oya/mail/security/threat-model.md:48-60` names inbound SMTP, SMTP submission, and IMAP/JMAP/REST read as external interfaces, but the threat model is Proposed and does not override accepted ADRs.
- `docs/decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md:91-122` is Proposed and explicitly keeps mail in closed-beta until deliverability gates land; it is contextual evidence, not accepted ingress authority.
- `docs/decisions/ADR-0201-email-transactional-comms-adapter-substrate.md:37-55` is Accepted for transactional email adapter providers and an SMTP fallback, but it scopes shared outbound email-comms adapters rather than the full Mail mailbox ingress/IMAP protocol edge.

Current direct LoadBalancer inventory:

- `oya/mail/iac/helm/templates/service.yaml:25-41` declares `inbound-smtp-mx` as `Service.type=LoadBalancer` on TCP ports 25 and 465.
- `oya/mail/iac/helm/templates/service.yaml:42-57` declares `outbound-submission` as `Service.type=LoadBalancer` on TCP port 587.
- `oya/mail/iac/helm/templates/service.yaml:58-75` declares `imap` as `Service.type=LoadBalancer` on TCP ports 143, 993, and 4190.
- `oya/mail/iac/helm/templates/networkpolicy.yaml:13-26` currently allows `0.0.0.0/0` ingress to the SMTP/submission/IMAP/Sieve port set, and also permits TCP/443 in the `imap-frontend` stanza for the JMAP/REST path that must remain under the gateway boundary when promoted.

## Surface disposition table

| Surface | Ports | Public principal | Current repo state | Classification | Required disposition before runtime promotion |
|---|---:|---|---|---|---|
| Inbound SMTP MX | 25, 465/TCP | external sender / recipient MTA | mail-owned `LoadBalancer` | unaccepted non-HTTP protocol-edge direct LoadBalancer | Define accepted protocol-edge authority or move the listener into an edge-owned Gateway API TCP/TLS route / mail-protocol-edge tier with mail workloads remaining `ClusterIP` behind mTLS/SPIFFE, L4 DDoS/rate limit, STARTTLS/MTA-STS/TLS-RPT plan, DMARC/SPF/DKIM/ARC/Rspamd controls, trace/audit emission, and CNP allowlists. |
| SMTP submission | 587/TCP | authenticated tenant user/service | mail-owned `LoadBalancer` | unaccepted non-HTTP protocol-edge direct LoadBalancer | Prefer authenticated submission through an edge-owned protocol listener with SASL/OIDC binding, per-tenant/IP rate limits, open-relay refusal, DKIM key-custody audit, and workload `ClusterIP` only. If an exception is chosen, it needs accepted ADR authority and a gate allowlist. |
| IMAP/IMAPS | 143, 993/TCP | legacy mailbox client | mail-owned `LoadBalancer` | unaccepted non-HTTP protocol-edge direct LoadBalancer | Keep JMAP/REST under `api-gateway`; for legacy IMAP, define an explicit protocol-edge route or deprecation/migration path, with TLS 1.3, auth step-up/app-password controls, mailbox context isolation, L4 rate limits, audit, and workload `ClusterIP` only. |
| ManageSieve | 4190/TCP | authenticated mailbox filter client/admin | mail-owned `LoadBalancer` | unaccepted non-HTTP protocol-edge direct LoadBalancer | Do not expose directly until either retired behind an authenticated admin/JMAP flow or accepted as a protocol-edge route with the same edge ownership and compensating controls as IMAP. |

## Narrow migration/spec plan

1. Author or amend accepted authority before changing runtime manifests:
   - Option A: amend ADR-0157/ADR-0182 with a bounded `non_http_protocol_edge` section that keeps ownership under `api-gateway` / edge platform but allows SMTP/IMAP/Sieve-specific L4/TLS listeners.
   - Option B: author a dedicated Mail Protocol Edge ADR, accepted by council-architecture, axis-network, axis-mail, ops-security, ops-sre-reliability, and ops-deliverability, that declares the exception, controls, owner, and gate shape.
2. Introduce a gate taxonomy that distinguishes:
   - `api_gateway_http_grpc_load_balancer` allowlisted gateway/Envoy resources,
   - `authorized_non_http_protocol_edge` resources with accepted ADR refs and compensating controls,
   - all other tenant-facing microservice `LoadBalancer` resources, which fail closed.
3. Move public listener ownership out of `oya/mail` workload Helm templates:
   - keep mail worker/application Services as `ClusterIP`,
   - represent SMTP/submission/IMAP/Sieve entrypoints as edge-owned Gateway API `TCPRoute`/`TLSRoute` or a dedicated `mail-protocol-edge` adapter owned by the gateway/edge platform,
   - forward to mail workloads over mesh-internal mTLS/SPIFFE with CiliumNetworkPolicy allowlists and OTel/audit correlation.
4. Add runtime evidence in a later card:
   - rendered Helm/Gateway API manifests,
   - Kubernetes dry-run/schema validation,
   - LoadBalancer inventory gate output,
   - protocol smoke checks for SMTP banner/STARTTLS/submission auth/IMAPS/ManageSieve as applicable,
   - observability/audit evidence and rollback notes.

## Non-claims

- This card does not accept an exception for the existing direct mail LoadBalancers.
- This card does not migrate Helm resources, apply Kubernetes manifests, or mutate any live cluster.
- This card does not change the NETWORK-001 OpenAPI gateway/mesh route contract.
- This card does not claim SMTP/IMAP/Sieve runtime readiness, deliverability readiness, production readiness, SLO conformance, rollout completion, or rollback exercise.

## Runtime follow-up

Queue a follow-up implementation lane to turn this classification into accepted authority plus a mechanical gate and/or migration patch. The follow-up should own the protocol-edge ADR/spec update, edge-owned route manifests, fail-closed LoadBalancer inventory classification, and live/dry-run protocol evidence. Until that lands, any repo-wide claim that all tenant-facing ingress is gateway/edge-owned must exclude these mail services or fail closed.
