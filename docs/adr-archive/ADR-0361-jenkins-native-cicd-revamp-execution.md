---
id: ADR-0361
status: Superseded
planning_impact: true
date: 2026-05-25
owners:
  - council-architecture
  - ops-platform
  - axis-dev-cli
  - ops-sre-reliability
supersedes: []
superseded_by: [ADR-0515]
amends:
  - ADR-0359-jenkins-completely-replaces-github-actions.md
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.

# ADR-0361: Execute the cloud-ci-native CI/CD revamp — license-vetted hyperscaler supply-chain stack, retire GitHub Actions, drop the parity gate (superseded by ADR-0515)

## Status

Superseded by ADR-0515 — 2026-06-06: the license-vetted supply-chain tool stack (cargo-deny/Opengrep/gitleaks/Syft/Trivy/osv/cosign/in-toto-SLSA/Kyverno) is retained as ADR-0515's layer-e gate steps. Executes ADR-0359 (cloud-ci completely replaces GitHub Actions) and ADR-0349 (cloud-ci+ArgoCD substrate). Tool selections are license-vetted against the repo's OSI-strict policy (the Redis→Valkey precedent). Cutover-in-code now; the live GitHub branch-protection API change + production cloud-ci GitHub App apply at deploy.

## Context

CI is 36 fragmented GitHub Actions workflows producing the 15 required branch-protection status checks; Actions is also GitHub-budget-blocked. ADR-0359 decided cloud-ci fully replaces it but was unexecuted. The local cloud-ci+SeaweedFS+ArgoCD farm (ADR-0349) + the O1–O7 optimization program (ADR-0360) are the substrate. The blocker for a clean cutover is that the `hyperscaler-maturity-claims` / `protection-context-match` gate hard-codes a GitHub-Actions pipeline closure (it reads `pr-review.yml` + `ci-failure-fix-loop.yml`), so retiring Actions requires repointing that gate at the cloud-ci pipeline. Best-practice research (Google/AWS/Microsoft/Oracle supply-chain patterns) + a license audit produced the allowed tool stack below.

## Decision

1. **cloud-ci-native, per-microservice lanes + root orchestrator.** Each
   `microservices/<ms>/ci/` pipeline runs its lane via a shared library; the root
   gate runs `oya verify --affected` (PR) / `--ci-required` (merge). Reuses O1
   affected-scope + N-way parallel agents.
2. **License-vetted supply-chain stack (OSI/self-hostable only):**
   `cargo-deny` (license allow/deny + RustSec + bans — the BSL/SSPL block primitive),
   SAST via **Opengrep / Semgrep CE engine with repo-owned rules** (NOT the proprietary
   Semgrep registry rules), `gitleaks` (MIT) secret scan, SBOM via `cargo-cyclonedx` +
   `Syft` (CycloneDX primary), `Trivy` + `osv-scanner` vuln/IaC scan, `cosign` signing +
   `in-toto`/SLSA provenance, `Kyverno verifyImages` admission, `Argo CD` + `Argo
   Rollouts` CD. All Apache-2.0/MIT/LGPL-2.1.
3. **Shift-left order (mandatory = blocks merge):** lint → cargo-deny → SAST → gitleaks
   → tests → SBOM → Trivy+osv → cosign+provenance → Kyverno admission → Argo Rollouts.
4. **Retire GitHub Actions in-code:** delete the 36 `.github/workflows` lanes and the
   cloud-ci/GitHub-Actions parity workflow; remove the parity lane from
   the gate catalog. **Parity is explicitly dropped** — cloud-ci-native, not dual-maintained.
5. **Repoint the closure gate:** rewrite `validate_pr_review_pipeline` /
   `validate_pipeline_closure` to validate the cloud-ci pipeline closure instead of the
   Actions workflows, preserving the linear-history, no-force-push, agent-review-authority,
   fix-loop, and merge-queue assertions.
6. **Branch-protection contexts** swap Actions→cloud-ci in `infra/branch-protection/dev.json`
   AND `.github/branch-protection.yaml`, kept in agreement for `protection-context-match`.

### Forbidden / caveat (license policy)

Snyk (proprietary SaaS), Drone CI (source-available Polyform), Mend Renovate CE/EE
(closed EULA), and the Semgrep registry rules (Semgrep Rules License v1.0) are
FORBIDDEN. Renovate OSS (AGPL-3.0) and trufflehog (AGPL-3.0) are ALLOWED self-hosted
(network-copyleft only on a modified served instance). cloud-ci core is MIT-compatible;
its plugins are mixed-license and MUST be allowlisted at the infra layer.

## Consequences

- One pipeline replaces 36 workflows; supply-chain is hardened end-to-end (SBOM +
  vuln + signing + provenance + admission), all license-clean and self-hostable.
- The closure-gate rewrite is the load-bearing change: it keeps `oya verify` green
  through the cutover and must preserve enforcement strength (TDD against a golden
  cloud-ci pipeline definition).
- Live cutover (GitHub branch-protection API + production cloud-ci GitHub App reporting
  the new contexts) is deploy-time; until then this is the cutover-in-code, reviewable
  in the PR, and GitHub Actions checks remain red (budget) but are being removed.
- All throughput/coverage/maturity claims stay blocked_until_required_evidence_is_green.
