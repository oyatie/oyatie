---
doc_class: PolicySpec
title: Cluster Isolation Specification
microservice: cloud-k8s
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-cloud
deciders: council-architecture, ops-security, axis-cloud, council-privacy
related_adrs: [ADR-0028, ADR-0117, ADR-0121, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_artifacts:
  - microservices/cloud-k8s/threat-model.md (T-S-04, T-I-02, T-E-02, T-E-03)
  - microservices/cloud-k8s/dpia.md (R-01, R-05, R-06)
  - microservices/cloud-k8s/policy/tenant-scope.cedar
  - microservices/cloud-k8s/policy/ci-scope.cedar
  - microservices/cloud-k8s/policy/auditor-scope.cedar
  - microservices/cloud-k8s/policy/public-read.cedar
review_cadence: quarterly + on every Kubernetes / Istio / Cilium version upgrade
doc_status: published
---

# Cluster Isolation Specification (cloud-k8s µservice)

## Purpose

Define the load-bearing isolation invariants of the on-prem Kubernetes substrate. Authoritative reference for SOC 2 examiners (CC6.1 / CC6.2 / CC6.6), ISO 27001 auditors (A.5.15 / A.8.2 / A.8.3 / A.8.20 / A.8.22), GDPR Art. 32 reviewers, KR PIPA Art. 23 / Art. 29 reviewers, HIPAA §164.312(a)(1) reviewers, and KR-CSAP reviewers asking *"how does cloud-k8s prevent tenant-A's pod from talking to / mounting / observing tenant-B's resources?"*

## Per-Pack Cluster Boundary

### Invariant CI-01: One cluster per regional pack

Each regional pack runs an **isolated Kubernetes cluster**:

| Pack | Cluster name | Region | Activated? |
|---|---|---|---|
| pack-kr | `kr-cluster-1` | OCI ap-seoul-1 (also: on-prem KR primary cell per ADR-0121) | YES (M01 launch) |
| pack-eu | `eu-cluster-1` (+ DR `eu-cluster-2`) | OCI eu-frankfurt-1 + eu-amsterdam-1 | Conditional (post-SCC) |
| pack-us | `us-cluster-1` (+ DR) | OCI us-ashburn-1 + us-phoenix-1 | Conditional |
| pack-us-healthcare | `us-hc-cluster-1` (HIPAA-eligible; isolated from non-HC) | OCI us-ashburn-1 (HIPAA-eligible) | Conditional (post-BAA) |
| pack-jp | `jp-cluster-1` | OCI ap-tokyo-1 | Conditional |
| pack-sg | `sg-cluster-1` | OCI ap-singapore-1 | Conditional |
| pack-au | `au-cluster-1` (+ DR) | OCI ap-sydney-1 + ap-melbourne-1 | Conditional |
| pack-in | `in-cluster-1` (+ DR) | OCI ap-hyderabad-1 + ap-mumbai-1 | Conditional |
| pack-br | `br-cluster-1` (+ DR) | OCI sa-saopaulo-1 + sa-vinhedo-1 | Conditional |
| pack-ae | `ae-cluster-1` (+ DR) | OCI me-abudhabi-1 + me-dubai-1 | Conditional |
| pack-ksa | `ksa-cluster-1` (+ DR) | OCI me-jeddah-1 + me-riyadh-1 | Conditional |

Cross-pack workload scheduling is **forbidden** at the cluster boundary; a tenant pinned to pack-eu **cannot** schedule into the pack-us cluster. Workload µservices declare their `pack_assignment` at deploy time; cloud-k8s rejects scheduling requests that violate pack-pinning.

### Invariant CI-02: Cross-cluster communication only via Istio multi-cluster mTLS

Inter-cluster (cross-pack) network communication uses **Istio multi-cluster mesh** (primary-remote topology after M03 per ADR-0121 §"Migration triggers"). All cross-cluster traffic is mTLS-terminated at each cluster's east-west gateway; no cleartext crosses pack boundaries. Payload replication between packs is forbidden (per `policy/data-residency.md`); only mesh control-plane gossip + JWT-validated API calls flow.

LEAN check `oya-check-cross-cluster-mtls-strict` validates every multi-cluster peer carries an mTLS-strict PeerAuthentication CR.

## Namespace Tenancy Model

### Tenant namespace mapping

```text
canonical_tenant_id         = <opaque-string-issued-at-onboarding>  (NOT used in K8s metadata)
hashed_tenant_id            = sha256(canonical_tenant_id ++ deployment_salt)[..16]
namespace_label "tenant_id" = hashed_tenant_id
namespace_name              = "tenant-" + hashed_tenant_id
```

Properties:
- `canonical_tenant_id` is OpenBao-bound; cluster never receives raw value.
- `deployment_salt` per-cluster secret; rotated 12mo; rotation event audit-chain-emitted.
- `hashed_tenant_id` 16 hex chars (64-bit) namespace.
- Namespace label `tenant_id` is the load-bearing scoping label for every NetworkPolicy / AuthorizationPolicy / Cedar evaluation.

### Reserved namespaces

| Namespace | Purpose | Write authority | Read authority |
|---|---|---|---|
| `kube-system` | Kubernetes control-plane components | platform operators (JIT) | platform operators + observability |
| `istio-system` | istiod + Istio gateway components | axis-cloud (JIT) | observability |
| `cilium-system` | Cilium agent + operator | axis-cloud (JIT) | observability |
| `cosign-system` | Kyverno + Cosign admission webhooks | axis-foundry (JIT) | observability |
| `cloud-k8s-system` | this µservice's own components (api-proxy, bootstrap-worker) | axis-cloud (JIT) | observability |
| `oya-foundry` | Foundry runtime components | axis-foundry (JIT) | observability |
| `oya-observability` | observability µservice Grafana stack | axis-observability | observability operators |
| `oya-ci` | CI runner-scoped namespace | CI principal | CI principal |

Any attempt to create a namespace matching reserved prefixes (`kube-*`, `istio-*`, `cilium-*`, `cosign-*`, `cloud-k8s-*`, `oya-*`) by non-platform principals is refused at admission.

## Network Isolation Invariants (Cilium + Istio Layered)

### Invariant CI-03: Default-deny NetworkPolicy in every tenant namespace

Every tenant namespace ships with a **default-deny** NetworkPolicy at namespace creation:

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-all
  namespace: tenant-<hashed-id>
spec:
  podSelector: {}
  policyTypes: [Ingress, Egress]
```

Cilium kernel-layer enforcement applies. Tenants then add explicit allow-rules via Cedar fragments → derived NetworkPolicy CRs (per `network-policy` BC).

### Invariant CI-04: Default-deny Istio AuthorizationPolicy

Same shape at L7. Every tenant namespace ships with a default-deny AuthorizationPolicy:

```yaml
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: default-deny-all
  namespace: tenant-<hashed-id>
spec: {}
```

Empty `spec` = deny all in Istio semantics. Tenants add explicit allow-rules.

### Invariant CI-05: PeerAuthentication STRICT mesh-wide

```yaml
apiVersion: security.istio.io/v1
kind: PeerAuthentication
metadata:
  name: mesh-default-strict
  namespace: istio-system
spec:
  mtls:
    mode: STRICT
```

Applies cluster-wide. Cleartext between pods is refused. LEAN check `oya-check-istio-strict-mtls` validates this CR is present + unmodified.

### Invariant CI-06: Per-cross-namespace allowance is Cedar-explicit

Cross-namespace communication (e.g., tenant-A's workload → cell µservice's shared scheduler) is **only** allowed via Cedar policy fragments that produce both:
- A Cilium NetworkPolicy allow-rule (L3/L4)
- An Istio AuthorizationPolicy allow-rule (L7)

The `network-policy` BC's `usecase` layer reads the Cedar fragment + emits both CRs atomically. LEAN check `oya-check-cedar-derived-policy-paired` validates every cross-namespace NetworkPolicy has a matching AuthorizationPolicy.

## API-Server Mediation (No Direct Access Invariants)

### Invariant CI-07: All kube-apiserver access through `kubernetes-api-proxy`

Direct port 6443 access to the kube-apiserver is forbidden:

- Cluster nodes apply a NetworkPolicy at the host level refusing inbound traffic on 6443 from outside the api-proxy pod.
- Internal callers (kubelet, controller-manager, scheduler) use in-cluster service `kubernetes.default.svc.cluster.local` which is mesh-routed and AuthorizationPolicy-gated.
- External callers (operator `kubectl`, Foundry agent capability calls, CI runners) route through `https://k8s-api-<pack>.oyatie.dev` (Envoy ingress → `kubernetes-api-proxy` → kube-apiserver).
- `kubernetes-api-proxy` validates OIDC + applies Cedar policy + emits audit-chain record per call.

LEAN check `oya-check-kubernetes-api-proxy-only-path` validates the host-NetworkPolicy is present.

### Invariant CI-08: Cedar policy on every API call

`kubernetes-api-proxy` evaluates the request against `policy/tenant-scope.cedar` (tenant calls), `policy/ci-scope.cedar` (CI principal), `policy/auditor-scope.cedar` (auditor JIT), and `policy/public-read.cedar` (anonymous public surfaces). Deny is the default per Cedar's deny-overrides semantics.

### Invariant CI-09: Audit-chain emission per API call

Every API call processed by `kubernetes-api-proxy` produces an audit record:

```text
{
  call_id: <ULID>,
  principal_spiffe: <SVID>,
  method: GET | POST | PATCH | DELETE | WATCH,
  resource: <gvk + name + namespace>,
  policy_decision: ALLOW | DENY,
  reason: <RFC string if explicit>,
  upstream_status: <HTTP status>,
  ts: <ISO8601>,
  signature: Ed25519(...)
}
```

Forwarded to `audit-chain` µservice with Merkle proof per Bominal ADR-0028. Tampering detected via chain integrity.

## Pod Security Invariants

### Invariant CI-10: Pod Security Standard `restricted` enforced on tenant namespaces

Per Kubernetes PSS v1.30+. Enforced by Kyverno admission policy. Refuses Pod specs that:
- Run as root (`runAsNonRoot: false` or unset)
- Use privileged containers (`privileged: true`)
- Use host namespaces (`hostNetwork`, `hostPID`, `hostIPC`)
- Mount host paths (`hostPath` volumes)
- Use capabilities outside the allowed list (`NET_BIND_SERVICE` only)
- Disable seccomp default
- Allow privilege escalation

LEAN check `oya-check-pod-security-standard-restricted` validates the Kyverno policy is present + unmodified.

### Invariant CI-11: Service Account tokens not auto-mounted

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: default
  namespace: tenant-<hashed-id>
automountServiceAccountToken: false
```

Default for every tenant namespace. Pods that need API access must explicitly mount a Bound Service Account token (k8s 1.21+) via projected volume; Kyverno enforces.

### Invariant CI-12: Supply-chain admission

Every image pull is verified via:
- Cosign signature against a trusted public key (per-namespace ConfigMap).
- Sigstore Rekor transparency log (secondary verification).
- Trivy CVE scan; admission refused on Critical CVE.
- Grype scan for SBOM completeness.

Per ADR-0117 §"Supply chain". LEAN check `oya-check-cosign-admission` validates Kyverno policy presence.

## etcd Isolation

### Invariant CI-13: etcd at-rest encrypted (KMS envelope)

kube-apiserver `--encryption-provider-config` flag set; KMS provider per pack. envelope key rotated 90d (audit-chain-emitted). Direct disk read yields ciphertext only.

### Invariant CI-14: etcd peer + client mTLS strict

| Connection | Auth mode |
|---|---|
| etcd peer-to-peer | mTLS strict; CN-bound certs from per-cluster CA |
| kube-apiserver → etcd | mTLS strict; per-component CN |
| etcdctl direct (operator) | mTLS strict + JIT cert from OpenBao with TTL ≤ 4h |
| Any other direct | REFUSED at NetworkPolicy + etcd auth |

### Invariant CI-15: etcd snapshot 5-min cadence

`oya-cloud-k8s-cluster-bootstrap-worker` triggers `etcdctl snapshot save` every 5 min. Snapshot is Ed25519-signed at creation + uploaded to per-pack object storage. Snapshot retention 14d.

## Failure Modes

### FM-01: NetworkPolicy regression → cross-tenant cleartext

**Behaviour:** LEAN check refuses merge; continuous-state validator alarms on live drift.
**Detection:** `oya-check-network-policy-conformance` lane + continuous validator + Cilium policy-drift alert.
**Recovery:** auto-rollback to git state; ops-security incident if intentional.

### FM-02: PeerAuthentication CR mutated to PERMISSIVE

**Behaviour:** LEAN refuses merge; continuous-state validator alarms.
**Detection:** `oya-check-istio-strict-mtls`.
**Recovery:** auto-rollback; ops-security incident.

### FM-03: kube-apiserver direct access attempt

**Behaviour:** NetworkPolicy refuses; alert fires.
**Detection:** `kubernetes_api_direct_access_attempt_total > 0` over 1m.
**Recovery:** identify caller; revoke credentials; trace path.

### FM-04: SA token mounted in tenant pod (Kyverno bypass attempted)

**Behaviour:** Kyverno admission refuses; alert fires.
**Detection:** Kyverno admission events.
**Recovery:** review pod-spec author; PR + LEAN should have caught earlier.

### FM-05: Cosign signature missing or mismatched

**Behaviour:** admission refuses image pull.
**Detection:** Kyverno admission event; `oya_admission_image_rejected_total > 0`.
**Recovery:** investigate supply chain.

### FM-06: Cross-pack scheduling attempted

**Behaviour:** Workload pinned to pack-eu attempts scheduling into pack-us cluster.
**Detection:** `cloud-k8s.cluster.schedule` capability validates pack assignment; rejects mismatch.
**Recovery:** correct workload manifest; investigate misconfig.

### FM-07: Cedar policy fragment introduces over-broad allow

**Behaviour:** PR review + Cedar fuzz CI lane catches at PR; live runtime audit detects post-merge.
**Detection:** CI lane + `oya_authorization_anomaly_total > 0`.
**Recovery:** revert Cedar fragment; engage council-privacy.

### FM-08: Bootstrap kubeadm token leaked

**Behaviour:** Secret-scanner detects on commit; OpenBao rotates the token.
**Detection:** `oya-governance-evidence-secret-scan` + GitHub secret-scanning.
**Recovery:** rotate token (TTL ≤ 24h already); audit nodes joined within leak window.

## Audit Trail

Every cluster mutation through `kubernetes-api-proxy` emits an audit-chain record per CI-09. Per-tenant audit reads supported via Cedar `policy/tenant-scope.cedar` (tenant reads own namespace's events only).

Retention:
- pack-us-healthcare: ≥ 6y (HIPAA §164.316(b)(2))
- pack-kr: ≥ 5y (KR commercial code)
- pack-eu: 2y default (purpose-limited)
- other packs: 2y default

## Per-Pack Overlays

### pack-kr

- **KR PIPA Art. 29 (technical safeguards)**: invariants CI-01 .. CI-15 map to the 12 prescribed safeguards.
- **KR-ISMS-P §2.7 (접근통제)**: kubernetes-api-proxy + Cedar + 2-person rule fully satisfy.
- **KR CSAP §"격리" (isolation)**: per-pack cluster boundary + namespace + NetworkPolicy + AuthorizationPolicy + PSS-restricted = 4-layer defense.

### pack-us-healthcare

- **HIPAA §164.312(a)(1) (access control)**: multi-namespace + Cedar.
- **HIPAA §164.312(c)(1) (integrity)**: Cosign + audit-chain.
- **HIPAA §164.312(e)(1) (transmission)**: Istio mTLS strict + Envoy TLS 1.3.

### pack-eu

- **GDPR Art. 25 (by design)**: default-deny posture; pseudonymisation via hashed tenant-id; multi-tenancy.
- **GDPR Art. 32 (security of processing)**: invariants satisfy Art. 32(1)(a)(b)(c)(d).
- **NIS2 Annex I**: cluster is "important entity"; 24h+72h+1mo incident reporting integrated.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/cloud-k8s-isolation-overlay.md`.

## Verification

- `cargo run -p oya-dev-cli -- gate validate cluster-isolation-conformance --pack <pack>` — exit 0.
- Annual cross-tenant pen-test.
- Quarterly Cedar fragment + NetworkPolicy + AuthorizationPolicy drift audit.
- Continuous: LEAN lanes + state-drift CronJob.

## References

- `microservices/cloud-k8s/threat-model.md`.
- `microservices/cloud-k8s/dpia.md`.
- `microservices/cloud-k8s/policy/*.cedar`.
- ADR-0117, ADR-0121, ADR-0139, ADR-0131, ADR-0140.
- CIS Kubernetes Benchmark v1.9.
- NSA/CISA Kubernetes Hardening Guide v1.2.
- Istio security model — `istio.io/latest/docs/concepts/security/`.
- Cilium NetworkPolicy — `docs.cilium.io/en/stable/security/policy/`.
- Kubernetes Pod Security Standards — `kubernetes.io/docs/concepts/security/pod-security-standards/`.
