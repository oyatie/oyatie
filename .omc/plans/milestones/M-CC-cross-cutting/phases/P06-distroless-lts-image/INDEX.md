---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M-CC-P06
title: Distroless + Image-Discipline + Dependency-Seam/LTS Phaseout
status: stub
purpose: Every production binary ships in distroless + smallest-form; direct deps are LTS/ADR-governed; release-critical runtime deps are seam-contained, ledgered, and trigger-phaseout governed.
---

# M-CC-P06 — Distroless + Dependency-Seam/LTS + Image Discipline

## Purpose
Per MASTERPLAN §6/§8 and `.omc/specs/masterplan.json` dependency-seam invariant. Verified LTS roster lands at [`../../../../specs/lts-versions-verified-2026-05-12.md`](../../../../specs/lts-versions-verified-2026-05-12.md) (pending agent). Round-5 dep-seam findings from `../../../../ralplan-dep-seam-phaseout-round-5.md` are folded here: ship release-critical products on Hyper/Tokio first, but force wrapper/newtype seams, tech-debt ledger ownership, trigger-based phaseout, replacement parity, and CI enforcement before debt deepens.

## Acceptance
- `oya-foundry-fitness-image-discipline` lane CI-blocks: non-distroless base, shells/package-managers in production image, image size > budget (per binary).
- `oya-foundry-fitness-lts-dependency` lane CI-blocks: any direct dep that drifts from current LTS without ADR-tracked exception.
- `oya-check-dependency-seam-discipline` composite lane enforces layer metadata, seam import boundaries, tech-debt ledger coverage/freshness, vendor residue, CVE watch, review contract, and monotonic status transitions.
- `.omc/registries/tech-debt-ledger.json` exists with 11 seed deps, top-level `default_evaluator_policies`, trigger DSL, DRI handles, replacement targets, CVE acceleration, and ADR citations.
- ADR-0091..ADR-0094 are authored/indexed at the states required by round 5; ADR-0093 becomes Accepted only with CODEOWNERS + same-PR guard in Step 6.
- Image size budget table published at `docs/standards/image-size-budgets.md`.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Distroless base + image-discipline lane | stub | [`IP-001-distroless-image-lane.md`](IP-001-distroless-image-lane.md) |
| IP-002 | Dependency-seam discipline + tech-debt ledger + LTS roster | expanded from round-5 findings | [`IP-002-lts-dependency-lane.md`](IP-002-lts-dependency-lane.md) |
| IP-003 | Static / musl-linked binary build pipeline | stub | [`IP-003-static-musl-build.md`](IP-003-static-musl-build.md) |

## Estimated parallelism
5 agents: distroless/image lane, LTS roster, dependency-seam composite lane, trigger-DSL/ledger, and static-musl pipeline. Step 0→1 in IP-002 remains sequential; post-Step-1 work fans out.

## Symbols-touched
`crates/oya-foundry-fitness-{image-discipline,lts-dependency}-kernel`, `crates/oya-check-dependency-seam-discipline`, `crates/oya-foundry-trigger-dsl-{kernel,runtime}`, `.omc/registries/tech-debt-ledger.json`, `Dockerfile.distroless`, `docs/standards/image-size-budgets.md`.

## Agent-handoff
```
icm store -t context-oyatie -c "M-CC-P06 complete: distroless + image-discipline + dependency-seam/LTS lanes green; tech-debt ledger rows=11; trigger DSL split; replacement parity + static/musl pipeline live" -i critical -k "M-CC,P06,distroless,lts,dependency-seam,image,complete"
```
