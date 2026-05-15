---
doc_class: Checklist
checklist_id: CHK-DONE
status: Accepted
date: 2026-05-12
purpose: |
  Extends `docs/AGENTS.md §Done-Definition checklist` D1-D18 with per-change-class variants. Walked before every PR is declared "ready to merge" and re-walked at the loop-cancellation boundary per `docs/AGENTS.md §Long-running loop rule`.
enforcing_fitness_lane: guard-pr-merge-review.mjs + per-lane CI status
owner_team: axis-foundry + council-architecture
related:
  - docs/AGENTS.md
  - docs/templates/pull-request-template-v2.md
  - docs/checklists/pre-flight-checklist.md
  - docs/checklists/pr-review-checklist.md
adrs_cited:
  - ADR-0052  # inventory ledger (D17 migration rows)
  - ADR-0053  # sanctioned primitives (agent tool use)
  - ADR-0054  # scaffold-claim pattern (grit symbols)
doc_status: published
---

# Done-Definition Checklist

> Walk **all** core rows D1-D18 (from `docs/AGENTS.md`). Then walk the **per-change-class** rows that apply. Each row carries a typed verification path: lane name, command, or explicit `(advisory)` marker.

## Core (apply to every change class)

- [ ] **D1** All `pre-flight-checklist.md` items checked. *Verification:* per-item reviewer audit on PR.
- [ ] **D2** Affected canonical docs updated in same PR per `docs/DOC-CATALOG.md`. *Lane:* `oya-foundry-fitness-doc-catalog`.
- [ ] **D3** New ADRs (if any) authored from `docs/templates/adr-template-v2.md`. *Lane:* `oya-foundry-fitness-adr-shape`.
- [ ] **D4** New runbooks (if any) authored from `docs/templates/runbook-template-v2.md`; discoverable in `docs/RUNBOOKS-INDEX.md`. *Lane:* `oya-foundry-fitness-runbook-index-resolves`.
- [ ] **D5** New capabilities (if any) ship record + eval set + autonomy tier + audit topic + Cosign signing. *Lane:* `oya-foundry-fitness-capability-publish`.
- [ ] **D6** New schemas carry `data_class` per field. *Lane:* `oya-foundry-fitness-data-class`.
- [ ] **D7** Per-PR fitness lanes pass: `oya-foundry-fitness-{license, data-class, cohesion, glossary, adr-citation, brand-residue, bypass, flat-crates, runbook-index-resolves, doc-catalog}`. *Verification:* CI status check.
- [ ] **D8** Reviewer agent ran; verdict in `## Code Review`. *Lane:* `guard-pr-merge-review.mjs`.
- [ ] **D9** `cargo nextest run --workspace --all-features --no-fail-fast` passes. *Verification:* output in `## Verification`.
- [ ] **D10** `cargo clippy --workspace --all-features --all-targets -- -D warnings` passes. *Verification:* output.
- [ ] **D11** `cargo deny check` passes. *Verification:* output.
- [ ] **D12** `repoctl pre-push` passes. *Verification:* output.
- [ ] **D13** Performance changes carry benchmark + ≥2 stress scenarios. *Lane:* `oya-foundry-fitness-perf-evidence`.
- [ ] **D14** Schema migrations ship up + down + dry-run + per-tenant + per-cell rollback. *Lane:* `oya-foundry-fitness-schema-migration`.
- [ ] **D15** PR has 5 canonical H2s; `## Code Review` at merge. *Lane:* `traceability-validator`.
- [ ] **D16** Audit-chain emission `EVT-*` ID in `## Evidence`. *Lane:* `oya-foundry-fitness-audit-emission`.
- [ ] **D17** `docs/MISTAKES-LEDGER.md` row added if mechanical prevention shipped. *Lane:* `oya-foundry-fitness-mistakes-ledger-cite`.
- [ ] **D18** `docs/CHANGELOG.md` row added if canonical doc touched. *Lane:* `oya-foundry-fitness-changelog-row`.

## Per-change-class additions

### feature
- [ ] DD authored per `docs/templates/design-doc-template.md` and accepted before any IP claim. *Lane:* `oya-foundry-fitness-design-doc-shape` (advisory until DD-lane lifts).
- [ ] Eval set / property tests / fuzz scaffolding present for new parser/serializer paths. *Lane:* `oya-foundry-fitness-qa-coverage`.
- [ ] Feature-flag wired; canary rollout planned per `docs/RELEASE-MANAGEMENT.md`. *Lane:* `oya-foundry-fitness-release-readiness`.

### bugfix
- [ ] Regression test added that fails on the buggy commit and passes on the fix. *Lane:* `oya-foundry-fitness-qa-coverage`.
- [ ] `MFL-NNNN` row added if class of bug is recurrence-prone. *Lane:* `oya-foundry-fitness-mistakes-ledger-cite`.

### refactor
- [ ] Public API surface unchanged (per `cargo public-api`). *Command:* `cargo public-api --diff`.
- [ ] `cargo-semver-checks` clean. *Command:* `cargo semver-checks check-release`.
- [ ] Linus good-taste audit row in `## Code Review`. *(advisory)*

### migration
- [ ] Schema up + down + dry-run + per-tenant + per-cell rollback shipped. *Lane:* `oya-foundry-fitness-schema-migration`.
- [ ] Inventory row added per `docs/checklists/inventory-update-checklist.md` (ADR-0052). *Lane:* `oya-foundry-fitness-inventory-tracker`.

### docs
- [ ] `docs/DOC-CATALOG.md` trigger event named in PR `## Issue`. *Lane:* `oya-foundry-fitness-doc-catalog`.
- [ ] `doc-style-reviewer` agent verdict captured. *Lane:* `guard-pr-merge-review.mjs`.

### chore
- [ ] No production behavior change. *(advisory)*
- [ ] CI lanes still green. *Verification:* CI status.

### capability
- [ ] Capability record at `product-control/capabilities/<id>.yaml` validates against schema. *Lane:* `capability-schema-validator`.
- [ ] Eval set (golden + adversarial + linguistic) min-pass-rate met. *Lane:* `oya-foundry-fitness-capability-publish`.
- [ ] Cedar policy + runtime gate present when tier ≥ T2. *Lane:* `oya-foundry-fitness-autonomy-ceiling`.
- [ ] Cosign keyless OIDC signature + Syft SBOM + SLSA L2+ provenance attested. *Lane:* `oya-foundry-fitness-supply-chain`.

### plugin
- [ ] Plugin manifest validates. *Lane:* `oya-foundry-fitness-plugin-manifest`.
- [ ] No external network calls outside declared allowlist. *Lane:* `oya-foundry-fitness-plugin-network`.

### runbook
- [ ] `docs/RUNBOOKS-INDEX.md` entry added; row resolves to real file. *Lane:* `oya-foundry-fitness-runbook-index-resolves`.
- [ ] `last_verified` date set; drill ETA noted. *(advisory)*

### ADR
- [ ] `adr-template-v2.md` shape complete (Decision/Drivers/Alternatives/Why-chosen/Consequences/Follow-ups). *Lane:* `oya-foundry-fitness-adr-shape`.
- [ ] `docs/ADR-INDEX.md` updated. *Lane:* `oya-foundry-fitness-adr-citation`.

### pack-update (regional pack)
- [ ] Regulator-watch impact named per pack. *Lane:* `oya-foundry-fitness-compliance-matrix`.
- [ ] `regional-packs/<pack>/` updated. *Lane:* `oya-foundry-fitness-pack-coverage`.

## Loop-cancellation re-walk

Per `docs/AGENTS.md §Long-running loop rule`, when operating in Ralph / autopilot / ultrawork / team loop, **MUST** re-walk every applicable row above against the latest state before exiting. Loops **MUST NOT** exit silently. Cancel via `/oh-my-claudecode:cancel` only when (a) change complete and verified, OR (b) loop structurally blocked.

If any row is unchecked, the change is not "done." Loop back; do not declare success.
