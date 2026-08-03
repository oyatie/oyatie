# Worker brief — G011 settle-protocol automation (one worker, one PR)

Friction: FRIC-1781082000/FRIC-1781100200 in `/Users/jasonlee/Developer/oyatie/.omc/ultragoal/friction-ledger.jsonl` (read the FRIC-1781100200 rows — they ARE the spec). Three freshness-gate catches in one day were the same trap: faces regenerate from TRACKED paths and scm-facts records per-path last_touch_commit, so (a) regenerating with untracked new files bakes wrong faces, and (b) committing content + faces together self-invalidates. The terminating protocol: commit content first → materialize → FACES-ONLY settle commit. Make this mechanical.

Work ONLY in `/Users/jasonlee/oyatie-worktrees/g011-settle-automation` (branch `agent/g011-settle-automation`, base = current origin/dev). NEVER touch `/Users/jasonlee/Developer/oyatie`. Never run omc orphan-cleanup.

## Deliverables (one PR)
1. **Remediation text teaches the protocol** in `cloud/cloud-ci/gates/oya-cloud-ci-freshness-app/src/lib.rs`: the `generated_face_stale` finding detail and `render_remediation()` must state, verbatim-ish: commit content changes first; faces regenerate from TRACKED paths; never mix content and regenerated faces in one commit; then run the materialize command; then commit the faces-only diff. Keep `FACE_REMEDIATION_COMMAND` as the command constant; add a `FACE_SETTLE_PROTOCOL` constant for the protocol text. Update/extend the existing remediation tests (there is a `remediation_includes_exact_sanctioned_commands` test — extend, do not weaken).
2. **Settle binary**: a second `[[bin]]` in the same crate (same single concern: freshness check + repair), e.g. `oya-cloud-ci-face-settle`. Behavior:
   - Default (check mode): if tracked NON-face changes are dirty or staged → exit 1 printing the protocol; else regenerate faces (reuse the crate's existing regeneration module — same buck2 producer targets as CI) and print whether faces are stale, exit 1 if stale (with protocol), 0 if clean.
   - `--settle`: require clean non-face tree (else exit 1 + protocol); regenerate; if face diffs exist, `git add` ONLY the four face paths and print the suggested commit command. With `--settle --commit`, additionally create the SSH-signed commit `chore: settle generated cloud-ci faces` (commit only if staged set is exactly face paths; abort otherwise).
   - Production code: no unwrap/expect/panic, `#![forbid(unsafe_code)]` in the bin root (the existing main.rs LOW finding — fix it there too while present).
3. BUCK target for the new bin + tests: dirty-tree refusal fixture, faces-only staging fixture (use a temp git repo fixture like the existing cli_fixtures pattern), protocol-text assertions. Cited tests must exist.
4. Doc touch: `docs/oya-ci/gate-catalog.md` freshness row gains one line pointing at the settle binary. No ADR needed — this implements ADR-0539's remediation pathway; PR body cites ADR-0539 + FRIC-1781100200.

## Rules
- buck2 build + buck2 test = green signal (cargo supplementary). Lock refresh ONLY via `cargo metadata >/dev/null` (no new crate — no lock change expected; the new bin lives in the existing crate).
- OBEY THE PROTOCOL YOU ARE AUTOMATING: content commits first; then materialize; then a faces-only settle commit (your own PR will be checked by the freshness gate).
- SSH-signed commits; `git push -u origin agent/g011-settle-automation`; open PR to dev with `gh`, body citing ADR-0539 + FRIC-1781100200 + buck2 evidence.
