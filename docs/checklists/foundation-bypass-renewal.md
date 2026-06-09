---
doc_status: published
---

# Oyatie Checklist — Foundation Bypass Renewal

> **Trigger:** per bypass expiry
> **Owner:** `architecture-governance` accountable; owning crate team executes.
> **Validator:** cloud-ci/oya-ci foundation-bypass gate evidence in `oya-ci-required` plus PR traceability evidence per [`STANDARDS-AND-TEMPLATES.md`](../STANDARDS-AND-TEMPLATES.md) §3. Retired local wrapper output is not accepted as renewal authority.

## Pre-flight
1. Identify the expiring `registry/foundation-bypasses/*.yaml` record.
2. Read all dependent docs per [`DOC-CATALOG.md`](../DOC-CATALOG.md), especially the ADR or plan that justified the exception.
3. Notify the owning crate team, `architecture-governance`, and the validator listed in the PR traceability record.

## Steps
1. Re-run the blocked gate locally without the exception and capture the current failure or passing output.
2. If the gate now passes, remove the YAML record in the same PR that includes the remediation evidence.
3. If the gate still fails, update the record only with a narrower rationale, a new remediation owner, and the shortest regression window accepted by `architecture-governance`.
4. Link the renewal PR to the remediation issue and include the gate output in the PR evidence section.
5. Do not merge a renewal that widens scope, changes `gate_bypassed`, or extends an exception without reviewer sign-off from both `architecture-governance` and the owning crate team.

## Validation
- `oya-ci-required` run URL showing the foundation-bypass and PR-traceability gate results
- reviewer-agent traceability verdict for the renewal PR
- Per-step evidence captured in the PR.
- Owner team and `architecture-governance` sign-off.

## Sources
[`STANDARDS-AND-TEMPLATES.md`](../STANDARDS-AND-TEMPLATES.md), [`DOC-CATALOG.md`](../DOC-CATALOG.md), [`MISTAKES-LEDGER.md`](../MISTAKES-LEDGER.md).
