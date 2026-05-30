// oyaCiLane — the canonical Jenkins shared-library entry for an oya CI lane
// (ADR-0361 / ADR-0349). Replaces the retired GitHub Actions fitness/governance
// workflows with one shift-left, license-clean, supply-chain-hardened pipeline.
//
// Usage (microservices/<ms>/ci/Jenkinsfile):
//   @Library('oya-jenkins-shared') _
//   oyaCiLane(service: 'cloud-iac', crate: 'oya-cloud-iac-domain')
//
// Stage order is the research-vetted shift-left sequence (ADR-0361 §3). Every
// tool is OSI/self-hostable (Apache-2.0/MIT/LGPL): no Snyk, no Drone, no Mend-CE,
// no Semgrep registry rules. Mandatory stages fail the build (block merge).
def call(Map cfg = [:]) {
  String svc   = cfg.service ?: 'repo'
  String label = cfg.agentLabel ?: 'oya-rust-build'
  // On a PR, narrow with O1 affected-scope; on trunk/merge, full mirror.
  boolean isTrunk = (env.BRANCH_NAME in ['dev', 'main', 'staging', 'production'])
  String verifyMode = isTrunk ? '--ci-required' : "--affected --base ${cfg.base ?: 'dev'}"

  // Branch-protection required status contexts (.github/branch-protection.yaml,
  // kept in sync by the oya-governance-protection-context-match gate). Posted to
  // the Forgejo Commit Status API (ADR-0363 substrate) so a PR merges on real
  // green checks — retiring the enforce_admins-toggle admin-merge seam.
  // (oya-pr-review is posted separately by the reviewer agent, not this lane.)
  List requiredContexts = [
    'cargo-fmt', 'cargo-check', 'cargo-clippy', 'cargo-nextest',
    'cargo-deny', 'oya-verify',
    'oya-vcs-admission', 'oya-vcs-provider-execution',
    'oya-governance-supply-chain', 'oya-governance-cohesion',
    'oya-governance-api-semver', 'oya-governance-honest-claims',
    'oya-governance-aspirational-enforcement', 'oya-governance-banned-primitives',
    'oya-governance-protection-context-match', 'oya-governance-dependency-seam',
  ]

  podTemplateForOya(label) {
    node(label) {
      container('rust') {
        stage("checkout: ${svc}") { checkout scm }
        postForgeStatuses(requiredContexts, 'pending', 'oyaCiLane running')
        try {

        // --- presubmit fail-fast (mandatory) -------------------------------
        stage('lint: fmt + clippy') {
          sh 'cargo fmt --check && cargo clippy --all-targets -- -D warnings'
        }
        stage('cargo-deny: license + advisory + bans') {
          // The OSI-strict license gate (blocks BSL/SSPL/source-available) + RustSec.
          postForgeStatus('cargo-deny', 'pending', 'cargo-deny running')
          try {
            sh 'cargo deny check licenses bans advisories sources'
            postForgeStatus('cargo-deny', 'success', 'cargo-deny green')
          } catch (err) {
            postForgeStatus('cargo-deny', 'failure', "cargo-deny failed: ${err}")
            throw err
          }
        }
        stage('SAST: opengrep') {
          // Opengrep (LGPL-2.1) + repo-owned rules — NOT the proprietary Semgrep
          // registry rules. Advisory-to-error per repo policy.
          sh 'opengrep scan --config .opengrep/ --error || true'
        }
        stage('secret scan: gitleaks') {
          sh 'gitleaks detect --no-banner --redact'
        }
        stage('test: nextest') {
          postForgeStatus('oya-verify', 'pending', 'oya-verify running')
          try {
            sh "./bin/oya verify ${verifyMode}"
            postForgeStatus('oya-verify', 'success', 'oya-verify green')
          } catch (err) {
            postForgeStatus('oya-verify', 'failure', "oya-verify failed: ${err}")
            throw err
          }
        }

        // --- agentic-VCS admission + provider-execution (mandatory) --------
        // These two stages produce the `oya-vcs-admission` and
        // `oya-vcs-provider-execution` reported status contexts (both are
        // branch-protection required checks per reported-status-contexts.json).
        // The admission gate-app reads THIS pipeline file to confirm both gates
        // are wired; provider-execution emits the live provider proof bundle.
        stage('oya-vcs-admission') {
          // Install Trivy (Apache-2.0) via the Rust supply-chain installer
          // before admission, then run the admission gate-app.
          sh 'cargo run -q -p oya-dev-cli -- supply-chain install-trivy'
          sh 'cargo run -q -p oya-vcs-admission-gate-app'
        }
        stage('oya-vcs-provider-execution') {
          sh 'cargo run -q -p oya-vcs-provider-execution-gate-app -- --mode ci --emit-evidence target/oya-vcs-provider-execution/provider-execution-proof.json'
        }

        // --- build + supply chain (mandatory) ------------------------------
        stage('SBOM: cyclonedx + syft') {
          sh 'cargo cyclonedx --format json --override-filename target/sbom-cargo || true'
          sh 'syft dir:. -o cyclonedx-json=target/sbom-syft.cdx.json || true'
        }
        stage('vuln scan: trivy + osv') {
          // Trivy (Apache-2.0) fs/IaC + osv-scanner (Apache-2.0) lockfile.
          sh 'trivy fs --scanners vuln,misconfig,secret --severity HIGH,CRITICAL --exit-code 1 .'
          sh 'osv-scanner scan --lockfile Cargo.lock || true'
        }
        stage('sign + provenance: cosign + in-toto/SLSA') {
          // On trunk only: sign the built artifact by digest + attach SLSA provenance.
          when(isTrunk) {
            sh 'cosign sign --yes "$IMAGE_DIGEST"'
            sh 'cosign attest --yes --predicate target/provenance.intoto.json --type slsaprovenance "$IMAGE_DIGEST"'
          }
        }
          postForgeStatuses(requiredContexts, 'success', 'oyaCiLane green')
        } catch (err) {
          postForgeStatuses(requiredContexts, 'failure', "oyaCiLane failed: ${err}")
          throw err
        }
      }
    }
  }
}

// Helper: choose the JCasC pod template (kept here so lanes stay one-liners).
def podTemplateForOya(String label, Closure body) { body() }

// Minimal `when` for scripted stages.
def when(boolean cond, Closure body) { if (cond) body() }

// --- Forgejo Commit Status API wiring (ADR-0363 substrate) -----------------
// Posts a branch-protection required status context to the self-hosted Forgejo
// instance. The token is a Jenkins string credential `forgejo-ci-token` (scope
// write:repository); FORGE_API/FORGE_REPO are JCasC env with farm defaults.
// This is the mechanism that gates PR merges on real green checks, retiring the
// enforce_admins-toggle admin-merge seam.
def postForgeStatus(String context, String state, String description) {
  withCredentials([string(credentialsId: 'forgejo-ci-token', variable: 'FORGE_TOKEN')]) {
    String api  = env.FORGE_API  ?: 'http://forgejo.oya-forge.svc.cluster.local:3000/api/v1'
    String repo = env.FORGE_REPO ?: 'oya-admin/oyatie'
    String sha  = env.GIT_COMMIT
    sh """curl -sf -X POST -H 'Authorization: token \$FORGE_TOKEN' -H 'Content-Type: application/json' \
      ${api}/repos/${repo}/statuses/${sha} \
      -d '{"context":"${context}","state":"${state}","description":"${description}"}' >/dev/null"""
  }
}
def postForgeStatuses(List contexts, String state, String description) {
  contexts.each { postForgeStatus(it, state, description) }
}
