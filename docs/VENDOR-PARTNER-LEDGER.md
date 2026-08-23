---
purpose: Oyatie — Vendor + Partner Ledger
doc_status: published
---

# Oyatie — Vendor + Partner Ledger

> **Status:** Draft v0.1 skeleton — 2026-05-09. Concrete entries populate as deps are adopted.
> **Owner:** `gtm-partnerships` + `ops-security` (license-tier review).
> **Companion:** [`decisions/ADR-0013-product-license-policy.md`](decisions/ADR-0013-product-license-policy.md), [TOOLCHAIN.md §3 + §7](TOOLCHAIN.md), [COMPLIANCE-MATRIX.md](COMPLIANCE-MATRIX.md).

## 1. Vendor categories

| Category | Examples |
|---|---|
| Hyperscaler substrate | OCI, AWS |
| KR cloud peer | Naver Cloud, NHN Cloud, KT Cloud, Kakao Cloud |
| AI model providers | Anthropic, OpenAI, Google Gemini |
| Hardware partners | Samsung Foundry / fab partner; NVIDIA / AMD / Intel for GPU; Ampere / ARM for CPU; Supermicro / Dell / Inspur for chassis |
| KR SI partners | Samsung SDS, LG CNS, SK C&C, POSCO ICT |
| KR adtech ecosystem | KODA, 한국디지털광고협회, Kakao Moment, Naver 검색광고, Daum |
| KR identity | NICE, KCB, SCI Plus, KCS (본인확인서비스 designated) |
| KR payment partners | KICC, NICEpay, Toss Payments, Inicis, KakaoPay, NaverPay |
| KR settlement | KFTC (금융결제원), BOK (한국은행), KSD (한국예탁결제원) |
| Per-region equivalent partners | per regional pack |

## 2. External dependency ledger (Rust crate sample)

Per [TOOLCHAIN §3](TOOLCHAIN.md) language-stack matrix. Each row is a real or planned dep.

| Dependency | Version | License | Tier | Purpose | Owner | Replacement plan |
|---|---|---|---|---|---|---|
| `tokio` | 1.x | MIT | allowed-kernel | Async runtime | `axis-foundry` | n/a |
| `axum` | 0.7+ | MIT | allowed-kernel | HTTP server | `platform-api-sdk` | n/a |
| `serde` | 1.x | MIT/Apache-2 | allowed-kernel | Serialization | `axis-foundry` | n/a |
| `sqlx` | 0.7+ | MIT/Apache-2 | allowed-kernel | Postgres driver | `platform-eventing-og` | n/a |
| `rustls` | 0.22+ | MIT/Apache-2/ISC | allowed-kernel | TLS | `ops-security` | n/a |
| `tracing` + `tracing-subscriber` | latest | MIT | allowed-kernel | Structured logging | `ops-sre-reliability` | n/a |
| `chrono` / `time` | latest | MIT/Apache-2 | allowed | Date / time | various | n/a |
| `clap` | 4.x | MIT/Apache-2 | allowed | CLI parsing | `axis-foundry` (CLI) | n/a |
| `bacon` | latest | Apache-2 / MIT | allowed | Background dev watcher (`cargo check / clippy / nextest` on save); engineer's primary feedback loop | dev-only | n/a |
| `cargo-machete` | latest | Apache-2 / MIT | allowed | Unused-dependency sweeper per-PR + quarterly; surfaces accidental dep adoption | dev-tool + CI lane | n/a |
| `cargo-nextest` | latest | Apache-2 / MIT | allowed | Canonical test runner; never bare `cargo test`; per project memory + ADR-0024 | dev + CI | n/a |
| `sccache` | latest | Apache-2 / MIT | allowed | Compilation cache local + S3-backed remote; 60-90% incremental cache hit | dev + CI | n/a |
| `wasmtime` | latest | Apache-2 | allowed | WASM sandbox per ADR-0023 | `axis-foundry` | n/a |
| `tantivy` | latest | MIT | allowed | Inverted index for search per ADR-0047 | `axis-search` | replace with in-house if scale demands |
| `pgvector` (KEEP `vector` Rust binding) | TBD | MIT-style | allowed | Vector store per ADR-0047 | `platform-eventing-og` + `axis-search` | scale-tier replacement (Milvus gated; in-house HNSW long-horizon) |
| `cargo-deny` | latest | MIT/Apache-2 | allowed | License scanning gate | `ops-security` | n/a |
| Cosign | latest | Apache-2 | allowed | Sigstore signing per ADR-0039 | `ops-security` | n/a |
| Trivy | latest | Apache-2 | allowed | 4-layer scan per ADR-0039 | `ops-security` | n/a |
| Istio Ambient | latest | Apache-2 | allowed | Service mesh per ADR-0044 | `axis-cloud` | n/a |
| Envoy | latest | Apache-2 | allowed | Gateway per ADR-0013 | `axis-cloud` | n/a |
| Harbor | latest | Apache-2 | allowed | Container registry per ADR-0044 | `axis-cloud` | n/a |
| OpenBao | latest | MPL-2 | allowed | Secrets per ADR-0043 | `ops-security` | n/a |
| OpenTofu | latest | MPL-2 | allowed | IaC per ADR-0050 | `axis-cloud` | n/a |
| VictoriaMetrics | latest | Apache-2 | allowed | Metrics per ADR-0045 | `ops-sre-reliability` | n/a |
| Yrs (Rust port of Y.js) | latest | MIT | allowed | CRDT for Workspace Docs | `axis-workspace` | n/a |
| `webrtc-rs` | latest | MIT/Apache-2 | allowed | Meet SFU primitives | `axis-workspace` | in-house tuning |
| **Grafana 10.x** | latest | AGPL-3.0 | **forbidden in product code; dev-only carve-out** | Visualization | `ops-sre-reliability` | replace with in-house Leptos observability portal OR commercial Grafana licensing — gated |
| **Loki / Tempo** | latest | AGPL-3.0 | **forbidden in product code; dev-only carve-out** | Log + trace store | `ops-sre-reliability` | in-house substitution OR commercial license |
| **Mimir** | latest | AGPL-3.0 | **forbidden in product code; dev-only carve-out** | Long-term metrics | `ops-sre-reliability` | replace with VictoriaMetrics (already adopted) |
| **PostGIS** | latest | GPL-2 | **requires-review** | Geospatial extension; product use needs legal isolation analysis | `axis-cloud` (or replace with geo-rs in-house) | `geo-rs` + Sedona-class long-horizon |
| **pgroonga** | latest | LGPL | **requires-review** | KR morphology in Postgres for Search per ADR-0047 | `axis-search` | port mecab-ko in Rust + Tantivy long-horizon |
| **WireGuard kernel module** | n/a | GPL-2 | **dev/admin only; not product runtime** | Bastion VPN | `axis-cloud` | BoringTun (BSD-3) at scale |
| **MinIO** | latest | AGPL-3.0 | **forbidden** | Object storage | `axis-cloud` | use OCI Object Storage; in-house at scale |
| **Redis 7.4+** | n/a | RSAL | **requires-review** (or pin pre-7.4) | KV cache | various | Valkey (BSD-3) or DragonflyDB (BSD-3) |
| **HashiCorp Vault 1.14+** | n/a | BUSL | **forbidden** | Secrets | `ops-security` | OpenBao (already adopted) |
| **HashiCorp Terraform 1.6+** | n/a | BUSL | **forbidden** | IaC | `axis-cloud` | OpenTofu (already adopted) |
| **Elasticsearch 7.11+** | n/a | SSPL | **forbidden** | Search backend | `axis-search` | OpenSearch (Apache-2) — OpenSearch path explicitly required per ADR-0047 |

## 3. Partner ledger

| Partner | Type | Region | Status |
|---|---|---|---|
| OCI (Oracle Cloud Infrastructure) | Hyperscaler substrate | Global; KR-Seoul + KR-Chuncheon | Active per ADR-0021/0117/0119/0173/0183 |
| AWS | Hyperscaler substrate | Global | Active opportunistic |
| Anthropic | AI provider (Claude) | Global | Active per Foundry adapter |
| OpenAI | AI provider | Global | Active per Foundry adapter |
| Google Gemini | AI provider | Global | Active per Foundry adapter |
| Naver Cloud | KR cloud peer | KR | Co-sell candidate |
| KICC / NICEpay / Toss Payments | KR PG | KR | Per fintech vertical |
| KISA-designated 본인확인서비스 providers | KR identity | KR | Per regulated tenant onboarding |
| KR Big-4 SI (Samsung SDS, LG CNS, SK C&C, POSCO ICT) | Delivery | KR | Channel partner candidates |

## 4. Contract recency ledger

> **As of:** 2026-05-10.
> **Gate:** `presubmit` (retired CLI `gate validate vendor-contract-recency`).
> **Bootstrap declaration:** No signed vendor or partner contract rows are recorded in this ledger yet; replace the declaration row with one row per signed contract before any signed vendor / partner contract is adopted.

| Contract ID | Vendor / partner | Status | Expiry date | Renewal task | Owner |
|---|---|---|---|---|---|
| `vcr-no-signed-contracts-2026-05-10` | All listed vendors and partners | no-signed-contracts | n/a | n/a | `gtm-partnerships` + `ops-security` |

## 5. Contract recency SLA

- All contracts ≥ 90 days from expiry get a renewal task auto-opened per `governance-vendor-contract-recency`
- Per quarter: vendor-risk review per `ops-security` + `gtm-partnerships`
- Per `EVT-LICENSE-POLICY-CHANGE`: re-review affected vendor

## 6. Sources
[`decisions/ADR-0013-product-license-policy.md`](decisions/ADR-0013-product-license-policy.md), [TOOLCHAIN.md §3 + §7](TOOLCHAIN.md), Cargo.toml workspace deps, ADR-0013/0014/0039/0044/0050.


---

> **§Note (2026-05-21 transition):** References to `governance-*` in this historical document are intentional — they describe past state. New work uses `governance-*` per the 2026-05-21 transition directive.