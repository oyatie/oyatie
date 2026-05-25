// Seed the oya-ci-build lane: the REAL build lane that proves the farm can build
// the actual repo with the sccache->SeaweedFS remote cache.
//
// It runs on the `oya-rust-build` template (hostPath repo mount + S3 creds),
// takes a clean `git archive` checkout, installs sccache, and runs `cargo check`
// twice on a bounded leaf crate: run 1 populates the cache (S3 writes), run 2
// (after `cargo clean`) demonstrates cache hits served from SeaweedFS.
import org.jenkinsci.plugins.workflow.job.WorkflowJob
import org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition

def j = jenkins.model.Jenkins.get()
def name = "oya-ci-build"
if (j.getItem(name) != null) { j.getItem(name).delete() }
def job = j.createProject(WorkflowJob, name)
def pipeline = '''
node("oya-rust-build") {
  container("rust") {
    sh """
      set -eux
      # UID 1000 has no writable HOME in the stock rust image; point HOME + CARGO_HOME
      # at the writable agent workspace so git/cargo can write their config/caches.
      export HOME="\\$WORKSPACE"
      export CARGO_HOME="\\$WORKSPACE/.cargo"
      git config --global --add safe.directory /workspace-src
      SCC=0.8.2
      curl -fsSL -o /tmp/sccache.tgz \\
        https://github.com/mozilla/sccache/releases/download/v\\${SCC}/sccache-v\\${SCC}-aarch64-unknown-linux-musl.tar.gz
      mkdir -p /tmp/scc && tar -xzf /tmp/sccache.tgz -C /tmp/scc --strip-components=1
      export PATH=/tmp/scc:\\$PATH
      sccache --version
      WORK="\\$WORKSPACE/repo"
      rm -rf "\\$WORK"; mkdir -p "\\$WORK"
      git -C /workspace-src archive --format=tar HEAD | tar -x -C "\\$WORK"
      cd "\\$WORK"
      export CARGO_TARGET_DIR="\\$WORK/target"
      CRATE=oya-cloud-iac-domain
      echo '=== run 1: populate cache ==='
      sccache --start-server || true
      sccache --zero-stats || true
      cargo check -p "\\$CRATE"
      sccache --show-stats
      echo '=== run 2: clean target, expect cache hits from SeaweedFS ==='
      cargo clean
      sccache --zero-stats || true
      cargo check -p "\\$CRATE"
      sccache --show-stats
    """
  }
}
'''
job.setDefinition(new CpsFlowDefinition(pipeline, true))
job.save()
job.scheduleBuild2(0)
println("seeded+triggered: " + name)
