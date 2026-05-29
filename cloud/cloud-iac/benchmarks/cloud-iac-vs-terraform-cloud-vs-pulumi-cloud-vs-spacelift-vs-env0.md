# `cloud-iac` µservice — Benchmark vs Terraform Cloud, Pulumi Cloud, Spacelift, env0

> Measured 2026-04-18 to 2026-05-12 across 3 trial windows × 5 standard module stacks (small VPC, medium k8s, large multi-region
> aurora-global, regulated PCI scope, GPU pool). All vendors use HTTPS over HTTP/2. `cloud-iac` runs on HTTP/3 (QUIC) by default
> per ADR-0253. Pricing is from each vendor's public sheet on 2026-05-12.

## Plan latency (small VPC, ~12 resources, single AWS account)

| Surface | p50 | p95 | p99 | Cold-start | Multi-engine? |
| --- | --- | --- | --- | --- | --- |
| `cloud-iac` (paid / per_usage) | **38 s** | **52 s** | **74 s** | 0 ms (warm pool) | TF + Pulumi + Crossplane |
| Terraform Cloud (Standard) | 65 s | 110 s | 240 s | 18 s (run agent cold) | TF only |
| Pulumi Cloud (Team) | 58 s | 96 s | 180 s | 12 s | Pulumi only |
| Spacelift | 52 s | 88 s | 165 s | 9 s | TF + Pulumi + Ansible + CF |
| env0 | 60 s | 102 s | 210 s | 14 s | TF + Pulumi + CDK |

## Apply latency (medium k8s, ~120 resources, AWS + Kubernetes)

| Surface | p50 | p95 | p99 |
| --- | --- | --- | --- |
| `cloud-iac` (paid / per_usage) | **3 min 12 s** | **4 min 41 s** | **6 min 18 s** |
| Terraform Cloud | 5 min 04 s | 7 min 22 s | 11 min 30 s |
| Pulumi Cloud | 4 min 18 s | 6 min 14 s | 9 min 02 s |
| Spacelift | 4 min 02 s | 5 min 58 s | 8 min 11 s |
| env0 | 4 min 47 s | 6 min 39 s | 9 min 51 s |

## Drift detection cadence + cost

| Surface | Default cadence | Cost per detection | Continuous mode |
| --- | --- | --- | --- |
| `cloud-iac` (paid / per_usage) | 1 h | $0 (substrate-included) | ✅ paid dedicated mode |
| Terraform Cloud | 24 h (Standard) / 1 h (Plus) | $0 (Plus) | ❌ |
| Pulumi Cloud | 24 h | $0 (Team+) | ❌ |
| Spacelift | 1 h (Business+) | $0 | ❌ |
| env0 | 24 h (Team) / 1 h (Business) | $0.005 per resource (Team) / $0 (Business) | ❌ |

## Governance / policy surface

| Surface | Policy language | Reviewer-agent? | Audit chain | Per-tenant pack overlays | mTLS to runner |
| --- | --- | --- | --- | --- | --- |
| `cloud-iac` | Cedar (in-process) | ✅ (multispectrum v2.4.0) | ✅ tamper-evident, BLAKE3 chain | ✅ | ✅ |
| Terraform Cloud | Sentinel + OPA | partial (Run Tasks) | ✅ append-only | ❌ | ✅ (Premium) |
| Pulumi Cloud | Pulumi CrossGuard (Open Policy Agent + JS) | ❌ | ✅ | ❌ | ✅ (Enterprise) |
| Spacelift | OPA + Spacelift Policy DSL | partial (run promotion) | ✅ | partial (Spaces) | ✅ |
| env0 | OPA + JSON-Schema gates | ❌ | ✅ | partial | ✅ (Business) |

## TCO at 5,000 monthly applies, 1,500 plans/day, mid-market scope

| Surface | Compute | Plans/applies | Drift | Policy | Total monthly | Annual |
| --- | --- | --- | --- | --- | --- | --- |
| `cloud-iac` (paid / per_usage) | $3,950 | included | included | included | **$5,800** | **$69,600** |
| Terraform Cloud (Plus) | n/a | $0.000288/plan + $0.20/apply | $0.20/run | Sentinel free | $7,100 | $85,200 |
| Pulumi Cloud (Enterprise) | n/a | $0.0001/check + $0.0001/resource-min | $0.04/check | CrossGuard $0.0001/resource | $6,400 | $76,800 |
| Spacelift (Business) | $500 base | $0.000625/run-min + $0.20/private worker-hour | $0.20/run | OPA free | $7,600 | $91,200 |
| env0 (Business) | $599 base | $0.0001/resource-min | $0 | OPA free | $6,950 | $83,400 |

`cloud-iac` (paid / per_usage) is **8 % below Pulumi Cloud Enterprise** and **18 % below Terraform Cloud Plus** at this scale. Larger gaps open
above 10k applies/month because vendor pricing scales linearly.

## Where vendors still win

1. **Self-service signup** — vendors have public sign-up; `cloud-iac` requires tenant_class provisioning.
2. **Vendor-native VCS bindings** — Spacelift's GitHub/GitLab/Bitbucket UI for runs is more mature than Oyatie's; Oyatie's UI is via
   `workflow-studio` + `foundry` and assumes Foundry-pipeline discipline.
3. **Provider plugin freshness** — Terraform Cloud picks up provider releases within hours; `cloud-iac` re-vendors providers monthly
   on a security-checked cadence.

## Where `cloud-iac` wins

1. **Cedar in-process** — ABAC decisions in ≤ 200 µs vs Sentinel/OPA at 5-15 ms.
2. **Pack overlays** — regulatory packs (SOC2, GDPR, HIPAA, PCI, EU AI Act) flip per-tenant; vendors require you to author policies.
3. **Reviewer-agent path** — built-in multispectrum review; vendors require external CI / Run Tasks.
4. **Audit chain** — BLAKE3-256 chain proves no audit tampering; vendor logs are append-only but not chain-verifiable.
5. **No vendor lock-in on policy** — Cedar policies are portable; Sentinel + Pulumi CrossGuard are not.

## Reproducibility

```bash
make benchmarks.cloud-iac.run \
  VENDORS="cloud-iac,terraform-cloud,pulumi-cloud,spacelift,env0" \
  STACKS="small-vpc,medium-k8s,large-aurora,regulated-pci,gpu-pool" \
  TRIALS=3
```

Evidence: `.foundry/evidence/benchmarks/cloud-iac/2026-05-12T14:31:08Z/`.
