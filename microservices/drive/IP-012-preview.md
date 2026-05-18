---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-drive-foundation
impl_plan_id: IP-012-preview
status: pending
execution_unit: ChangeSet
owner: axis-drive + ops-security
acceptance_lanes: [cargo-build, cargo-nextest, oya-governance-preview-sandbox-conformance, cis-k8s]
---

# IP-012: preview BC — libvips + qpdf + LibreOffice-gVisor + ffmpeg sandbox

## Intent

Stand up `oya-drive-preview-*` BC per ADR-DRIVE-0005. Per-renderer sandbox + per-renderer timeout + per-tenant quota + sandbox-isolation E2E test.

## Crates

`oya-drive-preview-{kernel,domain,usecase,api,adapter,adapter-libvips,adapter-qpdf,adapter-libreoffice,adapter-ffmpeg,rest,worker,app}` (12 crates).

## Acceptance Gates

```bash
cargo nextest run --test e2e_preview_sandbox_egress_denied
cargo nextest run --test e2e_preview_sandbox_host_fs_denied
cargo nextest run --test e2e_preview_sandbox_malicious_macro
cargo run -p oya-dev-cli -- gate validate cis-k8s --microservice drive
```

## References

- ADR-DRIVE-0005.
- PRD-drive §FR-09; AC-13.
