---
purpose: Oyatie Runbook — Laptop CAS lab (NativeLink warm-cache operator drills)
doc_status: published
---

# Laptop CAS — Buck2 cache-only opt-in (fail-closed)

## Existing surfaces (do not weaken)

| Surface | Role |
|---------|------|
| `toolchains/cache/` | Execution platform with `remote_cache_enabled` + `allow_cache_uploads` (defaults false) |
| `infra/ci/buckconfig/warm-cache-rw.buckconfig` | In-cluster writer overlay (dark while license false) |
| `infra/ci/buckconfig/warm-cache-ro.buckconfig` | In-cluster reader overlay |
| `specs/cache-warmth-policy.json` | Per-class warmth / read / write |
| `specs/cache-warm-license.json` | **Kill-switch** — `warm_reads_licensed: false` until GREEN canary |

Root `.buckconfig` must **never** select the cache platform by default (dark-by-default).

## Lab overlays

Examples (also under `~/oyatie-cas/buckconfig/`):

- `docs/runbooks/assets/laptop-cas/buckconfig/warm-cache-lab-rw.buckconfig`
- `docs/runbooks/assets/laptop-cas/buckconfig/warm-cache-lab-ro.buckconfig`

Point `[buck2_re_client]` at:

- Local: `grpcs://127.0.0.1:50051` (writer) / `:50052` (reader)
- After tunnel: `grpcs://cas-writer.lab.oyatie.dev` / `cas-reader.lab.oyatie.dev`

Materialize a mode-0600 `.buckconfig.local` from
`~/oyatie-cas/tls/client-{writer,reader}.{crt,key}` and `ca.crt`. These lab overlays are
hand-managed preflight and do not pass through the fleet endpoint profile or its `grpc://`
materialization grammar. They mirror production identity hygiene only; identity never belongs in a
committed overlay.

## Opt-in classes (policy already lists; license gates reads)

After go-gate only: `presubmit-trusted-*`, `gate-fleet-shared-graph`, `dev-agentic-iteration`, `postmerge-dev-trunk`.  
`integrity-canary` and `untrusted-author-presubmit` stay cold.

## Phase A binding

- Document + lab opt-in: **this runbook**
- Fleet `cache_read` / `warm_reads_licensed` flip: **NOT in Phase A** — wait for canary GREEN + reviewed license edit on the owning tip
