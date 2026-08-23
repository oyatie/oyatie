---
purpose: Oyatie — Release Management
doc_status: published
---

# Oyatie — Release Management

> **Status:** Published compatibility doc — authority drift reconciled 2026-07-01.
> **Owner:** `ops-sre-reliability` + `axis-foundry` (engineering platform / CI lanes).
> **Companion:** [SLO-CATALOG.md](SLO-CATALOG.md), [QA-TEST-STRATEGY.md](QA-TEST-STRATEGY.md), [INCIDENT-MANAGEMENT.md](INCIDENT-MANAGEMENT.md), ADR-0050 (Argo Rollouts), ADR-0039 (supply-chain Trivy/Cosign/SBOM), ADR-0042 (GitOps + trunk-based).

## 1. Protected-integration + release-branch model (per ADR-0042, amended by ADR-0363/ADR-0515)

- One protected integration branch (`dev`); short-lived feature branches per team worker brief.
- Release branch (`release/x.y.z`) cut at tag time.
- No long-lived unprotected `develop` branch; `dev` is the protected PR target named by the operating contract.
- No fast-forward merges to `dev` (squash/rebase only); per branch protection.

## 2. CI lane catalog

Per `docs/standards/ci-lanes.md` (legacy catalog) + pipeline gate apps. The single protected merge authority is `presubmit`; local `oya` output is bridge evidence only and cannot independently authorize merge.

| Lane | Trigger | Hard-fail? |
|---|---|---|
| `cargo-fmt` | every PR | yes |
| `cargo-clippy --workspace --all-features --all-targets -- -D warnings` | every PR | yes |
| `cargo-nextest --workspace --all-features` | every PR | yes |
| `cargo check --workspace --all-targets --all-features` | every PR (PM-2 mitigation) | yes |
| `cargo deny check licenses` | every PR | yes (per License Policy ADR) |
| presubmit Rust gate packet: architecture-boundaries, catalog, claim-ceiling, foundation-bypass, plane-class | every PR | yes, through `presubmit` |
| `governance-license` | every PR | yes |
| `governance-data-class` | every PR | yes |
| `governance-cohesion` (cross-axis drift) | every PR | yes (warn first wave; block at W-Foundation gate) |
| `governance-doc-catalog` | PRs touching `docs/**` | yes |
| `pipeline-slo-coverage` | every PR | yes |
| `governance-blast-radius` | every PR | label-emit |
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
| Nightly `--all-features` matrix on `dev` / promoted release tip | nightly | observability |
| RUSTSEC + cargo-audit | per PR + daily | yes (per #63) |
| OpenAPI semver-diff gate | per PR touching `contracts/` | yes (per ADR-0040) |

## 3. Progressive delivery (per ADR-0050 Argo Rollouts)

Per surface:
- Canary (5% → 25% → 50% → 100%) with metric-gated automatic rollback
- Blue/green for stateful surfaces
- Per-region phased rollout (KR-Seoul1 first, then per regional pack order)
- Burn-rate gate: roll back if SLO budget burns ≥ 14.4× over 1h post-deploy

## 4. Release cadence

- Per-axis cadence varies; default: weekly minor; monthly major
- `release/x.y.z` branch cut at tag; cherry-picks only for hotfix
- Release notes are drafted by the current release-governance automation or by the PR author; Release Please is optional and only applies when a live repo config/workflow exists.
- Trust-portal updated on regulator-impact releases
- Post-release: per-region SLO baseline check before next release

### 4.1 Post-merge product-completion gate

A squash merge only enters post-merge verification. A change is not product-complete until a closeout packet records:

1. promoted commit SHA plus post-merge `presubmit` status URL in the green state;
2. rollout verification for the deployed artifact, canary/flag state, or explicit `no deployable artifact` rationale;
3. rollback note naming the command, runbook, digest, or no-op rationale;
4. observability check naming the golden-signal/SLO dashboard and time window;
5. browser UX/user-story evidence for user-visible surfaces, or explicit `not user-visible` rationale;
6. release-governance/release-note impact: release PR/link or generated notes from the configured release system, or `no user-facing release-note impact` when there is no user-facing release surface.

## 5. Hotfix path

- `hotfix/x.y.z+1` branch from latest release tag
- Mandatory: Sev-1/2 incident open
- Mandatory: rollback path documented before merge
- Skip: weekly cadence
- Cherry-pick to the protected integration/release branch after merge

## 6. Pre-release verification (per ADR-0040 9-item readiness)

Per release-candidate, attach the presubmit release packet (legacy `/release-verify` output is local bridge evidence only):
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

- **Capability/intelligence automation**: per-capability semver + sunset per ADR-0001/0230; eval-set pass required per release
- **Cloud control plane**: API versioning per ADR-0040 stricter; major bumps require 12-month deprecation
- **Workspace Mail / Doc / Drive**: data-format compatibility ≥ 2 prior versions
- **Workspace Meet**: WebRTC compatibility tested per browser matrix

## 8. Release automation capabilities

Release operations are capability-governed automation surfaces:
- `release.cut.weekly` capability
- `release.hotfix.dispatch` capability
- `release.evidence.regenerate` capability
- `release.canary.advance` capability

## 9. Sources
ADR-0050, 0188, 0207, 0229, 0230, 0231, CLAUDE.md, docs/standards/ci-lanes.md, docs/standards/code-review.md.
