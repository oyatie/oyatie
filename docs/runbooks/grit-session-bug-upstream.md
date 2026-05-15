---
purpose: Auto-backfilled purpose for grit-session-bug-upstream.md
---

---
doc_class: Runbook
purpose: Field guide for the upstream grit session bug — recognize the FK constraint failure on grit claim, apply the ADR-0054 ICM scaffold-claim fallback, and surface the upstream issue.
runbook_id: RB-GRIT-SESSION-BUG-UPSTREAM
owner_team: axis-foundry
related_adr: docs/decisions/ADR-0054-grit-scaffold-claim-pattern.md
last_validated: 2026-05-14
---

# Grit session bug — upstream tracking + workaround

## Symptom

```
$ grit claim --agent <id> --intent "<slice>" <symbol>
Error: FOREIGN KEY constraint failed
Caused by:
    Error code 787: Foreign key constraint failed
```

`grit status` is otherwise clean (`No active locks`), and the failing
command has a well-formed `<symbol>` argument. This is an upstream SQLite
state-management bug in the grit session store, not an agent-side
authoring error.

## Diagnosis (2 minutes)

1. `grit status` returns "No active locks" or the local set looks consistent.
2. Re-running the same `grit claim` reproduces the FK error.
3. The symbol you're claiming has no other live owner (verify via `icm recall-context "<slice>" --limit 5`).

If all three hold, you've hit the upstream bug.

## Workaround (sanctioned by ADR-0054)

Declare the claim in ICM under the `scaffold-locks-oyatie` topic and
proceed:

```
icm store -t context-oyatie \
  -c "Claiming <symbols> via ADR-0054 FK fallback. Agent: <id>." \
  -i high \
  -k "<phase-id>,grit-fk-fallback,ADR-0054,claim-open"
```

Then perform the work as if grit had granted the claim. Close with:

```
grit done --agent <id>
```

`grit done` is idempotent against unknown agents and will release
cleanly even when the FK error blocked the initial claim.

## Upstream issue

- **Upstream repo**: https://github.com/rtk-ai/grit
- **Filed**: 2026-05-12 by axis-foundry.
- **Status**: open; awaiting upstream maintainer reply on root cause.
- **Workaround validated**: ADR-0054 ICM scaffold-claim has been used in
  ~50+ session claims to date without producing duplicate writes or
  conflicts (every fallback names the affected symbol set in the ICM
  content field, audited via `icm recall-context grit-fk-fallback`).

## When to update this runbook

- When upstream ships a fix → demote the workaround to "historical" and
  reinstate strict `grit claim` paths.
- When a second symptom pattern emerges → add it to "Symptom" above.

## Linus good-taste row

Special cases eliminated by this runbook:
- The fallback path is sanctioned by an ADR, not folklore — agents can
  cite ADR-0054 instead of inventing their own workaround.
- The workaround uses the same `claim → work → done` shape as the happy
  path — `grit done` always runs, so the upstream fix lands as a single
  flip of the diagnosis branch, not a rewrite of the close-out flow.
