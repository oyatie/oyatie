# Tech Stack NOW — source-backed (initial sweep 2026-06-06)

READ-ONLY map of what the code/deps in `/Users/jasonlee/Developer/source` actually
use **today**, cited from real `Cargo.toml`, `Cargo.lock`, `third-party/BUCK`, and
`infra/` manifests — NOT from ADRs. ADR numbers appear only where the Cargo.toml/config
comment itself cites them as the in-file rationale.

**NOW vs ASPIRATION distinction used throughout:**
- **OWNED-IMPL** = real client library wired into the crate (resolved in Cargo.lock).
- **VENDORED BRIDGE** = third-party crate wraps an external system at an adapter seam.
- **TRAIT-STUB** = a `-kernel`/`-adapter` crate that declares the port but has **no
  client dependency** — the "adapter" today is a pure-trait or in-memory placeholder.
  This is the dominant pattern and the single most important finding.

Scope sampled: root `Cargo.toml` (723 workspace members, `resolver=2`, edition 2024,
rust-version 1.95.0), the `[workspace.dependencies]` table (lines 1068–1156), `Cargo.lock`
(resolved versions), `third-party/BUCK` (vendored crate set), `infra/` (observability,
talos, cilium, capi, kyverno, gitops, seaweedfs, external-secrets, forge, ci), `.github/`,
`Jenkinsfile`, `.buckconfig`, plus ~30 individual crate Cargo.tomls.

---

## 0. Headline findings

1. **Eventing is entirely TRAIT-STUB.** The Kafka / Pulsar / NATS / Redpanda / Valkey /
   Postgres event-bus adapters (`oya/workflow-engine/crates/oya-workflow-engine-event-bus-adapter-{kafka,pulsar,nats,redpanda,...}`)
   each depend ONLY on the parent `oya-workflow-engine-event-bus-adapter` crate. **No
   `rdkafka`, no `pulsar`, no `async-nats` anywhere** — confirmed: `grep rdkafka|pulsar|async-nats`
   across all Cargo.tomls = 0 hits; `third-party/BUCK` contains 0 of these crate names.
   The only real broker client in the lock is `redis 0.27.6` (Valkey-compatible), wired in
   exactly one crate (`oya-cloud-intelligence-eventsink-valkey-adapter`).
2. **Most cloud "adapters" are trait-stubs.** S3 (`oya-cloud-storage-adapter-s3`), object-store
   kernel, vector-store kernel, timeseries kernel, OIDC client kernel, and the OpenBao **KMS**
   adapter all have empty/path-only `[dependencies]` — no `aws-sdk-s3`, no `object_store`, no
   vector-DB client, no OpenBao SDK. They define ports; backends are deferred.
3. **The real, wired backends are a short list:** `sqlx` (Postgres), `clickhouse`, `redis`
   (Valkey), `cedar-policy`, `kube`/`k8s-openapi`, `webauthn-rs`, `leptos`/`wasm-bindgen`,
   `reqwest`, `tonic`, `prometheus`, `opentelemetry`.
4. **CI is contradictory between code and config.** `Jenkinsfile` header asserts it "replaces
   the retired GitHub Actions workflows" (cites ADR-0361), yet a live GHA workflow
   (`.github/workflows/backbone-microservices-ci.yml`) still exists — and it globs the **legacy
   flat `crates/oya-*` layout** that no longer matches the real `oya/<ms>/crates/` tree. So
   GHA is present-but-stale; Jenkins is the intended gate; buck2 is the affected-scope engine.
5. **Dual HTTP backbone.** Both `hyper 1` (workspace pin, doctrine: hyper-direct outbound)
   and `hyper 0.14.32` (transitive, via reqwest/older deps) resolve in `Cargo.lock`.
6. **Crypto is single-backend by mandate:** `aws-lc-rs` everywhere; `ring` explicitly
   forbidden (in-file comments, ADR-0506). rustls via aws-lc-rs; no native-tls/OpenSSL.

---

## 1. Language / Build / HTTP runtime

| Concern | NOW (cited) | Status |
|---|---|---|
| Language | Rust, **edition 2024**, `rust-version = "1.95.0"` (`Cargo.toml:730-732`) | OWNED |
| Workspace | single virtual workspace, **723 members**, `resolver = "2"` (`Cargo.toml:1-727`) | OWNED |
| Primary build | **cargo** (workspace) | OWNED |
| Secondary build | **buck2** — `.buckconfig` (cells: root/prelude/toolchains/third-party; bundled prelude), `BUCK`/`.buckroot` at root + `third-party/BUCK` (566 KB vendored set); `infra/ci/buck2-affected-gate.sh` | VENDORED BRIDGE (affected-scope gating) |
| Async runtime | **tokio 1** (`rt-multi-thread,net,macros`, `Cargo.toml:1102`) — used by **51** crates | OWNED |
| HTTP server | **axum 0.8** (`Cargo.toml:1111`, lock 0.8.9) — 12 crates; **hyper 1** + **hyper-util 0.1** + **tower 0.5** (`:1100-1119`) | OWNED |
| HTTP (transitive) | **hyper 0.14.32** also resolved (lock) — dual-hyper | mixed |
| HTTP client | **reqwest 0.13** (`rustls,blocking,json`, `:1117`, lock 0.13.4) — 9 crates (GitHub adapters, OpenBao intelligence adapter, gateways) | OWNED |
| TLS / crypto | **aws-lc-rs 1** (`:1074`), **hyper-rustls 0.27** (aws-lc-rs+webpki-tokio, `:1108`); `ring` FORBIDDEN; 7 crates pin aws-lc-rs | OWNED, single-backend |
| RustCrypto | **hmac 0.12 / sha2 0.10 / subtle 2.6** (webhook HMAC verification, `:1122-1124`) | OWNED |
| gRPC | **tonic 0.14.6** + **tonic-prost** + **prost 0.14.3** + `protoc-bin-vendored 3.2.0` (`:1077-1081`) — 5 crates; transport in `oya-shared-backbone-grpc-transport-adapter` | OWNED |
| Serialization | **serde 1.0 / serde_json / serde_yaml 0.9 / toml 0.8** (`:1075-76,1134-35`) | OWNED |
| CLI / util | **clap 4.5, anyhow 1, tempfile 3, walkdir 2, regex 1** (`:1130-1136`) | OWNED |
| Release profile | `panic=abort, lto=true, codegen-units=1, strip=symbols` (`:1207-1211`) | OWNED |

**Release profile** is hardened for static-binary/distroless deploy (`Dockerfile.distroless` at root).

---

## 2. Datastores

| Store | NOW (cited) | Status |
|---|---|---|
| **Postgres** | **sqlx 0.8.6** (`runtime-tokio-rustls,postgres`, `Cargo.toml:1082`, lock 0.8.6). Real adapter: `libs/oya-shared-postgres-command-adapter-sqlx` (deps: `sqlx.workspace`). **6** crates use sqlx. Also `oya-tenant-rbac-postgres-rls-*` (RLS). | OWNED-IMPL |
| **ClickHouse** | **clickhouse 0.13** (`lz4`, `:1145`, lock 0.13.3). Isolated to ONE crate `libs/oya-shared-olap-clickhouse-adapter` (in-file: "dependency-seam adapter-only", pin matches LTS 26.3.10.60). | OWNED-IMPL (1 seam) |
| **Valkey / Redis** | **redis 0.27** (`tokio-comp,streams`, lock 0.27.6) — pinned **directly in one crate**, NOT in workspace deps: `oya-cloud-intelligence-eventsink-valkey-adapter` (XADD to Valkey Streams). The `event-bus-adapter-valkey` is a separate trait-stub (no redis dep). | OWNED-IMPL (1 seam) |
| **TimescaleDB** | none. `libs/oya-shared-timeseries-kernel` deps = **empty**. | TRAIT-STUB |
| **Milvus / vector DB** | none. `libs/oya-shared-vector-store-kernel` deps = **empty**; no qdrant/milvus/lance/tantivy in lock or BUCK. `oya-search-index-vector-domain` is domain-only. | TRAIT-STUB |
| **SeaweedFS / object store** | no Rust client. `libs/oya-shared-object-store-kernel` deps = empty; `oya-cloud-storage-adapter-s3` deps = path-only (no `aws-sdk-s3`/`object_store`). SeaweedFS exists only as **infra k8s manifest** `infra/seaweedfs/seaweedfs.k8s.yaml`. | TRAIT-STUB (code) / infra-only |
| **Citus** | not referenced (plain Postgres via sqlx). | absent |

**Net:** of the datastore catalog, only **Postgres, ClickHouse, Valkey** are actually wired
into Rust code. Object-store, vector, timeseries are ports awaiting backends.

---

## 3. Eventing / Messaging

| Broker | NOW (cited) | Status |
|---|---|---|
| **Kafka** | `oya-workflow-engine-event-bus-adapter-kafka` — deps = **parent crate only**. No `rdkafka`. | TRAIT-STUB |
| **Pulsar** | `...-adapter-pulsar` — parent-only. No `pulsar` crate. | TRAIT-STUB |
| **NATS** | `...-adapter-nats` — parent-only. No `async-nats`. | TRAIT-STUB |
| **Redpanda** | `...-adapter-redpanda` — parent-only (Redpanda is Kafka-API; still no client). | TRAIT-STUB |
| **Valkey (event bus)** | `...-adapter-valkey` — parent-only stub. (Distinct from the intelligence eventsink which DOES use `redis`.) | TRAIT-STUB |
| **Postgres (event bus)** | `...-adapter-postgres` — parent-only. | TRAIT-STUB |
| **Outbox pattern** | OWNED in-house: `oya-shared-transactional-outbox-*` (kernel/adapter-sqlx/dispatch-app/poller/worker/runtime-tokio) + `oya-shared-outbox-broker-http-adapter` (transports over HTTP, deps = dispatch-app only). The real eventing today is the **transactional-outbox-over-Postgres(sqlx)+HTTP**, not a message broker. | OWNED-IMPL |

**Net:** ALL named brokers (Kafka/Pulsar/NATS/Redpanda) are non-functional stubs. The
functioning event path is the in-house transactional outbox (sqlx + HTTP broker adapter).

---

## 4. Policy / AuthZ

| Concern | NOW (cited) | Status |
|---|---|---|
| **Cedar** | **cedar-policy 4.11** (`Cargo.toml:1140`, lock 4.11.0). Wired into 5 adapter crates: `managed-k8s-tenant-quota-adapter-cedar`, `cloud-intelligence-authz-cedar-adapter`, `identity-workload-authz-cedar-adapter` (+ its app), `ci-webhook-gateway-authz-cedar-adapter`. Also a `oya/policy/crates/oya-policy-cedar-{api,domain}` BC. | OWNED-IMPL (vendored Cedar engine) |

---

## 5. Identity / Crypto / Auth

| Concern | NOW (cited) | Status |
|---|---|---|
| **WebAuthn** | **webauthn-rs 0.5.5** (lock). Consumers: `libs/oya-shared-webauthn-server-kernel` (in-file: "wraps webauthn-rs v0.5+ at adapter boundary" — but its own `[dependencies]` list only serde; webauthn-rs is pulled via `oya/identity/crates/oya-identity`). | VENDORED BRIDGE |
| **OIDC** | `libs/oya-shared-oidc-client-kernel` deps = serde only; `oya-identity-oidc-issuer-kernel`, `oya-identity-workload-oidc-adapter` present. No external OIDC SDK wired. | TRAIT-STUB |
| **Zitadel** | **not referenced** anywhere (0 hits in Cargo.toml/lock/BUCK). Identity is in-house OIDC issuer + webauthn-rs, not Zitadel. | absent |
| **aws-lc-rs** | crypto backbone (see §1); FIPS backend deferred (in-file ADR-0506). | OWNED |
| **KMS / secrets** | `oya-cloud-kms-adapter-openbao` deps = **path-only (no OpenBao SDK)** → TRAIT-STUB. `oya-cloud-intelligence-openbao-adapter` talks to OpenBao via **reqwest + base64** (REST), not an SDK. OpenBao itself is infra: `infra/external-secrets/clustersecretstore-openbao-oya.yaml`. | mixed (1 stub, 1 reqwest-REST bridge) |

---

## 6. Kubernetes / OS / Cloud-native substrate

| Concern | NOW (cited) | Status |
|---|---|---|
| **kube-rs** | **kube 3.1** (`client,runtime,rustls-tls`, `:1154`, lock 3.1.0) + **kube-runtime 3.1** + **k8s-openapi 0.27** (lock 0.27.1). Consumers: `managed-k8s-control-plane-host-adapter-capi`, `ci-controller-k8s-adapter` (+ app). In-file: the CAPI/Kamaji reconciliation is "honest-deferred" — adapter returns typed `Unimplemented`. | VENDORED BRIDGE (partial; live reconcile deferred) |
| **Talos** | OS substrate — `infra/talos/` (schematic.yaml bakes **Kata Containers / cloud-hypervisor** extension; controlplane/worker patches; cilium-values; bare-metal + installation-media). In-file cites ADR-0147 (kata-cloud-hypervisor pin). | infra-only (config, no Rust) |
| **Cluster API (CAPI)** | `infra/capi/` (clusterctl.yaml, clusters Helm chart, ClusterResourceSet) + `infra/sidero-metal/` (bare-metal CAPI provider) + `cloud/managed-k8s-control-plane-host/...-adapter-capi`. | infra + stubbed adapter |
| **Cilium** | `infra/cilium/cell-boundaries` + `infra/talos/cilium-values.yaml` (CNI). | infra-only |
| **Kyverno** | `infra/kyverno/` (policies + `oya-vcs-admission`) — admission policy engine. | infra-only |
| **Karpenter** | **not referenced.** | absent |
| **Istio** | **not referenced** (Cilium is the network layer). | absent |
| **Kata Containers** | runtime extension via Talos (`smoke-kata.sh`, `kata-runtimeclass.yaml`). | infra-only |
| **GitOps / Argo** | `infra/gitops/` (Helm chart, `root-app.yaml` = Argo app-of-apps), `infra/ci/argocd/` (Argo Rollouts demo, git-server). | infra-only |
| **External Secrets** | `infra/external-secrets/` (ClusterSecretStore→OpenBao, ExternalSecret for forgejo CI token). | infra-only |
| **Container registry** | `infra/registry/registry.k8s.yaml`. | infra-only |

---

## 7. CI/CD

| Concern | NOW (cited) | Status |
|---|---|---|
| **Jenkins** | **Intended primary gate.** Root `Jenkinsfile` (in-file: "ADR-0361 … replaces the retired GitHub Actions workflows"; reports `oya-verify`/`oya-supply-chain`/`oya-pr-review` status contexts; merge-queue serialized). Shared lib `infra/ci/jenkins/shared-library/vars/oyaCiLane.groovy`; seed jobs (farmwide/parallel-lanes/smoke); BuildKit agent image. | LIVE (declared canonical) |
| **GitHub Actions** | **Present but stale.** Only `.github/workflows/backbone-microservices-ci.yml` remains; its path globs target the **legacy flat `crates/oya-*` layout** (e.g. `crates/oya-messenger-message-stream-*`) which does NOT match the real `oya/<ms>/crates/` tree → effectively dormant. Contradicts the Jenkinsfile "retired GHA" claim. | STALE / contradiction |
| **buck2** | affected-scope engine: `infra/ci/buck2-affected-gate.sh`, `BUCK`/`third-party/BUCK`. | LIVE (gating) |
| **oya-ci / oya-cd "shadow"** | The bespoke CI is materializing as **first-class Rust microservices**, not just config: `oya/ci-controller/` (kernel + github-adapter + k8s-adapter + app), `oya/ci-tide/` (merge-queue: kernel + github-adapter + app, uses reqwest for GitHub status/merge API), `oya/ci-webhook-gateway/` (app + ed25519/github/jenkins/cedar adapters + HMAC verify). Plus `infra/ci-webhook-gateway/`. | OWNED-IMPL (in-house CI product, dogfooded) |
| **Forgejo** | Still present in infra: `infra/forge/` (forgejo.yaml, argocd-app, jenkins token template) + `infra/external-secrets/externalsecret-forgejo-ci-token.yaml`. | infra-only (slated for removal per backlog) |

**Net:** three live CI surfaces overlap — Jenkins (declared canonical gate) + buck2
(affected scope) + an in-house Rust `oya-ci` product (controller/tide/webhook-gateway) —
while a stale GHA workflow and Forgejo manifests linger. This is an in-flight migration, not
a settled pipeline.

---

## 8. Observability

| Concern | NOW (cited) | Status |
|---|---|---|
| **Metrics store** | **VictoriaMetrics v1.111.0** (Prometheus-compatible, single-node) — `infra/observability/observability.k8s.yaml`. In-file header explicitly chooses VM over Mimir ("low-resource, Apache-2.0"). | infra LIVE |
| **OTLP ingest** | **OpenTelemetry Collector** `otel/opentelemetry-collector-contrib:0.119.0` → prometheusremotewrite → VictoriaMetrics. | infra LIVE |
| **Dashboards** | **Grafana 11.4.0** (datasource = VictoriaMetrics-as-prometheus; admin pw from Secret). | infra LIVE |
| **LGTM (Loki/Tempo/Mimir)** | **NOT deployed.** In-file: "Traces/logs (Tempo/Loki/Vector) are a follow-up; metrics + OTLP + dashboards are the core." Mimir explicitly replaced by VictoriaMetrics. | deferred |
| **OTel (Rust SDK)** | **opentelemetry 0.32 + opentelemetry-otlp 0.32 + opentelemetry_sdk 0.32** (`:1085-1087`). Wired in `libs/oya-shared-hyperscaler-metrics-adapter-otlp`. | OWNED-IMPL |
| **prometheus (Rust)** | **prometheus 0.13** in `libs/oya-shared-hyperscaler-metrics-adapter-prometheus`. Metrics kernel has two real adapters (otlp + prometheus). | OWNED-IMPL |
| **Tracing** | **tracing 0.1.44 + tracing-subscriber 0.3.23** (json/env-filter, `:1083-1084`); `oya-shared-tracing-client-kernel`, `oya-observability-tracing-adapter`. | OWNED-IMPL |

**Net:** observability is **OTel→VictoriaMetrics→Grafana**, NOT the LGTM stack. Loki/Tempo/Mimir
are absent today; Mimir was deliberately swapped for VictoriaMetrics.

---

## 9. Frontend

| Concern | NOW (cited) | Status |
|---|---|---|
| **Leptos** | **leptos 0.8.19** (pinned `=`, lock 0.8.19) in `oya/application/crates/oya-application-shell-frontend-prototype` (SSR + hydrate, `[package.metadata.leptos]` site config, port 3000). Named "prototype". | OWNED-IMPL (prototype) |
| **WASM** | **wasm-bindgen =0.2.121, wasm-bindgen-futures =0.4.71, web-sys =0.3.98** under `cfg(target_arch="wasm32")`; `console_error_panic_hook =0.1.7`. | OWNED-IMPL |
| **Yew / Dioxus** | not referenced (Leptos is the chosen frontend). | absent |
| Governance | a check crate `libs/oya-check-client-stack-discipline` enforces the Leptos/WASM client-stack choice. | OWNED |

---

## 10. Coverage / caps (no silent caps)

**Fully opened (Cargo.toml read):** root workspace + dependencies table; clickhouse adapter;
valkey eventsink adapter; webauthn kernel; vector-store kernel; all 4 event-bus broker
adapters (kafka/pulsar/nats/redpanda); identity; oidc-client-kernel; object-store-kernel;
postgres-command-adapter-sqlx; timeseries-kernel; storage-adapter-s3; frontend-prototype;
ci-controller-github-adapter; ci-tide-github-adapter; cloud-kms-openbao adapter;
cloud-intelligence-openbao adapter; ulid-id-kernel; grpc-transport-adapter; outbox-broker-http-adapter;
prometheus + otlp metrics adapters. **Cargo.lock** grepped for ~35 candidate libs (resolved
versions cited). **third-party/BUCK** grepped for ~20 broker/db/identity crate names.

**Config opened:** `.buckconfig`, root `Jenkinsfile`, `.github/workflows/backbone-microservices-ci.yml`
(head), `infra/observability/observability.k8s.yaml`, `infra/talos/schematic.yaml`, plus directory
listings of all `infra/` subdirs and `cloud/` services.

**NOT individually opened (acknowledged caps):** the other ~700 crate Cargo.tomls were not each
read — findings rely on (a) Cargo.lock resolved set, (b) `third-party/BUCK` vendored set, (c)
workspace `[dependencies]` table, and (d) targeted greps for each library across all Cargo.tomls.
This is sound for "which libs are in use" but a per-crate stub-vs-impl census of all 700 was not
performed. Specifically NOT opened: every `oya/*/crates/*-domain` (assumed std-only domain layer),
the full ERP vertical set (hr/payroll/crm/treasury/warehouse/etc.), and all `libs/oya-check-*` /
`libs/oya-governance-*` (assumed std + serde governance checks). `Cargo.lock` was not exhaustively
diffed against the workspace table for transitive-only surprises beyond the ~35 probed names.
`infra/cilium/cell-boundaries`, `infra/kyverno/policies`, and the Jenkins groovy seed jobs were
listed but not line-read.
