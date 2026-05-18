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

## ChangeSet metadata

```yaml
changeset_id: CS-DRIVE-IP-012-preview
depends_on_changesets: [CS-DRIVE-IP-003-file-store-kernel-domain, CS-DRIVE-IP-013-dlp-virus-scan]
parallel_safe_with_changesets: [CS-DRIVE-IP-009-share-link, CS-DRIVE-IP-011-search-index]
enables: []
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Image 4K preview p95 ≤ 1s; PDF 100p first page p95 ≤ 1s; Office 50p first page p95 ≤ 1s | `cargo nextest run --test e2e_preview_latency` |
| AC-02 | Sandbox refuses all egress (Internet + intra-cluster) — verified by tcpdump trace | `cargo nextest run --test e2e_preview_sandbox_egress_denied` |
| AC-03 | Sandbox refuses host filesystem reads outside the sandboxed dir | `cargo nextest run --test e2e_preview_sandbox_host_fs_denied` |
| AC-04 | Malicious macro (sample LibreOffice payload) cannot execute outside sandbox | `cargo nextest run --test e2e_preview_sandbox_malicious_macro` |
| AC-05 | `oya gate validate cis-k8s --microservice drive` exits 0 (gVisor / seccomp present) | CIS Kubernetes Benchmark v1.9 |

## Build Sequence

1. Kernel: `PreviewRenderer`, `Sandbox`, `RenderQueue` ports.
2. Domain: `PreviewSpec`, `RenderResult`, `RenderTimeout`.
3. Per-renderer adapters: `-adapter-libvips` (image), `-adapter-qpdf` (PDF), `-adapter-libreoffice` (Office in gVisor), `-adapter-ffmpeg` (video).
4. Worker that drains preview requests; per-renderer timeout (≤ 30s) + tenant quota.
5. `cargo nextest run --test e2e_preview_sandbox_egress_denied`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-drive FR | FR-09 (preview without download) |
| PRD-drive NFR | NFR perf — preview latencies; NFR security — sandbox |
| PRD-drive AC | AC-13 |
| ADR | ADR-DRIVE-0005 |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| LibreOffice CVE exploited via malformed Office file | gVisor + seccomp + read-only rootfs; output rasterised to PNG |
| qpdf parser CVE | qpdf in gVisor; output PDF re-emitted via safe rasteriser |
| Render queue starvation under burst | Per-tenant quota + priority queue |

## References

- ADR-DRIVE-0005.
- PRD-drive §FR-09; AC-13.
- gVisor runtime documentation (`gvisor.dev/docs`).
- CIS Kubernetes Benchmark v1.9.
- libvips documentation (`libvips.github.io/libvips`).
- qpdf documentation (`qpdf.readthedocs.io`).
- LibreOffice headless mode reference (`api.libreoffice.org`).
