// Seed the oya-ci-smoke pipeline used to validate the local CI farm end-to-end:
// controller -> cloud -> agent -> Buck2 build graph.
import org.jenkinsci.plugins.workflow.job.WorkflowJob
import org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition

def j = jenkins.model.Jenkins.get()
def name = "oya-ci-smoke"
if (j.getItem(name) != null) { j.getItem(name).delete() }
def job = j.createProject(WorkflowJob, name)
def pipeline = '''
node("oya-rust-ci") {
  container("rust") {
    sh 'rustc --version && buck2 --version && buck2 uquery //:buck2-authority-policy-check && echo CACHE_ENV BUCK2_CLIENT_TTL=$BUCK2_CLIENT_TTL'
  }
}
'''
job.setDefinition(new CpsFlowDefinition(pipeline, true))
job.save()
job.scheduleBuild2(0)
println("seeded+triggered: " + name)
