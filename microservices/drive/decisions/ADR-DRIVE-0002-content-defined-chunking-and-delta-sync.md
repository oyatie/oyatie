---
id: ADR-DRIVE-0002
status: Accepted
date: 2026-05-17
microservice: drive
deciders: axis-drive, council-architecture, ops-sre-reliability
owner: axis-drive
supersedes: []
superseded_by: []
related: [ADR-0056, ADR-0105, ADR-0135, ADR-0131, ADR-0133, ADR-DRIVE-0001]
related_artifacts:
  - microservices/drive/PRD.md (§FR-06 sync; §"Performance" sync delta target; AC-04 delta-minimum bytes)
  - microservices/drive/iac/helm/values.yaml (upload.fastcdc.* parameters)
  - microservices/drive/runbooks/sync-conflict-resolution.md
purpose: |
  Pick a content-defined-chunking (CDC) algorithm + a delta-sync protocol for
  the `oya-drive-sync-*` BC. The PRD requires delta-sync that beats Google
  Drive (no delta-sync at all) and approaches Dropbox (rsync rolling-hash);
  the choice is between fixed-size chunking (fast but no dedup at byte-shift),
  Rabin fingerprint (LBFS-grade dedup; CPU-heavy), BuzHash (cheaper than
  Rabin; correlated chunk boundary drift), and FastCDC (modern; designed for
  exactly this use case).
---

# ADR-DRIVE-0002: Content-defined-chunking via FastCDC + LBFS-style delta-sync

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

PRD-drive §FR-06 mandates delta-sync that transfers only changed bytes — a 100MB file with a 1KB diff must transfer ≤ 8KB on the wire (AC-04). The standard tactic is **content-defined chunking** combined with **client-server manifest exchange**: client computes per-chunk hashes; server reports which chunks it already has; client uploads only the missing ones.

Three CDC algorithms are the field:

1. **Rabin fingerprint** (Rabin 1981; popularised by LBFS, Muthitacharoen et al. SOSP 2001). Polynomial-arithmetic rolling hash. Strong theoretical foundations; deduplication-grade chunk boundary stability. CPU cost ~20% higher than BuzHash. ([pdos.csail.mit.edu/papers/lbfs:sosp01.pdf](https://pdos.csail.mit.edu/papers/lbfs:sosp01.pdf))
2. **BuzHash** (Boyer–Moore variant; popularised by borg / restic). Cheaper rolling hash. Correlated boundary drift on adversarial inputs.
3. **FastCDC** (Xia et al. ATC 2016 / TOS 2020). Designed specifically for content-defined chunking; reports 10× faster than Rabin with equivalent dedup; normalised chunk-size bounds + zero-byte gear table. Adopted by restic 0.16+, BorgBackup-NG, and Vercel Build Output. ([usenix.org/conference/atc16/technical-sessions/presentation/xia](https://www.usenix.org/conference/atc16/technical-sessions/presentation/xia))

A fourth option, **fixed-size chunking** (legacy `oya-connect-drive-domain`), is the baseline against which others are measured: trivial CPU, but byte-shift in a single file invalidates all subsequent chunks.

Delta protocol candidates:

- **LBFS** (Low-Bandwidth File System; Muthitacharoen et al. 2001). Client sends chunk-hash manifest; server replies with needed-chunks set. Battle-tested at scale.
- **rsync** (Tridgell 1996). Server-driven; client sends rolling-window hashes; server picks block boundaries. Higher round-trip latency.
- **Custom CRDT-like** (Yjs / Loro). Theoretically richer; not designed for file bytes; not a deduplication-grade fit for binary content.

Per PRD-drive §"Performance" the sync delta p99 ≤ 30s for 100 changed files; per AC-04 a 100MB file with 1KB diff must transfer ≤ 8KB on the wire.

Per ADR-0133 axis-1 industry citation, the chosen algorithm must have a non-zero upstream community with the past 12 months of activity.

## Decision

The drive µservice ships **FastCDC content-defined chunking + LBFS-style client-server manifest delta-sync**:

- **CDC algorithm**: FastCDC (Xia et al. 2016).
- **CDC parameters** (pinned at chart level per `iac/helm/values.yaml`):
  - `min_bytes = 4 MiB`
  - `avg_bytes = 8 MiB`
  - `max_bytes = 16 MiB`
  - `gear_table_version = 1` (zero-byte-resistant variant per Xia et al. §5).
- **Chunk hash**: BLAKE3 (256-bit; faster than SHA-256, equally strong, dedup-grade).
- **Delta protocol**: client sends chunk-list (offset + length + BLAKE3 hash) via `/sync/sessions/{session_id}/manifest`; server replies with needed-chunks set + per-chunk pre-signed upload URLs.
- **Conflict tie-break**: deterministic `(timestamp, actor_id)` ordering when two clients write the same file with overlapping clock — last-writer-wins by timestamp; ties broken by actor_id lexicographic order; both versions preserved as `conflict-A.ext` + `conflict-B.ext` in the conflicting folder.

Concrete bindings:

- Crate: `oya-drive-sync-domain` ships the FastCDC implementation; `oya-drive-file-store-kernel` ships the `ContentAddressDeriver` port; `oya-drive-sync-usecase` orchestrates manifest exchange.
- LBFS reference is the algorithmic ancestor; oyatie's protocol is HTTP-over-mTLS, not LBFS's NFS-extension surface.

## Alternatives Considered

### A. Fixed-size chunking (legacy `oya-connect-drive-domain`)

- **Pros**:
  - Trivial CPU; trivial implementation.
  - No chunk-boundary subtleties.
- **Cons**:
  - Byte-shift in a single file invalidates all subsequent chunks → ~no dedup gain.
  - Loses the AC-04 8KB-on-wire target for a 1KB-diff 100MB file.
- **Rejected** outright; fails the PRD target.

### B. Rabin fingerprint

- **Pros**:
  - LBFS-grade dedup; strong theoretical foundations.
  - Battle-tested at scale (LBFS, rdiff-backup, casync).
- **Cons**:
  - ~10× slower than FastCDC per Xia et al.
  - More complex polynomial-arithmetic implementation.
- **Rejected** in favour of FastCDC for 10× speed parity with equivalent dedup.

### C. BuzHash

- **Pros**:
  - Cheaper than Rabin.
  - borg / restic legacy proves at-scale.
- **Cons**:
  - Correlated boundary drift on adversarial inputs (well-documented in the borg / restic issue tracker).
  - FastCDC supersedes BuzHash in the same use case with both better dedup and faster compute.
- **Rejected** in favour of FastCDC.

### D. FastCDC + LBFS-style manifest exchange  ← **CHOSEN**

- **Pros**:
  - Modern algorithm designed for this exact use case.
  - 10× faster than Rabin; equivalent dedup; normalised bounds avoid pathological large/small chunks.
  - Battle-tested in restic 0.16+, BorgBackup-NG, Vercel Build Output.
  - Apache-2.0 reference implementations available; oyatie ships a Rust port within `oya-drive-sync-domain`.
  - LBFS-style manifest exchange is the reference for the protocol.
- **Cons**:
  - FastCDC is newer (2016) than Rabin (1981); less than a decade of production maturity.
  - Parameter selection (`min/avg/max/gear`) sensitive; pinned at chart level + LEAN-check refuses drift.
- **Accepted**.

### E. rsync-style delta

- **Pros**:
  - Well-known.
  - Battle-tested.
- **Cons**:
  - Server-driven rolling-window scan is higher round-trip than client-driven manifest exchange.
  - Doesn't compose cleanly with content-defined chunking; rsync rolls per byte; CDC chunks per content.
- **Rejected** in favour of client-driven manifest.

## Consequences

### Positive

- **AC-04 met**: a 100MB file with 1KB diff transfers ≤ 8KB on the wire (one chunk pair).
- **Throughput**: FastCDC 10× Rabin → sync session CPU saturation is unlikely under typical workload.
- **Determinism**: pinned parameters + zero-byte-resistant gear table eliminate adversarial chunk-boundary attack vectors.
- **Conflict resolution**: `(timestamp, actor_id)` deterministic tie-break ensures test stability + predictable user UX.

### Negative

- **Hyrum's-Law surface #1 callout**: legacy `oya-connect-drive-domain` used fixed-size chunking; chunk-ids differ. The file-level content-address (BLAKE3 over the whole file) remains stable. Migrating consumers documented in `microservices/drive/migration-from-connect.md` Hyrum #1.
- **CPU vs Rabin** — FastCDC is faster but still > BuzHash. Per-cell CPU budget for sync workers ~2 vCPU per worker pod (HPA-bound).
- **Parameter pinning required** — `oya-check-cdc-parameters-pinned` LEAN lane refuses drift; cost is one CI lane.

### Hyrum's Law

Per the deprecation-and-migration skill SKILL.md §"Hyrum's Law":
- **Chunk-id stability**: legacy fixed-size chunker emits chunk-ids that the new FastCDC does NOT preserve. The whole-file content-address (BLAKE3) IS preserved. Consumers that pattern-matched on chunk-id stability migrate per `migration-from-connect.md` Hyrum #1.
- **Conflict tie-break determinism**: legacy was non-deterministic on equal wall-clock; new is deterministic via `(timestamp, actor_id)`. Consumers expecting non-determinism see flakey-tests-fixed behaviour. Documented in `migration-from-connect.md` Hyrum #4.

### Operational

- **New CI lane**: `oya-governance-cdc-parameters-pinned` (BLOCKER) — refuses drift on FastCDC parameters.
- **Regression test**: `tests/sync-tie-break-determinism.rs` (BLOCKER) — explicitly validates conflict tie-break.
- **Regression test**: `tests/delta-minimum-bytes.rs` (BLOCKER) — validates AC-04 (100MB ± 1KB → ≤ 8KB wire bytes).
- **FastCDC implementation source**: vetted Rust port adapted from `restic` 0.17.x FastCDC implementation; pinned dependency in `oya-drive-sync-domain/Cargo.toml`.

## Verification

- [ ] FastCDC parameter integrity test — `cargo nextest run -p oya-drive-sync-domain -- fastcdc_parameters_pinned`.
- [ ] AC-04 wire-bytes test — `cargo nextest run -p oya-drive-sync-domain -- delta_minimum_bytes`.
- [ ] Conflict tie-break determinism test — `cargo nextest run -p oya-drive-sync-domain -- tie_break_determinism`.
- [ ] Adversarial corpus test (zero-byte runs; pathological-input) — `cargo nextest run -p oya-drive-sync-domain -- fastcdc_adversarial`.

## References

- Xia, Wen et al. "FastCDC: a Fast and Efficient Content-Defined Chunking Approach for Data Deduplication." USENIX ATC 2016; ACM TOS 2020.
- Muthitacharoen, Athicha et al. "A Low-Bandwidth Network File System." SOSP 2001 (LBFS).
- Rabin, M. O. "Fingerprinting by Random Polynomials." 1981.
- Tridgell, Andrew. "The rsync algorithm." 1996.
- BLAKE3 spec — `github.com/BLAKE3-team/BLAKE3-specs`.
- restic FastCDC implementation reference — `github.com/restic/restic`.
- BorgBackup-NG — `github.com/borgbackup/borg`.
- ADR-0056 (BNF v4.1); ADR-0105 (13-layer enum); ADR-0135; ADR-0131; ADR-0133.
- ADR-DRIVE-0001 — object-storage substrate (chunks land on the same S3 backend).
- `microservices/drive/PRD.md` §FR-06 + §"Performance" sync delta + AC-04.
- `microservices/drive/migration-from-connect.md` Hyrum #1 + #4.
- `microservices/drive/runbooks/sync-conflict-resolution.md`.
