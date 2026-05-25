# Jenkins SLSA/cosign/SBOM evidence — 2026-05-25T18:34:40Z
Proves the oyaCiLane supply-chain stages produce REAL, verifiable evidence (ADR-0361).
image: localhost:5001/ci/rust-agent@sha256:ae598b8dfe66 (the O5 agent image)
- SBOM (syft CycloneDX): 19932 components -> sbom.cdx.json.gz
- Trivy image scan: {'HIGH': 233, 'CRITICAL': 17}
  FINDING: stock rust:1-bookworm base carries 250 HIGH/CRITICAL -> harden O5 to slim/distroless base.
- cosign sign (by digest): OK
- cosign attest SLSA provenance (slsa-provenance.predicate.json) + CycloneDX SBOM: OK
- cosign verify-attestation: 'signatures verified against the specified public key' (tlog claim verified offline)
=> SLSA-L3 signing/provenance re-grounds in the Jenkins pipeline (measured), not a .github/workflows YAML claim.
