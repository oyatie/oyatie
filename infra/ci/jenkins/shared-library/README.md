# oya-jenkins-shared — CI shared library (ADR-0361)

The single source of the CI lane definition that replaced the 36 retired GitHub
Actions workflows. Configured as a Jenkins Global Pipeline Library named
`oya-jenkins-shared` (JCasC); lanes call `@Library('oya-jenkins-shared') _`.

## Entry point: `oyaCiLane(Map cfg)`

Runs the research-vetted **shift-left, license-clean, supply-chain-hardened** stage
order (ADR-0361 §3). Mandatory stages block merge:

| Stage | Tool | License |
|---|---|---|
| lint | cargo fmt + clippy `-D warnings` | — |
| license/advisory/bans | **cargo-deny** (blocks BSL/SSPL/source-available) | MIT/Apache-2.0 |
| SAST | **Opengrep** (own rules; NOT Semgrep registry rules) | LGPL-2.1 |
| secret scan | **gitleaks** | MIT |
| test | `oya verify --affected` (PR) / `--ci-required` (trunk) | — |
| SBOM | **cargo-cyclonedx** + **Syft** (CycloneDX) | Apache-2.0 |
| vuln/IaC | **Trivy** + **osv-scanner** | Apache-2.0 |
| sign + provenance (trunk) | **cosign** + **in-toto/SLSA** | Apache-2.0 |

Admission (**Kyverno verifyImages**) + CD (**Argo CD** + **Argo Rollouts**) run in the
cluster, not the CI lane — see `infra/ci/argocd/` and `infra/kyverno/`.

## Layout
- `vars/oyaCiLane.groovy` — the lane definition.
- `examples/microservice-lane.Jenkinsfile` — the thin per-`<ms>` form.
- Root `Jenkinsfile` (repo root) — the repo-wide governance gate + lane fan-out.

## Migration
Existing inline lanes (e.g. `microservices/cloud-iac/ci/Jenkinsfile`, which carries
its own kata/cosign pod spec) migrate to the one-liner `oyaCiLane(...)` form as the
shared library's pod template subsumes the inline copy — incremental, per microservice.

## Forbidden (OSI-strict license policy, ADR-0361)
Snyk (proprietary SaaS), Drone (source-available), Mend Renovate CE/EE (closed EULA),
Semgrep **registry rules** (proprietary v1.0). Jenkins core is MIT; plugins are
mixed-license and allowlisted at the infra layer. Renovate OSS (AGPL) + trufflehog
(AGPL) are allowed self-hosted.

## Honest status
This is pipeline-as-code. Tool invocations assume the tools are baked into the agent
image (O5) or available on PATH; the production agent image bake + the Jenkins library
registration (JCasC) are deploy-time. No CI-throughput/coverage claim is made here.
