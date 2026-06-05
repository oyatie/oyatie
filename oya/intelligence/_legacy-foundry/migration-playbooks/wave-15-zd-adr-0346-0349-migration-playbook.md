---
doc_class: MigrationPlaybook
microservice: intelligence-legacy-foundry
source_vendor: Wave 15-ZD doctrine propagation
related_adrs: [ADR-0346, ADR-0347, ADR-0348, ADR-0349, ADR-0513]
date: 2026-05-21
doc_status: legacy_superseded_guidance
superseded_by: ADR-0513
authority_posture: Buck2/Prow/Kubernetes-native oya-ci-required
---

# Legacy Foundry migration playbook — current authority overlay

The legacy Foundry subtree is retained for traceability only. Do not copy its former local verifier, bridge substrate, or local CLI procedures into active work. Any active Intelligence lane that still needs a legacy behavior must re-express it as:

- Buck2-owned build/test/check/coverage evidence;
- Prow/Kubernetes-native oya-ci-required status;
- CUE/KRM desired-state reconciliation;
- signed Intelligence control-plane operation plus operation-ledger and audit-chain evidence;
- GitHub PR/Actions shadow evidence only while the temporary lane-unlocker bridge remains in use.

Exact retired substrate names belong in retired registries or historical ADR provenance, not active implementation instructions.
