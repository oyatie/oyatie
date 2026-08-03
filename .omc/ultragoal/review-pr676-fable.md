# Review of Record — PR #676 (agent/g011-target-parity-deliteral) — AMENDED HEAD f3a2dd6d5

Reviewer: Fable (Claude), fresh-context Torvalds-lens. Base dev @ 3ae4f2ea9.
Head f3a2dd6d5 (content e6ab09d4b + faces-settle f3a2dd6d5). Supersedes the earlier APPROVE of
pre-amendment head 9ef0e0635. Commissioned by FRIC-1781116000; amended per the codex supplementary
lens (review-pr676-codex.md BLOCK).

## VERDICT: APPROVE (head f3a2dd6d5)

The amendment correctly closes both codex findings AND preserves the FRIC-1781116000 de-literalization
goal, because it splits the retired literal's two conflated roles: identity-pinning moves to set-equality
against the committed baseline, while growth-oracle duty moves to an independent DOWN-ONLY ceiling that
slice PRs never edit. Both buck2 targets pass at the amended head; settle protocol, signatures, and
baseline byte-equality all hold.

## Correction to my prior review (codex was right on F3)

My earlier review (review-pr676-fable.md @ 9ef0e0635) APPROVEd and rated the literal->set-equality swap
"strictly stronger." That was WRONG on one axis. This self-test runs the producer LIVE over today's
corpus (OYA_CI_PRODUCER_BIN via $(exe ...) in BUCK), so the old `assert_eq!(len, 614)` was a REAL
independent growth oracle: a constant not derived from any generated artifact, asserting against the live
measurement. Replacing it wholesale with `unwired_tests == baseline_keys` made the only growth check
depend on the baseline — which, until the merge-base ratchet lands (FRIC-1781112000), is PR-LOCAL and
launderable (regenerate faces + baseline in one PR, set-equality still passes). Codex finding 1 (lost
independent oracle) and finding 2 (verdict could emergently flip to Green on drift) were both correct.
The amendment fixes exactly these.

---

## Findings (file:line — severity — confidence)

1. tests/target_parity.rs:122-133 — independent down-only growth tripwire — POSITIVE / HIGH.
   `const DEBT_CEILING: usize = 614;` + `assert!(unwired_tests.len() <= DEBT_CEILING, ...)`. This restores
   the live-corpus growth oracle codex F1 required, NOT derived from any generated artifact, so it cannot
   be laundered by a same-PR baseline regen. Closes codex finding 1.

2. tests/target_parity.rs:135-137 — unconditional Verdict::Red restored — POSITIVE / HIGH.
   `assert_eq!(evaluate(&face).verdict, Verdict::Red);` replaces the baseline-emptiness-driven expectation.
   The GO-LIVE flip to Green is now forced to be its own reviewed change, never an emergent side effect of
   producer/baseline drift. Closes codex finding 2. Confirmed against src/lib.rs:51-55 (Green iff
   violations empty) — today's corpus is non-empty so Red is the correct invariant while the campaign runs.

3. tests/target_parity.rs:109-121 — set-equality + fail-closed baseline read — POSITIVE / HIGH.
   Retained from the prior head and unchanged: `unwired_tests == baseline_keys` pins identity (catches the
   constant-cardinality identity-swap class the bare literal was blind to), and every JSON step is
   expect/unwrap (Tier-3, ADR-0083) so a malformed/missing baseline PANICS = RED. No silent-pass branch.
   This now COMPOSES with the ceiling: set-equality owns identity, ceiling owns growth.

4. DEBT_CEILING does NOT reintroduce the slice-PR conflict — POSITIVE / HIGH — the load-bearing judgment.
   Old: `assert_eq!(len, 614)` is EXACT equality — a slice PR fixing one member drops live len to 613,
   fails `== 614`, forcing all 12 slice PRs to edit the literal (the 12-way conflict + staleness trap
   FRIC-1781116000 commissioned removing). New: `len <= 614` is a DOWN-ONLY inequality — fixing members
   drops len to 613/612/... all of which satisfy `<= 614` with ZERO edit to the const. The ceiling moves
   only to ratchet down as a deliberate reviewed step, never mechanically per-slice. The conflict magnet
   is NOT reintroduced. This is the precise distinction that lets the amendment satisfy codex's growth-
   oracle requirement and FRIC-1781116000's de-literalization simultaneously.

5. Faces settle — parity-neutral — POSITIVE / HIGH.
   f3a2dd6d5 faces diff field histogram: 70× last_touch_commit, 2× source_inputs_digest, 2× head_time_secs;
   ZERO has_test_code / has_rust_test_target / has_buck / member_path churn. Producer-mechanical (the
   content commit SHA e6ab09d4b propagating into last_touch_commit + digest recompute). No laundering room.

6. gate-baseline.generated.json — byte-identical to dev — POSITIVE / HIGH.
   `git diff origin/dev...HEAD -- '*gate-baseline.generated.json'` is EMPTY. The baseline set-equality pins
   against is the reviewed dev baseline; no smuggled regeneration.

7. Settle protocol + signatures — compliant — POSITIVE / HIGH.
   e6ab09d4b = content commit, test file ONLY (1 file), FIRST. f3a2dd6d5 = faces-only settle, LAST (2
   generated files). Both Good ED25519 SSH (SHA256:5grGNUtX...). Diff vs dev is exactly the three claimed
   files.

8. evidence/friction-ledger.jsonl — absent / unverifiable — LOW / MEDIUM — NON-BLOCKING.
   No friction-ledger.jsonl in the worktree; FRIC IDs appear only in test comments. Gap analysis derived
   from gate mechanics in src/lib.rs instead. Evidence gap, not a code defect.

---

## Do the ceiling+Red amendments close the codex findings without reintroducing the slice-PR conflict? — YES

- Codex finding 1 (lost independent live-corpus growth oracle): CLOSED by the DEBT_CEILING tripwire
  (finding 1), which is independent of every generated artifact and asserts against the live measurement.
- Codex finding 2 (verdict could emergently flip to Green on drift): CLOSED by restoring unconditional
  Verdict::Red (finding 2); the one-way GO-LIVE transition is now a required separate reviewed change.
- Slice-PR conflict / staleness trap (FRIC-1781116000): NOT reintroduced, because the ceiling is a
  down-only `<=` inequality, not an exact `==`. Slice PRs that wire members lower the live count under the
  ceiling without editing it (finding 4). The original 12-way merge-conflict magnet was the EXACT-equality
  property, which is gone.

Net: the amended test now carries THREE separable, correctly-scoped assertions — identity (set-equality vs
baseline), growth (independent down-only ceiling), and verdict (unconditional Red) — where the bare literal
conflated identity+growth into one exact-equality assertion that was both too brittle (slice conflicts) and,
once removed, too weak (no independent oracle). This is the right factoring.

---

## Commands run (exact output lines)

$ git rev-parse HEAD -> f3a2dd6d510117a63351fb99bc91f0735263fe74
$ buck2 test //...:oya-cloud-ci-target-parity-app-unittest //...:oya-cloud-ci-target-parity-app-gate
  ✓ Pass: ...-unittest (0.0s)  test result: ok. 5 passed; 0 failed; ... finished in 0.00s
  ✓ Pass: ...-gate (12.3s)     test target_parity_face_reports_live_corpus_debt ... ok
                               test result: ok. 1 passed; 0 failed; ... finished in 12.16s
  Tests finished: Pass 2. Fail 0. Timeout 0. Fatal 0. Skip 0. Omit 0. Infra Failure 0. Build failure 0
  (clean compile — no unused-const warning on DEBT_CEILING)
$ git diff origin/dev...HEAD -- '*gate-baseline.generated.json'  -> EMPTY (byte-identical to dev)
$ faces field histogram -> 70 last_touch_commit, 2 source_inputs_digest, 2 head_time_secs; 0 parity fields
$ git show --stat e6ab09d4b -> tests/target_parity.rs ONLY (1 file)   [content commit, first]
$ git show --stat f3a2dd6d5 -> accounting-registry + scm-facts generated (2 files) [faces-only, last]
$ git log --show-signature -2 -> both Good ED25519 SHA256:5grGNUtX...

The gate target ran the REAL producer over the live corpus and the amended assertions (set-equality +
ceiling + unconditional Red) all passed end-to-end against today's measurement.

## Residual risk
The DEBT_CEILING (614) is a hand-maintained constant: if the campaign wires members but nobody ratchets
the ceiling down, the tripwire silently loosens (it still blocks GROWTH past 614, but no longer pins the
campaign's progress floor) — staleness re-enters as an under-tightened ceiling rather than as a merge
conflict. That is a deliberate, acceptable trade (loose-but-safe vs brittle), and the set-equality + the
eventual merge-base ratchet (FRIC-1781112000) remain the tighter floor. The deepest residual remains
unchanged from my prior review: full cross-PR laundering protection (regenerate faces+baseline together)
still lives in the firewall runner's merge-base ratchet_regression + the gate-baseline.signoff.json door,
which are outside this diff and were not re-run here — the ceiling now backstops the GROWTH direction of
that gap locally, but the merge-base oracle is still the system of record FRIC-1781112000 owns.
