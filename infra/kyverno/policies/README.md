# Kyverno image-signature policies

This directory contains the local admission policy for the Oyatie in-cluster
registry image signing chain.

## Bring-up sequence

1. Mint the cosign keypair in OpenBao first at
   `sref://openbao/oya/ci/cosign-key`.
   - Private key property: `cosign-key`
   - Optional key password property: `password`
   - Public key property: `cosign.pub`
2. Apply `infra/kyverno/policies/verify-image-signed.yaml` so External Secrets
   Operator projects:
   - `oya-ci/cosign-key` for BuildKit signing Jobs
   - `oya-ci/cosign-pub` for Kyverno verification
3. Re-run the BuildKit Jobs for:
   - `registry.oya-registry.svc.cluster.local:5000/ci-webhook-gateway:dev`
   - `registry.oya-registry.svc.cluster.local:5000/rust-ci:dev`
   - `registry.oya-registry.svc.cluster.local:5000/llm-gateway:dev`
4. After all three images have signatures, keep the Kyverno
   `verify-oya-registry-images-signed` `ClusterPolicy` in `Enforce` mode.

Do not apply the enforcing policy before the OpenBao key exists and the first
signed image builds have completed, or new Pods using the local registry will be
rejected by admission.
