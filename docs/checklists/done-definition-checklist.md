---
doc_class: Checklist
checklist_id: CHK-DONE
status: Accepted
date: 2026-05-12
purpose: |
  Extends `docs/AGENTS.md §Done-Definition checklist` D1-D20 with
  per-change-class variants. D1-D18 are walked before every PR is declared
  "ready to merge"; D19-D20 are walked after squash merge before product-complete.
  Re-walk at the loop-cancellation boundary per `docs/AGENTS.md §Long-running
  loop rule`.
enforcing_fitness_lane: guard-pr-merge-review.mjs + per-lane CI status
owner_team: cloud-governance + architecture-governance
related:
  - docs/AGENTS.md
  - docs/templates/pull-request-template-v2.md
  - docs/checklists/pre-flight-checklist.md
  - docs/checklists/pr-review-checklist.md
adrs_cited:
  - ADR-0052  # inventory ledger (D17 migration rows)
  - ADR-0053  # sanctioned primitives (agent tool use)
doc_status: published
---

# Done-Definition Checklist

> Walk core rows D1-D18 before declaring a PR ready to merge. After squash
> merge, walk D19-D20 before declaring the work product-complete. Then walk the
> **per-change-class** rows that apply. Each row carries a typed verification
> path: lane name, command, or explicit `(advisory)` marker.

## Core (apply to every change class)

- [ ] **D1** All `pre-flight-checklist.md` items checked. *Verification:* per-item reviewer audit on PR.
- [ ] **D2** Affected canonical docs updated in same PR per `docs/DOC-CATALOG.md`. *Lane:* `oya-governance-doc-catalog`.
- [ ] **D3** New ADRs (if any) authored from `docs/templates/adr-template-v2.md`. *Lane:* `oya-governance-adr-shape`.
- [ ] **D4** New runbooks (if any) authored from `docs/templates/runbook-template-v2.md`; discoverable in `docs/RUNBOOKS-INDEX.md`. *Lane:* `oya-governance-runbook-index-resolves`.
- [ ] **D5** New capabilities (if any) ship record + eval set + autonomy tier + audit topic + Cosign signing. *Lane:* `oya-governance-capability-publish`.
- [ ] **D6** New schemas carry `data_class` per field. *Lane:* `oya-governance-data-class`.
- [ ] **D7** Per-PR fitness lanes pass: `oya-governance-{license, data-class, cohesion, glossary, adr-citation, brand-residue, bypass, flat-crates, runbook-index-resolves, doc-catalog}`. *Verification:* CI status check.
- [ ] **D8** Reviewer agent ran; verdict in `## Code Review`. *Lane:* `guard-pr-merge-review.mjs`.
- [ ] **D9** Buck2/cloud-ci test lane passes for the affected Rust targets. *Verification:* output in `## Verification`.
- [ ] **D10** Buck2/cloud-ci lint lane passes for the affected Rust targets. *Verification:* output.
- [ ] **D11** `cargo deny check` passes. *Verification:* output.
- [ ] **D12** Protected PR status `oya-ci-required` is green before merge. *Verification:* CI/status URL; optional local pre-push or legacy verifier output is shift-left evidence only and never merge authority.
- [ ] **D13** Performance changes carry benchmark + ≥2 stress scenarios. *Lane:* `oya-governance-perf-evidence`.
- [ ] **D14** Schema migrations ship up + down + dry-run + per-tenant + per-cell rollback. *Lane:* `oya-governance-schema-migration`.
- [ ] **D15** PR has 5 canonical H2s; `## Code Review` at merge. *Lane:* `traceability-validator`.
- [ ] **D16** Audit-chain emission `EVT-*` ID in `## Evidence`. *Lane:* `oya-governance-audit-emission`.
- [ ] **D17** `docs/MISTAKES-LEDGER.md` row added if mechanical prevention shipped. *Lane:* `oya-governance-mistakes-ledger-cite`.
- [ ] **D18** `docs/CHANGELOG.md` row added if canonical doc touched. *Lane:* `oya-governance-changelog-row`.
- [ ] **D19** Post-merge product-completion packet recorded after squash merge:
  promoted commit `oya-ci-required` status URL, rollout verification, rollback note,
  observability/golden-signal check, browser UX/user-story evidence, and
  release-governance/release-note impact (Release Please applies only when a live repo
  config/workflow exists). *Verification:* PR comment or release evidence bundle
  linked from `## Evidence`; see `docs/checklists/pre-merge.md §After merge`.
- [ ] **D20** Agent observations harvested before closeout: review chat,
  review-agent output, scratch/workspace notes, PR comments, and Kanban
  comments; dedupe against active cards; then create/link follow-up, maturity,
  feature-improvement, or fix cards, or document duplicates/no-action rationale.
  New/linked cards MUST include: source context, classification,
  affected card/PR/artifact, acceptance criteria, verification path,
  suggested owner/profile, and dependencies/conflict notes. *Verification:* Kanban
  card/comment links or explicit duplicate/no-action note in the completion packet.

## Per-change-class additions

### feature
- [ ] DD authored per `docs/templates/design-doc-template.md` and accepted before any IP claim. *Lane:* `oya-governance-design-doc-shape` (advisory until DD-lane lifts).
- [ ] Eval set / property tests / fuzz scaffolding present for new parser/serializer paths. *Lane:* `oya-governance-qa-coverage`.
- [ ] Feature-flag wired; canary rollout planned per `docs/RELEASE-MANAGEMENT.md`. *Lane:* `oya-governance-release-readiness`.

### bugfix
- [ ] Regression test added that fails on the buggy commit and passes on the fix. *Lane:* `oya-governance-qa-coverage`.
- [ ] `MFL-NNNN` row added if class of bug is recurrence-prone. *Lane:* `oya-governance-mistakes-ledger-cite`.

### refactor
- [ ] Public API surface unchanged (per `cargo public-api`). *Command:* `cargo public-api --diff`.
- [ ] `cargo-semver-checks` clean. *Command:* `cargo semver-checks check-release`.
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
- [ ] `regional-packs/<pack>/` updated. *Lane:* `oya-governance-pack-coverage`.

## Loop-cancellation re-walk

Per `docs/AGENTS.md §Long-running loop rule`, when operating in Ralph / autopilot / ultrawork / team loop, **MUST** re-walk every applicable row above against the latest state before exiting. Loops **MUST NOT** exit silently. Cancel via `/oh-my-claudecode:cancel` only when (a) change complete and verified, OR (b) loop structurally blocked.

If any row is unchecked, the change is not "done." Loop back; do not declare success.
