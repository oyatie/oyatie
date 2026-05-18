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

## References

- ADR-DRIVE-0005 (preview pipeline sandboxing carries DLP review surface).
- PRD-drive §FR-10; §FR-11; AC-11; AC-12.
- ClamAV docs; OPSWAT MetaDefender docs; EICAR test signature.
