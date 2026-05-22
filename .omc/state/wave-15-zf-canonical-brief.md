# Wave 15-ZF canonical brief — doctrine propagation for ADR-0346..0349

**Date**: 2026-05-21
**Wave**: 15-ZF (doctrine propagation, NOT implementation)
**Predecessor**: Wave 15-ZG (PR #177 — cargo-nextest fix)
**Successor**: Wave 15-ZA/ZB/ZD/ZE (implementation sub-waves)
**Goal**: Propagate ADR-0346/0347/0348/0349 doctrine into every artifact type across the corpus.

## What "propagation" means here

NOT implementation. NOT new code. Each subagent annotates / scaffolds / cross-references the artifacts so that:
- Every µservice's PRD/ARCH/manifest knows about the new doctrine
- Every applicable runbook references the new lanes
- Every Cedar policy mentions the new actor roles
- Every machine-readable mirror reflects the new ADR entries

Implementation (actually writing `oya verify` extension, running the foundry-fitness rename, declaring `sharding_automation` manifest field bodies, authoring Jenkins/ArgoCD OpenTofu modules) is downstream Wave 15-ZA/ZB/ZD/ZE.

## Ground reads (MANDATORY before any edit)

Each subagent MUST read these files in this order BEFORE touching any artifact:

1. `.omc/state/session-snapshot-2026-05-21-pre-compact-final.md` — session state + landed ADRs
2. `.omc/state/oyatie-architecture-2026-05-21.md` — canonical architecture authority chain
3. `tools/hooks/_canonical-primitives.md` — canonical primitives (oya git, OpenAPI 3.2.0, Valkey, etc.)
4. The 4 new ADRs themselves:
   - `docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md`
   - `docs/decisions/ADR-0347-foundry-fitness-to-governance-bulk-rename.md`
   - `docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md`
   - `docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md`
5. `CLAUDE.md` (root) — project rules

## Artifact-type → agent mapping (23 codex agents, 16 artifact types)

Each agent owns one or two artifact types across the corpus.

| Agent | Artifact type(s) | Touch count est. |
|---|---|---:|
| ZF-1 | docs/machine-readable/decisions.json + decisions.tsv refresh | 2 files |
| ZF-2 | docs/ADR-INDEX.md + docs/decisions/INDEX.md + ADR-INVENTORY.tsv | 3 files |
| ZF-3 | tools/hooks/_canonical-primitives.md + tools/agent-skills/AGENTS.md | 2 files |
| ZF-4 | specs/root-hub-pointers.json + specs/master-plan-sequencing.json amendment | 2 files |
| ZF-5 | docs/AGENTS.md + docs/AGENTS-OPERATING-CONTRACT.md | 2 files |
| ZF-6 | 77× microservices/<ms>/PRD.md — add §X "Doctrine refs: ADR-0346..0349" cross-ref block | 77 |
| ZF-7 | 77× microservices/<ms>/ARCH.md — section update for sharding_automation + Jenkins/ArgoCD CI/CD context | 77 |
| ZF-8 | 77× microservices/<ms>/manifest.json — add `sharding_automation` block scaffold (per ADR-0348 D-1 schema, defaults to declared-disabled until Wave 15-ZD wires bodies) | 77 |
| ZF-9 | 77× microservices/<ms>/IPs/ — add IP-WAVE-15-ZD-sharding-automation.md per µservice | 77 |
| ZF-10 | 77× microservices/<ms>/runbooks/ — auto-rebalance + hot-split + cold-merge runbook scaffolds | 77 |
| ZF-11 | 77× microservices/<ms>/threat-models/ — add §autosharding-event-drift threat model entry | 77 |
| ZF-12 | 77× microservices/<ms>/dpia/ — add §automation-event-driven-data-flow DPIA entry | 77 |
| ZF-13 | 77× microservices/<ms>/contracts/ — annotate OpenAPI components for sharding_automation event surface | 77 |
| ZF-14 | 77× microservices/<ms>/cedar/ — add policies for cell-orchestrator principals + Jenkins/ArgoCD service principals | 77 |
| ZF-15 | 77× microservices/<ms>/slos/*.openslo.yaml — add SLO targets for autosharding events (hot-split latency, rebalance migration time) | 77 |
| ZF-16 | 77× microservices/<ms>/capabilities/ — add capability rows for `autosharding.{autosharded, auto_rebalance, dynamic_sharding}` | 77 |
| ZF-17 | 77× microservices/<ms>/README.md — section §Doctrine references list ADR-0346..0349 | 77 |
| ZF-18 | 77× microservices/<ms>/migration-playbooks/ — Wave 15-ZD migration playbook scaffold | 77 |
| ZF-19 | .github/workflows/ — Jenkinsfile mirror parity comment block on every CI workflow file | ~50 |
| ZF-20 | scripts/check.sh + bin/oya wrapper — add `--include-deferred` flag plumbing for `oya verify` per ADR-0346 D-8 | ~5 files |
| ZF-21 | registry/foundation-bypasses + registry/governance-migration-ledger.tsv — add ADR-0346..0349 entries | 2 files |
| ZF-22 | docs/templates/pull-request-template.md + docs/templates/adr-template.md — mention ADR-0346 pre-push contract | 2 files |
| ZF-23 | docs/products/*/PRD.md — add §"AI substrate + Cellular automation" cross-references where applicable | ~10 files |

## Per-agent contract (mandatory steps)

1. **Read 5 ground docs** (above) before any edit
2. **Read the target ADR file(s)** thoroughly
3. **For each artifact under your ownership**:
   - Find the canonical insertion point (e.g., §"Doctrine references", §"Related ADRs")
   - Insert the cross-reference + brief 2-3 sentence summary
   - Use the EXACT wording pattern from the ADR's `enforced_by:` list — no paraphrasing
4. **Validate locally**:
   - `cargo run -q -p oya-dev-cli -- gate validate adr-citation --docs-dir docs --decisions-dir docs/decisions`
   - For machine-readable artifacts: `cargo run -q -p oya-dev-cli -- doc adr-index --write`
   - For manifests: `cargo run -q -p oya-dev-cli -- gate validate cohesion`
5. **Commit + push** (no PR — direct to branch `post-merge-2026-05-18`):
   ```bash
   git add -A microservices/ docs/ specs/ tools/ .github/ registry/ bin/ scripts/
   git commit -m "Wave 15-ZF-<N>: propagate ADR-0346..0349 into <artifact-type>"
   git push origin post-merge-2026-05-18
   ```
6. **Report**: 1-paragraph summary — files touched, validation status, commit SHA

## Coordination

23 agents push to the same branch. Each agent works in distinct paths (per the table above), so file-level conflicts are unlikely. Push races handled via `git pull --rebase origin post-merge-2026-05-18` then retry.

## Reasoning effort

`codex exec -c model_reasoning_effort=xhigh --skip-git-repo-check --sandbox danger-full-access --dangerously-bypass-approvals-and-sandbox` (mandatory per `feedback_codex_dispatch_canonical_2026_05_21`).

## Dispatch ceiling

Per `feedback_dispatch_ceiling_claude_only_2026_05_20`: codex-only, max 8 simultaneous. 23 agents → 3 batches of 8 (last batch = 7).

**Batch 1 (foundational)**: ZF-1, ZF-2, ZF-3, ZF-4, ZF-5, ZF-20, ZF-21, ZF-22 — touch 1-5 files each, fast. These land canonical-primitives + index entries that downstream batches reference.

**Batch 2 (µservice manifest + IP)**: ZF-6, ZF-7, ZF-8, ZF-9, ZF-16, ZF-17, ZF-18, ZF-23 — each touches 77 µservices in their lane. Sequential per-µservice within an agent; parallel across agents.

**Batch 3 (µservice doctrine surface)**: ZF-10, ZF-11, ZF-12, ZF-13, ZF-14, ZF-15, ZF-19 — runbooks, threat models, DPIA, contracts, Cedar, SLOs, workflows. 77 each.

## Verification

After all 23 agents complete:
- Orchestrator runs full corpus gates:
  - `oya gate validate adr-citation`
  - `oya gate validate cohesion`
  - `oya gate validate authority-cohesion`
  - `oya doc adr-index --write`
  - `oya doc inventory --write`
  - `cargo nextest run --workspace`
- Expected: all green (no regressions; new annotations are additive)

## What "done" looks like

- Every artifact across 77 µservices + corpus-level docs carries explicit cross-references to ADR-0346/0347/0348/0349 where applicable
- Machine-readable mirrors (decisions.json, ADR-INDEX.md) include all 4 ADRs
- adr-citation gate passes
- Wave 15-ZA/ZB/ZD/ZE can proceed with implementation knowing the doctrine surface is already wired
