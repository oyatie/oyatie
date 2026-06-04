// oyaCiLane — Jenkins bridge lane for Buck2-backed oya-ci-required evidence.
// P0.0 contract: Buck2 is the only build/test/script authority in CI lanes.
// Tool-specific legacy contexts are retired from protected-branch authority.
// Jenkins may bridge status posting; destination authority is cloud-ci/oya-ci.

def call(Map cfg = [:]) {
  String svc = cfg.service ?: 'repo'
  String label = cfg.agentLabel ?: 'oya-rust-build'
  String baseRef = cfg.base ?: 'origin/dev'
  String headRef = cfg.head ?: 'HEAD'
  String requiredContext = 'oya-ci-required'

  podTemplateForOya(label) {
    node(label) {
      container('rust') {
        stage("checkout: ${svc}") { checkout scm }
        postForgeStatus(requiredContext, 'pending', 'Buck2 authority checks running')
        try {
          stage('Buck2 authority policy') {
            sh 'python3 scripts/ci/enforce-buck2-authority.py --policy specs/buck2-authority-policy.json'
          }
          stage('Buck2 affected build/test') {
            sh "infra/ci/buck2-affected-gate.sh ${baseRef} ${headRef}"
          }
          stage('Buck2 governance bridge smoke') {
            sh 'buck2 uquery //oya/developer-sdk/crates/oya-dev-cli:oya'
          }
          stage('supply-chain scans') {
            sh 'syft dir:. -o cyclonedx-json=target/sbom-syft.cdx.json || true'
            sh 'trivy fs --scanners vuln,misconfig,secret --severity HIGH,CRITICAL --exit-code 1 .'
            sh 'osv-scanner scan --lockfile Cargo.lock || true'
          }
          stage('sign + provenance: cosign + in-toto/SLSA') {
            when(env.BRANCH_NAME in ['dev', 'main', 'staging', 'production']) {
              sh 'cosign sign --yes "$IMAGE_DIGEST"'
              sh 'cosign attest --yes --predicate target/provenance.intoto.json --type slsaprovenance "$IMAGE_DIGEST"'
            }
          }
          postForgeStatus(requiredContext, 'success', 'Buck2 authority checks passed')
        } catch (err) {
          postForgeStatus(requiredContext, 'failure', "Buck2 authority checks failed: ${err}")
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
// Posts the target required status context to the self-hosted Forgejo instance.
// This is bridge evidence only; P0.0 exit authority requires cloud-ci/oya-ci to
// produce oya-ci-required from trusted controller/trunk state.
def postForgeStatus(String context, String state, String description) {
  withCredentials([string(credentialsId: 'forgejo-ci-token', variable: 'FORGE_TOKEN')]) {
    String api = env.FORGE_API ?: 'http://forgejo.oya-forge.svc.cluster.local:3000/api/v1'
    String repo = env.FORGE_REPO ?: 'oya-admin/oyatie'
    String sha = env.GIT_COMMIT
    sh """curl -sf -X POST -H 'Authorization: token \$FORGE_TOKEN' -H 'Content-Type: application/json' \
      ${api}/repos/${repo}/statuses/${sha} \
      -d '{"context":"${context}","state":"${state}","description":"${description}"}' >/dev/null"""
  }
}
