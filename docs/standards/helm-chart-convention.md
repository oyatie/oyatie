---
doc_class: RetiredRedirect
status: Retired
date: 2026-06-05
canonical_authority: docs/standards/kubernetes-desired-state-authority.md
planned_enforcement_ref: buck2 build //:kubernetes-native-anti-pattern-check
---

# Retired redirect — Helm chart convention

This file is retained only to preserve historical links.

Canonical first-party Kubernetes desired-state authority moved to
[CUE/Kubernetes desired-state authority](kubernetes-desired-state-authority.md).

Do not use this file as current implementation guidance.

Current rule:

- CUE packages own first-party cloud-cell, pod, and workload desired state.
- Generated Kubernetes manifests are Buck2/Prow-checked artifacts.
- Helm is adapter compatibility only for third-party charts, generated wrappers,
  or temporary import/export migration seams.
- Hand-authored first-party Helm templates and Helm CLI deploy flows are not
  canonical policy/deployment authority.
