## Summary

Closes the **inventory** slice of bead **oyatie-oso.20** / card **R-DUAL-0615**.

Re-queried **ADR-0615** (amends ADR-0562) on current `origin/dev` and inventoried the five dual-home / mis-absorbed trees that exist on trunk:

| Path | Crates | Registry absorb | Live home | Disposition |
|------|-------:|-----------------|-----------|-------------|
| `oya/drive` | 0 | storage | `storage/facade/drive` | **park** (keep facade; future app product) |
| `oya/recordings` | 0 | storage | `storage/facade/recordings` | **park** |
| `oya/emergency` | 0 | comms (wrong) | — | **park** → future **move** `app/healthcare` |
| `oya/imaging` | 0 | storage (wrong) | — | **park** → future **move** `app/healthcare` |
| `oya/diagnostics` | 0 | observability (name collision) | — | **park** → future **move** `app/health-diagnostics` |

### Authority (ADR re-query)

- **Q2** emergency (ED clinical) → `app/healthcare` (not comms)
- **Q3** drive/recordings domain crates **confirmed** at `storage/facade/*`; future consumer product → `app/`
- **Q4** imaging (PACS) → `app/healthcare`
- **Q8** diagnostics (clinical lab) → `app/health-diagnostics` (NOT healthcare)
- Structural absorbs→app_products execute only in Batch-5 move-plans (`pending_relocations`); this PR does **not** open those plans

### This PR

1. Evidence JSON: `evidence/reorg/r-dual-0615-oya-disposition-inventory-20260805.json`
2. Short markdown: `evidence/reorg/r-dual-0615-oya-disposition-inventory-20260805.md`

### Explicit non-goals

- **No path moves** (no multi-cap mega-move)
- **No deletes** — trees are non-empty scaffolds with live cedar-parity / product-protocol / tier-classification consumers
- **No new `*-move-plan.json`** (ADR-0614; inventory-only concern)
- No CAS / RE / runners / k8s port coupling
- Follow-on move cards listed in evidence `residuals_for_followon_beads`

## Test plan

- [ ] Evidence JSON parses; counts match tree (`find oya/{drive,recordings,emergency,imaging,diagnostics} -name Cargo.toml` → 0)
- [ ] ADR-0615 Q2/Q3/Q4/Q8 rulings match disposition rows
- [ ] `git diff --stat` is evidence-only (no `oya/*` path moves)
- [ ] Cloud CI `oya-ci-required` green (docs/evidence)

Bead: oyatie-oso.20
