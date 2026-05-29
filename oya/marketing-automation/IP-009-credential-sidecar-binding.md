---
doc_class: ImplementationPlan
ip_id: IP-009-credential-sidecar-binding
microservice: marketing-automation
bounded_contexts: [webhook-subscription, ad-network-seam, social-seam, marketplace-audience-license, deliverability]
related_adrs: [ADR-0244, ADR-0253-amendment, ADR-0263, ADR-0314, ADR-0321, ADR-0328]
status: proposed
date: 2026-05-21
owner: axis-marketing-automation + ops-security
tenant_class_aware: true
---

# IP-009: Credential Sidecar Binding

## A. Problem

Marketing Automation connects to tenant webhooks, ad networks, social publishers, marketplace DealSets, and deliverability providers. The stamped IP did not say how secrets are acquired or constrained. The actual risk is credential sprawl: API keys for HubSpot-like app marketplace behavior, Marketo webhook endpoints, Mailchimp audience sync, and ad/social seams must never live in journey state or event payloads.

## B. Approach

Use the existing OpenBao sidecar posture referenced by Cedar (`provider_credential_mode in ["none", "openbao_sidecar_ttl_60s"]`) and local IaC (`iac/local-openbao-policy.hcl`, `iac/local-secret-binding.yaml`, `iac/secret-bindings.yaml`). The service receives opaque credential handles, mints short-lived provider tokens through the sidecar, and records only credential handle ids plus audit-chain references.

## C. Deliverables

| Artifact | Change |
|---|---|
| `iac/local-openbao-policy.hcl` | Scope secrets to `marketing-automation/<tenant_id>/<provider>/<credential_id>` with read-only TTL leases. |
| `iac/local-secret-binding.yaml` and `iac/secret-bindings.yaml` | Bind sidecar volume/env to the app and worker pods without exposing provider secrets in config maps. |
| `src/config.rs` | Add explicit config for sidecar endpoint, lease TTL, and allowed provider namespaces. |
| `src/usecase/mod.rs` | Pass credential handle context to policy; never persist raw secret values. |
| `runbooks/webhook-signature-failure.md` and `runbooks/provider-migration-rollback.md` | Add credential rotation and revoked-handle recovery. |

## D. Implementation

1. Inventory provider credential consumers: webhook signing, ad-network audience sync, social seam publication, marketplace audience-license settlement, deliverability DNS/provider checks.
2. Add `CredentialHandle` validation to domain or config layer with tenant-bound provider namespace.
3. Require Cedar context `provider_credential_mode == "openbao_sidecar_ttl_60s"` for any provider call that needs a secret.
4. Configure OpenBao policy so workers can read only tenant/provider paths assigned to the current principal and deployment cell.
5. Add audit event fields `credential_handle_id`, `lease_id_hash`, and `provider_namespace`; never log token material.
6. Add tests for missing sidecar, expired lease, wrong tenant namespace, and revoked credential handle.
7. Keep OAuth consent and provider marketplace UX outside this IP; this binds runtime secret access only.

## E. Acceptance

- `cargo test -p oya-marketing-automation-campaign-journey-app credential`
- `cargo run -p oya-dev-cli -- gate validate secret-bindings --microservice marketing-automation`
- `cargo run -p oya-dev-cli -- gate validate bypass --microservice marketing-automation`
- Manual evidence: no `secret`, `api_key`, or provider token field appears in persisted Marketing Automation command receipts.

## F. Evidence

- Local policy: `policy/campaign-journey-authorization.cedar` references `openbao_sidecar_ttl_60s`.
- Local IaC: `iac/local-openbao-policy.hcl`, `iac/local-secret-binding.yaml`, `iac/secret-bindings.yaml`.
- Local runbooks: `webhook-signature-failure.md`, `provider-migration-rollback.md`.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| HubSpot Marketing Hub | Private-app and marketplace credentials get tenant-bound lease handling instead of app-global storage. |
| Adobe Marketo Engage | REST/Bulk API credentials can rotate without changing campaign or journey state. |
| Mailchimp | Audience and webhook integrations use handle-based secret access with audit-chain evidence. |

## H. Local Traceability

- Secret store: OpenBao sidecar.
- IaC file: `iac/local-openbao-policy.hcl`.
- IaC file: `iac/local-secret-binding.yaml`.
- IaC file: `iac/secret-bindings.yaml`.
- Config target: `src/config.rs`.
- Cedar context: `provider_credential_mode`.
- Allowed mode: `none`.
- Allowed mode: `openbao_sidecar_ttl_60s`.
- Provider surface: webhook signing.
- Provider surface: ad-network audience sync.
- Provider surface: social publishing seam.
- Provider surface: deliverability DNS/provider checks.
- Provider surface: marketplace audience license settlement.
- Audit field: `credential_handle_id`.
- Audit field: `lease_id_hash`.
- Runbook: `webhook-signature-failure.md`.
- Runbook: `provider-migration-rollback.md`.
- Failure state: raw provider token persisted anywhere is a blocker.
