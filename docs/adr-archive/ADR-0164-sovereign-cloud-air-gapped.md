---
id: ADR-0164
status: Superseded
deciders: council-architecture, axis-cloud-secrets, axis-cloud-k8s, axis-tenancy, axis-foundry, axis-audit-chain, council-legal
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-709]
related: [ADR-0009, ADR-0043, ADR-0049, ADR-0121, ADR-0143, ADR-0145, ADR-0158, ADR-0161, ADR-0162]
related_specs:
  - /specs/sovereign-cloud-air-gapped-canonical.json
  - /specs/per-microservice-flat-layout.json
---

# ADR-0164 — Sovereign Cloud / Air-Gapped Deployment (per-pack variant; on-prem registry / Bao / audit-chain shard / no external egress)

## Status

Accepted (2026-05-18). Establishes the canonical air-gapped deployment model for sovereign packs. Per-pack variant flips an "air-gap" overlay across the µservice fleet to satisfy KSA / EU sovereign / KR FSC / UAE / public-sector / fintech tenants whose regulators forbid external network egress, external container registries, external secret stores, external audit storage, and external LLM provider calls.

## Context

ADR-0049 fixed residency. ADR-0009 fixed cells. ADR-0121 chose kubeadm + containerd + Istio + Envoy as the onprem K8s baseline. None named the **air-gap variant** that some sovereign tenants require.

The regulator landscape:

- **AWS GovCloud** — physically + logically separated AWS region, US-citizen ops only, FedRAMP High, ITAR-compliant. No data leaves GovCloud.
- **Azure Government / Azure Government Secret / Azure Government Top Secret** — analogous; IL5 / IL6 boundaries.
- **Google Cloud for Government** — Assured Workloads, US-citizen ops, FedRAMP High.
- **KR FSC 「전자금융감독규정」 §15** — financial cloud must operate within KR; no external egress for financial-PII workloads.
- **KSA NCA** (National Cybersecurity Authority) **ECC-1** — sovereign-tenant data must stay within KSA; air-gapped variants required for sensitive ministries.
- **EU GDPR + Schrems II** — sovereign EU tenants prefer EU-only operation (no US transfer).
- **UAE TDRA + NESA** — sensitive sectors require on-shore.

Without an air-gap variant:

- A KSA-public-sector tenant cannot adopt oyatie because `foundry-providers` calls Anthropic/OpenAI/Google APIs (external egress).
- A KR-fintech tenant cannot adopt oyatie because the cloud-secrets µservice depends on the cloud KMS (external egress).
- A EU-sovereign tenant cannot adopt oyatie if the container registry is in a US region.

ADR-0164 introduces the air-gap overlay: per-pack flip that swaps each external dependency for an in-cell equivalent.

## Decision

Each sovereign pack declares `air_gap: true|false` in its pack manifest. When true, the following overlay applies:

### (a) On-prem container registry

- Each cell deploys Harbor 2.x (CNCF graduated) as its in-cell container registry.
- Image pull policy: `imagePullPolicy: IfNotPresent` + image references are rewritten to `registry.{cell}.svc.cluster.local/oya/<ms>:<tag>`.
- A pre-flight job (per-cell) mirrors images from the external build registry (where the global build pipeline pushes) to the per-cell Harbor BEFORE the cell loses external egress. After air-gap activation, no external pull occurs.
- Image signatures (Sigstore Cosign per ADR-0146 / SLSA L3) verified at pull time by the in-cell Harbor + Kyverno admission controller.

### (b) On-prem secret store

- The `cloud-secrets` µservice runs **OpenBao** (the BSL-free Vault fork) in-cell with HSM-backed seal key (per ADR-0043).
- No external KMS dependency. The cloud KMS code path is replaced by the OpenBao Transit secrets-engine.
- Per-tenant secret keys live in the in-cell HSM partition only.

### (c) On-prem audit-chain shard

- Per ADR-0162, sovereign tenants get dedicated audit-chain shards. In an air-gap pack, the shard's storage + sealing keys are in-cell. The fleet-wide root anchor (normally published to a global trust-portal datastore) is published to an in-region public chain in air-gap variants (or simply held in-cell with quarterly external publication via offline media if the regulator permits).

### (d) No external API egress

- All `foundry-providers` external LLM calls (Anthropic, OpenAI, Google Gemini) are **forbidden** in air-gap mode. Egress NetworkPolicy + Cilium L7 egress policy denies; Istio `ServiceEntry` for these external hosts is absent.
- The foundry µservice falls back to on-prem LLMs:
  - **vLLM** serving Llama 3.x / DeepSeek / Qwen / Mistral OSS models on the cell's GPU pool.
  - **Ollama** for smaller models / dev tier.
  - Per-pack model selection in `microservices/foundry/iac/kustomize/components/pack-{ksa,kr-fsc,...}/values.yaml`.
- Other external calls (DNS, NTP, OCSP, certificate-authority CRL) route through in-cell proxies (CoreDNS for DNS; chrony in-cell for NTP; in-cell PKI for cert validation).

### (e) On-prem observability + telemetry endpoints

- No external telemetry (no Datadog, no Honeycomb SaaS, no New Relic).
- All metrics + traces + logs go to the in-cell observability µservice (Prometheus/Mimir + Tempo + Loki backing).

### (f) On-prem CI runner option

- Sovereign tenants may require that even the CI build artifacts come from in-region runners. A per-pack overlay points the µservice deploy pipeline at in-region GitHub Actions self-hosted runners (or in-region GitLab runners) running in a separate "build cell" with the same air-gap shape.

### (g) Documentation + compliance evidence

- Each air-gap pack ships a per-pack compliance attestation document at `microservices/governance/catalog/pack-{name}-air-gap-attestation.md` listing the regulator alignment (FedRAMP High, KR FSC, KSA NCA ECC-1, etc.) + the per-control mapping.

### Pack matrix at GA

| Pack | air_gap | Regulator | On-prem LLM choice |
|---|---|---|---|
| `pack-us-shared` | false | — | external (Anthropic/OpenAI default) |
| `pack-eu` (sovereign) | false (per ADR-0049 strict EU residency) | GDPR + EHDS | external EU-region only |
| `pack-eu-sovereign-airgap` | true | German BSI C5 / French SecNumCloud / Italian ACN | vLLM Llama 3 + Mistral |
| `pack-kr` (general) | false | PIPA | external KR-region only |
| `pack-kr-fsc` (KR fintech) | true | KR FSC 전자금융감독규정 | vLLM HyperCLOVA-X (Naver Cloud) + Llama 3 |
| `pack-kr-public` | true | 전자정부법 | vLLM Llama 3 |
| `pack-jp` | false | APPI | external JP-region only |
| `pack-ksa` (sovereign) | true | NCA ECC-1 | vLLM Falcon (G42) + Llama 3 |
| `pack-uae` | true | NESA / TDRA | vLLM Falcon + Llama 3 |
| `pack-us-gov` | true | FedRAMP High / ITAR | vLLM Llama 3 |

## Alternatives considered

### Alternative A — No air-gap support; refuse sovereign tenants

- **Pros:** zero overlay complexity.
- **Cons:** entire markets (KSA, UAE, KR-fintech, KR-public, EU-BSI-C5, US-Gov) unreachable; ADR-0049 already commits to per-pack residency, so refusing air-gap is an inconsistency.
- **Rejected because:** sovereign markets are core (per [Bominal-inheritance precedence] feedback).

### Alternative B — One global "air-gap mode" toggle (not per-pack)

- **Pros:** simpler mental model.
- **Cons:** different sovereign packs have different air-gap requirements (KSA requires on-prem LLM but accepts in-region observability SaaS; US-Gov requires both on-prem; KR-fintech is intermediate). A single toggle is too coarse.
- **Rejected because:** the regulator-by-regulator variation forces per-pack overlay.

### Alternative C — Per-pack air-gap variant overlay (this ADR)

- **Pros:** correct granularity; each pack flips the dependencies it needs; standard Kubernetes overlay mechanism (kustomize); CI-validatable per pack.
- **Cons:** per-pack overlay maintenance; per-pack compliance evidence.
- **Accepted.**

### Alternative D — Commercial sovereign-cloud reseller partnership (AWS GovCloud / Azure Government as the substrate)

- **Pros:** zero in-house sovereign substrate; reseller carries the regulator burden.
- **Cons:** AWS GovCloud is US-only; no analog for KSA / UAE / KR; partnership economics scale unfavorably; the ADR-0121 portability invariant requires our own substrate.
- **Rejected because:** doesn't cover the markets we need; portability invariant.

### Alternative E — Hardware-attestation per workload (confidential computing only; no air-gap)

- **Pros:** AMD SEV-SNP / Intel TDX provide cryptographic isolation; could argue "we don't need air-gap if compute is attested".
- **Cons:** regulators do not yet accept confidential-compute attestation as equivalent to physical air-gap for sovereign workloads (NCA / FedRAMP-High posture). Hardware attestation is *additional* defense not *substitute*.
- **Rejected because:** regulator acceptance gap; we adopt confidential compute (per ADR-0147 sandboxing ladder) as additional defense WITHIN the air-gap variant.

## Consequences

### Positive

1. **Sovereign markets become reachable.** KSA / UAE / KR-fintech / KR-public / EU-sovereign / US-Gov all addressable.
2. **Per-pack overlay is the natural Kubernetes pattern.** Kustomize components; per-pack values; no µservice fork required.
3. **CI gate per-pack.** A pack marked `air_gap: true` is validated at build time: no external egress allowed; image references rewritten; OpenBao + on-prem LLM + on-prem audit pinned.
4. **Compliance evidence rolled up per pack.** SOC 2 / ISO 27001 / FedRAMP / KR FSC / NCA ECC-1 / NESA / BSI C5 attestations are per-pack documents.
5. **Foundry on-prem LLM path is durable.** vLLM + open-weight models are the production path for sovereign packs; aligns with the ADR-0026 in-house AI substrate roadmap.

### Negative

1. **Operational cost per air-gap pack.** Harbor + OpenBao + vLLM + per-cell observability stack are real ops.
2. **Image-mirroring choreography.** Build pipeline → external Harbor → mirror-job → per-cell Harbor → cell K8s. Each step is a potential failure mode.
3. **On-prem LLM quality gap.** Open-weight models are competitive but not always at parity with frontier APIs. Sovereign-pack tenants accept this gap explicitly at onboarding.
4. **Air-gap pack onboarding is slow.** Each new sovereign pack requires regulator engagement + compliance document authoring + per-cell validation drill.
5. **Cross-pack support burden.** Engineering must support both external-egress and air-gap code paths in every µservice that touches an external dependency.

### Operational

1. Per-pack overlay declared in `microservices/cloud-k8s/iac/kustomize/components/air-gap-{ksa,kr-fsc,...}/`.
2. CI lane `cloud-ci/Rust gate packet air-gap-overlay` enforces (a) air-gap packs reference no external host in any `ServiceEntry` / `NetworkPolicy egress`, (b) image references are rewritten to in-cell Harbor, (c) `foundry-providers` external LLM clients are absent from air-gap pack image build.
3. `microservices/cloud-k8s/PRD.md` and `microservices/cloud-secrets/PRD.md` updated with air-gap variant section (Companion).
4. Per-pack compliance attestation document at `microservices/governance/catalog/pack-{name}-air-gap-attestation.md`.
5. Pre-flight image mirror job templated at `microservices/cloud-iac/iac/helm/harbor-mirror/`.
6. vLLM Helm chart at `microservices/foundry/iac/helm/vllm/` with per-pack model selection.

## References

- AWS GovCloud architecture — https://aws.amazon.com/govcloud-us/
- Azure Government / Azure Government Secret / Top Secret — https://azure.microsoft.com/en-us/explore/global-infrastructure/government/
- Google Cloud for Government — https://cloud.google.com/security/compliance/government
- FedRAMP High baseline — https://www.fedramp.gov/
- KR FSC 「전자금융감독규정」 §15.
- KSA NCA ECC-1 (Essential Cybersecurity Controls).
- UAE TDRA + NESA Information Assurance Standards.
- German BSI C5 cloud attestation.
- French SecNumCloud / ANSSI.
- Italian ACN cloud cybersecurity framework.
- Harbor (CNCF graduated container registry) — https://goharbor.io/
- OpenBao (BSL-free Vault fork) — https://openbao.org/
- vLLM — https://docs.vllm.ai/
- ADR-0009 — cell architecture.
- ADR-0026 — in-house AI model substrate roadmap.
- ADR-0043 — HSM + KMS.
- ADR-0049 — residency (per-pack).
- ADR-0121 — onprem K8s stack (portability invariant).
- ADR-0143 — foundry per-BC release pointer.
- ADR-0145 — inter-µservice communication reform (egress invariant).
- ADR-0146 — distroless container base (Sigstore verification in-cell).
- ADR-0147 — sandboxing runtime ladder.
- ADR-0158 — multi-region disposition (sovereign-pin).
- ADR-0161 — CSI storage class canonical (per-pack matrix).
- ADR-0162 — per-tenant audit-chain slicing (sovereign shard).
