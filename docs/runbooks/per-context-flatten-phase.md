---
purpose: Oyatie Runbook — Per-Context Flatten Phase
doc_status: published
---

# Oyatie Runbook — Per-Context Flatten Phase

> **Status:** Active
> **Owner:** `council-architecture`
> **Severity supported:** Sev 3
> **Last verified:** 2026-05-11 by Codex in local drill
> **Related:** [ADR-0015](../decisions/ADR-0015-architectural-flattening-target.md), [ROADMAP.md](../ROADMAP.md), [DESIGN.md](../DESIGN.md), [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md)

---

## Trigger

Open this runbook before starting or closing one ADR-0015 migration phase for a bounded context or axis.

The phase order is fixed:

```text
Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5 → Phase 6 → Phase 7
kernel    contracts  domain    app       api/worker/adapter  runtime  sweep
```

---

## Pre-checks (5 minutes max)

- [ ] Confirm the previous phase is green for the same context — verify with the phase evidence bundle or retired `./bin/oya verify` output.
- [ ] Confirm no PR ahead in the merge queue touches root `Cargo.toml [workspace.members]`.
- [ ] Confirm no new top-level `modules/`, `services/`, or `platform/` tree exists.
- [ ] Confirm every candidate crate has a catalog target and role from ADR-0015.
- [ ] Confirm downstream consumers are identified for any contract or runtime phase.

If a pre-check fails, do not start the phase; open [workspace-members-merge-queue.md](workspace-members-merge-queue.md) for serialization or split the scope into smaller move PRs.

---

## Steps

1. ☐ Declare the phase scope.
   Expected: the PR description lists context, phase number, crates, contracts, and downstream consumers.
   If differs: stop and write the scope before edits.

2. ☐ Sequence move PRs from inward roles to outward roles.
   Expected: `kernel` changes land before `domain`, then `app`, then `api`/`worker`/`adapter`, then `runtime`.
   If differs: stop; do not make outward roles depend on an unlanded inward move.

3. ☐ For each crate move, use [flat-crates-move-pr.md](flat-crates-move-pr.md).
   Expected: each PR carries its own catalog update and targeted verification.
   If differs: split the move PR.

4. ☐ Run phase-level dependency and catalog checks after each PR lands.
   Command: `presubmit` (retired CLI `gate validate architecture-boundaries && cargo run -p tooling-cli-dev-runtime -- catalog validate`)
   Expected: no forbidden role edge and every workspace package has a catalog record.
   If differs: halt the phase until fixed.

5. ☐ Close the phase with an evidence bundle.
   Expected: bundle includes moved crates, unchanged external contract summary, verification commands, reviewer verdict, and deferred follow-ups.
   If differs: keep the phase open.

---

## Rollback

- Roll back the last move PR first; do not roll back earlier inward-role PRs unless their verification also failed.
- Restore root `Cargo.toml [workspace.members]` and the matching catalog record in the same rollback.
- Re-run `presubmit` (retired CLI `gate validate architecture-boundaries`) and the moved crate tests before reopening the phase.

---

## Verification

- [ ] Phase move PRs all have passing targeted tests.
- [ ] `presubmit` (retired CLI `gate validate architecture-boundaries --self-test`) passes.
- [ ] `presubmit` (retired CLI `gate validate architecture-boundaries`) passes.
- [ ] `cargo run -p tooling-cli-dev-runtime -- catalog validate` passes.
- [ ] `cargo run -p tooling-cli-dev-runtime -- gate validate quality-lanes` passes.
- [ ] `cargo run -p tooling-cli-dev-runtime --bin repoctl -- pre-push` passes at phase close or the phase records an explicit local-resource blocker plus targeted substitutes.

---

## Post-incident updates

If a phase stalls or breaks `main`:
- [ ] Add or update the corresponding MFL row in [MISTAKES-LEDGER.md](../MISTAKES-LEDGER.md).
- [ ] Update [ROADMAP.md](../ROADMAP.md) if the phase order or blast radius changes.
- [ ] Update [DESIGN.md](../DESIGN.md) if the clean-architecture role boundary changes.

---

## Audit-chain emission

Each phase emits an engineering evidence event with:
- runbook id: `per-context-flatten-phase`
- context and phase number
- crate list and PR list
- downstream consumer list
- verification commands and outcomes
- remaining risks

---

## Sources scanned

- [ADR-0015](../decisions/ADR-0015-architectural-flattening-target.md)
- [ROADMAP.md](../ROADMAP.md)
- [DESIGN.md §4 and §8](../DESIGN.md)
- [templates/runbook-template.md](../templates/runbook-template.md)
