---
doc_class: Runbook
shape: Runbook
length_cap: 200
microservice: policy
related_adrs:
  - ADR-0280
inbound_citations:
  - policy/README.md
---

# Runbook — stale policy snapshot in a cell

**Alert:** `policy_snapshot_age_seconds` above its objective for a cell/tenant, or a rise in
`RouteToAuthoritative` verdicts.

**What is NOT happening:** requests are not being authorized from stale state. The invariant
(ADR-0280 §D-13.E) makes staleness deny-or-route, never silently authorize. A stale snapshot is an
*availability* incident, not a *security* one. Treat it as such — do not reach for a bypass.

## Triage order

1. **Is the snapshot signed and current?** Compare the cell's `served_version` against the G plane's
   latest published version. If they match, the age metric is measuring `asserted_at`, and the G
   plane has simply not published lately — that is normal, not an incident.
2. **Is the G plane publishing?** If not, this is a G-plane incident. The cell keeps serving its
   last-known-good snapshot by design. Existing sessions are unaffected.
3. **Is distribution reaching this cell?** `DistributeSnapshot` denials point at
   `signature_verified == false` (F4). A snapshot that fails verification is **refused, not served** —
   check trust-anchor rotation before anything else.
4. **Is this cell authoritative for the tenant?** If not, `RouteToAuthoritative` is correct behaviour
   and the caller should be retrying against the named shard. Rising route verdicts with healthy
   authoritative shards is a routing problem, not a policy problem.

## What you must never do

- **Do not widen the staleness tolerance to clear the alert.** Tolerance is the caller's declared
  safety bound (F3). Raising it converts a refusal into a stale authorization — the exact failure the
  invariant forbids.
- **Do not disable signature verification** to get a snapshot loaded. F4 refuses unverified snapshots
  on both the evaluate and the distribute path, deliberately.
- **Do not hand-edit a snapshot on a node.** Version regression is refused by `swap` (I6); a node
  patched out of band will diverge from every other replica in the cell.

## Escalation

`axis-policy-engine`. If the G plane is down and a tenant needs a policy change, the change waits —
there is no break-glass that publishes unsigned or unsoaked policy (F6, F7), and the soak window is
what makes rollback cheap.
