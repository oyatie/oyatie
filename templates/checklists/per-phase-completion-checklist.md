---
doc_class: Checklist
checklist_id: CHK-PHASE
status: pending approval
purpose: |
  Phase-internal verification. Walked before flipping a phase INDEX `status:` from `in-progress` to `merged` and before emitting the phase-handoff icm event.
lift_target: oyatie/templates/checklists/per-phase-completion.md
enforcing_fitness_lane: oya-governance-plan-hierarchy
owner_team: council-architecture
related:
  - .omc/plans/MASTERPLAN.md
  - /templates/phase-index-template.md
  - /templates/checklists/per-implementation-plan-checklist.md
---

# Per-Phase Completion Checklist

> Walk **all** rows before flipping the phase INDEX status to `merged`. Each row names a lane / command / advisory.

## Structural

- [ ] **PH1** All IPs under `phases/P0N-<slug>/` have `status: merged` in their frontmatter. *Lane:* `oya-governance-plan-hierarchy`.
- [ ] **PH2** Phase INDEX `§Acceptance` criteria all met (each row has a lane / command / advisory; all PASS). *Verification:* per-row evidence captured in PR.
- [ ] **PH3** Phase INDEX `§Symbols touched (high level)` reconciled — no orphan symbols claimed-but-not-released via `grit`. *Command:* `oya-tooling-agent-read grit-status --phase P0N-<slug> --orphans`.
- [ ] **PH4** No `<!-- forward-reference: wave-1 -->` markers remain pointing at artifacts the phase was supposed to ship. *Lane:* `oya-governance-forward-reference`.

## Engineering bar

- [ ] **PH5** All hyperscaler practices enumerated in milestone INDEX `§Inherited hyperscaler practices` for this phase have evidence captured (e.g., postmortem on Sev-1/2 events; design doc accepted; PRFAQ archived). *(advisory)*
- [ ] **PH6** `cargo test --workspace` green on the phase-final merge commit. *Verification:* command output.
- [ ] **PH7** Distroless image build (if phase ships binaries) passes image-size budget. *Lane:* `oya-governance-image-discipline`.
- [ ] **PH8** Supply-chain attestation: Cosign + Syft SBOM + SLSA provenance for every artifact emitted in this phase. *Lane:* `oya-governance-supply-chain`.

## Docs + governance

- [ ] **PH9** `docs/CHANGELOG.md` rows added for every canonical-doc touch. *Lane:* `oya-governance-changelog-row`.
- [ ] **PH10** `docs/MISTAKES-LEDGER.md` rows added for any regression-class fixes shipped in phase. *Lane:* `oya-governance-mistakes-ledger-cite`.
- [ ] **PH11** ADRs authored in phase have `status: Accepted` (or `Proposed` with explicit deferral noted in milestone INDEX `§Risk`). *Lane:* `oya-governance-adr-shape`.
- [ ] **PH12** Runbooks authored in phase resolve in `docs/RUNBOOKS-INDEX.md`. *Lane:* `oya-governance-runbook-index-resolves`.

## Audit-chain + handoff

- [ ] **PH13** Audit-chain emits `EVT-PHASE-COMPLETED` with phase ID + merge SHA + IP list. *Lane:* `oya-governance-audit-emission`.
- [ ] **PH14** Phase-handoff icm event emitted (verbatim from phase INDEX `§Agent-handoff`). *Command:* `icm store -t phase-handoff …`.
- [ ] **PH15** Next-phase INDEX `gates_on:` row marks this phase `merged`. *Lane:* `oya-governance-plan-hierarchy`.
- [ ] **PH16** Status reporting row added to `docs/status-reports/YYYY-Www.md` per `MASTERPLAN.md §11 cadence`. *(advisory)*

## Loop-cancellation re-walk

If walking this checklist inside a Ralph / autopilot / ultrawork / team loop, re-walk against the latest state before exiting (per `docs/AGENTS.md §Long-running loop rule`). Loops **MUST NOT** exit silently.
