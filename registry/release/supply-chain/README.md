# Release supply-chain evidence

This directory stores one YAML attestation record per digest-pinned release artifact.

Pre-release state is intentionally empty. `registry/release/images.yaml` declares that
empty scope explicitly so local pre-release checks do not carry fake image digests.
On a release tag, `oya gate validate release-supply-chain --phase release` requires
records with dual-SBOM, Trivy, Cosign/Rekor, provenance, audit-event, and zero open
HIGH/CRITICAL evidence for every artifact in the manifest.
