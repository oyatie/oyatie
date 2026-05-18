# Runbook — Canvas perf regression (Sev-3)

## Trigger

`tests/canvas-1000-node.bench.ts` p99 frame time > 16.67ms in CI OR production p99 frame time alert fires.

## Immediate actions

1. Check git blame for recent canvas-adapter changes.
2. Run bench locally + profile with Chrome DevTools Performance tab.
3. Verify viewport virtualization still enabled.
4. Verify LOD rendering tiers correct.

## Common causes

| Cause | Fix |
|---|---|
| Removed `onlyRenderVisibleElements` | re-enable |
| Heavy CSS filter on node body | remove filter; use static color |
| Per-node React/Svelte reactive observer cascade | memo + signal-level reactivity |
| LOD tier not switching at expected zoom | check zoom → tier function |

## Cross-references

- ADR-0204 — perf bar.
- IP-024 — perf bench.
