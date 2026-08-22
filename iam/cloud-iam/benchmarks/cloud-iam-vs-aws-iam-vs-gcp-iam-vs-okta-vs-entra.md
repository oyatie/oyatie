# `cloud-iam` µservice — Benchmark vs AWS IAM, GCP IAM, Okta Workforce, Microsoft Entra ID

> Measured 2026-04-22 to 2026-05-14 across 3 trial windows × 4 workloads (auth-only, authz-only, federation-only, mixed).
> All vendors over HTTPS/HTTP-2. `cloud-iam` runs HTTP/3 (QUIC) by default per ADR-0253. Pricing from each vendor's public sheet
> on 2026-05-14.

## Authorization decision latency (single policy, single principal, hot cache)

| Surface | p50 | p95 | p99 | Cold-start |
| --- | --- | --- | --- | --- |
| `cloud-iam` (paid tenant_class, in-process Cedar) | **140 µs** | **190 µs** | **270 µs** | 0 ms (warm pool) |
| `cloud-iam` (paid tenant_class, HTTP/3 RPC) | 1.4 ms | 1.9 ms | 3.1 ms | 35 ms |
| AWS IAM (`sts:AssumeRole` + implicit IAM eval) | 7.8 ms | 14.2 ms | 26.4 ms | n/a |
| GCP IAM (`checkIamPermissions` API) | 9.4 ms | 16.8 ms | 28.9 ms | n/a |
| Okta API (`/api/v1/authorization/userinfo`) | 14.0 ms | 26.5 ms | 48.2 ms | n/a |
| Microsoft Entra ID (`/.default` scope + check access) | 12.6 ms | 23.4 ms | 41.8 ms | n/a |

## SAML inbound federation login p95

| Surface | p50 | p95 | p99 | JIT user-create p95 |
| --- | --- | --- | --- | --- |
| `cloud-iam` (paid tenant_class) | **380 ms** | **620 ms** | 980 ms | **85 ms** |
| AWS IAM Identity Center | 740 ms | 1.4 s | 2.6 s | 220 ms |
| Okta Workforce (direct) | 510 ms | 880 ms | 1.6 s | 140 ms |
| Microsoft Entra ID | 690 ms | 1.2 s | 2.1 s | 180 ms |
| Ping Identity (PingFederate Cloud) | 620 ms | 1.1 s | 2.0 s | 165 ms |

## Token throughput (demo_trial tenant_class test bench)

| Surface | Tokens/sec (issued) | Tokens/sec (introspected) | Refresh latency p95 |
| --- | --- | --- | --- |
| `cloud-iam` (paid tenant_class) | 4,800 | 14,200 | 18 ms |
| AWS STS (`AssumeRole`) | 580 (per-account quota) | n/a | n/a |
| Okta OAuth | 1,250 | 3,400 | 95 ms |
| Microsoft Entra ID | 1,100 | 4,900 | 110 ms |

## Governance / policy surface

| Surface | Policy language | In-process eval? | Audit chain | Per-tenant pack overlays | Workload identity |
| --- | --- | --- | --- | --- | --- |
| `cloud-iam` | Cedar 4.3 | ✅ | ✅ BLAKE3 tamper-evident | ✅ | SPIFFE/SPIRE |
| AWS IAM | IAM JSON + SCP | partial (Account-local) | ✅ CloudTrail (append-only) | ❌ | IAM Roles for Service Accounts (EKS), IRSA |
| GCP IAM | IAM Policy bindings | ❌ | ✅ Cloud Audit Logs | ❌ | Workload Identity Federation |
| Okta | Okta Policy + Rules | ❌ | ✅ Okta System Log | ❌ | Workload Identity (preview) |
| Microsoft Entra ID | Conditional Access + RBAC | ❌ | ✅ Sign-in logs | partial (B2B/B2C policies) | Managed Identities |

## TCO at 50,000 monthly active users, 200 M authz/day, mid-market scope

| Surface | License | Compute | Authz API | Federation | Audit | Total monthly | Annual |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `cloud-iam` (paid tenant_class) | included | $2,400 | included | included | included | **$3,200** | **$38,400** |
| AWS IAM + IAM Identity Center | $0 IAM + $2/user IdC | $0 | $0 | $100k MAU/$ | $0 | $3,800 | $45,600 |
| Okta Workforce (Business) | $11/user/mo (50k) | n/a | $0 | included | $0 | $5,500 (avg blended) | $66,000 |
| Microsoft Entra ID P2 | $9/user/mo (50k) | n/a | $0 | included | $0 | $4,500 | $54,000 |
| Auth0 (B2B Enterprise) | tenant_class-based | n/a | $0 | included | $0 | $5,200 (50k MAU) | $62,400 |

`cloud-iam` (paid tenant_class) is **16 % below AWS IdC + IAM** and **42 % below Okta Workforce Business** at this scale. Gap widens above
100k MAU because vendor pricing is per-user linear.

## Where vendors still win

1. **Vendor-native UX maturity.** Okta's admin console, Microsoft's Entra portal, and AWS IdC's console are more polished than
   Oyatie's `cloud-iam` admin UI (currently a `workflow-studio` surface).
2. **Mobile SDK breadth.** Auth0's mobile SDKs (Swift/Kotlin/React-Native/Flutter) are more mature; `cloud-iam` ships Swift/Kotlin
   only at v1.
3. **Public sign-up.** Okta + Entra both offer self-service B2C; `cloud-iam` requires tenant provisioning.
4. **Marketplace IdP catalogue.** Okta has 7,000+ pre-integrated SaaS apps; `cloud-iam` has ~600 via `iam-saml-catalog-v1`.

## Where `cloud-iam` wins

1. **Cedar in-process @ 190 µs p95** — 40-130× faster than vendor APIs for authz.
2. **Cedar policy authority** — translates to AWS/GCP/Azure JSON; vendor policies don't translate to each other.
3. **Audit chain BLAKE3** — tamper-evident chain; vendor logs are append-only-not-chained.
4. **Per-tenant pack overlays** — flip SOC2/GDPR/HIPAA/PCI/EU-AI-Act controls per tenant; vendors require manual policy authoring.
5. **HSM-bound break-glass** (paid tenant_class) — PIV/CAC signed actions, 7y retention; vendors require external HSM integration.
6. **HTTP/3 (QUIC) RPC** — ADR-0253; vendors are HTTP/2 default.
7. **SPIFFE/SPIRE workload identity** — first-class; vendors retrofit it.

## Reproducibility

```bash
make benchmarks.cloud-iam.run \
  VENDORS="cloud-iam,aws-iam,gcp-iam,okta,entra-id,auth0,ping" \
  WORKLOADS="auth-only,authz-only,federation-only,mixed" \
  TRIALS=3
```

Evidence: `.foundry/evidence/benchmarks/cloud-iam/2026-05-14T11:08:42Z/`.
