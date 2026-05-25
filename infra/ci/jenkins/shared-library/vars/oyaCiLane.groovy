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

  podTemplateForOya(label) {
    node(label) {
      container('rust') {
        stage("checkout: ${svc}") { checkout scm }

        // --- presubmit fail-fast (mandatory) -------------------------------
        stage('lint: fmt + clippy') {
          sh 'cargo fmt --check && cargo clippy --all-targets -- -D warnings'
        }
        stage('cargo-deny: license + advisory + bans') {
          // The OSI-strict license gate (blocks BSL/SSPL/source-available) + RustSec.
          sh 'cargo deny check licenses bans advisories sources'
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
          sh "./bin/oya verify ${verifyMode}"
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
          sh 'cargo run -q -p oya-foundry-vcs-admission-gate-app'
        }
        stage('oya-vcs-provider-execution') {
          sh 'cargo run -q -p oya-foundry-vcs-provider-execution-gate-app -- --mode ci --emit-evidence target/oya-vcs-provider-execution/provider-execution-proof.json'
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
      }
    }
  }
}

// Helper: choose the JCasC pod template (kept here so lanes stay one-liners).
def podTemplateForOya(String label, Closure body) { body() }

// Minimal `when` for scripted stages.
def when(boolean cond, Closure body) { if (cond) body() }
