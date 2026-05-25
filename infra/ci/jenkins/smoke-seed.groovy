// Seed the oya-ci-smoke pipeline used to validate the local CI farm end-to-end:
// it schedules a pod from the JCasC `oya-rust-ci` template and runs the rust
// toolchain on the ephemeral agent (proving controller -> cloud -> agent -> build).
//
// Apply via the script console (Manage Jenkins > Script Console) or:
//   curl -s -b cookies -u admin:$PW -H "$CRUMB" \
//        --data-urlencode "script@infra/ci/jenkins/smoke-seed.groovy" \
//        http://localhost:8080/scriptText
import org.jenkinsci.plugins.workflow.job.WorkflowJob
import org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition

def j = jenkins.model.Jenkins.get()
def name = "oya-ci-smoke"
if (j.getItem(name) != null) { j.getItem(name).delete() }
def job = j.createProject(WorkflowJob, name)
def pipeline = '''
node("oya-rust-ci") {
  container("rust") {
    sh 'rustc --version && cargo --version && echo CACHE_ENV SCCACHE_BUCKET=$SCCACHE_BUCKET SCCACHE_ENDPOINT=$SCCACHE_ENDPOINT'
  }
}
'''
job.setDefinition(new CpsFlowDefinition(pipeline, true))
job.save()
job.scheduleBuild2(0)
println("seeded+triggered: " + name)
