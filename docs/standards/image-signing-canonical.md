---
doc_class: Standard
title: Container Image Signing (Canonical)
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-18
owner_team: axis-supply-chain
deciders: axis-supply-chain, council-security, council-architecture
related_adrs: [ADR-0146, ADR-0039]
review_cadence: annually
doc_status: published
---

# Container Image Signing (Canonical)

## Authority

Every container image produced by oyatie CI MUST be signed with
[sigstore cosign](https://www.sigstore.dev/) and accompanied by an
[SLSA L3](https://slsa.dev/spec/v1.0/levels#build-l3) provenance
attestation. Every container image MUST also pass
[Trivy](https://aquasecurity.github.io/trivy/) scanning with zero
HIGH/CRITICAL findings before promotion past `dev`.

## Contract

### 1. Sigstore cosign signing

Every image built by `.github/workflows/cosign.yml` is signed via
keyless OIDC (GitHub Actions identity → Fulcio CA). Resulting
attestations land in the OCI registry alongside the image.

```yaml
- name: cosign sign
  run: |
    cosign sign --yes \
      --bundle "${IMG}@${DIGEST}.cosign.bundle" \
      "${IMG}@${DIGEST}"
```

### 2. SLSA L3 provenance

Provenance is emitted via `.github/workflows/slsa.yml` using the
`slsa-framework/slsa-github-generator` reusable workflow. Provenance
captures:

- builder.id (GitHub Actions workflow).
- buildType (the workflow file path).
- materials (every input git SHA).
- entryPoint (the canonical Cargo target).

### 3. Trivy scanning

Every image is scanned by Trivy before push. Promotion blocked when:

- Any CRITICAL CVE is present.
- Any HIGH CVE is present without a documented `.trivyignore`
  exception ADR.

### 4. Admission policy

The cluster admission controller (per
`microservices/cloud-k8s/iac/kustomize/components/cosign-policy/`)
REFUSES any image without a valid cosign signature traceable to the
oyatie GitHub Actions OIDC identity.

### 5. Validation

The `check-image-signing-discipline` gate enforces that:

- Every µservice that publishes a container image has a cosign
  signature reference in `microservices/<ms>/iac/helm/<chart>/values.yaml`.
- `.github/workflows/cosign.yml` exists, is runnable, and signs
  every image referenced under `microservices/*/iac/helm/*/values.yaml`.
- The cosign workflow targets sigstore Fulcio (not a private CA).
- A Trivy scan step exists in the publish workflow.

### 6. Existing supply-chain gates

This standard layers on top of the existing
`check-supply-chain` gate. The image-signing gate adds the
container-image axis; the supply-chain gate covers Cargo + npm
dependency provenance.

## References

- sigstore cosign — keyless signing flow.
- SLSA L3 — provenance generators.
- Trivy — image scanning.
- ADR-0146-container-base-image-distroless-nonroot.
- ADR-0039-supply-chain-evidence.
