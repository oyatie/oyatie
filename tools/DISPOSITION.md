# tools/ disposition (north-star)

**Not merge authority.** SSOT machine form: `evidence/reorg/rr-tools-disposition-20260806.json`.

## First principles

- `tools/` is a **debt zone** (NORTH-STAR-SHAPE): no new product surface.
- Reorg here is often **rewrite** (process → `.grok/`) or **refactor** (gates → `ci/facade`), **not** a path bijection move-plan.
- **Delete** only with consumer-absence proof (membership, registry, hooks, ADRs).

## Classes in use

| Class | Meaning for tools |
|-------|-------------------|
| keep | Still load-bearing; leave until epic complete |
| refactor | Rehome/structure under platform path without behavior rewrite |
| rewrite | Replace implementation / host (e.g. process kit) |
| delete | Remove after consumers gone |
| blocked | Consumers prevent execute |

## Do not

- Mass-delete governance CLIs while gate_registration binds them
- Vendor OpenSK source in this tree without ADR-0508 follow-up IP
- Add new `tools/oya-*` apps without disposition row

See evidence JSON for per-path class + blockers + follow-on slices.
