# Oyatie Hyperscaler Capability Gap Analysis

> Repo-grounded analysis (web egress was unavailable; big-4 bar is model-knowledge `[K]`, current state is repo-cited `[R]`). Produced 2026-06-01. Source: architect agent, persisted by main loop.

**Evidence legend:** `[R]` = repo-cited evidence; `[K]` = knowledge inference about the big-4 bar.

## Central finding (root cause, not symptom)

The repo has a **massive, disciplined planning + contract substrate** but almost **zero actuation**. `[R]` cloud/ = ~101.5K LOC Rust across 25 services; oya/ = ~425K LOC across ~90 services; 346 ADRs; ~150 machine-readable specs. But the cloud control plane is structurally **honest-deferred**: domain kernels define typed contracts, provider adapters *translate request shapes* without performing I/O. Verbatim `[R]` `cloud/cloud-compute/.../adapter-aws/src/lib.rs`: "It does not hold credentials, call AWS SDKs, or perform network I/O." `[R]` `cloud/managed-k8s-control-plane-host/.../src/lib.rs`: "The LIVE CRD reconciliation ... is **not** implemented ... every port method returns `ProvisioningError::Unimplemented`." The **only plane with real runtime I/O is cloud-intelligence** (the LLM gateway): real codex/anthropic adapters, REST forwarding, `tokio::main` server, OAuth SubscriptionPool `[R]` ADR-0384 D1-D5. So the gap is not "design" — it is **the reconciler/actuator/data-plane layer that turns contracts into provisioned resources**, plus the live CI/CD and multi-region operational evidence that gate any production claim.

## Capability-parity matrix (12 planes)

| # | Plane | Big-4 table-stakes `[K]` | Oyatie current (repo-cited) `[R]` | Gap |
|---|-------|--------------------------|-----------------------------------|-----|
| 1 | Control-plane / API | Declarative resource registry, long-running-op ledger, regional API, idempotent CRUD, conformance | `specs/cloud-control-plane-canonical.json` (status **Proposed-target**), ORN model, LRO + facets defined; no live op execution | P0 |
| 2 | Compute (VM/fn) | EC2/Lambda: real hypervisor + scheduler + fleet lifecycle | `cloud-compute` 12.2K LOC domain+vm/k8s/fn APIs + aws/oci shape-adapters; **no SDK calls, no I/O** | P0 |
| 3 | Storage (obj/blk/file) | S3/EBS/EFS durable data plane, lifecycle, replication | `cloud-storage` 7.3K LOC contracts; SeaweedFS transitory primary (ADR-0196), bespoke-Rust S3 named but unbuilt | P0 |
| 4 | Networking/DNS/edge | VPC/Route53/LB/CDN/PrivateLink real dataplane | `cloud-network` 11.7K LOC vpc/lb/dns contracts; **no envoy/cilium/nftables/BGP wiring** | P0 |
| 5 | Identity/access | IAM principals, policy sim, workload identity, STS | `cloud-iam` 7K LOC + Cedar (ADR-0243 universal gate) + `oya/identity` 16.6K + ADR-0394 bespoke IdP; strongest plane | P1 |
| 6 | Observability | CloudWatch/Monitoring: metrics+logs+traces+SLO burn-rate | `oya/observability` 6.7K LOC; OTel contract (ADR-0263/0042); Loki/Tempo/Mimir/Grafana (ADR-0383); no measured burn-rate evidence | P1 |
| 7 | Billing/FinOps | Metering→rated invoice, budgets, quotas, cost alloc | `cloud-finops` 4.9K + ADR-0478/0479/0480 bespoke billing/meter/cost; meter→invoice reconciliation unbuilt | P1 |
| 8 | Security/compliance/KMS | KMS+HSM envelope, Security Hub/GuardDuty posture, WAF | `cloud-kms` 4.8K contracts; OpenBao (ADR-0043); ADR-0506 aws-lc-rs; no HSM root-of-trust runtime, no posture engine | P0 |
| 9 | Data/AI platform (+LLM-gw) | RDS/Spanner/BigQuery managed DBs; AI inference | `cloud-data` 12.6K contracts (no engine); **LLM-gateway is REAL** (cloud-intelligence + ADR-0384 subscription-pooling) | P0 (DB) / P2 (LLM-gw) |
| 10 | Developer experience | SDKs, CLI, IaC, console, portal | `oya/developer-sdk`, ADR-0167 CLI, ADR-0170 portal, ADR-0339 IaC modules; Buck2 (ADR-0392/0512); SDK coverage thin | P2 |
| 11 | Reliability/multi-region | Cells, active-active, RTO/RPO, chaos, failover drills | `cloud-cell`/`cell-lifecycle`/`cell-rebalancer` 5.2K (ADR-0009/0248/0351); ADR-0158 active-active, ADR-0165 chaos; **no drill evidence** | P0 |
| 12 | Marketplace/ecosystem | Curated catalog, provenance, license, billing attach | `cloud-marketplace` 3.1K contracts; ADR-0249/0314; provenance/install runtime unbuilt | P3 |

## Top ~15 highest-leverage gaps (prioritized + sequenced)

**Foundation (unblock everything):**
1. **[P0] Reconciler/actuation runtime** — close the `Unimplemented` seam. Turn the held kube-rs/CABPT clients in `managed-k8s-control-plane-host` into live CRD reconciliation. Single highest-leverage gap; every cloud-* plane inherits it.
2. **[P0] Live self-hosted CI/CD go-live** — retire relax-merge; get oya-ci-gate green e2e on Talos Jenkins/Argo farm. Per memory: "CI-go-live is the unlock." Without it, no plane gets production evidence.
3. **[P0] Long-running-operation ledger + control-plane API server** — the LRO ledger in `cloud-control-plane-canonical.json` must become a running async service (model exists; server does not, except cloud-intelligence).
4. **[P0] KMS/HSM root-of-trust runtime** — envelope encryption + rotation/destroy lifecycle as live service (currently contract-only); gates storage, data, secrets.

**Platform spine (one real provisionable resource per plane):**
5. **[P0] Compute actuation** — wire `cloud-compute` to a real backend (Talos/CAPI for K8s, vfkit/libvirt for VM) so one VmInstance + one K8sCluster actually provision.
6. **[P0] Object-storage data plane** — promote SeaweedFS adapter to live, or start bespoke-Rust S3 (ADR-0196 + object-store roadmap memory).
7. **[P0] Networking dataplane** — bind VPC/subnet/SG/LB contracts to cilium/envoy actuation (ADR-0148/0253).
8. **[P0] Managed-DB actuation** — `cloud-data` to a real Postgres/ClickHouse provisioner (ADR-0045/0193).
9. **[P1] Observability burn-rate evidence** — multi-window SLO burn-rate measured against running services (ADR-0180/0210/0383); required to lift `public_sla_or_slo` nonclaim.
10. **[P1] Billing meter→invoice reconciliation** — make `oya-meter`→`oya-billing` produce rated line items with reconciliation tests (ADR-0478/0479).

**Operational hardening:**
11. **[P0] Multi-region failover + chaos drill evidence** — execute cell-rebalancer/DR drills and record evidence (ADR-0158/0165/0343); lifts `production_ready` nonclaim.
12. **[P1] IAM policy-simulation + tenant-isolation negative tests** at runtime (Cedar preview strong on contract; needs live deny/allow harness).
13. **[P1] Conformance harness** — Kubernetes conformance run + declarative-API contract tests (currently explicitly non-claimed).

**Product vertical (dogfood):**
14. **[P2] LLM-gateway hardening to GA** — cloud-intelligence is real but Proposed; finish ADR-0384 v1 (Anthropic+Codex), add quota/audit, then dogfood. Highest-maturity asset; closest to shippable.
15. **[P2] Developer SDK + console depth** — thin today; needed for any external consumption (ADR-0170/0167).

## ADRs needing amendment + where a NEW ADR is needed

**Amend (gaps in existing decisions):**
- **ADR-0028 / 0128 / 0509 / 0512** (cloud architecture + decomposition + monorepo) — add an explicit **actuation/reconciler layer contract**: every cloud-* domain MUST have a named adapter performing live I/O behind the `Unimplemented` seam, with a promotion gate.
- **ADR-0123 / 0135** (hyperscaler-maturity-claim-gate / aspirational-enforcement) — add **per-plane "actuation evidence required"** rows distinct from "contract present," so contract-completeness can't be mistaken for capability.
- **ADR-0196** (object storage) — add bespoke-Rust S3 trigger criteria + SeaweedFS-live promotion gate.
- **ADR-0376 / 0375** (managed-K8s / Talos-CAPI-Argo) — own the live Kamaji CRD reconciliation follow-on the code defers.
- **ADR-0349/0359/0361/0408/0511/0513/0514** (CI/CD substrate) — reconcile 7+ overlapping CI ADRs into ONE canonical CI ADR; the spread is itself a risk.
- **ADR-0384** (LLM-gateway subscription pool, **Proposed**) — promote to Accepted + add quota/billing/audit-tap deliverables to reach GA.
- **ADR-0158 / 0343 / 0165** (multi-region/DR/chaos) — add **drill-evidence exit criteria** (measured, not declared).

**New ADRs needed:**
- **NEW: Control-plane actuation & reconciler runtime** — the missing layer-of-record (operator framework choice, LRO execution engine, idempotency/retry). Keystone absence.
- **NEW: Bespoke storage/network dataplane decision** — which bespoke-Rust dataplane(s) get built vs which OSS adapters stay (per hyperscaler-lens filter).
- **NEW: Production-evidence promotion gate** — single normative gate binding production-quality-kits + SLO burn-rate + drill evidence to the `production_ready`/`public_sla_or_slo` nonclaim lift.

## New platform spec — skeleton outline

```
1. Purpose & non-claims (inherit cloud-hyperscaler-parity-taxonomy nonclaims)
2. Layered model: contract kernel -> reconciler runtime -> actuation adapter -> dataplane
   DECISION: operator framework (bespoke-Rust vs kube-rs controller-runtime analog)
3. Control-plane API surface (resource registry + LRO ledger + regional routing)
   DECISION: API protocol (gRPC vs REST), versioning (ADR-0342 hybrid)
4. Per-plane actuation contract (12 planes x {backend, idempotency, quota, audit, SLO})
   DECISION: per-plane backend (compute=CAPI/vfkit; storage=SeaweedFS->bespoke S3; net=cilium; db=pg/clickhouse)
5. Identity/policy integration (Cedar PDP + workload identity + STS-equivalent)
6. Observability & SLO evidence contract (burn-rate, drill records)
7. Billing/metering integration (meter->rated->invoice)
8. Multi-region/cell topology + failover + DR evidence
9. Security/KMS/HSM root-of-trust runtime
10. CI/CD + promotion gates (single canonical lane) + production-evidence gate
11. Marketplace/ecosystem (deferred P3)
12. Dogfood tenancy boundary (cloud/ platform vs oya/ tenant; zero internal bypass)
```

## Sequenced delivery plan — milestone skeleton (aligned to post-foundation roadmap A/B/C/D)

```
PHASE A — Foundation (exit: CI green e2e, relax-merge retired, reconciler runtime lands)
  M-A1 Live self-hosted CI/CD go-live (gap #2)
  M-A2 Reconciler/LRO runtime + control-plane API server (gaps #1,#3)
  M-A3 KMS/HSM root-of-trust runtime (gap #4)
  Exit: one Unimplemented seam closed end-to-end with drill evidence

PHASE B — Platform spine (exit: ONE real provisionable resource per P0 plane)
  M-B1 Compute actuation (#5)   M-B2 Storage dataplane (#6)
  M-B3 Networking dataplane (#7) M-B4 Managed-DB actuation (#8)
  M-B5 Observability burn-rate + multi-region drill evidence (#9,#11)
  Exit: lift production_ready nonclaim for the spine planes

PHASE C — Product vertical dogfood (exit: oya/ runs on cloud/ as tenant)
  M-C1 LLM-gateway GA (#14)  M-C2 IAM runtime harness (#12)
  M-C3 Billing reconciliation (#10) M-C4 conformance harness (#13)
  Exit: a real oya/ microservice provisioned + billed + observed via cloud/

PHASE D — Bespoke + ecosystem (exit: bespoke dataplane + marketplace runtime)
  M-D1 bespoke-Rust S3/network  M-D2 marketplace provenance+install (#15)
  M-D3 SDK/console depth
```

## Consensus addendum (steelman against the prioritization)

- **Antithesis:** Prioritizing actuation (#1) over more contracts could be wrong if the **business goal is dogfood of `oya/` products, not selling IaaS**. If oya/ services only need K8s + Postgres + object store + LLM-gateway (all near-real already), VM/networking/marketplace actuation is premature; the leverage move is **finish LLM-gateway + one DB + observability and ship a product vertical**. cloud-intelligence being the only real plane supports this read — the team's *revealed* priority is the LLM gateway, not EC2-parity.
- **Tradeoff tension:** Bespoke-Rust-first doctrine vs time-to-actuation. Every "build bespoke dataplane" decision (S3, network, DB) **directly delays** lifting production nonclaims. The hyperscaler-lens filter permits self-hostable OSS adapters as transitory — the plan must explicitly choose adapter-now vs bespoke-later per plane or it stalls on green forever.
- **Synthesis:** Sequence by **dogfood need**, not big-4 catalog completeness. Phase B should provision only the planes an actual `oya/` vertical consumes (K8s, storage, DB, observability, LLM-gw, IAM); treat VM/networking-dataplane/marketplace as contract-frozen until a tenant needs them.
- **Principle flags:** (1) **Honest-claims** is well-respected in code (explicit `Unimplemented`, nonclaims) — do not let a "parity matrix" doc become an implicit maturity claim. (2) **CI-ADR sprawl** (0349/0359/0361/0408/0511/0513/0514) violates single-canonical-source doctrine; consolidate before more CI work.

**Key references:** `cloud/cloud-compute/crates/oya-cloud-compute-adapter-aws/src/lib.rs:4-7`; `cloud/managed-k8s-control-plane-host/crates/*/src/lib.rs:12-26`; `cloud/cloud-intelligence/` (only real-I/O plane); `specs/cloud-hyperscaler-parity-taxonomy.json`; `specs/cloud-control-plane-canonical.json`; `docs/decisions/ADR-0384-llm-gateway-oauth-subscription-pool-redesign.md`; ADR-0123/0128/0509/0512; ADR-0349/0359/0361/0408/0511/0513/0514.
