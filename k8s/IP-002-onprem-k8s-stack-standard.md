---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-onprem-k8s-substrate
impl_plan_id: IP-002-onprem-k8s-stack-standard
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-cloud + ops-security
acceptance_lanes: [docs-link-check, governance-version-pinning-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: docs/standards/cloud-k8s-stack.md (cross-cutting standard)

## Intent

Author `docs/standards/cloud-k8s-stack.md` — the authoritative LTS version pin matrix + admission-controller configuration + etcd encryption posture + kubeadm config reference for every pack cluster. Referenced by IP-001 IaC, IP-014 branch-protection, and the CIS K8s Benchmark lane.

## ChangeSet boundary

One new doc + cross-references in PRD / threat-model / compliance. No Rust code.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `docs/standards/cloud-k8s-stack.md` | create | LTS pins, admission config, etcd encryption, kubeadm config |
| `microservices/cloud-k8s/PRD.md` | update | Add reference link |
| `microservices/cloud-k8s/threat-model.md` | update | Reference admission section |

## Code Shape (excerpt)

```yaml
# docs/standards/cloud-k8s-stack.md §"LTS pin matrix"
component_versions:
  containerd: "2.3.0"     # first annual LTS per ADR-0121
  runc: "1.4.0"
  cni_plugins: "1.6.0"
  kubernetes: "1.35"      # N-1; supported window 1.36/1.35/1.34
  istio: "1.29.2"
  envoy: "1.32.x"         # bundled with Istio
  cilium: "1.16.0"        # LTS
  cosign: "2.5.x"
  kyverno: "1.13.x"

admission_controllers:
  - PodSecurityStandard:restricted   # tenant ns; per policy/cluster-isolation.md CI-10
  - Kyverno:                          # Cosign + Trivy + Pod Security + RBAC
      enforce_mode: enforce
  - ValidatingAdmissionPolicy:       # k8s native; redundant with Kyverno
      enforce_mode: enforce

etcd_encryption:
  provider: kms
  kms_provider: opencbao
  key_rotation_days: 90
  envelope_mode: AES-256-GCM

kubeadm_config:
  controlPlaneEndpoint: "<dns>:6443"
  apiServer:
    extraArgs:
      authorization-mode: Node,RBAC
      audit-log-path: /var/log/kubernetes/audit.log
      audit-policy-file: /etc/kubernetes/audit-policy.yaml
      audit-log-maxage: "30"
      audit-log-maxbackup: "10"
      audit-log-maxsize: "100"
      encryption-provider-config: /etc/kubernetes/encryption-config.yaml
      tls-min-version: "VersionTLS13"
```

## Acceptance Gates

```bash
markdown-link-check docs/standards/cloud-k8s-stack.md
cargo run -p dev-cli -- gate validate version-pinning-conformance
cargo run -p dev-cli -- gate validate cis-k8s-benchmark --microservice cloud-k8s --dry-run
```

## Test Plan

- Doc render: markdown lints pass.
- Cross-reference check: every IP-001 chart values.yaml has version matching this doc.

## Halt Conditions

- Any value differs from upstream LTS support window — escalate to council-architecture.

## Next IP

[`IP-003-cluster-bootstrap-kernel.md`](IP-003-cluster-bootstrap-kernel.md)

## References

- ADR-0121 §"Version pins".
- ADR-0117 §"Supply chain".
- `microservices/cloud-k8s/PRD.md`.
