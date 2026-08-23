---
purpose: Oyatie Runbook — Workspace Members Merge Queue
doc_status: published
---

# Oyatie Runbook — Workspace Members Merge Queue

> **Status:** Active
> **Owner:** `council-architecture`
> **Severity supported:** Sev 3
> **Last verified:** 2026-05-11 by Codex in local drill
> **Related:** [ADR-0015](../decisions/ADR-0015-architectural-flattening-target.md), [PRD.md](../PRD.md), [RELEASE-MANAGEMENT.md](../RELEASE-MANAGEMENT.md), [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md)

---

## Trigger

Open this runbook when two or more active PRs need to modify root `Cargo.toml [workspace.members]`, or when a flattening PR conflicts on workspace membership, catalog records, or crate path ownership.

---

## Pre-checks (5 minutes max)

- [ ] Confirm the conflict is specifically on workspace membership, crate path ownership, or `registry/catalog` rows.
- [ ] Confirm each PR has a single bounded crate/context move.
- [ ] Confirm no PR introduces top-level `modules/`, `services/`, or `platform/`.
- [ ] Confirm every queued crate has a unique package name and catalog record path.

If the conflict is broader than workspace membership, route to [per-context-flatten-phase.md](per-context-flatten-phase.md) and split the phase.

---

## Steps

1. ☐ Select one workspace-members PR as the merge head.
   Expected: only one PR owns root `Cargo.toml` at a time.
   If differs: stop all but the merge-head PR.

2. ☐ Rebase or retarget the remaining PRs after the merge head lands.
   Expected: each remaining PR refreshes root `Cargo.toml` and its catalog record from current `main`.
   If differs: do not merge stale workspace-member edits.

3. ☐ Validate the merge-head PR.
   Command: `presubmit` (retired CLI `gate validate architecture-boundaries --self-test && presubmit (retired CLI gate validate) architecture-boundaries`)
   Expected: flat-crates self-test and workspace checks pass.
   If differs: fix the merge-head PR before unblocking the queue.

4. ☐ Validate catalog coverage.
   Command: `cargo run -p tooling-cli-dev-runtime -- catalog validate`
   Expected: every workspace member has a catalog record. Extra catalog records are currently allowed; stricter reverse-sync is a separate policy change.
   If differs: add or fix missing catalog records.

5. ☐ Release the next PR in the queue only after the previous PR lands green.
   Expected: no overlapping root `Cargo.toml [workspace.members]` edits.
   If differs: restore serialization.

---

## Rollback

- Revert only the most recently merged workspace-members PR.
- Restore the matching catalog record state.
- Re-run `presubmit` (retired CLI `gate validate architecture-boundaries`) before merging the next PR.
- If more than one PR landed with conflicting membership, stop the queue and run a phase-level audit from [per-context-flatten-phase.md](per-context-flatten-phase.md).

---

## Verification

- [ ] Merge-head PR has no concurrent root `Cargo.toml [workspace.members]` owner.
- [ ] `presubmit` (retired CLI `gate validate architecture-boundaries --self-test`) passes.
- [ ] `presubmit` (retired CLI `gate validate architecture-boundaries`) passes.
- [ ] `cargo run -p tooling-cli-dev-runtime -- catalog validate` passes.
- [ ] `cargo run -p tooling-cli-dev-runtime -- gate validate cargo-prefix` passes.
- [ ] `cargo run -p tooling-cli-dev-runtime --bin repoctl -- pre-push` passes before closing the queue or the queue records an explicit local-resource blocker plus targeted substitutes.

---

## Post-incident updates

If a queue conflict reaches `main`:
- [ ] Add a row to [MISTAKES-LEDGER.md](../MISTAKES-LEDGER.md).
- [ ] Update this runbook with the missing serialization step.
- [ ] Add a mechanical CI check if humans had to detect the conflict manually.

---

## Audit-chain emission

Each queue invocation emits an engineering evidence event with:
- runbook id: `workspace-members-merge-queue`
- queued PR list
- selected merge order
- root `Cargo.toml` owner per step
- catalog records touched
- final verification command outcomes

---

## Sources scanned

- [ADR-0015](../decisions/ADR-0015-architectural-flattening-target.md)
- [PRD.md §6 and §7](../PRD.md)
- [RELEASE-MANAGEMENT.md](../RELEASE-MANAGEMENT.md)
- [templates/runbook-template.md](../templates/runbook-template.md)
