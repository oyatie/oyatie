---
id: ADR-0181
status: Accepted
deciders: council-architecture, ops-sre-reliability, axis-supply-chain-security
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: []
amended_by: [ADR-0349]
related: [ADR-0039, ADR-0041, ADR-0114, ADR-0124, ADR-0146, ADR-0148, ADR-0160]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/gitops-vcs-replacement.json
renumber_note: "Originally allocated ADR-0175 in PR #143 Fix-L round 2; renumbered to ADR-0181 after a multi-stage rebump because ADR-0175-0178 were concurrently allocated by Fix-J / Fix-K agents."
---

# ADR-0181 — Container image promotion pipeline: dev → staging → prod (cosign-signed tier promotion)

## Status

Accepted (2026-05-18). Authored as part of PR #143 Fix-L anti-hyperscaler pattern audit round 2.

## Context

ADR-0039 (supply-chain-security) requires every container image to ship with a Cosign signature, an SBOM, and Trivy scan evidence. ADR-0041 (gitops-trunk-based) declares the code promotion ladder. ADR-0124 (own-merge-queue) handles code change admission. ADR-0146 (distroless-nonroot) pins the base image.

None of these declares the **image promotion ladder**: how does a container image authored on `dev` reach `staging` and then `production` with explicit per-tier signature evidence? Without an explicit ladder, three failure modes occur:

1. **Image bypass** — a developer-tagged image (e.g., `:dev-abc123`) is referenced from a production Helm chart by accident; the prod cluster pulls a never-staged image.
2. **Tier impersonation** — an attacker who compromises the dev image registry path can write images that look promotable; without per-tier Cosign signing the prod cluster has no way to refuse.
3. **Audit gap** — the SOC2 / KISA audit trail for "what shipped to production" has no signed-evidence chain.

Hyperscaler precedents:

- **AWS** — ECR cross-account image promotion via Cosign signatures; cluster ECR pull policy restricts to signed-by-prod-tier identity.
- **GCP** — Artifact Registry promotion ladder + Binary Authorization with Attestor certificates; GKE refuses unsigned images.
- **Google internal (Borg / GKE Enterprise)** — promotion graph with per-tier attestor identities; image cannot land on prod fleet without staging attestor signing.
- **Stripe** — internal canary-image promotion via signed-tag swap.

## Decision

Oyatie declares a **three-tier container image promotion ladder**: `dev` → `staging` → `production`. Each tier has a distinct Cosign signing identity (Sigstore Fulcio OIDC-bound). Each cluster's pull policy restricts pulls to images carrying the appropriate-tier signature.

### Promotion ladder

```
                  dev signer      staging signer       prod signer
                  (OIDC: dev)     (OIDC: staging)      (OIDC: prod)
                       │                │                    │
   git tag rc-X ───────┴─► dev tag ────┴─► staging tag ─────┴─► prod tag
                       (build CI)       (promote CI)        (promote CI + manual gate)
```

1. **Tag schema** — `oyatie.dev/<ms>:<sha>-dev`, `oyatie.dev/<ms>:<sha>-staging`, `oyatie.dev/<ms>:<sha>-prod`. SHA is the upstream git commit SHA; tier suffix marks the promotion frontier.
2. **Cosign keyless signing** — each tier signs with a tier-bound Fulcio OIDC identity (no long-lived secrets). The OIDC identity is the GitHub Actions or Foundry-pipeline tier-bound workflow.
3. **Promotion gates**:
   - **dev → staging.** Trivy scan green + SBOM attached + ADR-0114 canary observability rollback green on the staging environment for 24h.
   - **staging → prod.** Trivy delta-scan green + ADR-0160 progressive-delivery-flagger green on canary cohort + ADR-0114 metric-gated-rollback green on production cohort for 6h.
4. **Cluster pull policy enforcement (Kyverno / Cilium L7)** — each cluster's admission policy (per ADR-0117 Kyverno consolidation) refuses pods referencing images NOT signed by the cluster's tier-bound Cosign verifier.
   - dev cluster accepts `*-dev` OR `*-staging` (staging falls forward to dev for rehearsal).
   - staging cluster accepts `*-staging` ONLY.
   - production cluster accepts `*-prod` ONLY.
5. **Re-tag without re-build** — promotion copies the image bytes from tier-N registry path to tier-N+1 path WITHOUT a rebuild; the bytes are identical, only the signature evidence accumulates. This is the AWS ECR cross-account pattern.
6. **Promotion audit-chain integration** — every promotion emits an audit-chain seal (per ADR-0145 Invariant 1) at the calling tier; the seal is the canonical "this image promoted to this tier at this time by this identity" evidence.

### Per-µservice manifest declaration

```json
"image_promotion": {
  "promotion_ladder": "dev-staging-prod",
  "staging_soak_hours": 24,
  "prod_canary_soak_hours": 6,
  "promotion_audit_seal_required": true
}
```

### Gate enforcement

`oya-check-image-promotion-discipline` lane validates:

- Every µservice's Helm chart references images via tier-bound tags.
- Every cluster's Kyverno ClusterPolicy includes the tier-bound Cosign verifier.
- Promotion CI workflow signs with the tier-bound Fulcio identity.
- Gate runs DEFERRED initially; STRICT mode lands after the per-cluster Kyverno policies ship.

## Alternatives considered

### A. Single registry path with `latest`-style tags (status quo)
- **Pros:** simplest.
- **Cons:** no per-tier signature evidence; matches no hyperscaler practice; audit-gap.
- **Rejected.**

### B. Three-tier ladder with re-build at each tier
- **Pros:** explicit per-tier provenance.
- **Cons:** re-build introduces drift (build env may differ across tiers); image bytes differ across tiers; defeats the "identical bytes through the ladder" invariant; supply-chain audit becomes harder.
- **Rejected:** matches no hyperscaler practice (AWS / GCP both copy bytes).

### C. Three-tier ladder with byte-copy + per-tier Cosign signature (accepted)
- **Pros:** matches AWS ECR cross-account + GCP Binary Authorization; identical bytes; signature evidence accumulates per tier; auditable.
- **Cons:** authoring cost (3 CI workflows; per-cluster Kyverno policies).
- **Accepted.**

### D. Notary v1 / DCT (Docker Content Trust)
- **Pros:** Docker-native.
- **Cons:** Notary v1 deprecated; Sigstore/Cosign is the CNCF-incubating canonical successor (ADR-0039 already names Cosign).
- **Rejected.**

### E. Tier-bound long-lived signing keys (vs OIDC keyless)
- **Pros:** simpler to operate.
- **Cons:** long-lived secrets violate ADR-0043 OpenBao "no long-lived signing material" discipline.
- **Rejected.**

## Consequences

### Positive

1. **Production cluster only pulls promoted images** — bypass is impossible by construction (Kyverno admission refuses).
2. **Per-tier Cosign signature is an audit-chain primitive** — SOC2 / KISA evidence for "what shipped where" is canonical.
3. **Identical image bytes through the ladder** — supply-chain provenance is exact (Trivy / SBOM evidence travels with the image).
4. **Sigstore Fulcio keyless** — no long-lived signing material; aligns with ADR-0043.
5. **Audit-chain seal at every promotion** — operator history of every promotion is canonical evidence.

### Negative

1. **Three CI workflows per µservice** — build (with dev signing), promote-to-staging, promote-to-prod. One-time authoring cost; canonical workflow template lives in `.github/workflows/`.
2. **Per-cluster Kyverno policy** — three per-environment ClusterPolicies for Cosign verifier-tier binding.
3. **Promotion latency** — dev → prod minimum 30h (24h staging soak + 6h prod canary soak). For hotfixes, a dedicated `expedited` promotion path with elevated review skips the staging soak; documented in `docs/standards/expedited-promotion.md`.

### Operational

1. ALL µservices declare `manifest.json#image_promotion`.
2. Per-tier Cosign Fulcio OIDC identities declared in `iac/cluster-policy/cosign-verifiers/<tier>.yaml`.
3. Per-cluster Kyverno ClusterPolicy enforces the tier-bound Cosign verifier (per ADR-0117).
4. CI workflow template `.github/workflows/image-promote.yaml.template` ships the canonical three-tier workflow.
5. `oya-check-image-promotion-discipline` gate authored; DEFERRED mode initially.
6. The Foundry pipeline's webhook-driven µservice promotion (per ADR-0112) integrates the audit-chain seal at the promotion-frontier of every tier hop.

## References

- AWS ECR cross-account image promotion + Cosign signature pattern.
- GCP Binary Authorization + Artifact Registry promotion ladder.
- Google internal (Borg / GKE Enterprise) promotion graph reference.
- Sigstore / Cosign / Fulcio — https://www.sigstore.dev
- CNCF Notary v2 (Sigstore alignment) — Notary v1 retired.
- ADR-0039 supply-chain security (Trivy + Cosign + SBOM + signed commits).
- ADR-0041 gitops-trunk-based-and-release-branch-cut-at-tag.
- ADR-0043 secrets-management-openbao-and-hsm-per-cell.
- ADR-0114 canary-observability-rollback.
- ADR-0117 repo-hygiene-gitignore-audit-config-and-kyverno-consolidation.
- ADR-0124 own-merge-queue-webhook-driven.
- ADR-0146 container-base-image-distroless-nonroot.
- ADR-0148 service-mesh-cilium.
- ADR-0160 progressive-delivery-flagger.
