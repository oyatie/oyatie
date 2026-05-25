// Root CI orchestrator (ADR-0361) — the canonical repo-wide gate that replaces
// the retired GitHub Actions workflows. Jenkins reports its consolidated status
// contexts (oya-verify, oya-supply-chain, oya-pr-review) to the PR; branch
// protection requires those (infra/branch-protection/dev.json).
//
// Per-microservice lanes live at microservices/<ms>/ci/Jenkinsfile and run in
// parallel on the farm (N-way, O1 affected-scope). This root pipeline runs the
// repo-wide governance gate + fans out to the affected lanes.
@Library('oya-jenkins-shared') _

pipeline {
  agent none
  options {
    timeout(time: 60, unit: 'MINUTES')
    disableConcurrentBuilds(abortPrevious: true) // merge-queue serializes trunk (ADR-0111)
  }
  stages {
    stage('repo-wide governance gate') {
      steps {
        // The whole oya gate suite + cargo mirror, affected-scoped on PR.
        oyaCiLane(service: 'repo')
      }
    }
    stage('affected per-microservice lanes') {
      steps {
        // Lanes are discovered + fanned out in parallel; each is a
        // microservices/<ms>/ci/Jenkinsfile invoking oyaCiLane(service: <ms>).
        echo 'fan-out handled by the multibranch lanes; see microservices/<ms>/ci/Jenkinsfile'
      }
    }
  }
  post {
    success { echo 'oya-verify: PASS' }
    failure { echo 'oya-verify: FAIL' }
  }
}
