---
id: ADR-0535
title: "Cross-product versioning + reproducible-build attestation + distribution channel (OCI + pinned-git, crates.io optional-mirror-only) + repo-automation (oya-deps.toml bump-bot); re-authors Proposed ADR-0037/0041/0342/0345/0050 into clean release-governance canon"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-08
door: one-way
owner: founder
supersedes: [ADR-0037, ADR-0041, ADR-0342, ADR-0345, ADR-0050]
superseded_by: [ADR-700]
depends_on: [ADR-0516, ADR-0532]
amends: []
related: [ADR-0091, ADR-0181, ADR-0512, ADR-0516, ADR-0519, ADR-0526, ADR-0529, ADR-0531, ADR-0532]
related_specs:
  - /specs/bespoke-cloud-toolchain-services.json
  - /specs/oss-stewardship-registry.json
  - /specs/masterplan.json
milestone: W3
---

# ADR-0535: Cross-product versioning + release governance

## Status

**Accepted — 2026-06-08 (founder-ruled; door: one-way).**

Detail under Component 3 of ADR-0516. This ADR **re-authors into clean release-governance canon** the
five Proposed ADRs that predated the buck2 + scm-facts + oya-ci substrate: ADR-0037 (public-API
stability tiers + deprecation), ADR-0041 (gitops trunk-based + release-branch-cut-at-tag), ADR-0342
(API versioning hybrid date+SemVer), ADR-0345 (OSS stewardship classes + CVE-response SLA), and
ADR-0050 (automation-first pipeline). Per the amend=supersede rule, those Proposed files are git-rm'd
once their decision substance is re-homed here (and into ADR-0519/0529 for the ADR-0050 automation
governor). All forbidden "foundry" vocabulary from the source ADRs is scrubbed on re-author.

## Context

Release governance was scattered across five Proposed ADRs, each authored before the canonical buck2
build graph (ADR-0392/0522), the scm-facts seam (ADR-0526), and the oya-ci product line (ADR-0532).
They cite a retired CLI, the forbidden "foundry" owner/vocab (ADR-0050, ADR-0041), and external SaaS
that contradict the owned-cloud-native + no-external-blob doctrine. The masterplan-SSOT
resolve-every-Proposed rule requires each to be ratified or re-authored; this ADR re-authors them into
one clean canon consistent with the fabric (ADR-0516).

## Decision

**(1) VERSIONING — the 3-axis model (re-authored from ADR-0037 + ADR-0342).** Each of the seven
products (ADR-0532) carries its OWN SemVer 2.0.0 + a published versioned config schema (`$id` +
`schema_version`) so an adopter pins a product version and a config-schema version independently. The
three axes:

- **Crate/SDK axis (SemVer, re-authored from ADR-0037):** SDK packages use `MAJOR.MINOR.PATCH`. MAJOR =
  breaking interface change; MINOR = additive; PATCH = bug fix. The public-API stability tiers
  (preview / stable / GA) with their breaking-change policy and deprecation lead-times (preview: none;
  stable: ≥6 months; GA: ≥12 months) are preserved. Contracts-first artifacts live at `contracts/`;
  SDKs are generated per language; every deprecated endpoint emits a deprecation telemetry event.
- **Product axis:** `oya-vX.Y.Z` with the version-prefix as a CONFIG KNOB so an adopter uses
  `widget-vX.Y.Z`.
- **External-API axis (HYBRID, re-authored from ADR-0342):** date-based versions (`YYYY-MM-DD`) on the
  public boundary (the Stripe/Anthropic/OpenAI/AWS/GitHub pattern) carried by header + URL prefix +
  proto field, with SemVer on the SDK boundary; each SDK release pins one date-version under the hood.
  Last N=3 public versions supported in parallel; ≥180-day post-deprecation window; per-tenant version
  pinning; every tenant-affecting breaking change requires a paired sunset-class ADR + RFC 8594/9745
  deprecation headers. (LTS = 12 months; 90-day EOL warn; 180-day sunset.)

A NET-NEW cross-product compatibility matrix (which producer pairs with which config-schema / build
overlay / runner) is required for an adopter to pin a coherent stack.

**(2) RELEASE BRANCH MODEL (re-authored from ADR-0041).** Trunk-based development; short-lived feature
branches; release branches cut at tag time (`vX.Y.Z`), not maintained ahead of tag; squash-or-rebase
merge only (linear history); branch-protection-as-code; a merge queue serializing any PR that touches a
workspace-root manifest. (The legacy "foundry" owner and CLI-gate framing are dropped; gates are the
oya-ci-required Rust pipeline per ADR-0515.)

**(3) REPRODUCIBLE-BUILD as an ATTESTED product promise.** A `reproducible:true` attestation recorded
per release-cut in `oya-cd-release-ledger`, verified in a FRESH clean checkout (not the warm tree), per
the hermetic doctrine; reproducibility gates ship BORN-ADVISORY until the verified-missing
`scripts/check.sh` + CI wiring that `deny.toml` falsely claims is closed.

**(4) DISTRIBUTION CHANNEL.** Given the no-external-blob/self-host doctrine, the engine ships as OCI
artifacts + pinned-git-release (mirrored, content-addressed) with crates.io as an OPTIONAL convenience
mirror that is NEVER a build dependency; every published artifact reproducible-from-source.

**(5) REPO-AUTOMATION (P7 oya-govbot; re-authored from ADR-0345 + ADR-0050).** A closed-schema
`oya-deps.toml` (LTS roster + license allow/deny as DATA + the supply-chain triad
cargo-audit/deny/vet + the OSS stewardship registry + CVE SLAs) drives an IN-HOUSE Rust bump-bot that
opens scm-facts ChangeSets (provider-neutral, NOT GitHub PRs) with license/advisory/strict-version
gates pre-run. The OSS stewardship classes (Maintainer / Contributor / Consumer) and their CVE-response
SLAs (Contributor: P0 ≤7d, P1 ≤30d; Consumer: pin update ≤14d) become DATA in the stewardship registry.
The verified-absent renovate.json/dependabot.yml are NOT adopted; build the release-governance gate
crates + auto-changelog + EOL/sunset + branch-protection-drift bots, all on the same closed-schema
config, all emitting scm-facts ChangeSets. The Rust stable toolchain drift guard lives at
`ci/facade/generated-artifact-freshness/src/rust_toolchain_drift.rs`, embedded in the Rust
`oya-cloud-ci-freshness-app` gate and wired into `oya-ci-required` so the repo follows the pinned
stable channel without split-brain manifests, Docker tags, workflow pins, or active-doc residue;
GitHub Actions remains only the transitional runner adapter, while the policy source of truth is the
owned cloud-ci Rust gate/API. The automation-first doctrine (formerly ADR-0050) is
governed by the AUTO/ADVISE/GATE safety governor (ADR-0519/0529).

Implementation guardrail (2026-06-25): the owned dependency-automation contract is born as
`oya-deps.toml`, with root ownership carried by `OWNERS` and the first Rust cloud-ci enforcement
surface at `ci/facade/dependency-automation/BUCK`,
`ci/facade/dependency-automation/Cargo.toml`,
`ci/facade/dependency-automation/OWNERS`,
`ci/facade/dependency-automation/src/lib.rs`,
`ci/facade/dependency-automation/src/main.rs`, and
`ci/facade/dependency-automation/tests/dependency_automation.rs`.

## Drivers

- The published-artifact-vs-no-external-blob doctrine collision (resolved: OCI + pinned-git,
  crates.io mirror-only).
- The verified gaps: the semver-check crate is a scaffold stub; renovate/dependabot are absent;
  `deny.toml` claims nonexistent CI.
- The scm-facts VCS-agnostic seam ("git is transitional", ADR-0526); the ratified-vision constraint
  that transitional substrates sit behind stable interfaces.

## Alternatives considered

- **(a) adopt Renovate/Dependabot** — rejected (external SaaS, not VCS-agnostic, contradicts
  owned-cloud-native + scm-facts).
- **(b) crates.io as the primary channel** — rejected (makes a public registry a build dependency,
  violates no-external-blob/self-host).
- **(c) workspace-lockstep versioning forever** — rejected (external adopters must pin a product
  independently).

## Consequences

RESOLVES the five Proposed ADRs by re-authoring them into this clean release-governance canon
(amend=supersede): **ADR-0037** (API stability tiers → versioning axis 1), **ADR-0041** (release branch
model → §2; "foundry" owner scrubbed), **ADR-0342** (API versioning hybrid → versioning axis 3),
**ADR-0345** (OSS stewardship + CVE SLA → the oya-deps.toml stewardship registry DATA), **ADR-0050**
(automation-first → §5 bump-bot + the ADR-0519/0529 safety governor; "foundry-driven triage" framing
dropped, "foundry" owner scrubbed). Each predecessor file is git-rm'd AFTER its substance lands here;
inbound references are scrubbed via the canon-id-crosswalk at the INTEGRATE phase (not in this authoring
phase). OQ-3 (distribution channel) and OQ-8 (one govbot vs three sub-products) are carried to founder
(ADR-0521). door:one-way.

---
*Accepted 2026-06-08 (founder-ruled; door:one-way). Source: PLATFORM-PRODUCTIZATION-ARCHITECTURE.md
(RATIFY-TO-ADR). Re-authors ADR-0037/0041/0342/0345/0050 (amend=supersede; "foundry" vocab scrubbed).
Detail under Component 3 of ADR-0516.*
