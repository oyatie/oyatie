# Stabilize receipt — PR #1664 `integ/office` (SeatB absorb)

| Field | Value |
|-------|-------|
| tip | b9e5ae8dd |
| base | `origin/dev` |
| envelope | `app/office/**` |
| dirty→restack | YES — merged `#1933` (`713dc2ea1`) |
| fence | CLEAN — PR diff vs `origin/dev` is `app/office/**` only |
| Cargo.lock authored | NO (BAN) |
| cross-integ / mega-rail | NO (BAN) |
| merge | HOLD — BAN merge-red until Land-clean + green `oya-ci-required` |

## In-domain actions

1. Restack DIRTY tip onto `origin/dev` (merge-only; no path rewrite).
2. Add `app/office/OWNERS` (`axis-cloud-platform`) — closes ownership-floor gap for new `app/office/**` paths.
3. Local smoke: `buck2 build //app/office/oya-office-kernel:oya-office-kernel` → BUILD SUCCEEDED.

## Prior CI signal

All historical `oya-ci-required` runs on this PR concluded **cancelled** (supersede/queue), then aggregator RED. Not a proven in-domain compile failure.

## Elevate (out of envelope — do not mega-rail here)

| Blocker | Owner rail | Why out of fence |
|---------|------------|------------------|
| Retire `scan-root-liveness` `forward_declarations` for `app` once `app/` lands | tip-free `integ/ci` / first-lander companion | `ci/facade/scan-root-liveness/**` |
| Workspace glob cover / exclude `app/office/oya-*` (dual-home) | Cargo.lock sole-owner / tip-free specs-os | root `Cargo.toml` + lock |
| `capability-registry` `app_products.current_dirs`: `oya/office` → `app/office` | `integ/specs` | `specs/**` |
| Reachability / citation corpus re-freeze | tip-free specs/governance | hub + policy pins |
| Shrink-only delete `oya/office/**` | `integ/oya` | source drain |

## Verdict

**Stabilized for observation:** restacked + fenced + OWNERS. **Not Land-clean.** No merge.
