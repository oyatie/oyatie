// Seed oya-ci-codegen: measures the shared-cache payoff for a real Buck2 build.
// Buck2 owns codegen execution; no Cargo command is used.
import org.jenkinsci.plugins.workflow.job.WorkflowJob
import org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition

def j = jenkins.model.Jenkins.get()
def name = "oya-ci-codegen"
if (j.getItem(name) != null) { j.getItem(name).delete() }
def job = j.createProject(WorkflowJob, name)

def shellScript = '''
set -eux
git config --global --add safe.directory /workspace-src
WORK="$WORKSPACE/repo"; rm -rf "$WORK"; mkdir -p "$WORK"
git -C /workspace-src archive --format=tar HEAD | tar -x -C "$WORK"
cd "$WORK"
T0=$(date +%s)
buck2 build //libs/oya-http-runtime-hyper-adapter:oya-http-runtime-hyper-adapter
T1=$(date +%s)
echo "CODEGEN agent=$(hostname) wall_seconds=$((T1-T0))"
buck2 status || true
'''

def pipeline = """
node('oya-rust-build') {
  stage('cold codegen (agent 1 — Buck2 build)') {
    timeout(time: 30, unit: 'MINUTES') { container('rust') { sh '''${shellScript}''' } }
  }
}
node('oya-rust-build') {
  stage('warm codegen (agent 2 — fresh pod, Buck2 cache reuse)') {
    timeout(time: 20, unit: 'MINUTES') { container('rust') { sh '''${shellScript}''' } }
  }
}
"""
job.setDefinition(new CpsFlowDefinition(pipeline, true))
job.save()
job.scheduleBuild2(0)
println("seeded+triggered: " + name)
