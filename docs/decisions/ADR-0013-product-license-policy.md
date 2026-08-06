---
id: ADR-0013
status: Accepted
doc_status: published
---

# ADR-0013: Product license policy — allowed (Apache-2 / MIT / BSD-2/3 / ISC / 0BSD / MPL-2 / Unicode), forbidden in product code (AGPL / GPL), requires-review tier (LGPL / SSPL / BUSL / Elastic / RSAL / TSL / Confluent / AWS-FSL / Commons Clause), dev-only carve-out, oya-governance-license CI lane, per-release SBOM

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `council-architecture` + `ops-security` + `legal`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0011, ADR-0014, ADR-0019

---

## Context

The cohesion thesis (ADR-0001) and the regional-pack architecture (ADR-0010) both depend on Oyatie shipping product code that customers (including KR sovereign tenants, EU GAIA-X consumers, US FedRAMP-bound buyers, fintech and healthcare verticals) can adopt without inheriting copyleft, source-availability, or commercial-restriction obligations. AGPL/GPL in product code creates downstream redistribution obligations that conflict with Oyatie's customer-facing commercial terms; SSPL/BUSL/Confluent/AWS-FSL/Elastic/RSAL/TSL conflict with cloud-provider deployment of those binaries. LEDG-004 records the prior conflict — the retired license posture accepted AGPL/GPL internal/server-side; this ADR resolves it by formalizing license tiers.

The PRD §6 constraint 8 makes Apache-2 / MIT / BSD / Mozilla-2 the bar; the toolchain manifest (TOOLCHAIN.md §7) lists `cargo-deny` + per-language equivalents as the enforcement. This ADR pins the policy authoritatively and defines the per-tier review process so PRs cannot accidentally introduce a forbidden license.

---

## Decision

We adopt a **three-tier license policy** for product code with an explicit dev-only carve-out, a CI lane that hard-fails forbidden licenses, and a per-release SBOM generation requirement.

### Tier 1 — Allowed in product code (no review required)

| License family | SPDX identifiers |
|---|---|
| Apache 2.0 | `Apache-2.0`, `Apache-2.0 WITH LLVM-exception` |
| MIT | `MIT`, `MIT-0` |
| BSD permissive | `BSD-2-Clause`, `BSD-3-Clause`, `BSD-3-Clause-Clear` |
| ISC | `ISC` |
| Public-domain-equivalent | `0BSD`, `Unlicense`, `CC0-1.0` |
| Mozilla | `MPL-2.0` |
| Unicode | `Unicode-DFS-2016`, `Unicode-3.0` |
| Zlib | `Zlib`, `libpng-2.0` |

### Tier 2 — Forbidden in product code (CI hard-fails)

| License family | SPDX identifiers | Why |
|---|---|---|
| GPL | `GPL-2.0`, `GPL-3.0`, `GPL-2.0-or-later`, `GPL-3.0-or-later` | Copyleft propagation conflicts with customer redistribution rights |
| AGPL | `AGPL-3.0`, `AGPL-3.0-or-later` | Network-use copyleft conflicts with SaaS deployment |
| Commercial-license-only | (no SPDX; per-vendor terms) | License terms conflict with customer redistribution |

### Tier 3 — Requires review (PR cannot merge without `council-architecture` + `legal` sign-off)

| License family | Why |
|---|---|
| LGPL (`LGPL-2.0`, `LGPL-2.1`, `LGPL-3.0`) | Linking restriction; conditionally OK for dynamically linked libs only |
| SSPL (`SSPL-1.0`) | Source-available with strong-copyleft on services; unsafe to redistribute |
| BUSL (`BUSL-1.1`) | Time-bounded source-available; conditional on the convert-to-Apache transition |
| Elastic License (`Elastic-2.0`) | SaaS restriction; conflicts with cloud axis |
| RSAL (`RSAL-1.0`, Redis Source Available) | SaaS restriction |
| TSL (`TSL-2.0`) | SaaS restriction |
| Confluent Community License | SaaS restriction; conflicts with cloud Kafka offering |
| AWS-FSL (`FSL-1.1`) | Functional Source License |
| Commons Clause | Adds non-commercial restriction to permissive license |
| MongoDB SSPL, Sentry FSL, etc. | Per-vendor SaaS restrictions |
| Custom OSS-with-restriction | Per-license review |

### Dev-only carve-out

A Tier 2 or Tier 3 dep MAY ship in `dev-dependencies` (cargo) or `devDependencies` (npm) only when:

- It is invoked at build / test / docs / fixtures only (never linked into a product binary).
- The catalog record under `registry/catalog/<crate>.yaml` declares `dev_only_deps: [<dep-id>: <license>]`.
- `oya-governance-license` validates the dep is not transitively linked into any product target.

Examples typically OK as dev-only: GPL-licensed code generators, AGPL-licensed dev databases (e.g. MongoDB Community in test fixtures), BUSL test harnesses.

### Per-language enforcement

| Language | Tool | Config |
|---|---|---|
| Rust | `cargo deny` | `deny.toml` enforces allow + deny lists |
| TypeScript | `license-checker` + custom validator | per-package |
| Python | `pip-licenses` + custom validator | per-package |
| Go | `go-licenses` | per-module |
| WASM plugins | `oya-intelligence-marketplace` license check at upload time | per-plugin |

### CI lane: `oya-governance-license`

The lane runs on every PR touching `Cargo.lock`, `pnpm-lock.yaml`, `requirements.txt`, `go.sum`, or any catalog record. It:

1. Resolves the full transitive dep tree.
2. Maps every dep to an SPDX identifier (failing on unidentifiable licenses).
3. Hard-fails any Tier 2 license in product code.
4. Emits a `requires-review` label for any Tier 3 license; merge blocked until review sign-off in PR `## Code Review` block.
5. Validates per-microservice allow-list (some axes may further restrict; e.g. defense pack may forbid even Tier 3 LGPL).
6. Writes the dep graph + license map into the per-PR build artifact for SBOM generation.

Cloud-ci replacement surface (ADR-0515 migration of this policy into the single `oya-ci-required` context):

- `ci/facade/license-policy/BUCK`
- `ci/facade/license-policy/Cargo.toml`
- `ci/facade/license-policy/src/lib.rs`
- `ci/facade/license-policy/tests/license_policy.rs`

### Per-release SBOM

Per-release tag, the build pipeline:

1. Generates an SBOM in CycloneDX 1.5 format covering every shipped binary + plugin.
2. Cosign-signs the SBOM and Rekor-anchors it.
3. Publishes the SBOM to the trust portal (per ADR-0003 audit-chain integration).
4. Emits `EVT-RELEASE-SBOM-PUBLISHED` to the chain.

### Boundary

- Applies to: every crate under `crates/oya-*`, every npm package under web/UI subtrees, every Python package, every Go module, every WASM plugin published to the marketplace.
- Does not apply to: third-party SaaS Oyatie consumes operationally (e.g. GitHub itself, Slack); per-tenant customer-uploaded artifacts (which carry the customer's license terms).
- Repository-default license posture: root [`/LICENSE`](../../LICENSE) is proprietary and all rights reserved. This ADR governs dependency acceptance, release SBOM posture, and package/component metadata; it does not grant repository-wide Apache-2.0 or any other open-source rights. Explicit file-level/component-level notices remain scoped exceptions for the identified material only.

---

## Consequences

### Positive

- License risk becomes a CI question, not a manual audit.
- Closes LEDG-004 (license posture conflict with the retired license posture) at the policy level.
- Per-release SBOM aligns with US Executive Order 14028, EU CRA, and KR sovereignty buyers' expectations.
- Tier 3 review is a controlled escape valve — not all source-available licenses are equal, and per-PR review preserves judgment.

### Negative

- Some best-of-breed deps (Redis 7.4+, MongoDB, Elasticsearch ≥ 7.11, Confluent stack, Terraform ≥ 1.6) become Tier 3 or unavailable; alternative-selection cost is real (see ADR-0014).
- License-tier debates can become contentious; mitigated by clear SPDX-mapped tier table.
- SBOM generation adds 30–60 s per release; acceptable.

### Operational

- On-call: `EVT-LICENSE-LANE-DENY` alerted to ops-security; `EVT-RELEASE-SBOM-PUBLISHED` confirms compliance posture per release.
- Runbooks: `runbooks/license-tier-3-review.md`, `runbooks/sbom-regenerate.md`, `runbooks/forbidden-license-rollback.md`.
- CI: `oya-governance-license` is a P0 lane.
- Per-pack overrides: a pack MAY add stricter allow-lists (defense pack forbids LGPL); never weaker.

---

## Alternatives considered

### Alternative A — Allow all OSI-approved licenses

- **Pros:** widest dep choice.
- **Cons:** AGPL/GPL conflict with downstream redistribution; LEDG-004.
- **Rejected because:** customer commercial terms.

### Alternative B — Forbid only AGPL; allow GPL with linking carve-out

- **Pros:** more deps available.
- **Cons:** GPL linking is a perpetual minefield in static-link Rust; library boundaries blur in practice.
- **Rejected because:** operational risk.

### Alternative C — Per-axis license policy (each axis decides its own)

- **Pros:** axis autonomy.
- **Cons:** cohesion violation; cross-microservice consumer of an axis with looser policy inherits the obligation.
- **Rejected because:** ADR-0001.

### Alternative D — Allow Tier 3 freely without review

- **Pros:** fewer review bottlenecks.
- **Cons:** SaaS-restriction licenses (SSPL / BUSL / RSAL / Elastic) silently land in product, and the resulting redistribution conflict surfaces only at customer audit.
- **Rejected because:** failure mode is regulator-visible and unrecoverable per release.

---

## Open questions

1. **Q1.** Does the BUSL-1.1 conditional convert-to-Apache trigger affect a forward compatibility plan? Default: opt-in per-dep; track conversion date in vendor-partner-ledger. → owner: `legal`.
2. **Q2.** Per-region pack license overrides — does KR-pack inherit from this ADR or re-declare? Default: inherit; packs may further restrict. → ADR-0010.
3. **Q3.** Vendor-partner-ledger (`docs/VENDOR-PARTNER-LEDGER.md`) — auto-populated from `cargo deny` output, or hand-curated? Default: auto with hand annotations. → owner: `ops-security`.
4. **Q4.** Customer-uploaded WASM plugin licensing — Marketplace gate on upload? Default: yes; MUST be Tier 1 or open Tier 3 review. → ADR-0014.
5. **Q5.** Generated-code license (e.g. SDK codegen output) — does it inherit the registry's spec license? Default: SDKs ship Apache-2 by default. → ADR-0011.

---

## References

- `docs/PRD.md` §6 constraint 8 (license posture: Apache-2 / MIT / BSD / Mozilla-2 allowed; AGPL forbidden in product code; GPL forbidden in product code; SSPL and BUSL require ADR review)
- `docs/TOOLCHAIN.md` §7 (license manifest), §3 (per-stack license calls — Apache-2 broker, Apache-2 mesh, Apache-2 supply-chain stack)
- `docs/CONTRADICTION-LEDGER.md` LEDG-004 (license posture conflict)
- ADR-0001 (cohesion), ADR-0011 (per-spec generated-SDK license), ADR-0014 (build-vs-buy with license tier as input), ADR-0019 (per-release SBOM cadence)
- SPDX License List (https://spdx.org/licenses/)
- US Executive Order 14028 (SBOM); EU Cyber Resilience Act
