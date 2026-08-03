# Ultragoal durable-records INDEX — session 2026-06-09/10 (ddaf47cc)

Single traceable manifest of every durable artifact from this session. Entry point: read `RESUME-PROMPT.md` to continue; this INDEX is the map.

## Ultragoal core (`.omc/ultragoal/`)
- `RESUME-PROMPT.md` — **START HERE next session.** Concise load-bearing resume (read-order, governance, state, decisions-waiting, durable goal, first actions).
- `CHECKPOINT-2026-06-10.md` — full state snapshot + Addendum (merge-conflict glob-fix) + Addendum 2 (buck2 cache cold/warm).
- `brief.md` — mission + AMENDMENTS 1–13 + WIND-DOWN (all founder directives, binding).
- `goals.json` + `ledger.jsonl` — 13-story durable plan (G001 complete, G002 active).
- `friction-ledger.jsonl` — **51 frictions = the G011 enforcement backlog**; each row pairs a friction with its enforcement_fix. The pipeline-as-product spec.
- `RECOMMENDATION-corpus-liveness-graph.md` — fundamental fix for decay/drift/staleness/missed-directives (idea-refine output; → ADR-0541 Proposed, PR #666).
- `merge-train.md` — manual merge-train protocol (interim until Tide queue).
- `team-briefs/` — substrate-lane + G12-consolidation lane briefs (omc team, legacy tmux runtime).

## Dispatch protocol (added 2026-06-10; AMENDED same day: Fable (Claude) dispatch supersedes codex-exec — READ BY EVERY TEAMMATE; ralplan-consensus r2)
- **Founder directive 2026-06-10 (later same day):** execution teammates AND review agents dispatch as Fable subagents; review agents load /using-superpowers + /using-agent-skills and run /oh-my-claudecode:ultraqa with the rubric (Torvalds + hyperscaler lenses). In-flight codex lanes ran to completion only; no new codex WORKER dispatches (codex exec remains available as a supplementary review lens per AMENDMENT 14a).
- `TEAMMATE-PREAMBLE.md` — standing onboarding pack: §0 meta-skill bootstrap (propagates to nested subagents), read-first list, standing protocols (settle protocol, buck2-only, isolation, SHA-pinned pre-open review filter), §2.5 self-serve ralplan planning (owned-arch + hyperscaler litmus), §3 premise gate + lane-loop-until-depleted + six escalation triggers + final-message contract, §3.1 dispatch ledger semantics.
- `RUBRIC-torvalds-review.md` — standing 7-axis adversarial review rubric (incl. hyperscaler lens axis 6, owned-architecture axis 7) for teammates' pre-open filter and leader verification passes.
- `dispatch-ledger.jsonl` — append-only lane orchestration ledger (single-writer=leader; PR state wins reconciliation); seeded 2026-06-10 with in-flight lanes.
- `../plans/dispatch-protocol-ralplan.md` + `../plans/dispatch-protocol-architect-verdict.md` — the ralplan consensus record for this protocol (fable architect + codex critic, dual-model; r3); status pending founder approval.

## Lane specs + briefs (2026-06-10 G011 session; each brief = a dispatched contract)
- `SPEC-G011-glob-members.md` — shipped: #661/#662 (ADR-0538; lane briefs were inline in the team task text).
- `SPEC-G011-freshness-gate.md` + `BRIEF-g011-freshness-worker.md` — shipped: #664 (ADR-0539).
- `SPEC-G011-target-parity-gate.md` + `BRIEF-g011-target-parity-worker.md` — shipped: #665 (ADR-0540).
- `BRIEF-g011-settle-automation-worker.md` — shipped: #668.
- `BRIEF-g011-enforcement-liveness-worker.md` — shipped: #669 (FRIC-012 closed).
- `BRIEF-g011-rust-test-wiring-generator.md` — IN FLIGHT (burn-down batch-1).
- `BRIEF-g011-main-checkout-guard.md` — IN FLIGHT (FRIC-022 guard, first Rust-hook pattern).

## Session memories (`~/.claude/projects/-Users-jasonlee-Developer-oyatie/memory/`)
`MEMORY.md` indexes 21 founder-directive memories. Load-bearing set: root-goal-json-stale · authz-rbac-abac-pbac · proven-patterns-rust-reimplementation · all-cli-retirement · cloud-native-k8s-native-ops · pipeline-universal-product · intelligence-destination-cloud · ports-designed-for-owned-stack · buck2-primary-build · quality-torvalds-review-discipline · automation-maximalism-staleness · enforcement-layering · rust-hooks-scripts-tools · testing-standards-multilayer · rust-purity-buck2-everywhere · w2-ast-tree-sitter-transitional · quorum-not-etcd-class · cloud-idp-vs-oya-product-identity · buck2-cache-cold-vs-warm · corpus-liveness-graph.

## Research corpora (`.omc/research/`)
- `hyperscaler-research-wave1-20260609.json` — identity/authz/cells/console/control-plane/o11y/delivery-fabric × 5 companies.
- `hyperscaler-research-wave2-20260609.json` — KMS/network/data/storage/compute/billing/messaging/gateway/audit × 5.
- `repo-survey-synthesis-20260609.json` — FD-001 + code-reality survey + goal stories.
- `tree-sitter-evaluation-20260610.json` — W2 AST parity checklist (bespoke-rowan decision).
- `corpus-liveness-precedents-20260610.json` — 104-agent deep-research corpus, 25/25 claims verified 3-0 (Kythe/Glean/SCARF/Sensenmann/g3doc + refutations); evidence base of ADR-0541.

## Repo-tracked landings (on `dev`, survive independently)
23 PRs merged this session (see CHECKPOINT). Key: G001 contract-lock + ADR-0536/0537 (Proposed); FD-001 substrate G02–G09 + G12 consolidation; #659 buck2 cache-key fix. `delete_branch_on_merge=true` set on repo.

## Cross-session anchors (repo root, tracked)
- `HANDOFF.md` — prior-session founder-authoritative handoff (this session's outputs layer on top; .omc/ is the live working state).
- `specs/root-hub-pointers.json` · `CLAUDE.md` · `AGENTS.md` — canonical entry surfaces.

## Open decisions (founder, door:one-way) — tracked in tasks #15/#16 + RESUME-PROMPT
identity-architecture ratification · ADR-0536/0537 sign-off · corpus-liveness-graph research→ADR · FRIC-003 signing enforcement · #644 XPROXY sanction-or-close.
- `SHARED-KERNEL-CANDIDATES.md` — founder AST-reuse directive operationalized: 6 consolidation classes (BUCK parsing, ADR front-matter, OWNERS walk, gate-baseline model, JSONL ledgers, hook events) with ordering; tasks #7/#10 linked.
