---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: cloud-k8s
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-cloud + ops-security
deciders: council-architecture, ops-security, axis-cloud, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + NIST SP 800-154 + CIS Kubernetes Benchmark v1.9 + NSA/CISA Kubernetes Hardening Guide v1.2
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0121, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/hyperscaler-gates.json]
review_cadence: quarterly + on every Kubernetes / containerd / Istio / Envoy / Cilium version change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC6.7, CC6.8, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.18, A.5.23, A.5.26, A.5.31, A.5.32, A.5.33, A.8.2, A.8.3, A.8.5, A.8.7, A.8.8, A.8.11, A.8.20, A.8.21, A.8.22, A.8.23, A.8.24, A.8.25, A.8.26, A.8.27, A.8.28, A.8.31"
  - "GDPR Arts. 25, 28, 30, 32, 33"
  - "CIS Kubernetes Benchmark v1.9 (every control)"
  - "NSA/CISA Kubernetes Hardening Guide v1.2 (every recommendation)"
suggested_frameworks_by_pack:
  pack-kr: ["KR-ISMS-P §2.5 (인적보안), §2.6 (암호화), §2.7 (접근통제), §2.8 (운영)", "KR PIPA Art. 29 (technical safeguards)", "KR CSAP cloud-security certification controls"]
  pack-us-healthcare: ["HIPAA §164.308 (Administrative)", "§164.310 (Physical)", "§164.312 (Technical)", "§164.314 (Organizational)"]
  pack-eu: ["GDPR Arts. 25 + 32", "NIS2 2022/2555", "DORA 2022/2554"]
  pack-jp: ["APPI Arts. 20 (security control), 21 (employee/entrustee supervision)"]
doc_status: published
---

# Threat Model: cloud-k8s µservice

## Purpose

Identify, classify, and mitigate threats to the on-prem Kubernetes substrate's confidentiality, integrity, and availability posture. `cloud-k8s` hosts every other oyatie µservice — a compromise here cascades. This document is the canonical security artifact reviewed by SOC 2 Type 2 examiners, ISO 27001 auditors, KR-CSAP reviewers, and HIPAA OCR auditors at first-tenant onboarding in each jurisdiction.

## Scope

### In-scope

All components introduced by ADR-0121 (on-prem k8s stack) and ADR-0131 (per-microservice flat layout) for the cloud-k8s µservice:

| Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|
| kubeadm 1.35 (control-plane bootstrap) | `cloud-k8s-cluster-bootstrap-*` (10 crates) |
| containerd 2.3.0 LTS + runc 1.4.0 (CRI) | `cloud-k8s-node-lifecycle-*` (8 crates) |
| Cilium CNI 1.16 LTS (eBPF dataplane + NetworkPolicy + Hubble) | `cloud-k8s-network-policy-*` (8 crates) |
| Istio 1.29.2 (istiod control-plane) | `cloud-k8s-service-mesh-control-plane-*` (9 crates) |
| Envoy 1.32 (Istio sidecars + ingress gateway) | `cloud-k8s-ingress-controller-*` (9 crates) |
| CSI drivers (OCI Block Volume + Object Storage + File Storage; CephFS / Ceph RBD / SeaweedFS on-prem) | `cloud-k8s-csi-storage-driver-*` (11 crates) |
| kube-apiserver / kube-controller-manager / kube-scheduler / etcd | `cloud-k8s-kubernetes-api-proxy-*` (10 crates) |
| Cosign + Kyverno admission controller | (uses governance µservice supply-chain authority) |

### Out-of-scope

- Threats to the underlying compute / network / storage fabric — owned by the `cloud-iac` µservice's threat model. cloud-k8s inherits cloud-iac threats as upstream.
- Threats to workload µservices themselves — each owns its own threat-model.md.
- Threats to OpenBao / secrets — owned by `cloud-secrets` µservice's threat model.
- Threats to observability dataplane — owned by `observability` µservice's threat model (cluster sends telemetry there).
- Threats specific to Bominal-side cloud counterparts — separate Bominal threat-model.

## Trust Boundaries

```text
┌─ External (internet) ────────────────────────────────────────────────────────┐
│                                                                              │
│   Tenant API consumers           Tenant browsers           Public probes     │
│         │                              │                          │          │
│         │ (HTTPS + mTLS to mesh)       │ (HTTPS, OIDC)             │ (HTTPS) │
│         ▼                              ▼                          ▼          │
│  ┌─ Envoy public ingress (Istio Gateway) ────────────────────────────────┐   │
│  │  - TLS 1.3 termination                                                │   │
│  │  - SNI validation                                                     │   │
│  │  - WAF + rate-limit + OWASP CRS                                       │   │
│  │  - DDoS protection (provider + Cloudflare)                            │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                              │                                               │
└──────────────────────────────│───────────────────────────────────────────────┘
                               ▼
┌─ Per-pack Kubernetes cluster (one per pack) ─────────────────────────────────┐
│                                                                              │
│  Trust boundary 1: External → Ingress (Envoy/Istio Gateway)                  │
│                                                                              │
│  ┌─ Cilium L3/L4 + Istio Ambient L7 (per ADR-0148; no sidecars) ──────┐     │
│  │  - L3/L4: Cilium eBPF mTLS + identity-aware NetworkPolicy           │     │
│  │  - L7: Istio Ambient ztunnel (node-level) + per-namespace waypoints │     │
│  │  - mTLS strict via Cilium + Istio Ambient (no Envoy sidecar)        │     │
│  │  - per-namespace AuthorizationPolicy at waypoint (Cedar-derived)    │     │
│  │  - per-namespace NetworkPolicy via Cilium eBPF (Cedar-derived)      │     │
│  └─────────────────────────────────────────────────────────────────────┘     │
│                                                                              │
│  Trust boundary 2: Pod → kube-apiserver (mediated by api-proxy)              │
│                                                                              │
│  ┌─ kubernetes-api-proxy ──────────────────────────────────────────────┐     │
│  │  - HTTP reverse-proxy in front of kube-apiserver                    │     │
│  │  - Cedar policy decision on every call                              │     │
│  │  - audit-chain emission per call                                    │     │
│  │  - operators + Foundry agents + CI lanes route through here         │     │
│  │  - Direct port 6443 access REFUSED at NetworkPolicy                 │     │
│  └─────────────────────────────────────────────────────────────────────┘     │
│                              │                                               │
│  Trust boundary 3: kube-apiserver → etcd (mTLS strict)                       │
│                                                                              │
│  ┌─ etcd cluster (3-node Raft after M04 HA) ──────────────────────────┐      │
│  │  - peer + client mTLS strict                                       │      │
│  │  - encryption at rest (KMS-backed envelope encryption)             │      │
│  │  - snapshot cadence ≤ 5 min RPO                                    │      │
│  │  - backup retention per pack residency                             │      │
│  └────────────────────────────────────────────────────────────────────┘      │
│                                                                              │
│  Trust boundary 4: kubelet → containerd (CRI) → runc → workload pod          │
│                                                                              │
│  ┌─ Node (worker) ───────────────────────────────────────────────────┐       │
│  │  - containerd 2.3.0; seccomp runtime/default; AppArmor profiles   │       │
│  │  - runc 1.4.0                                                     │       │
│  │  - kubelet authn/authz strict; bootstrap-token TTL ≤ 24h          │       │
│  │  - host-path mounts forbidden by Kyverno admission                │       │
│  │  - privileged-by-default forbidden                                │       │
│  └───────────────────────────────────────────────────────────────────┘       │
│                                                                              │
│  Trust boundary 5: CNI (Cilium) → kernel eBPF → packet plane                 │
│                                                                              │
│  Trust boundary 6: CSI (per-backend) → backing storage                       │
│                                                                              │
│  Trust boundary 7: Inter-cluster (multi-cluster Istio federation; M03)       │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

Seven trust boundaries:
1. **External → Ingress** (Envoy Gateway TLS 1.3 termination + WAF).
2. **Pod → kube-apiserver** (mediated by `kubernetes-api-proxy`; no direct 6443).
3. **kube-apiserver → etcd** (mTLS strict + KMS-backed envelope encryption at rest).
4. **kubelet → containerd → runc → pod** (seccomp + AppArmor + non-privileged + host-path-forbidden).
5. **CNI dataplane** (Cilium eBPF; NetworkPolicy enforcement at kernel layer).
6. **CSI → backing storage** (per-backend driver; QoS-class-bound).
7. **Inter-cluster federation** (Istio multi-cluster mTLS; M03).

## Assets & Data Classification

Per Bominal ADR-0028 (audit-chain + data-class taxonomy) and the `check-data-class` LEAN lane.

| Asset | Class | Sensitivity | Retention | Authoritative store |
|---|---|---|---|---|
| Cluster control-plane state (etcd) | `INTERNAL_ONLY` + `AUDIT` (mutation records) | Critical | etcd in-cluster + 14d snapshot + Mimir audit log | etcd + audit-chain |
| Node attestation records (kubeadm join token, TLS certs) | `SECRET` | Critical | OpenBao; 24h TTL on join tokens | OpenBao |
| Kubernetes API audit log | `AUDIT` + `BEHAVIORAL_TENANT_PRODUCT` (pod-spec contents) | High | per pack: 6y (us-hc), 5y (kr), 2y default | Loki + audit-chain |
| etcd encryption key (envelope key for at-rest encryption) | `SECRET` | Critical | KMS-backed; 90d rotation | OpenBao + KMS |
| Istio mTLS root CA + intermediate CAs | `SECRET` | Critical | OpenBao with HSM-backed where available; 1y rotation | OpenBao |
| Envoy TLS cert (public ingress) | `SECRET` | High | OpenBao; 90d rotation (cert-manager managed) | OpenBao + cert-manager |
| Workload Pod specs (per-tenant; carry env vars / volume mounts) | `BEHAVIORAL_TENANT_PRODUCT` + sometimes `PII_IDENTIFYING` (env vars may leak user-ids) | Medium-High | retention per workload µservice | etcd + audit-chain |
| Persistent Volumes (per backend) | `BEHAVIORAL_TENANT_PRODUCT` (varies; some `PHI` in pack-us-healthcare) | High | per pack residency | backend (block / object / file) |
| NetworkPolicy / AuthorizationPolicy CRs | `INTERNAL_ONLY` (policy text); `BEHAVIORAL_TENANT_PRODUCT` (tenant identifiers in peer selectors) | Medium | git history + Mimir audit log | etcd + audit-chain |
| Cilium policy state + flow logs (Hubble) | `BEHAVIORAL_TENANT_PRODUCT` | Medium | Loki 7d hot + 30d cold | Loki |
| Istio xDS configuration (in-memory) | `INTERNAL_ONLY` | Low | transient | istiod memory |
| Container images (oyatie-owned) | `INTERNAL_ONLY` | Low | image registry; Cosign-signed | Harbor + Cosign attestation |
| Bootstrap kubeconfig (`/etc/kubernetes/admin.conf`) | `SECRET` | Critical | OpenBao; no on-disk persistence beyond bootstrap window | OpenBao |
| Foundry capability invocation records | `AUDIT` | High | append-only audit chain | audit-chain |

## Actors

| Actor | Trust level | Authentication | Capability |
|---|---|---|---|
| External tenant API consumer | Untrusted | OIDC + per-tenant API key (rotated 30d) | Read/write own tenant's resources via Envoy Gateway → Pod |
| External tenant browser user | Untrusted | OIDC + MFA | Same, plus admin UI per Cedar |
| Workload µservice in own cluster | Semi-trusted | SPIFFE SVID; mTLS via Cilium L3/L4 + Istio Ambient ztunnel/waypoint (no sidecar; per ADR-0148) | Pod-to-Pod within namespace; cross-namespace per AuthorizationPolicy at the waypoint |
| Operator (axis-cloud) | Trusted internal | OIDC + MFA + JIT via OpenBao | Cluster mutation through `kubernetes-api-proxy`; never direct 6443 |
| Foundry agent | Trusted internal | OIDC-bound + autonomy-ceiling | Cluster mutation via Foundry capability surface (`cloud-k8s.cluster.bootstrap`, etc.); audit-chain emit per call |
| CI runner | Semi-trusted internal | `WORKFLOW_PAT` + reserved Mimir tenant + namespace-scoped SA | Read-only on cluster + write to `ci` namespace |
| `cell` µservice | Trusted internal | SPIFFE SVID | Tenant cell-scheduling via Workflow events; namespace policy reads via Ontology |
| `observability` µservice | Trusted internal | SPIFFE SVID | Telemetry collection from kubelet / cAdvisor / Cilium Hubble |
| External auditor | Read-only external | OIDC + MFA + JIT short-lived | Read-only on cluster audit log + Cedar evaluation; cannot mutate |
| Attacker — opportunistic | Untrusted | none | Scans + low-skill exploitation; baseline assumption |
| Attacker — targeted | Untrusted | none | Sophisticated; supply-chain + kernel-exploit aware |
| Insider — accidental | Trusted internal | OIDC + MFA | Misconfigure CR; mitigated by PR review + admission webhook |
| Insider — malicious | Trusted internal | OIDC + MFA | Worst-case; mitigated by least-privilege + 2-person rule on admin ops + audit-chain |

## STRIDE Threat Catalog

Each threat carries: ID; category; asset; description; likelihood (L/M/H); impact (L/M/H); risk score; mitigations; owner; residual risk; framework controls satisfied.

### Spoofing (S)

**T-S-01 — Forged kubeadm join token used to register attacker node**
- Asset: Cluster membership; node attestation
- Likelihood: M / Impact: H (attacker node could host adversarial workloads alongside tenant pods) / Risk: **H**
- Mitigations:
  - kubeadm bootstrap-token TTL ≤ 24h with one-time-use enforcement.
  - Token issued via OpenBao with bound-node-fingerprint claim; kubelet TLS bootstrap validates fingerprint.
  - kubelet `--rotate-certificates` enabled; cert auto-rotation < 90d.
  - Node attestation: TPM / vTPM measurement quoted at first registration where hardware supports.
  - CIS K8s Benchmark control 4.2.6 (kubelet only-anonymous-auth-false) + 4.2.7 (--authorization-mode=Webhook) enforced.
- Owner: ops-security + axis-cloud
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2, CC6.6; ISO 27001 A.5.15, A.5.17, A.8.2, A.8.3, A.8.5; CIS K8s 4.2.x; NSA Kubernetes Hardening §"Node Authentication"

**T-S-02 — Spoofed Foundry agent identity invoking cluster mutation capabilities**
- Asset: Foundry capability invocation surface
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Every Foundry capability call carries SPIFFE SVID; kubernetes-api-proxy validates SVID against expected agent identity registry.
  - audit-chain emission per call includes the SVID + Cedar decision + autonomy-ceiling check outcome.
  - 2-person rule on autonomy-tier T3 calls (cluster bootstrap, control-plane upgrade, etcd restore).
- Owner: ops-security + axis-foundry
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6, CC7.1; ISO 27001 A.5.15, A.8.3, A.8.7

**T-S-03 — Attacker spoofs SNI for cross-tenant routing**
- Asset: Envoy ingress; tenant-A's external endpoint
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - SNI value validated against TLS cert SAN list; mismatch refused at Envoy.
  - VirtualService routing keyed on validated host header AND mTLS client cert (for service-to-service); host header alone never authoritative.
  - Per-tenant cert managed via cert-manager; private keys per OpenBao; no shared cert across tenants.
- Owner: axis-cloud + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.5, A.8.24

**T-S-04 — Sidecar Envoy proxy bypass (workload bypasses Istio mesh)**
- Asset: mTLS strict between pods
- Likelihood: M / Impact: H (cross-namespace cleartext) / Risk: **H**
- Mitigations:
  - PeerAuthentication `mode: STRICT` mesh-wide.
  - Cilium NetworkPolicy backs the mesh-level enforcement: even if a pod bypasses its sidecar, Cilium eBPF refuses direct east-west cleartext at kernel layer.
  - Kyverno admission webhook refuses Pod specs that disable sidecar injection (`sidecar.istio.io/inject: false`) except in operator-approved namespaces.
- Owner: axis-cloud + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.20, A.8.21, A.8.22; CIS K8s 5.x (Network Policies); NSA Hardening §"Network Separation"

**T-S-05 — Container image with spoofed Cosign signature**
- Asset: Supply chain (container images)
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Cosign signature verification against trusted public key (per-namespace ConfigMap; CODEOWNERS-bound).
  - Sigstore Rekor transparency log queried as second-factor verification.
  - Kyverno admission webhook refuses unsigned + mismatch-key images.
  - Per ADR-0117 §"Supply chain": Trivy + Grype CVE scan at admission; admission refuses Critical CVEs.
- Owner: axis-foundry + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6, CC6.8; ISO 27001 A.8.7, A.8.30; CIS K8s 1.x; NSA Hardening §"Supply Chain Security"

### Tampering (T)

**T-T-01 — etcd data tampering (direct write bypassing kube-apiserver)**
- Asset: etcd cluster
- Likelihood: L / Impact: H (cluster state forgery) / Risk: **M**
- Mitigations:
  - etcd peer + client mTLS strict; cert pool restricted to kube-apiserver service account + ops-security JIT.
  - etcdctl direct access requires 2-person rule + OpenBao JIT elevation; emits audit-chain record.
  - etcd auth enabled; per-component role; least-privilege.
  - Encryption at rest with KMS envelope; direct disk read gives ciphertext only.
- Owner: ops-security + axis-cloud
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6, CC8.1; ISO 27001 A.8.4, A.8.11, A.8.20, A.8.24; CIS K8s 2.x (etcd)

**T-T-02 — Container image tampering at registry**
- Asset: Harbor image registry
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Cosign signed digests; admission verifies digest + signature.
  - Harbor content trust enabled; immutable tags.
  - Registry storage WORM where supported.
- Owner: cloud-iac + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.8.7, A.8.31; CIS K8s 1.x

**T-T-03 — Tampering with Kubernetes admission webhook (Kyverno) config**
- Asset: Kyverno ClusterPolicy CRs
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Kyverno CRs git-versioned at `microservices/cloud-k8s/iac/kustomize/base/`; PR-reviewed.
  - kubernetes-api-proxy refuses direct Kyverno-CR mutations from non-operator principals.
  - LEAN check `check-kyverno-policy-conformance` validates CRs match git source.
- Owner: axis-cloud + ops-security
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.8.32; CIS K8s 1.x (Admission Controllers)

**T-T-04 — Tampering with Cilium NetworkPolicy / Istio AuthorizationPolicy CRs (security regression)**
- Asset: Per-namespace policy
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Per-namespace policies derived from Cedar fragments (PR-reviewed; CODEOWNERS-bound).
  - LEAN check `check-network-policy-conformance` validates deployed CR matches git source.
  - Continuous-state-validator CronJob compares live CR to git; drift = alert + auto-rollback.
- Owner: axis-cloud + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.8.20, A.8.21; CIS K8s 5.x

**T-T-05 — kube-apiserver audit-log tampering**
- Asset: API audit log
- Likelihood: L / Impact: H (loss of audit evidence) / Risk: **M**
- Mitigations:
  - kube-apiserver `--audit-log-path` AND forward to Loki via Alloy sidecar (defence-in-depth).
  - Audit log forwarded to audit-chain µservice for Ed25519 + Merkle seal; tamper detected by chain integrity check.
  - Local audit log on append-only filesystem; rotation signs each rotation with Ed25519.
- Owner: ops-security + audit-chain
- Residual: L
- Frameworks: SOC 2 CC6.6, CC7.4; ISO 27001 A.8.15, A.8.16; CIS K8s 1.2.18-1.2.24

**T-T-06 — etcd snapshot tampering during restore**
- Asset: etcd snapshot
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Snapshot files Ed25519-signed at creation by `cloud-k8s-cluster-bootstrap-worker`.
  - Restore primitive validates signature before applying.
  - Snapshot storage WORM where supported.
- Owner: axis-cloud + ops-security
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.8.13, A.8.16; CIS K8s 2.x

### Repudiation (R)

**T-R-01 — Operator denies cluster mutation authorship**
- Asset: kube-apiserver call attribution
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - kubernetes-api-proxy attributes every call to a SPIFFE SVID or OIDC subject; emits audit-chain record per call.
  - audit-chain Ed25519 + Merkle seal per Bominal ADR-0028.
  - JIT elevation records the operator's identity + reason + approver.
- Owner: ops-security + audit-chain
- Residual: L
- Frameworks: SOC 2 CC4.1, CC8.1; ISO 27001 A.5.27, A.5.28, A.8.15

**T-R-02 — Foundry agent denies capability invocation**
- Asset: Foundry capability invocation record
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - Per-capability invocation record carries SVID + autonomy-tier + Cedar decision + execution timestamp; Ed25519-sealed.
- Owner: axis-foundry + audit-chain
- Residual: L
- Frameworks: SOC 2 CC4.1; ISO 27001 A.5.27, A.8.15

### Information Disclosure (I)

**T-I-01 — etcd disk read exposes cluster state**
- Asset: etcd persistent volume
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - At-rest encryption with KMS envelope; direct disk read yields ciphertext only.
  - PV storage class enforces encryption-at-rest; CSI driver propagates.
- Owner: cloud-iac + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.5.33, A.8.11, A.8.24; GDPR Art. 32(1)(a); HIPAA §164.312(a)(2)(iv)

**T-I-02 — Cross-namespace pod-to-pod cleartext leaks tenant data**
- Asset: East-west pod traffic
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Istio PeerAuthentication `mode: STRICT` mesh-wide.
  - Cilium NetworkPolicy kernel-layer enforcement.
  - Defence-in-depth: T-S-04 mitigations.
- Owner: axis-cloud + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.8.20, A.8.21, A.8.22; CIS K8s 5.x; NSA Hardening §"Network Separation"

**T-I-03 — Envoy SNI sniffing reveals tenant identity to passive observer**
- Asset: TLS handshake metadata
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - ECH (Encrypted Client Hello) when supported by tenant browser; rolling enablement as ECH ecosystem matures.
  - Multi-tenant shared SNI front-door: marketing front-door uses a shared SNI; tenant identity is in the path (which is over TLS).
  - Cloudflare-side privacy proxies for tenants that opt-in.
- Owner: axis-cloud + council-privacy
- Residual: M (TLS protocol limitation)
- Frameworks: GDPR Art. 32 (state-of-the-art)

**T-I-04 — Container escape → host filesystem read**
- Asset: Worker node filesystem
- Likelihood: L (depends on kernel CVE) / Impact: H / Risk: **M**
- Mitigations:
  - seccomp `runtime/default` baseline; per-class AppArmor profiles.
  - runc 1.4.0 current; CVE-tracked.
  - Kyverno admission refuses `privileged: true` + `hostPID: true` + `hostNetwork: true` + `hostIPC: true` outside narrowly-permitted system namespaces.
  - HostPath mounts refused except for explicitly-listed system DaemonSets.
  - User namespaces enabled (k8s 1.30+) where pod-spec opts in.
  - gVisor / Kata Containers available as opt-in sandbox class.
- Owner: ops-security + axis-cloud
- Residual: M (kernel CVE residual is irreducible)
- Frameworks: SOC 2 CC6.1; ISO 27001 A.8.4, A.8.31; CIS K8s 5.2.x (Pod Security Standards); NSA Hardening §"Pod Security"

**T-I-05 — kubectl exec captures workload secrets**
- Asset: Running container memory + env vars
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - kubernetes-api-proxy refuses `pods/exec` to non-operator principals; operators require JIT elevation.
  - Cedar policy: every `pods/exec` requires `reason` field; audit-chain emits.
  - Per Kyverno: refuse `pods/exec` to namespaces tagged `production-tier`.
- Owner: ops-security
- Residual: L-M
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.15, A.8.2

**T-I-06 — CSI volume cross-tenant data leak (mistakenly remounted)**
- Asset: PV from tenant-A mounted in tenant-B's pod
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - PVC `claimRef` enforces 1:1 binding; CSI driver re-validates on attach.
  - Per-namespace StorageClass enforces tenant-namespace pin.
  - `reclaimPolicy: Delete` for non-persistent classes; explicit cleanup on PVC delete.
  - Block-volume backend uses per-tenant LUN ownership; mount refuses if owner mismatch.
- Owner: axis-cloud + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.11

**T-I-07 — Cilium Hubble flow log leaks tenant traffic metadata**
- Asset: Hubble flow log
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - Hubble enabled in default mode only; deep-inspection disabled by default (only enabled in incident-response JIT mode).
  - Hubble logs are `BEHAVIORAL_TENANT_PRODUCT`; per-tenant Cedar scope on read.
  - Cross-tenant aggregation forbidden by Cedar policy.
- Owner: axis-cloud + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.8.12, A.8.16

### Denial of Service (D)

**T-D-01 — Control-plane DoS (kube-apiserver flood)**
- Asset: kube-apiserver
- Likelihood: H / Impact: H / Risk: **H**
- Mitigations:
  - kubernetes-api-proxy applies per-principal rate limits before forwarding.
  - kube-apiserver `--max-requests-inflight` + `--max-mutating-requests-inflight` tuned per capacity-model.md.
  - Priority + Fairness (APF) enabled with per-FlowSchema reservations.
  - HPA on api-proxy replicas; min 3 / max 50.
- Owner: ops-sre-reliability + axis-cloud
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.6, A.8.14; CIS K8s 1.2.x (kube-apiserver flags)

**T-D-02 — etcd quorum loss (network partition or 2/3 etcd node failure)**
- Asset: etcd Raft quorum
- Likelihood: M (after M04 HA; until then single etcd = N/A but full-outage) / Impact: H / Risk: **H**
- Mitigations:
  - 3-node etcd quorum after M04 (single-node at M01 with documented downgrade per ADR-0121).
  - Cross-AZ placement; AZ-failure tolerance.
  - 5-min snapshot cadence; etcd restore primitive tested quarterly.
  - On 2/3 loss: cluster freezes (read-only); cluster-bootstrap worker initiates restore from latest snapshot.
- Owner: ops-sre-reliability + axis-cloud
- Residual: L (subsequent-to-M04-completion)
- Frameworks: SOC 2 CC7.1, CC9.1; ISO 27001 A.5.30, A.8.14

**T-D-03 — Node-network partition isolates worker pool**
- Asset: Pod scheduling
- Likelihood: M / Impact: M-H / Risk: **M-H**
- Mitigations:
  - Pod anti-affinity + topology-spread-constraints across AZ.
  - Cluster-autoscaler (M02) replaces lost nodes within scheduling-budget window.
  - PodDisruptionBudgets enforced per workload.
- Owner: ops-sre-reliability + axis-cloud
- Residual: M
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.14

**T-D-04 — CNI (Cilium) failure → pod-to-pod connectivity loss**
- Asset: Cilium agent on every node
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Cilium agent runs as DaemonSet with liveness probe.
  - Per-node redundancy: containerd holds last-known-good Cilium config; pod creation pauses (does not error) during agent restart.
  - Cilium operator + cilium-cli for ops debug; daily synthetic connectivity probe.
- Owner: axis-cloud
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.8.6, A.8.14

**T-D-05 — Istio control-plane outage (istiod unavailable)**
- Asset: istiod Pod
- Likelihood: L / Impact: M (data-plane survives; new config doesn't propagate) / Risk: **M**
- Mitigations:
  - istiod runs HA with ≥ 3 replicas.
  - Envoy sidecars cache last-known-good xDS config and continue serving traffic; mesh outage doesn't crash data plane.
  - Canary istiod upgrade with prior revision retained for rollback.
- Owner: axis-cloud
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.8.14

**T-D-06 — Ingress DDoS via Envoy public gateway**
- Asset: Envoy gateway
- Likelihood: H / Impact: H / Risk: **H**
- Mitigations:
  - DDoS protection at provider edge (Cloudflare / OCI shield).
  - Envoy rate-limit filter per-IP + per-tenant.
  - WAF (OWASP CRS) for HTTP-layer attacks.
  - Auto-scale ingress replicas; HPA on CPU + connection count.
  - Connection-tracking limits.
- Owner: ops-sre-reliability + axis-cloud
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.6; OWASP DoS

**T-D-07 — Resource exhaustion (per-tenant cluster-quota breach)**
- Asset: Cluster capacity
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - Per-namespace ResourceQuota + LimitRange.
  - Cluster-autoscaler (M02) scales within global capacity envelope.
  - Per-tenant cost-budget alerts (cloud-iac µservice).
- Owner: ops-sre-reliability
- Residual: M
- Frameworks: SOC 2 CC7.1, CC9.1

**T-D-08 — Image pull failure cascade (registry outage)**
- Asset: Harbor registry availability
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - imagePullPolicy: IfNotPresent for production-tier (no forced re-pull).
  - Harbor HA + replicated; pull-through cache for upstream images.
  - Mirrored registry per-pack region.
- Owner: cloud-iac + axis-cloud
- Residual: L

### Elevation of Privilege (E)

**T-E-01 — Pod escape via overly-permissive PodSecurityStandard**
- Asset: Worker node
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - PodSecurityStandard `restricted` enforced on tenant namespaces.
  - Kyverno admission refuses Pods that don't meet `restricted` profile.
  - User namespaces enabled where feature-gated.
- Owner: ops-security + axis-cloud
- Residual: L
- Frameworks: SOC 2 CC6.6; ISO 27001 A.8.4, A.8.7; CIS K8s 5.2.x

**T-E-02 — RBAC misconfiguration grants `cluster-admin` to wrong subject**
- Asset: Kubernetes RBAC
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - All RBAC managed via OpenTofu / Kustomize at `microservices/cloud-k8s/iac/`; PR-reviewed.
  - kubernetes-api-proxy adds Cedar layer ON TOP of RBAC; no operator gets cluster-admin without JIT.
  - LEAN check `check-rbac-conformance` greps for `cluster-admin` binding + flags.
  - Annual RBAC audit.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.5.18, A.8.2, A.8.3; CIS K8s 5.1.x (RBAC)

**T-E-03 — Service Account token mounted in Pod stolen → API access**
- Asset: SA tokens in pods
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - `automountServiceAccountToken: false` by default on tenant namespaces.
  - Bound Service Account tokens (k8s 1.21+); short TTL.
  - kubernetes-api-proxy validates SA tokens; refuses tokens used outside their bound pod identity.
  - Kyverno refuses pods that mount tokens unless explicitly listed.
- Owner: ops-security + axis-cloud
- Residual: L
- Frameworks: SOC 2 CC6.6; ISO 27001 A.5.15, A.8.3, A.8.7; CIS K8s 5.1.5

**T-E-04 — Kyverno policy bypass via crafted CR**
- Asset: Admission controller
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Kyverno v1.13+ used; CVE-tracked.
  - Policies fuzz-tested at CI time (`check-kyverno-fuzz` lane).
  - Defence-in-depth: Cilium / Istio policy layer below Kyverno.
- Owner: axis-cloud + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC8.1; ISO 27001 A.5.15, A.8.28

**T-E-05 — kubelet exposed → arbitrary container exec**
- Asset: kubelet API (port 10250)
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - kubelet `--anonymous-auth=false`, `--authorization-mode=Webhook`.
  - NetworkPolicy refuses kubelet access from outside `kube-system`.
  - CIS K8s 4.2.x enforced.
- Owner: ops-security + axis-cloud
- Residual: L
- Frameworks: CIS K8s 4.2.x; NSA Hardening §"kubelet"

### Supply-chain (Sup) — CRI / CNI / CSI specific

**T-Sup-01 — Compromised containerd / runc binary on disk**
- Asset: Node runtime binary
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Binary install via signed Debian/RPM package; signature verified at install.
  - File integrity monitoring on `/usr/local/bin/containerd*` + `/usr/local/sbin/runc`.
  - Reboot triggers re-verify checksum.
- Owner: ops-security + cloud-iac
- Residual: L
- Frameworks: NSA Hardening §"Supply Chain"; CIS K8s 4.1.x

**T-Sup-02 — Compromised Cilium agent binary / kernel module**
- Asset: CNI binary
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Cilium installed via signed Helm chart; checksum verified.
  - eBPF programs are kernel-verified before load.
- Owner: ops-security + axis-cloud
- Residual: L

**T-Sup-03 — Compromised Istio / Envoy upstream image**
- Asset: Container image supply chain
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Cosign verification against published Istio signing key.
  - Trivy CVE scan at admission.
- Owner: axis-cloud + ops-security
- Residual: L

**T-Sup-04 — Compromised CSI driver image (per backend)**
- Asset: CSI provisioner
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Cosign verification of CSI driver images.
  - CSI driver runs with minimum privileges (no host-mount beyond declared).
- Owner: axis-cloud + ops-security
- Residual: L

## LINDDUN Privacy-Threat Catalog

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | Pod-to-Pod traffic (Hubble flow) | Multiple flows linkable to a single tenant via flow metadata. | Per-tenant Cedar scope on flow-log reads. | L |
| T-L-02 | Identifiability | Kubernetes API audit log | Pod-spec contents (env vars, volume mounts) may include tenant-identifying strings. | Redactor on audit-log forward to Loki; data_class annotations enforced. | M (engineering discipline) |
| T-L-03 | Non-repudiation | Operator cluster-mutation | Operator may deny a mutation. | audit-chain seal per call. | L |
| T-L-04 | Detectability | Workload pod runtime metrics | Burst timing correlates with tenant business events. | Reasonable; consent at onboarding. | M |
| T-L-05 | Disclosure | Auditor access | Auditor scoped to one tenant could pivot via cluster-wide reads. | Cedar policy enforces tenant-scope on auditor read; pen-tested annually. | L |
| T-L-06 | Unawareness | End-user (tenant's user) | End-user unaware of pod-level tracking via cluster telemetry. | Joint-controllership clause in tenant DPA. | M |
| T-L-07 | Non-compliance | Right-to-erasure on PV data | Tenant-end-user requests erasure; PV data spans multiple snapshots / blocks. | DSR cascade through CSI volume snapshot lifecycle; 30d SLA. | M |

## Mitigations Catalog (cross-reference)

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| Vanilla kubeadm + LTS pinned versions | Preventive | axis-cloud | `check-version-pinning-conformance` |
| etcd encryption-at-rest (KMS envelope) | Preventive | ops-security + cloud-iac | `check-etcd-encryption` lane |
| Istio PeerAuthentication STRICT mesh-wide | Preventive | axis-cloud | `check-istio-strict-mtls` lane |
| Cilium NetworkPolicy enforced kernel-layer | Preventive | axis-cloud | `check-network-policy-conformance` |
| Cosign + Kyverno admission | Preventive | axis-foundry | `check-cosign-admission` lane |
| kubernetes-api-proxy mediation (no direct 6443) | Preventive | axis-cloud + ops-security | NetworkPolicy probe + audit-chain coverage |
| Cedar policy on every API call | Preventive + Audit | ops-security | Cedar fragment coverage CI lane |
| audit-chain Ed25519 seal per cluster mutation | Detective + Non-repudiation | audit-chain | audit-chain regression tests |
| 2-person rule for autonomy-tier T3 ops | Preventive (insider) | ops-security | OpenBao JIT elevation logs |
| Pod Security Standard `restricted` enforced | Preventive | axis-cloud | Kyverno admission policy |
| CIS Kubernetes Benchmark v1.9 lane (BLOCKER) | Preventive + Continuous | axis-cloud | `check-cis-k8s-benchmark` lane |
| NSA/CISA Kubernetes Hardening Guide v1.2 | Preventive | axis-cloud + ops-security | `check-nsa-k8s-hardening` lane |

## Residual Risk Acceptance

Residual risks above L:

| Risk ID | Residual | Why accepted | Re-review date |
|---|---|---|---|
| T-I-03 (Envoy SNI sniffing) | M | TLS protocol limitation; ECH still rolling out. | Quarterly |
| T-I-04 (container escape via kernel CVE) | M | Kernel CVE residual irreducible; mitigations to L feasible only with gVisor / Kata everywhere (cost). | Quarterly |
| T-D-03 (node-network partition) | M | Multi-AZ topology bounds; cannot fully eliminate. | Quarterly |
| T-D-07 (resource exhaustion) | M | Per-tenant quota enforced; envelope-level breach still possible. | Quarterly |
| T-L-02 (audit log identifiability) | M | Pod-spec contents are operator-authored; engineering discipline floor. | Quarterly |
| T-L-04 (timing-detectability) | M | Tenant business reality; consent at onboarding. | Annually |
| T-L-06 (end-user unawareness) | M | Joint-controllership cascade. | Annually |
| T-L-07 (DSR best-effort on PV) | M | Snapshot retention bounds; DSR cascade best-effort. | Annually |

Sign-off (this document is RW until council sign-off captured):
- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`

## Per-Pack Overlay Sections

### pack-kr (Korea PIPA + ISMS-P + CSAP)

- **KR PIPA Art. 29 (technical safeguards)**: every "T-*-NN" mitigation maps to the 12 prescribed safeguards (access control + encryption + integrity verification + audit log ≥ 1y + IDS + …).
- **KR-ISMS-P §2.6 (암호화)**: etcd-at-rest + Istio mTLS strict satisfy the encryption requirement.
- **KR-ISMS-P §2.7 (접근통제)**: kubernetes-api-proxy + Cedar policy + 2-person rule + JIT elevation satisfy access-control.
- **KR-ISMS-P §2.8 (운영)**: runbooks + DR drills + capacity model.
- **KR CSAP**: cloud-security certification controls (cell-isolation + audit retention ≥ 5y + cross-border-forbidden) inherited.

### pack-us-healthcare (HIPAA)

- **HIPAA §164.312(a)(2)(iv) (encryption at rest)**: etcd KMS-envelope + per-PV encryption.
- **HIPAA §164.312(b) (audit controls)**: audit-chain seal + retention ≥ 6y for HIPAA-tagged tenants.
- **HIPAA §164.312(c)(1) (integrity)**: Cosign + Merkle audit-chain.
- **HIPAA §164.312(d) (person/entity authentication)**: OIDC + SPIFFE + 2-person rule.
- **HIPAA §164.312(e)(1) (transmission security)**: Istio mTLS strict + Envoy TLS 1.3.
- **HIPAA §164.310 (physical safeguards)**: inherited from cloud-iac µservice + Oracle HIPAA attestation.

### pack-eu (GDPR + NIS2 + DORA)

- **GDPR Art. 25 (by design)**: pseudonymisation + multi-tenancy + DSR.
- **GDPR Art. 32 (security of processing)**: every mitigation contributes.
- **NIS2 Annex I**: cluster is "important entity"; 24h+72h+1mo incident-reporting timelines integrated.
- **DORA 2022/2554**: for EU financial-services tenants — operational-resilience testing applies; quarterly DR drill satisfies.
- **eIDAS 910/2014 Art. 26**: Ed25519 audit-chain seals are AdES.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/cloud-k8s-overlay.md` follow the same structure.

## Compliance Cross-Mapping

| Framework | Coverage | Mapping doc |
|---|---|---|
| SOC 2 Type 2 | CC6.x (access) ← kubernetes-api-proxy + Cedar + RBAC; CC7.x (operations) ← runbooks; CC8.x (change mgmt) ← PR + LEAN | `microservices/cloud-k8s/compliance.md` |
| ISO 27001:2022 | A.5–A.8 cited inline | same |
| GDPR | Arts. 25/28/30/32/33 cited inline | same |
| CIS Kubernetes Benchmark v1.9 | every control mapped to a mitigation | same |
| NSA/CISA Kubernetes Hardening v1.2 | every recommendation mapped | same |

## Re-review Triggers

- Kubernetes version upgrade (every minor).
- containerd / runc / Istio / Envoy / Cilium version upgrade.
- New pack activation.
- Annual scheduled review (Q2).
- Post-incident.
- CIS / NSA guidance update.

## References

- ADR-0028 (Bominal audit chain; inherited).
- ADR-0056, ADR-0105, ADR-0106 (architecture).
- ADR-0117 (cloud-native progression), ADR-0120 (Rust-first), ADR-0121 (this µservice's substrate).
- ADR-0139 (SLO gate), ADR-0131 (per-microservice flat), ADR-0140 (Cedar).
- `microservices/cloud-k8s/PRD.md`.
- `microservices/cloud-k8s/dpia.md` (paired privacy artifact).
- CIS Kubernetes Benchmark v1.9 — `cisecurity.org/benchmark/kubernetes`.
- NSA/CISA Kubernetes Hardening Guide v1.2 — `nsa.gov/Cybersecurity/Cybersecurity-Advisories-Guidance/`.
- OWASP Kubernetes Security Top Ten — `owasp.org/www-project-kubernetes-top-ten`.
- Microsoft Threat Modeling (STRIDE); LINDDUN (KU Leuven).
- Istio security model — `istio.io/latest/docs/concepts/security/`.
- Cilium security — `docs.cilium.io/en/stable/security/`.
