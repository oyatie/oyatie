---
doc_class: Checklist
checklist_id: CHK-IP
status: Accepted
date: 2026-05-12
purpose: |
enforcing_fitness_lane: oya-governance-plan-hierarchy
owner_team: council-architecture
related:
  - docs/templates/implementation-plan-template.md
  - docs/checklists/done-definition-checklist.md
  - docs/checklists/agent-completion-checklist.md
adrs_cited:
  - ADR-0052  # inventory ledger (migration-class IP rows)
  - ADR-0054  # scaffold-claim (symbol coverage)
doc_status: published
---

# Per-IP Completion Checklist


## Structural

- [ ] **IP0** IP declares `execution_unit: ChangeSet` and its scope is claimable, independently verifiable, bundleable, promotable, and not over-broad. Split before execution if unrelated lock scopes, packages, or deployables are present. *Lane:* `oya-governance-plan-hierarchy`.
- [ ] **IP2** IP frontmatter `final_shape_compliance: true` honored — no `TODO` / `FIXME` / `unimplemented!()` / `todo!()` introduced (outside `flaky/` or ADR-tracked carve-outs). *Lane:* `oya-governance-no-placeholder`.

## Acceptance + verification

- [ ] **IP5** Every command from IP `§Acceptance test commands` produced its expected pass token; outputs pasted in PR `## Verification`. *Verification:* command outputs.
- [ ] **IP6** Done-definition checklist (`docs/checklists/done-definition-checklist.md`) D1-D20 walked. *Lane:* per-row.
- [ ] **IP7** Linus good-taste row in IP `§Decision log` is non-empty (or explicitly `"none — no candidates"`). *Lane:* `oya-governance-plan-hierarchy`.

## Dependency + supply chain

- [ ] **IP8** Every entry in IP `§Dependency additions` cleared `cargo deny check` and is current LTS (or has ADR-tracked exception). *Lane:* `oya-governance-lts-dependency`.
- [ ] **IP9** If IP ships a deployed binary: distroless image built, image-size budget met. *Lane:* `oya-governance-image-discipline`.
- [ ] **IP10** If IP ships a deployed binary: Cosign signature + Syft SBOM + SLSA L2+ provenance attested. *Lane:* `oya-governance-supply-chain`.

## Audit + rollback

- [ ] **IP11** Audit-chain `EVT-<topic>` emitted; ID pasted in PR `## Evidence`. *Lane:* `oya-governance-audit-emission`.
- [ ] **IP12** Rollback procedure (IP `§Rollback procedure`) was dry-run validated (where safe) or has a runbook reference. *(advisory; required for migration-class IPs — Lane:* `oya-governance-schema-migration`).
- [ ] **IP14-INV** If migration-class IP: inventory ledger row appended per `docs/checklists/inventory-update-checklist.md` (ADR-0052). *Lane:* `oya-governance-inventory-tracker`.

## Hand-off

- [ ] **IP15** `§Next IP pointer` resolves to a real file. *Lane:* `oya-governance-plan-hierarchy`.
- [ ] **IP16** Parent phase INDEX `§Implementation Plans` row updated to `merged`. *Lane:* `oya-governance-plan-hierarchy`.
- [ ] **IP17** IP frontmatter `status: merged` flipped in same PR. *Lane:* `oya-governance-plan-hierarchy`.

