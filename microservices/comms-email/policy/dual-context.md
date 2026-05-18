# Dual-context policy — `comms-email` µservice

> ADR anchor: ADR-0201, ADR-0183.

## Two contexts

Every authorization decision in comms-email carries two
contexts:

1. **Application context** — Cedar policies in this directory.
   Owns: who may send, who may onboard a from-domain, who may
   touch the suppression list.
2. **Admission context** — Kyverno admission policies in
   `policy/admission/` (rendered from this directory at CI).
   Owns: which Helm releases may target which packs, which
   provider configurations are valid for the active cluster.

## Why two

Per ADR-0183 policy engine separation:

- Cedar owns application-tier authz (per-request decisions
  against user / service-account principals).
- Kyverno owns admission-tier policy (per-manifest decisions
  against K8s objects at deploy time).

Putting both in one engine fails because they have different
data sources (Cedar uses request context + ABAC; Kyverno uses
the K8s API). Putting them in different engines without an
explicit dual-context contract leads to drift — sovereign packs
might enforce in Cedar but admit a Helm chart that violates the
pack rule.

## Drift detection

A CI lane reads:

- The Cedar policies in this directory.
- The Kyverno policies in `policy/admission/`.
- The pack overlay declarations.

It asserts:

- Every Cedar `forbid` for a pack/provider pair is mirrored in
  Kyverno admission.
- Every pack overlay's `allowed_providers` declaration is
  represented in both Cedar and Kyverno.

Mismatches fail the gate.

## See also

- `policy/residency.md` for data-residency rules.
- `policy/comms-email-*.cedar` for the four Cedar fragments.
