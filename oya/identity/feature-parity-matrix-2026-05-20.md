# Identity feature parity matrix - 2026-05-20

Citation anchors:
1. Canonical audit sequence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-4234`.
2. Master plan constraints: `specs/master-plan-sequencing.json:704-889`.
3. Identity PRD: `microservices/identity/PRD.md:1-1642`.
4. Identity architecture: `microservices/identity/ARCHITECTURE.md:1-880`.
5. Documentation rigor: `docs/standards/documentation-rigor.md:133-139` and `docs/standards/documentation-rigor.md:58-83`.

External source anchors:
- Auth0 Organizations: `https://auth0.com/docs/manage-users/organizations/organizations-overview`.
- Auth0 MFA: `https://auth0.com/docs/secure/multi-factor-authentication`.
- Auth0 Inbound SCIM: `https://auth0.com/docs/authenticate/protocols/scim/configure-inbound-scim`.
- Auth0 Attack Protection: `https://auth0.com/docs/secure/attack-protection`.
- Auth0 rate-limit policy: `https://auth0.com/docs/policies/rate-limit-policy`.
- Okta app integrations and SCIM: `https://developer.okta.com/docs/guides/create-an-app-integration/scim/main/`.
- Okta MFA authenticators: `https://help.okta.com/oie/en-us/content/topics/identity-engine/authenticators/about-authenticators.htm`.
- Okta FastPass: `https://help.okta.com/oie/en-us/content/topics/identity-engine/devices/fp/fp-main.htm`.
- Okta SCIM concepts: `https://developer.okta.com/docs/concepts/scim/`.
- Okta Identity Governance: `https://help.okta.com/oie/en-us/content/topics/identity-governance/iga.htm`.
- Okta rate limits: `https://developer.okta.com/docs/reference/rate-limits/`.
- Microsoft Entra Conditional Access: `https://learn.microsoft.com/en-us/entra/identity/conditional-access/concept-conditional-access-policies`.
- Microsoft Entra passkeys/FIDO2: `https://learn.microsoft.com/en-us/entra/identity/authentication/how-to-authentication-passkeys-fido2`.
- Microsoft Entra provisioning: `https://learn.microsoft.com/en-us/entra/identity/app-provisioning/how-provisioning-works`.
- Microsoft Entra ID Protection: `https://learn.microsoft.com/en-us/entra/id-protection/id-protection-dashboard`.
- Microsoft Entra ID Governance: `https://learn.microsoft.com/en-us/entra/id-governance/identity-governance-overview`.

Local evidence anchors:
- Identity OIDC/WebAuthn/SCIM/step-up contract: `microservices/identity/contracts/openapi/identity.yaml:49-462`.
- Identity event contract: `microservices/identity/contracts/asyncapi/identity-events.yaml:1-196`.
- Identity proto contract: `microservices/identity/contracts/proto/identity.proto:15-99`.
- Identity passkey ADR: `microservices/identity/decisions/ADR-ID-001-passkey-primary-webauthn-recovery-envelope.md:47-207`.
- Identity competitor claim: `microservices/identity/competitor-parity-matrix.md:16-100`.
- Identity tenant_class adoption matrix: `microservices/identity/ADR-0330 and ADR-0331 tenant_class model:1-181`.

## 1. Counterpart 1 - Auth0 capability surface

01. Universal Login hosted authorization-server login flow; source: Auth0 Universal Login docs.
02. Branded hosted login pages; source: Auth0 Universal Login docs.
03. Social identity provider connections; source: Auth0 identity fundamentals docs.
04. Enterprise identity provider connections; source: Auth0 enterprise/SCIM docs.
05. SAML enterprise connection support; source: Auth0 inbound SCIM docs.
06. OpenID enterprise connection support; source: Auth0 inbound SCIM docs.
07. Okta Workforce enterprise connection support; source: Auth0 inbound SCIM docs.
08. Microsoft Entra enterprise connection support; source: Auth0 inbound SCIM docs.
09. Organizations for B2B tenants; source: Auth0 Organizations docs.
10. Organization membership management; source: Auth0 Organizations docs.
11. Organization login prompt and organization discovery flow; source: Auth0 Organizations docs.
12. Organization-specific branding through page templates; source: Auth0 Organizations docs.
13. Organization roles for application access; source: Auth0 Organizations docs.
14. Organization APIs for customer self-management; source: Auth0 Organizations docs.
15. Machine-to-machine access scoped to organizations; source: Auth0 Organizations docs.
16. Management API for users, clients, organizations, roles, and configuration; source: Auth0 docs.
17. Management API rate-limit policy; source: Auth0 rate-limit policy docs.
18. Authentication API for MFA factor management; source: Auth0 MFA API docs.
19. MFA with push notifications; source: Auth0 MFA docs.
20. MFA with SMS notifications; source: Auth0 MFA docs.
21. MFA with voice notifications; source: Auth0 MFA docs.
22. MFA with one-time passwords; source: Auth0 MFA docs.
23. MFA with WebAuthn security keys; source: Auth0 MFA docs.
24. MFA with WebAuthn device biometrics; source: Auth0 MFA docs.
25. MFA with email notification; source: Auth0 MFA docs.
26. MFA with Cisco Duo; source: Auth0 MFA docs.
27. MFA recovery codes; source: Auth0 MFA docs.
28. Adaptive MFA customization through Actions; source: Auth0 MFA docs.
29. Actions extensibility for post-login and other flows; source: Auth0 Actions docs.
30. Actions secrets/dependency model; source: Auth0 Actions docs.
31. Attack protection bot detection; source: Auth0 Attack Protection docs.
32. Attack protection suspicious IP throttling; source: Auth0 Attack Protection docs.
33. Attack protection brute-force protection; source: Auth0 Attack Protection docs.
34. Attack protection breached-password detection; source: Auth0 Attack Protection docs.
35. Inbound SCIM endpoint per enterprise connection; source: Auth0 SCIM docs.
36. SCIM token generation and rotation; source: Auth0 SCIM docs.
37. SCIM user create/update/delete testing flow; source: Auth0 SCIM docs.
38. Session revocation on SCIM deactivate/block; source: Auth0 SCIM docs.
39. Tenant logs for SCIM and security events; source: Auth0 SCIM and attack protection docs.
40. Log/event monitoring surfaces; source: Auth0 security docs.
41. Compliance posture including GDPR and HIPAA in security docs; source: Auth0 security docs.
42. Rate limits across tenants, APIs, endpoints, and extensibility products; source: Auth0 rate-limit policy docs.
43. Private Cloud environment request limit classes; source: Auth0 rate-limit policy docs.
44. Custom domains and tenant configuration constraints; source: Auth0 Organizations limitations.
45. Account recovery through MFA reset and recovery-code paths; source: Auth0 MFA reset docs.
46. Authentication factor enrollment management API; source: Auth0 MFA API docs.
47. Localization and hosted login customization; source: Auth0 Universal Login docs.
48. Confidential-client-only Management API guidance; source: Auth0 Organizations docs.
49. Postman-based SCIM validation collection; source: Auth0 SCIM docs.
50. Tenant administrator attack-protection notifications; source: Auth0 Attack Protection docs.

## 2. Counterpart 2 - Okta capability surface

01. Okta Identity Engine hosted identity platform; source: Okta OIE docs.
02. App integrations for secure SSO; source: Okta app integration docs.
03. OIDC app integration support; source: Okta app integration docs.
04. SAML app integration support; source: Okta app integration docs.
05. Secure Web Authentication support; source: Okta app integration docs.
06. WS-Fed app integration support; source: Okta app integration docs.
07. SCIM app provisioning support; source: Okta app integration docs.
08. OAuth 2.0 API service integration support; source: Okta app integration docs.
09. Public Okta Integration Network listing path; source: Okta app integration docs.
10. Private app integration path; source: Okta app integration docs.
11. Integration security validation for public OIN integrations; source: Okta app integration docs.
12. MFA factor enrollment policies; source: Okta authenticators docs.
13. Email authenticator; source: Okta authenticators docs.
14. Phone authenticator; source: Okta authenticators docs.
15. IdP authenticator; source: Okta authenticators docs.
16. Custom OTP authenticator; source: Okta authenticators docs.
17. Duo Security authenticator; source: Okta authenticators docs.
18. Google Authenticator support; source: Okta authenticators docs.
19. Symantec VIP support; source: Okta authenticators docs.
20. YubiKey OTP support; source: Okta authenticators docs.
21. Smart Card support; source: Okta authenticators docs.
22. Passkeys/FIDO2 WebAuthn support; source: Okta authenticators docs.
23. Okta Verify TOTP and Push; source: Okta authenticators docs.
24. Okta FastPass phishing-resistant authenticator; source: Okta FastPass docs.
25. Okta FastPass passwordless public-key authentication; source: Okta FastPass docs.
26. Okta FastPass device posture collection and evaluation; source: Okta FastPass docs.
27. Temporary access code; source: Okta authenticators docs.
28. Authenticator method characteristics such as device-bound, hardware-protected, phishing-resistant, and user-verifying; source: Okta authenticators docs.
29. SCIM 2.0 and SCIM 1.1 support; source: Okta SCIM docs.
30. SCIM create/read/update/delete lifecycle operations; source: Okta SCIM docs.
31. SCIM deprovisioning through `active=false`; source: Okta SCIM docs.
32. SCIM profile sourcing; source: Okta SCIM docs.
33. Profile attribute mapping; source: Okta SCIM docs.
34. Group push and group lifecycle; source: Okta SCIM/provisioning docs.
35. Provisioning from upstream and downstream apps; source: Okta provisioning docs.
36. Lifecycle Management; source: Okta Identity Governance docs.
37. Okta Workflows for custom automation; source: Okta Identity Governance docs.
38. Access Governance; source: Okta Identity Governance docs.
39. Access Certifications campaigns; source: Okta Identity Governance docs.
40. Access Requests; source: Okta Identity Governance docs.
41. Entitlement Management; source: Okta Identity Governance docs.
42. Resource collections; source: Okta Identity Governance docs.
43. Resource owners and labels; source: Okta Identity Governance docs.
44. Separation-of-duties rules; source: Okta Identity Governance docs.
45. Governance reports, APIs, and System Log events; source: Okta Identity Governance docs.
46. Rate-limit buckets with org and nested client-app scope; source: Okta rate-limit docs.
47. API-specific example quotas such as `/oauth2/v1/authorize`; source: Okta rate-limit docs.
48. Authenticated-user buckets such as `/api/v1/users/me`; source: Okta rate-limit docs.
49. HTTP 429 enforcement and reset windows; source: Okta rate-limit docs.
50. Rate-limit dashboard, System Log, and response headers; source: Okta rate-limit docs.

## 3. Counterpart 3 - Microsoft Entra ID identity platform capability surface

01. Microsoft identity platform for application authentication and authorization; source: Microsoft Entra identity-platform docs.
02. Microsoft Entra ID tenant and directory identity; source: Microsoft Entra docs.
03. Workforce tenant support; source: Microsoft identity platform docs.
04. External ID / customer and partner identity surface; source: Microsoft Entra docs.
05. Workload identity assignments in Conditional Access; source: Conditional Access docs.
06. Conditional Access policy engine; source: Conditional Access docs.
07. Conditional Access users and groups assignment; source: Conditional Access docs.
08. Conditional Access directory role targeting; source: Conditional Access docs.
09. Conditional Access external/guest user targeting; source: Conditional Access docs.
10. Conditional Access workload identity targeting; source: Conditional Access docs.
11. Conditional Access target resources; source: Conditional Access docs.
12. Conditional Access network and location signals; source: Conditional Access docs.
13. Conditional Access sign-in risk condition; source: Conditional Access docs.
14. Conditional Access device platform condition; source: Conditional Access docs.
15. Conditional Access client app condition; source: Conditional Access docs.
16. Conditional Access device attribute filter; source: Conditional Access docs.
17. Grant controls for MFA, authentication strength, compliant device, hybrid joined device, approved client app, app protection policy, password change, and terms of use; source: Conditional Access docs.
18. Session controls for app-enforced restrictions, Conditional Access App Control, sign-in frequency, persistent browser session, continuous access evaluation, and resilience defaults; source: Conditional Access docs.
19. Privileged Identity Management integration guidance for role activation evaluation; source: Conditional Access docs.
20. Passkeys/FIDO2 authentication for workforce users; source: Entra passkey docs.
21. Device-bound passkeys on FIDO2 security keys and Microsoft Authenticator; source: Entra passkey docs.
22. Synced passkeys; source: Entra passkey docs.
23. Passkey profiles; source: Entra passkey docs.
24. Passkey attestation policy; source: Entra passkey docs.
25. AAGUID allow/block restrictions; source: Entra passkey docs.
26. Group-targeted passkey profiles; source: Entra passkey docs.
27. Authentication methods including Windows Hello, passkeys, certificate-based authentication, and platform credentials; source: Entra authentication overview.
28. Automatic provisioning for SaaS apps and other systems; source: Entra provisioning docs.
29. SCIM 2.0 provisioning endpoint support; source: Entra provisioning docs.
30. User create/update/remove automation; source: Entra provisioning docs.
31. Group create/update/remove automation for selected apps; source: Entra provisioning docs.
32. On-premises provisioning agent and connector translation; source: Entra provisioning docs.
33. TLS 1.2 encrypted provisioning channel; source: Entra provisioning docs.
34. Identity Protection risk detections; source: Entra ID Protection docs.
35. Unified risk signals from Microsoft Defender and non-Microsoft sources; source: Entra ID Protection docs.
36. Identity Risk Score and risk-based Conditional Access trigger; source: Entra ID Protection docs.
37. Identity Governance access lifecycle; source: Entra ID Governance docs.
38. Lifecycle workflows; source: Entra ID Governance docs.
39. Entitlement management; source: Entra ID Governance docs.
40. Access packages; source: Entra ID Governance docs.
41. Access reviews and recertification; source: Entra ID Governance docs.
42. Separation-of-duties checks in access request paths; source: Entra ID Governance docs.
43. Guest identity governance; source: Entra ID Governance docs.
44. Connectors to cloud and on-premises applications; source: Entra ID Governance docs.
45. App integration with SAML, OIDC, SCIM, REST/SOAP, AD groups, and on-prem directories; source: Entra ID Governance docs.
46. Terms-of-use enforcement via Conditional Access; source: Entra ID Governance docs.
47. B2B external guest user support; source: Conditional Access docs.
48. Microsoft Graph APIs for identity operations; source: Microsoft Entra docs.
49. Service limits and throttling guidance; source: Microsoft Entra service-limit docs.
50. SLA/resilience guidance for Entra identity services; source: Microsoft Entra architecture docs.

## 4. Union-coverage matrix

| Capability | Auth0 | Okta | Entra | Union required | Oyatie identity has | Gap classification |
|---|---|---|---|---|---|---|
| Hosted login / authorization-server UX | yes | yes | yes | yes | partial via contract, no UI | catch-up |
| OIDC/OAuth 2.0 issuer | yes | yes | yes | yes | yes, OpenAPI paths | parity |
| JWKS publication | yes | yes | yes | yes | yes, OpenAPI and ADR | parity |
| OAuth PKCE support | yes | yes | yes | yes | implied in PRD and contract | partial |
| SAML app/enterprise federation | yes | yes | yes | yes | yes, PRD and federation binding | parity |
| Enterprise OIDC federation | yes | yes | yes | yes | yes | parity |
| Social IdP federation | yes | partial | partial | yes | named in PRD | partial |
| B2B organization model | yes | yes | yes | yes | partial tenant/org model | catch-up |
| Organization membership admin | yes | yes | yes | yes | not explicit API | gap |
| Organization roles | yes | yes | yes | yes | partial via Cedar/roles | partial |
| Customer self-admin organization API | yes | partial | yes | yes | not explicit | gap |
| Machine-to-machine organization access | yes | yes | yes | yes | service principals implied | partial |
| MFA push | yes | yes | yes | yes | not explicit factor | gap |
| MFA SMS | yes | yes | yes | yes | SMS rejected/absent | additive-security divergence |
| MFA voice | yes | yes | partial | yes | absent | conscious gap |
| MFA OTP/TOTP | yes | yes | yes | yes | partial | partial |
| MFA email | yes | yes | yes | yes | partial | partial |
| MFA Duo/external factor | yes | yes | partial | yes | not explicit | gap |
| MFA recovery codes | yes | yes | yes | yes | recovery envelope | ahead/different |
| WebAuthn security keys | yes | yes | yes | yes | yes | parity |
| WebAuthn device biometrics | yes | yes | yes | yes | yes | parity |
| Passkeys/FIDO2 | yes | yes | yes | yes | yes | parity |
| Synced passkeys | partial | yes | yes | yes | yes, via ADR policy distinction | parity |
| Device-bound passkeys | yes | yes | yes | yes | yes | parity |
| AAGUID allow/block | partial | partial | yes | yes | yes | ahead/parity |
| Attestation policy | partial | yes | yes | yes | yes | parity |
| Phishing-resistant auth policy | yes | yes | yes | yes | yes | parity |
| Temporary access pass/code | partial | yes | yes | yes | recovery ceremony, not TAP | partial |
| Step-up authentication | yes | yes | yes | yes | yes | parity |
| ACR class model | partial | yes | yes | yes | yes | parity |
| JIT approval for critical operations | partial | partial | yes/PIM | yes | yes, ADR-identity-005 | parity |
| Privileged identity management | partial | partial | yes | yes | partial JIT only | catch-up |
| Conditional access policy engine | partial | yes | yes | yes | Cedar + ACR partial | partial |
| Device posture evaluation | partial | yes | yes | yes | not explicit | gap |
| Network/location risk signals | yes | yes | yes | yes | edge rules partial | partial |
| Sign-in risk detection | yes | yes | yes | yes | IP-014 deferred | catch-up |
| Unified risk score | partial | partial | yes | yes | deferred | gap |
| Bot detection | yes | yes | partial | yes | edge authz partial | partial |
| Suspicious IP throttling | yes | yes | yes | yes | runbook + edge config | partial |
| Brute-force protection | yes | yes | yes | yes | runbook + edge config | partial |
| Breached password detection | yes | yes | yes | yes | not central due passkey-first | conscious gap |
| SCIM inbound provisioning | yes | yes | yes | yes | yes | parity |
| SCIM outbound provisioning | partial | yes | yes | yes | deferred per local matrix | gap |
| SCIM group lifecycle | yes | yes | yes | yes | yes | parity |
| SCIM deprovision active=false | yes | yes | yes | yes | partial | partial |
| HRIS connector | partial | yes | yes | yes | yes, IP-009 | parity |
| Profile sourcing | partial | yes | yes | yes | not explicit | gap |
| Profile attribute mapping | yes | yes | yes | yes | partial | partial |
| App integration catalog | yes | yes | yes | yes | absent | gap |
| Public integration submission workflow | yes | yes | yes | yes | absent | gap |
| API service integrations | yes | yes | yes | yes | partial | partial |
| Management API | yes | yes | yes | yes | partial contracts | partial |
| Admin dashboard | yes | yes | yes | yes | dashboard JSON only | gap |
| Tenant logs | yes | yes | yes | yes | audit-chain events | ahead/different |
| Event streaming/log export | yes | yes | yes | yes | AsyncAPI events | parity |
| Rate-limit dashboard | yes | yes | yes | yes | no admin surface | gap |
| Rate-limit headers/policies | yes | yes | yes | yes | partial SLO/rate docs | partial |
| API quota tiers | yes | yes | yes | yes | capability availability partial | partial |
| Access requests | partial | yes | yes | yes | governance handoff only | gap |
| Access certifications | partial | yes | yes | yes | absent | gap |
| Entitlement management | partial | yes | yes | yes | Cedar primitives partial | catch-up |
| Resource owners | partial | yes | yes | yes | absent | gap |
| Separation-of-duties rules | partial | yes | yes | yes | JIT two-person rule partial | partial |
| Terms-of-use enforcement | partial | yes | yes | yes | not explicit | gap |
| Guest/external user governance | yes | yes | yes | yes | tenant/audience partial | partial |
| Workload identities | yes | yes | yes | yes | SPIFFE doctrine in chat, thin docs | partial |
| Service-to-service principal verification | yes | yes | yes | yes | yes, proto verifier | parity |
| Recovery-code path | yes | yes | yes | yes | recovery envelope stronger | ahead |
| Operator unable to decrypt recovery secret | partial | partial | partial | yes for Oyatie | yes | additive |
| Session rotation after recovery | partial | yes | yes | yes | yes, ADR-ID-001 | parity |
| Backchannel logout/session revocation | yes | yes | yes | yes | partial via events | partial |
| Audit-chain sealing | partial | partial | partial | yes for Oyatie | yes, intended | additive |
| Per-tenant audit slicing | partial | partial | partial | yes for Oyatie | yes, intended | additive |
| Sovereign/air-gapped deployment | no | no | partial | yes for Oyatie strategy | partial, deployment blocked | catch-up |
| Per-pack residency | partial | partial | yes | yes | yes, docs | ahead/partial |
| demo_trial OCI Always Free | no | no | no | yes for Oyatie | no | gap |
| Multi-context OpenTofu deployment | no | no | no | yes for Oyatie | no | gap |
| Supported OS manifest | no | no | no | yes for Oyatie | no | gap |
| Rust-strict backend | no | no | no | yes for Oyatie | current corpus yes | additive |
| Developer Actions/extensibility | yes | yes Workflows | yes Graph/Logic Apps | yes | Cedar/events partial | catch-up |
| No-code/low-code workflow hooks | partial | yes | yes | yes | governance/workflow handoff only | gap |
| Compliance controls | yes | yes | yes | yes | yes | parity |
| HIPAA support | yes | yes | yes | yes | yes docs | parity |
| GDPR support | yes | yes | yes | yes | yes docs | parity |
| SOC2/ISO support | yes | yes | yes | yes | yes docs | parity |
| KR/KSA/UAE pack overlays | no | no | partial | yes for Oyatie | yes docs, deployment partial | additive/blocked |
| Minor/age-tenant_class handling | partial | partial | partial | yes for Oyatie | yes PRD/IP journeys | additive |
| Survivor/emergency identity flows | partial | partial | partial | yes for Oyatie | yes IP journeys | additive |
| Healthcare break-glass | partial | partial | yes | yes | yes | parity |
| FIDO MDS/AAGUID refresh worker | partial | partial | yes | yes | yes IP-006 | parity |
| JWKS emergency rotation | yes | yes | yes | yes | yes ADR/runbook | parity |
| HSM-backed signing | yes enterprise | yes enterprise | yes | yes | yes docs | parity |
| BYOK/key custody | yes enterprise | yes enterprise | yes | yes | OpenBao/HSM | parity |
| Fine-grained tenant Cedar policy | partial | yes | yes | yes | yes | parity |
| Context-split principal model | no | no | partial | yes for Oyatie | partial/scaffolded | catch-up |
| Personal/work dual-context protection | no | partial | partial | yes for Oyatie | yes design, partial maturity | additive/partial |
| Integration test set | yes | yes | yes | yes | test plans only | partial |
| Raw measured benchmark evidence | partial | partial | partial | yes for Oyatie | absent | gap |
| SLA/SLO public evidence | yes | yes | yes | yes | SLO docs yes | partial |
| Control-plane deployment docs | yes | yes | yes | yes | partial, missing OpenTofu | gap |
| Tenant onboarding automation | yes | yes | yes | yes | absent per context | gap |
| Self-host install story | no | no | partial | yes for Oyatie | partial docs, missing IaC | catch-up |
| Colo install story | no | no | partial | yes for Oyatie | missing IaC | gap |
| Public-cloud managed service story | yes | yes | yes | yes | product docs yes, IaC no | partial |

## 5. Capability families summary table

| Capability family | Union required count | Oyatie present | Oyatie partial | Oyatie absent | Headline |
|---|---:|---:|---:|---:|---|
| Core authentication protocols | 12 | 8 | 4 | 0 | Strong protocol base, missing hosted UX detail. |
| Passkeys and MFA | 18 | 8 | 6 | 4 | Passkey-first is strong; conventional MFA breadth is thinner. |
| Federation and organizations | 12 | 4 | 6 | 2 | Federation exists; organization administration is under-specified. |
| Lifecycle and provisioning | 14 | 5 | 5 | 4 | Inbound SCIM strong; outbound SCIM/profile sourcing weak. |
| Risk, attack protection, and conditional access | 16 | 3 | 8 | 5 | Edge/runbook exists; continuous risk/device posture missing. |
| Governance and privileged access | 15 | 2 | 5 | 8 | JIT approval is good; access reviews/entitlements missing. |
| Admin, extensibility, and integration catalog | 14 | 2 | 5 | 7 | Contracts exist; product operations surface missing. |
| Audit, compliance, and sovereignty | 16 | 10 | 5 | 1 | Oyatie has additive audit/residency ambition but deployability is blocked. |
| Deployment and platform doctrine | 10 | 1 | 2 | 7 | Canonical ADR-0328 constraints are the main gap. |
| Benchmark and operations evidence | 8 | 2 | 3 | 3 | SLOs exist, measured results incomplete. |

## 6. Headline gap analysis - top 15 missing capabilities

01. Gap: all six canonical OpenTofu deployment contexts are missing.
    Evidence: no `iac/oyatie-public-cloud`, `guest-on-aws`, `oci-guest`, `on-prem`, `colo`, or `oyatie-iaas` path exists.
    Hook: create OpenTofu modules behind `cloud-iac` and keep Helm/Kustomize as Kubernetes payloads.
02. Gap: OCI Always Free demo_trial Always Free profile is missing.
    Evidence: `ADR-0330 and ADR-0331 tenant_class model:15-24` exceeds Always Free budget; no `iac/oci-guest/always-free/` exists.
    Hook: split demo_trial into OCI Always Free and general demo_trial variants.
03. Gap: supported OS manifest is missing.
    Evidence: no `supported-oses.json`; master plan requires per-microservice manifest.
    Hook: add manifest with 13 tenant_class-1 OSes, 2 tenant_class-2 test-only architectures, out-of-scope declarations, package formats, and CI lanes.
04. Gap: organization self-administration API is under-specified.
    Evidence: Auth0 Organizations provides roles and organization APIs; Oyatie has tenant roles but no comparable admin API contract.
    Hook: extend OpenAPI with org membership, org roles, org connection, and delegated admin endpoints.
05. Gap: outbound SCIM provisioning is deferred.
    Evidence: local competitor matrix marks outbound SCIM deferred (`competitor-parity-matrix.md:47-49`).
    Hook: add outbound SCIM connector, rate policy, event replay, and deprovision semantics.
06. Gap: access certification campaigns are absent.
    Evidence: Okta and Entra both expose governance/access-review capability; identity docs have no campaign surface.
    Hook: hand off campaign workflow to governance but keep principal/session evidence in Identity.
07. Gap: access requests and entitlement packages are absent.
    Evidence: counterpart governance docs expose requestable access and entitlements.
    Hook: add entitlement grant subject model and events consumed by Governance/Cedar.
08. Gap: separation-of-duties rule surface is partial.
    Evidence: JIT approval two-person rule exists, but no general SOD rule model exists.
    Hook: add SOD policy capability record and cross-service handoff.
09. Gap: device posture evaluation is absent.
    Evidence: Okta FastPass and Entra Conditional Access include device signals; Oyatie only has authenticator class and edge rules.
    Hook: add device posture claims, attestation source, and Cedar predicates.
10. Gap: continuous risk score is deferred.
    Evidence: local matrix admits continuous risk scoring is Phase 2/IP-014 (`competitor-parity-matrix.md:38-40`).
    Hook: finish IP-014 with risk event contract and session re-evaluation semantics.
11. Gap: integration gallery / catalog publication workflow is absent.
    Evidence: Okta OIN and Entra gallery are counterpart surfaces.
    Hook: add customer integration submission lifecycle and verification checklist.
12. Gap: tenant rate-limit admin surface is absent.
    Evidence: Okta and Auth0 document rate-limit policies and dashboards.
    Hook: expose per-tenant quota configuration, headers, dashboards, and alert events.
13. Gap: workload identity is thin.
    Evidence: chat history says CI/deploy/service actors authenticate as principals, but local docs do not productize workload identities.
    Hook: add workload principal capability record and OpenAPI/admin endpoints.
14. Gap: benchmark raw evidence is absent.
    Evidence: benchmark doc has values and result-path claims, but no result artifact was observed.
    Hook: add ADR-0212 measured benchmark result bundle after build phase.
15. Gap: cross-microservice handoff document is absent.
    Evidence: no `cross-microservice-handoffs.md`.
    Hook: add handoff doc for tenancy, policy-engine, governance, audit-chain, observability, cloud-iac, cloud-iam, cloud-secrets, and billing.

## 7. Additive surface - Oyatie capabilities not present in all counterparts

01. Additive: tenant-scoped internal operator and CI/deploy actor identity, grounded in chat-history doctrine (`8f603fc7...jsonl:552-571`).
02. Additive: personal/work dual-context principal boundary and recovery separation (`decisions/ADR-ID-001...md:34-40`).
03. Additive: survivor-safety recovery journeys across `IP-journey-j04` and related journey IPs.
04. Additive: minor safety and parental binding journeys across `PRD.md:740-749` and journey IPs.
05. Additive: emergency 911/healthcare break-glass subject resolver IPs.
06. Additive: OpenBao recovery envelope custody that prevents support-operator decryption (`decisions/ADR-ID-001...md:57-74`).
07. Additive: audit-chain Merkle/Ed25519 seal model in local competitor matrix (`competitor-parity-matrix.md:72-81`).
08. Additive: per-pack regulatory overlays for KR, KSA, AE, EU, and US healthcare.
09. Additive: air-gapped sovereign pack ambition, although deployment evidence is blocked.
10. Additive: EU AI Act capability tagging in local parity matrix (`competitor-parity-matrix.md:79-81`).
11. Additive: Rust-strict backend policy for identity service implementation.
12. Additive: multi-context principal resolver, though current maturity is scaffolded and conflicts with manifest GA.
13. Additive: Cedar-first tenant policy composition across principal, session, ACR, and data-use claims.
14. Additive: explicit demo_trial OCI Always Free obligation, though not yet met.
15. Additive: six-context deployment doctrine across public cloud, guest AWS, guest OCI, on-prem, colo, and Oyatie-as-cloud-provider.
16. Additive: identity as provider-control-plane prerequisite for Oyatie-as-cloud-provider.
17. Additive: local HSM/OpenBao per-pack signing key custody model.
18. Additive: audit event parity for recovery denial and recovery success.
19. Additive: AAGUID catalog freshness and policy rollback as first-class operations.
20. Additive: context-split tokens carrying tenant, audience, home cell, credential epoch, and recovery epoch.

## 8. Union-gap remediation slices for aggregation

01. Slice FP-IDENTITY-01: add `supported-oses.json` before any parity claim is promoted, because counterpart maturity cannot compensate for Oyatie's own tenant_class-1 OS doctrine gap.
02. Slice FP-IDENTITY-02: add all six OpenTofu context module directories and keep Helm/Kustomize under those deployable contexts.
03. Slice FP-IDENTITY-03: add `iac/oci-guest/always-free/` and a demo_trial OCI capability profile with explicit hard caps.
04. Slice FP-IDENTITY-04: add organization membership, organization role, organization connection, and delegated-admin OpenAPI endpoints to match Auth0 Organizations.
05. Slice FP-IDENTITY-05: add outbound SCIM connector contracts and idempotency semantics to match Okta and Entra provisioning breadth.
06. Slice FP-IDENTITY-06: add profile sourcing and attribute mapping semantics so HRIS and external directory sync can be built without asking a service owner.
07. Slice FP-IDENTITY-07: add access request and access certification event contracts, with Governance owning workflow and Identity owning principal/session evidence.
08. Slice FP-IDENTITY-08: add entitlement package and separation-of-duties primitives so Cedar can evaluate governance decisions consistently.
09. Slice FP-IDENTITY-09: add device posture claims, device assurance evidence, and policy predicates to close the Okta FastPass and Entra Conditional Access gap.
10. Slice FP-IDENTITY-10: finish continuous risk scoring IP-014 with risk event taxonomy, scoring inputs, policy output, and session re-evaluation behavior.
11. Slice FP-IDENTITY-11: add rate-limit policy endpoints, headers, dashboards, and audit events to close Auth0/Okta operational parity.
12. Slice FP-IDENTITY-12: add integration catalog lifecycle docs for private, partner, and public integrations, including security review and publication states.
13. Slice FP-IDENTITY-13: add workload identity capability records for CI agents, deploy actors, service accounts, and SPIFFE-bound service-to-service calls.
14. Slice FP-IDENTITY-14: add admin-console contract or explicit downstream ownership if another microservice owns the UI.
15. Slice FP-IDENTITY-15: add measured benchmark result artifacts and update the local benchmark doc from modeled claims to measured evidence.
16. Slice FP-IDENTITY-16: add `cross-microservice-handoffs.md` with forward and reverse references for Tenancy, Governance, Policy Engine, Audit Chain, Observability, Cloud IAM, Cloud Secrets, Cloud IaC, and Billing.
17. Slice FP-IDENTITY-17: downgrade `competitor-parity-matrix.md` from full parity to partial parity until governance/risk/admin/deployment surfaces land.
18. Slice FP-IDENTITY-18: add provider-by-provider social IdP matrix for Apple, Google, Kakao, LINE, WeChat, Naver, and future regional IdPs.
19. Slice FP-IDENTITY-19: add recovery-envelope throughput, storage, retention, and audit limits per tenant_class.
20. Slice FP-IDENTITY-20: add tenant-facing SLA/SLO publication model that separates target numbers from measured evidence.
21. Slice FP-IDENTITY-21: add passkey compatibility matrix across browsers, OSes, native apps, and mobile device flows.
22. Slice FP-IDENTITY-22: add privileged identity management scope beyond one-time JIT approval, including role activation, elevation expiry, and reviewer constraints.
23. Slice FP-IDENTITY-23: add terms-of-use and consent enforcement hooks for Entra-style Conditional Access parity.
24. Slice FP-IDENTITY-24: add breach/password legacy handling policy for migration tenants while preserving passkey-first doctrine.
25. Slice FP-IDENTITY-25: add raw API quota/cell quota fixtures per tenant_class so rate-limit and benchmark numbers become reproducible.
