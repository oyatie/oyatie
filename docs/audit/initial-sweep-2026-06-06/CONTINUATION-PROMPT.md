Continue the oyatie consolidation → enforced-canon → platform program. Assume NO prior context.

READ BEFORE ANY WORK (protocols, procedures, state, backlog — do not skip):
1. linux/docs/audit/initial-sweep-2026-06-06/HANDOFF.md — state, repos, guardrails
2. …/FULL-SCOPE-OF-WORK.md — THE backlog (tracks 0–6), sequenced
3. …/synthesis/decision-record-oyatie-canon.md — every founder ruling (SSOT)
4. …/PHASE-0-FIREWALL-PLAN.md — the active plan
5. source/AGENTS.md — source protocols/regimen
6. source/specs/masterplan.json + source/.omx/backlog/platform-readiness-backlog.md — generated SSOT + program
7. source/.omc/plans/monorepo-consolidation-migration.md — migration regimen
8. …/justify-account-robustness/00-JUSTIFY-ACCOUNT-ROBUSTNESS.md — audit findings

REPOS/GUARDRAILS: MUTATE ~/Developer/source (branch feat/oya-ci-tide); READ audit docs in ~/Developer/linux. Push github-mirror ONLY (never origin/Forgejo). SIGN commits. NO dev PR. Every source mutation is door:one-way → founder sign-off. Forgejo is DROPPED (D2); GitHub is the only interim forge → bespoke Sapling-inspired SCM. Foundry context dead → platform→Intelligence (ADR-0363 §5; Governance stays its OWN service; "all→intelligence" unresolved → follow source until ruled).

DISCIPLINE (non-negotiable): GROUND + SOURCE-BACK every claim/directive (cite path:line) before asserting — any contradiction/drift is a PROCEDURE failure. VERIFY, never assume "done": an item is complete ONLY with evidence (tests/RED-GREEN green, the gate actually blocks a known-bad input, grep/build proof) checked in a SEPARATE verification pass — no false-green. If blocked or contradicting a ruling, surface to the founder; do not guess.

DO THE WORK — clear the ENTIRE backlog in FULL-SCOPE-OF-WORK.md, in dependency order:
- Track 0 — Phase-0 firewall: build the live oya-ci-required producer (GitHub commit-status poster) + the generated accounting-registry + the 4 keystone gates, each RED/GREEN-proven. HALT before any live GitHub ruleset flip (founder-paired credential step).
- Track 1 — Phase-1 amendments (gate-verified, single-owner): A-CI; A-FOUNDRY (foundry→intelligence); A-INTEGRITY (KCMVP restore, dup-0377, phantom-0150, 3-axis status-enum, regenerate indexes); Forgejo-eradication sweep (→ GitHub-interim/Sapling); A-STRUCT (pure-split); A-TASTE; A-IDENTITY; A-VOCAB; Proposed-ledger; CC-1..13.
- Track 2 — doc-reorg (Diátaxis 44→6) + total-accounting (every file owned+justified+reachable) + stale-file audit.
- Tracks 3–6 — net-new ADRs (D1 meta-ADR, effective-dating kernel, pillars E–Q); the 6-repo migration (M-lanes + the dev-merge); the platform build-out; deferred ratchets.

PER ITEM, follow: /plan → research → /spec → /test → /build → /review → fix → /simplify → /ship.

CONTINUE until the ENTIRE backlog is cleared.
