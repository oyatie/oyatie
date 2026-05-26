---
doc_class: Standard
shape: ~
length_cap: 250
authority_tier: 2
status: pending approval
purpose: |
  Cross-cutting dependency policy. Defines LTS pinning (per the verified roster),
  license posture (no AGPL / GPL / SSPL / BUSL / RSAL in product code),
  `cargo-vet` + `cargo-deny` enforcement, the Renovate configuration baseline,
  and the provider-SDK strategy: Anthropic / OpenAI / Gemini SDKs sit behind a
  `ProviderAdapter` trait so the workspace remains provider-agnostic per
  MASTERPLAN Directive 4.
lift_target: oyatie/docs/standards/dependency-policy.md
canonical_authority: docs/CONSTITUTION.md
enforced_by: oya-governance-lts-dependency
companion_docs:
  - docs/standards/security-review.md
  - docs/standards/code-style-rust.md
  - docs/standards/image-discipline.md
  - .omc/scratch/lts-versions-verified-2026-05-12.md
---

# Dependency Policy

## Constitutional authority — [CONSTITUTION.md](../CONSTITUTION.md)

Every direct runtime, framework, base image, and supply-chain tool the
workspace depends on MUST be pinned, license-clean, and reviewed via the
supply-chain triad. This standard codifies the policy; the program-level
inventory lives in `.omc/scratch/lts-versions-verified-YYYY-MM-DD.md`.

## 1. LTS pinning

Per [`.omc/scratch/lts-versions-verified-2026-05-12.md`](../specs/lts-versions-verified-2026-05-12.md)
and MASTERPLAN §2 Directive 8:

- Every direct dependency tracks the **current LTS** major.minor where the
  project publishes an LTS line.
- Projects without a formal LTS (SQLite, Cosign, OpenTelemetry, Envoy,
  upstream K8s) track the **current stable** + a designated LTS-substitute
  pin (e.g., Canonical 1.32 LTS for K8s).
- The LTS roster is refreshed **quarterly** and on any major upstream LTS
  announcement; the verified-as-of date is recorded in
  `.omc/scratch/lts-versions-verified-YYYY-MM-DD.md`.
- Lane: `oya-governance-lts-dependency` checks every direct
  dependency against the roster on every PR.

### 1.1 Current floor (2026-05-12)

Per the verified-LTS spec:

| Component | Pin (≥) | Component | Pin (≥) |
|---|---|---|---|
| Rust toolchain | 1.95.0 stable | Debian / distroless base | trixie / static-debian13 |
| Rust edition / rustfmt style | 2024 | OpenSSL | 3.5 LTS or 4.0 |
| Node.js | 24 Active LTS (or 22) | Prometheus | 3.11+ (3.5 EOS 2026-07-31) |
| Python | 3.14 (or 3.13 maint.) | Cosign | v3.0.6 |
| Go | 1.26 | Trivy | v0.70.0 (NOT v0.69.4) |
| PostgreSQL | 18 (5-yr/major) | cargo-deny | 0.19.5 (MSRV 1.85) |
| ClickHouse LTS | 26.3 | OpenBao | v2.5.3 (Vault forbidden) |
| Kubernetes / containerd | 1.36 (or Canonical 1.32) / 2.3 LTS | | |

## 2. License posture

Per [`CONSTITUTION.md`](../CONSTITUTION.md) §Prohibitions Item 9, the
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
| Redis ≥ 8.0 | RSALv2 / SSPLv1 / AGPLv3 tri-license | **Valkey** (BSD-3-Clause) or pre-7.4 Redis (BSD-3-Clause) |
| HashiCorp Vault | BUSL-1.1 | **OpenBao** (MPL-2.0) |
| MongoDB (server) | SSPLv1 | **PostgreSQL** + JSONB or **ClickHouse** |
| Elasticsearch ≥ 7.11 | SSPLv1 / Elastic License v2 | **OpenSearch** (Apache-2.0) or **ClickHouse** |
| `gnu-time` (in containers) | GPLv3 | `time` builtin / busybox `time` |

Lane: `oya-governance-license` (`cargo-deny check licenses`) refuses
any forbidden license on every PR.

## 3. Supply-chain triad

Per [`security-review.md`](security-review.md) §2:

| Tool | Scope | Lane |
|---|---|---|
| `cargo-audit` | RustSec advisory DB | `oya-governance-cargo-audit` |
| `cargo-deny` | license + advisory + source + duplicate | `oya-governance-license` |
| `cargo-vet` | human-audit trail | `oya-governance-cargo-vet` |

Pinning rules:

- `cargo-deny` MUST be at a version with MSRV ≤ workspace
  `rust-version`. Current target: cargo-deny **0.19.5** (MSRV 1.85),
  compatible with the current Rust 1.95.0 workspace pin (per §1.1).
- `cargo-vet` audits live under `supply-chain/audits.toml`; share-points
  imported from AWS and Mozilla published audits.

## 4. Renovate baseline

Per [`.omc/scratch/hyperscaler-best-practices-2026-05-12.md`](../specs/hyperscaler-best-practices-2026-05-12.md)
Domain 4: **Renovate** is the canonical dependency-update bot
(supports 30+ ecosystems vs Dependabot's 14). Dependabot remains
enabled for security-advisory fan-in only.

Baseline `renovate.json`:

```json
{
  "$schema": "https://docs.renovatebot.com/renovate-schema.json",
  "extends": ["config:base", ":semanticCommits"],
  "schedule": ["before 06:00 on Monday"],
  "labels": ["deps", "renovate"],
  "rangeStrategy": "bump",
  "packageRules": [
    { "matchUpdateTypes": ["minor", "patch"], "matchCurrentVersion": "!/^0/",
      "automerge": true, "platformAutomerge": true },
    { "matchUpdateTypes": ["major"], "automerge": false },
    { "matchPackagePrefixes": ["openssl", "rustls", "ring"], "labels": ["deps","security"] }
  ],
  "vulnerabilityAlerts": { "labels": ["security"], "automerge": false }
}
```

Lane: `oya-governance-renovate-config` validates the file is
present and grouped.

Source: [Renovate docs](https://docs.renovatebot.com/).

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

- `oya-foundry-adapter-anthropic` — Anthropic claude API (no official Rust
  SDK; in-tree HTTP client over reqwest+rustls).
- `oya-foundry-adapter-openai` — OpenAI API (community crate
  `async-openai 0.38.1`; cargo-vet certified).
- `oya-foundry-adapter-gemini` — Google Gemini (no official Rust SDK;
  in-tree HTTP client).
- `oya-cloud-adapter-aws`, `-gcp`, `-azure`, `-oci` — cloud SDK adapters.
- `oya-platform-adapter-secrets-openbao` — secrets adapter.

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

Lane `oya-governance-provider-coupling` refuses provider-specific
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
| OLAP | ClickHouse 26.3 LTS | (none currently disallowed) |
| Search / log | OpenSearch (Apache-2.0) | Elasticsearch ≥ 7.11 (SSPLv1) |
| In-memory cache | Valkey | Redis ≥ 8.0 (RSALv2) |
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
   substitute table; if none fits, file an ADR exemption.
3. **Skipping `cargo-vet` certification for a new crate.**
4. **Pinning to a non-LTS line when an LTS exists** (e.g., Node.js
   Current channel for production).
5. **Adopting Vault, Redis ≥ 8, MongoDB, or Elasticsearch ≥ 7.11.** All
   forbidden per §2.1.
6. **`latest` tags** anywhere. See [`image-discipline.md`](image-discipline.md).

## 10. Sources scanned

- [`.omc/scratch/lts-versions-verified-2026-05-12.md`](../specs/lts-versions-verified-2026-05-12.md);
  [`.omc/scratch/hyperscaler-best-practices-2026-05-12.md`](../specs/hyperscaler-best-practices-2026-05-12.md)
  Domain 3 + 4.
- [Mozilla — cargo-vet](https://mozilla.github.io/cargo-vet/);
  [cargo-deny](https://embarkstudios.github.io/cargo-deny/);
  [Renovate](https://docs.renovatebot.com/).
- [OpenBao](https://openbao.org/), [Valkey](https://valkey.io/),
  [GitLab Handbook — ADR 007 OpenBao](https://handbook.gitlab.com/handbook/engineering/architecture/design-documents/secret_manager/decisions/007_openbao/).
