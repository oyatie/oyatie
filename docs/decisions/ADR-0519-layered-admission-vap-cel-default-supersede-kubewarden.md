---
id: ADR-0519
status: Accepted
planning_impact: true
date: 2026-05-31
deciders:
  - founder
  - council-architecture
  - ops-security
  - axis-cloud-k8s
owner: council-architecture
supersedes:
  - ADR-0379
superseded_by: []
amends: []
related:
  - ADR-0379
  - ADR-0183
  - ADR-0023
  - ADR-0246
  - ADR-0181
  - ADR-0039
  - ADR-0514
  - ADR-0515
  - ADR-0148
  - ADR-0007
related_specs: [/specs/platform-architecture.json]
door: two-way
---
# ADR-0519 — Layered Kubernetes admission: in-tree VAP+CEL default, bespoke-Rust signing webhook (supersedes ADR-0379)

## Status
Accepted — 2026-05-31 (founder-locked, via best-practice-research + idea-refine consensus).
Supersedes ADR-0379 (Kubewarden as the default admission substrate). The Cedar-vs-admission
SEPARATION principle (ADR-0183 → ADR-0379) is carried forward UNCHANGED: Cedar owns
application-layer L7 authorization; a distinct mechanism owns Kubernetes resource admission.
Only the admission mechanism changes — from a single webhook engine (Kubewarden) to a layered
model defaulting to in-tree Kubernetes-native policy.

## Context
ADR-0379 made **Kubewarden** the default Kubernetes admission/policy substrate, justified by its
Rust→WASM policy model (dogfood alignment with ADR-0023 Wasmtime + Rust-everywhere). On
re-examination (2026-05-31), that decision has three gaps, confirmed by cited best-practice
research:

1. **It never evaluated in-tree Kubernetes-native admission.** ValidatingAdmissionPolicy (VAP)
   reached GA in **k8s 1.30** (2024-04-24) and MutatingAdmissionPolicy (MAP) reached GA in
   **k8s 1.36** (2026-04-22). Both use CEL and run **in-process in the API server** — zero
   external controller, zero vendor, zero managed-service dependency, lowest latency,
   upstream-maintained. Our cluster is **k8s v1.36.1**, so both are GA-native today.
2. **It picked the least-mature option.** As of 2026: **Kyverno is CNCF Graduated** (2026-03-24),
   **OPA Graduated** (2021); **Kubewarden is still CNCF Sandbox** (since 2022), single-vendor
   (SUSE/Rancher), with a markedly smaller community and adoption footprint. On the
   mature+trusted+well-maintained lens, Kubewarden is the weakest.
3. **It ignored the now-standard layered pattern.** The 2026 best practice — endorsed upstream
   and baked into both Gatekeeper and Kyverno (which now *generate* VAPs) — is: use in-tree
   VAP+CEL for the common/declarative cases, and escalate to a webhook engine ONLY for what CEL
   cannot express. GKE Policy Controller = Gatekeeper; AKS Azure Policy auto-generates VAPs.

Critically, **image signature verification (cosign) is architecturally impossible in pure CEL**
— the Kubernetes docs use exactly this as the canonical "needs a webhook" example, because it
requires an outbound registry fetch + cryptographic verification that in-process CEL cannot do.
That residue is the one genuine case for a webhook on this platform.

## Decision

### (1) Default admission substrate = in-tree VAP + MAP + CEL
Kubernetes-native **ValidatingAdmissionPolicy + MutatingAdmissionPolicy + CEL** is the DEFAULT
admission/policy substrate for all **declarative structural and mutation policy**: registry
allowlist, digest-pinning (no `:latest`), Pod Security Standards restricted-by-default, nonroot
UID 65532, label/annotation discipline, runtime-tier shape rules. No external controller, no
webhook to operate, fail-closed in-process. This is the strongest hyperscaler-lens fit (the
substrate is Kubernetes itself) and is GA on our cluster (v1.36.1).

### (2) Webhook tier — ONLY for the residue CEL cannot express
A webhook is used solely for policy that requires outbound network, external data, or
cryptography. The flagship (and currently only identified) case is **container image signature
verification (cosign)**.

### (3) Signature-verification destination = a bespoke-Rust cosign-verify admission webhook
The Tier-2 residue is served by a **bespoke-Rust validating admission webhook** that performs
**key-based cosign** verification (static public key; ctlog/Rekor lookup disabled — correct for a
self-hosted cluster with no public Fulcio/Rekor). It shares the cosign-verify code path with the
**bespoke OCI-registry + signer product** (the designated product surface). This is the dogfood
end-state for admission's residue; a bespoke admission *engine* is explicitly NOT built (that
would re-implement cert-rotation/HA/fail-modes/policy-CRDs — an anti-pattern). **Signing
enforcement is DEFERRED** (nothing enforces it today; it is not on the CI-gate-loop critical
path) and lands as a fast-follow when the registry+signer product matures.

### (4) Kubewarden demoted; Kyverno removed from base overlay; Cedar unchanged
- **Kubewarden** is DEMOTED from default to an **optional Rust/WASM escalation** — sanctioned
  only if a genuine CEL-can't-express, non-signature policy case appears. Its Sandbox-tier /
  single-vendor maturity is acknowledged.
- **Kyverno** is removed from the base platform overlay (ADR-0379 §2 already said adapter-only;
  the live cluster still running it is drift to clean up).
- **Cedar** is unchanged — universal application-layer (L7) authorization at the Istio Ambient
  `ext_authz` waypoint. Zero overlap with cluster admission.

## Rejected Alternatives
- **Keep Kubewarden as default (ADR-0379):** rejected — least-mature (Sandbox), single-vendor,
  and ignores the in-tree-first layered standard; the Rust/WASM benefit does not outweigh
  depending on a Sandbox engine for cases Kubernetes now handles natively.
- **Adopt Kyverno for the webhook tier:** considered (Graduated, broad adoption, generates VAPs,
  key-based cosign via `ImageValidatingPolicy`); rejected as the destination because the residue
  is narrow and ties into the bespoke OCI-registry+signer product. Remains a valid interim if
  speed ever trumps dogfood; not chosen.
- **sigstore policy-controller / Ratify / Connaisseur:** valid mature options for signing;
  rejected as destination for the same reason (bespoke-Rust shares product code).
- **Bespoke admission ENGINE in Rust:** rejected — anti-pattern; only a bespoke webhook for the
  narrow residue, never the primary substrate.
- **Keyless/Fulcio cosign:** rejected for self-hosted — requires Fulcio/Rekor/OIDC infra we do
  not host; key-based cosign verifies fully offline.

## Consequences
- **Positive:** default substrate is upstream Kubernetes itself (zero dependency, zero vendor,
  lowest latency, fail-closed in-process); matches the 2026 industry-standard layered pattern;
  removes dependency on a Sandbox-tier single-vendor engine for the common cases; the residue
  aligns with a product oya is already building.
- **Negative/cost:** the 8 canonical policies (ADR-0183's set) must be (re)authored as VAP/CEL;
  the bespoke-Rust signing webhook is owned code (cert rotation via cert-manager, HA, latency
  budget) — bounded, and deferred until needed.
- **Neutral:** Cedar's role and the admission-vs-app-authz separation are unchanged; ADR-0515's
  buck2-native OCI assembly + ADR-0039 cosign supply-chain are agnostic to the admission mechanism.

## Verification
- The base platform overlay deploys NO admission webhook engine as default; VAP/MAP policies are
  authored in-tree. The ADR index lists ADR-0519 Accepted and ADR-0379 Superseded-by ADR-0519.
- The 8 canonical policies exist as ValidatingAdmissionPolicy + bindings (CEL), confirmed on a
  k8s ≥1.30 cluster.
- When signing is enabled: a bespoke-Rust validating webhook enforces key-based cosign on
  `registry.oya-registry…:5000/*` images; verified end-to-end (unsigned image rejected, signed
  image admitted) with no public Fulcio/Rekor dependency.

## References
- ADR-0379 — Kubewarden as default admission substrate (**superseded here**).
- ADR-0183 — Cedar (app authz) vs admission engine separation (principle carried forward).
- ADR-0023 — Wasmtime server-side WASM sandbox (Kubewarden's alignment rationale; now optional).
- ADR-0246 — policy-engine-substrate-promotion (review for amendment).
- ADR-0181 — image promotion / signed-image admission (review for amendment).
- ADR-0039 — cosign / SBOM supply chain.
- ADR-0514 / ADR-0515 — buck2-native OCI build + image assembly (the registry+signer product track).
- ADR-0148 / ADR-0007 — Cilium+Istio Ambient ext_authz / Cedar.
- Evidence (2026-05-31, cited): kubernetes.io VAP GA (1.30) + MAP GA (1.36) + admission-webhook
  good-practices; cncf.io (Kyverno Graduated 2026-03-24, OPA Graduated 2021, Kubewarden Sandbox);
  Gatekeeper/Kyverno VAP-generation docs; GKE/AKS managed-policy docs.
- Founder decision 2026-05-31: layered admission, in-tree VAP+CEL+MAP default, bespoke-Rust
  cosign-verify webhook for the signature residue.
