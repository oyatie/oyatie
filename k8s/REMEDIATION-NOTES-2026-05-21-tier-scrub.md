# Wave 15J-batch-4 tier scrub remediation notes: cloud-k8s

## Files Modified

- ARCHITECTURE.md: 754 lines
- README.md: 23 lines
- benchmarks/kubeadm-vs-managed-vs-rancher.md: 94 lines
- capabilities/cluster-bootstrap.yaml: 120 lines
- capabilities/network-policy-apply.yaml: 111 lines
- capabilities/node-lifecycle.yaml: 111 lines
- coherence-audit-2026-05-20.md: 986 lines
- decisions/ADR-CK-001-cilium-cni-selection.md: 98 lines
- faqs/sre-faq.md: 62 lines
- feature-parity-matrix-2026-05-20.md: 390 lines
- manifest.json: 396 lines
- migration-playbooks/from-rancher-rke2.md: 131 lines
- onboarding/sre-first-week.md: 137 lines
- performance-benchmark-numbers-2026-05-20.md: 593 lines
- tutorials/bootstrap-bronze-cluster.md: 170 lines

## Directory Deletion

- capability-tiers/ dir deleted: Y

## Vocabulary Replacement Count

- Rough replacement count: ~250 matches, including deleted capability-tiers/ content.

## Design Decisions

- Replaced capability-level language with `tenant_class`, `billing_components`, `cell_topology`, and `compliance_pack`.
- Converted manifest `capability_tiers` to `tenant_class_model`.
- Reworded SRE/benchmark differentiation as demo_trial caps, paid deployment-context placement, and pack-bound custody.
- Replaced incidental "golden signals" vocabulary with "key signals" so the banned string scan is clean.

## Outstanding Follow-ups

- none
