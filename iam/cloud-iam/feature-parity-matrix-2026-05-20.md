# cloud-iam feature parity matrix - 2026-05-20

## Citation anchor block

1. Canonical service direction: `docs/decisions/ADR-0700-ci-admission-live-apex.md` D-15 through D-20, plus `docs/standards/brief-template.md` lines 746-758 requiring `cloud-iam` across all six contexts.
2. Machine-readable direction: `specs/master-plan-sequencing.json` deployment contexts lines 704-745, OpenTofu substrate lines 747-775, supported OSes lines 777-815, language policy lines 817-855, and OCI Always Free lines 856-867.
3. Service-local purpose: ADR-0329 + ADR-0330 + ADR-0331 lines 7-91, `iam/cloud-iam/faqs/iam-engineer-faq.md` lines 7-182, and `iam/cloud-iam/reference-implementations/issue-scoped-token-rust-sdk.md` lines 1-182.
4. Product-level cloud contract: `docs/products/cloud/PRD.md` lines 117-120 and 172-204; historical coverage inventory at `docs/DOC-COVERAGE.md` lines 130-136 is provenance only and is not current product-state evidence.
5. Counterpart sources: AWS IAM docs at `https://docs.aws.amazon.com/IAM/latest/UserGuide/introduction.html`, AWS IAM Identity Center docs at `https://docs.aws.amazon.com/en_en/singlesignon/latest/userguide/what-is.html`, Google Cloud IAM docs at `https://docs.cloud.google.com/iam/docs/overview`, and Microsoft Entra docs at `https://learn.microsoft.com/en-us/entra/fundamentals/what-is-entra`.

## 1. Counterpart 1 - AWS IAM With IAM Identity Center Capability Surface

AWS-01 Authentication and authorization for AWS resources, including who is signed in and what permissions are granted, per AWS IAM introduction lines 7-17.
AWS-02 IAM users for account-local identities, though AWS best practice now pushes workforce humans toward federation.
AWS-03 IAM groups for attaching permissions to sets of IAM users.
AWS-04 IAM roles as assumable identities with temporary credentials rather than long-term passwords or access keys.
AWS-05 Role trust policies controlling which principals can assume a role.
AWS-06 AWS STS temporary credential issuance for `AssumeRole`, `GetSessionToken`, `AssumeRoleWithSAML`, and `AssumeRoleWithWebIdentity`.
AWS-07 Federated access through SAML or OIDC into AWS role assumption.
AWS-08 IAM Identity Center workforce SSO to AWS accounts and applications, per Identity Center lines 7-16.
AWS-09 Identity Center identity source connection and user/group sync from an external IdP or the built-in identity store.
AWS-10 Identity Center permission sets for reusable AWS account access templates.
AWS-11 Identity Center multi-account assignment of users/groups to permission sets.
AWS-12 Identity Center AWS access portal for assigned accounts and applications.
AWS-13 Identity Center trusted identity propagation across AWS managed applications, per Identity Center lines 25-35.
AWS-14 Identity Center organization instances as production recommended deployment shape.
AWS-15 Identity Center account instances for isolated select application deployments.
AWS-16 IAM identity-based policies attached to users, groups, or roles.
AWS-17 IAM resource-based policies attached to resources.
AWS-18 Permissions boundaries that cap delegated IAM role/user permissions.
AWS-19 AWS Organizations service control policies as organization-level guardrails.
AWS-20 AWS Organizations resource control policies as resource-side guardrails.
AWS-21 Session policies that further scope temporary sessions.
AWS-22 Session tags and ABAC context; IAM quota docs allow up to 50 session tags.
AWS-23 Policy conditions such as TLS, source, MFA, tags, service path, and request attributes.
AWS-24 AWS managed policies for common job functions.
AWS-25 Customer managed policies and inline policies.
AWS-26 IAM policy validation through IAM Access Analyzer.
AWS-27 IAM Access Analyzer external and cross-account access findings.
AWS-28 IAM Access Analyzer policy generation from CloudTrail activity, per AWS best-practices lines 82-84.
AWS-29 IAM last-accessed information to remove unused users, roles, policies, and credentials.
AWS-30 Root-user protection and root-user best-practice workflow.
AWS-31 MFA requirements and phishing-resistant MFA guidance, per AWS best-practices lines 59-62.
AWS-32 Long-term access key rotation guidance for exceptional cases.
AWS-33 Service-linked roles owned by AWS services.
AWS-34 Service roles for workloads and services.
AWS-35 IAM Roles Anywhere for X.509-based temporary credentials outside AWS, per AWS best-practices lines 46-48.
AWS-36 Workload temporary credentials delivered by EC2, Lambda, ECS, EKS, and related compute services.
AWS-37 Cross-account role delegation and external ID support.
AWS-38 IAM account aliases and naming limits.
AWS-39 IAM quota and object limit visibility through `GetAccountSummary`.
AWS-40 IAM resource quotas including roles per account, policies per account, OIDC providers per account, and managed policies per role.
AWS-41 STS quota of 600 requests per second per account per Region for credentialed STS operations, per AWS IAM quota lines 70-95.
AWS-42 Role session duration up to 12 hours, per AWS IAM quota lines 132-136.
AWS-43 SAML response size and policy character limits.
AWS-44 CloudTrail logging of IAM/STS/account actions.
AWS-45 Multi-region access for Identity Center accounts and applications, per Identity Center lines 45-47.
AWS-46 Application access in Identity Center for AWS managed and customer managed applications.
AWS-47 OAuth 2.0 application integration with Identity Center for custom applications.
AWS-48 Certificate rotation for SAML application integrations.
AWS-49 Attribute mappings from IdP to Identity Center/application attributes.
AWS-50 Least-privilege reduction guidance backed by Access Analyzer and managed-policy transition.
AWS-51 Guardrail separation between permissions that grant access and policies that bound access.
AWS-52 Eventual consistency warning for IAM changes; AWS introduction lines 24-26 warns not to place IAM mutations on critical high-availability paths.
AWS-53 No-additional-charge baseline for IAM, Identity Center, and STS, per AWS introduction lines 27-30.
AWS-54 Integration across many AWS services; AWS IAM is embedded in the resource authorization layer.
AWS-55 Confused-deputy mitigations through external IDs and trust conditions.

## 2. Counterpart 2 - Google Cloud IAM Capability Surface

GCP-01 Fine-grained authorization for Google Cloud resources, expressed as who can do what on which resources, per Google IAM overview lines 341-353.
GCP-02 Principals representing human users and workloads, per Google IAM overview lines 357-370.
GCP-03 Google Accounts as human principals.
GCP-04 Google Groups as principal collections.
GCP-05 Cloud Identity and Google Workspace domain principals.
GCP-06 Federated workforce identities in workforce identity pools.
GCP-07 Workload service accounts as workload principals.
GCP-08 Federated workload identities in workload identity pools.
GCP-09 Resource hierarchy inheritance through organizations, folders, projects, and service resources.
GCP-10 Basic roles for broad project-level access.
GCP-11 Predefined roles managed by Google Cloud services.
GCP-12 Custom roles at organization and project level.
GCP-13 Permission reference and grantable role discovery.
GCP-14 Allow policies attached to resources, with role bindings and inheritance.
GCP-15 Deny policies attached to projects/folders/organizations, with up to 500 deny policies per resource from quota docs.
GCP-16 Principal access boundary policies restricting resources a principal is eligible to access.
GCP-17 Access policies for Eventarc resources.
GCP-18 Simultaneous policy evaluation across allow, deny, and principal access boundary policies, per policy-types lines 419-438.
GCP-19 IAM Conditions for conditional role bindings.
GCP-20 Temporary access configuration for time-limited grants.
GCP-21 Privileged Access Manager for just-in-time elevated access.
GCP-22 PAM entitlement and grant lifecycle.
GCP-23 PAM audit of entitlement and grant events.
GCP-24 Service accounts for applications and compute workloads, per service-account overview lines 341-348.
GCP-25 Service account impersonation.
GCP-26 Service account keys, with explicit security-risk caution.
GCP-27 Domain-wide delegation for service accounts acting as Workspace/Cloud Identity users.
GCP-28 Workload Identity Federation for on-premises and multicloud workloads without service account keys.
GCP-29 Workload Identity Federation providers for AWS, Microsoft Entra ID, GitHub, GitLab, Kubernetes, Okta, AD FS, Terraform, OIDC, and SAML, per workload federation lines 341-365.
GCP-30 Workforce Identity Federation for employee/partner/contractor SSO through external IdPs, per workforce federation lines 341-361.
GCP-31 Syncless workforce federation that avoids copying users into Google-managed accounts.
GCP-32 Attribute-based access from IdP claims/assertions.
GCP-33 OAuth/OIDC and SAML protocol support for workforce federation.
GCP-34 Service Account Credentials API for short-lived credentials.
GCP-35 Security Token Service API for token exchange and introspection.
GCP-36 IAM API v1/v2/v3 split for allow/custom roles, deny policies, and principal access boundary policies.
GCP-37 Quotas for IAM v1 read/write requests, including 6,000 reads and 600 writes per project per minute.
GCP-38 Quotas for Workload Identity Federation reads/writes and client-based quotas.
GCP-39 Quotas for Security Token Service exchange/introspection requests.
GCP-40 Limits for custom roles, including 300 custom roles per organization and project.
GCP-41 Limits for allow policy principals, including 1,500 principals in a single policy.
GCP-42 Limits for deny policies and deny rules.
GCP-43 Limits for principal access boundary rules, resources, and bound policies.
GCP-44 Service account key limits and service account naming limits.
GCP-45 Access boundary rules and OAuth access token lifetime limits.
GCP-46 IAM policy history review.
GCP-47 Security insights and role recommendations.
GCP-48 Policy Analyzer for resource access analysis.
GCP-49 Policy Simulator for proposed access changes.
GCP-50 Policy Troubleshooter for permission errors.
GCP-51 Linting for conditional role bindings.
GCP-52 Resource-based access configuration.
GCP-53 Tags and conditional access.
GCP-54 IAM Recommender/Gemini-assisted predefined role suggestions.
GCP-55 Best-practice guidance for groups and service account keys.
GCP-56 Cloud Audit Logs integration for IAM and admin activity.
GCP-57 Integration with Google Cloud console, `gcloud`, REST, and client libraries.
GCP-58 Direct access and service-account impersonation modes for workload federation.
GCP-59 Organization-level federation pools and policy bindings.
GCP-60 Eventual propagation behavior and troubleshooting guides for IAM errors.

## 3. Counterpart 3 - Microsoft Entra ID Capability Surface

ENTRA-01 Microsoft Entra ID as cloud identity and access management for authentication, policy enforcement, and protection across users, devices, apps, and resources, per Entra docs lines 37-41.
ENTRA-02 Tenant and custom-domain identity model.
ENTRA-03 User identity management.
ENTRA-04 Group identity management.
ENTRA-05 Device identity management.
ENTRA-06 Application registrations.
ENTRA-07 Enterprise applications.
ENTRA-08 Service principals.
ENTRA-09 Managed identities for Azure resources.
ENTRA-10 Workload identities for applications, services, and containers, per Entra docs lines 79-84.
ENTRA-11 External ID for business partners, guests, and consumer applications, per Entra docs lines 73-78.
ENTRA-12 B2B guest collaboration.
ENTRA-13 Customer identity and access management.
ENTRA-14 SAML/OIDC/OAuth application federation through Microsoft identity platform.
ENTRA-15 OpenID and OAuth token issuance.
ENTRA-16 Microsoft Graph API for administration, user lifecycle, and governance automation, per Entra docs lines 111-115.
ENTRA-17 Microsoft Entra admin center for unified administration.
ENTRA-18 Conditional Access as Zero Trust policy engine using user, group, agent, IP, device, application, and risk signals, per Conditional Access lines 32-70.
ENTRA-19 Conditional Access decisions to block, grant, require MFA, require authentication strength, require compliant device, require hybrid joined device, require app protection, require password change, or require terms of use.
ENTRA-20 Conditional Access report-only mode and coverage view.
ENTRA-21 Security defaults for baseline protections.
ENTRA-22 MFA and authentication strengths.
ENTRA-23 Passwordless/passkey support through authentication methods.
ENTRA-24 Identity Protection risk detection and risk-based Conditional Access.
ENTRA-25 Privileged Identity Management for time-based and approval-based privileged role activation, per PIM lines 34-56.
ENTRA-26 PIM eligible and active assignments.
ENTRA-27 PIM activation with MFA, justification, approval, time bounds, and audit history.
ENTRA-28 PIM for Microsoft Entra roles.
ENTRA-29 PIM for Azure resource roles.
ENTRA-30 PIM for groups.
ENTRA-31 Access reviews for groups, apps, privileged roles, and guest access.
ENTRA-32 Entitlement management and access packages.
ENTRA-33 Lifecycle workflows for joiner/mover/leaver automation.
ENTRA-34 HR-driven inbound provisioning from Workday and SuccessFactors.
ENTRA-35 Application provisioning using SCIM, LDAP, SQL, and app connectors, per governance lines 49-53.
ENTRA-36 Dynamic groups and attribute-driven access assignment.
ENTRA-37 Automatic assignment policies.
ENTRA-38 Governance dashboard.
ENTRA-39 Audit and sign-in logs.
ENTRA-40 Identity governance for external users and guest lifecycle, per governance lines 56-59.
ENTRA-41 Separation-of-duties checks in entitlement management.
ENTRA-42 Recurring access recertification with reviewer workflow.
ENTRA-43 Agent identity governance preview, including sponsor and blueprint models, per governance lines 112-119.
ENTRA-44 Verified ID and decentralized identity credentials, per Entra docs lines 69-72.
ENTRA-45 Domain Services for managed LDAP/Kerberos/NTLM legacy integration, per Entra docs lines 42-46.
ENTRA-46 Private Access for zero-trust access to private applications and multicloud/internal resources.
ENTRA-47 Internet Access for SaaS and internet resource access controls.
ENTRA-48 External collaboration settings.
ENTRA-49 Administrative units and delegated administration.
ENTRA-50 Custom security attributes.
ENTRA-51 Continuous Access Evaluation.
ENTRA-52 Configurable token lifetime policy constraints and default access-token lifetime variability.
ENTRA-53 Microsoft Graph throttling with global limits such as 130,000 requests per 10 seconds per app across tenants.
ENTRA-54 Directory service limits and restrictions for tenant resources, groups, devices, applications, and service principals.
ENTRA-55 National cloud and sovereign-cloud variants.
ENTRA-56 Licensing tenant_class policies: Free, P1, P2, Suite, External ID, Workload ID, ID Governance, and standalone products.
ENTRA-57 Security Copilot integration for identity risk investigation and access troubleshooting.
ENTRA-58 Device compliance integration through Intune.
ENTRA-59 Microsoft Defender for Cloud Apps integration for session control.
ENTRA-60 Developer identity platform SDK and protocol ecosystem.

## 4. Union-Coverage Matrix

| Capability | AWS IAM + IdC | Google Cloud IAM | Microsoft Entra ID | Union required | Oyatie cloud-iam has | Gap classification |
|---|---|---|---|---|---|---|
| Human principal model | Yes | Yes | Yes | Yes | Partial: `User` documented in FAQ lines 21-25 | Build contract missing |
| Workload principal model | Yes | Yes | Yes | Yes | Yes: workload/SPIFFE in FAQ lines 21-25 and 129-132 | Needs schema/tests |
| Service account model | Yes | Yes | Yes | Yes | Yes: tenant_class policy lines 17-20 | Needs data model |
| Federated identity model | Yes | Yes | Yes | Yes | Yes: SAML/OIDC tutorial and FAQ lines 30-34 | Needs contract |
| Group model | Yes | Yes | Yes | Yes | Partial: IdP groups map to Cedar roles | Native group lifecycle missing |
| Role model | Yes | Yes | Yes | Yes | Yes: tenant_class policy and migration docs | Needs role API |
| Policy model | Yes JSON | Yes allow/deny/PAB | Yes Conditional Access/RBAC | Yes | Yes Cedar | Needs schema and simulator |
| Policy conditions | Yes | Yes | Yes | Yes | Partial: Cedar context examples | Needs condition catalog |
| Explicit deny | Yes | Yes | Yes | Yes | Yes via Cedar examples | Needs API schema |
| Principal access boundary | Partial via boundaries/SCP/RCP | Yes PAB | Partial CA/admin units | Yes | Partial via Cedar only | Missing named surface |
| Permission boundaries | Yes | Partial | Partial | Yes | Not explicit | Missing |
| Organization guardrails | Yes SCP/RCP | Yes org policies/PAB | Yes tenant policies | Yes | Not explicit | Missing |
| Temporary credentials | Yes STS | Yes STS/service accounts | Yes OAuth tokens | Yes | Yes scoped token and STS broker | Needs contract |
| Token introspection | Partial | Yes STS introspection | Yes OAuth validation patterns | Yes | Yes in reference implementation | Needs API spec |
| Token revocation | Partial | Partial | Yes | Yes | Yes in reference implementation | Needs propagation SLO |
| Session tags/claims | Yes | Yes attributes | Yes claims/attributes | Yes | Partial binding/context | Needs schema |
| MFA enforcement | Yes | Via IdP/Context-Aware/PAM | Yes | Yes | Partial: tutorial requires MFA | Needs native policy |
| Phishing-resistant auth | Yes | Via federation | Yes | Yes | Not explicit | Missing |
| Passwordless/passkeys | Partial | Via IdP | Yes | Yes | Not explicit | Missing or delegated |
| Identity source sync | Yes Identity Center | Workforce federation syncless | Yes provisioning | Yes | Partial JIT only | Lifecycle gap |
| SAML federation | Yes | Yes | Yes | Yes | Yes | Contract missing |
| OIDC federation | Yes | Yes | Yes | Yes | Yes | Contract missing |
| SCIM provisioning | Identity Center/IdP | External | Yes | Yes | Not explicit | Missing |
| HR-driven lifecycle | No native core | External | Yes | Yes | Not explicit | Missing/delegated |
| Access reviews | Access Analyzer adjacent | IAM/PAM adjacent | Yes | Yes | Not explicit | Missing |
| Entitlement management | Permission sets | PAM entitlements | Access packages | Yes | Partial JIT elevation | Missing governance surface |
| Just-in-time elevation | Partial | Yes PAM | Yes PIM | Yes | Yes runbook and tenant_class policy | Needs workflow contract |
| Approval workflow | Identity Center apps/permissions | PAM grants | PIM/entitlements | Yes | Partial runbook | Needs API and UI |
| Audit history | CloudTrail | Cloud Audit Logs | Audit/sign-in logs | Yes | Yes audit-chain | Needs event schema |
| Policy analyzer | Access Analyzer | Policy Analyzer | Graph/reports | Yes | Not explicit | Missing |
| Policy simulator | Access Analyzer validation | Policy Simulator | What-if/CA | Yes | Not explicit | Missing |
| Policy troubleshooter | Access Analyzer | Troubleshooter | Troubleshooting tools | Yes | Not explicit | Missing |
| Least-privilege recommendations | Access Analyzer | Recommender | Governance/insights | Yes | Not explicit | Missing |
| Unused access analysis | Yes | Security insights | Access reviews | Yes | Not explicit | Missing |
| Device identity | No core IAM | Partial endpoint ecosystem | Yes | Yes | Not explicit | Missing/delegated |
| Application identity | Roles/service-linked | Service accounts | App registrations/service principals | Yes | Partial service accounts | Needs app model |
| Managed identity | IAM roles on compute | Service accounts attached to resources | Managed identities | Yes | Partial workload SVID | Needs provider mapping |
| X.509 workload federation | IAM Roles Anywhere | Workload federation | Workload ID certificates | Yes | Yes SPIFFE/SVID concept | Needs contract |
| External ID/confused deputy | Yes | Attribute conditions | App consent/scopes | Yes | Partial tenant binding | Needs invariant |
| Cross-account/cross-tenant bridge | Yes cross-account roles | Org/folder/project bindings | B2B/cross-tenant access | Yes | Yes cross-tenant bridge in FAQ | Needs detailed spec |
| Provider IAM translation | Native AWS only | Native GCP only | Native Azure only | Yes for Oyatie | Yes Cedar to AWS/GCP/Azure stated | Needs formal translators |
| Digest provenance | No general native | No general native | No general native | Additive | Yes concept in FAQ/migration | Needs contract |
| Immutable audit-chain anchor | CloudTrail immutable-ish | Audit logs | Audit logs | Yes | Yes BLAKE3/HSM concept | Needs event schema |
| Quota model | Yes | Yes | Yes | Yes | Not local | Missing |
| Rate limiting | Yes quotas | Yes quotas | Yes Graph throttling | Yes | Partial runbook rate-limit command | Needs formal limits |
| SLO definitions | Service-level | Service-level | Service-level | Yes | Partial local targets | Missing OpenSLO |
| OS/package matrix | Not product feature | Not product feature | Not product feature | Canonical Oyatie | Missing | Canonical gap |
| Multi-context deployment | AWS-native | GCP-native | Microsoft-native | Canonical Oyatie all-six | Missing IaC | Canonical gap |
| OCI Always Free demo_trial tenant_class | No | No | No | Canonical Oyatie | Missing | Canonical gap |
| Admin portal | Yes | Yes | Yes | Yes | Not local | Missing/delegated |
| Developer SDK | AWS SDK/CLI | gcloud/client libs | MSAL/Graph SDK | Yes | Rust SDK example only | Needs SDK surface |
| Marketplace app catalog | Identity Center apps | Workforce/federation providers | Enterprise apps/gallery | Yes | Not local | Missing/delegated |
| Legacy LDAP/Kerberos | IAM no | Cloud Identity/Managed AD adjacent | Domain Services | Partial | Not local | Missing/delegated |
| Sovereign/regional packs | AWS regions/partition | Google sovereignty | Microsoft national cloud | Yes | paid tenant_class mentions regional IdPs | Needs detail |
| Compliance evidence | CloudTrail/Config/Access Analyzer | Audit logs/SCC | Audit/PIM/Governance | Yes | Audit-chain concept | Needs compliance doc |
| Break-glass | Root/PIM patterns | PAM/emergency access | Emergency access accounts/PIM | Yes | Yes paid tenant_class | Needs workflow/tests |
| Session recording | Not core | Not core | Adjacent security tools | Additive if offered | Mentioned in runbook dashboard | Needs owner |
| AI/agent identities | Limited | Workload identities | Entra Agent ID preview | Emerging | Foundry principals | Needs governance model |
| Billing/cost awareness | IAM no charge | IAM no direct cost | License tenant_class policies | Yes for Oyatie | Tier costs partial | Needs cost budget |
| Revocation propagation SLO | Eventual consistency | Propagation documented | Token/CAE behavior | Yes | 80ms p95 claim | Needs measured proof |
| High-assurance HSM signing | KMS/CloudHSM adjacent | Cloud KMS/HSM adjacent | Managed HSM adjacent | Yes | paid tenant_class HSM signature | Needs cloud-kms handoff |

## 5. Capability Families Summary

| Family | Union required count | Oyatie present count | Oyatie partial count | Oyatie absent count | Notes |
|---|---:|---:|---:|---:|---|
| Core identity/principal model | 8 | 4 | 4 | 0 | Concepts present; schema/contracts absent |
| Authorization/policy model | 12 | 3 | 5 | 4 | Cedar is strong; analyzers/boundaries/simulators missing |
| Federation and SSO | 10 | 4 | 4 | 2 | SAML/OIDC path strong; SCIM/lifecycle weak |
| Temporary credentials and STS | 8 | 4 | 2 | 2 | Scoped tokens good; quotas/propagation proof missing |
| Governance and privileged access | 12 | 2 | 3 | 7 | JIT runbook exists; PIM/access-review/entitlement system absent |
| Workload identity | 7 | 3 | 3 | 1 | SPIFFE concept strong; managed identity/provider contracts missing |
| Audit/compliance/evidence | 8 | 2 | 4 | 2 | Audit-chain concept strong; event schema/compliance docs absent |
| Operations/SLO/quotas | 8 | 0 | 3 | 5 | Runbooks strong; SLO and quota artifacts absent |
| Deployment/platform portability | 7 | 0 | 1 | 6 | Canonical all-six and OS/IaC coverage absent |
| User/admin experience | 7 | 0 | 1 | 6 | UX and marketplace breadth largely unowned locally |
| Additive Oyatie surface | 6 | 4 | 2 | 0 | Cedar portability, digest provenance, audit-chain, Foundry principals promising |

## 6. Headline Gap Analysis - Top 15 Missing Capabilities

1. Missing service-local IAM/STS OpenAPI contract: the product PRD points to `contracts/openapi/cloud/cloud-iam-v1.yaml`, but the service path has no contract file.
2. Missing principal/entity schema: FAQ sketches users, workloads, service accounts, federated identities, and entity-store fields, but no schema or migration exists.
3. Missing policy analyzer/simulator/troubleshooter: AWS and Google both offer analyzer/simulator/troubleshooter surfaces; `cloud-iam` only documents translator checks.
4. Missing access reviews: Microsoft Entra treats recurring access reviews as central governance; `cloud-iam` has incident runbooks but not recertification.
5. Missing entitlement/access-package model: Entra entitlement management and Google PAM entitlements have no local equivalent beyond JIT elevation.
6. Missing lifecycle provisioning: SCIM, HR-driven lifecycle, joiner/mover/leaver workflows, and group lifecycle are not defined here.
7. Missing Conditional Access/risk/device signal model: Entra's policy engine combines user, group, agent, IP, device, app, and risk signals; `cloud-iam` only has MFA context examples.
8. Missing device identity boundary: Entra has device identities; `cloud-iam` does not say whether device identity is delegated or absent.
9. Missing quota/limit model: AWS, Google, and Microsoft publish limits; local tenant_class policy only gives some role/principal and latency targets.
10. Missing OCI dynamic-group adapter: canonical docs require `cloud-iam` to map to OCI dynamic groups/policies, but the service docs do not mention it.
11. Missing OpenTofu context modules: all provider-context claims lack deployable IaC.
12. Missing OS/package/CI matrix: the service has no supported OS manifest.
13. Missing audit event schemas: audit-chain names are pervasive, but there is no machine-readable event catalog.
14. Missing admin UX and application catalog posture: AWS Identity Center and Entra provide rich app/admin portals; local docs only mention workflow-studio as less mature.
15. Missing measured benchmark harness: the existing benchmark doc claims measured numbers but does not attach signed evidence under the service path.

Implementation hooks:
Hook 1 should land the OpenAPI contract before new tutorial expansion.
Hook 2 should land `principal`, `role`, `policy`, `session`, `token`, `idp`, and `audit_event` schemas.
Hook 3 should add `policy analyze`, `policy simulate`, and `policy explain` use cases to the Rust domain.
Hook 4 should add `access_review` and `entitlement` domain objects or explicitly delegate them to a governance service.
Hook 5 should add SCIM and lifecycle boundaries, even if ownership moves to platform identity.
Hook 6 should add a risk-signal input contract from security/identity/observability.
Hook 7 should add OCI Always Free and dynamic-group support in the adapter model.
Hook 8 should add all-six OpenTofu modules and CI gates.
Hook 9 should add OpenSLO files and quota tables by tenant_class policy/context.
Hook 10 should add audit event schemas and compliance evidence pointers.

## 7. Additive Surface

Additive-01 Cedar as a single portable policy authority across AWS/GCP/Azure/OCI/on-prem is stronger than a single-cloud native IAM policy model.
Additive-02 Cedar-to-provider IAM translation with a BLAKE3 digest pointer gives provenance that counterparts do not expose as a universal cross-cloud primitive.
Additive-03 Audit-chain anchoring of every authorization decision creates a product-native evidence chain if event schemas and storage contracts land.
Additive-04 Refusal of static AWS access keys is stricter than many AWS migration defaults and aligns with temporary credential doctrine.
Additive-05 Tiered SPIFFE SVID rotation is more explicit than most public IAM tenant_class policy matrices.
Additive-06 Foundry pipeline principals as first-class IAM subjects anticipates autonomous build/deployment identities.
Additive-07 Cross-tenant bridge subjects are a clear M&A and managed-service differentiator if tenant-boundary invariants land.
Additive-08 The runbook set treats provider over-permit as a security incident, which is a strong cross-cloud IAM operating stance.
Additive-09 The service explicitly separates Cedar source authority from provider IAM artifacts, reducing provider lock-in.
Additive-10 The paid tenant_class policy's HSM-signed audit root could exceed basic public-cloud audit-log assurances if wired to `cloud-kms`.
Additive-11 The tenant_class policy matrix attempts to pair scale, federation breadth, audit retention, and compliance overlays in one service-specific tenant_class policy model.
Additive-12 The local Rust SDK example gives a concrete developer workflow, but it needs generated SDK ownership and versioning.

## 8. Parity Verdict

The service has a strong identity: portable cloud IAM with Cedar authority, short-lived credentials, IdP federation, provider translation, and audit-chain evidence.
It is not yet union-complete against AWS IAM plus Identity Center, Google Cloud IAM, and Microsoft Entra ID.
The highest-risk missing family is governance: access reviews, entitlement management, lifecycle workflows, policy analysis, and risk/device signal policy.
The highest-risk canonical mismatch is deployment: all six required contexts lack OpenTofu, OS manifest, and OCI Always Free demo_trial tenant_class reconciliation.
The strongest local differentiator is Cedar portability with digest-backed provider translation.
The parity decision is partial, with P1 remediation required before claiming hyperscaler IAM maturity.

## 9. Capability Remediation Ledger

Ledger R01 Core API: create `roles.create`, `roles.get`, `roles.list`, `roles.delete`, `policies.attach`, `policies.detach`, `sts.issue`, `sts.introspect`, and `sts.revoke` operations because repo-level cloud PRD line 172 already names IAM plus STS as the service surface.
Ledger R02 Principal taxonomy: define `User`, `ServiceAccount`, `Workload`, `Federated`, `BridgeSubject`, and `FoundryAgent` as machine-readable entities because ADR-0331 tenant_class adoption template lines 12-61 rely on those categories.
Ledger R03 Principal lifecycle: add create/disable/delete/rotate flows because counterparts all expose identity lifecycle either directly or through an adjacent identity plane.
Ledger R04 Group lifecycle: add group/team binding semantics or explicitly delegate groups to platform identity because Entra and Google both make group membership central to IAM evaluation.
Ledger R05 Role lifecycle: preserve the ADR-0331 tenant_class adoption template role ceilings but add role immutability, versioning, and rollback semantics.
Ledger R06 Policy lifecycle: define Cedar policy packages, revisions, approvals, activation, rollback, and digest publication.
Ledger R07 Policy attachment: model principal, group, role, tenant, resource, and provider-projection attachment points.
Ledger R08 Policy simulation: add a simulator endpoint because AWS Policy Simulator and Google Policy Troubleshooter create a practical parity expectation.
Ledger R09 Policy analysis: add unused privilege, external exposure, wildcard permission, and provider-overpermit analysis.
Ledger R10 Policy explanation: add denial and allow-path explanation with Cedar proof snippets and provider-translation traces.
Ledger R11 Permission boundaries: decide whether to implement AWS-style permissions boundaries or a Cedar-native max-permission envelope.
Ledger R12 Principal access boundary: decide whether Google-style Principal Access Boundary is represented as a Cedar scope contract.
Ledger R13 Organization guardrails: define tenant/org-level policy guardrails analogous to AWS SCPs and Google organization policies.
Ledger R14 Root or tenant owner: define the highest-privilege break-glass subject and its restrictions because every counterpart has an emergency-account pattern.
Ledger R15 MFA requirements: move local password/TOTP from demo_trial tenant_class prose into explicit factor policy and risk policy inputs.
Ledger R16 Conditional Access: model signals for user, group, resource, app, IP, network, device, risk, location, and session.
Ledger R17 Device identity: state whether device trust is first-class in `cloud-iam` or consumed from another service.
Ledger R18 Risk signals: define producer contracts for impossible travel, suspicious IP, credential leak, and anomalous elevation.
Ledger R19 Access reviews: add campaign, reviewer, decision, expiry, and remediation workflows for Entra parity.
Ledger R20 Entitlement packages: add access package and assignment objects or explicitly declare them out of cloud-iam ownership.
Ledger R21 PIM/JIT: turn the JIT misuse runbook into a product contract with eligible, active, pending, expired, and revoked states.
Ledger R22 Approval workflow: define approver policy, quorum, expiry, escalation, and audit requirements.
Ledger R23 Break-glass workflow: define key ceremony, quorum, HSM signature, recording, and automatic post-incident review.
Ledger R24 SCIM provisioning: define SCIM ownership or ingestion boundaries for workforce identity imports.
Ledger R25 SAML federation: convert the tutorial path into a durable SAML IdP contract with metadata, certificate rollover, ACS errors, and replay defense.
Ledger R26 OIDC federation: define issuer discovery, JWKS cache, nonce/state handling, and stale metadata behavior.
Ledger R27 Workforce federation: map Google Workforce Identity Federation and AWS Identity Center concepts into local IdP provider objects.
Ledger R28 Workload federation: map AWS OIDC roles, GCP Workload Identity Federation, Azure workload identities, SPIFFE, and Kubernetes service accounts.
Ledger R29 OCI federation: add OCI dynamic group and policy adapter because canonical context rules require guest-on-oci support.
Ledger R30 Azure adapter: define Microsoft Graph app/service-principal projection and throttling behavior.
Ledger R31 GCP adapter: define IAM binding write behavior and policy size limit handling.
Ledger R32 AWS adapter: define IAM role, trust policy, permissions policy, STS, and eventual consistency handling.
Ledger R33 Provider abstraction: business logic must use a provider-neutral adapter boundary rather than direct vendor API calls.
Ledger R34 Provider drift: add periodic provider-state reconciliation with digest comparison.
Ledger R35 Translation loss: classify exact, conservative-over, conservative-under, and unsupported projections.
Ledger R36 Provider overpermit incident: preserve the runbook's security posture by making overpermit a typed alert and automatic rollback trigger.
Ledger R37 Token format: define scoped-token claims, issuer, audience, tenant, resource, policy digest, context digest, and expiry.
Ledger R38 Token introspection: specify cacheability, negative cache behavior, revoked-token response, and tenant binding.
Ledger R39 Token revocation: define cell-local, cross-cell, and provider-propagation semantics separately.
Ledger R40 Session policy: define inline session restrictions and max duration by tenant_class policy.
Ledger R41 STS quotas: publish per-tenant issue/introspect/revoke quotas aligned with tenant_class policy and context.
Ledger R42 Authorization quotas: publish hot-path authorize quotas separately from provider-translation quotas.
Ledger R43 Audit event catalog: define `principal.created`, `role.created`, `policy.attached`, `token.issued`, `token.revoked`, `jit.activated`, and provider-translation events.
Ledger R44 Audit retention: reconcile 90d/1y/3y/7y retention with tenant_class policy, context, and OCI Always Free storage limits.
Ledger R45 Audit integrity: define BLAKE3/HSM roots, signature cadence, verification API, and evidence export.
Ledger R46 Compliance overlays: map FedRAMP, SOC 2, ISO, KCMVP, eIDAS, Aadhaar, and national-cloud controls to tenant_class policy obligations.
Ledger R47 Regional isolation: define region, cell, tenant, sovereign pack, and data-residency boundaries.
Ledger R48 Admin UX: define a UI/API split for policy authoring, federation setup, token inspection, and incident response.
Ledger R49 SDK ownership: state whether the Rust SDK example becomes generated SDK output or remains educational sample code.
Ledger R50 cloud-ci ownership: define the branch-protected `oya-ci-required` / owned `oya-ci` gate surface for intern-buildable workflows instead of shell fragments.
Ledger R51 Migration tooling: replace Okta/AWS extraction examples with a governed Rust migration command and fixture tests.
Ledger R52 Import validation: add dry-run, diff, approval, and rollback behavior for imported identities and policies.
Ledger R53 Account linking: define how external IdP subjects link to Oyatie principals and how conflicts resolve.
Ledger R54 Tenant isolation: define cross-tenant bridge invariants and safe delegation boundaries.
Ledger R55 Managed-service mode: define how `oyatie-as-cloud-provider` tenants get dedicated or shared IAM control planes.
Ledger R56 On-prem mode: define offline IdP metadata, HSM availability, and local audit export behavior.
Ledger R57 Colo mode: define facility-network assumptions and whether provider translation is absent or local.
Ledger R58 Guest-on-AWS mode: define AWS API quotas, STS quota backpressure, and IAM eventual consistency.
Ledger R59 Guest-on-GCP equivalent: no separate canonical context exists, so Google provider support must be adapter capability, not deployment-context identity.
Ledger R60 Guest-on-OCI mode: define Always Free demo_trial tenant_class and paid paid tenant_class profiles separately.
Ledger R61 OpenTofu modules: add per-context modules before claiming deployability.
Ledger R62 State backends: publish per-context state backend, locking, encryption, signing, and recovery model.
Ledger R63 Module signing: connect OpenTofu modules to ADR-0039 sigstore expectations.
Ledger R64 OS manifest: add Tier-1, Tier-2, out-of-scope, package format, and CI lane declarations.
Ledger R65 Build invocation: document the branch-protected `oya-ci-required` / owned `oya-ci` build target; local Cargo runs are rehearsal only and are not release authority.
Ledger R66 SLO files: add OpenSLO for authorize, token issue, introspect, revoke, federation callback, provider translation, and audit append.
Ledger R67 Capacity model: add role, principal, policy, IdP, token, and audit-event ceilings by tenant_class policy/context.
Ledger R68 Cost budget: add tenant_class policies cost envelopes, with OCI demo_trial tenant_class explicitly zero paid infrastructure.
Ledger R69 DPIA: add personal-data, federation, audit-retention, and cross-border transfer analysis.
Ledger R70 Compliance doc: add control mappings and evidence generation points.
Ledger R71 Incident response: consolidate the three runbooks under a service-level incident taxonomy.
Ledger R72 Failure modes: add provider outage, IdP outage, audit-chain outage, cache corruption, HSM outage, and quota exhaustion modes.
Ledger R73 Cross-service handoffs: add producer/consumer contracts for identity, cloud-kms, cloud-billing, cloud-network, observability, and workflow-studio.
Ledger R74 Benchmark harness: move claimed benchmark values behind signed measurement artifacts.
Ledger R75 Parity gate: do not claim union coverage until the ledger rows for governance, analysis, deployment, OS, IaC, SLO, and contract surfaces are represented by artifacts.
