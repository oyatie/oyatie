---
doc_class: Checklist
checklist_id: CHK-IP
status: pending approval
purpose: |
  IP-internal verification. Walked before flipping an IP `status:` from `in-progress` to `merged` and before the worker agent marks the GitHub PR lane ready.
lift_target: oyatie/docs/checklists/per-implementation-plan.md
enforcing_fitness_lane: oya-governance-plan-hierarchy
owner_team: council-architecture
related:
  - /templates/implementation-plan-template.md
  - /templates/checklists/done-definition-checklist.md
  - /templates/checklists/agent-completion-checklist.md
---

# Per-IP Completion Checklist

> Walk **all** rows before marking the GitHub PR lane ready and before flipping IP `status:` to `merged`. Each row names a lane / command / advisory.

## Structural

- [ ] **IP0** IP declares `execution_unit: ChangeSet` and its scope is claimable, independently verifiable, bundleable, promotable, and not over-broad. Split before execution if unrelated lock scopes, packages, or deployables are present. *Lane:* `oya-governance-plan-hierarchy`.
- [ ] **IP1** All `lane_owned_paths` from frontmatter stayed inside this worktree/PR lane. *Verification:* `git diff --name-only origin/dev...HEAD` output.
- [ ] **IP2** IP frontmatter `final_shape_compliance: true` honored — no `TODO` / `FIXME` / `unimplemented!()` / `todo!()` introduced (outside `flaky/` or ADR-tracked carve-outs). *Lane:* `oya-governance-no-placeholder`.
- [ ] **IP3** All `agent_prerequisites` from frontmatter were read and cited in PR `## Traceability`. *(advisory)*
- [ ] **IP4** `§Lane-owned paths / symbols` set matches the actual diff (no over-claim, no under-claim). *Lane:* `repo-hygiene-automation-check` or a lane-owned Buck2 coverage target.

## Acceptance + verification

- [ ] **IP5** Every command from IP `§Acceptance test commands` produced its expected pass token; outputs pasted in PR `## Verification`. *Verification:* command outputs.
- [ ] **IP6** Done-definition checklist (`/templates/checklists/done-definition-checklist.md`) D1-D18 walked. *Lane:* per-row.
- [ ] **IP7** Linus good-taste row in IP `§Decision log` is non-empty (or explicitly `"none — no candidates"`). *Lane:* `oya-governance-plan-hierarchy`.

## Dependency + supply chain

- [ ] **IP8** Every entry in IP `§Dependency additions` is registered in the dependency rationales/allowlist registries and is current stable/LTS or has an explicit waiver. *Lane:* `buck2 build //:repo-hygiene-automation-check`.
- [ ] **IP9** If IP ships a deployed binary: distroless image built, image-size budget met. *Lane:* `oya-governance-image-discipline`.
- [ ] **IP10** If IP ships a deployed binary: Cosign signature + Syft SBOM + SLSA L2+ provenance attested. *Lane:* `oya-governance-supply-chain`.

## Audit + rollback

- [ ] **IP11** Audit-chain `EVT-<topic>` emitted; ID pasted in PR `## Evidence`. *Lane:* `oya-governance-audit-emission`.
- [ ] **IP12** Rollback procedure (IP `§Rollback procedure`) was dry-run validated (where safe) or has a runbook reference. *(advisory; required for migration-class IPs — Lane:* `oya-governance-schema-migration`).
- [ ] **IP13** IP completion evidence bundle was written under `/evidence/multispectrum/` and cited in PR `## Evidence`. *Verification:* file path + secret-scan output.

## Hand-off

- [ ] **IP14** `§Next IP pointer` resolves to a real file. *Lane:* `oya-governance-plan-hierarchy`.
- [ ] **IP15** Parent phase INDEX `§Implementation Plans` row updated to `merged`. *Lane:* `oya-governance-plan-hierarchy`.
- [ ] **IP16** IP frontmatter `status: merged` flipped in same PR. *Lane:* `oya-governance-plan-hierarchy`.

If any row is unchecked, **do not** mark the PR lane ready. Loop back per `/templates/checklists/agent-completion-checklist.md`.
