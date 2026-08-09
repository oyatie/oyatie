# cloud-iam performance benchmark numbers - 2026-05-20

## Citation anchor block

1. Canonical sequence: `docs/decisions/ADR-0700-ci-admission-live-apex.md` D-15 through D-20, especially D-20.152 requiring benchmark disclosure by OS, architecture, deployment context, and tenant class.
2. Machine-readable direction: `specs/master-plan-sequencing.json` lines 704-867 for all six deployment contexts, OpenTofu, OS support, Rust-only build policy, and OCI Always Free.
3. Service-local tenant_class policy targets: ADR-0329 + ADR-0330 + ADR-0331 lines 12-78 and `microservices/cloud-iam/benchmarks/cloud-iam-vs-aws-iam-vs-gcp-iam-vs-okta-vs-entra.md` lines 1-88.
4. Product-level SLO: `docs/products/cloud/PRD.md` line 172 states IAM plus STS p99 <= 100 ms and 99.99% availability.
5. Counterpart public numbers: AWS IAM/STS quotas at `https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_iam-quotas.html`, Google IAM quotas at `https://docs.cloud.google.com/iam/quotas`, Microsoft Graph throttling at `https://learn.microsoft.com/en-us/graph/throttling-limits`, and Microsoft access-token lifetime docs at `https://learn.microsoft.com/en-us/entra/identity-platform/access-tokens`.

## Explicit Methodology Disclosure

These are target numbers and sourced comparator limits, not fresh measured Oyatie benchmark results.
The existing service benchmark document claims measurements from 2026-04-22 through 2026-05-14, but this audit found no signed benchmark evidence under `microservices/cloud-iam/`.
Measured benchmarks must be added in the build phase under ADR-0212 with signed evidence, reproducible harnesses, hardware/context disclosures, and CI retention.
Counterpart "numbers" below are public quotas, limits, documented token lifetimes, and publicly documented throughput ceilings where official docs publish them.
When counterpart latency is not published, this report does not invent measured latency.
When this report uses a modeled latency target, it marks the number as an Oyatie target.

## 1. Methodology

M1 Benchmark claim type: target, public quota, public limit, or local unverified claim.
M2 Target workload A: Cedar authorize decision with hot entity cache and no provider call.
M3 Target workload B: HTTP/3 authorization call through `cloud-iam` API with tenant, principal, action, resource, and context binding.
M4 Target workload C: scoped token issue with policy binding and audit event emission.
M5 Target workload D: token introspection with active-session lookup.
M6 Target workload E: token revocation and propagation to in-cell caches.
M7 Target workload F: SAML/OIDC IdP assertion handling with metadata validation and JIT principal materialization.
M8 Target workload G: Cedar-to-provider IAM translation and queued downstream apply.
M9 Target workload H: JIT elevation request, approval, activation, use, and revoke events.
M10 Latency dimensions: p50, p95, p99 for authorize, token issue, introspect, revoke, federation login, translation enqueue, and JIT activation.
M11 Throughput dimensions: sustained RPS, burst RPS, per-tenant RPS, per-cell RPS, and provider-translation jobs per minute.
M12 Scale dimensions: principals, roles, role bindings, IdPs, active sessions, audit events per second, and retained policy versions.
M13 Resilience dimensions: revocation propagation, IdP metadata poll interval, audit-chain anchoring delay, failover RTO, and failover RPO.
M14 Disclosure context: Linux amd64 and arm64 are assumed for server targets until `supported-oses.json` lands.
M15 Disclosure architecture: OCI Always Free demo_trial tenant_class uses arm64 Ampere A1 target capacity per canonical profile.
M16 Disclosure OS: no Tier-1 OS-specific measured numbers exist yet.
M17 Disclosure deployment contexts: all six contexts are included as target overlays because ADR-0328 requires `cloud-iam` in all six.
M18 Disclosure tenant classes: demo_trial tenant_class, paid tenant_class, paid tenant_class, paid tenant_class.
M19 Disclosure measurement gap: no `src/`, `tests/`, benchmark harness, OpenSLO, or IaC exists under this service path today.
M20 Measurement stop condition for future build phase: signed benchmark evidence must include commit SHA, OS, architecture, context, tenant class, dataset size, concurrency, p50/p95/p99, and raw run artifact.

## 2. Counterpart Numbers

AWS-01 AWS STS default request quota: 600 requests per second per account per Region for credentialed STS operations.
AWS-02 AWS STS operations sharing that quota include `AssumeRole`, `DecodeAuthorizationMessage`, `GetAccessKeyInfo`, `GetCallerIdentity`, `GetFederationToken`, and `GetSessionToken`.
AWS-03 AWS STS cross-account AssumeRole quota is consumed by the calling account, not the target account.
AWS-04 AWS role session duration can range from 900 seconds to a role-configured maximum of 1 to 12 hours.
AWS-05 AWS customer managed policies per account default quota: 1,500; maximum quota: 10,000.
AWS-06 AWS groups per account default quota: 300; maximum quota: 500.
AWS-07 AWS roles per account default quota: 1,000; maximum quota: 10,000.
AWS-08 AWS managed policies per role default quota: 10; maximum quota: 25.
AWS-09 AWS OIDC providers per account default quota: 100; maximum quota: 700.
AWS-10 AWS role trust policy default length: 2,048 characters; maximum quota: 8,192 characters.
AWS-11 AWS inline role policy aggregate size: 10,240 characters.
AWS-12 AWS session policy and managed policy ARN packed input: 2,048 characters plus packed binary internal limit.
AWS-13 AWS session tags per session: 50.
AWS-14 AWS SAML response size limit for AssumeRoleWithSAML: 100,000 base64 encoded characters.
AWS-15 AWS IAM service cost: IAM, Identity Center, and STS are offered at no additional charge, with charges only for other AWS service usage.

GCP-01 Google IAM v1 read requests: 6,000 per project per minute.
GCP-02 Google IAM v1 write requests: 600 per project per minute.
GCP-03 Google IAM v2 deny-policy read requests: 5 per project per minute.
GCP-04 Google IAM v2 deny-policy write requests: 5 per project per minute.
GCP-05 Google IAM v3 principal access boundary read requests: 5 per project per minute.
GCP-06 Google IAM v3 principal access boundary write requests: 5 per project per minute.
GCP-07 Google Workload Identity Federation read requests: 600 per project per minute and 6,000 per client per minute.
GCP-08 Google Workload Identity Federation write requests: 60 per project per minute and 600 per client per minute.
GCP-09 Google Workforce Identity Federation read/update requests: 120 per organization per minute.
GCP-10 Google Workforce Identity Federation create/delete/undelete requests: 60 per organization per minute.
GCP-11 Google Service Account Credentials API credential generation: 60,000 per project per minute.
GCP-12 Google Service Account Credentials API sign JWT/blob: 60,000 per project per minute.
GCP-13 Google Security Token Service exchange requests: 6,000 per project per minute, global or regional.
GCP-14 Google Security Token Service introspection requests: 6,000 per project per minute, global or regional.
GCP-15 Google custom roles per organization or project: 300.
GCP-16 Google allow policy total principals limit: 1,500.
GCP-17 Google deny policies per resource: 500.
GCP-18 Google principal access boundary policies per organization: 1,000.
GCP-19 Google service account keys per service account: 10.
GCP-20 Google OAuth 2.0 access token maximum lifetime: 3,600 seconds by default, extendable to 43,200 seconds for selected service accounts.

MS-01 Microsoft Graph global limit: 130,000 requests per 10 seconds per app across all tenants.
MS-02 Microsoft Graph assignment service limit: 350 requests per 10 seconds per app per tenant.
MS-03 Microsoft Graph assignment service limit: 700 requests per 10 seconds per tenant for all apps.
MS-04 Microsoft Graph assignment service long-window limit: 10,000 requests per 3,600 seconds per app per tenant.
MS-05 Microsoft Graph assignment service tenant long-window limit: 20,000 requests per 3,600 seconds.
MS-06 Microsoft Graph invitation manager limit: 150 requests per 5 seconds per tenant for all apps.
MS-07 Microsoft Graph directory-related resource-unit quotas vary by tenant/app size; service-specific docs define S/M/L bands.
MS-08 Microsoft access token default lifetime: random 60 to 90 minutes, average 75 minutes.
MS-09 Microsoft access token default variability spreads demand to avoid hourly spikes.
MS-10 Microsoft tenants without Conditional Access can have two-hour default access token lifetime for some clients such as Teams and Microsoft 365.
MS-11 Microsoft long-lived token lifetime range can be 20 to 28 hours under documented Continuous Access Evaluation behavior.
MS-12 Conditional Access sign-in frequency interacts with the token lifetime range rather than replacing it.
MS-13 PIM supports time-bound role assignments with start and end dates.
MS-14 PIM activation can require MFA, justification, approval, notifications, and audit history.
MS-15 Entra governance docs do not publish a universal IAM authorization latency number; use Graph and token public limits instead of invented latency.

## 3. Oyatie Target Numbers By Tenant Class And Deployment Context

### 3.1 demo_trial tenant_class targets

demo_trial tenant_class baseline interpretation: low-cost development, trial, and small-tenant identity with local password plus TOTP, User and ServiceAccount principals, no external IdP federation unless explicitly upgraded.
demo_trial tenant_class OCI interpretation: `guest-on-oci` demo_trial tenant_class must fit OCI Always Free; this overrides the current ADR-0331 tenant_class adoption template `$45/mo` line for that context.

| Context | Authorize p95 | Authorize p99 | HTTP authorize p95 | Token issue p95 | Introspect p95 | Revoke propagation p95 | Sustained authorize RPS | Burst authorize RPS | Principal ceiling | Role ceiling | IdP ceiling | Audit retention |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| oyatie-public-cloud | 350 us | 900 us | 4 ms | 25 ms | 4 ms | 250 ms | 200 | 800 | 50 | 25 | 0 external | 90 days |
| guest-on-aws | 400 us | 1.0 ms | 5 ms | 35 ms | 5 ms | 300 ms | 150 | 600 | 50 | 25 | 0 external | 90 days |
| guest-on-oci | 450 us | 1.2 ms | 6 ms | 40 ms | 6 ms | 350 ms | 120 | 500 | 40 | 20 | 0 external | 90 days |
| on-prem | 500 us | 1.5 ms | 8 ms | 50 ms | 8 ms | 500 ms | 100 | 400 | 50 | 25 | 0 external | 90 days |
| colo | 500 us | 1.5 ms | 8 ms | 50 ms | 8 ms | 500 ms | 100 | 400 | 50 | 25 | 0 external | 90 days |
| oyatie-as-cloud-provider | 350 us | 900 us | 4 ms | 25 ms | 4 ms | 250 ms | 200 | 800 | 50 | 25 | 0 external | 90 days |

demo_trial tenant_class target B1: Cedar hot-path p95 follows the current local tenant_class policy at 350 us in-process for normal public-cloud contexts.
demo_trial tenant_class target B2: HTTP p95 follows the current local tenant_class policy at 4 ms for the strongest contexts.
demo_trial tenant_class target B3: OCI Always Free gets lower RPS and lower principal ceiling to respect the 4 OCPU/24 GB shared budget.
demo_trial tenant_class target B4: external IdP ceiling is zero because tenant_class policy demo_trial tenant_class does not enable Federated principal type.
demo_trial tenant_class target B5: demo_trial tenant_class audit retention remains 90 days as local tenant_class policy states.
demo_trial tenant_class target B6: token issue p95 is a target because no measured harness exists.
demo_trial tenant_class target B7: revoke propagation p95 is deliberately slower than paid tenant_class policies because demo_trial tenant_class has simpler cache topology.
demo_trial tenant_class target B8: p99 target remains below the product-level IAM/STS p99 <= 100 ms when provider calls are out of path.

### 3.2 paid tenant_class targets

paid tenant_class baseline interpretation: paid baseline with Workload principal support, SAML/OIDC federation, STS, SPIFFE/SPIRE, moderate scale, and one-year audit retention.

| Context | Authorize p95 | Authorize p99 | HTTP authorize p95 | Token issue p95 | Introspect p95 | Revoke propagation p95 | Sustained authorize RPS | Burst authorize RPS | Principal ceiling | Role ceiling | IdP ceiling | Audit retention |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| oyatie-public-cloud | 250 us | 700 us | 3 ms | 20 ms | 3 ms | 120 ms | 2,000 | 6,000 | 5,000 | 250 | 4 | 1 year |
| guest-on-aws | 300 us | 900 us | 4 ms | 30 ms | 4 ms | 150 ms | 1,500 | 4,500 | 5,000 | 250 | 4 | 1 year |
| guest-on-oci | 350 us | 1.0 ms | 5 ms | 35 ms | 5 ms | 180 ms | 800 | 2,400 | 2,500 | 200 | 3 | 1 year |
| on-prem | 400 us | 1.2 ms | 7 ms | 45 ms | 7 ms | 250 ms | 800 | 2,000 | 5,000 | 250 | 4 | 1 year |
| colo | 400 us | 1.2 ms | 7 ms | 45 ms | 7 ms | 250 ms | 800 | 2,000 | 5,000 | 250 | 4 | 1 year |
| oyatie-as-cloud-provider | 250 us | 700 us | 3 ms | 20 ms | 3 ms | 120 ms | 2,000 | 6,000 | 5,000 | 250 | 4 | 1 year |

paid tenant_class target S1: Cedar in-process p95 follows local tenant_class policy at 250 us where Oyatie controls runtime placement.
paid tenant_class target S2: HTTP p95 follows local tenant_class policy at 3 ms for public/provider contexts.
paid tenant_class target S3: tenant principal ceiling follows local tenant_class policy at 5,000 except constrained OCI target.
paid tenant_class target S4: IdP ceiling is four named providers for baseline SAML/OIDC support.
paid tenant_class target S5: token issue p95 stays below 45 ms even on-prem/colo.
paid tenant_class target S6: provider translation is asynchronous and not included in hot-path authorize p95.
paid tenant_class target S7: audit retention is one year.
paid tenant_class target S8: STS issuance target remains below product p99 <= 100 ms.

### 3.3 paid tenant_class targets

paid tenant_class baseline interpretation: production scale with broader IdP catalog, cross-cloud provider translation, 250k principal class, and three-year audit retention.

| Context | Authorize p95 | Authorize p99 | HTTP authorize p95 | Token issue p95 | Introspect p95 | Revoke propagation p95 | Sustained authorize RPS | Burst authorize RPS | Principal ceiling | Role ceiling | IdP ceiling | Audit retention |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| oyatie-public-cloud | 200 us | 600 us | 2 ms | 15 ms | 2 ms | 80 ms | 15,000 | 45,000 | 250,000 | 5,000 | 10 | 3 years |
| guest-on-aws | 250 us | 800 us | 3 ms | 25 ms | 3 ms | 100 ms | 10,000 | 30,000 | 250,000 | 5,000 | 10 | 3 years |
| guest-on-oci | 300 us | 900 us | 4 ms | 30 ms | 4 ms | 120 ms | 5,000 | 15,000 | 100,000 | 3,000 | 8 | 3 years |
| on-prem | 350 us | 1.1 ms | 6 ms | 40 ms | 6 ms | 180 ms | 5,000 | 12,000 | 250,000 | 5,000 | 10 | 3 years |
| colo | 350 us | 1.1 ms | 6 ms | 40 ms | 6 ms | 180 ms | 5,000 | 12,000 | 250,000 | 5,000 | 10 | 3 years |
| oyatie-as-cloud-provider | 200 us | 600 us | 2 ms | 15 ms | 2 ms | 80 ms | 15,000 | 45,000 | 250,000 | 5,000 | 10 | 3 years |

paid tenant_class target G1: Cedar p95 follows the current local tenant_class policy at 200 us for controlled contexts.
paid tenant_class target G2: HTTP p95 follows the tenant_class policy at 2 ms for controlled contexts.
paid tenant_class target G3: revoke propagation p95 target is 80 ms, matching the reference implementation guarantee.
paid tenant_class target G4: principal ceiling follows the tenant_class policy at 250,000 where capacity is not intentionally constrained.
paid tenant_class target G5: OCI guest target is lower because guest-on-oci must respect a customer tenancy and may be capacity constrained.
paid tenant_class target G6: IdP catalog expands to 10 providers, including Auth0/Ping/OneLogin/AWS Identity Center/GCP Workforce/Entra External ID from the tenant_class policy.
paid tenant_class target G7: provider translation enqueue target should be <= 250 ms p95 but provider apply completion is governed by provider quotas.
paid tenant_class target G8: audit retention is three years.

### 3.4 paid tenant_class targets

paid tenant_class baseline interpretation: hyperscaler/single-tenant capable, sovereign regulated support, unlimited design claim converted here to engineered planning targets.

| Context | Authorize p95 | Authorize p99 | HTTP authorize p95 | Token issue p95 | Introspect p95 | Revoke propagation p95 | Sustained authorize RPS | Burst authorize RPS | Principal planning target | Role planning target | IdP ceiling | Audit retention |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| oyatie-public-cloud | 150 us | 500 us | 1.5 ms | 10 ms | 1.5 ms | 50 ms | 100,000 | 300,000 | 10,000,000 | 100,000 | 50 | 7 years |
| guest-on-aws | 200 us | 700 us | 2.5 ms | 20 ms | 2.5 ms | 70 ms | 60,000 | 180,000 | 5,000,000 | 75,000 | 50 | 7 years |
| guest-on-oci | 250 us | 800 us | 3.5 ms | 25 ms | 3.5 ms | 90 ms | 30,000 | 90,000 | 2,000,000 | 50,000 | 30 | 7 years |
| on-prem | 300 us | 1.0 ms | 5 ms | 35 ms | 5 ms | 120 ms | 30,000 | 75,000 | 5,000,000 | 75,000 | 50 | 7 years |
| colo | 300 us | 1.0 ms | 5 ms | 35 ms | 5 ms | 120 ms | 30,000 | 75,000 | 5,000,000 | 75,000 | 50 | 7 years |
| oyatie-as-cloud-provider | 150 us | 500 us | 1.5 ms | 10 ms | 1.5 ms | 50 ms | 100,000 | 300,000 | 10,000,000 | 100,000 | 50 | 7 years |

paid tenant_class target P1: in-process p95 follows the tenant_class policy at 150 us where Oyatie controls placement.
paid tenant_class target P2: HTTP p95 follows the tenant_class policy at 1.5 ms for controlled contexts.
paid tenant_class target P3: this report replaces the tenant_class policy's word "unlimited" with planning targets because unlimited is not testable.
paid tenant_class target P4: provider contexts have lower targets because provider quotas and customer tenancy topology can constrain translation and federation flows.
paid tenant_class target P5: IdP ceiling is modeled at 50 for controlled contexts and 30 for OCI guest.
paid tenant_class target P6: audit retention is seven years, matching the tenant_class policy.
paid tenant_class target P7: revocation propagation p95 target is 50 ms in controlled contexts and 120 ms in on-prem/colo.
paid tenant_class target P8: JIT elevation activation target should be <= 2 seconds p95 excluding human approval wait.

## 4. Per-Context Overlay

Context O1 `oyatie-public-cloud`: best latency and throughput because Oyatie controls cell placement, cache topology, audit-chain path, and HSM/session signing.
Context O2 `oyatie-public-cloud`: provider translation still has downstream apply latency, but hot-path Cedar authorize must remain provider-free.
Context O3 `guest-on-aws`: STS/role-apply paths must respect AWS STS 600 RPS/account/Region default and IAM eventual consistency.
Context O4 `guest-on-aws`: Cedar authorize hot path remains local, but provider apply and role refresh are rate-limited by AWS account quotas.
Context O5 `guest-on-oci`: demo_trial tenant_class is capacity-limited to OCI Always Free resources under the canonical profile.
Context O6 `guest-on-oci`: paid tenant_class can exceed Always Free only if the tenant selects paid baseline; demo_trial tenant_class must not silently spill to paid resources.
Context O7 `on-prem`: p95 HTTP targets are looser because customer network, HSM, IdP, and hardware token paths vary.
Context O8 `on-prem`: provider translation may be absent or replaced by local directory/HSM integrations.
Context O9 `colo`: p95 HTTP targets match on-prem until colo-specific network and facility APIs are modeled.
Context O10 `colo`: identity remains Oyatie/Cedar; facility systems must not become authority.
Context O11 `oyatie-as-cloud-provider`: target matches public-cloud controlled path because cloud-iam is the provider-plane identity service.
Context O12 `oyatie-as-cloud-provider`: paid tenant_class must support single-tenant dedicated control planes and regional/sovereign overlays.

## 5. Comparison Narrative

Comparison C1 STS throughput: AWS publishes 600 RPS per account per Region for STS; Oyatie demo_trial tenant_class boundary targets exceed that only in controlled hot-path authorize, not provider STS apply.
Comparison C2 Google STS throughput: Google publishes 6,000 token exchange requests per project per minute; Oyatie paid tenant_class token targets require careful per-tenant capacity isolation.
Comparison C3 Microsoft Graph global throughput: Microsoft publishes 130,000 requests per 10 seconds per app across tenants; Oyatie paid tenant_class targets approach that class only in controlled provider-plane contexts.
Comparison C4 Token lifetime: Google and Microsoft public token lifetimes are hour-class; Oyatie scoped tokens should remain configurable but short-lived for cloud actions.
Comparison C5 Role scale: AWS default 1,000 roles and max 10,000 roles per account; Oyatie paid tenant_class 5,000 roles per tenant is within AWS max class but needs sharding for paid tenant_class.
Comparison C6 Policy/principal scale: Google allow policy 1,500 principal limit is much lower than Oyatie paid tenant_class principal target because Oyatie should not materialize one giant provider policy.
Comparison C7 Provider translation: public counterpart quotas imply translation must batch, shard, and degrade gracefully rather than blocking authorization.
Comparison C8 Hot-path latency: public docs do not publish comparable IAM authorization p95/p99; Oyatie local targets are product engineering targets, not benchmark proof.
Comparison C9 Revocation: Oyatie 80 ms/50 ms p95 targets are aggressive and need build-phase measurement.
Comparison C10 Audit anchoring: hourly HSM/BLAKE3 roots are a local evidence target; counterpart public docs provide audit logs but not equivalent cross-cloud digest provenance.
Comparison C11 OCI demo_trial tenant_class: no counterpart has the exact canonical constraint; Oyatie must prove the service fits within Always Free when deployed as guest-on-oci demo_trial tenant_class.
Comparison C12 Hyperscaler maturity: parity cannot be claimed until quota handling, context deployment, SLO files, and measured benchmarks land.

## 6. Build-Phase Measurement Backlog

Measure 1: hot Cedar authorize, in-process, 1/10/100 tenants, 10k/100k/1M principals.
Measure 2: HTTP authorize over HTTP/3 and HTTP/2 fallback, with request signing and tenant binding.
Measure 3: token issue with audit-chain event append and HSM/signing dependency mocked and real.
Measure 4: token introspection with cache hit, cache miss, and expired-token paths.
Measure 5: token revocation propagation across one process, one cell, cross-cell, and provider-plane contexts.
Measure 6: SAML ACS handling with 1KB, 25KB, and 100KB assertions.
Measure 7: OIDC callback handling with IdP metadata cache hit and stale metadata refresh.
Measure 8: Cedar-to-AWS translation enqueue, provider apply, and eventual consistency observation.
Measure 9: Cedar-to-GCP translation enqueue and IAM write quota behavior.
Measure 10: Cedar-to-Azure translation enqueue and Graph throttling behavior.
Measure 11: JIT elevation approval path excluding human wait and including all audit events.
Measure 12: audit-chain event emission under authorize-heavy load.
Measure 13: IdP metadata polling storm with certificate expiry.
Measure 14: OCI Always Free demo_trial tenant_class soak test for 24 hours at demo_trial tenant_class target RPS.
Measure 15: Tier-1 OS matrix smoke and benchmark on Linux amd64/arm64 plus macOS M5+ developer-tool path.

## 7. Measurement Acceptance Criteria

Acceptance A01: each benchmark run must record git commit, service version, tenant class, deployment context, OS, architecture, CPU class, memory class, and storage class.
Acceptance A02: each run must identify whether the value is in-process, loopback HTTP, same-cell network, cross-cell network, or provider API path.
Acceptance A03: each run must state whether provider API calls are real, mocked, replayed, or disabled.
Acceptance A04: each run must emit p50, p90, p95, p99, max, error rate, timeout rate, and saturation point.
Acceptance A05: each run must preserve request mix, read/write ratio, tenant count, policy count, principal count, role count, token count, and IdP count.
Acceptance A06: demo_trial tenant_class `guest-on-oci` runs must include an OCI Always Free budget attestation and resource inventory.
Acceptance A07: demo_trial tenant_class `guest-on-oci` runs must fail if paid compute, paid database, paid load balancer, paid NAT, or paid managed HSM is present.
Acceptance A08: demo_trial tenant_class non-OCI runs must state whether the cost target is local developer, public-cloud shared, or paid baseline.
Acceptance A09: paid tenant_class runs must include at least one external IdP and one workload identity path.
Acceptance A10: paid tenant_class runs must include at least three provider translation adapters or clearly state which adapter is under test.
Acceptance A11: paid tenant_class runs must include a dedicated-tenant isolation mode or state that the run is a capacity simulation.
Acceptance A12: authorize hot-path measurements must not include provider IAM writes.
Acceptance A13: provider translation measurements must separate enqueue latency from downstream provider apply latency.
Acceptance A14: token issue measurements must separate signing, audit append, and response serialization costs.
Acceptance A15: token introspection measurements must include cache hit, cache miss, revoked token, expired token, and wrong-tenant token paths.
Acceptance A16: revocation propagation measurements must include single-process, same-cell, cross-cell, and provider-observation paths.
Acceptance A17: federation callback measurements must include SAML signature validation and OIDC JWKS validation.
Acceptance A18: IdP metadata refresh measurements must include cache hit, stale cache, failed fetch, bad signature, and rollover.
Acceptance A19: JIT activation measurements must exclude human approval waiting time but include state transition and audit emission.
Acceptance A20: access-review measurements cannot be claimed until the service owns or delegates access reviews explicitly.
Acceptance A21: Conditional Access measurements cannot be claimed until a risk/device/network signal contract exists.
Acceptance A22: analyzer measurements must include policy parse, Cedar evaluation, provider projection analysis, and explanation generation.
Acceptance A23: cache measurements must report hit ratio, invalidation lag, stale-read rate, and memory footprint.
Acceptance A24: audit-chain measurements must report append latency, signing latency, verification latency, and export throughput.
Acceptance A25: HSM-dependent measurements must identify real HSM, simulated HSM, software key, or unavailable path.
Acceptance A26: every benchmark artifact must be signed or linked to a signed evidence bundle.
Acceptance A27: every published number must have provenance: measured, target, extrapolated, vendor quota, or industry estimate.
Acceptance A28: target numbers in this report must be replaced by measured numbers only after the build phase produces evidence.
Acceptance A29: a failed run must publish error taxonomy rather than silently dropping outliers.
Acceptance A30: p99 claims require enough sample size to make p99 meaningful for the workload.
Acceptance A31: throughput claims require a documented saturation method and backpressure behavior.
Acceptance A32: concurrent-operation claims require explicit tenant isolation and per-tenant fairness reporting.
Acceptance A33: provider quota tests must run below account safety limits unless an approved test account exists.
Acceptance A34: Microsoft Graph tests must obey service-specific throttling and capture `Retry-After` or throttle headers where available.
Acceptance A35: Google IAM tests must capture quota project and organization context.
Acceptance A36: AWS IAM/STS tests must capture account, Region, endpoint choice, and quota override status.
Acceptance A37: OCI tests must capture tenancy, region, compartment, dynamic group, and policy context.
Acceptance A38: on-prem tests must capture hardware, HSM, directory, and network topology.
Acceptance A39: colo tests must capture facility network and external dependency latency.
Acceptance A40: public-cloud/provider-plane tests must capture cell placement and cross-region routing.

## 8. Benchmark Interpretation Risks

Risk B01: counterpart public docs often publish quotas rather than latency; therefore parity cannot be inferred from latency numbers alone.
Risk B02: AWS IAM and STS quotas are not equivalent to Cedar hot-path authorize throughput.
Risk B03: Google IAM write quotas are not equivalent to local authorization evaluation throughput.
Risk B04: Microsoft Graph identity throttling is not equivalent to Entra policy decision latency.
Risk B05: provider eventual consistency can dominate user-visible completion even when Oyatie local work is fast.
Risk B06: a provider adapter can meet enqueue SLO while failing apply SLO; both must be reported separately.
Risk B07: demo_trial tenant_class Always Free can pass latency but fail sustained throughput under bursty federation workloads.
Risk B08: short-lived token issuance can stress audit-chain storage more than Cedar evaluation.
Risk B09: revocation propagation can be locally fast but externally delayed by provider tokens and caches.
Risk B10: IdP metadata rollover is a low-frequency path but a high-severity latency and reliability risk.
Risk B11: HSM signing can be absent in demo_trial tenant_class and mandatory in paid tenant_class; one benchmark cannot represent all tenant_class policies.
Risk B12: on-prem and colo variance requires ranges or certified hardware profiles, not a single universal number.
Risk B13: OS matrix runs may expose performance cliffs from cryptography, kernel networking, or packaging differences.
Risk B14: tenant isolation may reduce headline throughput but is required for paid tenant_class and paid tenant_class credibility.
Risk B15: audit retention targets can shift cost and write amplification, especially under OCI Always Free.
Risk B16: access-review and governance flows are workflow-heavy; raw RPS is the wrong primary metric for those capabilities.
Risk B17: admin UX latency matters for parity but is not measured by backend API benchmarks.
Risk B18: policy analyzer latency may grow with graph depth, provider projection count, and explanation verbosity.
Risk B19: benchmark fixtures must include denial cases, not only allow cases.
Risk B20: this report remains a target-number artifact until signed measured benchmark bundles land.
