---
purpose: "Cross-cutting dependency policy. Defines LTS pinning (per the verified roster), license posture (no AGPL / GPL / SSPL / BUSL / RSAL in product code), `cargo-vet` + `cargo-deny` enforcement, and the owned `deps.toml` dependency-automation baseline."
doc_status: published
---

---
doc_class: Standard
shape: ~
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Cross-cutting dependency policy. Defines LTS pinning (per the verified roster),
  license posture (no AGPL / GPL / SSPL / BUSL / RSAL in product code),
  `cargo-vet` + `cargo-deny` enforcement, the owned `deps.toml` dependency-automation baseline,
  and the provider-SDK strategy: Anthropic / OpenAI / Gemini SDKs sit behind a
  `ProviderAdapter` trait so the workspace remains provider-agnostic per
  MASTERPLAN Directive 4.
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json
planned_enforcement_ref: governance-lts-dependency
companion_docs:
  - docs/standards/security-review.md
  - docs/standards/code-style-rust.md
  - docs/standards/image-discipline.md
  - .omc/scratch/lts-versions-verified-2026-05-12.md
related_adrs:
  - ADR-0053
  - ADR-0052
  - ADR-0054
---

# Dependency Policy

## Doctrinal authority — [decision-principles.json](../../specs/decision-principles.json) + [forbidden-operations.json](../../specs/forbidden-operations.json)

Every direct runtime, framework, base image, and supply-chain tool the
workspace depends on MUST be pinned, license-clean, and reviewed via the
supply-chain triad. This standard codifies the policy; the program-level
inventory lives in `.omc/scratch/lts-versions-verified-YYYY-MM-DD.md`.

## 1. LTS pinning

Per [`.omc/scratch/lts-versions-verified-2026-05-12.md`](../../.omc/scratch/lts-versions-verified-2026-05-12.md)
and MASTERPLAN §2 Directive 8:

- Every direct dependency tracks the **current LTS** major.minor where the
  project publishes an LTS line.
- Projects without a formal LTS (SQLite, Cosign, OpenTelemetry, Envoy,
  upstream K8s) track the **current stable** + a designated LTS-substitute
  pin (e.g., Canonical 1.32 LTS for K8s).
- The LTS roster is refreshed **quarterly** and on any major upstream LTS
  announcement; the verified-as-of date is recorded in
  `.omc/scratch/lts-versions-verified-YYYY-MM-DD.md`.
- Lane: `governance-lts-dependency` checks every direct
  dependency against the roster on every PR.

### 1.1 Current floor (2026-05-12)

Per the verified-LTS spec:

| Component | Pin (≥) | Component | Pin (≥) |
|---|---|---|---|
| Rust toolchain | 1.97.1 stable | Debian / distroless base | trixie / static-debian13 |
| Rust edition / rustfmt style | 2024 | OpenSSL | 3.5 LTS or 4.0 |
| Node.js | 24 Active LTS (or 22) | Prometheus | 3.11+ (3.5 EOS 2026-07-31) |
| Python | 3.14 (or 3.13 maint.) | Cosign | v3.0.6 |
| Go | 1.26 | Trivy | v0.70.0 (NOT v0.69.4) |
| PostgreSQL | 18 (5-yr/major) | cargo-deny | 0.19.5 (MSRV 1.85) |
| ClickHouse LTS | 26.3 | OpenBao | v2.5.3 (Vault forbidden) |
| Kubernetes / containerd | 1.36 (or Canonical 1.32) / 2.3 LTS | | |

## 2. License posture

Per [`forbidden-operations.json`](../../specs/forbidden-operations.json) FO-09, the
following licenses MUST NOT appear in product-code dependencies:

- **AGPL** (Affero GPL).
- **GPL** (any version).
- **SSPL** (Server Side Public License).
- **BUSL** (Business Source License — including HashiCorp Vault).
- **RSAL** (Redis Source Available License).

Permitted licenses (per `deny.toml`):

- **0BSD**, **Apache-2.0**, **BSD-2-Clause**, **BSD-3-Clause**, **ISC**,
  **MIT**, **MPL-2.0**, **Unicode-3.0**.

### 2.1 Concrete forbidden-substitute table

| Forbidden | Why | Substitute |
|---|---|---|
| Redis ≥ 7.4 (Redis Inc. relicense 2024-03-20) | RSALv2 / SSPLv1 / AGPLv3 tri-license | **Valkey** (BSD-3-Clause — Linux Foundation fork, canonical per ADR-0336); pre-7.4 Redis (BSD-3-Clause) is license-clean fallback but non-canonical (absent upstream maintenance + absent hyperscaler-managed offering). DragonflyDB is NOT a permitted substitute (BSL-1.1 is on the forbidden-license list, see §2). |
| HashiCorp Vault | BUSL-1.1 | **OpenBao** (MPL-2.0) |
| MongoDB (server) | SSPLv1 | **PostgreSQL** + JSONB or **ClickHouse** |
| Elasticsearch ≥ 7.11 | SSPLv1 / Elastic License v2 | **OpenSearch** (Apache-2.0) or **ClickHouse** |
| `gnu-time` (in containers) | GPLv3 | `time` builtin / busybox `time` |

Lane: `governance-license` (`cargo-deny check licenses`) refuses
any forbidden license on every PR.

## 3. Supply-chain triad

Per [`security-review.md`](security-review.md) §2:

| Tool | Scope | Lane |
|---|---|---|
| `cargo-audit` | RustSec advisory DB | `governance-cargo-audit` |
| `cargo-deny` | license + advisory + source + duplicate | `governance-license` |
| `cargo-vet` | human-audit trail | `governance-cargo-vet` |

Pinning rules:

- `cargo-deny` MUST be at a version with MSRV ≤ workspace
  `rust-version`. Current target: cargo-deny **0.19.5** (MSRV 1.85),
  compatible with the current Rust 1.97.1 workspace pin (per §1.1).
- `cargo-vet` audits live under `supply-chain/audits.toml`; share-points
  imported from AWS and Mozilla published audits.

## 4. Owned dependency-automation baseline

ADR-0535 supersedes the earlier external-bot baseline: Oyatie uses a
closed-schema root [`deps.toml`](../../deps.toml) as DATA for an
in-house Rust bump-bot. The bot opens provider-neutral scm-facts ChangeSets,
runs license/advisory/version gates before proposing updates, and reaches merge
only through the single `presubmit` context. GitHub Actions is a
transitional runner adapter; GitHub PRs are an adapter surface, not the
canonical automation substrate.

Baseline invariants:

- `automation.engine = "owned-rust-bump-bot"`.
- `automation.changeset_transport = "scm-facts"`.
- `automation.external_bots = "disabled"`.
- `rust.update_policy = "latest-stable"` and `rust.pin` stays synchronized with
  `rust-toolchain.toml`, root workspace `rust-version`, container builder pins,
  and Buck2 toolchain notes.
- `supply_chain` points at `deny.toml`, the cargo-deny/advisory lane, cargo-vet,
  and the OSS stewardship registry.

Lane: `cloud-ci-dependency-automation` validates `deps.toml`, rejects
external bot configs, and catches Rust pin split-brain.

Source: ADR-0535 and root `deps.toml`.

## 5. Provider-SDK strategy — `ProviderAdapter`

Per MASTERPLAN Directive 4 (Provider-agnostic by default), every
provider-specific dependency lives in an `oya-*-adapter-<provider>-*`
crate. The `app` and `domain` layers depend only on a trait abstraction.

### 5.1 The `ProviderAdapter` shape

```rust
#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    type Request;
    type Response;
    type Error: std::error::Error + Send + Sync + 'static;

    async fn invoke(&self, req: Self::Request) -> Result<Self::Response, Self::Error>;
    fn provider_name(&self) -> &'static str;
    fn provider_version(&self) -> &'static str;
}
```

Concrete implementations:

- `intelligence-adapter-anthropic` — Anthropic claude API (no official Rust
  SDK; in-tree HTTP client over reqwest+rustls).
- `intelligence-adapter-openai` — OpenAI API (community crate
  `async-openai 0.38.1`; cargo-vet certified).
- `intelligence-adapter-gemini` — Google Gemini (no official Rust SDK;
  in-tree HTTP client).
- `cloud-adapter-aws`, `-gcp`, `-azure`, `-oci` — cloud SDK adapters.
- `platform-adapter-secrets-openbao` — secrets adapter.

### 5.2 Provider-SDK pinning

Per the verified-LTS spec:

| Provider | Python | TypeScript | Rust |
|---|---|---|---|
| Anthropic | `anthropic ≥ 0.101.0` | `@anthropic-ai/sdk ≥ 0.95.2` | **in-tree HTTP client** (no official Rust SDK) |
| OpenAI | `openai ≥ 2.36.0` | `openai ≥ 6.36.0` | `async-openai ≥ 0.38.1` (cargo-vet certified) |
| Gemini | `google-genai ≥ 2.0.1` | `@google/genai ≥ 2.0.1` | **in-tree HTTP client** (no official Rust SDK) |

The in-tree HTTP client uses `reqwest` + `rustls` + hand-rolled types
generated from each provider's published OpenAPI schema where available.

### 5.3 Provider-coupling lane

Lane `governance-provider-coupling` refuses provider-specific
imports outside `oya-*-adapter-<provider>-*` crates. The `app` and
`domain` layers see only the `ProviderAdapter` trait.

## 6. Secret-provider strategy

- Primary: **OpenBao ≥ v2.5.3** (MPL-2.0). The `SecretProvider` trait
  fronts every secret access.
- Cloud-native secret stores (AWS Secrets Manager, Google Secret
  Manager, Azure Key Vault) are **injection-only** adapters that pull
  from OpenBao at deploy time.
- HashiCorp Vault is **forbidden** (BUSL-1.1).

## 7. Database / data-store strategy

| Need | Pick | Forbidden alternative |
|---|---|---|
| Relational | PostgreSQL 18 | MySQL (license-clean but not in posture) |
| OLAP table format | **Apache Iceberg 1.7+** (Apache-2.0; canonical per ADR-0337; hyperscaler-managed via AWS S3 Tables + Snowflake Polaris + Google BigLake + Databricks Unity Catalog Iceberg REST + Azure Synapse Lake managed Iceberg) | Apache Delta Lake (adapter-only per ADR-0337); Apache Hudi (adapter-only per ADR-0337); ClickHouse-native MergeTree as tenant-visible OLAP write path (ADR-0337 §D-4) |
| OLAP compute engine | ClickHouse 26.3 LTS layered on Iceberg via the ClickHouse iceberg engine (production-ready since 24.8 LTS; per ADR-0337) | ClickHouse-native MergeTree as tenant-visible OLAP table format (permitted only for ClickHouse-internal projections / dictionaries / materialized views) |
| Search / log | OpenSearch (Apache-2.0) | Elasticsearch ≥ 7.11 (SSPLv1) |
| In-memory cache / pubsub / streams | **Valkey** 8.x (canonical per ADR-0336; hyperscaler-managed via AWS ElastiCache for Valkey + Google Memorystore for Valkey + Oracle Cloud Cache with Valkey + OCI Always Free) | Redis ≥ 7.4 (RSALv2/SSPLv1); DragonflyDB (BSL-1.1) |
| Document | PostgreSQL JSONB | MongoDB (SSPLv1) |
| Embedded | SQLite 3.53+ | n/a |

## 8. CI/CD platform

Per `.omc/scratch/hyperscaler-best-practices-2026-05-12.md` Domain 4: the
default platform is **GitHub Actions**. Self-hosted runners under the
Buildkite control plane are the cloud-portable analog for high-volume.
Bazel / Buck2 are **not adopted** at current scale (Cargo workspace +
`cargo-make`/`just` is sufficient).

## 9. Anti-patterns

1. **Pinning a provider SDK in `app` or `domain`.** Move to an adapter.
2. **Adding a crate that fails `cargo-deny licenses`.** Use the
   substitute table; if none fits, file an ADR-tracked extension
   (named license addition with sunset rationale).
3. **Skipping `cargo-vet` certification for a new crate.**
4. **Pinning to a non-LTS line when an LTS exists** (e.g., Node.js
   Current channel for production).
5. **Adopting Vault, Redis ≥ 8, MongoDB, or Elasticsearch ≥ 7.11.** All
   forbidden per §2.1.
6. **`latest` tags** anywhere. See [`image-discipline.md`](image-discipline.md).

## 10. Sources scanned

- [`.omc/scratch/lts-versions-verified-2026-05-12.md`](../../.omc/scratch/lts-versions-verified-2026-05-12.md);
  [`.omc/scratch/hyperscaler-best-practices-2026-05-12.md`](../../.omc/scratch/hyperscaler-best-practices-2026-05-12.md)
  Domain 3 + 4.
- [Mozilla — cargo-vet](https://mozilla.github.io/cargo-vet/);
  [cargo-deny](https://embarkstudios.github.io/cargo-deny/);
  [ADR-0535](../decisions/ADR-0535-cross-product-versioning-release-governance.md).
- [OpenBao](https://openbao.org/), [Valkey](https://valkey.io/),
  [GitLab Handbook — ADR 007 OpenBao](https://handbook.gitlab.com/handbook/engineering/architecture/design-documents/secret_manager/decisions/007_openbao/).

## 11. OSS stewardship registry — canonical aggregate per ADR-0345

Per [ADR-0345](../decisions/ADR-0345-oss-stewardship-class-policy-and-cve-response-sla.md),
every direct upstream OSS dependency Oyatie consumes — every crate listed in
§1 (LTS pinning), every license substitute named in §2.1, every supply-chain
tool in §3, every owned dependency-automation surface in §4, every provider SDK in §5,
every secret-provider substrate in §6, every data-store substrate in §7, every
CI/CD-platform dependency in §8 — MUST be classified into one of three
**OSS stewardship classes** at the canonical registry path:

- **`/specs/oss-stewardship-registry.json`** — canonical machine-readable
  registry, owned jointly by council-architecture + council-security +
  council-legal + ops-supply-chain + axis-compliance + ops-platform.

### 11.1 The three stewardship classes

| Class | Definition | CVE-response SLA | Resourcing field |
|---|---|---|---|
| **Maintainer** | Oyatie has commit + release authority on the upstream | substrate crates: P0 ≤ 3 days + P1 ≤ 14 days; utility crates: P0 ≤ 7 days + P1 ≤ 30 days (Oyatie authors the patch) | `maintainer_engineering_time_percent` (per crate; 0..100 percent of one FTE) |
| **Contributor** | Oyatie actively patches upstream + commits staff time | **P0 ≤ 7 days; P1 ≤ 30 days** (wall-clock from public CVE disclosure) | `contribution_budget_dev_days_per_quarter` (integer dev-days/quarter) |
| **Consumer** | Oyatie pins + audits without upstream contribution | **pin update ≤ 14 days** of public CVE disclosure (P0 upstream-monitored) | `audit_subscription_cost_usd` (integer USD/year) |

### 11.2 Floor enumeration (per ADR-0345 §D-3 / §D-4 / §D-5)

**Maintainer-class floor:** every `oya-*` crate (~200+); `shuffle-sharding`
(ADR-0333 substrate); `dev-cli` (ADR-0218); `shared-policy-engine-client`
(Cedar wrapper); `shared-workflow-engine`; `shared-ontology-projection`;
internal hooks + tools under `tools/` + `bin/`.

**Contributor-class floor (11 upstreams):** Cilium (ADR-0148, CNI/ClusterMesh);
Istio (ADR-0044, Ambient Mesh); Valkey (ADR-0336, in-memory KV substitute for
Redis); OpenTofu (ADR-0218, IaC substitute for HashiCorp Terraform); Cedar
(ADR-0243, universal policy engine); OpenBao (Vault substitute, MPL-2.0);
Wasmtime (ADR-0200, Wasm sandbox); Apache Iceberg (ADR-0337, OLAP table format);
Apache Kafka (ADR-0050, async message fabric); OpenSearch (Elasticsearch
substitute, Apache-2.0); Kyverno (ADR-0183, K8s admission gate).

**Consumer-class floor (~15-20 upstreams):** PostgreSQL (mature substrate;
massive community); ClickHouse (compute engine layered on Iceberg per ADR-0337);
Linux kernel mainline; RHEL / Oracle Linux / SLES / Ubuntu LTS / Debian /
Rocky / AlmaLinux / CentOS Stream / Amazon Linux / Flatcar / Photon OS / Talos
(OS support matrix per `feedback_os_support_matrix_2026_05_20`); `static-debian13`
distroless base (per `image-discipline.md`); OpenTelemetry (CNCF graduated);
Rust toolchain; cargo-deny; cargo-vet; cosign (Sigstore).

### 11.3 Terminology binding (normative)

OSS stewardship uses **class** (a relationship label), NOT **tier**. The word
"tier" is RESERVED in Oyatie corpus terminology for two and only two uses:

- **ADR-0248 cellular tiers** — Tier 0..4 (Foundation / Substrate / Capability /
  Application / Edge).
- **ADR-0338 pod runtime tiers** — Tier 0..3 (tenant-customer untrusted / substrate
  tenant-data-plane / first-party / edge perf-critical).

The lane **`governance-stewardship-class-vocabulary`** refuses corpus drift
toward the word "tier" in OSS-stewardship contexts. Enforced day-1 from ADR-0345
Acceptance; no grace window.

### 11.4 Enforcement lanes (per ADR-0345 §E)

- `check-oss-stewardship-registry-presence` — refuses corpus changes
  adding a new direct upstream (in `Cargo.toml`, OpenTofu providers, Helm
  charts, Dockerfile `FROM`) without a corresponding registry entry.
- `check-oss-stewardship-class-declaration` — refuses registry entries
  missing `stewardship_class ∈ {maintainer, contributor, consumer}`.
- `check-oss-stewardship-cve-sla` — refuses Contributor entries missing
  P0 ≤ 7-day + P1 ≤ 30-day fields; refuses Consumer entries missing pin-update
  ≤ 14-day field.
- `check-oss-stewardship-owner-team` — refuses entries missing `owner_team`
  value drawn from the council / axis / ops taxonomy.
- `check-oss-stewardship-resourcing-declaration` — refuses Maintainer
  entries missing `maintainer_engineering_time_percent`; Contributor entries
  missing `contribution_budget_dev_days_per_quarter`; Consumer entries missing
  `audit_subscription_cost_usd`.
- `check-oss-stewardship-license-cross-check` — refuses entries whose
  `license` value contradicts the forbidden-license list in §2 above +
  `specs/forbidden-operations.json` FO-09.
- `governance-stewardship-class-vocabulary` — refuses "tier" applied to
  OSS stewardship (day-1 BLOCKER).

### 11.5 SOC2 + ISO 27001 vendor-risk-management binding

The registry is the canonical evidence surface for:

- SOC2 Trust Services Criteria **CC2.3** (third-party risk) and **CC9.2**
  (vendor management).
- ISO 27001:2022 Annex **A.5.19** (information security in supplier
  relationships), **A.5.20** (addressing information security within supplier
  agreements), **A.5.21** (managing information security in the ICT supply
  chain), **A.5.22** (monitoring, review and change management of supplier
  services).

Auditors trace any OSS dependency through the registry to its stewardship
class + CVE SLA + owner team + license + ADR provenance. The audit-evidence
path is: **registry → SBOM → cosign attestation (per ADR-0181) → per-crate
manifest**. Per-pack compliance evidence (HIPAA / GDPR / SOC2 / PCI / CSAP /
EU AI Act Annex III per [ADR-0251](../decisions/ADR-0251-compliance-pack-cell-certification-levels.md))
layers on top of the registry for pack-specific vendor-risk-management
constraints.

### 11.6 Hyperscaler precedent

Hyperscaler-grade OSS posture declares stewardship explicitly per upstream:
AWS Open Source Engineering, Google OSPO annual report, Microsoft
opensource.microsoft.com, Meta opensource.fb.com, Netflix netflix.github.io,
Apple opensource.apple.com. Oyatie's registry is the corpus-wide analog.

### 11.7 Future substrate-adoption contract

Every NEW substrate-adoption ADR (e.g., a future substrate-adoption ADR that selects
a new upstream) MUST add a registry entry as part of the ADR's required
artifact. The `check-oss-stewardship-registry-presence` lane refuses
substrate adoption that skips registry declaration. Council-architecture +
council-security + ops-supply-chain joint approval is required for the dev-days
or audit-subscription budget.

Source: [ADR-0345](../decisions/ADR-0345-oss-stewardship-class-policy-and-cve-response-sla.md);
canonical registry at [`/specs/oss-stewardship-registry.json`](../../specs/oss-stewardship-registry.json).
