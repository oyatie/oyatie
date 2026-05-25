// EXAMPLE thin per-microservice lane (ADR-0361). Existing inline lanes (e.g.
// microservices/cloud-iac/ci/Jenkinsfile) migrate to this one-liner form as the
// shared library's pod template + supply-chain stages subsume their inline copy.
//
// Drop this as microservices/<ms>/ci/Jenkinsfile:
@Library('oya-jenkins-shared') _

oyaCiLane(
  service: 'my-microservice',
  agentLabel: 'oya-rust-build',
  base: 'dev',
)
