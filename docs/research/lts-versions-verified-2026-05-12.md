---
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
doc_status: published
---
# LTS Versions Verified — 2026-05-12

Captured by real-time web lookup. All versions reflect upstream-current LTS or stable channel as of 2026-05-12. Sources cited inline; full URL list at bottom.

Methodology: live web search against canonical upstream sources (project websites, GitHub releases, PyPI, npm, crates.io). For projects that do not publish a formal "LTS" track (SQLite, Cosign, OpenTelemetry, Envoy, K8s upstream, BoringSSL), the substitute reference is the current stable / mainline tag. Provider-SDK rows reflect the **published current release**, not LTS, because none of the three frontier-model SDKs publishes LTS.

---

## Languages

| Component | Current LTS / stable | Release date | Source URL | Notes |
|---|---|---|---|---|
| Rust toolchain | 1.95.0 | 2026-04-16 | https://blog.rust-lang.org/2026/04/16/Rust-1.95.0/ | stable channel; no formal LTS (six-week cadence). 1.96.0 beta tracking 2026-05-28. |
| Node.js Active LTS | 24.15.0 | 2026 (Active LTS) | https://nodejs.org/en/blog/release/v24.15.0 | Node 22 in Maintenance LTS; Node 26 is Current (LTS Oct 2026). |
| Python | 3.14.5 | 2026-05-10 | https://blog.python.org/2026/04/python-3150a8-3144-31313/ | 3.14 series is the latest feature release; 3.13.13 (2026-04-07) is the live maintenance line. |
| Go | 1.26.3 | 2026-05-07 | https://go.dev/doc/devel/release | Major Go 1.26 released 2026-02-10; latest patch 1.26.3. |
| TypeScript | 6.0 (stable) | early 2026 | https://www.typescriptlang.org/docs/handbook/release-notes/typescript-6-0.html | 7.0 Beta announced 2026-04-21 (Go-port); 6.0 is current stable. |

## Databases

| Component | Current LTS / stable | Release date | Source URL | Notes |
|---|---|---|---|---|
| PostgreSQL | 18.3 (also 17.9, 16.13, 15.17, 14.22) | 2026-02-26 | https://www.postgresql.org/about/news/postgresql-183-179-1613-1517-and-1422-released-3246/ | 18 is current major; 14 still in support. Five-year support per major. |
| ClickHouse LTS | 26.3.10.60-lts | 2026-05-08 | https://github.com/ClickHouse/ClickHouse/releases | 26.3 designated LTS; async inserts default; native JSON type. |
| SQLite | 3.53.1 | 2026-05-05 | https://sqlite.org/releaselog/3_53_1.html | **No LTS program** — current stable is the substitute. 3.53.0 was 2026-04-09. |
| Redis | 8.6.3 | May 2026 | https://eosl.date/eol/product/redis/ | Tri-license RSALv2/SSPLv1/AGPLv3 since 8.0 — **NOT cargo-deny compatible** for product code. Use Valkey or pre-7.4 if license posture matters. |

## Infra / orchestration

| Component | Current LTS / stable | Release date | Source URL | Notes |
|---|---|---|---|---|
| Kubernetes | 1.36.0 | 2026-04-22 | https://kubernetes.io/blog/2026/04/22/kubernetes-v1-36-release/ | N-2 policy (1.34/1.35/1.36 supported). Upstream has no LTS; **Canonical 1.32 LTS** is the long-track option (support through 2040). |
| Docker Engine | 29.3.1 | May 2026 | https://docs.docker.com/engine/release-notes/29/ | v29 makes containerd image store default. |
| containerd LTS | 2.3.0 | 2026-04-30 | https://github.com/containerd/containerd/releases | **First annual LTS**; 4-month minor cadence (Apr/Aug/Dec); 2-year support. |
| Istio | 1.29.2 (latest patch); 1.27/1.28/1.29 supported | 2026-04-13 | https://istio.io/latest/news/releases/1.29.x/announcing-1.29.1/ | No formal LTS; quarterly minors with N+2 support window. 1.29 GA 2026-02-16. |
| Envoy | 1.37.1 | early 2026 | https://github.com/envoyproxy/envoy/releases | Quarterly stable cadence (15th of each quarter). 1.36.5 also supported. |

## Container base images

| Component | Current LTS / stable | Release date | Source URL | Notes |
|---|---|---|---|---|
| Debian stable (trixie) | 13.4 | 2026-03-14 | https://www.debian.org/News/2026/20260314 | Full support to 2028-08-09, LTS to 2030-06-30. Debian 12 goes to LTS June 2026. |
| distroless/static-debian13 | current | rolling | https://github.com/GoogleContainerTools/distroless/blob/main/SUPPORT_POLICY.md | **debian13 is current**; debian12 deprecated path — migrate. EOL Sep 2026 (or D14+1y). |
| distroless/cc-debian13 | current | rolling | https://github.com/GoogleContainerTools/distroless/blob/main/SUPPORT_POLICY.md | Use cc-debian13 for glibc-linked binaries; same EOL window as static. |
| Alpine Linux | 3.23.4 | 2026-04-15 | https://www.alpinelinux.org/posts/Alpine-3.20.9-3.21.6-3.22.3-3.23.3-released.html | 3.23.4 supported through 2027-11-01; 3.22 to 2027-05-01. |

## Observability

| Component | Current LTS / stable | Release date | Source URL | Notes |
|---|---|---|---|---|
| OpenTelemetry Collector | v0.151.0 | 2026-04-30 | https://github.com/open-telemetry/opentelemetry-collector-releases/releases | No formal LTS; components have mixed stability tags. |
| Prometheus LTS | 3.5 LTS | 2025-06-03 (EOS 2026-07-31) | https://prometheus.io/docs/introduction/release-cycle/ | LTS = 1 year of bug/security/docs only. 3.11.0 is current mainline (2026-04-02) per https://github.com/prometheus/prometheus/releases/tag/v3.11.0. **3.5 LTS expires soon — plan migration.** |
| Grafana | 13.0.1+security-01 | 2026-05-12 | https://grafana.com/blog/grafanacon-2026-announcements/ | **No formal LTS program**. Current major + prior minor get patches. Grafana 13 launched at GrafanaCON 2026. |

## Security / supply chain

| Component | Current LTS / stable | Release date | Source URL | Notes |
|---|---|---|---|---|
| OpenSSL | 3.5 LTS (and 4.0) | 4.0 final 2026-04-14 | https://openssl-library.org/post/2026-04-14-openssl-40-final-release/ | 3.5 is the active LTS; 4.0 supported through 2027-05-14. 3.0 LTS ends 2026-09-07. New LTS cadence: every 2 years. |
| BoringSSL | rolling main (snapshot from boringssl.googlesource.com/+/master) | continuous | https://boringssl.googlesource.com/boringssl/+/master | **No LTS** — Google explicitly publishes no stable branch. Track `chromium-stable` ref if a pinned snapshot is needed. |
| Cosign | v3.0.6 | 2026-04-06 | https://github.com/sigstore/cosign/releases | v3 is the current major; `--bundle` is now required (breaking from v2). |
| Trivy | v0.70.0 | 2026-04-17 | https://github.com/aquasecurity/trivy/releases | **Avoid v0.69.4** — compromised supply-chain incident 2026-03-19. 0.69.3 (immutable) and 0.70.0 are safe. |
| cargo-deny | 0.19.5 | 2026-05-09 | https://github.com/EmbarkStudios/cargo-deny/blob/main/CHANGELOG.md | MSRV 1.85.0, edition 2024. Adds SARIF output; compatible with workspace Rust 1.95.0. |

## Provider SDKs

| Component | Current published | Release date | Source URL | Notes |
|---|---|---|---|---|
| Anthropic Python (`anthropic`) | 0.101.0 | 2026-05-11 | https://pypi.org/project/anthropic/ | Official. No LTS. |
| Anthropic TypeScript (`@anthropic-ai/sdk`) | 0.95.2 | 2026-05-11 | https://www.npmjs.com/package/@anthropic-ai/sdk | Official. No LTS. |
| Anthropic Rust | **No official SDK** | — | https://crates.io/crates/anthropic-ai-sdk | Community options: `anthropic-ai-sdk` 0.2.27, `claudius`, `adk-anthropic`. All MIT/Apache, **none official**. Recommend internal HTTP client over reqwest with hand-rolled types. |
| OpenAI Python (`openai`) | 2.36.0 | 2026-05-07 | https://pypi.org/project/openai/ | Official. |
| OpenAI TypeScript (`openai`) | 6.36.0 | 2026-05-11 | https://www.npmjs.com/package/openai | Official; requires Node ≥ 20. |
| OpenAI Rust (`async-openai`) | 0.38.1 | 2026-05-11 | https://github.com/64bit/async-openai/releases | **Unofficial** but de-facto standard. MIT. |
| Google Gemini Python (`google-genai`) | 2.0.1 | 2026-05-09 | https://pypi.org/project/google-genai/ | Unified SDK; `google-generativeai` legacy is deprecated. |
| Google Gemini TypeScript (`@google/genai`) | 2.0.1 | 2026-05-09 | https://www.npmjs.com/package/@google/genai | Unified SDK; Node ≥ 18. |
| Google Gemini Rust | **No official SDK** | — | https://crates.io/crates/gemini-rust | Community: `gemini-rust`, `google-generative-ai-rs`, `rust-genai`, `gemini-client-api`. **None official**. |

## Agent tooling

| Component | Current release | Release date | Source URL | License | Notes |
|---|---|---|---|---|---|
| rtk-ai/grit | v0.3.0 | 2026-04-06 | https://github.com/rtk-ai/grit/releases | Apache-2.0 (per org convention; see https://github.com/rtk-ai/icm/blob/main/LICENSE) | "Git for AI agents." Early-stage (0.3.x). |
| rtk-ai/icm | v0.10.39 | 2026 | https://github.com/rtk-ai/icm/releases | Apache-2.0 (https://github.com/rtk-ai/icm/blob/main/LICENSE) | "Permanent memory for AI agents." Single binary, MCP native. Apache-2.0 → cargo-deny pass. |
| rtk-ai/rtk (Rust Token Killer) | dev-0.39.0-rc.199 | 2026-04-29 | https://github.com/rtk-ai/rtk/releases | Apache-2.0 (LICENSE file) / website states MIT — discrepancy; LICENSE file authoritative | CLI proxy for token reduction (used by CLAUDE.md). |
| OpenBao | v2.5.3 | 2026-04-20 | https://github.com/openbao/openbao/releases | MPL-2.0 | Secrets manager (Vault fork). MPL-2.0 cleared by current `deny.toml`. |

---

## Currently-pinned in oyatie (cross-reference)

Sources read: `/Users/jasonlee/oyatie/rust-toolchain.toml` (`channel = "1.95.0"`); `/Users/jasonlee/oyatie/Cargo.toml` (workspace `rust-version = "1.95.0"`, edition 2024); `/Users/jasonlee/oyatie/rustfmt.toml` (`edition = "2024"`, `style_edition = "2024"`); `/Users/jasonlee/oyatie/docs/AGENTS.md` (Codex appendix references Node 20 for `pnpm build` / `pnpm test`); `/Users/jasonlee/oyatie/deny.toml` (license allow-list: 0BSD, Apache-2.0, BSD-2/3, ISC, MIT, MPL-2.0, Unicode-3.0).

| Component | Oyatie pinned | Current LTS / stable | Behind? | Severity |
|---|---|---|---|---|
| Rust toolchain | 1.95.0 (`rust-toolchain.toml`; workspace `rust-version`) | 1.95.0 (stable) | No | OK — cargo-deny 0.19 MSRV 1.85 is below the workspace pin. |
| Rust edition | 2024 (`workspace.package.edition`; rustfmt `edition`/`style_edition`) | 2024 available | No | OK — workspace crates and formatter policy are on the 2024 line. |
| Node.js | 20 (per docs/AGENTS.md Codex appendix) | 24 Active LTS (22 Maintenance) | One major behind LTS line | MED — Node 20 enters Maintenance Oct 2025; move to 22 or 24. |
| Python | not pinned in repo | 3.14.5 / 3.13.13 maintained | n/a | LOW (no Python product code in workspace currently). |
| Go | not pinned in repo | 1.26.3 | n/a | LOW (no Go service in workspace). |
| TypeScript | not pinned at repo root | 6.0 stable | unknown | MED — Foundry-workspace kernel (`governance-typescript-workspace-kernel`) should declare a TS floor. |
| PostgreSQL | not pinned (no migrations yet) | 18.3 | n/a | LOW. |
| ClickHouse | not pinned | 26.3 LTS | n/a | LOW. |
| Redis | not pinned | 8.6.3 (tri-license) | n/a | **HIGH if adopted** — RSALv2/SSPLv1/AGPLv3 fails `deny.toml`. Use Valkey or Redis ≤7.2 if needed. |
| Kubernetes | not pinned at cluster spec yet | 1.36.0 (Canonical LTS = 1.32) | n/a | LOW. |
| containerd | not pinned | 2.3 LTS | n/a | LOW. |
| Debian base | not pinned (no Dockerfile inventory found at root) | trixie / 13.4 | n/a | MED — `distroless/static-debian13` directive should be codified in DESIGN.md or release-pack contract. |
| OpenTelemetry | tracing 0.1.44 / tracing-subscriber 0.3.23 (workspace deps) | tracing crates current; collector v0.151.0 | tracing stack OK; collector unpinned | LOW. |
| OpenSSL | not pinned (likely via rustls or system) | 3.5 LTS / 4.0 | n/a | LOW — but document choice (rustls vs OpenSSL) per ADR. |
| Cosign | not yet integrated (D5 mentions signing) | v3.0.6 | n/a | **HIGH** (blocking) — `D5` Done-Definition cites Cosign signing for capability publish; pin missing. |
| Trivy | not pinned | v0.70.0 | n/a | MED — `D11` (`cargo deny check`) covers Rust but no container-image scan gate yet. |
| cargo-deny | invoked via `cargo deny check` (D11); version not pinned | 0.19.5 | n/a | MED — MSRV is compatible with Rust 1.95.0, but the tool version still needs a durable pin. |
| Anthropic / OpenAI / Gemini SDKs | not declared in any kernel Cargo.toml (RAG kernel exists but no provider crate) | see Provider SDKs table | n/a | MED — multi-provider adapter design must pick HTTP-direct vs community crate. |
| icm | used per repo CLAUDE.md mandate | v0.10.39 | n/a | LOW. |
| grit | referenced via `.grit/` dir | v0.3.0 | n/a | LOW. |
| OpenBao | not yet integrated (`platform-secrets-kernel` exists) | v2.5.3 | n/a | MED — kernel exists but no upstream binary contract yet. |

---

## Drift summary

The worst gaps, in priority order:

1. **Rust drift is closed for this snapshot.** The workspace now pins `rust-toolchain.toml` `channel = "1.95.0"`, `rust-version = "1.95.0"`, `edition = "2024"`, and repo-root rustfmt `edition = "2024"` / `style_edition = "2024"`; cargo-deny 0.19.5 MSRV 1.85 is below the workspace pin.
2. **cargo-deny is still unpinned.** Done-Definition D11 mandates `cargo deny check`; pinning the tool version is now the remaining supply-chain reproducibility gap after the Rust MSRV/edition bump.
3. **Cosign signing is required by D5 (capability publish) but no version is pinned anywhere.** v3.0.6 introduced a breaking change (`--bundle` is now mandatory); CI scripts assuming v2 will fail silently.
4. **Container base-image directive (distroless) is implied but not authoritatively pinned.** debian12 → debian13 transition is in flight upstream (debian12 distroless EOL Sep 2026); without an ADR the org will drift.
5. **Provider-SDK choice is undecided for Rust.** Anthropic and Gemini publish **no official Rust SDK**; OpenAI's `async-openai` is community. The multi-provider Foundry adapter needs an ADR before any kernel pulls in a provider crate (license + supply-chain implications).

---

## Recommended Master Plan §Principles language

> **LTS dependency enforcement.** Every direct runtime, framework, base image, and supply-chain tool that oyatie depends on MUST be pinned to an upstream LTS or designated stable channel, with a verified-as-of date recorded in `.omc/scratch/lts-versions-verified-YYYY-MM-DD.md`. The pin set is reviewed quarterly and on any major upstream LTS announcement.
>
> The workspace `rust-version` MUST equal the current Rust stable channel rounded down to the latest minor that has been live for ≥30 days (currently **1.95.0** → pin `rust-version = "1.95.0"`). The workspace edition and rustfmt parsing/style editions MUST be **2024**. `cargo-deny` MUST be pinned to a version whose MSRV is ≤ the workspace `rust-version` (currently **0.19.5**, MSRV 1.85, compatible with the current workspace pin).
>
> Container base images MUST be `gcr.io/distroless/static-debian13` for static-linked artifacts and `gcr.io/distroless/cc-debian13` for glibc-linked artifacts, with no `latest` tags; digest pinning is REQUIRED at release-pack time.
>
> Cosign signing for capability publish MUST use Cosign **≥ v3.0.6** with the new `--bundle` contract; release pipelines MUST be re-tested against v3 before declaring D5 satisfied.
>
> Provider SDKs: Python and TypeScript adapters MUST use the official Anthropic / OpenAI / Google publications (`anthropic ≥ 0.101.0`, `@anthropic-ai/sdk ≥ 0.95.2`, `openai ≥ 2.36.0`, `openai ≥ 6.36.0` for TS, `google-genai ≥ 2.0.1` for both). Rust adapters MUST NOT depend on an unofficial-vendor crate without an ADR that names the substitute and accepts the supply-chain surface; the default path is an in-tree HTTP client over a vetted TLS stack.
>
> Secrets management MUST converge on **OpenBao ≥ v2.5.3** (MPL-2.0, cargo-deny clean). HashiCorp Vault MUST NOT be introduced (BUSL-1.1 is forbidden under `deny.toml`).
>
> Redis MUST NOT be introduced at versions ≥ 8.0 (RSALv2/SSPLv1/AGPLv3 — forbidden under the cargo-deny allow-list). If a Redis-protocol store is required, the substitute is **Valkey** (BSD-3-Clause).
>
> Prometheus deployments MUST be migrated off **3.5 LTS** before its end-of-support **2026-07-31**.
>
> Trivy MUST be pinned to **≥ v0.70.0**, never v0.69.4 (compromised supply-chain release, 2026-03-19).

---

## Executive summary (250 words)

As of this update, oyatie's Rust LTS drift is closed for the current 1.95.0 stable snapshot: the workspace pins `rust-toolchain.toml` `channel = "1.95.0"`, Cargo `rust-version = "1.95.0"`, all Cargo workspace members report edition 2024, and repo-root `rustfmt.toml` pins both `edition = "2024"` and `style_edition = "2024"`. cargo-deny 0.19.5 (current, 2026-05-09) requires Rust 1.85 MSRV, so it is compatible with the current workspace pin. The remaining supply-chain task is to pin the cargo-deny tool version durably.

Second, **Cosign is required by D5 (capability publish) but unpinned**. The current Cosign v3.0.6 (2026-04-06) introduced a breaking change — `--bundle` is now mandatory — and any CI written for v2 will fail. A pin and a contract update are required before the next signed capability ships.

Third, **the distroless base-image directive needs codification**: debian13 is current, debian12 EOLs in September 2026, and no ADR captures the choice. The Master Plan should adopt `static-debian13` and `cc-debian13` as the canonical bases with digest pinning at release time.

Fourth, **provider-SDK strategy is undecided for Rust**. Anthropic and Google publish no official Rust SDK; only OpenAI has a de-facto community crate (`async-openai` 0.38.1). The multi-provider Foundry adapter needs an ADR before any kernel takes a provider dependency.

Fifth, **Prometheus 3.5 LTS expires 2026-07-31** — under three months out. Any observability work depending on 3.5 needs a migration plan to 3.11+ now.

Sources for every row are cited inline above; the canonical upstream URLs are the authoritative reference.
