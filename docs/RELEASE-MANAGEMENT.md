---
purpose: Oyatie — Release Management
doc_status: published
---

# Oyatie — Release Management

> **Status:** Draft v0.1 — 2026-05-09.
> **Owner:** `ops-sre-reliability` + `axis-foundry` (engineering platform / CI lanes).
> **Companion:** [SLO-CATALOG.md](SLO-CATALOG.md), [QA-TEST-STRATEGY.md](QA-TEST-STRATEGY.md), [INCIDENT-MANAGEMENT.md](INCIDENT-MANAGEMENT.md), ADR-0039 (supply-chain Trivy/Cosign/SBOM), ADR-0042 (trunk-based release discipline), and the native release-conveyor plan.

## 1. Trunk-based + release-branch model (per ADR-0042)

- One main branch (`main`); short-lived feature branches per team worker brief.
- Release branch (`release/x.y.z`) cut at tag time.
- No long-lived develop branch.
- No fast-forward merges to `main` (squash/rebase only); per branch protection.

## 2. CI lane catalog

Per `docs/standards/ci-lanes.md`, Buck2 target inventory, and generated
Prow/Kubernetes-native job metadata:

| Lane | Trigger | Hard-fail? |
|---|---|---|
| Buck2 format/lint/check target set | every PR | yes |
| Buck2 Rust test target set with nextest/LLVM coverage where applicable | every PR | yes |
| Buck2 dependency/license policy target set | every PR | yes (per License Policy ADR) |
| Buck2 architecture-boundary target | every PR | yes (post PG-1 per ADR-0015) |
| Buck2 catalog-contract target | every PR | yes |
| Buck2 claim-ceiling/foundation-bypass/plane-class targets | every PR or relevant catalog PR | yes |
| `oya-governance-license` | every PR | yes |
| `oya-governance-data-class` | every PR | yes |
| `oya-governance-cohesion` (cross-axis drift) | every PR | yes (warn first wave; block at W-Foundation gate) |
| `oya-governance-doc-catalog` | PRs touching `docs/**` | yes |
| `oya-governance-slo-coverage` | every PR | yes |
| `oya-governance-blast-radius` | every PR | label-emit |
| Trivy 4-layer scan (per ADR-0039) | every PR + nightly | yes |
| Cosign sign + Rekor anchor | release artifact | yes |
| SBOM generation (SPDX 2.3 + CycloneDX 1.5) | release artifact | yes |
| Affected-graph test runner | every PR | optimization |
| sccache hit-rate metric | every PR | observability |
| Insta snapshot test | per-test | yes |
| Per-capability eval harness | per-capability publish | yes |
| Visual regression (frontend) | per-frontend PR | yes (post `CHROMATIC_PROJECT_TOKEN` provisioning per #56) |
| Lighthouse budget | per-frontend PR | yes (per #69) |
| Property tests (per pure-function module) | per PR touching such module | yes (per #71) |
| Hot-path benchmark gate | per PR touching tagged surfaces | yes (per #72) |
| Nightly `--all-features` matrix on `main` | nightly | observability |
| RustSec/audit target | per PR + daily | yes (per #63) |
| OpenAPI semver-diff gate | per PR touching `contracts/` | yes (per ADR-0040) |

## 3. Progressive delivery (native release conveyor)

Per surface:
- Canary (5% → 25% → 50% → 100%) with metric-gated automatic rollback
- Blue/green for stateful surfaces
- Per-region phased rollout (KR-Seoul1 first, then per regional pack order)
- Burn-rate gate: roll back if SLO budget burns ≥ 14.4× over 1h post-deploy

## 4. Release cadence

- Per-axis cadence varies; default: weekly minor; monthly major
- `release/x.y.z` branch cut at tag; cherry-picks only for hotfix
- Release notes auto-drafted by Foundry capability `pr.release-note.draft`
- Trust-portal updated on regulator-impact releases
- Post-release: per-region SLO baseline check before next release

## 5. Hotfix path

- `hotfix/x.y.z+1` branch from latest release tag
- Mandatory: Sev-1/2 incident open
- Mandatory: rollback path documented before merge
- Skip: weekly cadence
- Cherry-pick to `main` after merge

## 6. Pre-release verification (per ADR-0040 9-item readiness)

Per release-candidate, run the Buck2/Prow release evidence bundle:
1. All CI lanes green on the release tag SHA
2. SBOM generated + Cosign-signed + Rekor-anchored
3. Per-region SLO budget ≥ 50%
4. No open Sev-1/2 incident affecting the release surfaces
5. License-policy gate green
6. Audit-chain emission test passed
7. DSR cascade test passed in last 30d
8. Per-region regulator-watch checked in last 14d
9. Per-pack regulatory pack version compatibility verified
10. Trust-portal mirror generated

## 7. Per-axis release exceptions

- **Foundry capabilities**: per-capability semver + sunset per ADR-0001/0230; eval-set pass required per release
- **Cloud control plane**: API versioning per ADR-0040 stricter; major bumps require 12-month deprecation
- **Workspace Mail / Doc / Drive**: data-format compatibility ≥ 2 prior versions
- **Workspace Meet**: WebRTC compatibility tested per browser matrix

## 8. Release on Foundry

Releases ARE Foundry capabilities (recursive):
- `release.cut.weekly` capability
- `release.hotfix.dispatch` capability
- `release.evidence.regenerate` capability
- `release.canary.advance` capability

## 9. Sources
ADR-0050, 0188, 0207, 0229, 0230, 0231, CLAUDE.md, docs/standards/ci-lanes.md, docs/standards/code-review.md.
