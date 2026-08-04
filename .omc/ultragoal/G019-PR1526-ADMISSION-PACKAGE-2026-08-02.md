# PR #1526 admission package — 2026-08-02

State: `CANDIDATE_OPEN_NOT_ADMITTED`
Admission: **blocked** until all gates below flip true.

## Exact object
- PR: https://github.com/jason931225/oyatie/pull/1526
- Head: `fd2cb9d2f0d47f4bcd84c1c76e1953e7be440ecc`
- Purpose: shard Oya YAML extraction faces (`shard_size=256`) so promoted `//oya:corpus-yaml-facts` no longer fails ARG_MAX/config-input scale
- Local exact-head: corpus-index-coverage unittest 44/44 + gate 19/19 PASS
- Contract self-review: `.omc/ultragoal/G019-PR1526-CONTRACT-SELF-REVIEW-2026-08-02.md` (structural only)
- Independent review: `FAILED_TRANSPORT` (no APPROVE)

## Admission checklist (all required)
- [ ] Candidate `gate · affected-set` SUCCESS on exact head
- [ ] Candidate `oya-ci-required` SUCCESS on exact head
- [ ] Independent review APPROVE on exact head (self-review insufficient)
- [ ] No open load-bearing REQUEST_CHANGES
- [ ] Merge method: squash only after above
- [ ] Post-merge: observe promoted tip for corpus-yaml-facts class green (not candidate green, not merge event)

## Residual contract notes
- After merge, `//oya:corpus-yaml-facts` label is first shard only; full union coverage is the corpus-index-coverage gate obligation
- Do not treat buck2 job green alone as affected-set green
- Attempts 1–2 were RUNNER_LOST_COMMUNICATION mid step 7; attempt 3 must reach true terminal

## Post-admit sequence
1. Observe promoted tip green
2. Restack #1523 (procedure already authored; no hand ADR-INDEX edit)
3. Rerun #1528 candidate CI if still needed
4. Only then W0-C..E / G023 path
