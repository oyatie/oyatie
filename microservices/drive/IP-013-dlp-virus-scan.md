---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-drive-foundation
impl_plan_id: IP-013-dlp-virus-scan
status: pending
execution_unit: ChangeSet
owner: axis-drive + ops-security + foundry-runtime
acceptance_lanes: [cargo-build, cargo-nextest, oya-check-scan-correctness-zero-tolerance]
---

# IP-013: dlp-virus-scan BC — ClamAV + OPSWAT + DLP rules + foundry-runtime ML handoff

## Intent

Stand up `oya-drive-dlp-virus-scan-*` BC. ClamAV primary + OPSWAT secondary (pack-us-healthcare + pack-eu); in-tree DLP rule engine + foundry-runtime ML handoff. Scan-at-upload pipeline.

## Crates

`oya-drive-dlp-virus-scan-{kernel,domain,usecase,api,adapter,adapter-clamav,adapter-opswat,worker,app}` (9 crates).

## Acceptance Gates

```bash
cargo nextest run -p oya-drive-dlp-virus-scan-domain -- eicar_quarantine
cargo nextest run -p oya-drive-dlp-virus-scan-domain -- dlp_blocks_share
cargo nextest run -p oya-drive-dlp-virus-scan-adapter-clamav -- signature_kat
cargo run -p oya-dev-cli -- gate validate scan-correctness-zero-tolerance --microservice drive
```

## ChangeSet metadata

```yaml
changeset_id: CS-DRIVE-IP-013-dlp-virus-scan
depends_on_changesets: [CS-DRIVE-IP-003-file-store-kernel-domain]
parallel_safe_with_changesets: [CS-DRIVE-IP-005-folder-hierarchy, CS-DRIVE-IP-010-permissions]
enables: [CS-DRIVE-IP-006-upload, CS-DRIVE-IP-012-preview, CS-DRIVE-IP-009-share-link]
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | EICAR test signature quarantined; never reaches durable bucket | `cargo nextest run -p oya-drive-dlp-virus-scan-domain -- eicar_quarantine` |
| AC-02 | DLP rule (PCI Primary Account Number, US SSN, EU IBAN) blocks share-out | `cargo nextest run -p oya-drive-dlp-virus-scan-domain -- dlp_blocks_share` |
| AC-03 | ClamAV signature KAT passes (known signature fixture set) | `cargo nextest run -p oya-drive-dlp-virus-scan-adapter-clamav -- signature_kat` |
| AC-04 | Zero-tolerance correctness lane green (100% scan coverage on uploads) | `cargo run -p oya-dev-cli -- gate validate scan-correctness-zero-tolerance --microservice drive` |

## Build Sequence

1. Kernel: `VirusScanner`, `DlpEngine`, `ScanVerdict` ports.
2. Domain: `ScanRequest`, `ScanResult`, `Quarantine`, `DlpRule`.
3. Adapters: `-adapter-clamav` (primary), `-adapter-opswat` (secondary; pack-us-healthcare + pack-eu).
4. Worker drains scan queue; foundry-runtime ML hand-off for advanced DLP.
5. `cargo nextest run -p oya-drive-dlp-virus-scan-*`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-drive FR | FR-10 (virus scan), FR-11 (DLP) |
| PRD-drive AC | AC-11, AC-12 |
| ADR | ADR-DRIVE-0005 |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Outdated AV signatures miss new malware | Hourly signature pull; CI alarm if signature db > 24h stale |
| DLP false-positive blocks legitimate share | Tenant-admin override workflow with audit-chain seal |
| Scan timeout on large file (≥ 10GB) | Stream-based scan; chunk-progressive verdict |

## References

- ADR-DRIVE-0005 (preview pipeline sandboxing carries DLP review surface).
- PRD-drive §FR-10; §FR-11; AC-11; AC-12.
- ClamAV documentation (`docs.clamav.net`).
- OPSWAT MetaDefender documentation (`docs.opswat.com/mdcore`).
- EICAR Anti-Virus test file specification (`www.eicar.org/anti-malware-testfile`).
- NIST SP 800-83 Rev. 1 (Guide to Malware Incident Prevention and Handling).
