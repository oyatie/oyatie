---
doc_class: Reference
shape: anchor
length_cap: 200
authority_tier: 1
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  Evidence base: how AWS, Google/Kubernetes, Microsoft Azure, Oracle/OCI/Java,
  Rust, Stripe, and Twilio version their SDKs, services, release branches, and
  LTS commitments. Anchors the oyatie policy choices in real-world precedent.
planned_enforcement_ref: self
related_adrs: [ADR-0041]
doc_status: published
---

# Versioning Comparison Matrix — oyatie

> **Status:** Accepted. **Owner:** `axis-foundry`. **Date:** 2026-05-12.

## 1. Hyperscaler comparison

| Provider | SDK / library | Service / API | Release branch | Cadence | LTS / support | Deprecation notice |
|---|---|---|---|---|---|---|
| **AWS** | SemVer 2.0.0 per SDK (e.g. `aws-sdk-rust v1.x.y`). Date-anchored "BehaviorVersion" on the SDK. | Per-service date string `YYYY-MM-DD` (e.g. S3 `2006-03-01`, EC2 `2010-04-01`). | Continuous; tags per SDK module | Continuous, near-daily SDK releases | SDK maintenance mode default 12 months | ≥ 12 months per customer agreement (security/IP exceptions) |
| **Google Cloud / Kubernetes** | SemVer 2.0.0 per library; AIP-185 says APIs use `v1` (not `v1.0`). Pre-release suffix `v1beta1`, `v1alpha1`. | Resource-name versioning (`v1`, `v1beta1`, `v1alpha1`) per AIP-180 / AIP-181. Kubernetes uses the same. | Kubernetes: `release-X.Y` cut 2 weeks before X.Y.0 | Kubernetes ~3 minor releases/year, ~1y patch window per minor | Kubernetes: 3 most recent minor releases; ~14 months patch support | GA APIs ≥ 12 months / 3 releases; beta ≥ 9 months / 3 releases |
| **Microsoft Azure** | SemVer 2.0.0 per SDK (BOM monthly for Java since Sep 2021). Beta libraries `X.Y.Z-beta.N`. | Query parameter `api-version=YYYY-MM-DD`; previews `YYYY-MM-DD-preview`. | Beta/feature branches against master | Monthly client-library BOM; per-service GA + preview | .NET LTS = 3 years; STS = 24 months (extended 2025); Azure preview ≥ 90 d notice | Preview ≥ 90 d; stable per-service contractual |
| **Oracle / OCI / Java** | OCI SDK SemVer per `MAJOR.MINOR.PATCH` (Java SDK v3.x.y current; v2 legacy). | OCI: per-service evolution; minor-version dance. | OCI: legacy/v2/master branch separated; current v3 main | Java: 6-month feature releases since Java 10 | Java LTS every 2 years (since Java 17, was 3); Oracle premier 8 years | OCI: 12-month backport of critical bug fixes / security on previous major |
| **Stripe** | SemVer 2.0.0 per library. | Date-based `Stripe-Version: 2024-10-01` header; account pinning. | n/a (no public release branches) | Continuous; one date per breaking-change set | "We never break" (infinite back-compat via version transformers) | n/a (no removal; transformers chain forever) |
| **Twilio** | SemVer 2.0.0 per library. | URI path date version `/2010-04-01/Accounts/...`. | n/a | Continuous | ≥ 12 months notice before deprecating an API version | 12 months |
| **Rust language** | SemVer 2.0.0 (`rustc 1.X.Y`). | Stable, beta, nightly channels. | 6-week train; beta + stable branches | Every 6 weeks, clockwork | Stable channel; no LTS proper | n/a |

## 2. SemVer vs CalVer evidence

| Scheme | Spec | Best fit | Used by |
|---|---|---|---|
| SemVer 2.0.0 | semver.org | Libraries, SDKs, anything with a `pub` API | AWS SDK, Google libraries, Azure SDK, OCI SDK, Rust, Stripe libs, Twilio libs |
| CalVer (`YYYY.MM`) | calver.org | OS distros, continuously-evolving SaaS, internal tooling | Ubuntu 24.04, IntelliJ 2024.2, Black 24.4.0 |
| Hybrid | mixed | Large enterprise-market systems with both axes | .NET (SemVer + November release year), Azure (SemVer SDK + date API), Stripe (SemVer SDK + date API) |

## 3. API versioning patterns

| Pattern | Where | Tradeoffs |
|---|---|---|
| URL path major (`/v1/`) | Google, Kubernetes, Twilio | Cache-friendly, observable; coarse — breaking-change cliff at major |
| URL path date (`/2010-04-01/`) | Twilio v2008/v2010 | Granular; URL churn; hard to roll back partial changes |
| Query param date (`?api-version=YYYY-MM-DD`) | Microsoft Azure | Cache-friendly with vary-by; per-call pinning; default behaviour can drift |
| Header date (`Stripe-Version: ...`) | Stripe | Invisible to URL observability; clean URLs; harder to debug |
| Per-service date suffix on SDK methods | AWS SDK BehaviorVersion | Stable URL surface; pushes versioning into the client; complex matrix |
| Stability suffix (`v1beta1`, `v1alpha1`) | Kubernetes, Google AIP-181 | Communicates promise level; consumes path-version namespace |

## 4. Release-branch naming conventions

| Convention | Used by | Tradeoff |
|---|---|---|
| `release/X.Y` | Atlassian Git Flow extension default | Slash separator clear in GH UI |
| `release-X.Y` | Kubernetes, original Git Flow spec | Dash separator; one path element |
| `vX.Y` | Some Rust crates | Confusable with tag form |
| `stable/X.Y` | Some Debian-style | Strong stability connotation |
| Trunk only (no branch) | Stripe, AWS | Cherry-pick-free; relies on date versioning |

## 5. LTS commitments

| Provider | Major LTS window | Notes |
|---|---|---|
| .NET | 3 years | Even-numbered majors (.NET 6, 8, 10) |
| .NET STS | 24 months (extended 2025) | Odd-numbered majors |
| Oracle Java | 8 years premier + 2 years extended | LTS every 2 years from Java 17 |
| Kubernetes | ~14 months patch support per minor | Top 3 minor releases |
| AWS SDK | 12 months maintenance default | Per-SDK |
| Azure preview | 90 days replacement notice | Preview only |
| Ubuntu LTS | 5 years standard + 5 years ESM | CalVer YY.04 only |

## 6. Cadence patterns

| Provider | Cadence |
|---|---|
| Rust | 6 weeks clockwork |
| Java | 6 months (March, September) |
| .NET | 12 months (every November) |
| Kubernetes | 3 minor releases/year (~14 weeks) |
| Azure SDK Java BOM | Monthly |
| AWS SDK | Continuous (near-daily) |
| Stripe API | Continuous; one date per breaking-change set |

## 7. Oyatie adopted policy

| Axis | Choice | Strongest precedent |
|---|---|---|
| Crate / SDK | SemVer 2.0.0 with workspace lockstep → independent at GA | Rust + AWS SDK + Google libraries |
| Product | SemVer-derived `vX.Y.Z`, MAJOR bump on external break | Kubernetes `v1.30.0` |
| External API | Hybrid: `v1` path + `?api-version=YYYY-MM-DD` query | Google AIP-180 + Microsoft Azure |
| Release branch | `release/X.Y` | Git Flow + Atlassian extension |
| Cadence | Per-axis (continuous for Foundry/Search; bi-weekly elsewhere) | Mix of AWS continuous + Azure monthly BOM |
| LTS | 12 months from major | Median across AWS / Kubernetes / .NET STS |
| Deprecation notice | 180 days | Calibrated between AWS 12mo and Stripe never |

## 8. Why the hybrid wins

Pure SemVer alone is wrong for services (forces a major bump for every
back-compat refinement). Pure CalVer alone is wrong for SDKs (breaks `cargo
update`). Pure date-based alone is wrong for stability tiers (no alpha/beta
signal). The hybrid carries the right signal in each dimension and matches
what the largest hyperscalers actually do.

## 9. Sources

See INDEX.md §Citations for the full URL list (≥ 20 sources across
semver.org, calver.org, AIP-180, AIP-181, AIP-185, Kubernetes deprecation
policy, Azure versioning policy, AWS lifecycle, Oracle JDK, Rust release
channels, Stripe versioning, Twilio versioning, cargo-semver-checks).

## 10. Lift target

`oyatie/docs/release/versioning-comparison-matrix.md` on approval.
