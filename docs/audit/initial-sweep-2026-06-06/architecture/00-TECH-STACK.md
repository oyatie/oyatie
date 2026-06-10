# 00 — Tech Stack: NOW | PLANNED | FINAL-IDEAL (per layer)

> **READ-ONLY synthesis.** NOW cells are cited from real deps
> (`/Users/jasonlee/Developer/source`: root `Cargo.toml` 723-member workspace,
> `Cargo.lock`, `third-party/BUCK`, `infra/` manifests) — **NOT from ADRs**.
> PLANNED / FINAL-IDEAL cells cite the decision-record ruling that governs them
> (`synthesis/decision-record-oyatie-canon.md` D-codes, founder-locked ADRs).
> Aggregated from `10-techstack-now.md` (NOW evidence) and `10-techstack-roadmap.md`
> (PLANNED/IDEAL rulings).
>
> **Doctrine driving every row (the universal ratchet — D-META / D-CLOUD-NATIVE):**
> own-the-endpoint / vendor-the-bridge / **build-the-owned-alongside → run parallel-shadow
> → cutover only when the evidence-gate fires → THEN retire the bridge.** Never retire a
> bridge before its owned replacement is built and proven (ADR-0510 trigger + ADR-0123
> maturity-claim-gate). NOW = vendored bridge live · PLANNED = confirmed owned piece being
> built alongside · FINAL-IDEAL = bespoke owned endpoint, dogfooded (oya = tenant
> `oyatie-internal`).
>
> **Status legend (NOW):** **OWNED-IMPL** = real client lib wired (in Cargo.lock) ·
> **VENDORED BRIDGE** = third-party crate at an adapter seam · **TRAIT-STUB** = port
> declared, no client dep (the dominant pattern, the single most important NOW finding).

---

## The per-layer table

| Layer | NOW (cited deps / infra) | PLANNED — confirmed next (built alongside) | FINAL-IDEAL — owned bespoke endpoint | Ruling |
|---|---|---|---|---|
| **Language / build** | Rust **edition 2024**, `rust-version 1.95.0`, single virtual workspace **723 members** `resolver=2` (`Cargo.toml:730-732`); **cargo** primary; **buck2** secondary affected-scope gate (`.buckconfig`, `third-party/BUCK` 566KB, `infra/ci/buck2-affected-gate.sh`); tokio 1 (51 crates), axum 0.8 (12), hyper 1 **+ dual hyper 0.14.32**, reqwest 0.13 (9), tonic 0.14.6 + prost 0.14 (5) — OWNED-IMPL | Keep cargo+buck2; buck2 build-graph over a CAS core becomes the CI brain | Owned build-graph engine inside oya-ci/cd (Buck2 build-graph = the differentiator) | D3, D8 |
| **Forge / SCM** | **GitHub** (`jason931225/oyatie`) = SOLE interim forge; **Forgejo dropped** (0 refs on `phase0/producer`); infra `infra/forge/` lingers (slated removal) — VENDORED BRIDGE | Stand up `cloud-scm` service; design CI forge-pluggably to the owned port (GitHub adapter now), Sapling-inspired change-graph | **Bespoke Sapling-inspired SCM** (`cloud/cloud-scm`; Piper/Sapling monorepo change-graph) — owned + dogfooded | D2, D-FORGE-CLARIFY |
| **CI / CD** | **THREE overlapping live surfaces:** GitHub Actions = sole live merge authority (`.github/workflows/backbone-microservices-ci.yml`, but globs the STALE flat `crates/oya-*` layout) · Jenkins scaffold (root `Jenkinsfile`, ADR-0361 "replaces retired GHA" — CONTRADICTS the live GHA) · buck2 affected-scope · in-house Rust **oya-ci** product (`ci-controller`/`ci-tide`/`ci-webhook-gateway` real crates, posts `oya-ci-required` NON-BLOCKING) — mixed (LIVE + STALE + OWNED-IMPL shadow) | oya-ci built in parallel/shadow NOW; gate LOGIC = Rust crates run by GHA (live) AND oya-ci (shadow); governance gates promoted spec→live Rust | **oya-ci / oya-cd** — one Rust-native CI/CD = `Run` + Buck2 build-graph over CAS; four adopted faces (Prow merge-queue · Tekton typed-task/provenance · Argo DAG · Argo-CD/Rollouts GitOps); **day-0 crown-jewel** | D3, D-CICD, D-CICD-AUTHORITY |
| **Build engine** | **BuildKit** + **Buildah** (hardened ~7-cap Job-per-build) behind owned `BuildSpec` port; scratch/distroless default; SBOM(SPDX)+SLSA hard-required; Kaniko archived — VENDORED BRIDGE | Keep BuildKit/Buildah behind `BuildSpec`; L7 container build-engine = DEFER_VENDORED (not Wave-0) | Owned build engine behind `BuildSpec` | D3, D7 |
| **Datastores** | **WIRED (OWNED-IMPL):** Postgres via **sqlx 0.8.6** (6 crates; `oya-shared-postgres-command-adapter-sqlx`; + tenant-rbac postgres-RLS) · **ClickHouse 0.13** (1 seam, `oya-shared-olap-clickhouse-adapter`) · **Valkey/redis 0.27** (1 crate, eventsink, direct-pinned). **TRAIT-STUB (port, empty deps):** TimescaleDB · Milvus/vector (`vector-store-kernel`) · SeaweedFS/object-store + `cloud-storage-adapter-s3` (no aws-sdk) · Citus absent | Finish L-0001 scrub (Postgres+Citus = vendored OLTP **until** owned proves parity); design every data port to the owned-engine ideal | **Own the ENTIRE data tier** — one distributed multi-model engine (OLTP+vector+OLAP+FTS+streaming), ratcheted per-model when proven (**L8 dropped** — owned DB has no source, design-spec only; future build campaign) | D4 |
| **Object / block storage** | **S3 + OCI** vendored bridges — `cloud-storage-adapter-{s3,oci}` + `-object-api`/`-block-api`/`-domain`; SeaweedFS = infra k8s manifest only — VENDORED BRIDGE (s3 adapter is path-only TRAIT-STUB today) | Keep S3/OCI behind the storage port; ratchet to owned when proven | Owned object/block storage (part of owned data tier) | D4 |
| **Eventing / streaming** | **ALL named brokers = TRAIT-STUB** — Kafka/Pulsar/NATS/Redpanda/Valkey/Postgres event-bus adapters depend on parent crate only (0 `rdkafka`/`pulsar`/`async-nats` in tomls/lock/BUCK). **Real path = in-house transactional outbox** (`oya-shared-transactional-outbox-*` sqlx+HTTP, `outbox-broker-http-adapter`) — OWNED-IMPL. Roadmap names **Pulsar** as canonical bridge (2 manifests); Kafka off the critical consistency path | One streaming substrate (Pulsar) behind a port; design eventing port to the owned-engine ideal | Owned eventing/streaming engine (endpoint via the ratchet) | D-EVENT, D-D1-TOPOLOGY |
| **Policy / authz** | **Cedar = OWNED-IMPL (vendored engine)** — `cedar-policy 4.11` wired in 5 adapters (`managed-k8s-tenant-quota-adapter-cedar`, `cloud-intelligence-authz-cedar-adapter`, `identity-workload-authz-cedar-adapter`, `ci-webhook-gateway-authz-cedar-adapter`) + `oya/policy/oya-policy-cedar-{api,domain}`. Cedar the LANGUAGE = permanent external CONTRACT | Own the EVALUATION ENGINE behind the Cedar contract — compile-to-Rust **PARC** (linux 0021), differential-tested vs Lean-verified oracle + Kani (PARC ~unestablished today, real net-new build) | **PARC** owned engine behind the permanent Cedar contract; served as **PaaS — Policy-as-a-Service** (single central PDP) | D6, D-GOVERNANCE-CENTRAL |
| **Governance / compliance** | **CLI-based interim** — `oya gate`/`oya check`/`oya verify` + 22 `oya-governance-*` lanes + evidence JSON; `governance` service spec-stage (live Rust build-out in progress); Jenkins/`oya gate` = legacy bridge | Move ALL governance+evidence to a cloud-native PIPELINE — gates = Rust binaries run by GHA (live) + oya-ci (shadow); governance promoted spec→live crates (~73% landed) | **Central authority via PaC/CaC + PaaS/CaaS** — Policy-as-Code (Cedar) · Compliance-as-Code (keystone gates) · Policy-as-a-Service (PARC PDP) · Compliance-as-a-Service (evidence/attestation pipeline) | D16, D-GOVERNANCE-CENTRAL, D-FOUNDRY-CLARIFY |
| **Identity** | **Zitadel = vendored OIDC/SCIM bridge** (IP-003 oidc-issuer-adapter, IP-008 scim-adapter, IP-016 scale-validation) — but **NOT yet in deps** (0 Zitadel refs in Cargo.toml/lock; OIDC kernel is TRAIT-STUB, deps=serde-only). Real today: **webauthn-rs 0.5.5** (VENDORED BRIDGE via `oya-identity`) + in-house OIDC issuer crates — OWNED-IMPL/bridge | **oya-identity owned kernel built alongside** — `oya-identity-oidc-issuer-kernel`, `oya-identity-workload-{oidc-adapter,authz-cedar-adapter}`; ADR-0476 founder-locked; Zitadel demoted canonical→Phase-1 bridge | **oya-identity** (bespoke Rust OIDC/SCIM/WebAuthn issuer) = owned endpoint; Zitadel retired on cutover | D5 |
| **Crypto** | **aws-lc-rs 1 = single mandated backend** (7 crates); **`ring` FORBIDDEN** (in-file ADR-0506); rustls via aws-lc-rs; hyper-rustls 0.27; RustCrypto hmac/sha2/subtle for webhook HMAC — OWNED-IMPL, single-backend | FIPS backend deferred (in-file ADR-0506); keep aws-lc-rs | Owned/FIPS-validated crypto posture behind the same single-backend mandate | D-META, ADR-0506 |
| **KMS / secrets** | **OpenBao = vendored bridge** — `cloud-kms-adapter-openbao` deps=path-only (TRAIT-STUB); `cloud-intelligence-openbao-adapter` talks OpenBao via **reqwest REST** (no SDK); `cloud-secrets-file-adapter` only built adapter; OpenBao itself = infra (`infra/external-secrets/`) — mixed (1 stub, 1 REST bridge) | Build owned `cloud-kms-{domain,api}` behind the KMS port (port already present); OpenBao runs as the bridge | **Owned KMS/secrets** (cloud-kms/cloud-secrets) dogfood product; OpenBao retired on cutover | D-META (infra-sovereignty ratchet) |
| **Compute / IaaS** | **Hyperscaler bridges** — `cloud-compute-adapter-{aws,oci}` (rent AWS/OCI capacity now) — VENDORED BRIDGE | Build owned `cloud-compute` control plane behind the compute port; sequence per infra-sovereignty ratchet (IaC→gateway/KMS→mail/cache/stream→DB/storage) | **Owned bespoke-hyperscaler compute** (oyatie IaaS) self-hosted on the Rust k8s/OS/kernel stack | D-CLOUD-NATIVE, D-SEQ |
| **Kubernetes** (self-hosting) | **Rust reimplementation in flight** — `kube-rs 3.1` + `k8s-openapi 0.27` on Talos (VENDORED BRIDGE; CAPI/Kamaji reconcile honest-deferred → returns `Unimplemented`); linux pilot `stack/kubernetes` = **139 crates** (95 apimachinery `oya-cloud-k8s-*` + 44 `ctrd_*` containerd); `cloud/cloud-k8s` = docs-only SSOT (0 crates) | Collapse the 2 STD service trees to one-version root; 95 k8s crates under `managed-k8s-control-plane-host`; 44 ctrd → container-runtime CREATE | **Owned Rust Kubernetes engine** (managed-k8s control plane) — self-hosting, serves cloud + tenants | D-CLOUD-NATIVE, D-CONFORM |
| **OS / node** (self-hosting) | **Talos-class bridge** — linux pilot `stack/operating-system` = **43 `talos-*` crates** (Rust port of Talos); infra `infra/talos/` bakes Kata/cloud-hypervisor (ADR-0147); Cilium CNI, Kyverno admission, CAPI+Sidero — infra-only YAML + VENDORED BRIDGE | Owned node-OS ratchets up from the Talos port toward kernel-level ownership (ADR-0025 ladder); H2 bare-metal-host = COMMITTED owned endpoint | **Owned Rust node-OS** on the framekernel; assume-breach Capsule/microVM default | D7, D-CLOUD-NATIVE |
| **Kernel** (self-hosting) | **linux pilot framekernel** — `stack/kernel` = **7 no_std crates** on the kernel's pinned nightly + custom targets (12 no_std workspaces excluded from STD root); Phase-3 timer/paging/RTC/CPUID | Grow framekernel toward Linux-syscall-ABI parity (user mode → syscall → ELF → process) | **Owned framekernel** = committed bottom of the self-hosting stack; coupled to Linux via syscall ABI + differential testing | D7, [[cloud-native-rust-stack]] |
| **Isolation / runtime** | **Vendored bridges** — Talos + Kata + Firecracker + wasmtime; fleet default = assume-breach microVM (ADR-0023) — infra/VENDORED BRIDGE | Owned node-OS → kernel-level ratchet (ADR-0025) | **framekernel + owned-VMM**; per-session microVM isolation native to the AI substrate | D7 |
| **Observability** | **OTel→VictoriaMetrics→Grafana** (NOT LGTM) — OTel Collector 0.119 → prometheusremotewrite → **VictoriaMetrics v1.111** → **Grafana 11.4** (infra LIVE); Rust SDK: opentelemetry 0.32 + prometheus 0.13 + tracing 0.1.44 (OWNED-IMPL, 2 metrics adapters otlp+prometheus). **Mimir deliberately swapped for VictoriaMetrics; Loki/Tempo deferred** | Add traces/logs (Tempo/Loki/Vector) as follow-up; keep metrics+OTLP+dashboards core | Owned observability plane (Datadog/Grafana-class product `oya/observability`) behind the tracing-adapter port | (infra ruling; D-CLOUD-NATIVE) |
| **Frontend** | **Leptos 0.8.19** (SSR+hydrate) + wasm-bindgen 0.2.121 / web-sys 0.3.98, ONE "prototype" crate (`oya-application-shell-frontend-prototype`); Yew/Dioxus absent; enforced by `oya-check-client-stack-discipline` — OWNED-IMPL (prototype) | Build out the Leptos app-shell from prototype → product | Owned Rust/WASM frontend stack across the workspace suite | (client-stack discipline check) |
| **AI / LLM (cloud-intelligence)** | **Vendored model-provider bridges** in `oya/intelligence` (**128 crates**, ~96k-LOC engine): `intelligence-adapter-{anthropic,openai,gemini}-api-{adapter,kernel}` + `-subscription-*` (DUAL: metered API + seat subscription) using reqwest; `cloud/cloud-intelligence` (8 crates) = egress broker only (`codex-adapter`, `openbao-adapter`, eventsink clickhouse/valkey) — VENDORED BRIDGE | **Re-home the 96k engine DOWN** `oya/intelligence` → `cloud/cloud-intelligence` (cloud owns framework+runtime); converge live provider calls; dual-consumption; 23-dim maturity program | **cloud-intelligence = cloud-owned Bedrock-analog FRAMEWORK + RUNTIME**; GA-parity + BEAT on 4 owned differentiators (Automated-Reasoning guardrails · per-session microVM · durable execution · eval-as-proof). oya-intelligence = thin per-tenant servicing layer | D-INTEL, D-LLM-DOMAIN |
| **Progressive delivery / chaos** | **Argo Rollouts + Chaos Mesh** behind ports (Flagger superseded, ADR-0160) — infra/VENDORED BRIDGE | Ratchet to owned later; reconcile 0040/0165 | Owned progressive-delivery/chaos (folds into oya-cd's Argo-Rollouts face) | D10, D-CICD |

---

## How to read the seam (clean-arch evidence)

Every row's mobility point is a **`-adapter-*` crate in front of an owned `-kernel`/`-domain`,
exposed through an `-api`/`-rest` port**. Three live exemplars prove the ratchet is
*structural*, not aspirational:

- **CI:** `oya-ci-controller-kernel` (owned) ← `-github-adapter` + `ci-webhook-gateway-jenkins-adapter`
  (bridges). Swap adapters at cutover; kernel is forge/CI-engine-agnostic.
- **Identity:** `oya-identity-oidc-issuer-kernel` (owned) ← Zitadel adapter IPs (bridge).
- **AI:** owned engine domain ← `intelligence-adapter-{anthropic,openai,gemini}-api`/`-subscription`
  (vendored provider bridges). Re-home down, converge calls, retire mocks.

Where the owned kernel does **not** yet exist, the row is honest: **PARC** ~unestablished
(real net-new build behind the permanent Cedar contract); the **owned distributed DB** has
no source (L8 dropped, design-spec only); **`cloud-scm`** has no dir yet (GitHub-interim only).

## Sequencing (D8 / D-SEQ)

"Own everything, sequenced" ≠ "build everything at once." **oya-ci is the day-0 crown-jewel**
(its output unblocks building the rest), but builds parallelize in fractal lanes. The
infra-sovereignty ratchet is an ordered list + per-substrate M0 evidence-gate (contract +
incumbent benchmark + evidence before owning), **no calendar dates**. No OWN-when-proven
cutover without ADR-0510's trigger + ADR-0123's measured maturity claim.

---

## Coverage / caps (no silent caps)

- **NOW evidence is sound for "which libs are in use"** — drawn from Cargo.lock resolved
  set + `third-party/BUCK` + workspace deps table + ~25 individually-opened crate
  Cargo.tomls + targeted greps. It is **NOT** a full per-crate stub-vs-impl census of all
  ~700 crates. Not line-read: every `*-domain` (assumed std-only), the full ERP vertical,
  all `oya-check-*`/`oya-governance-*` crates, cilium/kyverno policy bodies, Jenkins groovy
  seeds.
- **The single most important NOW caveat:** most named backends (all event brokers, S3,
  vector, timeseries, OpenBao-KMS, OIDC, Zitadel) are **TRAIT-STUBS** — ports declared, no
  client wired. The genuinely-wired set is short: Postgres(sqlx) · ClickHouse · Valkey(redis)
  · Cedar · kube-rs · webauthn-rs · Leptos/WASM · reqwest · tonic · prometheus/OTel.
- **PLANNED/IDEAL** rulings cross-read from `decision-record-oyatie-canon.md`
  (D2–D16, D-META/D-EVENT/D-CICD/D-CLOUD-NATIVE/D-GOVERNANCE-CENTRAL/D-INTEL/D-LLM-DOMAIN/
  D-FORGE/D-FOUNDRY/D-SEQ/D-CONFORM/D-D1-TOPOLOGY). Per-crate manifests of the 128
  intelligence + 139 k8s crates were counted, not each read; some data-tier pins
  (Milvus/Timescale/Iceberg/SeaweedFS) taken from D4's verified ADR list rather than
  re-grepped per-manifest.
