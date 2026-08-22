# CI/CD Pipeline Revamp — Jenkins-native, license-clean, hyperscaler-grade

## Problem Statement
How might we collapse 36 fragmented GitHub Actions workflows into one Jenkins-native,
license-clean, supply-chain-hardened pipeline — replacing Actions entirely — without
weakening governance or bricking the merge gate during cutover?

## Recommended Direction
**Per-microservice Jenkins lanes + a root orchestrator**, shift-left ordered, using
only OSI/self-hostable tools (research-vetted). Each `microservices/<ms>/ci/Jenkinsfile`
runs the lane via a shared library; the root gate runs `oya verify --affected` (PR) /
`--ci-required` (merge). This reuses O1 affected-scope + the N-way parallel farm.

The **license-clean stage order** (mandatory = blocks merge):
`cargo fmt/clippy` → **cargo-deny** (license allow/deny + RustSec advisories + bans —
*this is the BSL/SSPL block-enforcement primitive*) → **SAST** (Opengrep or Semgrep CE
engine with our own rules — NOT the now-proprietary Semgrep registry rules) →
**gitleaks** (MIT) secret scan → tests (`nextest`) → **SBOM** (cargo-cyclonedx +
Syft, CycloneDX primary) → **Trivy** + **osv-scanner** vuln/IaC scan → **cosign** sign +
**in-toto/SLSA** provenance → **Kyverno `verifyImages`** admission → **ArgoCD + Argo
Rollouts** canary. All Apache-2.0/MIT/LGPL — no Snyk, no Drone, no Mend-CE.

Cutover is **retire-in-code now**: delete the 36 workflows, drop the parity gate, and —
critically — **repoint the `hyperscaler-maturity-claims`/`protection-context-match` gate**
from `pr-review.yml`/`ci-failure-fix-loop.yml` to the Jenkins pipeline closure, so
`oya verify` stays green. Branch-protection contexts swap Actions→Jenkins in both
`dev.json` and `branch-protection.yaml` (kept in agreement).

## Key Assumptions to Validate
- [ ] The closure-gate rewrite preserves enforcement strength (linear-history,
      no-force-push, agent-review-authority, fix-loop, merge-queue) against the
      Jenkinsfile — TDD the new validator against a golden Jenkinsfile.
- [ ] A production Jenkins + GitHub App will report the new status contexts before the
      live branch-protection API swap (else PRs stall) — sequencing, deploy-time.
- [ ] Semgrep rules license risk is real → default to **Opengrep** (open rules) or
      repo-owned rules; never vendor the proprietary registry rules.
- [ ] Jenkins plugin licenses (mixed GPL / Eclipse Public License) are allowlisted at the infra layer.

## Minimum first slice (this lane, committed in-code)
**In:** ADR-0361 (cutover); a shared Jenkins library + one reference
`microservices/<ms>/ci/Jenkinsfile` lane + root orchestrator with the mandatory stage
order; the repointed closure gate (TDD); branch-protection context swap; deletion of
the 36 Actions workflows + parity gate; `oya verify` green.
**Out (deploy/infra-time):** live GitHub branch-protection API change; production
Jenkins controller + GitHub App registration; populating real Trivy/cosign creds.

## Not Doing (and why)
- **Semgrep registry rules** — proprietary (Semgrep Rules License v1.0, Dec-2024);
  use Opengrep/own rules. **Snyk** (proprietary SaaS), **Drone** (source-available),
  **Mend Renovate CE/EE** (closed EULA) — all forbidden by the OSI-strict policy.
- **Parity gate** (Actions↔Jenkins) — explicitly dropped; we go Jenkins-native, not
  dual-maintained.
- **Live cutover of GitHub settings / prod Jenkins** — deploy-time, not code-time;
  this lane ships the cutover-in-code, reviewable in the PR.
- **Per-gate O7 cache adoption** — separate incremental lane; not coupled here.

## Open Questions
- Status-context granularity: one `verify` required check, or a small set
  (`verify`, `pr-review`, `supply-chain`)? (Leaning: 2–3, mapped from the
  current 15 so reviewers see meaningful gates.)
- SBOM/attestation storage: in the OCI registry alongside the image (cosign attach) vs
  a SeaweedFS evidence bucket? (Leaning: registry via cosign, mirrored to evidence.)
- trufflehog (AGPL) as a second secret-scan source, or gitleaks (MIT) alone? (Leaning:
  gitleaks alone v1; trufflehog deferred.)
