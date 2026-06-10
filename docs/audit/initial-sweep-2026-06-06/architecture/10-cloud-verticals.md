# Cloud (Platform) Verticals — Architecture Inventory

READ-ONLY, source-backed. All facts below are derived from the REAL tree under
`/Users/jasonlee/Developer/source/cloud/` (directory listings, `crates/` contents,
`Cargo.toml` package names, and `manifest.json` `bounded_contexts`/`layers`), NOT from ADRs.

Scope: every directory under `/Users/jasonlee/Developer/source/cloud/` — **25 verticals** (confirmed by `ls`):
`cell-lifecycle cell-rebalancer cloud-billing cloud-billing-tax cloud-capacity cloud-cell cloud-compute cloud-data cloud-finops cloud-iac cloud-iam cloud-intelligence cloud-k8s cloud-kms cloud-marketplace cloud-network cloud-network-dns cloud-secrets cloud-storage cloud-tenancy managed-k8s-cluster-lifecycle managed-k8s-control-plane-host managed-k8s-sla-observability managed-k8s-tenant-quota tenancy`.

All 25 are **platform (`cloud/`) tree** — i.e. the hyperscaler substrate, not `oya/` product verticals.
(Crate package prefixes are `oya-` regardless, because `oya` is the org/workspace prefix; the *placement* under
`source/cloud/` is what marks them as platform.)

## Clean-arch lens used

Crate role suffixes = layers (inner→outer):
`kernel` (pure invariants) · `domain` (entities/value objects) · `usecase` (application services) ·
`app` (composition root) · `api` (typed contract surface / in-proc port) · `rest` (HTTP adapter) ·
`sdk` (client) · `worker` (async runner). `-adapter-<x>` + `-api`/`-rest` = the **PORTS/ADAPTERS** = the
mobility seams (swap impl without touching domain). One crate-set per bounded-context = **cohesion**;
multiple independently-scalable BCs in one service dir = **detachment-for-scale**.

## NO SILENT CAPS — coverage statement

- **22 of 25** verticals have a real `crates/` dir with Rust `Cargo.toml` packages — all fully opened (every crate listed below was enumerated via `ls crates/` and member `Cargo.toml`).
- **3 of 25 are spec-only / no `crates/` dir** (opened fully, confirmed no Rust crates):
  - `cell-lifecycle` — only design artifacts: `ARCH.md`, `contracts/{openapi.yaml,asyncapi,proto}` (e.g. `cell-lifecycle-design-events.yaml`, `cell-lifecycle-design.proto`), `capabilities`, `slos`, `threat-models`. `manifest.json` status = `implemented-local-foundation-truth-down` but no code crates yet.
  - `cell-rebalancer` — identical spec-only shape (no `crates/`, no `Cargo.toml` anywhere); design contracts + runbooks only.
  - `cloud-k8s` — **no `crates/` dir**; its `manifest.json` owner = `axis-cloud-compute` and bounded_context = `cloud-compute`, pointing the *implemented* crates back to the `cloud-compute/` service (the `cloud-k8s/` dir is kept "for compatibility" and holds OpenAPI/AsyncAPI/proto scaffolds + a large IP-* planning set). The cloud-k8s *target* crate set is defined only in `IP-*.md` (planned, not yet built): e.g. `oya-cloud-k8s-cluster-bootstrap-{kernel,domain,usecase,adapter-kubeadm}`, `oya-cloud-k8s-node-lifecycle-{kernel,domain,usecase,adapter}`, `oya-cloud-k8s-network-policy-{kernel,domain,usecase,adapter}`, `oya-cloud-k8s-service-mesh-control-plane-{kernel,usecase,adapter-istio}`, `oya-cloud-k8s-ingress-controller-{kernel,usecase,adapter-envoy}`, `oya-cloud-k8s-csi-storage-driver-{kernel,usecase,adapter-block,adapter-file,adapter-object}`, `oya-cloud-k8s-kubernetes-api-proxy-{kernel,usecase,adapter,rest,sdk,worker,app}`. These are PLANNED crate names only.

Everything else below is verified-present code.

---

## Grouped by hyperscaler substrate class

### A. Identity / Key / Secrets substrate (IAM · KMS · Secrets)

**`cloud-iam`** — `crates/`:
`oya-cloud-iam-domain`, `oya-cloud-iam-app`, `oya-cloud-iam-api`, `oya-cloud-iam-adapter-oci`, `oya-cloud-iam-adapter-selfhosted`.
- Layers: domain → app (composition) → api (port).
- **PORTS/ADAPTERS:** `adapter-oci` (managed/OCI IAM backend) vs `adapter-selfhosted` (self-run) — a clean 2-impl mobility seam (managed-cloud vs on-prem).
- Cohesion: single BC (`cloud-iam`). Role: hyperscaler **IAM control plane** (principals/policies/authz).

**`cloud-kms`** — `crates/`:
`oya-cloud-kms-domain`, `oya-cloud-kms-api`, `oya-cloud-kms-adapter-oci`, `oya-cloud-kms-adapter-openbao`.
- **PORTS/ADAPTERS:** `adapter-oci` (managed KMS) vs `adapter-openbao` (OpenBao/Vault self-host) — key-management mobility seam.
- Single BC. Role: **Key Management Service** (envelope keys, CMK, BYOK).

**`cloud-secrets`** — `crates/`:
`oya-secrets-domain`, `oya-secrets-file-adapter`.
- **PORTS/ADAPTERS:** `secrets-file-adapter` is the only *built* adapter today; the dir carries a large IP-* roadmap (`IP-006-resolver-adapter-openbao`, `IP-011-hsm-integration-adapter-hsm`, `IP-009-openbao-operator`) so OpenBao + HSM adapters are PLANNED (note: not yet crates). Substrate phase doc = `PHASE-01-OPENBAO-SECRETREFERENCE-SUBSTRATE.md`.
- Single BC. Role: **secret resolution** (SecretReference URI → value), rotation scheduler, per-tenant namespace.

### B. Compute / K8s / Cell substrate (compute · k8s · cell topology)

**`cloud-compute`** — `crates/`:
`oya-cloud-compute-domain`, `oya-cloud-dcops-domain`, `oya-cloud-resource-domain`,
`oya-cloud-compute-vm-api`, `oya-cloud-compute-k8s-api`, `oya-cloud-compute-functions-api`,
`oya-cloud-compute-adapter-aws`, `oya-cloud-compute-adapter-oci`.
- **DETACHMENT:** three product axes exposed as separate API surfaces from a shared domain — `vm-api` (IaaS VMs), `k8s-api` (managed Kubernetes), `functions-api` (FaaS). Plus side domains `dcops` (datacenter ops) and `resource` (resource model).
- **PORTS/ADAPTERS:** `adapter-aws` vs `adapter-oci` — provider mobility (the EC2/EKS/Lambda vs OCI Compute/OKE/Functions seam). This is the **real home of cloud-k8s's implemented code** (per `cloud-k8s/manifest.json` owner `axis-cloud-compute`).
- Role: the **EC2/EKS/Lambda-equivalent compute plane** (provider-neutral VM + managed-k8s + functions).

**`cloud-cell`** — `crates/`:
`oya-cell-domain`, `oya-cloud-cell-app`, `oya-cloud-region-api`, `oya-cloud-region-domain`,
`oya-regional-pack-api`, `oya-regional-pack-domain`.
- **DETACHMENT (multi-BC):** three bounded contexts in one dir — **cell** (`oya-cell-domain` + `oya-cloud-cell-app` composition root), **region** (`oya-cloud-region-domain` + `oya-cloud-region-api`), **regional-pack** (`oya-regional-pack-domain` + `oya-regional-pack-api`). Each BC carries its own api/domain pair → independently scalable.
- Role: **cell-based architecture topology** (cells, regions, regional regulatory packs) — the hyperscaler "cellular isolation" substrate.

**`cell-lifecycle`** *(spec-only, no crates)* — design contracts for cell create/drain/retire lifecycle. Role: cell **lifecycle controller** (planned).

**`cell-rebalancer`** *(spec-only, no crates)* — design contracts for cross-cell tenant rebalancing. Role: cell **placement/rebalance controller** (planned).

**`cloud-capacity`** — `crates/`: `oya-cloud-capacity-domain`, `oya-cloud-capacity-kernel`.
- Layers: kernel (capacity invariants) + domain. No adapters yet (inner-only). Single BC.
- Role: **capacity model / headroom** substrate (feeds scheduling & finops).

#### Managed-K8s (the "managed Kubernetes service" cluster of 4 micro-verticals)

These four are textbook clean-arch microslices (each `kernel`/`domain?`/`usecase?`/`app`/`api` + an `adapter-inmemory` test double):

**`managed-k8s-cluster-lifecycle`** — `crates/`:
`oya-managed-k8s-cluster-lifecycle-kernel`, `-api`, `-app`.
- Single BC. Role: managed-cluster create/upgrade/delete lifecycle. (No external adapter yet — inner + api + app only.)

**`managed-k8s-control-plane-host`** — `crates/`:
`oya-managed-k8s-control-plane-host-kernel`, `-api`, `-app`,
`-adapter-capi` (Cluster API), `-adapter-inmemory`.
- **PORTS/ADAPTERS:** `adapter-capi` (real Cluster-API control-plane host) vs `adapter-inmemory` (test/sim) — control-plane-hosting mobility seam.
- Role: **hosted control-plane** provisioning (the managed-master plane).

**`managed-k8s-sla-observability`** — `crates/`:
`oya-managed-k8s-sla-observability-kernel`, `-api`, `-app`, `-adapter-inmemory`.
- **PORTS/ADAPTERS:** `adapter-inmemory` only built (real telemetry sink adapter implied/planned).
- Role: **SLA/observability** for managed clusters.

**`managed-k8s-tenant-quota`** — `crates/`:
`oya-managed-k8s-tenant-quota-kernel`, `-api`, `-app`,
`-adapter-cedar` (Cedar policy authz), `-adapter-inmemory`.
- **PORTS/ADAPTERS:** `adapter-cedar` (policy-engine-backed quota enforcement) vs `adapter-inmemory` — quota-enforcement mobility seam.
- Role: **per-tenant quota** on managed clusters.

> The 4 managed-k8s dirs are **detachment-by-microservice**: one logical "managed Kubernetes" product split into 4 independently-deployable bounded contexts (lifecycle / control-plane-host / sla-observability / tenant-quota), each with its own crate stack — a deliberate scale/ownership seam.

### C. Storage / Data substrate

**`cloud-storage`** — `crates/`:
`oya-cloud-storage-domain`, `oya-cloud-storage-object-api`, `oya-cloud-storage-block-api`,
`oya-cloud-storage-adapter-s3`, `oya-cloud-storage-adapter-oci`.
- **DETACHMENT:** two product surfaces — `object-api` (S3-style object store) + `block-api` (EBS-style block) over a shared domain.
- **PORTS/ADAPTERS:** `adapter-s3` vs `adapter-oci` — storage-backend mobility (AWS S3 vs OCI Object Storage).
- Role: **object + block storage** plane.

**`cloud-data`** — `crates/`: `oya-cloud-data-domain`, `oya-cloud-data-kernel`.
- Inner-only (kernel + domain), single BC, no adapters yet. Role: **managed data services** substrate (DB/data-plane invariants).

### D. Network / DNS substrate

**`cloud-network`** — `crates/`:
`oya-cloud-network-domain`, `oya-cloud-network-vpc-api`, `oya-cloud-network-lb-api`,
`oya-cloud-network-adapter-oci`, `oya-cloud-network-adapter-selfhosted`, `oya-residency-domain`.
- **DETACHMENT:** two surfaces — `vpc-api` (virtual network) + `lb-api` (load balancer) over shared domain; plus a co-located **`oya-residency-domain`** BC (data-residency rules tied to network topology).
- **PORTS/ADAPTERS:** `adapter-oci` vs `adapter-selfhosted` — network-fabric mobility (managed vs on-prem).
- Role: **VPC + load-balancing** plane.

**`cloud-network-dns`** — `crates/`: `oya-cloud-network-dns-api`.
- Single `-api` crate only (thin/early). Single BC. Role: **managed DNS** (Route53-equivalent), split out from cloud-network as its own scalable vertical.

### E. Billing / FinOps / Marketplace / Capacity (commerce substrate)

**`cloud-billing`** — `crates/`:
`oya-cloud-billing-kernel`, `oya-cloud-billing-domain`, `oya-billing`,
`oya-meter`, `oya-metering-domain`, `oya-saas-bench-app`.
- **DETACHMENT (multi-BC):** **billing** BC (`-kernel`/`-domain` + legacy `oya-billing`) + **metering** BC (`oya-meter` + `oya-metering-domain`) + a bench/app (`oya-saas-bench-app`). Metering (usage capture) is intentionally separable from billing (invoicing) for independent scale.
- Role: **usage metering + billing/invoicing** plane.

**`cloud-billing-tax`** — `crates/`: `oya-cloud-billing-tax-app`.
- Single `-app` crate (composition root; likely wraps shared tax domain or is early). Single BC. Role: **tax calculation** vertical, detached from core billing.

**`cloud-finops`** — `crates/`:
`oya-cloud-finops-kernel`, `oya-cloud-finops-domain`, `oya-cloud-finops-api`, `oya-cost`.
- Layers: kernel → domain → api; `oya-cost` = shared cost-model BC. Single primary BC + cost sidecar.
- Role: **FinOps / cost management** (budgets, cost allocation, showback/chargeback).

**`cloud-marketplace`** — `crates/`:
`oya-cloud-marketplace-kernel`, `oya-cloud-marketplace-domain`, `oya-saas-plugin-marketplace-kernel`.
- **DETACHMENT:** two kernels — cloud **marketplace** (`-kernel`/`-domain`) + **saas-plugin-marketplace** (`oya-saas-plugin-marketplace-kernel`), i.e. cloud-listing marketplace vs in-product plugin marketplace as separable BCs.
- Role: **marketplace / catalog** plane.

### F. IaC / Tenancy substrate (declarative infra + multi-tenancy)

**`cloud-iac`** — `crates/`:
`oya-cloud-iac-domain`, `oya-cloud-iac-app`, `oya-cloud-iac-api`, `oya-cloud-iac-rest`, `oya-cloud-iac-runtime`.
- **Full clean-arch column:** domain → app → api → **rest** (HTTP adapter, present) → `runtime` (execution lib, `src/lib.rs`). `manifest.json` BC also references `oya-check-iac-tier-discipline` (governance check). Substrate doc: `PHASE-01-META-IAC-PIPELINE-SUBSTRATE.md`; heavy GitOps IP-* set (ArgoCD/Flux/OpenTofu, `IP-005-iac-renderer-adapter-trio` → renderer adapter trio planned).
- **PORTS/ADAPTERS:** `-rest` is the present adapter; renderer adapter-trio (argocd/flux + opentofu) is PLANNED per IP-005.
- Single BC. Role: **declarative infrastructure / GitOps renderer + applier** (Terraform/OpenTofu + ArgoCD substrate).

**`cloud-tenancy`** — `crates/`: `oya-tenant-cli`.
- Single CLI crate (operator tooling). Single BC. Role: **tenancy CLI** front-end (thin client to the `tenancy` service below).

**`tenancy`** — `crates/` (14 crates — the largest, most-detached vertical):
`oya-tenancy-api`, `oya-tenancy-domain`, `oya-tenancy-kernel`,
`oya-tenancy-tenant-lifecycle-kernel`, `oya-tenancy-isolation-policy-kernel`,
`oya-tenancy-cell-assignment-kernel`, `oya-tenancy-dsr-cascade-kernel`,
`oya-tenancy-lifecycle-locks-kernel`, `oya-tenancy-sub-scope-registry-kernel`,
`oya-tenancy-kyb-kyc-verifier-domain`, `oya-tenancy-dr-pairing-usecase`,
`oya-tenancy-per-tenant-quota-usecase`, `oya-tenancy-reserved-namespace-usecase`,
`oya-tenancy-data-residency-enforcer-adapter`.
- **DETACHMENT (textbook, multi-BC by feature-kernel):** a thin core (`-api`/`-domain`/`-kernel`) plus a *fan* of independently-evolving feature BCs each at its own layer:
  - kernels: tenant-lifecycle, isolation-policy, cell-assignment, dsr-cascade (data-subject-request), lifecycle-locks, sub-scope-registry;
  - domains: kyb-kyc-verifier;
  - usecases: dr-pairing, per-tenant-quota, reserved-namespace.
- **PORTS/ADAPTERS:** `oya-tenancy-data-residency-enforcer-adapter` = the residency enforcement seam (swap enforcement backend). IP-* roadmap also lists `IP-005-tenant-lifecycle-adapter-postgres`, `IP-023-sub-scope-registry-adapter-postgres`, `IP-020-data-residency-enforcer-adapter` (Postgres-backed adapters PLANNED).
- Role: **multi-tenancy control plane** (tenant lifecycle, isolation/RLS, cell assignment, DSR cascade, KYB/KYC, DR pairing, quota, residency) — the platform's core tenancy substrate. This is the canonical **detachment-for-scale** exemplar in cloud/.

### G. Intelligence substrate

**`cloud-intelligence`** — `crates/`:
`oya-cloud-intelligence-kernel`, `oya-cloud-intelligence-app`, `oya-cloud-intelligence-rest`,
`oya-cloud-intelligence-authz-cedar-adapter`, `oya-cloud-intelligence-codex-adapter`,
`oya-cloud-intelligence-eventsink-clickhouse-adapter`, `oya-cloud-intelligence-eventsink-valkey-adapter`,
`oya-cloud-intelligence-openbao-adapter`.
- **Richest adapter set in cloud/ — full hexagonal fan-out:** kernel → app → rest, with FIVE distinct ports each as a named adapter:
  - **authz** → `authz-cedar-adapter` (Cedar policy);
  - **AI/codegen** → `codex-adapter`;
  - **event sink** → `eventsink-clickhouse-adapter` **and** `eventsink-valkey-adapter` (two interchangeable sink backends = explicit mobility, ClickHouse vs Valkey);
  - **secrets** → `openbao-adapter` (OpenBao).
- Single primary BC, but the multi-adapter eventsink pair (clickhouse | valkey) is the cloud-side analogue of an event-bus-style swappable-backend seam.
- Role: **platform intelligence / analytics + AI assist** plane (telemetry ingest + authz-gated insight, secrets-aware).

---

## Cross-cutting findings

- **Mobility seams (ports/adapters) actually built today** (the architecture-mobility evidence):
  - Provider-pair seams `oci ↔ selfhosted`: `cloud-iam`, `cloud-network`.
  - Provider-pair seams `aws ↔ oci`: `cloud-compute`.
  - Provider-pair seams `s3 ↔ oci`: `cloud-storage`.
  - KMS seam `oci ↔ openbao`: `cloud-kms`.
  - Managed-k8s test/real seams: `adapter-capi ↔ adapter-inmemory` (control-plane-host), `adapter-cedar ↔ adapter-inmemory` (tenant-quota), `adapter-inmemory` (sla-observability).
  - Policy seam `adapter-cedar`: managed-k8s-tenant-quota + cloud-intelligence-authz-cedar.
  - **Multi-backend swappable sink** (closest to the event-bus "kafka/nats/pulsar/redpanda/valkey" pattern): `cloud-intelligence` eventsink `clickhouse ↔ valkey`.
  - Residency enforcement seam: `tenancy/oya-tenancy-data-residency-enforcer-adapter`.

- **Detachment-for-scale cases (multi-BC, independently scalable):**
  `tenancy` (10+ feature kernels/domains/usecases) · `cloud-cell` (cell + region + regional-pack) · `cloud-compute` (vm + k8s + functions APIs) · `cloud-storage` (object + block) · `cloud-network` (vpc + lb + residency) · `cloud-billing` (billing + metering) · `cloud-marketplace` (cloud + saas-plugin marketplaces) · and the **managed-k8s family** (4 separate service dirs = one product split for scale).

- **Cohesion (single-BC) cases:** `cloud-iam`, `cloud-kms`, `cloud-secrets`, `cloud-data`, `cloud-capacity`, `cloud-finops` (core), `cloud-network-dns`, `cloud-iac`, `cloud-billing-tax`, `cloud-tenancy`, `cloud-intelligence` (single BC, many adapters).

- **Layer-completeness spread (maturity signal):** only `cloud-iac` ships a present `-rest` surface; `tenancy` ships the deepest kernel/domain/usecase fan; the **managed-k8s quartet** are the cleanest kernel→api→app→adapter slices; several verticals are **inner-only** (`cloud-data`, `cloud-capacity` = kernel+domain, no adapter/api yet) — early-stage. Three verticals (`cell-lifecycle`, `cell-rebalancer`, `cloud-k8s`) are **spec-only, zero crates** (design contracts + IP plans).
