# Oyatie CI — the CI, deployed locally for now

The Oyatie CI (`specs/ci-farm-substrate-canonical.json` / ADR-0349 / ADR-0361):
a Jenkins controller (JCasC-only, `numExecutors=0`) that runs every microservice
lane on ephemeral Kubernetes agents, with the full supply-chain stack
(cargo-deny → SAST → secret-scan → tests → SBOM → Trivy → cosign/SLSA → Kyverno
→ ArgoCD/Rollouts).

**There is one CI, not a "local" one and a "production" one.** It runs now on a
**single local node** (colima `--kubernetes`, k3s) — this *is* the CI, deployed
locally; the only gap to full production posture is **scale/HA** (multi-node
elasticity, Kata isolation, HA storage), not a different system. The pipeline and
supply-chain stages are identical at scale.

## Run it

```bash
colima start --cpu 6 --memory 16 --disk 60 --kubernetes   # one-time
./infra/ci/deploy-local.sh
kubectl -n oya-ci-jenkins port-forward svc/oya-jenkins 8080:8080
# open http://localhost:8080  (admin password printed by the deploy script)
```

Validate end-to-end: seed `jenkins/smoke-seed.groovy` via the Script Console — it
schedules a pod from the `oya-rust-ci` template and runs the rust toolchain on the
ephemeral agent. A captured green run is in `evidence/ci/`.

Components:
- `jenkins/` — controller values + smoke + real build-lane seeds.
- `seaweedfs/seaweedfs-local.yaml` — SeaweedFS S3, the sccache remote-cache backend
  (`seaweedfs-s3:8333`, bucket `oya-ci-sccache-shared-prod`).
- `argocd/README.md` — ArgoCD (CD half; progressive delivery per ADR-0349).

Real build lane (`jenkins/build-lane-seed.groovy`, template `oya-rust-build`):
clones a clean tree from the repo (hostPath, RO), installs sccache, and runs
`cargo check` twice — proving the sccache→SeaweedFS cache (run 1 populates, run 2
after `cargo clean` is a 100% cache hit served from SeaweedFS). See
`evidence/ci/abc-execution-evidence.txt`.

Tear down: `helm -n oya-ci-jenkins uninstall oya-jenkins` (or `colima delete` to
remove the whole cluster).

## Single-node-now vs at-scale deltas (same CI, scale is the only gap)

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

## Measured evidence (`evidence/ci/`)

- `cross-agent-cache-measure.txt` — fresh agent: 100% cache hit (38/38) from SeaweedFS.
- `farmwide-cache.txt` — 150-crate `oya-dev-cli` graph: 100% cross-agent hit.
- `parallel-lanes.txt` — 3 lanes, peak 3 concurrent agents, ~3× over serial.
- `argo-rollouts-canary.txt` — canary `25%→analysis→50%→75%→promote`.
- `argocd-gitops-sync.txt` — ArgoCD synced from in-cluster git; self-heal in 6s.

## Honest status

- **Claimed when green:** a real Jenkins controller runs on local k8s under the
  JCasC config; the kubernetes agent cloud is configured.
- **NOT claimed:** measured build-farm throughput, cache hit-rate, autoscaling
  behavior, Kata isolation, or any CI SLO. Per spec `non_claims`, capacity
  figures remain design targets pending measured evidence on real capacity.
