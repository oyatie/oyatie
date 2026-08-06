---
id: ADR-0147
status: Superseded
deciders: council-architecture, ops-security, ops-sre-reliability, axis-cell-substrate, axis-meet, axis-docs, axis-translate, axis-drive, axis-social, axis-shorts, axis-anonymous, axis-network, axis-notes, axis-slides
date: 2026-05-18
amended_date: 2026-05-18
owner: ops-security
supersedes: []
superseded_by: [ADR-709]
related: [ADR-0064, ADR-0117, ADR-0131, ADR-0133, ADR-0139, ADR-0140 (retired per ADR-0145), ADR-0144]
related_memory: [feedback_quality_performance_scalability_bar, feedback_canonical_base_localization, feedback_repeat_mistake_prevention, feedback_no_silent_regression]
related_specs:
  - /specs/hyperscaler-gates.json
  - /specs/iac-canonical-base.json
purpose: |
  Replace the universal `runtimeClassName: gvisor` hard-coded pattern with a
  workload-class-tiered container sandboxing runtime ladder, matching how
  AWS / Google / Microsoft / Cloudflare actually pick runtimes per workload
  class. Eliminates the gvisor-overprescribes-and-underprotects regret.
  Amended 2026-05-18 to designate Cloud Hypervisor (Kata kata-clh handler) as
  the primary untrusted-content runtime in place of gVisor; gVisor retained
  as opt-in for cold-start-sensitive workloads.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0147: Container sandboxing runtime ladder

## Status

Amended — 2026-05-18 (original Accepted 2026-05-18). The amendment refines
the decision; it does not supersede.

## Date

2026-05-18.

## Context

Multiple per-µservice Helm charts have committed to gVisor as the universal
sandboxing answer:

- `microservices/docs/iac/helm/values.yaml` — `gvisor.runtimeClassName: gvisor`
  (export-import: Pandoc + WeasyPrint + Chromium-headless)
- `microservices/slides/iac/helm/values.yaml` — `exportPool.gvisor.runtimeClassName: gvisor`
  (LibreOffice + Chromium PDF/PPTX/MP4 export pool)
- `microservices/meet/iac/helm/meet/values.yaml` — `transcriptionWorker.runtimeClassName: gvisor`
  (Whisper transcription); `recordingMuxWorker.runtimeClassName: gvisor` (ffmpeg mux)
- `microservices/translate/iac/helm/translate/values.yaml` —
  `documentLocalizationWorker.runtimeClassName: gvisor` (Pandoc + LibreOffice)
- `microservices/notes/iac/helm/notes/values.yaml` —
  `importExportWorker.runtimeClassName: gvisor`
- `microservices/drive/iac/helm/templates/deployment.yaml` —
  inline `runtimeClassName: gvisor` for the preview BC
- `microservices/social/iac/helm/social/templates/deployment.yaml` —
  inline `runtimeClassName: gvisor` for the media-transcode-worker
- `microservices/community/iac/helm/community/templates/deployment.yaml` —
  inline `runtimeClassName: gvisor` for the attachment/media-preview worker

This is two compounded mistakes:

1. **Overprescription for app-tier workloads.** Rust REST/gRPC services
   (channel-store-rest, meeting-room-rest, search, etc.) do not run
   untrusted content; gVisor's user-space-kernel emulation adds 10-50%
   CPU + I/O penalty for zero marginal security gain when the workload
   is first-party trusted code.

2. **Underprotection for cryptographic workers.** Blind-signature
   ceremony nodes, KMS-bound workers, and signing oracles get
   user-space-kernel isolation when they need full-VM blast-radius
   guarantees. gVisor's syscall filter leaves the host kernel reachable
   on gofer-routed syscalls and shares the host's clock and RNG
   semantically; a Spectre-class side-channel or a syscall-filter
   bypass against gVisor's userspace kernel is materially cheaper than
   a Kata-Containers full-VM hypervisor escape.

Hyperscaler practice mirrors this nuance, not universal gVisor:

| Hyperscaler        | Untrusted content        | Crypto / per-request isolation         | App-tier                            |
|--------------------|--------------------------|-----------------------------------------|-------------------------------------|
| AWS                | Firecracker (Fargate)    | Firecracker (Lambda per-invocation)     | Bottlerocket + bare runc            |
| Google             | gVisor (Cloud Run)       | Kata or per-tenant VM (Confidential GKE)| GKE Autopilot bare runc             |
| Microsoft (Azure)  | Kata + Hyper-V (AKS)     | Confidential Containers (AMD SEV-SNP)   | AKS bare runc                       |
| Cloudflare         | V8 isolates (Workers)    | gVisor (Workers Unbound)                | n/a (no general-purpose tier)       |
| Anthropic (public) | Per-job VM isolation     | HSM + per-job VM                        | Bare runc                           |

The pattern is consistent: hyperscalers pick runtime per workload class
and per blast-radius requirement, never universally.

## Decision

oyatie adopts a **workload-class-tiered container sandboxing runtime
ladder**. The canonical mapping below replaces the universal-gVisor
default:

| Workload class                                         | Default runtime                                                  | Sovereign-tenant override                 |
|--------------------------------------------------------|------------------------------------------------------------------|-------------------------------------------|
| App-tier µservices (Rust REST/gRPC, WS gateways)       | none — bare Linux + CIS K8s restricted profile                   | n/a (restricted profile suffices)         |
| Untrusted-content transcoders/renderers                | Kata Containers + Cloud Hypervisor (`kata-clh`)                  | `kata-clh-sev-snp` (AMD SEV-SNP CC)       |
| Cryptographic workers (blind-sig, signing oracles)     | Kata Containers + Cloud Hypervisor + AMD SEV-SNP (`kata-clh-sev-snp`) | Bare HSM (FIPS 140-3 Level 3)         |
| AI inference (Whisper, ML models)                      | Kata Containers + Cloud Hypervisor (`kata-clh`) on CPU; `kata-clh-tdx` for GPU CC | `kata-clh-sev-snp` / `kata-clh-tdx` |
| Federation gateway (untrusted-internet egress)         | Kata Containers + Cloud Hypervisor (`kata-clh`) + restrictive egress NetworkPolicy | `kata-clh-sev-snp` for highest-risk |
| WASM-only workers                                      | runwasi (WebAssembly runtime)                                    | n/a                                       |
| Per-request ephemeral isolation                        | Kata-Firecracker (`kata-fc`)                                     | n/a                                       |
| Cold-start-sensitive workers (opt-in legacy)           | gVisor (`runsc`) — retained as opt-in escape hatch               | n/a                                       |

### Canonical Helm helpers

`microservices/governance/iac/helm/_oya-helpers/templates/_helpers.tpl`
exposes the ladder as five named helpers. Per-µservice charts MUST
invoke one of these helpers rather than emit `runtimeClassName:`
inline:

```yaml
spec:
  template:
    spec:
      {{- include "oya.runtimeClassName.untrustedContent" $ | nindent 6 }}
```

Helpers:

- `oya.runtimeClassName.appTier` — emits NOTHING. App-tier µservices
  run on bare Linux + CIS restricted profile; explicit `runtimeClassName`
  is the wrong answer.
- `oya.runtimeClassName.untrustedContent` — Kata + Cloud Hypervisor
  (`kata-clh`) default; `kata-clh-sev-snp` (AMD SEV-SNP confidential
  compute) for sovereign tenant tier.
- `oya.runtimeClassName.crypto` — Kata + Cloud Hypervisor +
  AMD SEV-SNP (`kata-clh-sev-snp`) for sovereign tier; full-VM
  isolation with cryptographic memory protection. FIPS 140-3 Level 3
  tenants further upgrade to bare HSM.
- `oya.runtimeClassName.aiInference` — Kata + Cloud Hypervisor
  (`kata-clh`) on CPU; `kata-clh-tdx` (Intel TDX) variant for
  GPU-passthrough confidential compute.
- `oya.runtimeClassName.federationGateway` — Kata + Cloud Hypervisor
  (`kata-clh`) + restrictive egress NetworkPolicy; `kata-clh-sev-snp`
  for highest-risk sovereign packs.
- `oya.runtimeClassName.gvisor` — DEPRECATED legacy compatibility
  helper. Retained only for cold-start-sensitive µservices that
  explicitly opt in; new uses MUST select one of the five workload-
  class helpers above.

### Cluster-side RuntimeClass installation

`microservices/governance/iac/kustomize/components/runtime-classes/`
provides the canonical RuntimeClass set installed in every cluster:

- `kata-clh` (handler: `kata-clh`) — primary untrusted-content +
  AI-inference (CPU) + federation-gateway runtime; Kata Containers
  with Cloud Hypervisor VMM.
- `kata-clh-sev-snp` (handler: `kata-clh-sev-snp`) — AMD SEV-SNP
  confidential compute for crypto + sovereign-tier overrides.
- `kata-clh-tdx` (handler: `kata-clh-tdx`) — Intel TDX confidential
  compute (typically paired with GPU passthrough for AI inference).
- `gvisor` (handler: `runsc`) — legacy opt-in for cold-start-
  sensitive workloads.
- `kata-qemu` (handler: `kata-qemu`) — legacy; new charts should
  prefer `kata-clh` (Cloud Hypervisor) for the same isolation
  posture at lower overhead.
- `kata-fc` (handler: `kata-fc`, for per-request isolation).
- `wasmtime` (handler: `wasmtime`, for WASM workers).

The cloud-k8s µservice composes this component into its base, which
guarantees every cluster reconciles a uniform set of RuntimeClass
objects before any µservice that references them can schedule pods.

### Cell scheduling awareness

Per the ADR-0333 successor contract (`microservices/tenancy/ARCHITECTURE.md#cell-assignment` plus `microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning`) §Cell scheduling +
runtime-affinity), cells must declare which RuntimeClass handlers are
provisioned on their host pool. Cells supporting `kata-qemu` use a
host-pool with nested-virt-capable nodes; cells supporting `gvisor`
use a vanilla host-pool. Tenant placement honours runtime affinity:
a tenant requiring Kata cannot be assigned to a gVisor-only cell.

## Amendment 2026-05-18: Cloud Hypervisor as primary

User directive 2026-05-18 ("switch to cloud hypervisor") replaces gVisor
with Kata Containers + Cloud Hypervisor (`kata-clh` handler) as the
primary untrusted-content / AI-inference (CPU) / federation-gateway
runtime. gVisor is retained as an opt-in escape hatch for cold-start-
sensitive workloads only.

### Rationale

1. **Rust-primary alignment.** Cloud Hypervisor is a Rust VMM
   (gVisor is Go); this matches oyatie's Rust-primary directive
   and reduces the foreign-language operational surface.
2. **Native confidential compute.** Cloud Hypervisor supports
   AMD SEV-SNP and Intel TDX, enabling sovereign-tenant
   cryptographic memory isolation without architectural redesign.
   gVisor provides no equivalent.
3. **Stronger isolation posture.** Cloud Hypervisor runs a full
   microVM via KVM — a sandbox escape requires a hypervisor +
   KVM bug, materially more expensive than a gVisor userspace-
   kernel bug.
4. **Lower steady-state overhead.** Cloud Hypervisor adds ~5-15%
   overhead for compute-bound workloads versus gVisor's 10-50%
   penalty. Whisper transcription, ffmpeg muxing, Pandoc/WeasyPrint
   rendering, LibreOffice export, and Chromium-headless PDF/PPTX
   are all CPU-bound and benefit directly.
5. **Confidential-compute path for FIPS 140-3 Level 3 sovereign
   tenants.** AMD SEV-SNP / Intel TDX support means sovereign-tier
   tenants gain cryptographic memory isolation without leaving the
   canonical helper surface.
6. **Production adoption.** Microsoft Azure Boost runs on Cloud
   Hypervisor; CNCF Sandbox project since 2023; Linux Foundation
   governance.
7. **gVisor cold-start advantage is irrelevant here.** No workload
   in the current ladder (Whisper, ffmpeg, Pandoc, LibreOffice,
   Chromium-headless) is cold-start sensitive — they all run as
   long-running pods. The 50 ms gVisor cold start versus Cloud
   Hypervisor's 125-250 ms cold start does not affect any
   measured SLO.

### Alternatives reconsidered

| Dimension                | gVisor (original primary)                                | Cloud Hypervisor (new primary)               |
|--------------------------|----------------------------------------------------------|----------------------------------------------|
| Implementation language  | Go                                                       | Rust (aligns with Rust-primary directive)    |
| Architecture             | userspace kernel                                         | full microVM via KVM                         |
| Isolation strength       | medium (userspace-kernel emulation)                      | strong (hardware-backed microVM)             |
| Steady-state overhead    | 10-50%                                                   | 5-15%                                        |
| Cold start               | ~50 ms                                                   | ~125-250 ms                                  |
| Confidential compute     | none                                                     | AMD SEV-SNP + Intel TDX                      |
| Notable production users | Google Cloud Run, Cloudflare Workers Unbound             | Microsoft Azure Boost, CNCF Sandbox          |

### Helper + RuntimeClass impact

- `oya.runtimeClassName.untrustedContent` → `kata-clh` (was `gvisor`);
  sovereign-tier override → `kata-clh-sev-snp` (was `kata-qemu`).
- `oya.runtimeClassName.crypto` → `kata-clh-sev-snp` for sovereign tier
  (was `kata-qemu`); FIPS 140-3 Level 3 still routes to bare HSM.
- `oya.runtimeClassName.aiInference` → `kata-clh` (was `gvisor`);
  confidential-compute variant `kata-clh-tdx` (Intel TDX) for
  GPU-passthrough.
- `oya.runtimeClassName.federationGateway` → `kata-clh` + restrictive
  egress NetworkPolicy (was `gvisor`); sovereign tier → `kata-clh-sev-snp`.
- `oya.runtimeClassName.appTier` — unchanged (still emits nothing).
- `oya.runtimeClassName.gvisor` — DEPRECATED legacy compat; retained
  only for cold-start-sensitive opt-in.

Three new RuntimeClass manifests land under
`microservices/governance/iac/kustomize/components/runtime-classes/`:
`kata-clh-runtime-class.yaml`, `kata-clh-sev-snp-runtime-class.yaml`,
`kata-clh-tdx-runtime-class.yaml`. Existing `gvisor`, `kata-qemu`,
`kata-fc`, `wasmtime` manifests remain (the first two for legacy
compatibility; the latter two retain their original workload classes).

## Alternatives considered

### Alternative (a): Universal gVisor

- **Pros:** One RuntimeClass to install; one mental model; cheap.
- **Cons:** App-tier µservices pay 10-50% CPU + I/O penalty for
  zero security gain; crypto workers get user-space-kernel
  isolation instead of full-VM isolation — blast-radius argument
  inverted; gVisor's clock + RNG sharing semantically weakens
  side-channel hardening for crypto.
- **Rejected because:** Over-prescribes for app-tier; under-
  protects for crypto. Both regrets material.

### Alternative (b): Universal Kata Containers

- **Pros:** Strong isolation everywhere; full-VM blast radius;
  matches AWS Firecracker philosophy on per-pod isolation.
- **Cons:** Cold-start budget (~250 ms) blocks the sub-100 ms
  pod-readiness SLO targets for app-tier; nested-virt host
  requirement increases per-node cost; HPA scale-out latency
  doubles vs runc.
- **Rejected because:** Cold-start budget incompatible with
  app-tier SLO bands; cost regression unjustifiable for
  non-untrusted-content workloads.

### Alternative (c): Universal Firecracker

- **Pros:** AWS-proven microVM pattern for serverless; strong
  per-pod isolation; <125 ms boot.
- **Cons:** AWS-specific tooling; per-pod cold-start still
  unsuitable for long-lived app-tier; runtime ecosystem
  outside AWS still maturing in 2026.
- **Rejected because:** Hyperscaler-portability regression;
  long-running pods do not benefit from microVM cold-start
  optimisation.

### Alternative (d): Universal bare Linux + LSM (AppArmor/SELinux)

- **Pros:** Zero runtime overhead; CIS-aligned posture.
- **Cons:** Insufficient for untrusted-content workers (Pandoc,
  LibreOffice, Chromium, ffmpeg, ImageMagick all have CVE
  history); insufficient for crypto blast-radius; insufficient
  for AI inference of third-party models.
- **Rejected because:** Cannot accommodate the workload classes
  that motivated sandboxing in the first place.

### Alternative (e): Workload-class ladder (this ADR)

- **Pros:** Matches AWS / Google / Microsoft / Cloudflare per-
  workload practice; app-tier avoids gVisor overhead; crypto
  gets full-VM blast radius; untrusted-content gets gVisor by
  default with Kata available for sovereign tenants; per-tenant
  override built in.
- **Cons:** Multiple RuntimeClass objects per cluster;
  operational complexity grows; cell-scheduler must honour
  runtime-affinity.
- **Accepted because:** Hyperscaler-aligned; honest about
  blast-radius differences; doesn't impose universal
  performance penalty; admits crypto needs stronger isolation
  than gVisor provides.

## Consequences

### Positive

1. **App-tier µservices avoid gVisor overhead.** Channel-store-rest,
   meeting-room-rest, search, notes core, etc. drop the 10-50% CPU + I/O
   gVisor penalty. CIS K8s restricted profile + Pod Security Standards
   restricted is sufficient for first-party Rust services.
2. **Cryptographic workers get stronger isolation.** Blind-signature
   ceremony nodes (anonymous µservice), KMS-bound signers, and
   signing oracles move from gVisor user-space-kernel to Kata full-VM
   isolation. Spectre-class side-channel hardening is materially
   stronger; syscall-filter bypasses don't reach the host kernel.
3. **Per-tenant runtime override enables sovereign-tenant tier
   upgrades.** A sovereign-tier tenant (KSA-government, EU-defense)
   can elect Kata over gVisor for untrusted-content workers via a
   pack-overlay-level values override, without architectural redesign.
4. **Hyperscaler-portability preserved.** The ladder reads naturally
   into AWS EKS (Firecracker option), GKE (gVisor option), AKS (Kata
   option). No hyperscaler-specific lock-in.
5. **CI-enforceable.** A follow-on lane can assert every Deployment
   either includes one of the five canonical helpers or has an
   explicit appTier exemption note.

### Negative

1. **Operational complexity grows.** Three RuntimeClass objects
   (`gvisor`, `kata-qemu`, `kata-fc`) plus optionally `wasmtime` must
   be installed and maintained on every cluster. CNI / CSI / device-
   plugin interactions are RuntimeClass-specific. Mitigation: the
   `microservices/governance/iac/kustomize/components/runtime-classes/`
   component standardises installation across clusters.
2. **CIS Kubernetes Benchmark validation per-RuntimeClass.** The
   benchmark applies CIS-1.10 controls per host-pool; Kata host-pools
   must additionally pass the Kata-specific validation set.
   Mitigation: tracked under the cloud-k8s µservice operations area.
3. **Cell scheduler runtime-affinity awareness.** Cells must declare
   which RuntimeClass handlers their host-pool provisions; placement
   honours runtime-affinity. Mitigation: cell PRD adds a brief
   runtime-affinity subsection (this ADR, deliverable 5).
4. **Migration cost.** Nine existing hard-coded `runtimeClassName:
   gvisor` references migrate to canonical helpers. Mitigation:
   refactor delivered alongside this ADR (deliverable 3).

### Comparisons to industry-standard practice

- **AWS Firecracker (Lambda/Fargate):** the design paper
  (Agache et al., NSDI 2020) is explicit that microVM isolation is
  selected per workload class — not as a universal default. Direct
  precedent for the ladder model.
- **Google gVisor (Cloud Run + Sandbox v2):** the gVisor security model
  paper (Young et al., 2019) acknowledges gVisor is unsuitable for
  crypto and per-request isolation; Google itself pairs gVisor with
  per-tenant VM isolation on Confidential GKE.
- **Microsoft Kata Containers + Confidential Computing on AKS:**
  Microsoft's AKS Kata documentation explicitly recommends Kata for
  crypto + multi-tenant high-trust workloads; runc for app-tier.
- **Cloudflare Workers Unbound:** Cloudflare documents the V8-isolate
  → gVisor → microVM ladder per use-case in its security blog.
- **Anthropic public statements:** training/inference container
  isolation uses per-job VM boundaries for untrusted code, not
  universal user-space-kernel emulation.
- **CIS Kubernetes Benchmark v1.10 (RuntimeClass guidance):** validates
  RuntimeClass usage per workload sensitivity, not as a universal
  default.
- **NIST SP 800-190 (Application Container Security Guide):** §4.3
  recommends runtime isolation matched to workload risk profile.

## References

- AWS Firecracker NSDI 2020 paper —
  https://www.usenix.org/conference/nsdi20/presentation/agache
- AWS Bottlerocket announcement (re:Invent 2019).
- Google gVisor security model paper (2019); gVisor handbook
  https://gvisor.dev/docs/.
- Cloud Hypervisor project (Linux Foundation hosted, CNCF Sandbox 2023)
  https://www.cloudhypervisor.org/.
- Kata Containers + Cloud Hypervisor integration docs
  https://github.com/kata-containers/kata-containers/blob/main/docs/how-to/how-to-use-kata-containers-with-cloud-hypervisor.md.
- AMD SEV-SNP specification ("AMD SEV-SNP: Strengthening VM Isolation
  with Integrity Protection and More") —
  https://www.amd.com/system/files/TechDocs/SEV-SNP-strengthening-vm-isolation-with-integrity-protection-and-more.pdf.
- Intel TDX (Trust Domain Extensions) specification —
  https://www.intel.com/content/www/us/en/developer/articles/technical/intel-trust-domain-extensions.html.
- Microsoft Azure Boost announcement (Ignite 2023) —
  https://techcommunity.microsoft.com/t5/azure-compute-blog/microsoft-azure-boost/ba-p/3955462.
- Microsoft Kata Containers + Confidential Computing on AKS
  https://learn.microsoft.com/en-us/azure/confidential-computing/.
- Cloudflare Workers Unbound + V8-isolate security model
  https://blog.cloudflare.com/workers-security/.
- Anthropic safety practices public statements (2024).
- CIS Kubernetes Benchmark v1.10.
- NIST SP 800-190 Application Container Security Guide.
- ADR-0064 — canonical-base-and-localization-packs.
- ADR-0117 — cloud-native infrastructure (residency).
- ADR-0131 — per-microservice flat layout.
- ADR-0133 — industry-best-practice + hyperscaler conformance.
- ADR-0139 — agentic SLO-gated promotion.
- ADR-0140 — Cedar policy enforcement substrate +
  cross-cutting-carriers exemption.
- ADR-0144 — EU AI Act graduated risk tier model.
- `microservices/governance/iac/helm/_oya-helpers/templates/_helpers.tpl`
  (canonical helper library).
- `microservices/governance/iac/kustomize/components/runtime-classes/`
  (canonical RuntimeClass install component).
- `microservices/tenancy/ARCHITECTURE.md#cell-assignment` and `microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning` (cell scheduling + runtime-affinity
  subsection).
