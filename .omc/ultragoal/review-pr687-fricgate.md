# Independent Fresh-Context Review — PR #687 (G013: friction-ledger closed-loop accounting meta-gate)

**Reviewer:** LEADER's fresh-context Fable reviewer of record (independent of the lane's pre-open filter)
**Subject head pinned:** `7a42dadf0de1e9e0c13ab2af6603b6b9177fc0af` (branch `agent/g013-fric-accounting-gate`)
**Worktree:** /Users/jasonlee/oyatie-worktrees/g013-fric-meta-gate
**ADR:** ADR-0544 (Proposed, door:one-way)
**Date:** 2026-06-11
**Lens:** ultraqa rigor + Torvalds (hostile inspection, intent AND execution verified separately, no credit for narration) + hyperscaler (would this be accepted as a merge-blocking governance gate?)

---

## 1. BUILD TRUTH (mandatory — verified by execution, not reading)

- `buck2 build //cloud/cloud-ci/gates/oya-cloud-ci-friction-accounting-app/...` → **BUILD SUCCEEDED** (exit 0).
- `buck2 test //cloud/cloud-ci/...` → **Pass 38, Fail 0, Timeout 0, Fatal 0, Skip 0, Omit 0, Infra Failure 0, Build failure 0** (exit 0).
- Per-target counts for the gate crate (re-run isolated):
  - `oya-cloud-ci-friction-accounting-app-unittest` → **16 passed, 0 failed**.
  - `oya-cloud-ci-friction-accounting-app-gate` (live-corpus + RED fixtures) → **10 passed, 0 failed**, including `live_friction_ledger_meets_the_closed_loop_contract`, all 7 RED fixtures, and the adversarial `red_baseline_is_shrink_only_new_debt_breaks_set_equality`.

No failures anywhere in the cloud-ci tree. CRITICAL build-gate satisfied.

---

## 2. CONTENT-ASSERT (scope / contamination)

`git diff origin/dev..HEAD --name-only` = 17 files, ALL sanctioned:

| Class | Files |
|---|---|
| GATE-DIR | BUCK, Cargo.toml, friction-accounting-policy.json, friction-accounting-baseline.json, src/lib.rs, tests/friction_accounting.rs |
| ADR | docs/decisions/ADR-0544-...md |
| REGISTRATION | .github/workflows/oya-ci-required.yml, docs/oya-ci/gate-catalog.md |
| CHANGELOG | docs/CHANGELOG.md |
| EVIDENCE | evidence/audit-chain.jsonl, evidence/multispectrum/g013-...json |
| CARGO.LOCK | Cargo.lock (+7 lines) |
| FACE(regen) | accounting-registry-app/{accounting-registry, decision-crosswalk, gate-baseline, scm-facts}.generated.json |

- **Zero out-of-scope files.**
- **Zero deletions** (`--diff-filter=D` empty) → no reverts of #685/#686/#644.
- #685's `tools/hooks/main-checkout-guard.sh` exception in `rust-first-automation-policy.json:364` is **INTACT**.
- Generated faces are pure regeneration: the only semantic change is ADR-0544 registered in the decision crosswalk (decision_count 364→365). No cross-lane contamination.
- The live ledger `.omc/ultragoal/friction-ledger.jsonl` (68 physical rows) is **NOT modified by the PR** — the gate observes it, does not edit it. Correct.

Content-assert: **CLEAN.**

---

## 3. NON-VACUITY + ANTI-LAUNDERING (the core)

### 3a. Live-repo test is real, not a stub
`tests/friction_accounting.rs::live_friction_ledger_meets_the_closed_loop_contract` (lines 66-146) actually:
- collects the policy-declared real ledger via `collect_observed_frictions(&root, &policy)` walking up to repo root (`specs/root-hub-pointers.json` sentinel),
- asserts `row_count >= 60` (live census floor — guards against an empty/silently-broken read passing),
- runs the real `evaluate_keyed`, asserts 4 frozen-empty codes are EMPTY on the live corpus, asserts the 4 shrink-only codes equal the committed baseline by **`BTreeSet` set-equality** (not counts), and enforces independent reviewed ceilings.
This is a genuine self-test over today's real ledger, not a stub.

### 3b. RED fixtures — every violation class fails closed (verified executing)
All present and passing without a filesystem:
- `red_unregistered_status_fails_closed` (line 184) — invalid status.
- `red_duplicate_primary_id_fails_closed` (line 193) — duplicate id.
- `red_blank_enforcement_fix_fails_closed` (line 202) — undisposed + missing-field.
- `red_closed_without_evidence_fails_closed` (line 211) — closed-without-evidence.
- `red_accepted_risk_without_evidence_fails_closed` (line 219).
- `red_orphan_update_only_friction_fails_closed` (line 229) — the update-only evasion the prior CRITICAL review caught.
- `red_baseline_is_shrink_only_new_debt_breaks_set_equality` (line 256) — drives the REAL evaluator with legacy+new debt and proves a NEW key breaks baseline set-equality (not a std-lib tautology).
Not a green-only suite. Non-vacuous.

### 3c. Anti-laundering paths attempted (independent re-derivation of kernel semantics in Python against the real policy + real ledger)
- **(iv) append a well-formed open friction** → GREEN. Logging is never punished. CORRECT.
- **(iv) append an UNDISPOSED row (blank enforcement_fix)** → RED: `friction_no_disposition` + `friction_missing_required_field`. Blocks. CORRECT — this is the whole point.
- **(iii) flip a row to terminal WITHOUT evidence** → RED: `friction_closed_without_evidence`. Blocks.
- **(i) brand-new unknown status** → RED: `friction_unknown_status` (frozen-empty). Blocks.
- **(ii) same-PR baseline inflation** → the baseline + ceilings are same-PR-editable, BUT: (1) ceilings are hand-fixed literal constants in test source (`tests/...rs:112-117`), NOT derived from any generated artifact; (2) the baseline is NOT producer-regenerated; (3) growing it to absorb new debt requires a REVIEW-VISIBLE edit to a frozen file. The merge-base structural fix is **DEFERRED-and-documented** (FRIC-1781112000), explicitly named in ADR-0544 §Decision (lines 112-119) and the baseline `_comment`. Per the verdict rule this is acceptable (MEDIUM) because the laundering path is review-visible, not silent.

I independently re-derived the full finding set from the real ledger: **all 4 shrink-only codes match the committed baseline EXACTLY**, and all 3 frozen-empty codes (`friction_unknown_status`, `friction_duplicate_primary_row`, `friction_no_disposition`) are genuinely empty. The baseline is honest, not inflated to pre-absorb debt.

### 3d. Append-vs-undispose distinction
The kernel folds event-sourced rows by id: a primary + later update rows is legitimate (not a duplicate, not an orphan); a NEW primary with no disposition / a terminal with no evidence / an unknown status all fail closed. The append-allowed / undisposed-blocked distinction — the entire thesis — is implemented and proven.

---

## 4. PACK-SHAPE / UNIVERSALITY (founder R0)

- `grep` over `src/lib.rs`: every `.omc/ultragoal/friction-ledger` and `FRIC-` string lives ONLY in doc comments (lines 3, 13, 31) and `#[cfg(test)]` fixtures (lines 466+). **No repo-specific literal in executable kernel code.**
- Production paths reference only generic schema keys (`ledger_path`, `gate_id`, `status_taxonomy`, `required_primary_fields`, `terminal_requires_evidence`, ...). All repo-specifics — ledger path, the 40+ entry status taxonomy, evidence rules — are DATA in `friction-accounting-policy.json`.
- The kernel fixes the ROW SCHEMA field-names (id/seen_at/status/status_update/friction/enforcement_fix/evidence) — this is the documented engine contract (ADR §Decision lines 71-75; policy `product_contract`), not a per-repo value. An adopting repo maps its ledger onto these names and repoints the policy. R0 satisfied.

---

## 5. DEFERRAL HONESTY

ADR-0544 documents its deferrals without enforcement theater:
- Merge-base shrink-only structural anti-laundering: DEFERRED, named FRIC-1781112000, §Decision lines 112-119; the ADR explicitly says "review-visible, not yet structural."
- OWNER half + TIME-BOUND aging of the SRE action-item model: DEFERRED, §Negative lines 186-190 ("SRE precedent half-applied").
- Undeclared buck2 input (warm-cache staleness of the live-corpus test): DISCLOSED §Negative lines 191-198, with the mitigation that the `oya-ci-required` matrix leg re-reads the ledger on a fresh runner every run (merge authority unaffected).
- Taxonomy-widening valve (same-PR taxonomy edit can reclassify around the forcing function): DISCLOSED §Negative lines 173-178.
What shipped is genuinely born-blocking (frozen-empty + shrink-only set-equality + ceilings), not advisory-pretending-to-enforce.

---

## FINDINGS (ranked)

- **CRITICAL:** none.
- **HIGH:** none.
- **MEDIUM-1 (deferred, documented, acceptable):** Same-PR baseline/ceiling/taxonomy laundering is review-visible-only, not structural. Mitigated today by reviewer visibility + non-regenerated baseline + literal ceilings; the merge-base meta-check (FRIC-1781112000) is the named follow-up. Not silently launderable → not CRITICAL. file: friction-accounting-baseline.json, tests/friction_accounting.rs:112-117; ADR-0544:112-119.
- **MEDIUM-2 (disclosed):** Live-corpus gate test reads an undeclared buck2 input (real ledger via repo-root walk), so a warm-cache `buck2 test` can serve a stale verdict. Merge authority unaffected (fresh-runner matrix leg re-reads). file: tests/friction_accounting.rs:22-33; ADR-0544:191-198.
- **LOW-1:** `status_match=prefix` means a new status sharing a registered prefix classifies silently rather than tripping `friction_unknown_status`. Documented trade-off; keep prefixes narrow. policy `product_contract.status_match_tradeoff`; ADR-0544:179-181.
- **LOW-2:** ADR-0544 status is **Proposed** (founder sign-off pending), not Accepted; the gate is landing ahead of formal acceptance. Consistent with the door:one-way authoring convention but worth noting at merge.

No vacuous test, no trivially-exploitable silent laundering path, no hardcoded repo-specifics in the kernel, no build/test failure.

---

## VERDICT: APPROVE
