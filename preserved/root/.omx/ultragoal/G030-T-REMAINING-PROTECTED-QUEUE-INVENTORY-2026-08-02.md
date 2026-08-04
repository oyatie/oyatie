# G030-T remaining protected queue inventory + TSV accounting correction — 2026-08-02

State: **PLANNING_ONLY — INVENTORY + ONE-ROW ACCOUNTING CORRECTION; NO DELETION/ACTIVATION**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Supplements G030-S and G030-N. No repository path or policy was changed.

## Accounting defect found

G030-N promoted three tip paths under `registry/release/` and `registry/capabilities/`:

1. `registry/capabilities/foundry-internal.json` — inside the G030 focus universe;
2. `registry/release/images.yaml` — inside the focus universe;
3. `registry/release/evidence-packs.tsv` — **outside** the focus universe.

The durable G030 focus family is exactly `md + yaml/yml + json + toml` (baseline count 13,959; specs+registry partition 1,176). A `.tsv` path is a real tracked artifact and may have a real consumer edge, but it is **not a member of the 1,176-row partition**. Promoting it into the partition inflated `GRAPH_WIRED_INPUT` by one and deflated `POLICY_PROTECTED_MACHINE_ARTIFACT` by one from G030-N onward.

The evidence-pack consumer edge remains true as domain fact; only the census arithmetic is corrected.

## Corrected totals

| Stage | Reported after stage | Corrected after stage |
|---|---|---|
| after M | 152 / 924 / 100 | unchanged |
| after N | 152 / 927 / 97 | **152 / 926 / 98** (only two in-universe promotions) |
| after O | 152 / 940 / 84 | 152 / 939 / 85 |
| after P | 152 / 948 / 76 | 152 / 947 / 77 |
| after Q | 152 / 956 / 68 | 152 / 955 / 69 |
| after corrected R | 152 / 982 / 42 | 152 / 981 / 43 |
| after S | 152 / 993 / 31 | **152 / 992 / 32** |

Anti-vacuity: 152 + 992 + 32 = 1,176.

Remaining protected queue: **19 fixture + 13 non-fixture = 32**.

## Exact remaining non-fixture residual (13)

All 13 exist at the immutable tip. None is under the canonical-JSON governed `specs/**/*.json` selector (they are registry paths or non-JSON specs companions).

| # | Path | Prior proof | Retention rationale |
|---|---|---|---|
| 1 | `registry/accounts/schema.json` | G030-J | unwired schema; stale `parser_ref`; no exact-path reader |
| 2 | `registry/accounts/README.md` | G030-J | contract documentation only |
| 3 | `registry/vcs/concurrent-safe-paths.yaml` | G030-L | ADR-0363 frozen historical companion; no current reader |
| 4 | `registry/vcs/event-router.yaml` | G030-L | ADR-0363 frozen; retired webhook receiver ABSENT |
| 5 | `registry/vcs/webhook-delivery-log.json` | G030-L | empty retired scaffold; receiver ABSENT |
| 6 | `registry/vcs/README.md` | G030-L | frozen contract documentation |
| 7 | `registry/foundation-bypasses/README.md` | G030-M | skipped by foundation `.yaml` extension filter |
| 8 | `registry/capabilities/foundry-supervisor.toml` | G030-N | no TOML loader defaults to this path |
| 9 | `registry/release/supply-chain/README.md` | G030-N | evidence walk accepts only `.yaml`/`.yml` |
| 10 | `registry/merge-queue-tick-log.json` | G030-P | prose-only; retired merge-queue writer ABSENT |
| 11 | `registry/claim-matrix/ops-portal.json` | G030-P | catalog-only; planned checker ABSENT |
| 12 | `specs/policy/cedar-scope-schema.md` | G030-R | Markdown; outside canonical-JSON selector |
| 13 | `specs/products/RETIREMENT.md` | G030-R | Markdown retirement companion |

This closes the 13-versus-12 inventory mismatch: the earlier “12 non-fixture” figure inherited the TSV inflation.

## Fixture residual (19) unchanged

G030-F / G030-G residual still holds:

- calendar PRD replay: 15 — colocated Python consumer, not Buck/protected-CI wired;
- CRATEADR owner-batch: 3 — ADR-retained, no measured machine consumer;
- DR RTO/RPO: 1 — transitional Python bridge, not Buck/protected-CI wired.

## Out-of-universe note

`registry/release/evidence-packs.tsv` remains a real release-evidence-pack default with a parser edge. It is simply not one of the 1,176 focus rows. Future corpus work that expands the focus family to include TSV must version the baseline schema; until then, do not move the 1,176 counters for it.

## Residual disposition

- Delete candidates: **0**
- No dual-negative consumer + authority proof has declassified any residual row to `DARK_BUREAUCRACY`.
- Remaining non-fixture rows are owner-retained companions, frozen historical evidence, unwired schemas, ABSENT planned checkers, or Markdown contracts.
- Fixture residual remains protected pending Buck-native consumers or owner rulings.
- G030-D nested `.omc/state` under docs and known-broken lifecycle owner rulings remain outside this residual inventory's delete authority.

## Review boundary

A final-queue independent audit failed with encrypted-content transport error and remains **FAILED_TRANSPORT_NOT_APPROVE**. This inventory is coordinator mechanical reconstruction from durable proofs + tip membership + focus-family definition.

## Non-actions

- No residual path edited or deleted.
- No G028 push/apply; no G023 deletion; no #1523 restack push.
- No cluster or canonical dirty checkout mutation.
- No claim that out-of-universe TSV is unwired.
