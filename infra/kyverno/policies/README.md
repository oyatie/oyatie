# Kyverno image-signature policies

This directory contains the local admission policy for the Oyatie in-cluster
registry keyless signing, provenance, and SBOM attestation chain.

## Bring-up sequence

1. Build and attest images from GitHub Actions or the successor owned CI runner
   with Sigstore keyless OIDC identity:
   - issuer: `https://token.actions.githubusercontent.com`
   - subject: `https://github.com/jason931225/oyatie/.github/workflows/.+@refs/(heads/dev|tags/v.+)`
   - Rekor transparency log: `https://rekor.sigstore.dev`
   - SLSA predicate type: `https://slsa.dev/provenance/v1`
   - SBOM predicate type: `https://cyclonedx.org/bom`
2. Apply `infra/kyverno/policies/verify-image-signed.yaml`.
3. Re-run the BuildKit Jobs for:
   - `registry.oya-registry.svc.cluster.local:5000/ci-webhook-gateway:dev`
   - `registry.oya-registry.svc.cluster.local:5000/rust-ci:dev`
   - `registry.oya-registry.svc.cluster.local:5000/cloud-intelligence:dev`
4. After all three images have keyless signatures, Rekor entries, SLSA
   provenance attestations, and CycloneDX SBOM attestations, keep the
   `verify-oya-registry-images-keyless` `ClusterPolicy` in `Enforce` mode.

Do not apply the enforcing policy before keyless signatures and attestations
exist for the local registry images, or new Pods using the local registry will be
rejected by admission.
