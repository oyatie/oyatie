---
id: ADR-SDK-0001
title: "Developer SDK signing keys stay inside OpenBao transit"
status: Proposed
date: 2026-05-18
microservice: developer-sdk
related_oyatie_adrs:
  - ADR-0131
  - ADR-0173
  - ADR-0213
  - ADR-0243
  - ADR-0244
  - ADR-0258
  - ADR-0263
decision_owner: axis-ecosystem + ops-security
---

# ADR-SDK-0001: Developer SDK signing keys stay inside OpenBao transit

## Context

- The developer-sdk microservice signs SDK release manifests, sandbox tenant bootstrap tokens, portal API tokens, payout adapter callbacks, and webhook test fixtures.
- The named pressure is `developer-distribution-trust-boundary`: a compromised SDK signing key can turn every downstream developer workstation into a supply-chain infection path.
- The prior incident class is `sdk-key-exported-to-ci-runner`: an earlier batch treated signing as a library call and left the private-key boundary ambiguous.
- The second prior incident class is `fixture-token-accepted-as-production`: generated developer fixtures were not cryptographically distinguishable from production release material.
- The third prior incident class is `openbao-transit-bypassed-by-local-ed25519`: code snippets implied that `ed25519_dalek::SigningKey` could live in process memory.
- ADR-0213 makes developer-sdk part of the Ecosystem-as-a-Service substrate, so developer-facing trust material is not a local app detail.
- ADR-0173 requires stack ownership and vendor-lock-in avoidance; the signing substrate must be portable across OCI, self-hosted, and sovereign cells.
- ADR-0243 requires every authorization and activation gate to be Cedar-evaluated; signing is a policy-governed action, not a helper routine.
- ADR-0244 requires tenant scoping even for sandbox and developer principals.
- ADR-0258 requires public API and SDK artifacts to have explicit versioning and deprecation contracts.
- ADR-0263 requires every signing attempt to produce structured telemetry, audit events, and trace correlation.
- Developer SDK artifacts cross trust boundaries more often than internal binaries because customers paste SDK bootstrap commands into CI.
- The artifact must be verifiable offline by developers without asking oyatie for a network round trip.
- The private key must never leave the HSM/OpenBao perimeter, including test, sandbox, and local development.
- The platform must support Ed25519 because RFC 8032 signatures are compact, deterministic, and widely supported by modern package ecosystems.
- The platform must still be able to rotate signing keys without breaking already released SDKs.
- The release pipeline must distinguish human-approved production releases from generated sandbox fixtures.
- The signing path must support per-cell deployment because sovereign packs cannot route signing calls across pack boundaries.
- The signing path must fail closed on OpenBao unavailability for production releases.
- The signing path may degrade to unsigned dry-run manifests only in explicitly labelled sandbox mode.
- The signing path must preserve key-use evidence for at least 7 years for SOC 2 CC6.1, ISO 27001 A.8.24, and supply-chain attestations.
- The developer portal must expose verification metadata so external developers can independently confirm artifact provenance.
- The key hierarchy must permit a future PQC hybrid signature without changing the external manifest shape.

## Decision

- We choose `OpenBao 2.0 transit engine with Ed25519 keys` as the only production signing substrate for developer-sdk.
- The named pattern is `Vault Transit signing without key export`, using OpenBao as the self-hostable implementation.
- The release signing key path is `transit/keys/developer-sdk/release-ed25519-v1`.
- The sandbox signing key path is `transit/keys/developer-sdk/sandbox-ed25519-v1`.
- The fixture signing key path is `transit/keys/developer-sdk/fixture-ed25519-v1`.
- Each path is scoped to the cell and pack: `transit/keys/{cell_id}/{pack_id}/developer-sdk/{purpose}-ed25519-v1`.
- Private keys are generated inside OpenBao.
- Private keys are non-exportable.
- Any code path that constructs an in-process `SigningKey` for production material is forbidden.
- The SDK manifest signer calls `POST /v1/transit/sign/{key}` with prehashed canonical bytes.
- The digest algorithm is SHA-512/256 over JSON Canonicalization Scheme bytes.
- The signature algorithm is Ed25519 per RFC 8032.
- Release manifests use JCS canonical JSON per RFC 8785.
- Release manifests include `artifact_id`, `semver`, `api_version`, `git_commit`, `slsa_predicate_ref`, `cell_id`, `pack_id`, `expires_at`, and `signature_ref`.
- Production manifests have `expires_at` no later than 398 days from signing.
- Sandbox manifests have `expires_at` no later than 30 days from signing.
- Fixture manifests have `expires_at` no later than 24 hours from signing.
- Production release signing requires Cedar action `developer-sdk.release.sign`.
- Sandbox signing requires Cedar action `developer-sdk.sandbox.sign`.
- Fixture signing requires Cedar action `developer-sdk.fixture.sign`.
- Production release signing requires two approvals: `axis-ecosystem.release-manager` and `ops-security.signing-officer`.
- Sandbox signing requires one approval from `axis-ecosystem.developer-platform-engineer`.
- Fixture signing requires the CI principal plus a linked non-production change id.
- Key rotation cadence is 180 days for production release keys.
- Key rotation cadence is 90 days for sandbox and fixture keys.
- Emergency key revocation target is 15 minutes from Sev-1 declaration to Cedar-denied new signing.
- Public verification keys are published at `GET /v1/sdk/signing-keys`.
- The signing-key endpoint returns only public keys and metadata.
- The endpoint is versioned with request-time API version pinning per ADR-0258.
- Every signature emits audit event `DeveloperSdkArtifactSigned`.
- Every denied signature emits audit event `DeveloperSdkSigningDenied`.
- Every key rotation emits audit event `DeveloperSdkSigningKeyRotated`.
- The p95 signing latency target is 75 ms inside one cell.
- The p99 signing latency target is 150 ms inside one cell.
- The monthly failed-signing budget is 0.05 percent of signing attempts.
- A production release cannot be promoted when signing audit coverage falls below 100 percent.

## Alternatives Considered

### Local Ed25519 key in release worker

- Pro: release worker can sign while OpenBao is unavailable.
- Pro: implementation is a small Rust library call.
- Pro: local unit tests are easy.
- Con: private key enters process memory.
- Con: CI runner compromise becomes signing-key compromise.
- Con: key rotation depends on redeploying every worker.
- Con: sovereign-cell key isolation is easy to bypass.
- Tradeoff: lower operational dependency but unacceptable supply-chain blast radius.
- Rejected because it repeats the `openbao-transit-bypassed-by-local-ed25519` incident class.

### Cloud KMS asymmetric signing only

- Pro: managed HSM-backed key custody.
- Pro: vendor SLAs are clear.
- Pro: audit logs are mature.
- Con: ADR-0173 rejects hard vendor dependence for the canonical substrate.
- Con: sovereign and self-hosted cells cannot assume the same vendor.
- Con: Cloud KMS Ed25519 availability differs by provider.
- Con: migration would leak provider semantics into SDK manifest verification.
- Tradeoff: operational convenience but weaker portability.
- Rejected as canonical; allowed as OpenBao HSM/autounseal backing where pack policy permits.

### Developer-managed signing keys

- Pro: developers can sign offline.
- Pro: oyatie does not hold developer-specific trust material.
- Pro: no platform signing throughput bottleneck.
- Con: revocation is fragmented across developer machines.
- Con: customers cannot distinguish official SDK artifacts from local experiments.
- Con: downstream supply-chain attestations lose a single trust root.
- Con: support cannot reproduce signature failures.
- Tradeoff: autonomy but no platform-grade provenance.
- Rejected for official SDK artifacts; developer-owned keys remain acceptable for local plugins outside this ADR.

### Sigstore Fulcio ephemeral certificate only

- Pro: modern supply-chain pattern.
- Pro: integrates with SLSA provenance.
- Pro: short-lived certificates reduce key lifetime.
- Con: external transparency log and CA dependency adds bootstrap complexity.
- Con: sovereign cells may not allow public Rekor publication.
- Con: Fulcio certs do not replace the need for a stable offline SDK verification key.
- Tradeoff: excellent release evidence but not enough as the only developer-sdk signature root.
- Partially accepted: Cosign/Sigstore attestations accompany OpenBao signatures.

## Consequences

- Positive: production private keys never leave OpenBao transit.
- Positive: SDK consumers get one stable verification story across all cells.
- Positive: release evidence is traceable through audit-chain, OpenTelemetry traces, and SLSA provenance.
- Positive: sandbox and fixture signatures are cryptographically distinct from production signatures.
- Positive: key rotation is centrally enforceable without redeploying signing workers.
- Positive: sovereign cells can run the same signing model without relying on a single cloud vendor.
- Negative: OpenBao becomes load-bearing for release signing.
- Negative: release promotion is blocked during OpenBao outages.
- Negative: developers lose the convenience of local production signing.
- Negative: test fixtures need explicit non-production keys.
- Neutral: Sigstore remains part of the release evidence bundle but not the root signing primitive.
- Neutral: a future PQC hybrid signature can add a second signature field without replacing Ed25519 immediately.
- Follow-up work: implement `SDK-IP-001-transit-signer-port`.
- Follow-up work: add `developer-sdk-signing-key-rotation` runbook.
- Follow-up work: add a public `/.well-known/oyatie-sdk-signing-keys.json` projection.
- Follow-up work: add `oya-governance-sdk-signature-boundary` CI lane.

## Implementation Notes

- Data shape `SdkReleaseManifestV1` is canonical JSON.
- Field `manifest_version` is a SemVer string and starts at `1.0.0`.
- Field `artifact_id` is a ULID prefixed by `sdk_art_`.
- Field `artifact_kind` is one of `rust_crate`, `npm_package`, `swift_package`, `kotlin_maven`, `python_wheel`, or `go_module`.
- Field `sdk_semver` follows SemVer 2.0.0.
- Field `api_version` follows ADR-0258 request-time pin format, for example `2026-05-18`.
- Field `source_git_commit` is a 40-character Git SHA-1 or future hash algorithm descriptor.
- Field `source_tree_hash` stores the VCS tree hash used for reproducible codegen.
- Field `slsa_predicate_ref` points to the signed SLSA predicate.
- Field `cell_id` is the issuing cell.
- Field `pack_id` is the compliance pack namespace.
- Field `signature_key_id` is the OpenBao transit key version.
- Field `signature_algorithm` is `Ed25519-RFC8032`.
- Field `signature` is base64url without padding.
- Field `signature_created_at` is RFC 3339.
- Field `expires_at` is RFC 3339.
- API endpoint `POST /v1/sdk/releases/{release_id}/sign` signs production manifests.
- API endpoint `POST /v1/sdk/sandboxes/{sandbox_id}/sign-bootstrap-token` signs sandbox bootstrap tokens.
- API endpoint `POST /v1/sdk/fixtures/{fixture_id}/sign` signs non-production fixtures.
- API endpoint `GET /v1/sdk/signing-keys` publishes active and retained public keys.
- API endpoint `GET /.well-known/oyatie-sdk-signing-keys.json` mirrors public verification keys.
- Cedar principal for production is `Oyatie::Principal::Service("developer-sdk.release-worker")`.
- Cedar principal for sandbox is `Oyatie::Principal::Service("developer-sdk.sandbox-worker")`.
- Cedar principal for fixture signing is `Oyatie::Principal::Service("developer-sdk.ci-fixture-worker")`.
- Cedar action `developer-sdk.release.sign` applies to resource `DeveloperSdk::ReleaseManifest`.
- Cedar action `developer-sdk.sandbox.sign` applies to resource `DeveloperSdk::SandboxToken`.
- Cedar action `developer-sdk.fixture.sign` applies to resource `DeveloperSdk::FixtureManifest`.
- Cedar context field `change_id` must reference an admitted Oya VCS changeset.
- Cedar context field `approval_count` must be at least 2 for production.
- Cedar context field `environment` must equal `production` for production signing.
- Cedar context field `key_purpose` must match the OpenBao key path purpose.
- Cedar context field `cell_id` must equal the caller cell.
- Cedar context field `pack_id` must equal the artifact pack.
- Example production permit: principal `developer-sdk.release-worker`, action `developer-sdk.release.sign`, resource `DeveloperSdk::ReleaseManifest::"sdk_art_01HX..."`, context `{environment:"production", approval_count:2, cell_id:"cell-us-001", pack_id:"core-enterprise"}`.
- Example sandbox permit: principal `developer-sdk.sandbox-worker`, action `developer-sdk.sandbox.sign`, resource `DeveloperSdk::SandboxToken::"sandbox_01HX..."`, context `{environment:"sandbox", approval_count:1, ttl_seconds:2592000}`.
- Example fixture forbid: principal `developer-sdk.ci-fixture-worker`, action `developer-sdk.release.sign`, resource `DeveloperSdk::ReleaseManifest`, context `{environment:"ci"}`.
- OpenBao policy `developer-sdk-release-signer` permits `update` only on `transit/sign/*/developer-sdk/release-ed25519-v1`.
- OpenBao policy `developer-sdk-release-signer` denies `read` on private key export paths.
- OpenBao policy `developer-sdk-key-rotator` permits `update` on `transit/keys/*/developer-sdk/*/rotate`.
- Signing worker uses SPIFFE ID `spiffe://oyatie.dev/ns/developer-sdk/sa/release-worker`.
- Signing worker runs in Kubernetes with HTTP/3 ingress disabled; it is internal mesh only.
- Mesh transport uses mTLS with TLS 1.3 floor.
- OpenTelemetry span name is `developer_sdk.sign_artifact`.
- Span attribute `sdk.artifact_id` carries the artifact id.
- Span attribute `sdk.signing_key_version` carries the OpenBao key version.
- Metric `oya_developer_sdk_signing_latency_ms` records histogram buckets `25,50,75,100,150,250,500`.
- Metric `oya_developer_sdk_signing_denied_total` is counter by `action`, `reason`, `cell_id`, and `pack_id`.
- Metric cardinality budget is 500 active series per cell.
- Dashboard `developer-sdk-signing-trust.json` displays latency, denials, rotations, and stale public-key clients.
- SLO `developer-sdk-signing-latency.openslo.yaml` sets p95 <= 75 ms and p99 <= 150 ms.
- SLO `developer-sdk-signing-availability.openslo.yaml` sets monthly availability >= 99.95 percent for sandbox and fixture signing.
- Production release signing has a stricter effective availability: failure blocks release, not runtime traffic.
- Failure mode `openbao_unavailable` blocks production release and returns HTTP 503 with `Retry-After`.
- Failure mode `cedar_denied` returns HTTP 403 and emits `DeveloperSdkSigningDenied`.
- Failure mode `key_version_stale` returns HTTP 409 and instructs caller to refresh public-key metadata.
- Failure mode `manifest_not_canonical` returns HTTP 422 and includes JCS diff hint.
- Failure mode `cell_mismatch` returns HTTP 403 and pages ops-security on repeated attempts.
- Rollback path for bad release signature is to revoke manifest, mark artifact `yanked`, and publish replacement manifest under a new artifact id.
- Rollback path for key compromise is emergency Cedar forbid, OpenBao key disable, public revocation list publish, and package-registry yanks.

## Verification

- Test `sdk_signing_private_key_never_exported` asserts no OpenBao export capability exists in release-worker policy.
- Test `sdk_release_manifest_jcs_canonical_roundtrip` signs identical input twice and verifies identical canonical digest.
- Test `sdk_release_signing_requires_two_approvals` verifies Cedar denies one-approval production context.
- Test `sdk_fixture_cannot_use_release_key` verifies CI fixture principal cannot access production action.
- Test `sdk_sandbox_token_ttl_limit` verifies sandbox signatures above 30 days are denied.
- Test `sdk_signature_public_key_endpoint_verifies_release` verifies external verifier can validate an artifact offline.
- Test `sdk_rotation_keeps_old_public_key_for_retained_artifacts` verifies old public keys remain visible until last signed artifact expires plus 7 years.
- Test `sdk_openbao_outage_blocks_production_release` injects OpenBao 503 and expects release promotion refusal.
- Test `sdk_signing_audit_coverage_is_complete` verifies every successful signing has audit event and trace id.
- Metric `oya_developer_sdk_signing_latency_ms` must meet p95 <= 75 ms in cell-local load test.
- Metric `oya_developer_sdk_signing_denied_total{reason="cell_mismatch"}` pages at any non-zero value in 5 minutes.
- Metric `oya_developer_sdk_public_key_stale_client_total` warns when old verifier metadata exceeds 24 hours.
- Dashboard `developer-sdk-signing-trust.json` must include panels for latency, OpenBao errors, Cedar denials, rotations, and key age.
- Dashboard `supply-chain-release-integrity.json` must link SDK signing events to SLSA predicate ids.
- CI check `oya-governance-sdk-signature-boundary` greps for in-process Ed25519 private-key construction under production paths.
- CI check `oya-governance-openbao-policy-no-export` validates OpenBao policies deny private-key export.
- CI check `oya-governance-cedar-action-coverage --microservice developer-sdk` verifies all signing endpoints map to Cedar actions.
- CI check `sdk-release-verifier` downloads the public key endpoint and verifies a canonical manifest.
- CI check `sdk-manifest-jcs` rejects non-canonical JSON.
- CI check `oya-governance-observability-emission --microservice developer-sdk` verifies ADR-0263 audit, metric, and trace fields.
- Load test signs 10,000 sandbox manifests in one cell and requires p99 <= 150 ms.
- Chaos test disables OpenBao standby and confirms active node failover produces no failed production signatures.
- Chaos test disables all OpenBao nodes and confirms fail-closed behavior.
- Security review inspects `transit/keys/*/developer-sdk/*` policy paths before promotion.
- Release readiness gate refuses promotion when public verification key endpoint omits the active key version.

## References

- ADR-0131: Per-microservice flat layout.
- ADR-0173: Vendor lock-in avoidance and stack ownership.
- ADR-0213: Ecosystem-as-a-Service architecture.
- ADR-0243: Cedar as Universal Gate.
- ADR-0244: Tenant as universal scoping primitive.
- ADR-0258: API versioning model.
- ADR-0263: Observability emission contract.
- RFC 8032: Edwards-Curve Digital Signature Algorithm, Ed25519.
- RFC 8785: JSON Canonicalization Scheme.
- NIST SP 800-57 Part 1 Rev. 5: Recommendation for Key Management.
- SLSA v1.0 provenance specification.
- Sigstore Cosign documentation.
- OpenBao transit secrets engine documentation.
- HashiCorp Vault transit secrets engine design notes as industry precedent.
- CNCF SPIFFE/SPIRE workload identity documentation.
- SOC 2 CC6.1 and CC7.2 trust service criteria.
- ISO/IEC 27001:2022 Annex A.8.24 cryptographic key management.
- in-toto attestation framework.
- The Update Framework specification, v1.0.31.
