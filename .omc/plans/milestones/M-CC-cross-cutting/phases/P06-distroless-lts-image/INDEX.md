---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M-CC-P06
title: Distroless + Image-Discipline + LTS-Dependency
status: stub
purpose: Every production binary ships in distroless + smallest-form; every direct dep tracks current LTS; CI enforces both.
---

# M-CC-P06 — Distroless + LTS + Image Discipline

## Purpose
Per MASTERPLAN §2 Directives 5 and 8. Verified LTS roster lands at [`../../../../specs/lts-versions-verified-2026-05-12.md`](../../../../specs/lts-versions-verified-2026-05-12.md) (pending agent).

## Acceptance
- `oya-foundry-fitness-image-discipline` lane CI-blocks: non-distroless base, shells/package-managers in production image, image size > budget (per binary).
- `oya-foundry-fitness-lts-dependency` lane CI-blocks: any direct dep that drifts from current LTS without ADR-tracked exception.
- Image size budget table published at `docs/standards/image-size-budgets.md`.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Distroless base + image-discipline lane | stub | [`IP-001-distroless-image-lane.md`](IP-001-distroless-image-lane.md) |
| IP-002 | LTS-dependency lane + LTS roster doc | stub | [`IP-002-lts-dependency-lane.md`](IP-002-lts-dependency-lane.md) |
| IP-003 | Static / musl-linked binary build pipeline | stub | [`IP-003-static-musl-build.md`](IP-003-static-musl-build.md) |

## Estimated parallelism
3 agents.

## Symbols-touched
`crates/oya-foundry-fitness-{image-discipline,lts-dependency}-kernel`, `Dockerfile.distroless`, `docs/standards/image-size-budgets.md`.

## Agent-handoff
```
icm store -t context-oyatie -c "M-CC-P06 complete: distroless + image-discipline + LTS-dependency lanes green; static/musl build pipeline live" -i critical -k "M-CC,P06,distroless,lts,image,complete"
```
