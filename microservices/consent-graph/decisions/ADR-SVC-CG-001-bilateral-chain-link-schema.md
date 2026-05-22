---
id: ADR-SVC-CG-001
title: "Bilateral consent-chain links use sealed cross-pointers"
status: Accepted
date: 2026-05-18
microservice: consent-graph
related_oyatie_adrs:
  - ADR-0003
  - ADR-0214
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0258
  - ADR-0263
decision_owner: axis-consent-graph + axis-audit-chain
---

# ADR-SVC-CG-001: Bilateral consent-chain links use sealed cross-pointers

## Context

- The named architectural pressure is `bilateral-consent-non-repudiation`.
- Consent-graph mediates data-sharing agreements between grantor tenants, grantee tenants, and data subjects.
- ADR-0214 establishes cross-tenant real-time visibility and makes bilateral consent evidence load-bearing.
- ADR-0003 establishes audit-chain and evidence emission as a platform primitive.
- Prior incident class `one-sided-consent-proof` allowed a grantor chain to show revocation while a grantee chain lacked the matching entry.
- Prior incident class `event-id-correlation-only` relied on shared event ids without cryptographic pairing.
- Prior incident class `cross-pointer-table-tamper` treated the cross-pointer row as truth instead of sealed evidence.
- A consent grant or revocation is legally sensitive under GDPR Art. 7, GDPR Art. 30, HIPAA §164.312(b), KR PIPA Art. 29, and SOC 2 CC7.2.
- The grantor must prove what it authorized.
- The grantee must prove what it received and when.
- The data subject must be able to challenge the existence or scope of a grant.
- The platform must detect if either side's chain is missing, reordered, or paired to the wrong counterparty.
- The platform must support sovereign pack boundaries; a grantor-region chain and grantee-region chain may live in different cells.
- The platform must avoid global blockchain consensus because ADR-0214 rejects cross-tenant public-ledger latency and data disclosure.
- The platform must support daily reconciliation across all active agreements.
- The platform must support emergency revocation where proof must still be available after fail-closed denial.
- The platform must keep proof material narrow: hashes, sequence ids, HMACs, and Merkle roots, not raw payload replication.
- The platform must survive a single compromised row store.
- The platform must survive a single compromised HMAC key by detecting inconsistency through audit-chain root mismatch.
- The platform must make verification implementable by an intern from this ADR and its references.

## Decision

- We choose `sealed bilateral cross-pointer` as the consent-chain pairing primitive.
- The named pattern is `Certificate Transparency-style Merkle proof with pairwise HMAC binding`.
- Each consent event writes a grantor-side chain link.
- Each consent event writes a grantee-side chain link.
- A cross-pointer binds the two chain links.
- A pair-confirmation entry seals the cross-pointer into both chains after primary link creation.
- The pair-confirmation entry is mandatory for grant, accept, scope-change, revoke, expire, and break-glass events.
- The cross-pointer table is an index, not the source of truth.
- The source of truth is the two sealed chain links plus pair-confirmation entries.
- `CrossPointerV1` contains grantor chain id, grantor sequence, grantor sealed time, and grantor Merkle root.
- `CrossPointerV1` contains grantee chain id, grantee sequence, grantee sealed time, and grantee Merkle root.
- `CrossPointerV1` contains agreement id, event id, consent subject id hash, and sharing mode.
- `CrossPointerV1` contains `paired_hmac`.
- `paired_hmac` is HMAC-SHA256 over canonical cross-pointer fields.
- The per-pair HMAC key is generated in OpenBao.
- The per-pair HMAC key path is `secret/consent-graph/pair-hmac/{grantor_tenant_id}/{grantee_tenant_id}/{agreement_id}`.
- The per-pair HMAC key rotation cadence is 365 days.
- A rotated HMAC key keeps prior verification versions for 7 years.
- Pair-confirmation must occur within 500 ms p99 after primary entry seal in one region.
- Pair-confirmation must occur within 2 seconds p99 across regions.
- Absence of pair-confirmation after 5 seconds emits Sev-2.
- Absence after 60 seconds emits Sev-1 and fail-closes the agreement.
- Reconciliation runs daily over every active agreement.
- Reconciliation recomputes HMAC and verifies both Merkle roots.
- Reconciliation verifies that event id, agreement id, subject hash, and sharing mode match.
- Reconciliation emits `ConsentGraphBilateralPairVerified` for successes.
- Reconciliation emits `ConsentGraphBilateralPairMismatch` for failures.
- Cedar action `consent-graph.cross_pointer.create` gates cross-pointer creation.
- Cedar action `consent-graph.cross_pointer.verify` gates verifier access.
- Cedar action `consent-graph.cross_pointer.repair` gates repair attempts.
- No actor may directly update a sealed pair-confirmation entry.

## Alternatives Considered

### Single grantor-side chain plus grantee receipt

- Pro: half the chain-write volume.
- Pro: simpler storage.
- Pro: easier grantor-region residency.
- Con: grantee can deny receipt by contesting the receipt path.
- Con: grantee-region audit evidence is weaker.
- Con: replay after regional outage becomes asymmetric.
- Con: data subject cannot compare both parties' sealed state.
- Tradeoff: simpler but weaker non-repudiation.
- Rejected.

### Event-id correlation only

- Pro: very easy to implement.
- Pro: no HMAC key management.
- Pro: no pair-confirmation entries.
- Con: event ids can be copied or paired incorrectly.
- Con: a tampered cross-pointer row can look plausible.
- Con: no cryptographic binding between roots.
- Con: does not satisfy HIPAA §164.312(b) audit integrity expectations.
- Tradeoff: operational simplicity but no tamper evidence.
- Rejected.

### Public blockchain anchoring

- Pro: external immutability.
- Pro: third-party verifiability.
- Pro: tamper evidence outside oyatie.
- Con: cross-tenant metadata disclosure.
- Con: unacceptable latency for real-time revocation.
- Con: sovereign packs may forbid external anchoring.
- Con: operational and legal complexity is disproportionate.
- Tradeoff: external proof but too much exposure and latency.
- Rejected.

### Shared global append-only ledger

- Pro: one chain to verify.
- Pro: simple total ordering.
- Pro: fewer bilateral pairs.
- Con: cross-pack residency violation.
- Con: cross-tenant blast radius.
- Con: global ordering is unnecessary for bilateral consent.
- Tradeoff: simpler verification but weaker sovereignty.
- Rejected.

## Consequences

- Positive: grantor and grantee both hold sealed evidence.
- Positive: tampering one side or one index row is detected.
- Positive: pair-confirmation makes the pairing itself auditable.
- Positive: daily reconciliation produces measurable integrity evidence.
- Positive: GDPR, HIPAA, KR PIPA, and SOC 2 audit expectations map to named events.
- Negative: chain volume roughly doubles for consent lifecycle events.
- Negative: pairwise HMAC keys require lifecycle management.
- Negative: cross-region pair-confirmation adds tail latency.
- Negative: reconciliation can produce noisy Sev-1 alerts if one chain lags.
- Neutral: pair-confirmation stores hashes and roots, not raw consent payload.
- Neutral: future signature-based pairing can coexist with HMAC by adding `pair_signature`.
- Follow-up work: implement `IP-013-audit-bridge-cross-pointer-integrity`.
- Follow-up work: add cross-pointer repair runbook.
- Follow-up work: add HMAC key rotation drill.
- Follow-up work: add dashboard panel for pair-confirmation lag.

## Implementation Notes

- Data shape `ChainLinkV1` contains `chain_id`, `seq`, `sealed_at`, and `merkle_root`.
- Data shape `CrossPointerV1` contains `grantor`, `grantee`, `agreement_id`, `event_id`, `subject_hash`, `sharing_mode`, `paired_hmac`, and `hmac_key_version`.
- Field `chain_id` is ULID prefixed by `cchain_`.
- Field `seq` is unsigned 64-bit integer.
- Field `sealed_at` is RFC 3339 with millisecond precision.
- Field `merkle_root` is base64url SHA-256 Merkle root.
- Field `agreement_id` is ULID prefixed by `dsa_`.
- Field `event_id` is ULID prefixed by `cg_evt_`.
- Field `subject_hash` is HMAC-SHA256 under the subject pseudonymization key.
- Field `sharing_mode` is `projection`, `aggregate`, or `attested_query`.
- Field `paired_hmac` is base64url HMAC-SHA256.
- Field `hmac_key_version` is OpenBao version.
- Canonicalization uses RFC 8785 JSON.
- API endpoint `POST /v1/agreements/{agreement_id}/chain-links` creates lifecycle chain links.
- API endpoint `POST /v1/agreements/{agreement_id}/cross-pointers` creates cross-pointers.
- API endpoint `GET /v1/agreements/{agreement_id}/cross-pointers/{event_id}` returns verification metadata.
- API endpoint `POST /v1/internal/cross-pointers/{event_id}/verify` runs targeted verification.
- API endpoint `POST /v1/internal/cross-pointers/{event_id}/repair` starts repair workflow.
- Cedar principal for creation is `Oyatie::Principal::Service("consent-graph.audit-bridge-worker")`.
- Cedar principal for verification is `Oyatie::Principal::Service("consent-graph.reconciler")`.
- Cedar action `consent-graph.cross_pointer.create` applies to `ConsentGraph::CrossPointer`.
- Cedar action `consent-graph.cross_pointer.verify` applies to `ConsentGraph::CrossPointer`.
- Cedar action `consent-graph.cross_pointer.repair` applies to `ConsentGraph::CrossPointer`.
- Cedar context field `grantor_tenant_id` must match grantor chain tenant.
- Cedar context field `grantee_tenant_id` must match grantee chain tenant.
- Cedar context field `agreement_state` must allow the lifecycle event.
- Cedar context field `pack_id` must match both chain residency rules or approved cross-pack terms.
- Example permit: principal `consent-graph.audit-bridge-worker`, action `consent-graph.cross_pointer.create`, resource `ConsentGraph::CrossPointer::"cg_evt_01HY"`, context `{agreement_state:"accepted", grantor_tenant_id:"tn_a", grantee_tenant_id:"tn_b", pack_id:"gdpr-eu"}`.
- Example forbid: same action with context `{grantor_tenant_id:"tn_a", grantee_tenant_id:"tn_b", pack_id:"kr-strict", geo_replicate:true}`.
- OpenBao HMAC key path is pack-local.
- HMAC key material never enters application logs.
- Pair-confirmation entry type is `CrossPointerPairConfirmed`.
- Pair-confirmation entry includes cross-pointer digest and HMAC key version.
- Audit event `ConsentGraphBilateralPairCreated` emits after cross-pointer write.
- Audit event `ConsentGraphBilateralPairConfirmed` emits after both confirmations.
- Audit event `ConsentGraphBilateralPairMismatch` emits on verification failure.
- Metric `oya_consent_graph_pair_confirmation_lag_ms` tracks lag.
- Metric `oya_consent_graph_cross_pointer_mismatch_total` counts mismatch reasons.
- Metric `oya_consent_graph_cross_pointer_verified_total` counts daily verification successes.
- Metric cardinality includes tenant pair hash, not raw tenant ids, for dashboard safety.
- Dashboard `consent-graph-bilateral-integrity.json` shows lag, mismatch count, verification coverage, and key age.
- SLO `bilateral-chain-link-integrity.openslo.yaml` sets mismatch budget zero.
- SLO `pair-confirmation-latency.openslo.yaml` sets p99 <= 500 ms same-region and <= 2 seconds cross-region.
- Failure mode `grantor_chain_missing` fails closed and opens Sev-1.
- Failure mode `grantee_chain_missing` fails closed and opens Sev-1.
- Failure mode `hmac_mismatch` quarantines agreement and opens Sev-1.
- Failure mode `merkle_root_mismatch` quarantines chain segment and starts audit-chain replay.
- Failure mode `pair_confirmation_late` fails closed after 60 seconds.

## Verification

- Test `cross_pointer_hmac_roundtrip` recomputes HMAC from canonical fields.
- Test `cross_pointer_rejects_wrong_grantor_root` verifies mismatch detection.
- Test `cross_pointer_rejects_wrong_grantee_root` verifies mismatch detection.
- Test `pair_confirmation_required_for_grant` verifies grant without confirmation fails verification.
- Test `pair_confirmation_required_for_revocation` verifies revocation without confirmation fails closed.
- Test `cross_pointer_cedar_grantor_grantee_match` verifies tenant context.
- Test `cross_pointer_kr_strict_geo_replication_forbidden` verifies pack overlay.
- Test `cross_pointer_key_rotation_verifies_old_pairs` verifies retained key versions.
- Test `cross_pointer_daily_reconciler_covers_all_active_agreements` verifies coverage.
- Test `cross_pointer_repair_requires_privileged_action` verifies repair gating.
- Metric `oya_consent_graph_pair_confirmation_lag_ms` must meet p99 <= 500 ms same-region.
- Metric `oya_consent_graph_cross_pointer_mismatch_total` must remain zero.
- Metric `oya_consent_graph_cross_pointer_verified_total` must equal active pair count daily.
- Dashboard `consent-graph-bilateral-integrity.json` must show key age and mismatch reason.
- Dashboard `consent-graph-consent-risk.json` must include fail-closed agreement count.
- CI check `consent-cross-pointer-schema` validates `CrossPointerV1` fixtures.
- CI check `consent-cross-pointer-hmac` validates canonical HMAC.
- CI check `consent-cross-pointer-cedar-coverage` verifies actions and resources.
- CI check `consent-cross-pointer-no-raw-payload` rejects raw consent payload in cross-pointer rows.
- CI check `oya-governance-observability-emission --microservice consent-graph` verifies ADR-0263 telemetry.
- CI check `consent-cross-pointer-root-lineage` verifies both roots descend from the active audit-chain root.
- CI check `consent-cross-pointer-key-version` verifies every row has a retained OpenBao key version.
- CI check `consent-cross-pointer-pack-residency` rejects cross-pack rows without approved transfer terms.
- CI check `consent-cross-pointer-subject-hash` rejects raw data-subject identifiers.
- Load test verifies 100,000 pair-confirmations under p99 target.
- Chaos test drops grantee confirmation and expects fail-closed after 60 seconds.
- Chaos test rotates HMAC key mid-stream and verifies old and new pairs.
- Security test tampers cross-pointer table row and expects mismatch.
- Audit query verifies every consent lifecycle event has paired confirmation evidence.
- Audit query `consent_pair_gap_report.sql` runs daily and must return zero open gaps older than 60 seconds.
- Dashboard panel `Pair confirmation lag by pack` alerts at p99 > 500 ms same-region.
- Dashboard panel `Cross-region pair lag` alerts at p99 > 2 seconds.
- Dashboard panel `Mismatch reason histogram` groups by `hmac_mismatch`, `root_mismatch`, and `missing_side`.
- Runbook drill `cross-pointer-late-confirmation` executes quarterly.
- Runbook drill `pair-hmac-key-rotation` executes semiannually.
- Evidence bundle `consent-graph-cross-pointer-quarterly.zip` retains CI logs, dashboard exports, and audit queries for 7 years.

## References

- ADR-0003: Audit-chain and evidence emission.
- ADR-0214: Cross-tenant real-time visibility.
- ADR-0243: Cedar as Universal Gate.
- ADR-0244: Tenant as universal scoping primitive.
- ADR-0251: Compliance pack cell certification levels.
- ADR-0258: API versioning model.
- ADR-0263: Observability emission contract.
- RFC 2104: HMAC.
- RFC 8785: JSON Canonicalization Scheme.
- RFC 6962: Certificate Transparency Merkle-tree pattern.
- GDPR Art. 7 and Art. 30.
- HIPAA 45 CFR §164.312(b).
- KR PIPA Art. 29.
- SOC 2 CC7.2.
- OpenBao KV and transit documentation.
- Google Certificate Transparency design notes.
