---
doc_class: DeprecationNotice
template_id: TPL-DEPRECATION-NOTICE
microservice: drive
deprecated_artifact: oya-connect-drive-* crate family
status: Deprecated
deprecation_date: 2026-05-17
removal_target: advisory — HG-DRIVE accepts at p99 SLOs sustained 30d
related_adrs: [ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-DRIVE-0001, ADR-DRIVE-0002, ADR-DRIVE-0003, ADR-DRIVE-0004, ADR-DRIVE-0005, ADR-DRIVE-0006]
related_specs: [/specs/microservices/drive.json, /specs/microservices/workspace/drive.json]
owner_team: axis-drive
date: 2026-05-17
doc_status: published
---

# Deprecation Notice: `oya-connect-drive-*` crate family

> Formal deprecation notice in the format prescribed by the agent-skills `deprecation-and-migration` skill SKILL.md §"Step 2: Announce and Document".

Per `feedback_no_silent_regression.md`: this notice is the loud, CI-detectable, time-boxed public surface for the deprecation of the legacy `oya-connect-drive-*` crate family.

## Status

**Deprecated as of 2026-05-17.**

## Replacement

`oya-drive-*` crate family under `microservices/drive/src/crates/` per ADR-0131. See **`microservices/drive/migration-from-connect.md`** for the full import-path map, Hyrum's-Law-bound surface callouts, configuration delta table, runbook continuity table, and step-by-step migration guide.

## Removal date

**Advisory — no hard deadline.** Concrete removal target is HG-DRIVE accepts at p99 SLOs sustained 30d (per ADR-0135 retirement trigger). Following the 5-month Strangler window in ADR-0134 (Phase 2 adapter soak + Phase 3 canary), the indicative advisory removal date is **2026-11-17**, gated on the SLO trigger.

## Reason

The legacy `oya-connect-drive-*` family was authored before the following ADRs crystallised; each ADR makes the legacy shape non-conforming:

1. **ADR-0132 — no-suite forward-policy.** `connect-*` encodes bundle membership at the architecture layer; bundle membership is a brand-layer concept and must not appear in crate names.
2. **ADR-0130 — agentic SLO-gated promotion.** Drive needs independent SLO targets per surface (file-list latency, upload throughput, download first-byte, search, sync delta, share-link mint, preview render, DLP correctness, WORM correctness, virus-scan correctness); a `connect-*` umbrella SLO cannot serve them.
3. **ADR-0131 — per-µservice flat layout.** Drive's IaC, runbooks, threat-model, DPIA, compliance, capacity-model, cost-budget all need to live under one folder.
4. **ADR-0133 — 11-pack-overlay program.** pack-kr / pack-eu / pack-us / pack-us-healthcare / pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa each live at per-µservice overlay granularity.
5. **ADR-DRIVE-0001 → ADR-DRIVE-0006** — drive-specific decisions (object-storage backend, CDC algorithm, share-link model, encryption-at-rest + E2E, preview sandbox, WORM policy) need to live at per-µservice ADR granularity, not at the Connect / Workspace suite level.

## Migration Guide pointer

→ **`microservices/drive/migration-from-connect.md`**

Includes: 1:1 import-path map; net-new-boundary features (Garage / SeaweedFS adapters, FastCDC delta-sync, libsodium E2E, WORM tier, gVisor preview sandbox, integrated DLP/virus scan); concrete `use` and `Cargo.toml` rewrites; configuration delta table; dual-context isolation invariant preservation; Hyrum's-Law surface callouts (chunk-boundary stability, share-link blob format, share-link TTL boundary, sync conflict tie-break, virus-scan timing, retention horizon, object-store key shape); runbook continuity table; 5-step migration recipe; 6-phase Strangler timeline; verification checklist.

## Affected packages enumerated

Per `find crates -maxdepth 1 -type d -name 'oya-connect-drive-*'` (2026-05-17 workspace state):

| Currently extant in `crates/` | Mapped replacement |
|---|---|
| `oya-connect-drive-domain` | split per BC → `oya-drive-{file-store,folder-hierarchy,upload,download,sync,share-link,permissions,search-index,preview,dlp-virus-scan,immutability-tier}-domain` |

Plus all planned `oya-connect-drive-{kernel,usecase,api,adapter*,rest,worker,sdk,app}-*` crates scaffolded during Phase 2 adapter authoring.

## Breaking changes flagged per `feedback_no_silent_regression`

| Change | Phase | Breaking? | Sunset notice |
|---|---|---|---|
| New `oya-drive-*` crates ship in parallel | 1 | No (additive) | — |
| New `oya-drive-file-store-adapter-garage` + `-adapter-seaweedfs` (ADR-DRIVE-0001) | 1 | No (net-new; no legacy counterpart) | — |
| New `oya-drive-sync-*` (FastCDC + LBFS delta-sync, ADR-DRIVE-0002) | 1 | No (net-new) | — |
| New `oya-drive-immutability-tier-*` (WORM, ADR-DRIVE-0006) | 1 | No (net-new) | — |
| FastCDC engine replaces fixed-size chunker | 1 | **Behaviourally divergent** for chunk-id stability per ADR-DRIVE-0002 | adapter does NOT mask divergence; documented in migration guide Hyrum #1 |
| Share-link strict-TTL (no 1s clock-skew tolerance) | 1 | **Behaviourally divergent** | documented Hyrum #3 |
| Share-link signing blob → Ed25519 over canonicalised JSON | 1 | **Format-divergent** | invisible at GET-URL level; documented Hyrum #2 |
| Sync conflict deterministic tie-break | 1 | **Behaviourally divergent** | documented Hyrum #4 |
| Virus-scan at upload time (not first download) | 1 | **Behaviourally divergent** | documented Hyrum #5 |
| WORM strict refuses tenant-root purge | 1 | **Behaviourally divergent** | documented Hyrum #6 |
| `oya-connect-drive-migration-adapter` shim authored | 2 | No (preserves legacy symbol surface) | — |
| Feature-flagged canary 10→50→100% | 3 | No (additive, gated) | — |
| Zero-usage verification | 4 | No (observability only) | — |
| **`oya-connect-drive-*` crates removed from workspace** | **5** | **YES — breaking** | **6-mo advisory sunset from 2026-05-17** |
| `microservices/connect/` umbrella folder removed | 6 | No | — |

Per `feedback_no_silent_regression.md`, the Phase 5 breaking change carries:

- **This deprecation notice** (renders the change loud + immediate + CI-detectable).
- **ADR-0134** (carries the migration policy decision).
- **ADR-DRIVE-0002 / ADR-DRIVE-0003 / ADR-DRIVE-0005 / ADR-DRIVE-0006** (specifically document the behavioural strengthenings as deliberate, owner-authored design choices — NOT silent regressions).
- **Version bump.** The `Cargo.toml` of every consumer crate is bumped per semver when its legacy imports are removed (treating the `oya-connect-drive-*` re-export as the public contract).
- **Sunset schedule.** 6-month advisory window from this notice; concrete date 2026-11-17 contingent on the HG-DRIVE SLO trigger.
- **Owning-axis migration ChangeSets.** axis-drive ships migration ChangeSets for every known internal consumer per the Churn Rule before Phase 5.

## Verification (per skill SKILL.md §"Verification")

- [ ] Replacement is production-proven and covers all critical use cases — HG-DRIVE gate at p99 SLO sustained 30d.
- [ ] Migration guide exists with concrete steps and examples — `migration-from-connect.md`.
- [ ] All active consumers have been migrated — verified by Phase 4 commands (see ADR-0134 §Phase 4).
- [ ] Old code, tests, documentation, configuration removed — Phase 5 commands.
- [ ] No references to the deprecated system remain — `rg "oya_connect_drive" --type rust` produces zero hits outside historical surfaces.
- [ ] Deprecation notices removed — this notice deletes itself in Phase 5.

## References

- ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0134.
- ADR-DRIVE-0001 (object-storage substrate selection).
- ADR-DRIVE-0002 (CDC + delta-sync).
- ADR-DRIVE-0003 (share-link security model).
- ADR-DRIVE-0004 (encryption-at-rest + E2E).
- ADR-DRIVE-0005 (preview pipeline sandboxing).
- ADR-DRIVE-0006 (immutability + WORM policy).
- `microservices/drive/migration-from-connect.md` — full migration guide.
- `microservices/drive/PRD.md` — target-state product definition.
- `microservices/drive/runbooks/*.md` — 7 runbooks.
- `feedback_no_silent_regression.md`.
- agent-skills deprecation-and-migration SKILL.md.
- agent-skills documentation-and-adrs SKILL.md.
- AWS S3 SigV4 + Object Lock spec; RFC 4918 (WebDAV); tus.io 1.0; RFC 9106 (Argon2); RFC 9110 (HTTP).
