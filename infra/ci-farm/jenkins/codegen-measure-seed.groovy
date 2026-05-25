// Seed oya-ci-codegen: measures the shared-cache payoff for a REAL CODEGEN build
// (cargo build, not check) — the case where sccache wall-clock savings are large
// (codegen is expensive; check is cheap). Cold agent compiles + codegens
// oya-http-runtime-hyper-adapter (hyper/tokio/tower) and populates SeaweedFS;
// a fresh warm agent (clean target) re-builds, served from the shared cache.
// Wall-clock cold-vs-warm is the measured evidence (addresses the honest caveat
// that the earlier `cargo check` numbers understated the cache's value).
import org.jenkinsci.plugins.workflow.job.WorkflowJob
import org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition

def j = jenkins.model.Jenkins.get()
def name = "oya-ci-codegen"
if (j.getItem(name) != null) { j.getItem(name).delete() }
def job = j.createProject(WorkflowJob, name)

def shellScript = '''
set -eux
export HOME="$WORKSPACE"
export CARGO_HOME="$WORKSPACE/.cargo"
git config --global --add safe.directory /workspace-src
if [ ! -x /tmp/scc/sccache ]; then
  curl -fsSL -o /tmp/sccache.tgz https://github.com/mozilla/sccache/releases/download/v0.8.2/sccache-v0.8.2-aarch64-unknown-linux-musl.tar.gz
  mkdir -p /tmp/scc && tar -xzf /tmp/sccache.tgz -C /tmp/scc --strip-components=1
fi
export PATH=/tmp/scc:$PATH
WORK="$WORKSPACE/repo"; rm -rf "$WORK"; mkdir -p "$WORK"
git -C /workspace-src archive --format=tar HEAD | tar -x -C "$WORK"
cd "$WORK"; export CARGO_TARGET_DIR="$WORK/target"
sccache --start-server || true; sccache --zero-stats || true
T0=$(date +%s)
cargo build -p oya-http-runtime-hyper-adapter
T1=$(date +%s)
echo "CODEGEN agent=$(hostname) wall_seconds=$((T1-T0))"
sccache --show-stats | grep -E "Compile requests|Cache hits|Cache misses|Cache hits rate" || true
'''

def pipeline = """
node('oya-rust-build') {
  stage('cold codegen (agent 1 — build + populate SeaweedFS)') {
    timeout(time: 30, unit: 'MINUTES') { container('rust') { sh '''${shellScript}''' } }
  }
}
node('oya-rust-build') {
  stage('warm codegen (agent 2 — fresh pod, cache-served codegen)') {
    timeout(time: 20, unit: 'MINUTES') { container('rust') { sh '''${shellScript}''' } }
  }
}
"""
job.setDefinition(new CpsFlowDefinition(pipeline, true))
job.save()
job.scheduleBuild2(0)
println("seeded+triggered: " + name)
