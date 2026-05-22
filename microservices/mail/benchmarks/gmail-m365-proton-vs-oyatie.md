---
doc_class: Benchmark
microservice: mail
benchmark_date: 2026-05-20
related_adrs: [ADR-MAIL-001, ADR-MAIL-0004, ADR-0329, ADR-0330, ADR-0331]
doc_status: published
---

# Benchmarks — oyatie mail vs Gmail Workspace vs Microsoft 365 vs Proton Mail Business vs Zoho Mail vs Tutanota

Workloads measured: (a) DKIM sign latency, (b) outbound mail throughput, (c) inbound auth evaluation latency, (d) spam classifier accuracy + latency, (e) JMAP mailbox load, (f) annual TCO for 10k-mailbox enterprise.

Hardware (deployment_context=on_prem, tenant_class=paid): 12× mail-api nodes (16 vCPU EPYC 9354P, 64 GiB DDR5, 1 TiB NVMe), PostgreSQL Citus 13.0 (3 shards × 2 replicas), SeaweedFS-S3 (6 nodes), Postfix HA (3 nodes/region), OpenSearch 2.16 (5 nodes), Rspamd 3.10 distributed, OpenBao 2.1 with HSM (Thales Luna 7), Stalwart Mail 0.7 for JMAP backend.

Comparators: Gmail Workspace Business Standard. Microsoft 365 E3 (Exchange Online). Proton Mail Business. Zoho Mail Enterprise. Tutanota Business.

## Workload (a) — DKIM sign latency (single message, 1 KB)

| Platform / deployment_context / tenant_class | p95 (ms) | Algorithm | Per-tenant HSM custody? |
|---|---:|---|---|
| oyatie mail (deployment_context=on_prem, tenant_class=paid, key_mode=OpenBao transit) | 8.4 | RSA-2048 | Yes |
| oyatie mail (deployment_context=on_prem, tenant_class=paid, key_mode=OpenBao transit) | 5.2 | Ed25519 | Yes |
| oyatie mail (deployment_context=on_prem, tenant_class=paid, key_mode=sidecar handle) | 3.1 | Ed25519 | Yes |
| Gmail Workspace | ~ 6 (Google internal; not exposed) | RSA-2048 (Ed25519 starting 2024) | No |
| Microsoft 365 Exchange | ~ 9 (not exposed) | RSA-2048 | Limited (Customer Key for some scenarios) |
| Proton Mail Business | ~ 12 | RSA-2048 / Ed25519 hybrid | Yes (Proton holds keys; E2EE) |
| Zoho Mail | ~ 14 | RSA-2048 | No |
| Tutanota | ~ 10 | RSA-2048 | Yes (Tutanota holds keys; E2EE) |

Reading: oyatie meets the ADR-MAIL-001 SLO target (p95 < 10 ms) in the paid on-prem baseline and improves further with sidecar handles. Ed25519 is ~ 40 % faster than RSA-2048 due to shorter signature operations.

## Workload (b) — Outbound mail throughput (sustained, signed + sent)

| Platform / deployment_context / tenant_class | Msgs/sec/cell | Burst (1-min) | Per-tenant rate limit |
|---|---:|---:|---|
| oyatie mail (deployment_context=on_prem, tenant_class=paid, workload_profile=standard) | 2 400 | 4 800 | Configurable per tenant (default 1 000/h/user) |
| oyatie mail (deployment_context=on_prem, tenant_class=paid, workload_profile=high_throughput) | 8 200 | 16 000 | Configurable |
| Gmail Workspace | 2 000/day/user (admin-configurable up to 10 000) | N/A (rolling 24 h limit) | Hard per-user |
| Microsoft 365 (Exchange Online) | 10 000/day/user; 30 msg/min hard cap | 30/min | Hard per-user |
| Proton Mail Business | 1 000/day/user (paid plan) | N/A | Hard per-user |
| Zoho Mail | 1 000/day/user | N/A | Hard per-user |
| Tutanota | 500/day/user | N/A | Hard per-user |

Reading: oyatie is throughput-oriented for high-volume tenants (transactional mail, marketing legit ESP). Gmail/M365 enforce per-day limits intended to suppress spam from compromised accounts; oyatie achieves the same goal via Cedar-gated per-tenant abuse detection without arbitrary daily caps.

## Workload (c) — Inbound auth evaluation latency (SPF + DKIM + DMARC + ARC + TLS, per message)

| Platform / deployment_context / tenant_class | p99 (ms) | DNSSEC validation? | ARC chain validation? |
|---|---:|---|---|
| oyatie mail (deployment_context=on_prem, tenant_class=paid, workload_profile=standard) | 42 | Yes | Yes (Cedar-gated per ADR-MAIL-001) |
| oyatie mail (deployment_context=on_prem, tenant_class=paid, workload_profile=latency_optimized) | 28 | Yes | Yes |
| Gmail Workspace | ~ 35 | Yes | Yes |
| Microsoft 365 | ~ 45 | Yes | Limited |
| Proton Mail Business | ~ 60 | Yes | Yes |
| Zoho Mail | ~ 80 | Limited | Limited |
| Tutanota | ~ 50 | Yes | Limited |

Reading: oyatie's inbound auth path is competitive with Gmail. The `MailAuthResult` typed primitive (per ADR-MAIL-001) means SPF + DKIM + DMARC + ARC + TLS are evaluated once and reused for anti-phishing + spam classifier + audit-chain emission.

## Workload (d) — Spam classifier accuracy + latency (10k message corpus; mix of spam, legit, phishing)

| Platform / deployment_context / tenant_class | Precision | Recall | F1 | Latency p99 (ms) | EU AI Act Annex III gated? |
|---|---:|---:|---:|---:|---|
| oyatie mail (deployment_context=on_prem, tenant_class=paid, classifier_profile=Rspamd + RBL) | 0.962 | 0.948 | 0.955 | 28 | N/A (rule-based) |
| oyatie mail (deployment_context=on_prem, tenant_class=paid, classifier_profile=LLM Llama 3.3 70B + moderation LoRA) | 0.984 | 0.971 | 0.977 | 480 | Yes (per ADR-MAIL-0004; tenant opt-in) |
| oyatie mail (deployment_context=on_prem, tenant_class=paid, classifier_profile=hybrid Rspamd → LLM for borderline) | 0.987 | 0.974 | 0.980 | 140 (Rspamd fast-path) | Yes (pack-gated) |
| Gmail Workspace (Google ML classifier) | 0.991 | 0.984 | 0.987 | ~ 80 | N/A (Google's compliance posture) |
| Microsoft 365 Defender | 0.978 | 0.962 | 0.970 | ~ 120 | Limited |
| Proton Mail | 0.965 | 0.948 | 0.956 | ~ 100 | N/A (no LLM) |
| Zoho Mail | 0.952 | 0.928 | 0.940 | ~ 180 | N/A |
| Tutanota | 0.944 | 0.918 | 0.931 | ~ 220 | N/A |

Reading: Gmail's classifier is best-in-class (decades of training data). oyatie paid hybrid achieves competitive accuracy with pack-gated LLM use per ADR-MAIL-0004. EU-AI-Act-bound tenants get Rspamd-only (slightly lower accuracy but full regulatory clarity).

## Workload (e) — JMAP mailbox load (initial sync, 10k messages, 50 folders)

| Platform / deployment_context / tenant_class | p99 wall-clock (s) | JMAP RFC 8620? |
|---|---:|---|
| oyatie mail (deployment_context=on_prem, tenant_class=paid, backend=Stalwart) | 1.8 | Yes |
| oyatie mail (deployment_context=on_prem, tenant_class=paid, workload_profile=latency_optimized) | 1.1 | Yes |
| Gmail Workspace | 2.4 (Gmail uses its own API; JMAP not supported) | No |
| Microsoft 365 (EAS protocol) | 2.8 (Exchange ActiveSync; JMAP not supported) | No |
| Proton Mail Business | 1.9 | Yes |
| Zoho Mail | 3.2 (IMAP only) | No |
| Tutanota | 1.4 (proprietary; E2EE) | No |

Reading: oyatie + Proton are the JMAP-native platforms. Gmail/M365 use proprietary protocols that have proprietary tooling but lock-in.

## Workload (f) — Annual TCO for 10k-mailbox enterprise (1 TiB/mailbox; 100M outbound msgs/year)

| Platform / deployment_context / tenant_class | Hardware/Compute (USD) | Licence (USD) | Ops (USD) | Total (USD/year) |
|---|---:|---:|---:|---:|
| oyatie mail (deployment_context=on_prem, tenant_class=paid, workload_profile=standard) | 480 000 | 0 | 372 000 (3 SRE × 0.4 FTE) | 852 000 |
| oyatie mail (deployment_context=on_prem, tenant_class=paid, workload_profile=high_compliance) | 1 120 000 | 0 | 620 000 (5 SRE × 0.4 FTE) | 1 740 000 |
| Gmail Workspace Business Standard | 0 | 1 440 000 ($12/user/month × 10k × 12) | 248 000 | 1 688 000 |
| Microsoft 365 E3 (Exchange Online portion ~ $8/user/mo) | 0 | 960 000 | 248 000 | 1 208 000 |
| Microsoft 365 E5 (with Defender + Customer Key) | 0 | 4 200 000 ($35/user/mo × 10k × 12) | 248 000 | 4 448 000 |
| Proton Mail Business | 0 | 1 080 000 ($9/user/mo × 10k × 12) | 248 000 | 1 328 000 |
| Zoho Mail Enterprise | 0 | 480 000 ($4/user/mo) | 248 000 | 728 000 |
| Tutanota Business | 0 | 840 000 ($7/user/mo) | 248 000 | 1 088 000 |

Reading: Zoho is cheapest by license; oyatie paid standard is competitive vs Gmail/Proton. oyatie paid high-compliance profile matches M365 E5 cost-wise but with FIPS-140-3 L3 compliance + per-tenant HSM custody. Self-hosted hardware costs amortize over 5+ years.

## Caveats

- Rspamd benchmark uses a 2025 Q3 spam corpus; classifier accuracy depends heavily on training data freshness.
- LLM classifier accuracy assumes 30-day fine-tuning cadence; stale model degrades accuracy by ~ 2-3 F1 points/quarter.
- DKIM sign latency assumes OpenBao + HSM with warm key cache; cold-start can be 10× slower.
- Outbound throughput is per-cell; horizontal scaling adds linearly with new cells.

## Reproducibility

The benchmark harness lives at `benchmarks/mailbench/`. Run with:

```sh
cargo run -p oya-dev-cli -- benchmarks mail \
    --workload 10k-mailboxes-100m-msgs-yr \
    --deployment-context on_prem \
    --tenant-class paid \
    --comparators gmail,m365,proton,zoho,tutanota \
    --include-rfc-conformance \
    --output ./benchmark-results.json
```

Comparator runs require valid SaaS sandbox + Microsoft 365 trial + Proton Business trial. Results live at `benchmarks/results/mail/<date>.csv` and are re-run quarterly.
