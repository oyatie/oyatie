// Seed oya-ci-parallel: proves the CONCURRENCY half of the farm — N per-
// microservice lanes run at once on separate ephemeral agents, each pulling
// shared deps from the SeaweedFS cache. Measures per-lane wall vs OVERALL wall:
// overall ~= max(lane), not sum(lane), is the parallelism evidence.
//
// Uses the `oya-rust-parallel` template (1 CPU request) so 3 lanes co-schedule
// on the single local node; production scales lanes with Karpenter (cap, not pool).
import org.jenkinsci.plugins.workflow.job.WorkflowJob
import org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition

def j = jenkins.model.Jenkins.get()
def name = "oya-ci-parallel"
if (j.getItem(name) != null) { j.getItem(name).delete() }
def job = j.createProject(WorkflowJob, name)

// single-quoted => literal $ (shell vars). __CRATE__ is substituted per lane.
def shellTemplate = '''
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
cargo check -p __CRATE__
T1=$(date +%s)
echo "LANE __CRATE__ agent=$(hostname) wall_seconds=$((T1-T0))"
sccache --show-stats | grep -E "Cache hits|Cache misses|Cache hits rate" || true
'''

def crates = ["oya-cloud-iac-domain", "oya-cloud-observability-domain", "oya-http-runtime-hyper-adapter"]
def branchBlocks = crates.collect { c ->
  def s = shellTemplate.replace("__CRATE__", c)
  "  '${c}': { node('oya-rust-parallel') { container('rust') { sh '''${s}''' } } }"
}.join(",\n")

def pipeline = """
def t0 = System.currentTimeMillis()
parallel([
${branchBlocks}
])
echo 'OVERALL_WALL_SECONDS=' + ((System.currentTimeMillis() - t0) / 1000)
"""
job.setDefinition(new CpsFlowDefinition(pipeline, true))
job.save()
job.scheduleBuild2(0)
println("seeded+triggered: " + name)
