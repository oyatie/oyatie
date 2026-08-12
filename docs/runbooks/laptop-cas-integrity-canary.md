# Laptop CAS — integrity-canary scaffold

Trust invariant (`specs/cache-warmth-policy.json`): warm IFF class is warm-eligible **and** the latest cold integrity-canary is GREEN.  
**Phase A:** scaffold against the laptop endpoint only. **Do not** enable fleet `cache_read` / flip `warm_reads_licensed`.

## Local probe (runs today)

```bash
~/oyatie-cas/canary/probe-lab-endpoint.sh
# Expected while lab is up: LAB_ENDPOINT_REACHABLE_COLD — not GREEN for fleet license
```

This is connectivity + TLS material presence — **not** digest byte-equality.

## Full canary (existing + lab target)

In-repo executor: `.github/workflows/cache-integrity-canary.yml` + schedule wrapper.  
Policy pin: `ci/facade/build-cache-policy/src/canary-policy.json`.

Lab adaptation (example YAML committed as asset, **not** under `.github/` yet — avoids thrashing `#1646` / `integ/ci` process_meta):

`docs/runbooks/assets/laptop-cas/canary/cache-integrity-canary-lab.example.yml`

When absorbed on `integ/ci`:

1. `workflow_dispatch`-only (or separate check name outside `oya-ci-required`)
2. Reader probe against `cas-reader.lab.oyatie.dev` with lab/reader mTLS secrets
3. Cold build still **no** overlay / no `actions/cache` of buck-out
4. Verdict GREEN is the only license to edit `specs/cache-warm-license.json`

## Explicit non-goals (Phase A)

- Flip `warm_reads_licensed` to true
- Enable warm overlays for `presubmit-trusted-*` / `gate-fleet-shared-graph` in merge CI
- Make merge admission depend on laptop uptime
