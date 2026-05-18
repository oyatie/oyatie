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
