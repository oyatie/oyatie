---
doc_class: BackfillReplayPosture
microservice: foundry-evidence
status: Accepted
date: 2026-05-17
owner_team: axis-foundry-evidence + ops-security + council-privacy
related_artifacts:
  - microservices/intelligence-evidence/policy/evidence-pack-integrity.md (EPI-11)
  - microservices/intelligence-evidence/runbooks/evidence-pack-rebuild.md
  - microservices/audit-chain/policy/seal-integrity.md (substrate; SI-07)
doc_status: published
---

# foundry-evidence — backfill + replay posture

## Principle: no historical-period writes (substrate invariant)

Per `microservices/audit-chain/policy/seal-integrity.md` SI-07, audit-chain NEVER accepts emits into historical periods; the Merkle root for a sealed period is immutable. foundry-evidence inherits this invariant: backfill of older invocations into older periods is **forbidden by substrate**.

Instead, three honest patterns are supported:

## Pattern 1 — Late signal aware of original pack

Trigger: signal arrives after pack assembled; `materially_significant=true`.

Mechanism: `runbooks/evidence-pack-rebuild.md` issues a NEW pack at the current period with `supersedes_pack_ref=<original_pack_id>`. Both packs visible in `evidence-query` (with `include_superseded=true`). Original carries `superseded_by_pack_ref` after the rebuild.

This pattern preserves chain integrity (no historical write) AND records the late signal in a fully audit-emitted way.

## Pattern 2 — Bulk replay from foundry-runtime backlog

Trigger: foundry-runtime was unable to call record_invocation for a window (e.g., bridge outage on the runtime side, not substrate side). foundry-runtime has its own WAL.

Mechanism:
1. foundry-runtime drains its WAL.
2. Each replayed invocation envelope carries `original_invocation_ts` AND `replay_observed_at`.
3. record_invocation accepts the envelope at the CURRENT period (because that's when foundry-evidence is observing it).
4. The pack carries both timestamps; `original_invocation_ts` is the actual invocation moment; `accepted_at` and `period_id` reflect when foundry-evidence sealed.
5. Regulator-export profiles use `original_invocation_ts` for compliance scoping (EU AI Act Art. 12 wants the actual operation time).

This is honest about both "when did it happen" and "when did we record it"; no fabrication.

## Pattern 3 — Forensic rebuild after detection of historical defect

Trigger: a historical pack is discovered to have an incorrect signal join (e.g., FM-06 eval-evidence join correctness violation).

Mechanism:
1. Original pack remains in the chain at its historical period (cannot be deleted; substrate-immutable).
2. NEW pack issued at current period via `runbooks/evidence-pack-rebuild.md` with `supersedes_pack_ref=<original>` + justification + 2-person rule.
3. Regulator-export consumers reading the affected `(tenant, time_range)` see BOTH the original AND the superseding pack; the bundle assembly logic prefers superseding packs and includes a `supersession_history` array per `framework_profile`.

## Pattern 4 — Schema migration (multi-version coexistence)

Trigger: pack schema version bump (e.g., `1.0` → `1.1`) with a sunset window.

Mechanism:
1. Sunset window opens (typically 90 days); both versions accepted.
2. New writes use new schema_version; old packs remain at their original version.
3. evidence-query + regulator-export know both versions.
4. Sunset closes; old version no longer accepted for new writes; reads of old packs continue indefinitely.

No retroactive rewrite of old packs.

## What is FORBIDDEN

- Direct INSERT into Postgres bypassing the recorder + bridge.
- Direct write to substrate WORM bypassing audit-chain emit RPC.
- Mutating an already-sealed pack (would break Merkle integrity).
- Writing into a historical period_id (substrate refuses).
- Soft-deleting a pack via SQL UPDATE (only retention-cascade RPC permitted).

## CI gates

- LEAN lane `historical-period-write-forbidden` blocks any code that constructs a `period_id` from anything other than substrate's current-period source.
- LEAN lane `direct-pg-write-forbidden` blocks any code path that bypasses the recorder.
- LEAN lane `pack-mutation-forbidden` blocks any mutation of a sealed pack row.

## Operator commands

```bash
# Pattern 1: rebuild for late signal
oya foundry-evidence pack-rebuild submit \
  --invocation-id <id> \
  --approver <spiffe> \
  --justification-file justification.txt

# Pattern 2: bulk replay (called automatically by foundry-runtime; documented for forensic transparency)
# (no operator command — runtime SDK handles this)

# Pattern 3: forensic rebuild
oya foundry-evidence pack-rebuild submit \
  --invocation-id <id> \
  --reason forensic_signal_join_defect \
  --approver <spiffe>

# Pattern 4: schema migration is Helm/Cargo-controlled; not a runtime operator command.
```

## Audit emission

Every backfill / replay / rebuild action is itself audit-emitted with a distinct `event_class`:
- `foundry.evidence.replay.bulk.v1` (Pattern 2; emitted per replayed invocation).
- `foundry.evidence.pack.superseded.v1` (Pattern 1 + 3; with justification_hash + approver).
- `foundry.evidence.schema.migration.v1` (Pattern 4; emitted on sunset boundary changes).

## Honest representation in evidence-query

- `include_superseded` query param; default `false` (returns only canonical view).
- Set `true` to see full supersession history.
- Bundle profiles always include both original + superseder; supersession is regulatory evidence.

## ADR-0133 honesty annotation

The backfill posture is a deliberate trade-off: chain integrity over operational convenience. The claim-matrix declares "no historical-period writes" as an asserted invariant. Any commit-claim that requires retroactive pack mutation is refused by `hyperscaler-maturity-claims` lane.

## References

- `microservices/intelligence-evidence/policy/evidence-pack-integrity.md` EPI-02, EPI-06, EPI-11.
- `microservices/audit-chain/policy/seal-integrity.md` SI-07.
- ADR-0024 (eval-evidence integration).
- ADR-0028 (audit-chain).
- ADR-0133 (claim honesty).
