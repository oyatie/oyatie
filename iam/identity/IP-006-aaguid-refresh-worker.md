---
doc_class: IP
ip_id: IP-006
microservice: identity
status: ga
related_adrs: [ADR-0188]
date: 2026-05-18
owner_team: axis-identity
---

# IP-006 — FIDO-MDS3 AAGUID refresh worker

## Goal

Worker that fetches the FIDO Alliance Metadata Service v3 (MDS3) blob every 24h, verifies the JWT signature against the FIDO root CA, decodes the embedded JWT body, and updates the per-pack AAGUID allowlist with the L1+ / L2+ FIDO-MDS3 certification levels declared per ADR-0188 §"Attestation policy".

## Files

| File | Purpose |
|---|---|
| `crates/oya-identity-webauthn-aaguid-refresher-worker/Cargo.toml` | manifest |
| `crates/oya-identity-webauthn-aaguid-refresher-worker/src/lib.rs` | trait `AaguidRefresher` + tokio worker |
| `crates/oya-identity-webauthn-aaguid-refresher-worker/src/mds3_parser.rs` | FIDO-MDS3 JWT parse + root-CA verify |
| `crates/oya-identity-webauthn-aaguid-refresher-worker/src/postgres_allowlist_store.rs` | per-pack allowlist persistence |
| `crates/oya-identity-webauthn-aaguid-refresher-worker/tests/refresher.rs` | tests |
| `microservices/identity/specs/fido-mds3-allowlist-schema.json` | schema for stored allowlist |

## Algorithm

1. GET `https://mds.fidoalliance.org/`.
2. Parse JWT (3-segment).
3. Verify signature against pinned FIDO root CA.
4. Decode body: array of metadata statements with `aaguid` + `statusReports`.
5. For each AAGUID:
   - If latest status is `FIDO_CERTIFIED_L1`, `FIDO_CERTIFIED_L1_PLUS`, `FIDO_CERTIFIED_L2`, `FIDO_CERTIFIED_L2_PLUS`, `FIDO_CERTIFIED_L3`, `FIDO_CERTIFIED_L3_PLUS` → include with cert level.
   - If latest status is `REVOKED`, `ATTESTATION_KEY_COMPROMISE`, `USER_KEY_REMOTE_COMPROMISE`, `USER_VERIFICATION_BYPASS` → exclude + alarm.
   - Otherwise → exclude.
6. For each pack:
   - PackRegulated allowlist = AAGUIDs at L1+ or higher.
   - AcrCritical allowlist = AAGUIDs at L2+ or higher.
7. Atomically swap the per-pack allowlist in Postgres.
8. Emit `IdentityAaguidAllowlistUpdated(pack, added, removed)` event.

## Worker schedule

- Tokio interval; default 24h.
- Jittered ±30min to avoid herd effect across packs.
- Manual trigger via `oya identity aaguid refresh --pack <pack>`.

## Tests

| Test | Mechanism |
|---|---|
| `parses_well_formed_mds3_blob` | fixture FIDO-MDS3 blob; assert N AAGUIDs extracted |
| `verifies_signature_against_pinned_root` | fixture with valid sig; tamper one byte → reject |
| `rejects_revoked_aaguids` | AAGUID with status REVOKED is excluded |
| `regulated_pack_includes_l1_and_above` | L1, L1+, L2, L2+, L3 all included |
| `acr_critical_excludes_l1_includes_l2` | L1 excluded; L2+ included |
| `swaps_allowlist_atomically` | partial-failure simulation; old allowlist preserved on failure |
| `emits_allowlist_updated_event` | event observed on disk-stored sink |
| `stale_metadata_alerts_after_48h` | clock-forward; alert observed |
| `network_failure_serves_cached` | fetch fails; cache served |
| `concurrent_refresh_idempotent` | trigger twice; only one write |

## Failure modes

- **FIDO endpoint unreachable**: serve cache up to 7 days; alert at 2 days.
- **Root CA expired**: bake current FIDO root CA into binary; refresh on chart upgrade.
- **Postgres unavailable**: keep cache in memory; retry write at next interval.

## Evidence

- `evidence/identity/aaguid-refresh/<pack>/<date>.json` — refresh outcome
- `evidence/identity/aaguid-allowlist-diff/<pack>/<date>.json` — added/removed
- `evidence/identity/mds3-blob-signature-verify/<date>.json` — signature verify pass/fail

## Acceptance — DONE when

- 10 tests passing.
- Live integration against FIDO Alliance MDS3 production endpoint.
- Per-pack allowlists populated in Postgres.
- `aaguid-refresh-freshness` SLO target 0.999 over 30 days.

## Cross-references

- ADR-0188 §"Attestation policy"
- FIDO Alliance Metadata Service v3 specification
- ADR-0148 mesh egress allowlist for `mds.fidoalliance.org`

## Counterpart references - 006-aaguid-refresh-worker

- Counterpart class: passkey / recovery assurance.
- GitHub account security and Twilio Verify show the user-facing recovery and step-up baseline; this IP keeps Oyatie stronger by binding the credential or recovery decision to tenant context, ACR, and sealed identity audit events rather than treating MFA as an app-local add-on.
- Verification anchor: this row intentionally includes a named counterpart from the Wave 15 grep allowlist while keeping the implementation reference service-local: `microservices/identity/competitor-parity-matrix.md`, `iam/identity/PRD.md`, `iam/identity/manifest.json`, and the contract/policy files cited above.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `iam/identity/IP-006-aaguid-refresh-worker.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=30`, `rpo_p99_seconds=0` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), KR-PIPA-2023-amendment(14400s/900s), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), PCI-DSS-L1-v4(86400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `iam/identity/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `iam/observability/slos/identity/oidc-token-issue-latency.openslo.yaml`, `iam/observability/slos/identity/oidc-token-verify-latency.openslo.yaml`, `iam/observability/slos/identity/webauthn-authenticate-latency.openslo.yaml`, `iam/observability/slos/identity/scim-availability.openslo.yaml`, `iam/identity/policy/cedar-acr-predicates.cedar`.
