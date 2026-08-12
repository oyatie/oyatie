---
purpose: Oyatie Runbook — Laptop CAS lab (NativeLink warm-cache operator drills)
doc_status: published
---

# Laptop CAS lab (NativeLink cache-only)

**Runtime home:** `~/oyatie-cas/` on the founder macOS arm64 always-on host.  
**Authority:** ADR-0560 / ADR-0556 / `specs/cache-warmth-policy.json` / `specs/cache-warm-license.json`.  
**Fleet rule:** `warm_reads_licensed` stays **false** until integrity-canary is GREEN against this endpoint. Do **not** flip fleet `cache_read`.

## What runs

| Piece | Detail |
|-------|--------|
| NativeLink v1.6.2 | Cache-only (CAS+AC). No scheduler/workers. |
| Store | Filesystem first (`~/oyatie-cas/data/**` + Docker volume `cas-fast-cache`) |
| Optional | MinIO profile (`docker compose --profile minio`) — not wired into `cas.json` yet |
| Restart | `unless-stopped` |
| Auth | **mTLS** on writer `:50051` and reader `:50052`. **BAN** anonymous gRPC. |

In-repo examples (no secrets): `docs/runbooks/assets/laptop-cas/`.

## Start

```bash
cd ~/oyatie-cas
./scripts/generate-lab-mtls.sh   # once; keys never committed
./start.sh
./canary/probe-lab-endpoint.sh  # expects LAB_ENDPOINT_REACHABLE_COLD
```

Ops health (localhost only): `http://127.0.0.1:50061/status`.

## Dual-arch

One NativeLink + one store serves **linux/amd64** (GHA writers), **linux/arm64** (local/soft GHA), and optional **darwin/arm64**. See `laptop-cas-dual-arch-digests.md`.

## Related runbooks

- Tunnel + Access: `laptop-cas-cloudflare-tunnel.md`
- Buck2 opt-in (fail-closed): `laptop-cas-buck2-cache-only.md`
- Canary scaffold: `laptop-cas-integrity-canary.md`

## Sibling track

ARC `oya-arm64` retirement is **not** this runbook — Phase B of the laptop-CAS plan owns teardown. Do not delete ARC from this lab bring-up.
