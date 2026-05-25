// Seed oya-ci-farmwide: scales the shared-cache proof to a SUBSTANTIAL transitive
// graph. `cargo check -p oya-dev-cli` (the verify toolchain) compiles a large
// slice of the workspace. Cold agent populates SeaweedFS; a fresh warm agent
// (clean target) shows the aggregate cross-agent cache-hit-rate + wall-clock
// speedup over many crates — farm-wide throughput evidence (measured, bounded).
import org.jenkinsci.plugins.workflow.job.WorkflowJob
import org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition

def j = jenkins.model.Jenkins.get()
def name = "oya-ci-farmwide"
if (j.getItem(name) != null) { j.getItem(name).delete() }
def job = j.createProject(WorkflowJob, name)

def shellScript = '''
set -eux
export HOME="$WORKSPACE"
export CARGO_HOME="$WORKSPACE/.cargo"
git config --global --add safe.directory /workspace-src
SCC=0.8.2
if [ ! -x /tmp/scc/sccache ]; then
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
cargo check -p oya-dev-cli --tests 2>&1 | tail -3
T1=$(date +%s)
echo "FARMWIDE agent=$(hostname) wall_seconds=$((T1-T0))"
sccache --show-stats
'''

def pipeline = """
node('oya-rust-build') {
  stage('cold (agent 1 — compile oya-dev-cli graph, populate SeaweedFS)') {
    timeout(time: 30, unit: 'MINUTES') { container('rust') { sh '''${shellScript}''' } }
  }
}
node('oya-rust-build') {
  stage('warm (agent 2 — fresh pod, cross-agent cache reuse over the graph)') {
    timeout(time: 20, unit: 'MINUTES') { container('rust') { sh '''${shellScript}''' } }
  }
}
"""
job.setDefinition(new CpsFlowDefinition(pipeline, true))
job.save()
job.scheduleBuild2(0)
println("seeded+triggered: " + name)
