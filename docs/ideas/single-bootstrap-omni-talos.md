> **SUPERSEDED 2026-05-27 by [ADR-0375](../decisions/ADR-0375-talos-capi-argocd-fleet-substrate.md)** — the same evaluation cycle landed on Talos + Cluster API + Argo CD (vendor-neutral OSS, CNCF-standard). The Omni-Managed recommendation below is preserved as historical record only; do not act on it.


# Single-Bootstrap Omni-Managed Talos Platform

> Status: direction locked 2026-05-26 (via /idea-refine). Build pending.
> Related: ADR-0148 (mesh), ADR-0147/0338 (Kata runtime tiers), ADR-0339 (IaC module library),
> ADR-0370/0371 (Talos substrate), `talos-substrate-doctrine` memory.

## Problem Statement
How might we bring up the entire platform — Omni + an HA Talos control-plane cluster it *manages* +
GitOps app delivery — from **one operator command**, everything expressed as IaC, on a single local
KVM/libvirt host?

## Recommended Direction: Omni-Managed
No Omni Terraform provider exists (siderolabs/omni#53 open; registries 404), so the cluster lifecycle
cannot live in OpenTofu. The requirement "Omni is the mastermind / especially omni, single bootstrap"
decides the fork: **Omni owns the cluster lifecycle via `omnictl cluster template sync`** (native,
backward-compat-guaranteed declarative IaC). Hyperscaler managed-control-plane shape: Omni = the
provisioner (GKE-control-plane analogue); clusters are declarative git templates; machines self-register
via SideroLink.

"Everything IaC" = three engines, each owning one layer:

| Layer | Engine | Owns |
|---|---|---|
| Infra | OpenTofu + Helm | Omni stack (docker provider), libvirt NAT net + SideroLink-booting VMs, Cloudflare tunnel/access |
| Cluster | omnictl cluster template (git YAML) | Omni-managed cluster: 3 dedicated CPs, `cni:none`, certSANs, `kubernetes.manifests` → Cilium 1.19.4 + Argo CD |
| Apps | Argo CD + declarative app-of-apps | app-of-apps GitOps (all in-cluster): **GitHub** `infra/gitops/vcs-substrate.yaml` (SCM + merge gate; infra/forge consolidated into infra/gitops per ADR-0515 D3), **ci** + signed agents + SeaweedFS + cargo mirror `infra/ci/` (CI), **OpenBao** `infra/kms/` (KMS), **observability** `microservices/observability/`, **Kyverno** (admission + cosign verify), **Istio Ambient** (L7, with worker pools), workloads |

**Substrate trio (ADR-0363: git + ci + GitHub)** lives entirely inside the Omni-managed cluster
as Argo CD apps. ci = CI (build/test → posts required-status contexts to GitHub's merge gate);
GitHub = SCM + merge gate; distinct from the Argo CD CD loop. The single
bootstrap only stands up **Omni → cluster → Cilium → Argo CD**; Argo CD pulls everything else.

**GitOps source of truth:** GitHub (`github.com/jason931225/oyatie`) **at bootstrap**; GitHub
(in-cluster, Argo CD-managed workload per `infra/gitops/vcs-argocd-app.yaml`) becomes primary only
after a deliberate post-bootstrap cutover (ADR-0247). This avoids the "Argo CD needs GitHub which
needs Argo CD" deadlock — GitHub is just another managed app; GitHub bootstraps the loop. GitHub's
primary role is the CI merge-gate substrate (ci required-status → merge gate, ADR-0363), not the
CD loop itself.
| Orchestration | one bootstrap entrypoint | sequences all; wraps the 2 imperative seams |

Single-bootstrap sequence: host-prep (libvirt/qemu, sudo) → `tofu apply` Omni stack → `omnictl
serviceaccount` + `omnictl download` SideroLink qcow2 → `tofu apply` VMs → wait for machine
registration → `omnictl cluster template sync` → Omni provisions cluster + applies Cilium + Argo CD →
Argo CD reconciles the rest. Operator runs ONE command.

## Decisions Locked (2026-05-26)
- Omni-managed (omnictl cluster templates), replacing the direct talos-provider bootstrap.
- Omni stack folded into OpenTofu via the `kreuzwerker/docker` provider.
- Cilium bumped to 1.19.4 (ADR-0148's 1.16 LTS is EOL — amended).

## Keep vs Replace (this session's build)
- Keep: libvirt VM provisioning (refactor `talos-cluster` module → SideroLink-booting nodes),
  `cilium-values.yaml` (→ the Cilium manifest in the template), `infra/cloudflare/omni/`.
- Replace: talos-provider bootstrap (`talos_machine_secrets/configuration/apply/bootstrap/kubeconfig`
  + `helm_release cilium`) → moves into the Omni cluster template.

## Endpoint naming (all Cloudflare, no Tailscale)
| Hostname | Who connects | Path / protocol | Cloudflare Access? |
|---|---|---|---|
| `omni.oyatie.dev` | humans (browser) | Omni UI/API → loopback :443 | yes (OTP) |
| `dex.oyatie.dev` | OIDC discovery | Dex → loopback :5556 | no (issuer creds) |
| `join.oyatie.dev` | Talos machines | SideroLink machine-join, WireGuard-over-gRPC tunnel (`--siderolink-use-grpc-tunnel`) → loopback :443 | no (join-token) |
| `k8s.oyatie.dev` | operators' kubectl | Kubernetes API via Omni k8s-proxy :8100 | yes / service-token |
| `127.0.0.1:443` | omnictl / automation | loopback (bypasses Cloudflare Access) | n/a |

Local libvirt VMs may also reach the machine plane directly on the libvirt bridge `10.42.0.1`
(faster, no Cloudflare round-trip); `join.oyatie.dev` is the path for off-host/remote machines. Tailscale
is fully retired from the critical path.

## Node-Pool Topology (what runs where)
A dedicated CP tier cannot host the platform stack — so the cluster is CP tier **+ worker pools**.

| Node group | Runtime / image | Runs |
|---|---|---|
| **control-plane** (3, dedicated, tainted) | vanilla Talos, runc | apiserver/etcd/scheduler/controller-manager + tolerating DaemonSets only (Cilium agent, node-exporter/otel DaemonSet, CSI node). Nothing else. |
| **worker `system`** (trusted) | vanilla, runc | Argo CD, GitHub, ci controller, OpenBao, observability backends, Kyverno, cert-manager, ingress/Gateway, istiod |
| **worker `ci`** (semi-trusted) | vanilla (Kata for untrusted PR exec) | ci agents (ephemeral) |
| **worker `tenant`** (untrusted) | Kata/CLH baked image, nested virt, `kata=enabled:NoSchedule` | tenant workloads (Tier-3, ADR-0147/0338) + ztunnel + waypoints |

Cilium runs on every node; ztunnel (Ambient) only on workload nodes. The Omni cluster template
declares ControlPlane + one `Workers` doc per pool (Kata extension/labels/taints via per-pool patches).

### Capacity reality (single 30 GiB host)
The full platform does NOT fit on one 30 GiB box (3 HA CPs ~9 GiB + observability 8–16 GiB +
GitHub/ci/OpenBao/ArgoCD). The **topology** is faithful; **capacity** is not. On one host run a
scaled-down shape (e.g. 3 small CPs + 1 system worker, minimal replicas, trimmed observability) and
grow horizontally by having **Omni add nodes/clusters** to the fleet. DECISION OPEN: scaled-down-shape
vs fewer-CPs-to-fit-more.

## Key Assumptions to Validate
- [ ] Mixed IaC engines (OpenTofu + omnictl-YAML + Argo CD) is acceptable as "everything IaC."
- [ ] `omnictl download` (authenticated, not a static URL) as a scripted pre-step is an acceptable seam.
- [ ] Cilium-then-ArgoCD via the template's `kubernetes.manifests` brings nodes Ready cleanly.

## Not Doing (and why)
- No Omni TF / community provider — doesn't exist / pre-alpha; unsafe.
- No direct talos-provider bootstrap — conflicts with Omni-managed (dual etcd control paths).
- No Kyverno/Istio Ambient yet — no workloads on a CP-only tier; Argo CD adds them with worker pools.
- No multi-host HA yet — single host is a known SPOF; real HA = more hosts later.

## Open Questions
- Where do the SideroLink media + Omni service-account key get stored (gitignored secrets/ vs OpenBao)?
- Does the bootstrap re-run idempotently (template sync is declarative; VM re-create is not)?
