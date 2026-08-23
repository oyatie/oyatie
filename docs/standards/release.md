---
purpose: Oyatie — Release Standard
doc_status: published
---

# Oyatie — Release Standard

> **Owner:** `ops-sre-reliability` + `axis-foundry`.
> **Companion:** [RELEASE-MANAGEMENT.md](../RELEASE-MANAGEMENT.md), ADR-0040 (progressive delivery), ADR-0041 (GitOps + trunk-based), ADR-0039 (supply chain).

## 1. Branch model (per ADR-0041)

- One protected integration branch (`dev`)
- Short-lived feature branches per Team worker brief
- Release branch (`release/x.y.z`) cut at tag time
- No long-lived unprotected `develop` branch; `dev` is the protected PR target
- No fast-forward (squash/rebase only)

## 2. Pre-release verification

Per `/release-verify` (formerly `/release-verify`):

1. ☐ All CI lanes green on the release tag SHA per [RELEASE-MANAGEMENT.md §2](../RELEASE-MANAGEMENT.md)
2. ☐ SBOM generated + Cosign-signed + Rekor-anchored per ADR-0039
3. ☐ Per-region SLO budget ≥ 50%
4. ☐ No open Sev-1/2 incident affecting release surfaces per [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md)
5. ☐ License-policy gate green per ADR-0013
6. ☐ Audit-chain emission test passed per ADR-0003
7. ☐ DSR cascade test passed in last 30d per ADR-0038
8. ☐ Per-region regulator-watch checked in last 14d per [COMPLIANCE-MATRIX](../COMPLIANCE-MATRIX.md)
9. ☐ Per-pack regulatory pack version compatibility verified per ADR-0010
10. ☐ Trust-portal mirror generated per [DOCUMENTATION.md §3](../DOCUMENTATION.md)

## 3. Progressive delivery (per ADR-0040)

- Canary 5% → 25% → 50% → 100% with metric-gated automatic rollback
- Per-region phased rollout (per regional pack order)
- Per-cell rollback per ADR-0009
- Burn-rate gate: rollback if 1h SLO burn ≥ 14.4×

## 4. Hotfix path

- `hotfix/x.y.z+1` from latest release tag
- Mandatory: open Sev-1/2 incident
- Mandatory: rollback path documented
- Skip: weekly cadence
- Cherry-pick to the protected integration/release branch post-merge

## 5. Per-axis release extensions

- **Capability/intelligence automation**: per-capability semver + sunset per ADR-0037; eval-set pass per release per ADR-0024
- **Cloud control plane**: stricter API versioning per ADR-0037; major bumps with 12-month deprecation
- **Workspace Mail / Doc / Drive**: data-format compatibility ≥ 2 prior versions
- **Workspace Meet**: WebRTC compatibility tested per browser matrix
- **Public API**: per-deprecation telemetry emitted (per ADR-0037)

## 6. Post-release

- Per-merge changelog row auto-emit through current release-governance automation or author closeout
- Per-release notes drafted by current release-governance automation or author closeout; Release Please applies only when repo config proves it
- Per-merge product-completion packet recorded: promoted SHA `presubmit`,
  rollout verification, rollback note, observability check, browser UX/user-story
  evidence, and release-governance/release-note impact (Release Please only when repo config proves it)
- Trust-portal updated for regulator-impact releases
- Per-region SLO baseline check before next release

## 7. Sources
[RELEASE-MANAGEMENT.md](../RELEASE-MANAGEMENT.md), ADR-0003/0009/0010/0013/0024/0037/0038/0039/0040/0041, CLAUDE.md release rules.
