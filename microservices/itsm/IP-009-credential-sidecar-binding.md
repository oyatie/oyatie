---
doc_class: IP
ip_id: IP-009-credential-sidecar-binding
microservice: itsm
status: rewritten-wave-15-ip-substance
date: 2026-05-21
owner_team: axis-itsm + cloud-secrets
counterparts: [ServiceNow ITSM, Jira Service Management, Freshservice]
source_artifacts:
  - microservices/itsm/manifest.json
  - microservices/itsm/src/config.rs
  - microservices/itsm/contracts/openapi-v1.yaml
  - microservices/itsm/policy/service-management-authorization.cedar
---

# IP-009 ITSM Credential Sidecar Binding

## A. Problem
ITSM needs credentials for marketplace service-catalog integrations, notification channels, mobile push, status publishing, and optional ServiceNow/Jira/Freshservice import adapters. The stamped IP said "credential sidecar" but never identified which credentials, which tenant mode, or which code boundary consumes them.

The gap is an OpenBao-backed sidecar contract that gives ITSM short-lived references without storing vendor tokens in tickets, CMDB records, or workflow payloads.

## B. Approach
Use the manifest's BYOK and tenant-class posture: demo_trial defaults to platform credentials, paid tenants can opt into BYOK, and HIPAA/FedRAMP-style packs can require BYOK.

Credential classes:

| Credential | Consumer | Secret reference shape |
|---|---|---|
| status-page publisher | `status-update` bounded context | `${openbao:secret/<tenant_id>/itsm/status-page}` |
| mobile push | IP-032 Mobile ITSM | `${openbao:secret/<tenant_id>/itsm/mobile-push}` |
| marketplace catalog provider | IP-014/IP-028 service catalog | `${openbao:secret/<tenant_id>/itsm/catalog-provider}` |
| import adapter | ServiceNow/Jira/Freshservice migration | `${openbao:secret/<tenant_id>/itsm/source-adapter}` |

No credential value should appear in REST, gRPC, AsyncAPI, audit events, or dashboard labels.

## C. Deliverables
- Extend `ServiceConfig` in `src/config.rs` with credential sidecar endpoint and lease TTL settings if not present.
- Define a `CredentialReference` value object in the domain or kernel layer when implementation begins.
- Add OpenAPI request constraints stating `deal_set_id` and `purpose` are allowed, raw credential fields are not.
- Add tests that reject requests containing credential-looking fields once validation exists.
- Bind sidecar authorization to Cedar policy so a requester cannot fetch operator credentials.

## D. Implementation
1. Inspect `ServiceConfig` and add sidecar endpoint, workload SVID, max lease TTL, and fail-closed behavior.
2. Define references as strings only in config/contracts; never deserialize secret material into ITSM domain entities.
3. Add a sidecar adapter that exchanges tenant + purpose + credential class for a short-lived handle.
4. Enforce tenant-class behavior: demo_trial platform default; paid optional BYOK; restricted packs require BYOK.
5. Gate sidecar fetch through `PolicyAuthorizer` using an action such as `itsm.credential.fetch`.
6. Emit audit evidence for lease issued, lease denied, and lease revoked, redacting the secret path beyond tenant/service/class.
7. Add tests covering raw-secret rejection, tenant mismatch denial, and expired lease retry.
8. Document rollback: disable sidecar fetch per credential class while keeping core ticket operations online.

## E. Acceptance
- ITSM code never stores provider tokens in `IncidentTicket`, audit event payloads, or contract messages.
- Credential lookup requires tenant id, principal id, purpose, and credential class.
- Demo/paid/BYOK behavior matches `manifest.json`.
- ServiceNow/Jira/Freshservice import tokens are treated as aliases to sidecar references, not source authority.

## F. Evidence
- `manifest.json` defines BYOK defaults and pack-required BYOK behavior for ITSM.
- `contracts/openapi-v1.yaml` currently has `deal_set_id` but no raw credential fields.
- `policy/service-management-authorization.cedar` is the service-level policy surface for privileged actions.
- ADR-0255 and ADR-0244 govern credential and tenant boundaries.

## G. Counterparts
| Counterpart | Gap closed by this IP |
|---|---|
| ServiceNow IntegrationHub credentials | Credentials become tenant-scoped OpenBao references |
| Jira Service Management app connections | App tokens cannot become project-level authority |
| Freshservice Orchestration Center credentials | BYOK and pack-required credential modes are explicit |

## H. Cold-start buildability notes
- Add `CredentialReference` as an opaque value object; do not expose `String` secrets.
- Keep sidecar lease duration under the manifest BYOK policy.
- Fail closed on sidecar outage for provider actions.
- Allow core incident operations to continue when optional provider credentials fail.
- Test that audit payloads never contain `openbao:secret` full paths.
- Use synthetic tenants for sidecar tests.
- Keep credential classes small until real adapters exist.
- Do not add Terraform or secret backend config in this IP unless files exist.
- Link source-adapter credentials to migration/backfill only.
- Record pack-required BYOK denial separately from generic missing credential.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/itsm/contracts/asyncapi-v1.yaml`, `microservices/itsm/contracts/itsm-v1.proto`, `microservices/itsm/contracts/local-asyncapi-v1.yaml`, `microservices/itsm/contracts/local-openapi-v1.yaml`, `microservices/itsm/contracts/local-operations-v1.proto`, `microservices/itsm/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.
