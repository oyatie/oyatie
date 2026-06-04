// Seed oya-ci-farmwide: measures Buck2 graph build reuse across fresh Jenkins
// agents. Buck2 is the execution authority; no Cargo command is used.
import org.jenkinsci.plugins.workflow.job.WorkflowJob
import org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition

def j = jenkins.model.Jenkins.get()
def name = "oya-ci-farmwide"
if (j.getItem(name) != null) { j.getItem(name).delete() }
def job = j.createProject(WorkflowJob, name)

def shellScript = '''
set -eux
git config --global --add safe.directory /workspace-src
WORK="$WORKSPACE/repo"
rm -rf "$WORK"; mkdir -p "$WORK"
git -C /workspace-src archive --format=tar HEAD | tar -x -C "$WORK"
cd "$WORK"
T0=$(date +%s)
buck2 build //oya/developer-sdk/crates/oya-dev-cli:oya
T1=$(date +%s)
echo "FARMWIDE agent=$(hostname) wall_seconds=$((T1-T0))"
buck2 status || true
'''

def pipeline = """
node('oya-rust-build') {
  stage('cold (agent 1 — Buck2 build, populate shared cache)') {
    timeout(time: 30, unit: 'MINUTES') { container('rust') { sh '''${shellScript}''' } }
  }
}
node('oya-rust-build') {
  stage('warm (agent 2 — fresh pod, Buck2 cache reuse)') {
    timeout(time: 20, unit: 'MINUTES') { container('rust') { sh '''${shellScript}''' } }
  }
}
"""
job.setDefinition(new CpsFlowDefinition(pipeline, true))
job.save()
job.scheduleBuild2(0)
println("seeded+triggered: " + name)
