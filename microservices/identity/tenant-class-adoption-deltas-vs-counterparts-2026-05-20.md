# Identity capability availability deltas vs counterparts - 2026-05-20

Citation anchors:
1. Canonical audit and OCI Always Free demo_trial requirement: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-4234`.
2. Master plan tenant_class/platform constraints: `specs/master-plan-sequencing.json:704-889`.
3. Identity PRD and tenant_class expectations: `microservices/identity/PRD.md:1-1642`.
4. Identity architecture and runtime assumptions: `microservices/identity/ARCHITECTURE.md:1-880`.
5. Documentation rigor intern-buildability bar: `docs/standards/documentation-rigor.md:133-139`.

External source anchors:
- Auth0 Organizations: `https://auth0.com/docs/manage-users/organizations/organizations-overview`.
- Auth0 MFA: `https://auth0.com/docs/secure/multi-factor-authentication`.
- Auth0 Inbound SCIM: `https://auth0.com/docs/authenticate/protocols/scim/configure-inbound-scim`.
- Auth0 Attack Protection: `https://auth0.com/docs/secure/attack-protection`.
- Okta MFA authenticators: `https://help.okta.com/oie/en-us/content/topics/identity-engine/authenticators/about-authenticators.htm`.
- Okta FastPass: `https://help.okta.com/oie/en-us/content/topics/identity-engine/devices/fp/fp-main.htm`.
- Okta SCIM: `https://developer.okta.com/docs/concepts/scim/`.
- Okta Identity Governance: `https://help.okta.com/oie/en-us/content/topics/identity-governance/iga.htm`.
- Microsoft Entra Conditional Access: `https://learn.microsoft.com/en-us/entra/identity/conditional-access/concept-conditional-access-policies`.
- Microsoft Entra passkeys/FIDO2: `https://learn.microsoft.com/en-us/entra/identity/authentication/how-to-authentication-passkeys-fido2`.
- Microsoft Entra provisioning: `https://learn.microsoft.com/en-us/entra/identity/app-provisioning/how-provisioning-works`.
- Microsoft Entra Identity Governance: `https://learn.microsoft.com/en-us/entra/id-governance/identity-governance-overview`.

Local tenant_class evidence:
- Current tenant_class adoption matrix: `microservices/identity/ADR-0330 and ADR-0331 tenant_class model:1-181`.
- Current demo_trial hardware envelope: `microservices/identity/ADR-0330 and ADR-0331 tenant_class model:15-24`.
- Current paid with per_seat billing_component tenant_class: `microservices/identity/ADR-0330 and ADR-0331 tenant_class model:55-85`.
- Current paid with per_usage billing_component tenant_class: `microservices/identity/ADR-0330 and ADR-0331 tenant_class model:87-121`.
- Current paid with compliance_pack gating tenant_class: `microservices/identity/ADR-0330 and ADR-0331 tenant_class model:123-148`.
- Current vendor displacement table: `microservices/identity/ADR-0330 and ADR-0331 tenant_class model:162-181`.

## 1. tenant_class definitions in Oyatie identity

01. demo_trial is the smallest supported identity tenant_class and must be usable for dev, sandbox, trial, demo, and small tenant contexts.
02. demo_trial in the current local tenant_class adoption matrix is not small enough for OCI Always Free because it requires four identity-api nodes at 8 vCPU and 32 GiB each plus 500 GiB NVMe each (`ADR-0330 and ADR-0331 tenant_class model:15-24`).
03. demo_trial must be redefined into two subprofiles: demo_trial-General and demo_trial OCI-Always-Free.
04. demo_trial-General can keep a small paid cell envelope.
05. demo_trial OCI-Always-Free must fit 4 OCPU, 24 GiB RAM total, 200 GB block, 10 GB object, 10 GB archive, two 20 GB Autonomous DBs, 10 TB egress/month, and 10 Mbps load balancer (`specs/master-plan-sequencing.json:857-867`).
06. demo_trial authentication scope: OIDC issuer, JWKS, WebAuthn passkey registration/authentication, basic SCIM inbound, tenant admin onboarding, and audit emit.
07. demo_trial risk scope: brute-force mitigation, static rate limits, basic AAGUID allow/block, and manual incident response.
08. demo_trial federation scope: one to three external IdP bindings per tenant, with staged migration support.
09. demo_trial governance scope: no access certification campaigns, no entitlement campaign engine, no broad device posture engine.
10. demo_trial deployment scope: all six contexts must be declared, but only OCI Always Free demo_trial subprofile is bound to Always Free.
11. demo_trial SLO target: OIDC and WebAuthn good enough for sandbox/trial and small paid tenants, not hyperscaler burst.
12. demo_trial context problem: current docs do not declare all six contexts, so demo_trial is not deployable as written.
13. paid with per_seat billing_component is the first paid baseline for production tenants.
14. paid with per_seat billing_component in the current matrix adds 12 API nodes, three regions, cross-region replay, external IdP, HRIS, and multi-context capabilities (`ADR-0330 and ADR-0331 tenant_class model:55-85`).
15. paid with per_seat billing_component authentication scope: OIDC, JWKS, WebAuthn, SCIM, step-up, HRIS, external IdP, and tenant policy.
16. paid with per_seat billing_component risk scope: adaptive rules, admin dashboards, and stronger incident response.
17. paid with per_seat billing_component federation scope: SAML/OIDC enterprise connections, social connections, and HRIS provisioning.
18. paid with per_seat billing_component governance scope: basic request/approval handoff to Governance, but no full access certification engine unless added.
19. paid with per_seat billing_component deployment scope: production in public cloud, guest AWS, guest OCI paid, and qualified on-prem/colo.
20. paid with per_seat billing_component SLO target: production baseline, region redundancy, and 99.99-class issuer availability.
21. paid with per_usage billing_component is the scaled production tenant_class.
22. paid with per_usage billing_component in the current matrix adds 16 nodes per AZ, mandatory hardware passkeys, JIT, continuous risk, bulk SCIM, biometrics, and higher resilience (`ADR-0330 and ADR-0331 tenant_class model:87-121`).
23. paid with per_usage billing_component authentication scope: high-volume token issue, mandatory passkeys for high-risk actions, and hardware-bound ACR gates.
24. paid with per_usage billing_component risk scope: continuous risk scoring, device posture, anomaly detection, and CAEP-style event processing.
25. paid with per_usage billing_component lifecycle scope: bulk SCIM and HRIS workflows at large enterprise scale.
26. paid with per_usage billing_component governance scope: access reviews, SOD checks, JIT approval, and governance evidence should be present to match counterparts.
27. paid with per_usage billing_component deployment scope: multi-region public cloud, guest cloud, on-prem, colo, and provider control-plane contexts.
28. paid with per_usage billing_component SLO target: high production scale, stronger audit commit targets, and regional failover.
29. paid with compliance_pack gating is the hyperscaler/single-tenant-capable tenant_class.
30. paid with compliance_pack gating in the current matrix adds sovereign/HSM/per-pack expectations (`ADR-0330 and ADR-0331 tenant_class model:123-148`).
31. paid with compliance_pack gating authentication scope: dedicated cell identity, single-tenant isolation, HSM-backed signing, sovereign key custody, and high assurance.
32. paid with compliance_pack gating risk scope: continuous risk, hardware-bound credentials, and region/cell-local incident containment.
33. paid with compliance_pack gating lifecycle scope: hyperscaler SCIM/HRIS throughput and exact replay controls.
34. paid with compliance_pack gating governance scope: full access certification, entitlement management, PIM/JIT, SOD, and audit-chain evidence.
35. paid with compliance_pack gating deployment scope: all six contexts, including on-prem, colo, and Oyatie-as-cloud-provider.
36. paid with compliance_pack gating SLO target: issuer 99.999-class and JWKS 99.9995-class availability targets.
37. Current tenant_class defect: no tenant_class definition explicitly names OCI Always Free demo_trial Always Free, despite ADR-0328 requiring every capability-adoption delta doc to state the mapping (`ADR-0328:4211-4212`).
38. Current tenant_class defect: current matrix claims vendor displacement before local OpenTofu deployment and OS support are aligned (`ADR-0330 and ADR-0331 tenant_class model:162-181`).
39. Current tenant_class defect: tenant_class definitions over-index on node counts and under-index on governance/admin capability deltas.
40. Required correction: tiers must describe capability, deployment, OS, IaC, risk, governance, and operational evidence, not only machine count.

## 2. Counterpart tenant_class mapping

### 2.1 Auth0 tenant_class mapping

01. Auth0 entry tenant_class equivalent: Free/Developer-style capability for small apps maps loosely to Oyatie demo_trial OCI or demo_trial-General.
02. Auth0 entry axis: hosted login, user store, basic social/database connections, and constrained rate/usage limits.
03. Auth0 B2B Organizations availability varies by plan, so organization parity belongs at Oyatie paid with per_seat billing_component or higher.
04. Auth0 MFA factor breadth maps to Oyatie paid with per_seat billing_component for conventional factor variety, even though Oyatie passkey-first is stronger on phishing resistance.
05. Auth0 Adaptive MFA and attack protection map to Oyatie paid with per_usage billing_component because Oyatie IP-014 risk scoring is not yet complete.
06. Auth0 Inbound SCIM maps to Oyatie paid with per_seat billing_component because enterprise provisioning should be paid baseline.
07. Auth0 SCIM token rotation and session revocation on deactivate map to Oyatie paid with per_seat billing_component/paid with per_usage billing_component.
08. Auth0 Actions extensibility maps to Oyatie paid with per_usage billing_component unless Cedar/event extension hooks are productized at paid with per_seat billing_component.
09. Auth0 Enterprise tenant_class maps to Oyatie paid with per_usage billing_component for B2B, SAML/OIDC, MFA, SCIM, and admin controls.
10. Auth0 Private Cloud / dedicated deployment maps to Oyatie paid with compliance_pack gating.
11. Auth0 does not map cleanly to Oyatie on-prem/colo sovereignty because Auth0 is primarily hosted/private-cloud, not a six-context portable substrate.
12. Auth0 does not expose Oyatie's audit-chain Merkle/Ed25519 seal as a standard counterpart capability.
13. Auth0 does not expose demo_trial OCI Always Free because that is an Oyatie cost/deployment doctrine.
14. Auth0 gap pressure on Oyatie demo_trial: hosted login polish, conventional factor breadth, and rate-limit UX.
15. Auth0 gap pressure on Oyatie paid with per_seat billing_component: organization admin APIs and inbound SCIM operations.
16. Auth0 gap pressure on Oyatie paid with per_usage billing_component: Actions-equivalent extensibility and adaptive attack/risk response.
17. Auth0 gap pressure on Oyatie paid with compliance_pack gating: dedicated/private-cloud operational evidence.
18. Auth0 counterpart summary: strong hosted CIAM product surface, weaker sovereignty/audit-chain/deployment portability compared with Oyatie ambition.

### 2.2 Okta tenant_class mapping

01. Okta Integrator Free/private integration environment maps loosely to Oyatie demo_trial for development and integration testing.
02. Okta SSO/OIDC/SAML app integrations map to Oyatie paid with per_seat billing_component because enterprise app access is a paid baseline.
03. Okta MFA/authenticator breadth maps to Oyatie paid with per_seat billing_component/paid with per_usage billing_component depending on factor and assurance class.
04. Okta FastPass maps to Oyatie paid with per_usage billing_component because it adds phishing resistance, passwordless public-key authentication, and device posture signals.
05. Okta Lifecycle Management maps to Oyatie paid with per_seat billing_component/paid with per_usage billing_component depending on SCIM complexity.
06. Okta Workflows maps to Oyatie paid with per_usage billing_component if workflow automation is exposed as customer extensibility.
07. Okta Identity Governance maps to Oyatie paid with per_usage billing_component/paid with compliance_pack gating because access certifications, access requests, entitlements, owners, labels, SOD, and reports are enterprise controls.
08. Okta Privileged Access/PAM-style capabilities map to Oyatie paid with per_usage billing_component/paid with compliance_pack gating, beyond the local JIT approval protocol.
09. Okta OIN public integration catalog maps to Oyatie paid with per_usage billing_component product operations.
10. Okta rate-limit dashboard/System Log/header observability maps to Oyatie paid with per_seat billing_component/paid with per_usage billing_component operational controls.
11. Okta does not map cleanly to OCI Always Free or on-prem/colo OpenTofu doctrine.
12. Okta does not provide Oyatie's personal/work dual-context model as a default capability.
13. Okta does not provide Oyatie's sovereign pack matrix as a default hosted capability.
14. Okta gap pressure on Oyatie demo_trial: minimal SSO/MFA polish and rate-limit UX.
15. Okta gap pressure on Oyatie paid with per_seat billing_component: lifecycle and SCIM completeness.
16. Okta gap pressure on Oyatie paid with per_usage billing_component: FastPass-like device posture and OIN-like integration catalog.
17. Okta gap pressure on Oyatie paid with compliance_pack gating: Identity Governance, entitlements, SOD, access certifications, and privileged access.
18. Okta counterpart summary: strongest on workforce lifecycle/governance breadth; Oyatie is strongest on sovereignty/audit ambitions but not current evidence.

### 2.3 Microsoft Entra ID tenant_class mapping

01. Microsoft Entra ID Free maps to Oyatie demo_trial for basic authentication and passkey availability.
02. Microsoft Entra ID P1 maps to Oyatie paid with per_seat billing_component for Conditional Access, SSO, MFA, and provisioning baselines.
03. Microsoft Entra ID P2 maps to Oyatie paid with per_usage billing_component for Identity Protection, risk-based Conditional Access, and privileged controls.
04. Microsoft Entra ID Governance maps to Oyatie paid with per_usage billing_component/paid with compliance_pack gating for lifecycle workflows, entitlement management, access packages, and access reviews.
05. Microsoft Entra External ID maps to Oyatie paid with per_seat billing_component/paid with per_usage billing_component for customer and partner identity flows.
06. Microsoft Entra Workload ID maps to Oyatie paid with per_seat billing_component/paid with per_usage billing_component service principal and workload principal surfaces.
07. Microsoft Entra passkey profiles map to Oyatie paid with per_usage billing_component for AAGUID restrictions, attestation, and group-targeted credential policy.
08. Microsoft Entra Conditional Access maps to Oyatie paid with per_usage billing_component because it includes many policy signals and access controls.
09. Microsoft Entra PIM maps to Oyatie paid with per_usage billing_component/paid with compliance_pack gating; Identity only has JIT IT approval currently.
10. Microsoft Entra provisioning maps to Oyatie paid with per_seat billing_component/paid with per_usage billing_component for SCIM and on-prem connectors.
11. Microsoft Entra ID Protection maps to Oyatie paid with per_usage billing_component because local IP-014 is deferred.
12. Microsoft Entra service limits/throttling map to Oyatie paid with per_seat billing_component/paid with per_usage billing_component operational maturity.
13. Microsoft Entra sovereign cloud/private deployment maps partially to Oyatie paid with compliance_pack gating, but not all Entra capabilities are self-hosted/colo portable.
14. Microsoft does not map to demo_trial OCI Always Free.
15. Microsoft does not map to Rust-strict local backend doctrine.
16. Entra gap pressure on Oyatie demo_trial: passkey profile and registration policy clarity.
17. Entra gap pressure on Oyatie paid with per_seat billing_component: Conditional Access and provisioning breadth.
18. Entra gap pressure on Oyatie paid with per_usage billing_component: Identity Protection, risk, PIM, workload identity, and governance.
19. Entra gap pressure on Oyatie paid with compliance_pack gating: hyperscaler-grade SLA, resilience, and sovereign cloud evidence.
20. Entra counterpart summary: strongest on integrated enterprise identity, risk, governance, and Microsoft ecosystem access; Oyatie must catch up on governance/risk and prove deployment portability.

## 3. Per-Oyatie-tenant_class delta tables

### 3.1 demo_trial tenant_class table

| Feature | Oyatie demo_trial | Auth0 equivalent | Okta equivalent | Entra equivalent | Gap classification |
|---|---|---|---|---|---|
| OIDC issuer | present by contract | present | present | present | parity |
| JWKS endpoint | present by contract/ADR | present | present | present | parity |
| OAuth PKCE | implied | present | present | present | partial |
| Hosted login UI | not documented | strong | strong | strong | catch-up |
| Basic WebAuthn | present | present | present | present | parity |
| Passkey-first policy | present | partial | present | present | ahead/parity |
| Recovery codes | recovery envelope | present | present | present | ahead/different |
| SMS MFA | rejected/absent | present | present | present | conscious gap |
| TOTP MFA | partial | present | present | present | partial |
| Email MFA | partial | present | present | present | partial |
| Push MFA | absent | present | present | present | gap |
| Inbound SCIM | present | present | present | present | parity |
| Outbound SCIM | absent/deferred | partial | present | present | gap |
| Organization roles | partial | present | present | present | partial |
| Organization admin API | absent | present | present | present | gap |
| Basic attack protection | partial runbooks | present | present | present | partial |
| Rate-limit admin UI | absent | present | present | present | gap |
| Basic audit events | present | logs | System Log | audit logs | parity |
| Audit-chain seal | intended | absent/partial | absent/partial | absent/partial | additive |
| Tenant admin onboarding | runbook present | present | present | present | partial |
| OCI Always Free | not met | none | none | none | Oyatie self-gap |
| OpenTofu six contexts | absent | not applicable | not applicable | not applicable | Oyatie self-gap |
| OS support manifest | absent | not comparable | not comparable | not comparable | Oyatie self-gap |
| Rust-strict backend | current corpus clean | not comparable | not comparable | not comparable | additive |
| Device posture | absent | partial | FastPass | Conditional Access | gap |
| Identity governance | absent | partial | present | present | gap |
| Access reviews | absent | partial | present | present | gap |
| Entitlement management | absent | partial | present | present | gap |
| Workload identities | thin | machine-to-machine | API services | Workload ID | partial |
| Sovereign/offline | docs partial | no | no | partial | additive but blocked |
| Raw benchmark evidence | absent | not universal | not universal | not universal | self-gap |
| Readiness verdict | useful auth substrate docs | mature hosted CIAM | mature workforce CIAM | mature enterprise ID | demo_trial is not shippable until OCI/IaC/OS fixed |

### 3.2 paid with per_seat billing_component tenant_class table

| Feature | Oyatie paid with per_seat billing_component | Auth0 equivalent | Okta equivalent | Entra equivalent | Gap classification |
|---|---|---|---|---|---|
| Enterprise OIDC federation | present | present | present | present | parity |
| Enterprise SAML federation | present | present | present | present | parity |
| Social IdPs | named | present | partial | partial | partial |
| HRIS adapter | IP present | partial | present | present | parity/partial |
| SCIM inbound | present | present | present | present | parity |
| SCIM group lifecycle | present | present | present | present | parity |
| SCIM outbound | deferred | partial | present | present | gap |
| Tenant policy isolation | Cedar | present | present | present | parity |
| Delegated organization admin | absent | present | present | present | gap |
| Organization branding | partial Helm overlay | present | present | present | partial |
| MFA factor breadth | passkey-heavy | broad | broad | broad | partial |
| Adaptive MFA/risk | deferred | present | present | present | catch-up |
| Step-up ACR | present | present | present | present | parity |
| JIT approval | present | partial | partial | PIM-related | partial/parity |
| Device posture | absent | partial | FastPass | CA device filters | gap |
| Rate-limit policy | partial | present | present | present | partial |
| Rate-limit dashboard | absent | present | present | present | gap |
| Event/log export | AsyncAPI | present | present | present | parity |
| Cross-region replay | listed | private-cloud | Okta infra | Entra infra | partial |
| Public cloud context | not IaC-backed | hosted | hosted | hosted | self-gap |
| Guest AWS context | not IaC-backed | no | no | no | self-gap |
| Guest OCI context | not IaC-backed | no | no | no | self-gap |
| On-prem context | not IaC-backed | no | partial agent | partial agent | self-gap |
| Colo context | not IaC-backed | no | no | partial | self-gap |
| Provider context | not IaC-backed | no | no | Microsoft-owned | self-gap |
| Supported OS matrix | absent | SaaS abstraction | SaaS abstraction | SaaS abstraction | self-gap |
| Access requests | absent | partial | present | present | gap |
| Access certifications | absent | partial | present | present | gap |
| Entitlement management | absent | partial | present | present | gap |
| Integration catalog | absent | marketplace/docs | OIN | gallery | gap |
| Admin console | dashboard JSON only | present | present | present | gap |
| Readiness verdict | strong protocol docs but platform gaps | mature CIAM | mature WIC | mature enterprise-market substrate | paid with per_seat billing_component is catch-up until admin/governance/deploy evidence lands |

### 3.3 paid with per_usage billing_component tenant_class table

| Feature | Oyatie paid with per_usage billing_component | Auth0 equivalent | Okta equivalent | Entra equivalent | Gap classification |
|---|---|---|---|---|---|
| Mandatory hardware passkeys | listed | enterprise MFA | strong auth | passkey profiles | parity |
| AAGUID allow/block | present | partial | partial | present | parity/ahead |
| FIDO MDS refresh | IP present | partial | present | present | partial |
| Continuous risk scoring | deferred IP-014 | present | present | present | catch-up |
| Identity risk score | absent | partial | partial | present | gap |
| Risk-based policy | partial Cedar | present | present | present | catch-up |
| Device posture | absent | partial | present | present | gap |
| Bulk SCIM | listed | partial | present | present | partial |
| SCIM retry semantics | partial | present | present | present | partial |
| HRIS high-volume | partial | partial | present | present | catch-up |
| Access certification campaigns | absent | partial | present | present | gap |
| Access request workflows | governance handoff | partial | present | present | gap |
| Entitlement packages | absent | partial | present | present | gap |
| Separation-of-duties rules | JIT narrow | partial | present | present | partial |
| Privileged identity mgmt | JIT only | partial | partial | present | catch-up |
| Workflow extensibility | events/Cedar partial | Actions | Workflows | Graph/Logic Apps | catch-up |
| Integration gallery | absent | docs/marketplace | OIN | gallery | gap |
| Tenant quota admin | absent | present | present | present | gap |
| Operational dashboards | Grafana JSON | tenant UI | admin UI | admin center | partial |
| Benchmarks raw artifacts | absent | vendor internal | vendor internal | vendor internal | self-gap |
| Multi-region issuer | listed | private-cloud | hosted | hosted | partial |
| OIDC 99.99 | PRD target | SLA-backed | SLA-backed | SLA-backed | target-only |
| JWKS 99.999 | PRD target | cache-backed | cache-backed | cache-backed | target-only |
| Audit-chain p99 | target-only | no | no | no | additive unmeasured |
| All six OpenTofu contexts | absent | no | no | no | self-gap |
| OS CI lanes | absent | SaaS abstraction | SaaS abstraction | SaaS abstraction | self-gap |
| Sovereign pack | docs partial | private cloud limited | limited | sovereign cloud | partial |
| Air-gap | docs partial | no | no | partial | additive but blocked |
| Workload identity | thin | M2M | API service | Workload ID | partial |
| Readiness verdict | not paid with per_usage billing_component-ready | mature enterprise | mature enterprise/governance | mature enterprise/governance/risk | paid with per_usage billing_component is catch-up on risk/governance/admin/deploy |

### 3.4 paid with compliance_pack gating tenant_class table

| Feature | Oyatie paid with compliance_pack gating | Auth0 equivalent | Okta equivalent | Entra equivalent | Gap classification |
|---|---|---|---|---|---|
| Single-tenant capable cell | intended | private cloud/dedicated | dedicated enterprise patterns | sovereign/dedicated cloud patterns | target-only |
| Sovereign HSM signing | intended | enterprise/private | enterprise | present in Microsoft ecosystem | partial |
| Per-pack key custody | intended | limited | limited | sovereign cloud partial | additive |
| On-prem deployment | no IaC | no | agents only | agents/limited | self-gap |
| Colo deployment | no IaC | no | no | limited | self-gap |
| Oyatie provider identity | no IaC | no | no | Microsoft-owned provider | self-gap |
| OpenTofu signed modules | absent | not comparable | not comparable | not comparable | self-gap |
| State backend per context | absent | not comparable | not comparable | not comparable | self-gap |
| OS tenant_class-1 matrix | absent | SaaS abstraction | SaaS abstraction | SaaS abstraction | self-gap |
| Hyperscaler token throughput | target-only | vendor internal | vendor internal | vendor internal | unmeasured |
| Hyperscaler SCIM throughput | target-only | vendor internal | vendor internal | vendor internal | unmeasured |
| Audit-chain seal | intended | no standard | no standard | no standard | additive |
| Audit-chain regional p99 | target-only | no standard | no standard | no standard | unmeasured |
| Identity governance complete | absent | partial | present | present | gap |
| Privileged identity complete | JIT only | partial | partial | present | catch-up |
| Risk protection complete | deferred | present | present | present | catch-up |
| Device posture complete | absent | partial | present | present | gap |
| Integration marketplace | absent | present | present | present | gap |
| Customer admin console | absent | present | present | present | gap |
| Legal/regulatory packs | strong docs | limited | limited | strong in Microsoft clouds | parity/partial |
| KR/KSA/UAE overlays | docs partial | no | no | partial | additive |
| Air-gapped operations | intended | no | no | partial | additive but blocked |
| 24h isolated hot path | chat doctrine | no public equivalent | no public equivalent | partial resilience | additive |
| Personal/work boundary | intended | no | partial | partial | additive |
| Survivor/minor/emergency flows | journey IPs | partial | partial | partial | additive |
| Workload/CI principal doctrine | chat doctrine | M2M partial | API partial | Workload ID | partial |
| Certification-ready evidence | incomplete | mature | mature | mature | catch-up |
| Benchmark raw evidence | absent | vendor internal | vendor internal | vendor internal | self-gap |
| Readiness verdict | not paid with compliance_pack gating-ready | mature hosted/dedicated | mature workforce/governance | mature enterprise/hyperscaler | paid with compliance_pack gating is architectural ambition, not evidence-backed yet |

## 4. OCI Always Free demo_trial = Always Free reconciliation

01. ADR-0328 and the master plan require OCI Always Free demo_trial to map to OCI Always Free for demo, sandbox, trial, and dev tenants (`specs/master-plan-sequencing.json:857-867`).
02. The current tenant_class adoption matrix does not state OCI Always Free demo_trial equals Always Free.
03. The current demo_trial hardware envelope is incompatible with Always Free.
04. Current demo_trial identity-api nodes alone require 32 vCPU and 128 GiB RAM across four nodes (`ADR-0330 and ADR-0331 tenant_class model:15-24`).
05. OCI Always Free allows 4 OCPU and 24 GiB RAM total for Ampere A1, plus two AMD E2.1.Micro instances.
06. Current demo_trial storage asks for 500 GiB NVMe per node, which exceeds the 200 GB block budget before databases or logs.
07. Current demo_trial supporting services include Postgres, Citus, Valkey, Kafka, OpenBao, and Zitadel.
08. OCI Always Free storage and database limits cannot host that entire demo_trial-General stack without reduction.
09. Required demo_trial OCI profile: one compact identity API replica plus one warm standby only if capacity permits.
10. Required demo_trial OCI profile: Zitadel deployment must be reduced or replaced with a compact local mode compatible with 4 OCPU.
11. Required demo_trial OCI profile: Postgres/Citus cannot run as full Citus cluster; use one small relational instance or Autonomous DB within free allocation.
12. Required demo_trial OCI profile: Valkey/Kafka must be disabled, embedded, or backed by minimal compatible free resources.
13. Required demo_trial OCI profile: audit events must buffer within capped storage and export when paid storage is selected.
14. Required demo_trial OCI profile: JWKS and token verify paths must be aggressively cacheable.
15. Required demo_trial OCI profile: SCIM write throughput must be capped and backpressure-friendly.
16. Required demo_trial OCI profile: no cross-cloud spillover is allowed to hide capacity breach (`ADR-0328:1875-1880`).
17. Required demo_trial OCI profile: `iac/oci-guest/always-free/` must include `main.tf`, `variables.tf`, `outputs.tf`, `versions.tf`, README, budget outputs, and state backend wiring.
18. Required demo_trial OCI profile: capability tenant_class adoption matrix must state which features are disabled, capped, or paid with per_seat billing_component on OCI.
19. Feature requiring paid with per_seat billing_component on OCI: large SCIM bulk import.
20. Feature requiring paid with per_seat billing_component on OCI: multi-region high availability.
21. Feature requiring paid with per_seat billing_component on OCI: Kafka-scale event streaming.
22. Feature requiring paid with per_seat billing_component on OCI: Citus/distributed identity database.
23. Feature requiring paid with per_seat billing_component on OCI: high-volume HRIS sync.
24. Feature requiring paid with per_seat billing_component on OCI: continuous risk scoring at high ingest.
25. Feature requiring paid with per_seat billing_component on OCI: dedicated HSM-backed signing beyond available free resources.
26. Feature requiring paid with per_seat billing_component on OCI: large tenant counts beyond five small tenants.
27. Feature requiring paid with per_seat billing_component on OCI: paid with compliance_pack gating-grade audit-chain regional p99.
28. Feature still demo_trial OCI: OIDC discovery/JWKS/token issue within capped throughput.
29. Feature still demo_trial OCI: WebAuthn registration/authentication within capped throughput.
30. Feature still demo_trial OCI: basic inbound SCIM with low write limits.
31. Feature still demo_trial OCI: basic tenant admin and passkey recovery envelope.
32. Feature still demo_trial OCI: audit emit with capped local retention.
33. Feature still demo_trial OCI: local Cedar ACR enforcement.
34. Feature still demo_trial OCI: one or two external IdP bindings.
35. Reconciliation result: demo_trial-General and demo_trial OCI must be separate rows in `tenant_class-matrix.md`.
36. Reconciliation result: current tenant_class adoption matrix is P0-incoherent with OCI Always Free doctrine.
37. Reconciliation result: no OCI Always Free demo_trial readiness claim should pass until the module and tenant_class row exist.
38. Reconciliation result: Wave 14 should aggregate this as a canonical tenant_class correction, not just identity-local prose cleanup.

## 5. Findings by tenant_class

01. demo_trial ahead: passkey-first and recovery-envelope design are stronger than generic username/password-first identity tiers.
02. demo_trial ahead: audit-chain event model is stronger than flat log-only counterpart assumptions.
03. demo_trial parity: OIDC, JWKS, WebAuthn, and inbound SCIM are present as contracts.
04. demo_trial catch-up: hosted login/admin UI, conventional MFA breadth, organization admin, and rate-limit dashboard are missing.
05. demo_trial self-gap: OCI Always Free, OpenTofu modules, and OS manifest are absent.
06. demo_trial classification: not ready because canonical deployment gates fail.
07. paid with per_seat billing_component ahead: per-pack residency and Cedar policy are stronger than many hosted CIAM defaults.
08. paid with per_seat billing_component parity: enterprise SAML/OIDC federation, inbound SCIM, step-up, and HRIS adapter are substantially designed.
09. paid with per_seat billing_component catch-up: outbound SCIM, delegated org admin, admin console, and tenant rate-limit UX remain incomplete.
10. paid with per_seat billing_component self-gap: production context modules are absent for public cloud, guest AWS, guest OCI, on-prem, colo, and provider context.
11. paid with per_seat billing_component classification: protocol-rich but platform-incomplete.
12. paid with per_usage billing_component ahead: personal/work dual-context identity and survivor/minor/emergency journeys are richer than standard counterpart feature lists.
13. paid with per_usage billing_component parity: hardware-bound passkeys, AAGUID policy, JIT approval, and high-assurance claims are in the design.
14. paid with per_usage billing_component catch-up: continuous risk scoring, device posture, governance campaigns, and integration gallery are behind Okta and Entra.
15. paid with per_usage billing_component self-gap: benchmark raw evidence and per-context CI lanes are missing.
16. paid with per_usage billing_component classification: architectural intent present, enterprise-operational parity incomplete.
17. paid with compliance_pack gating ahead: sovereign pack, audit-chain seal, air-gap, and provider-control-plane identity are differentiators.
18. paid with compliance_pack gating parity: HSM/OpenBao key custody intent aligns with enterprise/dedicated counterpart controls.
19. paid with compliance_pack gating catch-up: no access governance, PIM equivalent, device posture, admin console, or integration catalog at counterpart depth.
20. paid with compliance_pack gating self-gap: no OpenTofu context modules, no OS manifest, no raw benchmark evidence, no OCI Always Free mapping.
21. paid with compliance_pack gating classification: not evidence-backed; it remains a target architecture.
22. Cross-tenant_class finding P0: demo_trial OCI Always Free contradiction blocks valid tenant_class claims.
23. Cross-tenant_class finding P0: missing OpenTofu modules block every deployable tenant_class.
24. Cross-tenant_class finding P0: missing OS manifest blocks every deployable tenant_class.
25. Cross-tenant_class finding P1: competitor displacement claims should be downgraded until governance/risk/admin/deploy evidence exists.
26. Cross-tenant_class finding P1: tenant_class adoption matrix should describe user/admin/workload identity capabilities, not only machine envelopes.
27. Cross-tenant_class finding P1: local benchmark targets must be separated from measured evidence.
28. Cross-tenant_class finding P2: social provider matrix should specify which providers ship in each tenant_class.
29. Cross-tenant_class finding P2: HRIS connector scope should specify Workday, BambooHR, Rippling, and regional equivalents by tenant_class.
30. Cross-tenant_class finding P2: recovery envelope limits should state per-tenant_class storage and ceremony throughput.
31. Cross-tenant_class finding P2: audit retention should state per-tenant_class retention, export, and seal cadence.
32. Cross-tenant_class finding P2: JIT approval should state which tiers include two-person rule enforcement.
33. Cross-tenant_class finding P3: developer SDK support should be tiered and Rust-strict.
34. Cross-tenant_class finding P3: dashboard JSON exists but no customer admin UI surface is tiered.
35. Cross-tenant_class finding P3: Kustomize regulatory overlays should be nested under canonical OpenTofu context modules.
36. Final tenant_class verdict: Oyatie Identity is conceptually ahead on sovereignty, audit, recovery, and dual-context identity, but behind on counterpart governance/risk/admin operations and behind its own ADR-0328 deployment doctrine.

