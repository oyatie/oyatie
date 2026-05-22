---
doc_class: Benchmark
microservice: identity
benchmark_date: 2026-05-20
related_adrs: [ADR-ID-001, ADR-identity-001, ADR-identity-004, ADR-0316]
doc_status: published
---

# Benchmarks — oyatie identity vs Okta vs Auth0 vs Microsoft Entra ID vs Ping Identity vs JumpCloud

Workloads measured: (a) WebAuthn verify latency, (b) OIDC token issuance throughput, (c) SCIM bulk provisioning, (d) external IdP federation token exchange, (e) recovery ceremony wall-clock, (f) annual TCO for 50k-employee workforce.

Hardware (oyatie paid with per_seat billing_component on-prem): 12× identity-api nodes (16 vCPU EPYC 9354P, 64 GiB DDR5, 1 TiB NVMe), PostgreSQL Citus 13.0, Valkey 8.1 cluster (5 nodes), Kafka 3.8, OpenBao 2.1 with HSM, Zitadel 2.65 (per IP-001).

Comparators: Okta Workforce Identity. Auth0 Enterprise. Microsoft Entra ID P2. Ping Identity Enterprise. JumpCloud Platform Premium.

## Workload (a) — WebAuthn verify latency (hardware-backed YubiKey 5; full ceremony)

| Platform | p95 (ms) | Attestation validated? | AAGUID trust catalog enforced? |
|---|---:|---|---|
| oyatie identity (paid with per_seat billing_component) | 78 | Yes (per ADR-ID-001) | Yes (FIDO MDS auto-refresh) |
| oyatie identity (paid with per_usage billing_component) | 52 | Yes | Yes |
| Okta Workforce | ~ 85 | Yes (Okta Verify + FastPass) | Yes (FIDO MDS) |
| Auth0 Enterprise | ~ 110 | Yes | Yes |
| Microsoft Entra ID P2 | ~ 95 | Yes (Entra Authenticator) | Yes |
| Ping Identity Enterprise | ~ 92 | Yes (PingID) | Yes |
| JumpCloud Premium | ~ 140 | Yes | Limited |

Reading: oyatie meets the ADR-ID-001 SLO target (p95 ≤ 100 ms) at paid with per_seat billing_component and beats it at paid with per_usage billing_component. All major vendors validate attestation; JumpCloud's AAGUID catalog is less comprehensive.

## Workload (b) — OIDC token issuance throughput (sustained tokens/sec)

| Platform | Tokens/sec/cell | Per-token claims (size) |
|---|---:|---:|
| oyatie identity (paid with per_seat billing_component) | 4 800 | ~ 720 bytes (acr, amr, tenant_id, principal_id, audience_type, home_cell, credential_epoch, recovery_epoch + std OIDC claims) |
| oyatie identity (paid with per_usage billing_component) | 12 000 | ~ 720 bytes |
| Okta Workforce | ~ 8 000 | ~ 540 bytes |
| Auth0 Enterprise | ~ 6 000 | ~ 600 bytes |
| Microsoft Entra ID P2 | ~ 12 000 | ~ 580 bytes |
| Ping Identity Enterprise | ~ 7 000 | ~ 620 bytes |
| JumpCloud Premium | ~ 3 500 | ~ 480 bytes |

Reading: oyatie matches enterprise leaders on throughput while carrying richer claims (dual-context + credential_epoch + recovery_epoch enable per-action policy). Larger token size is the trade-off.

## Workload (c) — SCIM bulk provisioning (10k user-create operations)

| Platform | Total wall-clock (min) | Bulk endpoint? |
|---|---:|---|
| oyatie identity (paid with per_seat billing_component) | 2.1 | Yes (`/v2/Bulk` per ADR-identity-003) |
| oyatie identity (paid with per_usage billing_component) | 0.9 | Yes |
| Okta Workforce | ~ 3.2 | Yes |
| Auth0 Enterprise | ~ 5.4 | Yes (Auth0 Bulk API) |
| Microsoft Entra ID P2 | ~ 4.8 | Yes (Microsoft Graph batch) |
| Ping Identity Enterprise | ~ 4.0 | Yes |
| JumpCloud Premium | ~ 12.4 | Limited |

Reading: oyatie paid with per_usage billing_component is the fastest thanks to PostgreSQL Citus shard-parallel writes + Cedar policy pre-compilation. JumpCloud's bulk is single-threaded server-side.

## Workload (d) — External IdP federation token exchange (Okta → oyatie OIDC ID token)

| Platform | p99 round-trip (ms) | Inbound OIDC federation? |
|---|---:|---|
| oyatie identity (paid with per_seat billing_component) | 220 | Yes (per IP-011) |
| oyatie identity (paid with per_usage billing_component) | 140 | Yes |
| Okta Workforce (as IdP) | N/A (Okta is the source) | N/A |
| Auth0 (inbound federation) | ~ 240 | Yes |
| Microsoft Entra ID (B2B guest federation) | ~ 380 | Yes |
| Ping Identity Enterprise | ~ 280 | Yes |
| JumpCloud Premium | ~ 320 | Yes |

Reading: oyatie's federation token exchange is competitive. The path includes OIDC discovery cache, claim re-mapping to oyatie's audience_type model, and Cedar evaluation.

## Workload (e) — Recovery ceremony wall-clock (loss of all devices + recovery code + passphrase)

| Platform | Median wall-clock (min) | Operator can decrypt? |
|---|---:|---|
| oyatie identity (paid with per_seat billing_component) | 4 (user-driven) | No (per ADR-ID-001) |
| oyatie identity (paid with per_usage billing_component) | 4 | No |
| Okta Workforce | ~ 12 (Okta admin reset) | Yes (admin can reset to passwordless) |
| Auth0 Enterprise | ~ 8 | Yes (tenant admin can reset) |
| Microsoft Entra ID P2 | ~ 10 | Yes |
| Ping Identity Enterprise | ~ 9 | Yes |
| JumpCloud Premium | ~ 15 | Yes |

Reading: oyatie's recovery is faster + the only one where the operator CANNOT decrypt. All competitors have an admin-driven reset path which means an operator compromise can also compromise user accounts.

## Workload (f) — Annual TCO for 50k-employee workforce

| Platform | Hardware/Compute (USD) | Licence (USD) | Ops (USD) | Total (USD/year) |
|---|---:|---:|---:|---:|
| oyatie identity (paid with per_seat billing_component self-hosted) | 640 000 | 0 | 248 000 (2 SRE × 0.4 FTE) | 888 000 |
| oyatie identity (paid with per_usage billing_component) | 1 140 000 | 0 | 372 000 (3 SRE × 0.4 FTE) | 1 512 000 |
| Okta Workforce Identity ($6/user/mo) | 0 | 3 600 000 | 124 000 | 3 724 000 |
| Auth0 Enterprise ($8/user/mo) | 0 | 4 800 000 | 124 000 | 4 924 000 |
| Microsoft Entra ID P2 ($9/user/mo) | 0 | 5 400 000 | 124 000 | 5 524 000 |
| Ping Identity Enterprise ($5/user/mo, enterprise contract) | 0 | 3 000 000 | 124 000 | 3 124 000 |
| JumpCloud Premium ($24/user/mo) | 0 | 14 400 000 | 124 000 | 14 524 000 |

Reading: oyatie paid with per_seat billing_component is ~ 4× cheaper than Okta. Ping is the only competitor close on TCO. JumpCloud is the most expensive on per-seat pricing.

## Caveats

- Pricing reflects published list (mid-2025); enterprise discounts typically 30-40%.
- Hardware costs amortize over 5+ years; first-year capex higher.
- Some vendors (Microsoft Entra ID) bundle identity with broader productivity suites; pure identity comparison is harder.
- Throughput depends heavily on token size + claim complexity.

## Reproducibility

```sh
cargo run -p oya-dev-cli -- benchmarks identity \
    --workload 50k-employees \
    --tenant-class paid \
    --comparators okta,auth0,entra,ping,jumpcloud \
    --include-recovery-ceremony \
    --output ./benchmark-results.json
```

Results live at `benchmarks/results/identity/<date>.csv` and are re-run quarterly.
