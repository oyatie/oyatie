# Integ envelope judgments (northstar-judgment-split)

Volatile Amendment B / naming judgments live here — **not** inlined into
`specs/integ-branch-envelopes.json`.

## Layout

| Path | Role |
|------|------|
| `naming-sweep.json` | Bootstrap naming_sweep ledger (moved off envelopes) |
| `<unit-id>.yaml` | Per-unit judgment files carried **in-diff by the destination integ PR** |

## Rule

Envelopes JSON keeps topology, grammar, hubs, freeze prefixes, anti_drift, and
merge_windows. Claim validates the judgment file present in the same diff when a
path change requires `judgment_status=done`.

Do not re-serialize pending judgments through the sole-owner envelopes tip.
