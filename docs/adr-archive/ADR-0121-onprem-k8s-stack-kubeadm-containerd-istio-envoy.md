---
id: ADR-0121
status: Superseded
doc_status: published
date: 2026-05-16
supersedes: []
superseded_by: [ADR-0375]
related: [ADR-0009, ADR-0028, ADR-0043, ADR-0044, ADR-0117, ADR-0119, ADR-0120]
renumber_note: "Originally drafted as ADR-0119 in PR #19 (on-prem KR primary cell + OCI KR secondary cell bring-up, signed replacement of PR #16), merged 2026-05-16T08:57:20Z. Collided with ADR-0119-specs-flat-root-topology from PR #18 (merged 2026-05-16T08:53:49Z — 3m31s earlier). Per merge-order precedence and PRs-12-18 multispectrum review (evidence/debate/pr-18-F4-r1.json, pr-18-M1-r1.json), renumbered to ADR-0121 (ADR-0120 was already taken by rust-first-onprem-tooling)."
---

# ADR-0121: On-prem Kubernetes stack — vanilla kubeadm + containerd + Istio + Envoy

> **Status:** Superseded by [ADR-0375](ADR-0375-talos-capi-argocd-fleet-substrate.md)
> **Owner:** `axis-cloud` + `axis-foundry`
> **Date:** 2026-05-16
> **Decision driver:** user directive 2026-05-16 ("istio + envoy + kubeadm + containerd"), ADR-0043 production-cell hardening, ADR-0117 OKE parity

---

## Context

The on-prem host (KR primary cell per ADR-0043) needs a Kubernetes control plane to host non-trivial Foundry workloads, host the Istio service mesh, and act as a federable peer to the OCI OKE clusters that arrive in M03 / M04.

During stage-0 bring-up (2026-05-16), three options were considered:

1. **k3s** — Single-binary lightweight distribution (Rancher). ~70 MB RAM idle.
2. **k0s** — Mirantis equivalent of k3s. ~80 MB RAM idle.
3. **kubeadm + containerd** — Vanilla upstream Kubernetes. ~250 MB RAM idle. **Same code OKE worker nodes run.**

k3s was installed transiently and then removed. The user directed adopting **kubeadm + containerd + Istio + Envoy** as the on-prem stack going forward.

---

## Decision

The on-prem Kubernetes stack on the KR primary cell is:

| Layer | Component | Why |
|---|---|---|
| Container runtime | **containerd** (CRI) | Same runtime OCI OKE uses; canonical upstream choice. Aligns with ADR-0117. |
| Kubernetes distribution | **kubeadm** (vanilla upstream) | Maximum OKE-parity; no Rancher-specific bits to unwind at M03 promotion; CNCF conformance by construction. |
| Service mesh control plane | **Istio** (minimal profile initially) | Per ADR-0044 service-mesh + mTLS decision; canonical Envoy operator with strong ecosystem. |
| Service mesh data plane | **Envoy** (Istio sidecars + ingress gateway) | Pulled forward from M03 (per ADR-0044 §timeline) so on-prem Foundry traffic immediately benefits from L7 access logs + mTLS hooks; portable to OCI OKE unchanged. |

k3s and k0s are **explicitly rejected** for this cell. They remain valid options for **edge cells** (M07-onward Industrial Platform KR satellite cells, retail kiosks) where RAM is constrained and full-upstream parity is not required; future ADRs may sanction k3s/k0s for those classes.

---

## Rationale

- **OKE parity (highest weight).** The KR primary cell handles regulated tenant data per ADR-0043. When M03 OKE clusters come up, workloads MUST be portable without re-validation. kubeadm runs the same control-plane code as OKE; k3s does not (different etcd backend by default, different CNI, different ingress).
- **Service-mesh first.** Even with only 2-3 services on the on-prem host today, Istio's access logs + telemetry hooks + per-service mTLS posture give us cross-cutting observability that we'd otherwise hand-roll. Pulling Envoy forward into stage-0 amortizes the integration cost across the whole M03 fan-out.
- **Vanilla = audit-friendly.** Compliance lanes (KCminimum-shippable-tier for KR, FIPS 140-3 globally per ADR-0043) require explicit cryptographic-module provenance. Vanilla upstream Kubernetes + containerd + Istio components are individually documented in compliance crosswalks; k3s aggregates these in ways that make per-component validation harder.
- **Cost is acceptable.** ~250 MB RAM idle on a host with 32+ GB is irrelevant. Setup time (~10 min via kubeadm) is one-time.

---

## Consequences

### Required successor-IP

- `infra/onprem/k3s/install.sh` is **retired** (tombstone exit 64). `infra/onprem/kubeadm/install.sh` + `infra/onprem/containerd/install.sh` are the canonical install entrypoints.
- `infra/onprem/istio/install.sh` targets the kubeadm cluster (kubeconfig at `~/.kube/config`).
- Single-command bring-up: `sudo bash infra/onprem/setup.sh` (hardening + sanoid + reboots + foundry + openbao + podman + containerd + kubeadm + istio + diagnostics).
- Single-command diagnostics: `bash infra/onprem/diagnose.sh` (10-section GREEN/RED report; no sudo needed).
- Debian 13 (trixie) gotcha: `setup.sh` pins iptables-legacy and flushes nftables ruleset before kubeadm init — k8s 1.35's kube-proxy nftables mode segfaults on this kernel, and orphan nft rules from prior attempts break pod-to-pod traffic.
- Per ADR-0044, mTLS posture moves from `permissive` to `strict` after first cross-cell traffic is observed.
- The OCI side: OKE remains the target (per ADR-0117) for cloud cells. The on-prem cell and OKE cells form a federation; workloads should be portable.
- M02b-substrate-P22 exit-gate spec line "mTLS Istio between services — scheduled-for-distinct-tracked-work to M03 per ADR-0117 §1" is now **partially closed** for the on-prem cell (Istio installed, mTLS configurable). M03 work remains for the OKE side and cross-cluster mesh.

### Rejected for the primary cell, accepted for edge

- k3s is **acceptable** for future edge cells (M07-onward retail/industrial constrained-RAM deployments) and for ephemeral developer environments. New ADRs may sanction k3s for those classes — explicitly NOT for the primary cell.
- k0s, MicroK8s, Talos Linux are also rejected for the primary cell. Same OKE-parity argument.

### Migration triggers

If the on-prem primary cell graduates to multi-node HA (planned for M04), the same kubeadm cluster grows via `kubeadm join`. No distribution swap required. Adding the OCI OKE cluster as a federation peer happens in M03 via Istio multi-cluster install.

---

## Test plan

- `kubectl version --output=json` returns `clientVersion.gitVersion == serverVersion.gitVersion` and both align with the published OKE upstream version pin.
- `istioctl version --remote` shows the in-cluster Istio control-plane.
- `kubectl get pods -A` shows etcd, kube-apiserver, kube-controller-manager, kube-scheduler, kube-proxy, CoreDNS, the CNI pods, and Istio-system pods all `Running`.
- `kubectl create namespace smoke && kubectl label namespace smoke istio-injection=enabled && kubectl run smoke --image=nginxdemos/hello -n smoke && kubectl wait pod/smoke -n smoke --for=condition=Ready --timeout=120s` — single-pod smoke with sidecar.
- Audit: `containerd --version`, `runc --version`, and `cri-tools` versions captured in the M02b-substrate evidence trail.

---

## Security posture (per user directive 2026-05-16: long-term-best-fit at scale)

| Concern | Tool | Why long-term |
|---|---|---|
| Auto-patching (Debian security archive only) | `unattended-upgrades` | Debian-mainline since 2008; what every Debian-fleet operates with. |
| Debian package CVE tracking | `debsecan` | Debian-mainline; CVE-DB driven; no upstream lock-in. |
| Repo + agent-state + transient secret scanning | `gitleaks` | OWASP-adjacent; ~17k stars; pre-commit + CI mode; runs anywhere. |
| Filesystem + image + k8s CVE | `trivy` | Aqua Security; GitHub Advanced Security integration; Argo CD / OKE-native. |
| Rust workspace advisories | `cargo audit` | RustSec official; canonical Rust path. |
| Scan cadence | systemd timer (Sun 02:30) | Mirrors cleanup + restart timers; predictable. |
| Redaction | `scan.sh` sed filter | All output is redacted before write; never raw secrets in /var/log. |
| Scope | repo + `~/.claude` + `~/.codex` + `~/.cursor` + `~/.vscode` + `/tmp` + `/var/tmp` + `/var/log` + `~/.{ssh,oci,aws,kube,docker,gnupg,git-credentials,netrc}` + shell histories | Per user directive: agent state + transient + auth + histories. |
| Permission audit | bash `stat`/`find` | Auth files MUST be `0600`/`0700`; warns otherwise. |
| Audit chain | `/srv/oyatie/audit-chain/security-scan-events.jsonl` | ZFS-backed append-only; mirrors OpenBao audit pattern. |

The same toolchain is portable to OCI workloads in M03 (trivy + gitleaks run identically against OKE clusters; debsecan replaced by `dnf updateinfo` or equivalent on Oracle Linux nodes).

## Version pins (researched 2026-05-16; track LTS / current-supported)

| Component | Version | Source / rationale |
|---|---|---|
| containerd | **2.3.0** | First annual LTS, released 2026-04-30; minor cadence Apr/Aug/Dec. |
| runc | **1.4.0** | Current stable; 1.2.z EOL ~end Apr 2026. |
| CNI plugins | **1.6.0** | Current; nftables ipmasq + portmap. |
| Kubernetes (kubeadm) | **1.35** | N-1; supported window 1.36/1.35/1.34 per upstream N-2 policy. |
| Istio (control + Envoy) | **1.29.2** | Current supported; 1.29 released 2026-02-16; 1.27 EOL 2026-04-30. |

Bump these defaults via env vars (e.g., `K8S_VERSION=1.36` in `kubeadm/install.sh`). Each bump should land its own ADR amendment if it crosses a major.

## Sources scanned

- 2026-05-16 — user directive ("istio + envoy + kubeadm + containerd") + decision-context inheritance from ADR-0009 (cell architecture), ADR-0028 (cloud microservice architecture), ADR-0043 (OpenBao + per-cell HSM), ADR-0044 (service mesh), ADR-0117 (Bominal OCI A1 → OKE).
- Kubernetes upstream release branches (`kubernetes.io/releases`) — N-2 support policy.
- containerd 2.x LTS cadence (`containerd.io/releases`).
- Istio supported-releases matrix (`istio.io/latest/docs/releases/supported-releases/`).
- CNCF k8s conformance test reports for kubeadm vs k3s.
