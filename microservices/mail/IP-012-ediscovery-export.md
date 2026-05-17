---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-mail-dissolution-from-connect
impl_plan_id: IP-012-ediscovery-export
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-mail + ops-legal
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, oya-governance-ediscovery-chain-of-custody]
---

# IP-012: eDiscovery export bundle format + chain-of-custody verifier

## Intent

Define the sealed-bundle file format for eDiscovery exports + the chain-of-custody verifier. Re-derivable digest from source blocks (PRD §"Tenant Outcome 3" — sealed exports survive audit). EDRM XML 1.2 mapping for downstream legal tooling.

## ChangeSet boundary

Sub-crates within the legal-hold BC (`oya-mail-legal-hold-domain` adds `ediscovery_bundle.rs`); EDRM XML schema mapping; verifier binary `oya-mail-ediscovery-verifier`.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/mail/src/crates/oya-mail-legal-hold-domain/src/ediscovery_bundle.rs` | create | bundle format + manifest + per-block digest |
| `microservices/mail/src/crates/oya-mail-legal-hold-domain/src/edrm.rs` | create | EDRM XML 1.2 producer |
| `microservices/mail/src/crates/oya-mail-legal-hold-worker/src/export_job.rs` | create | streaming export (chunked S3 read + decryption + bundle append) |
| `microservices/mail/src/crates/oya-mail-ediscovery-verifier/` | create | standalone CLI verifier; third-party reproducible |
| `microservices/mail/tests/e2e/ediscovery-export.sh` | create | 10y-archive 5GB drill |

## Bundle format

```
ediscovery-<export_id>.tar.zst
├── manifest.json              # ULID export_id, hold_id, requested_by, approvers, blob list (sha256 each)
├── chain-of-custody.json      # Ed25519 over canonical(manifest)
├── edrm/                      # EDRM XML 1.2 representation
│   ├── Loadfile.xml
│   └── relativity-mapping.xml
├── messages/
│   ├── <message_id>.eml       # RFC 5322 form; decrypted under four-eyes co-sign (else encrypted)
│   └── <message_id>.json      # metadata
├── attachments/
│   └── <blob_sha>.bin
└── README.txt                  # human-readable verification instructions
```

## Chain-of-custody seal

```
seal = ed25519_sign(signing_key, sha256(canonical_manifest_json))
verify = ed25519_verify(public_key, seal, sha256(canonical_manifest_json))
```

Re-derivable digest: a verifier reads `manifest.json`, recomputes the sha256 for every listed block by hashing each `.eml` + `.bin` payload, compares to manifest entries. Any drift quarantines the bundle.

## Acceptance Gates

```bash
cargo run -p oya-mail-ediscovery-verifier -- verify --bundle <path>   # exit 0
cargo nextest run -p oya-mail-legal-hold-domain --test ediscovery
cargo run -p oya-dev-cli -- gate validate ediscovery-chain-of-custody --microservice mail
bash microservices/mail/tests/e2e/ediscovery-export.sh
```

## Test Plan

- Bundle digest re-derives from source blocks (PRD AC-09).
- Tampered .eml file → verify fails.
- Tampered manifest → verify fails.
- EDRM XML imports cleanly into Relativity / Logikcull / Casepoint reference tools (per ops-legal mandate).
- 10y archive 5GB drill: bundle produced ≤ 24h, verified, re-derived digest matches.

## Halt Conditions

- Verifier accepts tampered bundle → fail.
- Plaintext disclosure without four-eyes co-sign → fail.

## Next IP

[`IP-013-mail-workflow-handoff.md`](IP-013-mail-workflow-handoff.md)

## References

- EDRM XML 1.2 — `edrm.net/resources/standards/edrm-xml-1-2/`
- Federal Rules of Civil Procedure (FRCP) Rule 26(f) + Rule 34
- Sedona Conference Principles for ESI Production
- NIST SP 800-86 (Forensic Techniques)
- eIDAS Art. 32 (advanced electronic signatures)
- ISO 27037:2012 (digital evidence handling)
- Bominal ADR-0215 (retention/legal-hold dual-context)
