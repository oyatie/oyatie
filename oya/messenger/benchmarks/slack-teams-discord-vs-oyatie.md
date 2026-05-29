---
doc_class: Benchmark
microservice: messenger
benchmark_date: 2026-05-20
related_adrs: [ADR-MSG-001, ADR-MSGR-0001, ADR-MSGR-0003, ADR-0316]
doc_status: published
---

# Benchmarks — oyatie messenger vs Slack vs Microsoft Teams vs Discord vs Telegram vs WhatsApp Business

Workloads measured: (a) DM send + read latency, (b) 100k-member channel MLS Commit latency, (c) huddle join time, (d) cross-tenant DM E2E with verified-corp-email gate, (e) eDiscovery ciphertext export throughput, (f) annual TCO for a 50k-employee enterprise.

Hardware (oyatie paid on-prem): 12× messenger-api nodes (16 vCPU EPYC 9354P, 64 GiB DDR5, 1 TiB NVMe), PostgreSQL Citus 13.0 (3 shards × 2 replicas), ScyllaDB 6.0 (9-node, RF=3 across 3 DCs), Pulsar 3.3 (5-broker), LiveKit 1.7 (3-node SFU), OpenBao 2.1 with HSM (Thales Luna 7).

Comparators: Slack Enterprise Grid (50k seats). Microsoft Teams E5. Discord Enterprise (custom contract). Telegram Premium (per-MAU). WhatsApp Business Platform (Cloud API).

## Workload (a) — 1:1 DM send + read latency (text, < 1 KB)

| Platform | Send p99 (ms) | Read p99 (ms) | E2EE on send? |
|---|---:|---:|---|
| oyatie messenger (paid, MLS default ciphersuite) | 118 | 42 | Yes (RFC 9420) |
| oyatie messenger (paid advanced) | 78 | 28 | Yes (RFC 9420) |
| Slack Enterprise Grid | 280 | 95 | No (server-side TLS only) |
| Microsoft Teams E5 | 320 | 140 | No (E2EE for Teams meetings only) |
| Discord Enterprise | 240 | 90 | No |
| Telegram (Secret Chats) | 220 | 110 | Yes (opt-in only) |
| WhatsApp Business Cloud API | 380 | 280 | Yes (Signal protocol) |

Reading: oyatie achieves competitive send latency *while* doing full MLS encryption + Cedar evaluation + audit-chain emit. The MLS commit accept is amortized across the encryption work the client already does; ScyllaDB ciphertext write + Pulsar fanout dominate the path.

## Workload (b) — Large-group MLS Commit accept latency (100k members joining)

| Platform | Members | Add-member p99 (ms) | Group-creation total wall-clock |
|---|---:|---:|---:|
| oyatie messenger (paid, MLS default) | 100 000 | 478 | 11 min |
| oyatie messenger (paid advanced, MLS default) | 100 000 | 312 | 7 min |
| oyatie messenger (paid advanced, MLS P-384) | 100 000 | 692 | 14 min |
| oyatie messenger (paid advanced) | 500 000 | 798 | 38 min |
| Slack Enterprise Grid (no E2EE) | 100 000 | 84 (no key exchange) | 2 min |
| Microsoft Teams (no E2EE) | 10 000 (limit) | 92 | 1 min |
| Discord (server-side) | 100 000 | 110 | 4 min |
| Telegram (channel; one-to-many; not E2EE group) | 1 000 000 | 60 | 3 min |
| WhatsApp Business broadcast list (256-member limit) | 256 | 280 | 18 s |

Reading: oyatie pays an MLS-protocol tax for cryptographically-verifiable group membership. The cost is O(log N) per addition vs O(1) for server-side broadcasts. At 100k members, the SLO target (per ADR-MSG-001) is p99 ≤ 500 ms which we meet at paid default ciphersuite. P-384 ciphersuite is ~50% slower due to larger key operations.

## Workload (c) — Huddle join time (audio + video, 5 participants)

| Platform | p99 (ms) | SFU-side key visibility |
|---|---:|---|
| oyatie messenger (paid, LiveKit SFU) | 2 780 | SFU never sees keys (MLS-derived SRTP) |
| oyatie messenger (paid advanced, edge POPs) | 1 920 | SFU never sees keys |
| Slack Huddles | 1 800 | Slack's SFU sees keys (server-side mix) |
| Microsoft Teams Meet | 2 200 | Teams sees keys (server-side); E2EE only with Teams Premium for 1:1 |
| Discord Voice | 1 200 | Discord sees keys (server-side) |
| Telegram Voice Calls | 2 400 | Telegram does not have a server-side SFU for groups (1:1 P2P) |
| WhatsApp Group Voice | 2 100 | Server-side E2EE for groups (Signal protocol) |

Reading: oyatie's SFU is intentionally blind to media keys — this is a unique property. Slack/Teams/Discord SFUs all see media keys, which means a server compromise reveals plaintext audio/video. oyatie's MLS-derived SRTP keys are negotiated client-to-client via the channel's MLS group epoch; the SFU just routes opaque SRTP packets.

## Workload (d) — Cross-tenant DM with verified-corp-email E2E

| Platform | p99 (ms) | E2EE? | Verified-corp-email gate? |
|---|---:|---|---|
| oyatie messenger (paid) | 168 | Yes (MLS) | Yes (Cedar-gated) |
| Slack | 480 | No | Limited (Slack requires both orgs on Slack) |
| Microsoft Teams cross-org chat | 520 | No | Limited (Teams federation; manual approval per org) |
| Discord (cross-server DM) | 240 | No | No |
| Telegram | 220 | No (unless Secret Chat) | No |
| WhatsApp Business | 380 | Yes (Signal) | No (phone-number-based) |

Reading: oyatie is the only platform that combines MLS E2EE + per-tenant Cedar policy + verified-corporate-email gating. Slack and Teams cross-org chat support cross-organization conversations but without E2EE or cryptographic identity verification.

## Workload (e) — eDiscovery export (ciphertext + membership trail, 1M messages)

| Platform | Export wall-clock (min) | Plaintext exported? | Tenant-controlled key custody? |
|---|---:|---|---|
| oyatie messenger (paid) | 18 | No (ciphertext-only; tenant legal-hold appliance decrypts) | Yes |
| Slack Enterprise Grid eDiscovery | 24 | Yes (plaintext export) | No (Slack holds keys) |
| Microsoft Teams Compliance | 32 | Yes (plaintext export) | Limited (Customer Key for some scenarios) |
| Discord (no enterprise eDiscovery) | N/A | N/A | N/A |
| Telegram (no enterprise eDiscovery) | N/A | N/A | N/A |
| WhatsApp Business (no enterprise eDiscovery) | N/A | N/A | N/A |

Reading: oyatie's eDiscovery export is structurally different — server exports ciphertext + membership + audit-chain, tenant's legal-hold appliance owns the decryption keys. This satisfies "we cannot read your messages even under subpoena" while honoring lawful holds.

## Workload (f) — Annual TCO for 50k-employee enterprise (1B messages/year, 5k channels, 50 cross-tenant federations)

| Platform | Hardware/Compute (USD) | Licence (USD) | Ops (USD) | Total (USD/year) |
|---|---:|---:|---:|---:|
| oyatie messenger (paid self-hosted) | 640 000 | 0 | 372 000 (3 SRE × 0.4 FTE) | 1 012 000 |
| oyatie messenger (paid advanced) | 1 480 000 | 0 | 620 000 (5 SRE × 0.4 FTE) | 2 100 000 |
| Slack Enterprise Grid | 0 | 7 380 000 ($147.60/seat/year × 50k) | 248 000 | 7 628 000 |
| Microsoft Teams E5 | 0 | 6 600 000 ($132/seat/year × 50k; E5 license includes M365) | 248 000 | 6 848 000 |
| Discord Enterprise (custom) | 0 | est 1 800 000 (per-seat custom) | 372 000 | 2 172 000 |
| Telegram Premium | 0 | est 2 970 000 ($59.40/year per user) | 248 000 | 3 218 000 |
| WhatsApp Business Cloud API | 0 | 4 800 000 (per-conversation pricing; high-volume estimate) | 248 000 | 5 048 000 |

Reading: oyatie paid self-hosted is 7.5× cheaper than Slack Enterprise Grid at the same scale. paid advanced is still 3.5× cheaper than Slack while delivering FIPS-140-3 L3 compliance and 500k-member channels.

Caveats:

- Slack pricing reflects Enterprise Grid published list (mid-2025); enterprise discounts typically 30-40%.
- Teams E5 includes the broader M365 productivity platform; pure messenger comparison is harder.
- Hardware costs assume on-prem; cloud (AWS/GCP) increases compute cost ~ 2× for equivalent IOPS.
- Ops costs assume mature SRE; first-year ops is typically 2× steady-state.

## Reproducibility

The benchmark harness lives at `benchmarks/messengerbench/`. Run with:

```sh
cargo run -p oya-dev-cli -- benchmarks messenger \
    --workload 50k-employees-1b-msgs-yr \
    --tenant-class paid \
    --comparators slack,teams,discord,telegram,whatsapp \
    --include-mls-load-tests \
    --output ./benchmark-results.json
```

Comparator runs require valid SaaS sandbox accounts + Slack Enterprise Grid trial. Results live at `benchmarks/results/messenger/<date>.csv` and are re-run quarterly.

## Caveats

- MLS overhead is real and visible at 100k+ member groups; do not promise Slack-level send latency for large groups at default ciphersuite.
- LiveKit edge POPs require regional deployment — first-year deployment may not match paid advanced's huddle-join SLO until POPs are warm.
- eDiscovery export wall-clock depends heavily on the tenant's legal-hold appliance throughput; benchmark above is server-side only.
