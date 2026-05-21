---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-openbao-secretreference-substrate
impl_plan_id: IP-002-secretreference-uri-spec
status: pending
execution_unit: ChangeSet
owner: axis-cloud-secrets + council-architecture
acceptance_lanes: [doc-coverage, contract-test]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: SecretReference URI specification

## Intent

Author the canonical SecretReference URI grammar (ABNF) + JSON-schema fragment + reference parser test corpus. This spec is the contract every SDK + LEAN-A11 lane consumes.

## ChangeSet boundary

Spec document at `specs/secret-reference-uri.json` + grammar at `microservices/cloud-secrets/contracts/secret-reference-uri.abnf` + 100-entry parser test corpus at `microservices/cloud-secrets/contracts/secret-reference-uri-test-corpus.jsonl`.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `specs/secret-reference-uri.json` | create | canonical spec (machine-readable) |
| `microservices/cloud-secrets/contracts/secret-reference-uri.abnf` | create | ABNF grammar |
| `microservices/cloud-secrets/contracts/secret-reference-uri-test-corpus.jsonl` | create | 100 entries: 50 valid, 50 invalid (each with rationale) |

## ABNF

```
SecretReferenceURI = "openbao:secret/" path [ "@" version ]
path               = tenant-segment "/" microservice "/" name
tenant-segment     = "tenant:" 16HEXDIG / "shared"
microservice       = 1*( ALPHA / DIGIT / "-" )
name               = 1*( ALPHA / DIGIT / "_" / "-" )
version            = "v" 1*DIGIT
```

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate doc-coverage --doc microservices/cloud-secrets/contracts/
cargo test -p oya-cloud-secrets-secret-reference-resolver-domain test_uri_parser_corpus
```

## Test Plan

Corpus must round-trip parse + reject every invalid entry; SDK parsers in Rust/TS/Python all consume same corpus.

## Halt Conditions

- Grammar change that breaks existing consumer references — require ADR + migration plan.

## Next IP

`IP-003-resolver-kernel.md`

## References

- `microservices/cloud-secrets/policy/secret-isolation.md` §"TI-03"
- `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`

## Wave 15-IP-substance A-G

### A. Problem
`cloud-secrets` needs one parseable SecretReference contract so product µservices can carry `${openbao:secret/<path>}` without ever copying raw values into repos, chat, images, telemetry, or checkpoints. The gap is not generic secret storage; it is a repo-wide typed reference grammar that lines up with the PRD's zero raw-secret exposure requirement, the LEAN-A11 blocker, and the OpenAPI/proto contracts.

### B. Approach
Define the URI grammar and test corpus as service-owned contract artifacts, then require every Rust/TS/Python parser and the resolver domain crate to consume the same corpus. The grammar stays narrower than AWS/GCP resource identifiers and Vault paths: only the OpenBao scheme, tenant/pack-safe path segments, optional version, and bounded query keys are valid.

### C. Deliverables
- `specs/secret-reference-uri.abnf`, `specs/secret-reference-uri.json`, and `specs/secret-reference-uri-test-corpus.jsonl`.
- Contract alignment with `contracts/openapi/cloud-secrets.yaml` and `contracts/proto/cloud-secrets.proto`.
- Parser implementation in `oya-cloud-secrets-secret-reference-resolver-domain`, already named by `manifest.json` and catalog.
- Policy evidence from `policy/secret-isolation.md` and `policy/tenant-scope.cedar`.
- SLO linkage to `slos/secret-resolve-latency.openslo.yaml`.

### D. Ordered Implementation Steps
1. Freeze the ABNF and JSON schema for `${openbao:secret/<path>}`.
2. Add valid/invalid fixtures for tenant paths, version pins, and forbidden traversal/logging shapes.
3. Bind the schema into OpenAPI, proto, and SDK docs so generated clients do not invent local variants.
4. Implement the resolver-domain parser and round-trip serializer.
5. Add Rust/TS/Python corpus tests that all consume the same fixture file.
6. Wire LEAN-A11 to flag raw-secret literals and allow only SecretReference strings.
7. Publish migration notes for existing config references.

### E. Acceptance
- `cargo test -p oya-cloud-secrets-secret-reference-resolver-domain test_uri_parser_corpus`.
- `cargo run -p oya-dev-cli -- gate validate doc-coverage --doc microservices/cloud-secrets/contracts/`.
- `cargo run -p oya-dev-cli -- gate validate lean-a11 --microservice cloud-secrets`.
- Every accepted reference carries a tenant-safe path and never serializes a raw value.
- `secret-resolve-latency` SLO remains bound to runtime resolve, not grammar parsing alone.

### F. Evidence
Primary evidence lives in `PRD.md` FR-01/FR-02, `ARCHITECTURE.md` SecretReference resolver sections, `manifest.json` crate and contract lists, `policy/secret-isolation.md`, `contracts/openapi/cloud-secrets.yaml`, and `contracts/proto/cloud-secrets.proto`.

### G. Counterpart Comparison
AWS Secrets Manager ARNs, Google Secret Manager resource names, and HashiCorp Vault paths all provide runtime lookup indirection, but the parity matrices show they do not enforce Oyatie's `Secret<T>` wrapper, cache TTL ceiling, no-log guarantee, or LEAN-A11 raw-secret blocker. This IP closes that counterpart gap by making the reference grammar itself the only allowed configuration surface.

Grep-recognized counterpart anchor: GitHub Actions Secrets is referenced only as a CI distribution counterpart that this URI grammar must safely replace when workflows need secret handles. The primary comparator remains vendor secret-reference formats and Vault/OpenBao path semantics.

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/cloud-secrets/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`, `microservices/cloud-secrets/IP-002-secretreference-uri-spec.md`.

## DR posture (per ADR-0343)

- Target source: `microservices/cloud-secrets/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`openbao_seal_unseal`, `postgres_wal_g`, `audit_chain_merkle_seal`].
- Surface evidence: `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/manifest.json`, `microservices/cloud-secrets/IP-002-secretreference-uri-spec.md`.
