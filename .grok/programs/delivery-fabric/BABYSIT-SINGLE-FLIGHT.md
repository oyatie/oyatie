# Babysit single-flight (process fix)

**Status:** fabric law (2026-08-06)  
**Not merge authority.** Complements `FABRIC-REFINED.md` W3.

## Defect this fixes

Multiple workers polled the same open PR (session loops + W3 ticks + ad-hoc shells + implement lane), which:

- multiplies merge races
- amplifies tip thrash when dual-critic pins push
- converts “drive reorg” into “wait on CI” and stalls path-disjoint implement

## Rule (one owner per PR)

| Field | Law |
|-------|-----|
| **Babysit owner** | Exactly one: fabric class `pr-babysit-lanes` (W3) |
| **Implement (W2)** | After open/push: set lane `ready_for_babysit` + `pr=N`; **do not** run multi-minute `gh pr checks` / sleep poll loops |
| **Interactive chat** | One-shot status only unless explicitly sole babysit owner for that PR |
| **Claim key** | `(pr_number, head_sha)` — at most one armed merge-waiter |

## Single-flight claim file

Path: `.grok/mm-runs/_fabric/babysit-claims.json` (local runtime; not merge authority)

```json
{
  "1578": {
    "head": "<full sha>",
    "owner": "pr-babysit-lanes",
    "armed_at": "2026-08-06T15:00:00Z",
    "mode": "merge_on_green"
  }
}
```

**W3 tick:**

1. List open PRs + board soft_red/ci_red.
2. For each PR needing work: if claim exists for **same head** and owner is W3 and age < 45m → **heartbeat only**, do not spawn a second merge-waiter.
3. If head moved or claim expired/missing → claim, one babysit agent, one optional background waiter.
4. Cap **2** PRs per tick; prefer binding reds over soft.

## Implement lane (W2) handoff

After PR open + dual-critic packet:

- `status=ready_for_babysit` (not `waiting_ci` idle)
- Heartbeat note: `handoff pr=N head=…` — **no** arm re-poll
- Next W2 tick claims a **different** path-disjoint ready lane

## Anti-patterns (fail closed)

| Anti-pattern | Correct |
|--------------|---------|
| Session + W3 both poll same PR | Session implements or one-shots |
| W3 tick re-arms waiter every 5m | Claim same head → skip arm |
| Dual-critic tip push while CI in progress | Local pin until `oya-ci-required` green; one signed push |
| Treat “singleton free” wait as whole-program serial | Only next `*-move-plan.json` is serial; delete/rebrand/refactor parallel |

## Merge authority (unchanged)

`oya-ci-required` green + dual-critic APPROVE for tip → `mm-drive merge` (human GH APPROVE not required under supervise).
