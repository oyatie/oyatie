---
purpose: Oyatie Runbook — Laptop CAS lab (NativeLink warm-cache operator drills)
doc_status: published
---

# Laptop CAS — dual-arch digests

CAS does **not** need two NativeLink processes. Digests are content-addressed **per platform**:

| Writer | Digests written |
|--------|-----------------|
| GHA `ubuntu-latest` (amd64) | linux/amd64 action keys |
| GHA soft `ubuntu-24.04-arm` / local native Buck2 | linux/arm64 (and darwin/arm64 if used) |
| Optional Colima/Docker `linux/amd64` on Mac | same amd64 keys as GHA (**slow**; debug only) |

**One NativeLink + one store** on this Mac serves all of the above. Emulation is only for *running amd64 Buck2 on this Mac*, not for the cache server. NativeLink runs as an arm64 (or multi-arch) container — **server ISA ≠ blob ISA**.

## Populate strategy

1. **Primary amd64 warmth:** GHA `ubuntu-latest` with `cache_write=true` after go-gate.
2. **Primary arm64 warmth:** local native Buck2 on this Mac (+ soft arm GHA if desired).
3. **Optional amd64-on-Mac:** Colima VM `x86_64` / `docker buildx --platform linux/amd64` — document as slow path; **not** required for merge CI warmth.
4. Darwin/arm64 is a third namespace if macOS Buck2 is used — fine in the same CAS; do not expect linux/amd64 jobs to hit those keys.

## Go-gate

Fleet warm reads stay fail-closed (`specs/cache-warm-license.json` → `warm_reads_licensed: false`) (the integrity-canary trust chain is retired by ADR-0716; warm reads stay unlicensed until a successor CAS trust anchor is stood up).
