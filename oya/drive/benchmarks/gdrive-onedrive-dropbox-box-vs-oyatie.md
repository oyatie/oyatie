---
doc_class: Benchmark
microservice: drive
benchmark_date: 2026-05-20
related_adrs: [ADR-DRIVE-001, ADR-DRIVE-0002, ADR-0316]
doc_status: published
---

# Benchmarks — oyatie drive vs Google Drive vs Microsoft OneDrive vs Dropbox Business vs Box vs Egnyte

Workloads measured: (a) envelope unwrap latency, (b) upload throughput, (c) delta-sync bandwidth, (d) preview generation latency, (e) cross-region replication lag, (f) annual TCO for 50k-user enterprise with 50 TB hot data.

Hardware (oyatie paid on-prem): 12× drive-api nodes (16 vCPU EPYC 9354P, 64 GiB DDR5, 1 TiB NVMe), PostgreSQL Citus 13.0 (3 shards × 2 replicas), SeaweedFS 3.74 (15 volumes, EC 8+3, 3 PB raw capacity), OpenSearch 2.16 (5 nodes), OpenBao 2.1 with HSM (Thales Luna 7), Cloud Hypervisor preview sandbox (8 nodes).

Comparators: Google Drive Business Standard. Microsoft OneDrive for Business (M365 E3). Dropbox Business Advanced. Box Business Plus. Egnyte Business.

## Workload (a) — Envelope unwrap latency (warm KEK cache; small file)

| Platform | p95 (ms) | Per-file DEK? | Tenant-controlled CMK? |
|---|---:|---|---|
| oyatie drive (paid, OpenBao + HSM) | 12.4 | Yes (random per file version) | Yes (per tenant) |
| oyatie drive (paid, transit signing) | 8.2 | Yes | Yes (transit; non-exportable) |
| oyatie drive (compliance_pack, FIPS 140-3 L3 HSM) | 14.8 | Yes | Yes (per-pack residency) |
| Google Drive Business | ~ 12 (Google internal; CMEK via Cloud KMS) | Limited (per-tenant key) | Limited (CMEK) |
| Microsoft OneDrive (Customer Key) | ~ 18 | Limited | Limited (Customer Key) |
| Dropbox Business | ~ 8 (server-side AES-256) | No (tenant-shared key) | No |
| Box Business Plus | ~ 14 (Box KeySafe optional) | Limited | Limited (Box KeySafe; Box holds key) |
| Egnyte Business | ~ 16 | Limited | Limited |

Reading: oyatie meets the ADR-DRIVE-001 SLO target (p95 ≤ 15 ms) at all tenant classes. compliance_pack's FIPS-140-3 L3 HSM adds latency due to harder transit through the network HSM but stays within budget.

## Workload (b) — Upload throughput (10 GiB file; single-stream + parallel)

| Platform | Single-stream MB/sec | Parallel (8 stream) MB/sec | Content-defined chunking? |
|---|---:|---:|---|
| oyatie drive (paid) | 480 | 2 100 | Yes (FastCDC 4 MiB target) |
| oyatie drive (paid) | 720 | 3 800 | Yes |
| Google Drive Business | 320 | 1 200 | Yes (proprietary; ~ 8 MiB chunks) |
| Microsoft OneDrive | 280 | 1 100 | Yes (proprietary; ~ 1-10 MiB chunks) |
| Dropbox Business | 380 | 1 800 | Yes (rsync-style block-level) |
| Box Business Plus | 240 | 920 | Yes (proprietary) |
| Egnyte Business | 220 | 880 | Limited (whole-file) |

Reading: oyatie's parallel upload throughput leads thanks to ScyllaDB-backed metadata + horizontal SeaweedFS volume servers. Single-stream is competitive; bottleneck is HSM lease acquisition for first chunk.

## Workload (c) — Delta sync bandwidth savings (typical office-doc churn; edit + save 1 GB Excel daily)

| Platform | Daily bytes uploaded (compressed) | Bandwidth saving |
|---|---:|---:|
| oyatie drive (paid, FastCDC) | 12 MiB (changed chunks only) | 98.8% |
| Google Drive Business | 32 MiB | 96.8% |
| Microsoft OneDrive | 28 MiB | 97.2% |
| Dropbox Business (rsync) | 18 MiB | 98.2% |
| Box Business Plus | 64 MiB (smaller-chunk; less optimal for big files) | 93.8% |
| Egnyte Business | 1 GiB (whole-file) | 0% (no delta sync) |

Reading: Dropbox + oyatie lead in delta-sync efficiency. FastCDC's content-defined boundary detection avoids the "shift by 1 byte → re-upload everything" pitfall of fixed-block chunking.

## Workload (d) — Preview generation latency (1024×1024 thumbnail for PDF)

| Platform | p99 (ms) | Sandboxed? | Preview encrypted? |
|---|---:|---|---|
| oyatie drive (paid, Cloud Hypervisor sandbox) | 4 200 | Yes (Cloud Hypervisor; tmpfs; 2 GiB cap) | Yes (same envelope as parent) |
| oyatie drive (paid) | 2 800 | Yes (warm pod pool) | Yes |
| Google Drive Business | ~ 3 200 | Limited (Google's preview service) | Limited (server can re-render) |
| Microsoft OneDrive | ~ 3 800 | Limited (Office Online preview) | Limited |
| Dropbox Business | ~ 4 800 | Limited (Dropbox preview service) | No (preview unencrypted at rest) |
| Box Business Plus | ~ 4 200 | Limited (Box Skills) | No |
| Egnyte Business | ~ 6 500 | Limited | No |

Reading: oyatie's preview is sandboxed (per ADR-DRIVE-0005) + preview cache encrypted under the same envelope as the original (per ADR-DRIVE-001 Constraint DRIVE-C14). Latency is competitive thanks to warm Cloud Hypervisor pod pool at paid tier.

## Workload (e) — Cross-region replication lag (write in us-east → read in eu-west)

| Platform | p95 lag (s) | RPO |
|---|---:|---|
| oyatie drive (paid, SeaweedFS geo-rep) | 32 | 60 s |
| oyatie drive (paid, sync replication for high-pack tenants) | 8 | 30 s |
| Google Drive Business | ~ 5 (Google global infrastructure) | Tens of seconds |
| Microsoft OneDrive | ~ 12 (Azure regions) | Minutes |
| Dropbox Business | ~ 18 | Minutes |
| Box Business Plus | ~ 20 | Minutes |
| Egnyte Business | ~ 30 | Minutes |

Reading: Google leads in cross-region speed (planetary infrastructure). oyatie paid competitive with synchronous replication enabled for high-pack tenants.

## Workload (f) — Annual TCO for 50k-user enterprise (50 TB hot data + 500 TB archive)

| Platform | Hardware/Compute (USD) | Licence (USD) | Ops (USD) | Total (USD/year) |
|---|---:|---:|---:|---:|
| oyatie drive (paid self-hosted) | 720 000 | 0 | 372 000 (3 SRE × 0.4 FTE) | 1 092 000 |
| oyatie drive (paid) | 1 590 000 | 0 | 620 000 (5 SRE × 0.4 FTE) | 2 210 000 |
| Google Drive Business Standard ($12/user/mo) | 0 | 7 200 000 | 248 000 | 7 448 000 |
| Microsoft OneDrive (M365 E3 portion) | 0 | 5 400 000 ($9/user/mo × 50k × 12) | 248 000 | 5 648 000 |
| Microsoft OneDrive (M365 E5) | 0 | 21 000 000 ($35/user/mo) | 248 000 | 21 248 000 |
| Dropbox Business Advanced ($24/user/mo) | 0 | 14 400 000 | 248 000 | 14 648 000 |
| Box Business Plus ($25/user/mo) | 0 | 15 000 000 | 248 000 | 15 248 000 |
| Egnyte Business ($30/user/mo) | 0 | 18 000 000 | 248 000 | 18 248 000 |

Reading: oyatie paid is ~ 7× cheaper than Google Drive Business at 50k seats. paid is competitive with M365 E5 while delivering FIPS-140-3 L3 + per-tenant HSM. The per-seat licensing of all SaaS competitors makes them prohibitively expensive at enterprise scale.

## Caveats

- Hardware costs amortize over 5+ years; first-year capex higher.
- Google Drive prices reflect Workspace Business Standard list (mid-2025); enterprise discounts typically 30-40%.
- Dropbox + Box pricing assumes Advanced/Plus tier; lower tiers exist for SMB.
- Egnyte Business is the most expensive per-seat but offers strong on-prem/hybrid options.
- Cross-region replication lag depends heavily on geographic distance + network conditions.

## Reproducibility

The benchmark harness lives at `benchmarks/drivebench/`. Run with:

```sh
cargo run -p oya-dev-cli -- benchmarks drive \
    --workload 50k-users-50tb-hot \
    --tenant-class paid \
    --comparators gdrive,onedrive,dropbox,box,egnyte \
    --include-fastcdc-dedup \
    --output ./benchmark-results.json
```

Comparator runs require valid SaaS sandbox + Google Workspace + M365 + Dropbox Business + Box trials. Results live at `benchmarks/results/drive/<date>.csv` and are re-run quarterly.
