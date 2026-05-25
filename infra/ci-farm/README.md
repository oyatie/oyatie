# Oyatie CI farm — local k3s realization

Local-cluster realization of `specs/ci-farm-substrate-canonical.json`
(EXE-CI-FARM-SUBSTRATE-CANONICAL / ADR-0349): a Jenkins controller (JCasC-only,
`numExecutors=0`) that runs every microservice lane on ephemeral Kubernetes
agents.

This stands the substrate up on a **single-node local cluster** (colima
`--kubernetes`, k3s) so the controller + JCasC + agent-cloud + cache-wiring
contract can be executed and observed. It is **not** the production farm.

## Run it

```bash
colima start --cpu 6 --memory 16 --disk 60 --kubernetes   # one-time
./infra/ci-farm/deploy-local.sh
kubectl -n oya-ci-jenkins port-forward svc/oya-jenkins 8080:8080
# open http://localhost:8080  (admin password printed by the deploy script)
```

Validate end-to-end: seed `jenkins/smoke-seed.groovy` via the Script Console — it
schedules a pod from the `oya-rust-ci` template and runs the rust toolchain on the
ephemeral agent. A captured green run is in `evidence/ci-farm-local/`.

Tear down: `helm -n oya-ci-jenkins uninstall oya-jenkins` (or `colima delete` to
remove the whole cluster).

## Local-vs-production deltas

The local profile faithfully reproduces the **contract** (controller behavior,
JCasC, ephemeral k8s agents, sccache→S3 env wiring) and intentionally omits the
**capacity/hardening layer** that only a real multi-node farm provides:

| Spec element | Production | Local profile |
|---|---|---|
| `runtimeClass` | `kata-cloud-hypervisor` (ADR-0338) | k3s default `runc` |
| node capacity | Karpenter elastic autoscaling (ADR-0198) | single colima node |
| agent image | cosign-required, digest-pinned `registry.oyatie.dev/ci/rust` | public `rust:1-bookworm` |
| remote cache | sccache→SeaweedFS S3, OpenBao-bound creds | env wired; SeaweedFS deployed separately |
| service exposure | ingress / LoadBalancer | `ClusterIP` + `port-forward` |

These deltas do not alter the controller/agent/JCasC contract — they are exactly
the items the spec's `non_claims` keep unproven until measured on real capacity.

## Honest status

- **Claimed when green:** a real Jenkins controller runs on local k8s under the
  JCasC config; the kubernetes agent cloud is configured.
- **NOT claimed:** measured build-farm throughput, cache hit-rate, autoscaling
  behavior, Kata isolation, or any CI SLO. Per spec `non_claims`, capacity
  figures remain design targets pending measured evidence on real capacity.
