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
  - docs/templates/pull-request-template.md
  - docs/checklists/pre-flight-checklist.md
  - docs/checklists/pr-review-checklist.md
adrs_cited:
  - ADR-0052  # inventory ledger (D17 migration rows)
  - ADR-0053  # sanctioned primitives (agent tool use)
doc_status: published
---

# Done-Definition Checklist

> Walk **all** core rows D1-D18 (from `docs/AGENTS.md`). Then walk the **per-change-class** rows that apply. Each row carries a typed verification path: lane name, command, or explicit `(advisory)` marker.

## Core (apply to every change class)

- [ ] **D1** All `pre-flight-checklist.md` items checked. *Verification:* per-item reviewer audit on PR.
- [ ] **D2** Affected canonical docs updated in same PR per `docs/DOC-CATALOG.md`. *Lane:* `oya-governance-doc-catalog`.
- [ ] **D3** New ADRs (if any) authored from `docs/templates/adr-template.md`. *Lane:* ADR-shape validation.
- [ ] **D4** New runbooks (if any) authored from `docs/templates/runbook-template-v2.md`; discoverable in `docs/RUNBOOKS-INDEX.md`. *Lane:* `oya-governance-runbook-index-resolves`.
- [ ] **D5** New capabilities (if any) ship record + eval set + autonomy tier + audit topic + Cosign signing. *Lane:* `oya-governance-capability-publish`.
- [ ] **D6** New schemas carry `data_class` per field. *Lane:* `oya-governance-data-class`.
- [ ] **D7** Per-PR fitness lanes pass: `oya-governance-{license, data-class, cohesion, glossary, adr-citation, brand-residue, bypass, flat-crates, runbook-index-resolves, doc-catalog}`. *Verification:* CI status check.
- [ ] **D8** Reviewer agent ran; verdict in `## Code Review`. *Lane:* `guard-pr-merge-review.mjs`.
- [ ] **D9** Buck2 test evidence passes for touched/full target set. *Verification:* output in `## Verification`.
- [ ] **D10** Buck2 build/lint/static evidence passes with warnings treated as failures where applicable. *Verification:* output.
- [ ] **D11** Dependency/license/SBOM policy evidence passes through the current Buck2 policy target or registry-backed lane. *Verification:* output.
- [ ] **D12** `oya-ci-required` target context is green before merge; GitHub Actions bridge checks are shadow/compatibility evidence. *Verification:* required-context evidence.
- [ ] **D13** Performance changes carry benchmark + ≥2 stress scenarios. *Lane:* `oya-governance-perf-evidence`.
- [ ] **D14** Schema migrations ship up + down + dry-run + per-tenant + per-cell rollback. *Lane:* `oya-governance-schema-migration`.
- [ ] **D15** PR has 5 canonical H2s; `## Code Review` at merge. *Lane:* `traceability-validator`.
- [ ] **D16** Audit-chain emission `EVT-*` ID in `## Evidence`. *Lane:* `oya-governance-audit-emission`.
- [ ] **D17** `docs/MISTAKES-LEDGER.md` row added if mechanical prevention shipped. *Lane:* `oya-governance-mistakes-ledger-cite`.
- [ ] **D18** `docs/CHANGELOG.md` row added if canonical doc touched. *Lane:* `oya-governance-changelog-row`.

## Per-change-class additions

### feature
- [ ] DD authored per `docs/templates/design-doc-template.md` and accepted before any IP claim. *Lane:* `oya-governance-design-doc-shape` (advisory until DD-lane lifts).
- [ ] Eval set / property tests / fuzz scaffolding present for new parser/serializer paths. *Lane:* `oya-governance-qa-coverage`.
- [ ] Feature-flag wired; canary rollout planned per `docs/RELEASE-MANAGEMENT.md`. *Lane:* `oya-governance-release-readiness`.

### bugfix
- [ ] Regression test added that fails on the buggy commit and passes on the fix. *Lane:* `oya-governance-qa-coverage`.
- [ ] `MFL-NNNN` row added if class of bug is recurrence-prone. *Lane:* `oya-governance-mistakes-ledger-cite`.

### refactor
- [ ] Public API surface unchanged. *Command:* lane-owned Buck2 API compatibility target or documented equivalent.
- [ ] Semver/API compatibility evidence clean through Buck2 or a documented dual-build adapter when required.
- [ ] Linus good-taste audit row in `## Code Review`. *(advisory)*

### migration
- [ ] Schema up + down + dry-run + per-tenant + per-cell rollback shipped. *Lane:* `oya-governance-schema-migration`.
- [ ] Inventory row added per `docs/checklists/inventory-update-checklist.md` (ADR-0052). *Lane:* `oya-governance-inventory-tracker`.

### docs
- [ ] `docs/DOC-CATALOG.md` trigger event named in PR `## Issue`. *Lane:* `oya-governance-doc-catalog`.
- [ ] `doc-style-reviewer` agent verdict captured. *Lane:* `guard-pr-merge-review.mjs`.

### chore
- [ ] No production behavior change. *(advisory)*
- [ ] CI lanes still green. *Verification:* CI status.

### capability
- [ ] Capability record at `registry/capability-templates/<id>.yaml` validates against schema. *Lane:* `capability-schema-validator`.
- [ ] Eval set (golden + adversarial + linguistic) min-pass-rate met. *Lane:* `oya-governance-capability-publish`.
- [ ] Cedar policy + runtime gate present when tier ≥ T2. *Lane:* `oya-governance-autonomy-ceiling`.
- [ ] Cosign keyless OIDC signature + Syft SBOM + SLSA L2+ provenance attested. *Lane:* `oya-governance-supply-chain`.

### plugin
- [ ] Plugin manifest validates. *Lane:* `oya-governance-plugin-manifest`.
- [ ] No external network calls outside declared allowlist. *Lane:* `oya-governance-plugin-network`.

### runbook
- [ ] `docs/RUNBOOKS-INDEX.md` entry added; row resolves to real file. *Lane:* `oya-governance-runbook-index-resolves`.
- [ ] `last_verified` date set; drill ETA noted. *(advisory)*

### ADR
- [ ] `adr-template-v2.md` shape complete (Decision/Drivers/Alternatives/Why-chosen/Consequences/Follow-ups). *Lane:* `oya-governance-adr-shape`.
- [ ] `docs/ADR-INDEX.md` updated. *Lane:* `oya-governance-adr-citation`.

### pack-update (regional pack)
- [ ] Regulator-watch impact named per pack. *Lane:* `oya-governance-compliance-matrix`.
- [ ] `docs/regional-packs/<pack>/` documentation updated and any sovereign overlay artifact registered in `registry/sovereign-cloud/per-pack-overlay-status.tsv`. *Lane:* `oya-governance-pack-coverage`.

## Loop-cancellation re-walk

Per `docs/AGENTS.md §Long-running loop rule`, when operating in Ralph / autopilot / ultrawork / team loop, **MUST** re-walk every applicable row above against the latest state before exiting. Loops **MUST NOT** exit silently. Cancel via `/oh-my-claudecode:cancel` only when (a) change complete and verified, OR (b) loop structurally blocked.

If any row is unchecked, the change is not "done." Loop back; do not declare success.
