# consent-graph backfill + replay procedures

- Owner: axis-consent-graph + data-axis
- Date: 2026-05-18
- Authority: ADR-0214 §verification, ADR-0003 (audit-chain replay).

## 1. Use cases

1. **Initial deployment of consent-graph onto an existing oyatie cluster**: replay all historical
   tenant relationships (pre-consent-graph era) as "legacy" partner records.
2. **Recovery from Postgres data loss**: PITR restore + audit-chain replay.
3. **Schema migration**: agreement schema_version bump requires re-emission of derived state.
4. **Cross-pointer reconciliation backfill**: rebuild cross_pointers table from audit-chain.
5. **Pulsar tenant onboarding**: when a new region comes online, seed projection topics from existing
   agreement state.
6. **Audit-chain query backfill**: when audit-chain query-stack rolls back schema, re-emit consent-graph
   audit events from local outbox.
7. **Pack overlay change**: a new pack adds a residency rule; existing agreements need re-validation.

## 2. Initial deployment replay

Step-by-step for first-time deployment into an existing oyatie cluster with prior cross-tenant data
flows (informal pre-consent-graph era):

1. Inventory existing cross-tenant API tokens / shared DB views / EDI routes (out-of-band, manual).
2. For each, create a draft `DataSharingAgreement` via `agreement-sdk::draft` with:
   - scope inferred from token's permitted endpoints
   - mode = `Projection` default
   - sovereignty.cross_border_transfer_permitted derived from token's geographic constraint
   - terms.purpose_of_use = "legacy-migration"
3. Send to grantee for acceptance review.
4. On acceptance, deprecate the legacy token (T-30d sunset window).
5. After 30 days, revoke legacy tokens; cross-tenant flow now exclusively through consent-graph.

Tooling: `oya consent-graph backfill from-legacy --token-inventory <path> --grantee-batch <yaml>`.

## 3. Postgres PITR + audit replay

1. Restore Postgres from PITR snapshot (RPO ≤5min).
2. Compare audit-chain entries vs Postgres state (using IP-013 reconciler with custom window):
   - For each audit-chain entry in window where Postgres row missing → re-create row from event payload.
   - For each Postgres row with no matching audit event → quarantine for manual review (rare).
3. Run reconciliation report; verify zero divergences.
4. Resume serving traffic.

## 4. Schema migration replay

When schema_version bumps:
1. Deploy new schema's compiled-policies table in parallel.
2. Worker `schema-migration-worker` reads each active agreement → recompiles Cedar policy per new
   schema → writes to new table.
3. Cutover: enforcement-app switches to new table; old table retained 30d for rollback.
4. Sunset old schema after 6mo per ADR-0064.

## 5. Cross-pointer backfill

If `consent_graph_cross_pointers` table is lost:
1. Query audit-chain for all entries with `event_class` ∈ consent-graph-event-classes within the
   retention window.
2. For each, identify pair by matching agreement_id; reconstruct ChainLinkPair.
3. Recompute paired_hmac from OpenBao pair key.
4. Insert into cross_pointers table.
5. Run IP-013 reconciler to validate completeness.

Throughput: ~50K reconstructions/s with 16 parallel workers.

## 6. Pulsar topic seed for new region

When a new region brings up consent-graph:
1. Identify agreements where new region is a permitted grantee region.
2. For each, run `projection-gateway-sdk::mint` to create the topic in grantor's cluster (already
   exists; no-op if idempotent).
3. Grantee's new-region pods subscribe.
4. ProjectionEmit worker resumes; Pulsar replays last 7d retention to catch up the new subscriber.

## 7. Audit-chain query backfill

If audit-chain query stack is rebuilt:
1. Re-emit consent-graph events from local Pulsar outbox into audit-chain emission-api.
2. audit-chain seals + indexes; consent-graph queries against the rebuilt index.

## 8. Pack overlay change replay

When a new pack residency rule is added:
1. Worker `pack-overlay-replay-worker` reads all active agreements with this pack overlay.
2. Re-run `agreement-domain::resolve_eligible_grantee_regions` with new rules.
3. For agreements that no longer satisfy: emit warning event; require grantor + grantee
   re-acknowledgement; auto-suspend after 14d grace.
4. Audit-chain entries record the policy-rule-version applied to each agreement.

## 9. Replay safety

All replay flows MUST:
- Be idempotent (re-emission of same event yields same audit-chain seq).
- Emit a "replay" audit event class so reconciler can distinguish original from replayed.
- Not violate sovereignty (replay obeys current sovereignty config, even if original agreement
  predated the rule — fail closed if replay would violate).
- Respect rate limits (1K replays/s/region default).

## 10. Replay tooling

`oya consent-graph replay <subcommand>`:
- `oya consent-graph replay reconcile-cross-pointers --window 24h`
- `oya consent-graph replay backfill-from-legacy --token-inventory <path>`
- `oya consent-graph replay schema-migrate --from v1 --to v2 --tenant <id>`
- `oya consent-graph replay seed-region <region>`
- `oya consent-graph replay pack-overlay --pack kr --rule sovereignty-strict-v2`

Each command sealed in audit-chain with `replay_session_id`.

## 11. Verification

- Dry-run mode (`--dry-run`) outputs proposed actions without executing.
- Audit-chain replay shows total events processed + outcome counts.
- Reconciler runs post-replay to confirm zero divergences.

## 12. Risks

- **R**: Replay storm overwhelms audit-chain.
  **M**: Per-command rate limits + per-tenant rate limits.
- **R**: Replay double-emits → audit-chain bloat.
  **M**: Idempotency key (replay-session + event_id) deduplicates.
- **R**: Replay applies stale schema producing inconsistent state.
  **M**: Replay always uses current schema; old schema not supported for replay.
