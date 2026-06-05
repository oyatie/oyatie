---
doc_class: ImplementationPlan
ip_id: IP-024-threat-model-control-map
microservice: marketing-automation
bounded_contexts: [forms, webhooks, journeys, segments, attribution, marketplace-audience-license, credential-sidecar]
related_adrs: [ADR-0244, ADR-0253-amendment, ADR-0263, ADR-0297, ADR-0321, ADR-0328]
status: proposed
date: 2026-05-21
owner: axis-marketing-automation + ops-security
tenant_class_aware: true
---

# IP-024: Threat Model Control Map

## A. Problem

Marketing Automation combines public ingress, outbound provider calls, audience data, and cross-service handoffs. The stamped IP did not name threats or controls. The real gap is a control map connecting `threat-model.md`, Cedar policies, WAF rules, OpenBao secret bindings, transport settings, and runbooks to concrete attacker paths.

## B. Approach

Use `threat-model.md` as the source of threat scenarios and map them to implemented or planned controls in `policy/`, `policies/`, `iac/edge-waf.yaml`, `iac/ech-config.yaml`, `iac/local-openbao-policy.hcl`, and incident runbooks. Each control must be testable or explicitly marked as a follow-up.

## C. Deliverables

| Artifact | Change |
|---|---|
| `threat-model.md` | Add attacker paths for form spam, webhook replay, segment poisoning, attribution export abuse, credential theft, and suppression bypass. |
| `policy/abuse-defence.cedar` | Map bot and emergency bypass logic to threat rows. |
| `iac/edge-waf.yaml` | Bind public ingress threats to edge controls. |
| `iac/local-openbao-policy.hcl` | Bind secret theft threats to short-lived provider credential leases. |
| `runbooks/local-campaign-egress-hold.md` | Add containment for suspected compromised campaign or audience. |

## D. Implementation

1. Enumerate assets: campaign journey state, segment membership, consent ledger ref, attribution reports, provider credentials, webhook secrets, and audit-chain evidence.
2. Enumerate trust boundaries: public form/landing/tracking ingress, tenant API, internal gRPC, AsyncAPI, provider sidecar, marketplace, and cross-cell replication.
3. Map threats to controls: spoofing to signed tokens/mTLS, tampering to idempotency/audit-chain, repudiation to audit event ids, information disclosure to residency/Cedar, denial of service to WAF/capacity admission, elevation to tenant-scoped Cedar.
4. Add tests or validators for each blocker control; avoid prose-only controls where a policy or IaC file exists.
5. Identify residual risks: provider compromise, false-positive deliverability hold, and misconfigured tenant DNS.
6. Add incident response ownership per threat category.
7. Confirm all controls cite real local artifacts, not invented Terraform or Cedar entity types.

## E. Acceptance

- `buck2 build //:quality-lane-registry-authority-check # lane=threat-model --microservice marketing-automation`
- `buck2 build //:quality-lane-registry-authority-check # lane=security-and-hardening --microservice marketing-automation`
- Manual evidence: every high-severity threat has a local control artifact and runbook.

## F. Evidence

- Local docs: `threat-model.md`, `incident-response.md`.
- Local policies: `policy/abuse-defence.cedar`, `policy/campaign-journey-authorization.cedar`, `policies/local-deliverability-inspection.cedar`.
- Local IaC: `iac/edge-waf.yaml`, `iac/local-openbao-policy.hcl`.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| HubSpot Marketing Hub | Public marketing and private-app risk receives an explicit STRIDE-style control map. |
| Adobe Marketo Engage | API, webhook, and smart-campaign abuse paths are tied to controls. |
| Mailchimp | Audience and campaign abuse controls are verifiable in local policy/IaC files. |

## H. Local Traceability

- Threat doc: `threat-model.md`.
- Incident doc: `incident-response.md`.
- Cedar file: `policy/abuse-defence.cedar`.
- Cedar file: `policy/campaign-journey-authorization.cedar`.
- Cedar file: `policies/local-deliverability-inspection.cedar`.
- IaC file: `iac/edge-waf.yaml`.
- IaC file: `iac/local-openbao-policy.hcl`.
- Asset: campaign journey state.
- Asset: segment membership.
- Asset: consent ledger ref.
- Asset: attribution report.
- Asset: provider credential.
- Boundary: public form ingress.
- Boundary: internal gRPC.
- Boundary: AsyncAPI.
- Boundary: provider sidecar.
- Failure state: high-severity threat without local control.
- Failure state: invented Cedar entity type in threat control.

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-024-threat-model-control-map.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-024-threat-model-control-map.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
