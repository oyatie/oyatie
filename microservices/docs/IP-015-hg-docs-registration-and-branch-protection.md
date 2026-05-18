---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-015-hg-docs-registration-and-branch-protection
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-docs + ops-release-management
acceptance_lanes: [branch-protection-validate, oya-governance-hyperscaler-maturity-claims, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: HG-DOCS registration + branch-protection

## Intent

Register HG-DOCS as a BLOCKER lane in `.github/branch-protection.yaml`. Docs promotion past dev requires HG-DOCS green per ADR-0123 + ADR-0139. Wire all 9 OpenSLO manifests + all per-BC CI lanes into branch-protection.

## ChangeSet boundary

`.github/branch-protection.yaml` + `/registry/quality/lanes.yaml` updates.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `.github/branch-protection.yaml` | extend | add HG-DOCS + per-BC + per-SLO lanes |
| `/registry/quality/lanes.yaml` | extend | per-lane metadata for new HG-DOCS lanes |
| `/registry/claim-matrix/ops-portal.json` | extend | claim ownership for docs lanes |
| `/registry/hyperscaler-maturity-claims.json` | extend | HG-DOCS claim entry |

## New BLOCKER lanes registered

- `oya-governance-crdt-no-silent-loss` — per ADR-DOCS-0001 + PRD AC-06.
- `oya-governance-crdt-cross-microservice-consistency` — per ADR-DOCS-0001 + ADR-WS-0001.
- `oya-governance-per-block-acl` — per ADR-DOCS-0004 + PRD AC-04.
- `oya-governance-acl-enforcement-correctness` — per PRD AC-04.
- `oya-governance-export-sandbox-conformance` — per ADR-DOCS-0003 + PRD AC-09.
- `oya-governance-ooxml-import-fidelity` — per ADR-DOCS-0006 + PRD AC-03.
- `oya-governance-pdfa-conformance` — per PRD AC-10.
- `oya-governance-wcag-22-aa-conformance` — per PRD AC-11.
- `oya-governance-embed-resolver-acl-passthrough` — per ADR-DOCS-0004.
- `oya-governance-merkle-chain-continuity` — per ADR-0028.
- `oya-governance-ai-act-conformance` — per ADR-DOCS-0005.
- HG-DOCS — composite gate per ADR-0123.

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate branch-protection-validate
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice docs
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice docs
```

## Test Plan

- branch-protection.yaml parses + all referenced lanes exist in registry.
- HG-DOCS composite is consistent with the SLO files + conformance tests.

## Halt Conditions

- Any referenced lane missing from registry — block.
- HG-DOCS composite trips on a known-passing setup — block; root-cause.

## Phase exit

This IP closes M03-connect-dissolution-phase-01-docs-foundation phase. After all 15 IPs land, the docs µservice is "Phase 1 exit-gate ready" per ADR-0134 phase model (parallel ship; legacy `oya-connect-docs-*` still serves traffic; new `oya-docs-*` serves canary).

## References

- ADR-0123; ADR-0139; ADR-0131; ADR-0134.
- `.github/branch-protection.yaml`.
- `/registry/quality/lanes.yaml`.
- `microservices/docs/PHASE-01-DOCS-FOUNDATION.md`.
