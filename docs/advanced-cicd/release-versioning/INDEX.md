---
doc_class: Index
shape: anchor
length_cap: 80
authority_tier: 1
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  Catalogue of the release-versioning policy for oyatie. Maps each artefact to its
  lift target, the standard(s) it updates, and the lane(s) that enforce it.
planned_enforcement_ref: governance-orphan-detection
related_adrs: [ADR-0040, ADR-0041, ADR-0050]
doc_status: published
---

# Release-Versioning Policy — Index

> **Status:** Accepted. **Owner:** `axis-foundry`. **Date:** 2026-05-12.
> **Extends:** [ADR-0041](../../../docs/decisions/ADR-0709-general-live-apex.md).

## Strategy + specs (lift to `oyatie/docs/release/`)

| File | Lift target | Updates | Planned advisory lane: |
|---|---|---|---|
| [`release-versioning-strategy.md`](release-versioning-strategy.md) | `docs/release/release-versioning-strategy.md` | `docs/RELEASE-MANAGEMENT.md` §versioning | all 6 new lanes |
| [`crate-versioning-spec.md`](crate-versioning-spec.md) | `docs/release/crate-versioning-spec.md` | `docs/standards/RUST-CRATE-CONVENTIONS.md` | `semver-discipline` |
| [`api-versioning-spec.md`](api-versioning-spec.md) | `docs/release/api-versioning-spec.md` | `docs/SPEC.md §10`; `contracts/openapi/README.md` | `api-version-stability` |
| [`release-branch-cut-spec.md`](release-branch-cut-spec.md) | `docs/release/release-branch-cut-spec.md` | extends ADR-0041 | `release-branch-cut`, `cherry-pick-trail` |
| [`release-cherry-pick-agent-spec.md`](release-cherry-pick-agent-spec.md) | `docs/release/release-cherry-pick-agent-spec.md` | `docs/agents/AGENT-CATALOG.md` | `cherry-pick-trail`, `release-branch-cut` |
| [`version-eol-policy.md`](version-eol-policy.md) | `docs/release/version-eol-policy.md` | new `docs/release/EOL-LEDGER.md` | `version-eol-warning` |
| [`breaking-change-process.md`](breaking-change-process.md) | `docs/release/breaking-change-process.md` | new `docs/release/SUNSET-LEDGER.md` | `deprecation-notice`, `api-version-stability` |
| [`versioning-comparison-matrix.md`](versioning-comparison-matrix.md) | `docs/release/versioning-comparison-matrix.md` | reference only | self |

## Enforcement (lift to `oyatie/docs/standards/`)

| File | Lift target | Updates |
|---|---|---|
| [`enforcement-lanes.md`](enforcement-lanes.md) | `docs/standards/enforcement-lanes-release-versioning.md` | `docs/standards/INDEX.md` |

## New fitness lanes (6)

1. `governance-semver-discipline` (BLOCKER)
2. `governance-api-version-stability` (BLOCKER)
3. `governance-version-eol-warning` (HIGH → BLOCKER on EOL day)
4. `governance-release-branch-cut` (BLOCKER)
5. `governance-cherry-pick-trail` (HIGH)
6. `governance-deprecation-notice` (BLOCKER)

## New agent role (1)

`release-cherry-pick` — the only identity authorised to push to `release/X.Y`
branches and mint patch tags (Directive 12).

## New ledgers (2)

- `docs/release/EOL-LEDGER.md` — per-major-version EOL status (append-only).
- `docs/release/SUNSET-LEDGER.md` — per-deprecation 180-day sunset countdown.

## Citations (≥ 20; full list in [`versioning-comparison-matrix.md`](versioning-comparison-matrix.md) §Sources)

[SemVer](https://semver.org/) · [CalVer](https://calver.org/) · [Cargo SemVer](https://doc.rust-lang.org/cargo/reference/semver.html) · [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks) · [Rust RFC 0507](https://rust-lang.github.io/rfcs/0507-release-channels.html) · [AIP-180](https://google.aip.dev/180) · [AIP-181](https://google.aip.dev/181) · [AIP-185](https://google.aip.dev/185) · [K8s deprecation](https://kubernetes.io/docs/reference/using-api/deprecation-policy/) · [K8s version-skew](https://kubernetes.io/releases/version-skew-policy/) · [K8s patch releases](https://kubernetes.io/releases/patch-releases/) · [K8s release-eng versioning](https://github.com/kubernetes/sig-release/blob/master/release-engineering/versioning.md) · [AWS lifecycle](https://docs.aws.amazon.com/general/latest/gr/service-lifecycle.html) · [AWS SDK maintenance](https://docs.aws.amazon.com/sdkref/latest/guide/maint-policy.html) · [AWS SDK Rust](https://awslabs.github.io/aws-sdk-rust/) · [Azure versioning](https://learn.microsoft.com/en-us/azure/developer/intro/azure-service-sdk-tool-versioning) · [Azure SDK releases](https://azure.github.io/azure-sdk/policies_releases.html) · [.NET support](https://dotnet.microsoft.com/en-us/platform/support/policy/dotnet-core) · [.NET versions](https://learn.microsoft.com/en-us/dotnet/core/versions/) · [Java SE roadmap](https://www.oracle.com/java/technologies/java-se-support-roadmap.html) · [Java releases](https://www.baeldung.com/java-time-based-releases) · [OCI Java SDK](https://github.com/oracle/oci-java-sdk/blob/master/CHANGELOG.md) · [Stripe versioning](https://docs.stripe.com/api/versioning) · [Stripe blog](https://stripe.com/blog/api-versioning) · [Twilio v2008→v2010](https://www.twilio.com/docs/usage/api/upgrade-from-2008-to-2010-api) · [Git Flow](https://www.atlassian.com/git/tutorials/comparing-workflows/gitflow-workflow) · [Conventional Branch](https://conventional-branch.github.io/)
