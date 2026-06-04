// Seed oya-ci-parallel: proves Buck2 lane concurrency on separate ephemeral
// agents. Measures per-lane wall vs overall wall; no Cargo command is used.
import org.jenkinsci.plugins.workflow.job.WorkflowJob
import org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition

def j = jenkins.model.Jenkins.get()
def name = "oya-ci-parallel"
if (j.getItem(name) != null) { j.getItem(name).delete() }
def job = j.createProject(WorkflowJob, name)

def shellTemplate = '''
set -eux
git config --global --add safe.directory /workspace-src
WORK="$WORKSPACE/repo"; rm -rf "$WORK"; mkdir -p "$WORK"
git -C /workspace-src archive --format=tar HEAD | tar -x -C "$WORK"
cd "$WORK"
T0=$(date +%s)
buck2 build __TARGET__
T1=$(date +%s)
echo "LANE __TARGET__ agent=$(hostname) wall_seconds=$((T1-T0))"
buck2 status || true
'''

def targets = [
  "//cloud/cloud-iac/crates/oya-cloud-iac-domain:oya-cloud-iac-domain",
  "//oya/observability/crates/oya-cloud-observability-domain:oya-cloud-observability-domain",
  "//libs/oya-http-runtime-hyper-adapter:oya-http-runtime-hyper-adapter",
]
def branchBlocks = targets.collect { target ->
  def s = shellTemplate.replace("__TARGET__", target)
  "  '${target}': { node('oya-rust-parallel') { container('rust') { sh '''${s}''' } } }"
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
