---
ip_id: IP-010
title: "IP-010: IaC — Postgres schema, OpenBao policy, K8s manifests"
microservice: connect
bounded_context: cross-cutting
layers: [iac]
acceptance_status: design-ready
date: 2026-05-20
related_adrs: [ADR-0248, ADR-0253, ADR-0254, ADR-0295, ADR-0296]
companion_docs:
  - microservices/connect/iac/postgres-migration-001.sql
  - microservices/connect/iac/openbao-policy.hcl
  - microservices/connect/iac/helm-values-connect.yaml
  - microservices/connect/iac/network-policy.yaml
  - microservices/connect/iac/spiffe-workload-identity.yaml
  - microservices/connect/iac/external-secret.yaml
  - microservices/connect/iac/ingress-production.yaml
  - microservices/connect/iac/edge-waf.yaml
  - microservices/connect/iac/kustomize-base.yaml
doc_status: published
---

# IP-010: IaC — Postgres schema, OpenBao policy, K8s manifests

## Purpose

Author and validate the complete IaC suite for the connect microservice: Postgres schema migrations, OpenBao policy, Helm values, Cilium NetworkPolicy, SPIFFE workload identities, External Secret bindings, ingress (HTTP/3 + ECH + PQC), edge WAF config, and Kustomize base.

## Acceptance criteria

1. `postgres-migration-001.sql` applies cleanly to a fresh `connect` schema; RLS policies verified by synthetic cross-tenant probe (tries to read another tenant's grant → returns empty).
2. `openbao-policy.hcl` tested: connector-adapter-worker service account can read access tokens; CANNOT read refresh tokens (different path); PagerDuty service key readable with elevated TTL.
3. `helm-values-connect.yaml` deploys all 5 services with correct `runtimeClassName: kata-qemu` on adapter-worker.
4. `network-policy.yaml` passes Cilium connectivity test: adapter-worker can reach `cloud-secrets:8200`; cannot reach `tenancy` namespace directly.
5. `spiffe-workload-identity.yaml` registers all 4 ClusterSPIFFEIDs; kill-switch ConfigMap present.
6. `external-secret.yaml` syncs `connect-tls-production`, `connect-pagerduty-service-key`, `connect-internal-signing-key`.
7. `ingress-production.yaml` serves `Alt-Svc: h3=":443"` on catalog + oauth + webhook routes.
8. `edge-waf.yaml` rate-limit config: catalog browse 300/60s per-IP; emergency-services bypass enabled; PagerDuty never throttled.

## Definition of done

- [ ] `kubectl apply --dry-run=server -f iac/` passes
- [ ] Postgres migration CI lane passes
- [ ] OpenBao policy CI tests pass (hvac + test-vault)
- [ ] `helm template . -f iac/helm-values-connect.yaml | kubeval` passes


## A. Problem
`IP-010: IaC — Postgres schema, OpenBao policy, K8s manifests` is not a generic implementation packet; it closes the `010 iac postgres openbao` gap for `connect` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: ConnectorCatalog, OAuthBrokerService, WebhookReceiverService, ConnectorAdapter, DLQ, provider-BYOK, per-tenant webhook DNS, vendor rate-limit profile.

## B. Approach
Operational readiness is proven through Postgres/OpenBao/Kata/network-policy manifests plus SLO and load-test gates for connector, OAuth, webhook, and DLQ paths. The implementation must keep the µservice boundary intact: contracts remain under `microservices/connect/contracts/openapi/connect-integration.yaml` / `microservices/connect/contracts/proto/connect_integration.proto`, policy decisions remain in `microservices/connect/policy/connector-authorization.cedar`, operational proof remains in `microservices/connect/slos/connector-availability.openslo.yaml`, and the parity claim is checked against `microservices/connect/competitor-parity-matrix.md`.

## C. Deliverables
- `microservices/connect/PRD.md` — verify/update as the authoritative artifact for this IP.
- `microservices/connect/ARCHITECTURE.md` — verify/update as the authoritative artifact for this IP.
- `microservices/connect/contracts/openapi/connect-integration.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/connect/contracts/proto/connect_integration.proto` — verify/update as the authoritative artifact for this IP.
- `microservices/connect/contracts/asyncapi/connect-integration-events.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/connect/policy/connector-authorization.cedar` — verify/update as the authoritative artifact for this IP.
- `microservices/connect/slos/connector-availability.openslo.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/connect/runbooks/connector-cascade-failure.md` — verify/update as the authoritative artifact for this IP.
- `microservices/connect/catalog/oya-connect-connector-catalog-domain.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/connect/competitor-parity-matrix.md` — verify/update as the authoritative artifact for this IP.
- `microservices/connect/iac/network-policy.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/connect/iac/helm-values-connect.yaml` — verify/update as the authoritative artifact for this IP.
- Named code targets declared by this IP and `manifest.json` must be created only when the implementation PR actually adds the crates/types; this scrub does not pretend source files exist.

## D. Implementation Steps
1. Read `microservices/connect/PRD.md` and `microservices/connect/ARCHITECTURE.md` to confirm the bounded context, tenant class, and first-ship milestone for `connect`.
2. Diff the declared contract in `microservices/connect/contracts/openapi/connect-integration.yaml` and `microservices/connect/contracts/proto/connect_integration.proto` against the IP title so every endpoint/message has a matching domain type or explicit backlog gap.
3. Check `microservices/connect/policy/connector-authorization.cedar` plus adjacent Cedar/policy files before adding any mutation, share, webhook, agent, AI, or cross-tenant path.
4. Wire observability to `microservices/connect/slos/connector-availability.openslo.yaml` and the relevant dashboard/runbook; no acceptance claim counts without a metric or sealed evidence path.
5. Update the catalog/capability record such as `microservices/connect/catalog/oya-connect-connector-catalog-domain.yaml` so the service registry can discover the new boundary.
6. Run the IP-specific test/gate commands listed above; if a source crate is absent, record the absent crate as implementation debt rather than faking a green result.

## E. Acceptance
- Local artifact links resolve for `microservices/connect/PRD.md`, `microservices/connect/ARCHITECTURE.md`, `microservices/connect/contracts/openapi/connect-integration.yaml`, `microservices/connect/policy/connector-authorization.cedar`, `microservices/connect/slos/connector-availability.openslo.yaml`, and `microservices/connect/competitor-parity-matrix.md`.
- The implementation exposes no cross-tenant, cross-pack, credential, E2E, or vendor-call path without the policy file cited in this IP.
- At least one targeted unit/contract/gate command verifies the named behavior, and any skipped command is documented with the missing artifact.
- The final PR includes evidence that counterpart parity is improved or explicitly marks the remaining gap.

## F. Evidence
- `microservices/connect/PRD.md`
- `microservices/connect/ARCHITECTURE.md`
- `microservices/connect/contracts/openapi/connect-integration.yaml`
- `microservices/connect/contracts/proto/connect_integration.proto`
- `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`
- `microservices/connect/policy/connector-authorization.cedar`
- `microservices/connect/slos/connector-availability.openslo.yaml`
- `microservices/connect/runbooks/connector-cascade-failure.md`
- `microservices/connect/catalog/oya-connect-connector-catalog-domain.yaml`
- `microservices/connect/competitor-parity-matrix.md`
- `microservices/connect/competitor-parity-matrix.md` — counterpart gap table used for the comparison below.

## G. Counterparts
| Counterpart pressure | Oyatie closure for this IP |
|---|---|
| Zapier, n8n, Workato, Boomi, MuleSoft, Tray.io, Pipedream, AWS EventBridge, Stripe, Salesforce, Slack, GitHub, GitLab, HubSpot, Notion, Linear, Snowflake, and Twilio | Zapier/n8n/Workato define connector breadth; Stripe/Salesforce/Slack/GitHub/GitLab/HubSpot/Notion/Linear/Snowflake/Twilio adapters define early correctness probes; AWS EventBridge defines event-ingest durability pressure. This IP closes the relevant gap by binding `010 iac postgres openbao` to concrete `connect` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
