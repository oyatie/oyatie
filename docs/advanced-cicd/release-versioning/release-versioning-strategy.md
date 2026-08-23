---
doc_class: Strategy
shape: anchor
length_cap: 200
authority_tier: 1
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  Define the end-state release-versioning policy for oyatie across crates, products,
  external APIs, release branches, cadence, LTS, deprecation, and pre-release labels.
  Adopt the strongest pattern observed across hyperscalers (AWS, Google/Kubernetes,
  Microsoft Azure, Oracle/Java) so oyatie users, partners, and operators get a
  contract that matches industry expectation.
planned_enforcement_ref: governance-semver-discipline, governance-api-version-stability, governance-release-branch-cut, governance-version-eol-warning, governance-deprecation-notice, governance-cherry-pick-trail
related_adrs: [ADR-0040, ADR-0041, ADR-0050]
doc_status: published
---

# Release Versioning Strategy — oyatie

> **Status:** Accepted. **Owner:** `axis-foundry`. **Date:** 2026-05-12.

## 1. Why this policy

Every hyperscaler we benchmarked separates **library versioning** (semantic) from
**service / API versioning** (date-based or stability-suffixed). The library track
exists to let consumers reason about compile-time / link-time breakage; the API
track exists to let services evolve without forcing in-place migrations. Mixing
these axes is the most common versioning failure mode in industry; therefore
oyatie keeps them strictly separate.

See [`versioning-comparison-matrix.md`](versioning-comparison-matrix.md) for the
full evidence base (AWS / Google / Kubernetes / Microsoft / Oracle / Stripe /
Twilio / Rust / Java).

## 2. The three versioning axes

| Axis | Scheme | Example | Source of truth | Hyperscaler precedent |
|---|---|---|---|---|
| **Crate / SDK / library** | SemVer 2.0.0 | `intelligence-control-kernel v3.4.1` | `Cargo.toml` workspace | AWS SDK for Rust, Google libraries, Azure SDK, OCI SDK |
| **Product / platform** | SemVer-derived | `v3.4.0` | `release/X.Y` tag | Kubernetes (`v1.30.0`), .NET (`net10`), Java (`21 LTS`) |
| **External API** | Hybrid stability+date | `/foundry/v1/...?api-version=2026-05-12` | OpenAPI 3.1 spec | Google AIP-180 + Microsoft Azure + AWS + Stripe |

These axes evolve independently. A crate may publish `4.0.0` for an internal
breaking change without bumping the product version, and the product version
may roll forward without breaking any external API.

## 3. Crate / SDK versioning (lockstep → independent at GA)

- **SemVer 2.0.0 mandatory** for every public crate (any `pub` item that crosses
  a crate boundary). Per the Rust Cargo SemVer reference.
- **Workspace lockstep** until `W-Foundry-Preview` ships (single
  `[workspace.package] version = "X.Y.Z"`). This eliminates inter-crate skew
  during pre-GA.
- **Independent cadence after GA**: each crate may bump on its own once the
  internal API surface has stabilised and `governance-semver-discipline`
  is consistently green for 60 days.
- **Pre-release labels**: `alpha.N` → `beta.N` → `rc.N` per SemVer §9. Dev-only
  builds use `dev-snapshot.<YYYYMMDD>` (not registry-publishable).

Detail: [`crate-versioning-spec.md`](crate-versioning-spec.md).

## 4. Product / platform versioning

- **Format**: `vX.Y.Z` (SemVer-derived; `Z` is patch-only).
- **Bump rules**:
  - Major (X) on any breaking external-API change OR any cross-axis contract
    break. Reviewed via the breaking-change ADR template.
  - Minor (Y) on additive feature releases (default cadence rolls minor).
  - Patch (Z) on cherry-pick fixes to a `release/X.Y` branch.
- Product version is **stamped at release-branch cut** (see §6); patch increments
  happen only via the `release-cherry-pick` agent (Directive 12-bounded).

## 5. External API versioning (the hybrid)

Per Google AIP-180 (backwards compatibility) + AIP-181 (stability levels) +
Microsoft Azure `api-version` query parameter + AWS service date versioning:

- **Resource-name version segment** in the URL path: `v1`, `v1beta1`,
  `v1alpha1`, `v2`. This is the *stability* signal. GA = `v1`, beta = `v1beta1`,
  alpha = `v1alpha1`. Breaking change → `v2`, not in-place mutation.
- **Date-based fine-grain via query parameter** (optional, default = latest stable):
  `?api-version=2026-05-12`. Lets us ship additive, non-breaking refinements
  without minting `v1beta1` churn.
- **Per-axis prefix**: `/foundry/v1/...`, `/cloud/v1/...`, `/saas/v1/...`,
  `/workspace/v1/...`, `/search/v1/...`, `/ads/v1/...`. Aligns with the
  axis-prefix already adopted in `contracts/openapi/`.
- **OpenAPI 3.1 is the source of truth**; the spec carries the
  `x-stability` extension (`alpha` / `beta` / `stable` / `ga`) and the
  `x-introduced` / `x-deprecated` dates. Planned advisory lane:
  `governance-api-version-stability`.

Detail: [`api-versioning-spec.md`](api-versioning-spec.md).

## 6. Release branches

- **Naming**: `release/X.Y` (e.g. `release/3.4`). Cut from prod at the
  `vX.Y.0` tag. Compatible with Git Flow-style release-branch convention
  and matches Kubernetes' `release-X.Y` semantics.
- **Read-only**: only the `release-cherry-pick` agent may add commits (see
  [`release-cherry-pick-agent-spec.md`](release-cherry-pick-agent-spec.md)).
- **Tag pattern**: `vX.Y.Z` SemVer. Patches `Z ≥ 1` accumulate on the
  release branch; never on prod.
- Detail: [`release-branch-cut-spec.md`](release-branch-cut-spec.md).

## 7. Release cadence (per axis)

Per the four-layer branch pipeline (origin/dev → origin/staging → origin/prod →
release/X.Y), cadence is set per axis based on data-shape stability:

| Axis | Cadence | Rationale |
|---|---|---|
| Foundry | continuous (no scheduled release) | The 4-layer pipeline auto-promotes; release-branch cut on-demand |
| Cloud | bi-weekly cut from prod tag | Match infra change-window cadence; aligns with Azure SDK monthly BOM |
| SaaS | bi-weekly | Quarter-on-quarter SaaS feature cycle |
| Workspace | bi-weekly | Same as SaaS |
| Vertical-pack | bi-weekly | Customer-facing schema stability windows |
| Search | continuous | Index hot-swap; no branch-cut required |
| Ads | bi-weekly | Auction-model stability windows |

Rationale: hyperscalers vary from Rust's strict 6-week clockwork train to
Azure's monthly BOM to AWS' continuous service evolution. Oyatie matches each
axis to the upstream stability cadence it serves.

## 8. LTS policy

- **Every major version is LTS** for **12 months** from release.
- 90-day EOL warning emitted via `governance-version-eol-warning`
  (signals `EVT-VERSION-EOL-APPROACHING`).
- Tracks per-major-version row in `docs/release/EOL-LEDGER.md`.
- Precedent: Kubernetes patch window (~14 months), .NET LTS (3 years), Java LTS
  (5+ years on Oracle), Azure preview (90 d). Oyatie picks 12 months as the
  median commitment that still allows aggressive forward motion.

Detail: [`version-eol-policy.md`](version-eol-policy.md).

## 9. Deprecation / breaking-change policy

- **180-day sunset notice** for breaking API changes (between AWS' 12-month
  customer agreement and Stripe's "never break" model — calibrated to oyatie's
  enterprise-but-iterative posture).
- Breaking change requires:
  1. Frontmatter `breaking_change: true` on the PR.
  2. ADR using the breaking-change template (`/templates/`).
  3. Entry in `docs/release/SUNSET-LEDGER.md` (180-d countdown).
  4. Approval from BOTH `change-class-reviewer` AND `api-stability-reviewer`.
  5. Major-version bump on next release cut.
- Backwards-compatible changes (per AIP-180) MAY ship at any cadence under the
  current major.

Detail: [`breaking-change-process.md`](breaking-change-process.md).

## 10. Pre-release labels

| Label | Where it appears | Visibility | Hyperscaler analogue |
|---|---|---|---|
| `alpha.N` | crate + API `v1alpha1` | internal + design partners | Kubernetes alpha |
| `beta.N` | crate + API `v1beta1` | early-access tenants | Kubernetes beta, Azure preview |
| `rc.N` | crate + `release/X.Y` before tag | release-candidate channel | Rust beta channel |
| `dev-snapshot.<date>` | crate on origin/dev only | internal, not published | Rust nightly |

## 11. Lift target

`oyatie/docs/release/release-versioning-strategy.md` on approval.
