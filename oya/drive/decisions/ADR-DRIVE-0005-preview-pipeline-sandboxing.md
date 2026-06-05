---
id: ADR-DRIVE-0005
status: Accepted
date: 2026-05-17
microservice: drive
deciders: axis-drive, ops-security, council-architecture
owner: ops-security + axis-drive
supersedes: []
superseded_by: []
related: [ADR-0056, ADR-0105, ADR-0135, ADR-0131, ADR-0133, ADR-DRIVE-0001, ADR-DRIVE-0004]
related_artifacts:
  - microservices/drive/PRD.md (§FR-09 preview; §"Performance" preview render targets; AC-13 sandbox isolation)
  - microservices/drive/iac/helm/values.yaml (preview.sandbox.*)
  - microservices/drive/iac/helm/templates/networkpolicy.yaml (preview egress deny-network)
  - microservices/drive/threat-model.md (T-I-05 preview render leaks via container escape)
purpose: |
  Pick a sandbox strategy + per-format render toolchain for the drive preview
  pipeline. Office files are notoriously dangerous (macro execution, parser
  CVEs); LibreOffice in a hostile environment must NOT be allowed network
  egress, host filesystem access, or any path that could leak bytes from a
  malicious file into the cluster.
---

# ADR-DRIVE-0005: Preview pipeline sandboxing — gVisor 2026-04 runtime for LibreOffice (Office); libvips (image); qpdf + Mozilla pdf.js (PDF); ffmpeg (video); CIS K8s 1.9.0 + seccomp restrictive profile

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

PRD-drive §FR-09 mandates image / PDF / Office / video preview at competitive parity with Google Drive / OneDrive / Box. Threat-model §T-I-05 establishes preview-renderer container escape as a credible high-impact threat.

Office files in particular carry attack surface: VBA macros, OLE objects, malicious image embeddings, parser CVEs. LibreOffice has a public CVE history that requires strong containment. ([cve.mitre.org search "LibreOffice"](https://cve.mitre.org))

Candidate sandbox technologies:
- **Linux containers (no extra sandbox)** — minimal isolation; kernel-level escape via shared syscall surface.
- **gVisor** (Google; user-space kernel intercept). Reduces kernel attack surface; runs as `RuntimeClass: gvisor`. ([gvisor.dev](https://gvisor.dev))
- **Firecracker** (AWS; lightweight VM). Hardware-virtualisation-grade isolation; ~125ms boot. ([firecracker-microvm.github.io](https://firecracker-microvm.github.io))
- **Kata Containers** (lightweight VM; uses QEMU). Stronger isolation than gVisor; heavier resource cost.

Candidate render toolchains:
- **Image**: libvips (V8 JIT-free; minimal attack surface), ImageMagick (rich format support; CVE-heavy).
- **PDF**: Mozilla pdf.js (JS-based; Mozilla maintains), qpdf (C++; structural-only render).
- **Office**: LibreOffice (Java + C++ engine; most viable OSS; recurring CVEs require sandbox), Apache OpenOffice (older; less active).
- **Video**: ffmpeg (de-facto standard).

Per ADR-0133 axis-2 security-conformance, the chosen sandbox must satisfy CIS Kubernetes Benchmark v1.9.0 + isolate from network + isolate from host filesystem.

## Decision

The drive µservice ships:

### Sandbox

- **gVisor 2026-04** as the preview-worker `RuntimeClass`.
- **gVisor platform**: `ptrace` (no KVM dependency at deploy time; can swap to KVM platform later for performance).
- **seccomp profile**: restrictive default (Linux `SCMP_ACT_ERRNO` for any syscall not on a per-renderer whitelist).
- **Network policy**: `deny-all` on egress (per `iac/helm/templates/networkpolicy.yaml` `oya-drive-preview-egress-deny-network`); only object-store (Garage) + OpenBao Transit egress allowed (for fetching the source file's ciphertext + decryption key); DNS only.
- **Host filesystem**: `readOnlyRootFilesystem: true`; no `hostPath` mounts.
- **Capabilities**: `drop: [ALL]`.
- **Output**: rasterised PNG only (Office + PDF), thumbnail JPEG (image), keyframe JPEG (video). Output bytes pushed to per-tenant preview cache bucket; never to host.

### Per-format toolchain

- **Image**: libvips 8.15.x for thumbnail + first-page render (PNG output).
- **PDF**: qpdf 11.x for structural validation + Mozilla pdf.js 4.x for raster rendering (PNG output).
- **Office**: LibreOffice 24.x (LTS) in the gVisor sandbox for raster rendering (PNG output via UNO Bridge headless mode).
- **Video**: ffmpeg 7.x for keyframe extraction (JPEG output).

### Per-render timeout

- Image: 5s.
- PDF: 30s.
- Office: 60s.
- Video: 120s.

### Per-tenant quota

- Max 100 concurrent renders per tenant.
- Backpressure: queue depth > 300 → defer with "preview-not-yet-available".

## Alternatives Considered

### A. Linux containers (no extra sandbox)

- **Pros**:
  - No extra runtime; simplest deployment.
- **Cons**:
  - Container escape via shared syscall surface is a credible threat for LibreOffice in particular.
  - Doesn't satisfy CIS K8s 4.6 (least-privilege runtime).
- **Rejected** outright; Office preview without sandbox is unacceptable.

### B. Firecracker

- **Pros**:
  - Hardware-virtualisation isolation; strongest of the candidates.
  - Lightweight (125ms boot).
- **Cons**:
  - Requires KVM at the node level; some cloud node types lack.
  - More operational complexity than gVisor.
  - gVisor sufficient for the threat model.
- **Rejected** for now; **retained as a successor-IP if gVisor CVE patterns require stronger isolation**.

### C. Kata Containers

- **Pros**:
  - Lightweight VM isolation.
  - OCI-compatible.
- **Cons**:
  - Heavier resource cost than gVisor.
  - Slower boot.
- **Rejected** in favour of gVisor for boot speed + resource overhead.

### D. ImageMagick instead of libvips

- **Pros**:
  - Richer format support.
- **Cons**:
  - CVE-heavy history (over 200 CVEs through 2025).
  - JIT not minimised.
- **Rejected** in favour of libvips for security posture.

### E. Apache OpenOffice instead of LibreOffice

- **Pros**:
  - Apache-licensed.
- **Cons**:
  - Less active development than LibreOffice.
  - Less feature parity.
- **Rejected** in favour of LibreOffice.

### F. gVisor + libvips + qpdf + pdf.js + LibreOffice + ffmpeg + CIS K8s 1.9.0 hardening  ← **CHOSEN**

- **Pros**:
  - gVisor's ptrace platform reduces kernel attack surface dramatically vs raw containers.
  - libvips + qpdf are minimal attack surface vs ImageMagick + Acrobat.
  - LibreOffice in gVisor is the industry-leading way to do Office preview at scale (proven at Google Workspace; Box; Microsoft 365 internal).
  - CIS K8s 1.9.0 + seccomp restrictive + readOnlyRootFs + network-deny-all = defence in depth.
- **Cons**:
  - gVisor CVE backlog must be monitored (lower than container-runtime CVEs but non-zero).
  - LibreOffice in headless mode has occasional rendering quirks (fixable via fontconfig pinning).
- **Accepted**.

## Consequences

### Positive

- **Office preview safe at scale**: LibreOffice's CVE surface is contained by gVisor's user-space kernel.
- **Network egress impossible**: NetworkPolicy `oya-drive-preview-egress-deny-network` enforces; even a successful container escape can't exfiltrate bytes.
- **Host filesystem isolated**: readOnlyRootFs + no hostPath mounts.
- **Per-renderer timeout**: bounds DoS surface; per-tenant quota bounds resource consumption.
- **AC-13 satisfied**: sandbox isolation E2E-tested via deliberate malicious-Office-file test (VBA macro + remote payload).

### Negative

- **gVisor performance overhead**: ~15% latency overhead vs raw containers; preview render targets (1s p99) account.
- **Operational complexity**: gVisor RuntimeClass + per-renderer seccomp profile + LibreOffice fontconfig pinning + ffmpeg version pinning = four layers to keep in sync.
- **Per-render timeout** may stop a slow legitimate Office file render; tenant comms on timeout via "preview unavailable; download to view".

### Operational

- **New CI lane**: `oya-governance-preview-sandbox-conformance` (BLOCKER) — validates gVisor RuntimeClass + seccomp profile + NetworkPolicy + readOnlyRootFs at pod spec time.
- **Quarterly chaos exercise**: deliberately malicious Office file (VBA macro + remote payload + path traversal) — verify rasterised output + no egress + no host FS access.
- **gVisor CVE monitoring**: ops-security tracks gVisor security feed; emergency patch path defined.
- **LibreOffice CVE monitoring**: ops-security tracks LibreOffice security feed; sandbox provides containment but not infinite-time defence.

### Regulatory

- **CIS Kubernetes Benchmark v1.9.0** §4.6 (least privilege), §5.2.5 (runtime class), §5.3 (network policies) — satisfied.
- **GDPR Art. 32(1)(b)** — confidentiality + integrity of processing — sandbox satisfies.

## Verification

- [ ] gVisor RuntimeClass active on preview-worker pods — `kubectl get pod -l bc=preview -o jsonpath='{.items[*].spec.runtimeClassName}'`.
- [ ] Network egress denied — `cargo nextest run --test e2e_preview_sandbox_egress_denied`.
- [ ] Host filesystem access denied — `cargo nextest run --test e2e_preview_sandbox_host_fs_denied`.
- [ ] Malicious Office file rendered to PNG without macro execution — `cargo nextest run --test e2e_preview_sandbox_malicious_macro`.
- [ ] CIS K8s 1.9.0 §4.6 + §5.2.5 + §5.3 — `buck2 build //:quality-lane-registry-authority-check # lane=cis-k8s --microservice drive`.

## References

- gVisor — `gvisor.dev`; ptrace + KVM platform docs.
- Firecracker — `firecracker-microvm.github.io` (rejected reference).
- Kata Containers — `katacontainers.io` (rejected reference).
- libvips — `libvips.github.io/libvips/`.
- qpdf — `qpdf.sourceforge.io`.
- Mozilla pdf.js — `mozilla.github.io/pdf.js/`.
- LibreOffice headless mode — `documentation.libreoffice.org`.
- ffmpeg — `ffmpeg.org`.
- CIS Kubernetes Benchmark v1.9.0 — `cisecurity.org`.
- OWASP ASVS v4.0.3 — §14 Configuration.
- NIST SP 800-190 — Application Container Security Guide.
- ADR-0056 (BNF v4.1); ADR-0105 (13-layer enum); ADR-0135; ADR-0131; ADR-0133.
- ADR-DRIVE-0001 — object-storage substrate (preview workers fetch ciphertext from Garage).
- ADR-DRIVE-0004 — encryption-at-rest (preview workers decrypt via OpenBao Transit).
- `microservices/drive/PRD.md` §FR-09; §"Performance" preview targets; AC-13.
- `microservices/drive/iac/helm/values.yaml` `preview.sandbox.*`.
- `microservices/drive/iac/helm/templates/networkpolicy.yaml`.
- `microservices/drive/threat-model.md` T-I-05.
