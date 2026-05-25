// Seed oya-ci-cache-measure: MEASURES the CI-farm headline claim — shared build
// cache lets the substrate compile once and reuse across lane agents.
//
// Two separate node() blocks => two distinct ephemeral agent pods. Both compile
// the SAME crate (oya-http-runtime-hyper-adapter, pulls hyper/tokio/tower). The
// COLD agent populates SeaweedFS; the WARM agent (fresh pod, never compiled
// anything) can only win via the shared S3 cache. Wall-clock + hit-rate are the
// measured evidence (not asserted).
import org.jenkinsci.plugins.workflow.job.WorkflowJob
import org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition

def j = jenkins.model.Jenkins.get()
def name = "oya-ci-cache-measure"
if (j.getItem(name) != null) { j.getItem(name).delete() }
def job = j.createProject(WorkflowJob, name)

// single-quoted groovy string => $VARS are literal shell vars (no escaping).
def shellScript = '''
set -eux
export HOME="$WORKSPACE"
export CARGO_HOME="$WORKSPACE/.cargo"
git config --global --add safe.directory /workspace-src
SCC=0.8.2
if ! command -v sccache >/dev/null 2>&1 && [ ! -x /tmp/scc/sccache ]; then
  curl -fsSL -o /tmp/sccache.tgz https://github.com/mozilla/sccache/releases/download/v$SCC/sccache-v$SCC-aarch64-unknown-linux-musl.tar.gz
  mkdir -p /tmp/scc && tar -xzf /tmp/sccache.tgz -C /tmp/scc --strip-components=1
fi
export PATH=/tmp/scc:$PATH
WORK="$WORKSPACE/repo"
rm -rf "$WORK"; mkdir -p "$WORK"
git -C /workspace-src archive --format=tar HEAD | tar -x -C "$WORK"
cd "$WORK"
export CARGO_TARGET_DIR="$WORK/target"
sccache --start-server || true
sccache --zero-stats || true
T0=$(date +%s)
cargo check -p oya-http-runtime-hyper-adapter
T1=$(date +%s)
echo "MEASURE agent=$(hostname) wall_seconds=$((T1-T0))"
sccache --show-stats
'''

def pipeline = """
node('oya-rust-build') {
  stage('cold lane (agent 1 — populate SeaweedFS)') {
    container('rust') { sh '''${shellScript}''' }
  }
}
node('oya-rust-build') {
  stage('warm lane (agent 2 — fresh pod, cross-agent cache reuse)') {
    container('rust') { sh '''${shellScript}''' }
  }
}
"""
job.setDefinition(new CpsFlowDefinition(pipeline, true))
job.save()
job.scheduleBuild2(0)
println("seeded+triggered: " + name)
