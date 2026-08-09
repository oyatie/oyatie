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

Reconcile the canonical SecretReference URI grammar with the crate-backed parser and test corpus now present in `cloud/cloud-secrets/crates/oya-secrets-domain/**`. This spec remains the contract every SDK + LEAN-A11 lane consumes, but future machine-readable artifacts must track the shipped parser rather than invent a second shape.

## ChangeSet boundary

Current implementation surface: parser, normalized serializer, config-wrapper parser, and TTL clamp live in `cloud/cloud-secrets/crates/oya-secrets-domain/src/lib.rs`; direct parser/TTL coverage lives in `cloud/cloud-secrets/crates/oya-secrets-domain/tests/secret_reference_uri.rs`. Planned machine-readable JSON/ABNF/corpus artifacts must mirror this implementation before they become release authority.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `cloud/cloud-secrets/crates/oya-secrets-domain/src/lib.rs` | update/verify | current `SecretReferenceUri` parser, serializer, config wrapper, and TTL clamp |
| `cloud/cloud-secrets/crates/oya-secrets-domain/tests/secret_reference_uri.rs` | update/verify | current valid/invalid parser coverage and TTL ceiling tests |
| `secrets/contracts/secret-reference-uri.json` | planned | service-owned machine-readable mirror of the crate-backed contract, not independent authority |
| `secrets/contracts/secret-reference-uri.abnf` | planned | ABNF mirror of the crate-backed contract |
| `secrets/contracts/secret-reference-uri-test-corpus.jsonl` | planned | expanded corpus once generated SDKs consume the same cases |

## ABNF

```
SecretReferenceURI = "openbao:secret/" path [ "@" version ]
ConfigReference    = "${" SecretReferenceURI "}"
path               = segment *( "/" segment )
segment            = 1*( ALPHA / DIGIT / "_" / "-" / "." / ":" ) ; segment MUST NOT be ".."
version            = "v" positive-integer
positive-integer   = nonzero-digit *DIGIT
nonzero-digit      = %x31-39
```

## Acceptance Gates

```bash
buck2 test //cloud/cloud-secrets/crates/oya-secrets-domain:secret-reference-uri-test
buck2 build //cloud/cloud-secrets/crates/oya-secrets-domain:oya-secrets-domain[check]
```

## Test Plan

Current crate tests must round-trip canonical OpenBao references, parse `${...}` config wrappers, reject invalid prefixes/empty or traversal segments/query strings/bad versions/raw secret material, and clamp cache TTLs to <=60 seconds. Future SDK parsers in Rust/TS/Python must consume the same expanded corpus once the machine-readable artifacts are generated.

## Halt Conditions

- Grammar change that breaks existing consumer references — require ADR + migration plan.

## Next IP

`IP-003-resolver-kernel.md`

## References

- `secrets/policy/secret-isolation.md` §"TI-03"
- `secrets/contracts/proto/cloud-secrets.proto`
- `cloud/cloud-secrets/crates/oya-secrets-domain/src/lib.rs`
- `cloud/cloud-secrets/crates/oya-secrets-domain/tests/secret_reference_uri.rs`

## Wave 15-IP-substance A-G

### A. Problem
`cloud-secrets` needs one parseable SecretReference contract so product µservices can carry `${openbao:secret/<path>}` without ever copying raw values into repos, chat, images, telemetry, or checkpoints. The gap is not generic secret storage; it is a repo-wide typed reference grammar that lines up with the PRD's zero raw-secret exposure requirement, the LEAN-A11 blocker, and the OpenAPI/proto contracts.

### B. Approach
Define the URI grammar and test corpus as service-owned contract artifacts, then require every Rust/TS/Python parser and the resolver domain crate to consume the same cases. The current shipped grammar accepts only the OpenBao scheme, non-empty safe path segments, optional positive numeric version suffix, and `${...}` config wrappers; query strings and traversal are rejected.

### C. Deliverables
- Current crate-backed parser and TTL clamp in `cloud/cloud-secrets/crates/oya-secrets-domain/src/lib.rs`, with direct tests in `cloud/cloud-secrets/crates/oya-secrets-domain/tests/secret_reference_uri.rs`.
- Contract alignment with `contracts/openapi/cloud-secrets.yaml` and `contracts/proto/cloud-secrets.proto`.
- Machine-readable ABNF/JSON/corpus mirrors remain planned follow-ups and must match the crate-backed parser before generated SDKs depend on them.
- Policy evidence from `policy/secret-isolation.md` and `policy/tenant-scope.cedar`.
- SLO linkage to `slos/secret-resolve-latency.openslo.yaml`.

### D. Ordered Implementation Steps
1. Keep the crate-backed parser contract as `openbao:secret/<path>[@vN]` plus `${...}` config wrapper support.
2. Keep valid/invalid fixtures for generic safe path segments, version pins, forbidden traversal, forbidden query strings, and secret-shaped literals.
3. Mirror the shipped grammar into OpenAPI, proto, and SDK docs so generated clients do not invent local variants.
4. Preserve the resolver-domain parser, normalized serializer, and TTL clamp in `oya-secrets-domain`.
5. Add Rust/TS/Python corpus tests once a shared fixture file is generated from the same contract.
6. Wire LEAN-A11 to flag raw-secret literals and allow only SecretReference strings.
7. Publish migration notes for existing config references.

### E. Acceptance
- `buck2 test //cloud/cloud-secrets/crates/oya-secrets-domain:secret-reference-uri-test`.
- `buck2 build //cloud/cloud-secrets/crates/oya-secrets-domain:oya-secrets-domain[check]`.
- Branch-protected `oya-ci-required` / owned `oya-ci` evidence for doc-coverage and LEAN-A11 before external release claims.
- Every accepted reference carries a tenant-safe path and never serializes a raw value.
- `secret-resolve-latency` SLO remains bound to runtime resolve, not grammar parsing alone.

### F. Evidence
Primary evidence lives in `PRD.md` FR-01/FR-02, `ARCHITECTURE.md` SecretReference resolver sections, `manifest.json` crate and contract lists, `policy/secret-isolation.md`, `contracts/openapi/cloud-secrets.yaml`, `contracts/proto/cloud-secrets.proto`, and the crate-backed parser/tests under `cloud/cloud-secrets/crates/oya-secrets-domain/`.

### G. Counterpart Comparison
AWS Secrets Manager ARNs, Google Secret Manager resource names, and HashiCorp Vault paths all provide runtime lookup indirection, but the parity matrices show they do not enforce Oyatie's `Secret<T>` wrapper, cache TTL ceiling, no-log guarantee, or LEAN-A11 raw-secret blocker. This IP closes that counterpart gap by making the reference grammar itself the only allowed configuration surface.

Grep-recognized counterpart anchor: GitHub Actions Secrets is referenced only as a CI distribution counterpart that this URI grammar must safely replace when workflows need secret handles. The primary comparator remains vendor secret-reference formats and Vault/OpenBao path semantics.

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `secrets/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `secrets/contracts/openapi/cloud-secrets.yaml`, `secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `secrets/contracts/proto/cloud-secrets.proto`, `secrets/IP-002-secretreference-uri-spec.md`.

## DR posture (per ADR-0343)

- Target source: `secrets/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`openbao_seal_unseal`, `postgres_wal_g`, `audit_chain_merkle_seal`].
- Surface evidence: `secrets/runbooks/hsm-key-rotation.md`, `secrets/runbooks/openbao-restart.md`, `secrets/manifest.json`, `secrets/IP-002-secretreference-uri-spec.md`.
