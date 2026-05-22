# Identity performance benchmark numbers - 2026-05-20

Citation anchors:
1. Canonical benchmark disclosure requirement: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:4208-4209`.
2. Master plan constraints: `specs/master-plan-sequencing.json:704-889`.
3. Identity PRD performance and availability targets: `microservices/identity/PRD.md:779-811`.
4. Identity capacity model: `microservices/identity/capacity-model.md:26-158`.
5. Documentation rigor intern-buildability bar: `docs/standards/documentation-rigor.md:133-139`.

Methodology disclosure:
These are target numbers and comparison/planning numbers, not measured Oyatie production benchmarks.
Measured benchmarks must be added in the build phase with raw result artifacts, workload definitions, OS, architecture, deployment context, tenant class, and provenance per ADR-0212 and ADR-0328 D-20.152.
The local benchmark document currently gives modeled values but does not attach the raw result artifact that would make the numbers measured evidence (`microservices/identity/benchmarks/okta-auth0-entra-vs-oyatie.md:19-119`).
Public counterpart vendors do not publish one uniform latency/RPS benchmark for every identity capability; where exact public limits are absent, this document labels values as "target", "planning estimate", or "source limit".

External source anchors:
- Auth0 rate-limit policy: `https://auth0.com/docs/policies/rate-limit-policy`.
- Auth0 brute-force protection thresholds: `https://auth0.com/docs/secure/attack-protection/brute-force-protection`.
- Auth0 MFA factors: `https://auth0.com/docs/secure/multi-factor-authentication`.
- Auth0 SCIM behavior: `https://auth0.com/docs/authenticate/protocols/scim/configure-inbound-scim`.
- Okta rate limits: `https://developer.okta.com/docs/reference/rate-limits/`.
- Okta SCIM concepts: `https://developer.okta.com/docs/concepts/scim/`.
- Okta FastPass: `https://help.okta.com/oie/en-us/content/topics/identity-engine/devices/fp/fp-main.htm`.
- Microsoft Entra passkeys/FIDO2: `https://learn.microsoft.com/en-us/entra/identity/authentication/how-to-authentication-passkeys-fido2`.
- Microsoft Entra provisioning: `https://learn.microsoft.com/en-us/entra/identity/app-provisioning/how-provisioning-works`.
- Microsoft Entra Conditional Access: `https://learn.microsoft.com/en-us/entra/identity/conditional-access/concept-conditional-access-policies`.

## 1. Methodology

01. Benchmark dimension: token issue latency p50, p95, and p99.
02. Benchmark dimension: token verify latency p50, p95, and p99.
03. Benchmark dimension: JWKS read latency p50, p95, and p99.
04. Benchmark dimension: WebAuthn registration ceremony latency p50, p95, and p99.
05. Benchmark dimension: WebAuthn authentication ceremony latency p50, p95, and p99.
06. Benchmark dimension: SCIM user create/update/deactivate p50, p95, and p99.
07. Benchmark dimension: step-up ACR grant p50, p95, and p99.
08. Benchmark dimension: external IdP callback p50, p95, and p99.
09. Benchmark dimension: event emit completion p99.
10. Benchmark dimension: audit-chain commit p99.
11. Benchmark dimension: peak token issue throughput in requests per second.
12. Benchmark dimension: peak token verify throughput in requests per second.
13. Benchmark dimension: peak WebAuthn verify throughput in ceremonies per second.
14. Benchmark dimension: peak SCIM write throughput in writes per second.
15. Benchmark dimension: concurrent active sessions per cell.
16. Benchmark dimension: tenant count per cell.
17. Benchmark dimension: max users per tenant by tenant_class.
18. Benchmark dimension: recovery ceremony completion latency.
19. Benchmark workload token issue: client-credentials, authorization-code, refresh-token, and step-up token mixes.
20. Benchmark workload token verify: JWT validation with Cedar context extraction and cache-hit JWKS path.
21. Benchmark workload JWKS: cacheable public-key read and emergency key rotation overlap window.
22. Benchmark workload WebAuthn registration: challenge creation, attestation validation, credential binding, and audit event.
23. Benchmark workload WebAuthn authentication: challenge creation, signature validation, sign-count policy, AAGUID policy, and session issue.
24. Benchmark workload SCIM: create, read, update, deactivate, group add/remove, and retry/idempotency behavior.
25. Benchmark workload step-up: ACR challenge, MFA/passkey ceremony, IT approval when required, and token mint.
26. Benchmark workload federation: SAML/OIDC callback, external claim mapping, local step-up, and token issue.
27. Benchmark workload audit: exactly-once event emission to audit-chain and observability labels.
28. OS disclosure baseline: tenant_class-1 OS matrix must be added locally; current identity path has no `supported-oses.json`.
29. Architecture disclosure baseline: linux/arm64 and linux/amd64 are assumed for server contexts until the OS manifest lands.
30. Architecture disclosure for OCI Always Free: linux/arm64 Ampere A1 is the demo_trial OCI baseline because master plan caps the profile at 4 OCPU and 24 GiB RAM.
31. Architecture disclosure for macOS M5+: only developer/client validation is expected unless the service explicitly ships local components.
32. Deployment context disclosure: all six canonical contexts are evaluated: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`.
33. Tenant class disclosure: demo_trial OCI is demo/sandbox/trial/dev Always Free; paid with per_seat billing_component is paid baseline; paid with per_usage billing_component is production scale; paid with compliance_pack gating is hyperscaler/single-tenant capable.
34. Measurement gap: no current identity source tree or benchmark harness was found under `microservices/identity/src/`.
35. Measurement gap: no raw benchmark result CSV was found under identity, despite local benchmark doc referring to result paths.
36. Stop condition for future measured pass: raw results plus harness commit, OS/arch, context, tenant class, hardware profile, data size, and p50/p95/p99 distributions.

## 2. Counterpart numbers

### 2.1 Auth0 public and planning numbers

01. Auth0 source limit: brute-force protection default threshold is 10 incorrect attempts from one IP to one user identifier before mitigation; source: Auth0 brute-force protection docs.
02. Auth0 source limit: custom brute-force threshold range is 1 to 100 incorrect attempts; source: Auth0 brute-force protection docs.
03. Auth0 source limit: SMS notifications for brute-force protection are limited to 1 per hour per identifier; source: Auth0 brute-force protection docs.
04. Auth0 source limit: email notifications for brute-force protection are limited to 1 per hour per unique IP; source: Auth0 brute-force protection docs.
05. Auth0 source limit: brute-force protection blocks can remain until 30 days pass from last failed login attempt; source: Auth0 brute-force protection docs.
06. Auth0 source limit: SCIM endpoint can have up to two active generated tokens for rotation without downtime; source: Auth0 SCIM docs.
07. Auth0 source fact: rate limits vary by tenant, endpoint, API, extensibility product, and private cloud performance tenant_class; source: Auth0 rate-limit policy.
08. Auth0 planning estimate: public-tenant Management API sustained write ceilings should be modeled under hundreds to low thousands of writes per minute unless contract-specific limits prove higher.
09. Auth0 planning estimate: hosted login p95 for normal flows should be treated as external-network dominated, 300-900 ms before app callback, absent a vendor-specific tenant benchmark.
10. Auth0 planning estimate: SCIM bulk provisioning should be modeled as backpressure-friendly and retry-aware rather than a flat high-RPS API.
11. Auth0 planning estimate: adaptive MFA adds one network round trip and factor ceremony, modeled as 1-30 seconds user-time depending on factor.
12. Auth0 comparison note: Auth0 publishes operational rate-limit policy but not a universal token-issue p99 number.

### 2.2 Okta public and planning numbers

01. Okta source limit: example `/oauth2/v1/authorize` org bucket quota is 1200 requests per minute; source: Okta rate-limit docs.
02. Okta source limit: example nested `/oauth2/v1/authorize` client app quota is 600 requests per minute; source: Okta rate-limit docs.
03. Okta source limit: example `/api/v1/users/*` org quota is 1000 requests per minute; source: Okta rate-limit docs.
04. Okta source limit: example `/api/v1/users/me` authenticated-user quota is 40 requests per 10 seconds; source: Okta rate-limit docs.
05. Okta source behavior: most counters allow N requests per minute and reset every 60 seconds, not necessarily on wall-clock minute boundary; source: Okta rate-limit docs.
06. Okta source behavior: exceeded quota returns HTTP 429 until reset; source: Okta rate-limit docs.
07. Okta source behavior: rate-limit warnings and violations generate System Log events; source: Okta rate-limit docs.
08. Okta source SCIM retry behavior: integer `Retry-After` is honored, missing header defaults to 5 minutes, and retries can use exponential backoff; source: Okta SCIM docs.
09. Okta planning estimate: FastPass device-bound public-key verification should be modeled at sub-100 ms server time but user/device ceremony dominates end-to-end latency.
10. Okta planning estimate: OIN provisioning workloads should be modeled with 429-aware backpressure and not as unbounded SCIM write throughput.
11. Okta planning estimate: access certification campaigns are batch/governance jobs, measured in minutes to hours for campaign completion rather than request latency.
12. Okta comparison note: Okta publishes example API quotas but not a universal p99 login benchmark for every customer tenant.

### 2.3 Microsoft Entra public and planning numbers

01. Entra source limit: passkey policy object size limit is 20 KB for authentication methods policy; source: Entra passkey docs.
02. Entra source limit: up to three passkey profiles, including the Default profile, are supported at the time of the cited docs; source: Entra passkey docs.
03. Entra source condition: users must complete MFA within the past 5 minutes before registering a passkey; source: Entra passkey docs.
04. Entra source size: base passkey policy reference size is 1.44 KB; source: Entra passkey docs.
05. Entra source size: target with one applied passkey profile is about 0.23 KB; source: Entra passkey docs.
06. Entra source size: target with five applied passkey profiles is about 0.4 KB; source: Entra passkey docs.
07. Entra source size: passkey profile with no AAGUIDs is about 0.4 KB; source: Entra passkey docs.
08. Entra source size: passkey profile with ten AAGUIDs is about 0.3 KB; source: Entra passkey docs.
09. Entra source security: provisioning channel uses HTTPS TLS 1.2; source: Entra provisioning docs.
10. Entra source behavior: Conditional Access policies are evaluated when a token is issued for role/group targeting; source: Conditional Access docs.
11. Entra planning estimate: Conditional Access adds policy-evaluation latency at token issue; model p95 under 200 ms for policy decision target, excluding user MFA ceremony.
12. Entra planning estimate: SCIM provisioning is batch/backpressure governed; model throughput by connector and target app, not one universal value.
13. Entra comparison note: Entra publishes policy/size/behavior limits, but not a single public p99 login number applicable to every tenant and region.

## 3. Oyatie target numbers by tenant_class and deployment context

### 3.1 demo_trial - `oyatie-public-cloud`

01. Target token issue p50: 20 ms.
02. Target token issue p95: 55 ms.
03. Target token issue p99: 90 ms.
04. Target token verify p99: 15 ms.
05. Target JWKS read p99: 25 ms.
06. Target WebAuthn authenticate p95: 120 ms server time.
07. Target SCIM write p95: 400 ms.
08. Target step-up grant p95: 180 ms server time excluding human factor.
09. Target token issue throughput: 2500 requests per second per cell.
10. Target tenant ceiling: 50 tenants per cell.

### 3.2 demo_trial - `guest-on-aws`

01. Target token issue p50: 25 ms.
02. Target token issue p95: 65 ms.
03. Target token issue p99: 105 ms.
04. Target token verify p99: 18 ms.
05. Target JWKS read p99: 30 ms.
06. Target WebAuthn authenticate p95: 135 ms server time.
07. Target SCIM write p95: 450 ms.
08. Target step-up grant p95: 210 ms server time excluding human factor.
09. Target token issue throughput: 2000 requests per second per cell.
10. Target tenant ceiling: 40 tenants per cell.

### 3.3 demo_trial - `guest-on-oci` Always Free

01. Target token issue p50: 35 ms.
02. Target token issue p95: 95 ms.
03. Target token issue p99: 160 ms.
04. Target token verify p99: 30 ms.
05. Target JWKS read p99: 45 ms.
06. Target WebAuthn authenticate p95: 220 ms server time.
07. Target SCIM write p95: 900 ms.
08. Target step-up grant p95: 350 ms server time excluding human factor.
09. Target token issue throughput: 350 requests per second for the Always Free cell profile.
10. Target tenant ceiling: 5 small tenants per Always Free cell profile.

### 3.4 demo_trial - `on-prem`

01. Target token issue p50: 30 ms on qualified tenant_class-1 hardware.
02. Target token issue p95: 80 ms.
03. Target token issue p99: 130 ms.
04. Target token verify p99: 25 ms.
05. Target JWKS read p99: 40 ms.
06. Target WebAuthn authenticate p95: 170 ms server time.
07. Target SCIM write p95: 700 ms.
08. Target step-up grant p95: 260 ms server time excluding human factor.
09. Target token issue throughput: 1000 requests per second per local cell.
10. Target disconnected operation: 24 hours minimum for auth challenge/session hot path.

### 3.5 demo_trial - `colo`

01. Target token issue p50: 28 ms.
02. Target token issue p95: 75 ms.
03. Target token issue p99: 120 ms.
04. Target token verify p99: 22 ms.
05. Target JWKS read p99: 35 ms.
06. Target WebAuthn authenticate p95: 160 ms server time.
07. Target SCIM write p95: 650 ms.
08. Target step-up grant p95: 240 ms server time excluding human factor.
09. Target token issue throughput: 1200 requests per second per cell.
10. Target facility failover detection: 60 seconds.

### 3.6 demo_trial - `oyatie-as-cloud-provider`

01. Target token issue p50: 20 ms.
02. Target token issue p95: 50 ms.
03. Target token issue p99: 85 ms.
04. Target token verify p99: 12 ms.
05. Target JWKS read p99: 20 ms.
06. Target WebAuthn authenticate p95: 110 ms server time.
07. Target SCIM write p95: 350 ms.
08. Target step-up grant p95: 170 ms server time excluding human factor.
09. Target token issue throughput: 3000 requests per second per provider cell.
10. Target tenant ceiling: 75 small tenants per provider cell.

### 3.7 paid with per_seat billing_component targets across contexts

01. paid with per_seat billing_component `oyatie-public-cloud`: token issue p99 65 ms, verify p99 10 ms, WebAuthn p95 95 ms, SCIM p95 250 ms, step-up p95 140 ms, token issue 10000 rps, verify 75000 rps, 500 tenants per cell.
02. paid with per_seat billing_component `guest-on-aws`: token issue p99 80 ms, verify p99 14 ms, WebAuthn p95 115 ms, SCIM p95 320 ms, step-up p95 170 ms, token issue 7000 rps, verify 50000 rps, 350 tenants per cell.
03. paid with per_seat billing_component `guest-on-oci`: token issue p99 85 ms, verify p99 15 ms, WebAuthn p95 125 ms, SCIM p95 350 ms, step-up p95 190 ms, token issue 6000 rps, verify 45000 rps, 300 tenants per paid cell.
04. paid with per_seat billing_component `on-prem`: token issue p99 95 ms, verify p99 18 ms, WebAuthn p95 140 ms, SCIM p95 450 ms, step-up p95 220 ms, token issue 4000 rps, verify 30000 rps, 200 tenants per local cell.
05. paid with per_seat billing_component `colo`: token issue p99 85 ms, verify p99 15 ms, WebAuthn p95 125 ms, SCIM p95 375 ms, step-up p95 200 ms, token issue 5000 rps, verify 40000 rps, 250 tenants per facility cell.
06. paid with per_seat billing_component `oyatie-as-cloud-provider`: token issue p99 60 ms, verify p99 9 ms, WebAuthn p95 90 ms, SCIM p95 230 ms, step-up p95 130 ms, token issue 12000 rps, verify 90000 rps, 700 tenants per provider cell.
07. paid with per_seat billing_component availability target: OIDC/JWKS path 99.99% per month except JWKS cache endpoint 99.999%.
08. paid with per_seat billing_component audit target: event emit completeness 99.999% for security-critical events.
09. paid with per_seat billing_component recovery target: normal recovery ceremony server time p95 under 750 ms excluding human proof collection.
10. paid with per_seat billing_component concurrency target: 1 million active sessions per cell.

### 3.8 paid with per_usage billing_component targets across contexts

01. paid with per_usage billing_component `oyatie-public-cloud`: token issue p99 45 ms, verify p99 7 ms, WebAuthn p95 75 ms, SCIM p95 180 ms, step-up p95 110 ms, token issue 50000 rps, verify 250000 rps, 5000 tenants per region.
02. paid with per_usage billing_component `guest-on-aws`: token issue p99 60 ms, verify p99 10 ms, WebAuthn p95 90 ms, SCIM p95 230 ms, step-up p95 140 ms, token issue 35000 rps, verify 175000 rps, 3500 tenants per region.
03. paid with per_usage billing_component `guest-on-oci`: token issue p99 65 ms, verify p99 11 ms, WebAuthn p95 95 ms, SCIM p95 260 ms, step-up p95 155 ms, token issue 30000 rps, verify 150000 rps, 3000 tenants per region.
04. paid with per_usage billing_component `on-prem`: token issue p99 75 ms, verify p99 14 ms, WebAuthn p95 115 ms, SCIM p95 350 ms, step-up p95 180 ms, token issue 15000 rps, verify 90000 rps, 1000 tenants per customer region.
05. paid with per_usage billing_component `colo`: token issue p99 65 ms, verify p99 12 ms, WebAuthn p95 100 ms, SCIM p95 300 ms, step-up p95 165 ms, token issue 22000 rps, verify 125000 rps, 1800 tenants per facility region.
06. paid with per_usage billing_component `oyatie-as-cloud-provider`: token issue p99 40 ms, verify p99 6 ms, WebAuthn p95 70 ms, SCIM p95 160 ms, step-up p95 100 ms, token issue 75000 rps, verify 400000 rps, 7500 tenants per provider region.
07. paid with per_usage billing_component availability target: 99.995% issuer availability and 99.999% JWKS availability.
08. paid with per_usage billing_component audit target: critical identity event commit p99 under 250 ms.
09. paid with per_usage billing_component recovery target: account recovery session rebinding p95 under 500 ms server time.
10. paid with per_usage billing_component concurrency target: 10 million active sessions per region.

### 3.9 paid with compliance_pack gating targets across contexts

01. paid with compliance_pack gating `oyatie-public-cloud`: token issue p99 35 ms, verify p99 5 ms, WebAuthn p95 60 ms, SCIM p95 120 ms, step-up p95 80 ms, token issue 250000 rps, verify 1000000 rps, single-tenant isolation available.
02. paid with compliance_pack gating `guest-on-aws`: token issue p99 45 ms, verify p99 7 ms, WebAuthn p95 70 ms, SCIM p95 160 ms, step-up p95 100 ms, token issue 150000 rps, verify 700000 rps, single-tenant isolation available.
03. paid with compliance_pack gating `guest-on-oci`: token issue p99 50 ms, verify p99 8 ms, WebAuthn p95 80 ms, SCIM p95 180 ms, step-up p95 115 ms, token issue 125000 rps, verify 600000 rps, single-tenant isolation available.
04. paid with compliance_pack gating `on-prem`: token issue p99 60 ms, verify p99 10 ms, WebAuthn p95 95 ms, SCIM p95 240 ms, step-up p95 140 ms, token issue 50000 rps, verify 250000 rps, disconnected sovereign cell available.
05. paid with compliance_pack gating `colo`: token issue p99 52 ms, verify p99 9 ms, WebAuthn p95 85 ms, SCIM p95 210 ms, step-up p95 125 ms, token issue 80000 rps, verify 400000 rps, dedicated hardware cell available.
06. paid with compliance_pack gating `oyatie-as-cloud-provider`: token issue p99 30 ms, verify p99 4 ms, WebAuthn p95 55 ms, SCIM p95 100 ms, step-up p95 75 ms, token issue 500000 rps, verify 2000000 rps, provider-grade tenant isolation.
07. paid with compliance_pack gating availability target: issuer 99.999% and JWKS 99.9995%.
08. paid with compliance_pack gating audit target: critical event commit p99 under 100 ms with region-local seal path.
09. paid with compliance_pack gating recovery target: recovery rebinding p95 under 350 ms server time.
10. paid with compliance_pack gating concurrency target: 100 million active sessions per global deployment.

## 4. Per-context overlay

01. `oyatie-public-cloud` overlay: targets assume Oyatie-operated cell networking, internal OCI or owned hardware state backend, and full observability control.
02. `guest-on-aws` overlay: add 10-20 ms p99 allowance for AWS account boundary, VPC endpoint, KMS adapter, and CloudWatch export normalization.
03. `guest-on-oci` paid overlay: add 10-25 ms p99 allowance for OCI Object Storage/Vault/VCN primitives when not running as Oyatie public cloud.
04. `guest-on-oci` Always Free overlay: enforce hard throughput and tenant ceilings so the profile never spills into AWS or paid public capacity.
05. `on-prem` overlay: numbers are valid only after customer hardware, local HSM, DNS, and storage baselines pass qualification.
06. `on-prem` disconnected overlay: priority is successful local auth and audit backlog safety over global p99.
07. `colo` overlay: numbers assume facility network and remote-hands telemetry have been modeled through cloud-network/cloud-dcops.
08. `oyatie-as-cloud-provider` overlay: lowest latency targets are justified only after identity is integrated with provider-native `cloud-*` control planes.
09. All contexts: token verify target assumes local JWKS cache hit.
10. All contexts: external IdP callback target excludes third-party IdP latency.
11. All contexts: MFA/passkey target server time excludes user interaction time.
12. All contexts: SCIM target excludes customer HRIS/source-system delay.
13. All contexts: audit target assumes audit-chain is healthy; backlog mode must report separately.
14. All contexts: all targets are unmeasured until build-phase harness evidence exists.
15. All contexts: context-specific CI lanes must publish p50/p95/p99 distributions.

## 5. Comparison narrative

01. Token issue throughput: Oyatie paid with per_usage billing_component/paid with compliance_pack gating targets are ahead of the public Okta example authorize bucket, but the comparison is not apples-to-apples because Okta docs show quota examples, not tenant-specific hyperscale contract limits.
02. Token issue latency: Oyatie targets are aggressive and require local cell validation; counterparts do not publish a uniform p99 target.
03. Token verify latency: Oyatie can be ahead if verification is local JWT plus Cedar context and not a remote introspection call.
04. JWKS availability: Oyatie targets parity or ahead because PRD already targets JWKS 99.999% (`PRD.md:795-811`).
05. WebAuthn server latency: Oyatie targets parity for server-side ceremony, while user experience depends on device/platform flows like Okta FastPass and Entra passkeys.
06. SCIM throughput: Oyatie targets catch-up; counterpart SCIM flows emphasize backpressure, retry, and provisioning semantics over raw RPS.
07. Step-up latency: Oyatie targets parity for server-side ACR grant; human-factor and JIT approval flows will dominate user-time.
08. Risk scoring: Oyatie is catch-up because IP-014 is deferred while Auth0/Okta/Entra already expose attack/risk surfaces.
09. Device posture: Oyatie is catch-up because Okta FastPass and Entra Conditional Access have device/platform signal surfaces.
10. Governance batch jobs: Oyatie is behind because access certifications, requests, and entitlement campaigns are not implemented locally.
11. OCI Always Free demo_trial: Oyatie is currently behind its own doctrine because the tenant_class adoption matrix exceeds Always Free.
12. OpenTofu deployment: Oyatie is behind its own doctrine because no context modules exist.
13. OS matrix: Oyatie is behind its own doctrine because no `supported-oses.json` exists.
14. Audit-chain evidence: Oyatie can be ahead if audit-chain seal claims become measured p99 evidence.
15. Sovereign/offline operation: Oyatie can be ahead of hosted counterparts once on-prem/colo IaC and disconnected evidence exist.
16. Current benchmark confidence: medium for target shape, low for measured performance, because no raw harness output exists in the identity path.

## 6. Build-phase benchmark evidence plan

01. Evidence artifact: `benchmarks/results/identity/<date>/summary.json` should record tenant_class, context, OS, architecture, tenant class, hardware, and commit.
02. Evidence artifact: `benchmarks/results/identity/<date>/token-issue.csv` should record p50, p95, p99, max, error rate, and requests per second.
03. Evidence artifact: `benchmarks/results/identity/<date>/token-verify.csv` should separate JWKS cache-hit and cache-miss cases.
04. Evidence artifact: `benchmarks/results/identity/<date>/jwks-read.csv` should include scheduled and emergency rotation overlap windows.
05. Evidence artifact: `benchmarks/results/identity/<date>/webauthn-authenticate.csv` should record ceremony server time separately from human/device time.
06. Evidence artifact: `benchmarks/results/identity/<date>/webauthn-register.csv` should record attestation, AAGUID policy, and credential-bind time.
07. Evidence artifact: `benchmarks/results/identity/<date>/scim-write.csv` should record create, update, deactivate, group add, group remove, retry, and idempotency cases.
08. Evidence artifact: `benchmarks/results/identity/<date>/step-up.csv` should record ACR challenge, factor verification, JIT approval check, and token mint.
09. Evidence artifact: `benchmarks/results/identity/<date>/audit-emit.csv` should record event enqueue, audit-chain commit, and observability export time.
10. Evidence artifact: `benchmarks/results/identity/<date>/recovery.csv` should record recovery grant issue, session rotation, and delegated-token revocation time.
11. Workload fixture: demo_trial OCI Always Free must run on an Ampere A1-compatible linux/arm64 profile with 4 OCPU and 24 GiB RAM total.
12. Workload fixture: guest AWS must record instance family, region, VPC path, KMS adapter, and state backend.
13. Workload fixture: guest OCI paid must record compute shape, region, VCN path, Vault adapter, Object Storage backend, and lock table.
14. Workload fixture: on-prem must record customer hardware class, local HSM class, local DNS, storage, and disconnected-mode toggle.
15. Workload fixture: colo must record facility region, network underlay, BGP/MetalLB path, HSM class, and remote-hands dependency state.
16. Workload fixture: Oyatie-as-cloud-provider must record provider cell class, cloud-iam seam, cloud-kms seam, and cloud-storage state backend.
17. Acceptance rule: a tenant_class target passes only when p99, error rate, and audit completeness all meet target in the same run.
18. Acceptance rule: a throughput target passes only if the run sustains the target for at least 30 minutes without unbounded queue growth.
19. Acceptance rule: a SCIM target passes only with 429/retry/idempotency cases included.
20. Acceptance rule: a WebAuthn target passes only with valid, invalid, replayed, revoked-AAGUID, and unknown-AAGUID ceremonies included.
21. Acceptance rule: a recovery target passes only if operator-decrypt bypass remains impossible and audit evidence precedes session rebinding.
22. Acceptance rule: context readiness cannot pass with only public-cloud numbers when the tenant_class claims all six contexts.
23. Acceptance rule: OCI Always Free demo_trial cannot pass if any workload spills into paid OCI, AWS, or Oyatie-public paid capacity.
24. Acceptance rule: all benchmark harnesses must respect Rust-strict doctrine or carry an approved exception ADR.
25. Acceptance rule: benchmark reports must state whether numbers are measured, modeled, estimated, or source limits.
26. Current gap: none of these build-phase artifacts exists in the identity path at audit time.
27. Current gap: benchmark target tables in this document are therefore planning targets, not release evidence.
28. Current gap: local `benchmarks/okta-auth0-entra-vs-oyatie.md` should be revised to point at raw result artifacts after they exist.
29. Current gap: local SLO files should be linked to benchmark result families so SLO objectives and measured distributions stay consistent.
30. Current gap: local capability availability should include the same benchmark dimensions used here.
31. Wave 14 aggregation instruction: do not treat performance targets as measured readiness.
32. Wave 14 aggregation instruction: keep benchmark evidence blocked until raw result files and harness provenance are attached.
33. Wave 14 aggregation instruction: require OS/arch/context/tenant-class fields in every result artifact.
34. Wave 14 aggregation instruction: require distinct demo_trial OCI results instead of inheriting demo_trial-General numbers.
35. Wave 14 aggregation instruction: require per-context overlays to be measured independently.
