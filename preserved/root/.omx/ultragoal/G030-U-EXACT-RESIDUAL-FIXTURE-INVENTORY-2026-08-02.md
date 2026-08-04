# G030-U exact residual fixture inventory — 2026-08-02

State: **PLANNING_ONLY — EXACT 19-PATH FIXTURE RESIDUAL LOCKED; NO WIRING/DELETION**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Supplements G030-F and G030-T. No fixture, policy, gate, or cluster state was changed.

## Result

G030-T closed the non-fixture residual at 13 paths and corrected partition arithmetic to **152 / 992 / 32**. This note locks the complementary **19 fixture residual** to exact tip paths and owner/blocker classes. Delete candidates remain 0.

Together:

```text
POLICY_PROTECTED residual = 13 non-fixture + 19 fixture = 32
152 + 992 + 32 = 1176
```

## Exact residual paths (19/19 exist)

### Calendar PRD replay — 15

```text
specs/fixtures/calendar-prd/red-fixtures.json
specs/fixtures/calendar-prd/replay/ac/calendar-ac01-work-event-org-pillar-audit.fixture.json
specs/fixtures/calendar-prd/replay/ac/calendar-ac02-personal-detail-projection.fixture.json
specs/fixtures/calendar-prd/replay/ac/calendar-ac03-action-card-workflow-handoff.fixture.json
specs/fixtures/calendar-prd/replay/ac/calendar-ac04-legal-hold-preservation.fixture.json
specs/fixtures/calendar-prd/replay/ac/calendar-ac05-jurisdiction-retention-ux.fixture.json
specs/fixtures/calendar-prd/replay/asyncapi/calendar-asyncapi-v1-replay.fixture.json
specs/fixtures/calendar-prd/replay/authority/calendar-inventory-provenance-rejection.fixture.json
specs/fixtures/calendar-prd/replay/authority/calendar-prd-authority-source-lock.fixture.json
specs/fixtures/calendar-prd/replay/boundary/calendar-personal-work-pillar-boundary.fixture.json
specs/fixtures/calendar-prd/replay/contracts/calendar-produced-contracts.fixture.json
specs/fixtures/calendar-prd/replay/openapi/calendar-openapi-v1-replay.fixture.json
specs/fixtures/calendar-prd/replay/policy/calendar-build-parentage.fixture.json
specs/fixtures/calendar-prd/replay/proto/calendar-proto-v1-replay.fixture.json
specs/fixtures/calendar-prd/replay/ux/calendar-browser-accessibility-evidence.fixture.json
```

Disposition unchanged from G030-F: local Python replay consumer exists; not Buck2 / protected-CI wired. Owner lane = calendar product/replay migration. Not GRAPH_WIRED under build-graph criterion; not deletable.

### CRATEADR owner-batch — 3

```text
specs/fixtures/crate-adr-design-doc-coverage/tc-CRATEADR-002A-good-governance-check-gates-owner-batch.json
specs/fixtures/crate-adr-design-doc-coverage/tc-CRATEADR-002B-good-ci-control-plane-owner-batch.json
specs/fixtures/crate-adr-design-doc-coverage/tc-CRATEADR-002D-good-billing-metering-reorg-owner-batch.json
```

Disposition unchanged: ADR-retained authority; measured machine consumer unresolved. Owner lane = crate-ADR coverage. Not deletable by G030.

### DR RTO/RPO matrix — 1

```text
specs/fixtures/dr-rto-rpo-matrix/dr-001-dashboard-manifest.fixture.json
```

Disposition unchanged: transitional Python bridge only; native Rust/Buck2 successor required before retirement. Owner lane = DR/compliance. Not deletable by G030.

## Closed residual set after T+U

### Non-fixture (13) — from G030-T

1. `registry/accounts/schema.json`
2. `registry/accounts/README.md`
3. `registry/vcs/concurrent-safe-paths.yaml`
4. `registry/vcs/event-router.yaml`
5. `registry/vcs/webhook-delivery-log.json`
6. `registry/vcs/README.md`
7. `registry/foundation-bypasses/README.md`
8. `registry/capabilities/foundry-supervisor.toml`
9. `registry/release/supply-chain/README.md`
10. `registry/merge-queue-tick-log.json`
11. `registry/claim-matrix/ops-portal.json`
12. `specs/policy/cedar-scope-schema.md`
13. `specs/products/RETIREMENT.md`

### Fixture (19) — this note

All calendar / CRATEADR / DR paths listed above.

## Owner/blocker classes (no G030 execution)

| Class | Count | Required owner action |
|---|---:|---|
| Calendar local-replay not Buck-wired | 15 | migrate checker to Buck2/protected CI or accept permanent local-only with explicit declass authority |
| CRATEADR authority-retained unresolved consumer | 3 | cite live consumer, wire one, migrate corpus, or reviewed declassification |
| DR transitional Python bridge | 1 | native Rust/Buck2 successor + atomic bridge retirement |
| Accounts schema/README unwired | 2 | wire schema reader or retire claim |
| VCS frozen historical companions | 4 | keep frozen under ADR-0363; no reactivate/delete without owner |
| Foundation/release README companions | 2 | keep as contract docs; extension filters intentional |
| Foundry supervisor TOML unwired | 1 | wire loader or retire claim |
| Merge-queue tick-log / claim-matrix | 2 | restore writer/checker or retire planned surface |
| Markdown policy/retirement companions | 2 | keep as prose contracts |

## Anti-vacuity

- tip membership proven by `git ls-tree` for all 19 fixture paths;
- sum 15+3+1 = 19;
- residual protected total 13+19 = 32 matches corrected partition;
- no row promoted solely because a Python checker exists;
- no deletion authorized.

## Non-actions

- No fixture edit/delete/move.
- No Buck wiring PR opened from this census lane.
- No independent APPROVE claimed.
- G028 remains local-only pending real independent APPROVE.
