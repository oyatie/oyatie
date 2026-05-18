# consent-graph data residency model

- Owner: axis-consent-graph + privacy-officer
- Date: 2026-05-18
- Authority: ADR-0214 §2.5, ADR-SVC-CG-004, compliance.md.

## 1. Principles

1. **Grantor row never physically migrates**. The authoritative entity row lives only in the grantor's
   region/cell; cross-tenant access is a projection, not a copy.
2. **Projection topic resides in grantor's Pulsar cluster** (ADR-SVC-CG-004). Grantee subscribes
   cross-region.
3. **Projection cache on grantee side is bounded**. The cache is a denormalized view; it never holds
   raw row data fields outside the agreement's scope.
4. **Per-agreement opt-in geo-replication**. Cross-border replication requires explicit
   `geo_replicate_to_grantee_region=true` AND a permitted adequacy decision per the pack overlay.
5. **Pack overlay is source of truth**. Per-region rules in
   `iac/kustomize/overlays/<pack>/sovereignty-rules.yaml`.
6. **No data is "stateless" — every byte has a residency claim**.

## 2. Data classes and residency rules

### 2.1 Class A — agreement metadata (Postgres)
- Resides in: grantor's region only.
- Cross-region replica: DR region within same compliance zone (e.g., us-east-1 DR is us-west-2;
  ap-northeast-2 DR is ap-northeast-2 (multi-AZ only, no cross-region for KR pack)).
- Retention: 7y default; pack-configurable.

### 2.2 Class B — projection events (Pulsar topic payload)
- Resides in: grantor's region Pulsar cluster.
- Cross-region: forbidden by default; opt-in per agreement.
- Retention: 7d.

### 2.3 Class C — projection cache (grantee side)
- Resides in: grantee's region only.
- Cross-region: forbidden (this is grantee's local denorm).
- Retention: subject to ontology projection retention (typically 30d hot, archived per pack).

### 2.4 Class D — audit-chain entries (consent-graph bilateral)
- Resides in: each party's region's audit-chain instance.
- Cross-region: full cross-region replication NOT mandated; each party owns their own chain.
- Retention: 7y default; HIPAA 6y; AE/KSA 5y.

### 2.5 Class E — observability metrics + traces
- Cardinality-bounded; no PII.
- Resides in: each region's observability cluster.
- Cross-region: aggregated anonymous metrics for global dashboards.
- Retention: 30d metrics, 7d traces.

### 2.6 Class F — compiled Cedar policies
- Resides in: grantor's region (matches agreement row).
- Cross-region: not replicated (recompile-on-cold-start from agreement row).
- Cache TTL: lifetime of agreement.

### 2.7 Class G — OpenBao secrets (per-agreement keys)
- Resides in: grantor's region OpenBao instance.
- Cross-region: never (per OpenBao policy).
- Rotation: 90d for projection-topic-HMAC; 1y for pair-HMAC.

## 3. Pack-overlay residency matrix

| Pack | Class A | Class B | Class C | Class D | Class F |
|------|---------|---------|---------|---------|---------|
| kr | KR only | KR only | grantee region (if KR-grantee) else forbidden | KR only | KR only |
| eu | EU + Adequacy | EU only | grantee region within adequacy | EU + party region | EU only |
| us | US | US default; opt-in for partners | grantee region | US + party | US |
| us-healthcare | US-HIPAA-eligible | US-HIPAA-eligible | grantee US-HIPAA region | US-HIPAA | US-HIPAA |
| jp | JP + APPI-adequacy | JP default | grantee region (APPI adequacy required) | JP | JP |
| sg | SG | SG default | grantee region (PDPA §26) | SG | SG |
| au | AU | AU default | grantee region (APP 8) | AU | AU |
| in | IN | IN | grantee region (DPDP §10) | IN | IN |
| br | BR | BR default | grantee region (LGPD adequacy) | BR | BR |
| ae | AE | AE | grantee region (PDPL Art. 19) | AE | AE |
| ksa | KSA | KSA | grantee region (KSA PDPL) | KSA | KSA |

## 4. Enforcement

### 4.1 Pre-acceptance check (agreement-domain)
`domain::resolve_eligible_grantee_regions` consults pack overlay rules; agreement rejected at
acceptance time if violation.

### 4.2 Mint-time check (projection-gateway-kernel)
`assert_grantor_region(topic, expected)` ensures topic created in correct region.

### 4.3 Emit-time check (projection-gateway-kernel)
Re-asserts on every event emission — defense in depth.

### 4.4 Nightly audit job
`consent_graph_sovereignty_audit` worker:
- Lists all active projection topics.
- For each, queries Pulsar admin for actual cluster region.
- Compares against agreement-side stored region.
- Mismatch → P0 alert + auto-suspend.

### 4.5 PII classifier
Per IP-011 §5, the PII classifier (cross-border-forbidden categories) is checked at every emit. A
classifier mismatch is treated identically to a sovereignty violation.

## 5. Cross-border-permitted exceptions

Per-agreement opt-in must satisfy:
1. `cross_border_transfer_permitted=true` in `SovereigntyCfg`.
2. Adequacy decision check (per pack overlay).
3. Lawful basis recorded in `terms.metadata.cross_border_lawful_basis` ∈ {SCC, BCR, adequacy-decision,
   data-subject-explicit-consent, vital-interest, public-task}.
4. (For sensitive data Art. 9) explicit consent recorded.
5. (For US-Healthcare) BAA covering the cross-border recipient.

Auditor view: `GET /v1/agreements?cross_border=true&pack=eu` enumerates all cross-border agreements
for review.

## 6. Right to erasure cascade (cross-region)

When a DSAR erasure cascades:
1. consent-graph enumerates all active agreements projecting the subject.
2. For each, emits tombstone signal to grantee.
3. Grantee's ontology projection cache tombstones row.
4. Grantor's authoritative row tombstoned at ontology level.
5. Audit-chain records the cascade on both sides.
6. Compliance officer reviews evidence/dsar-erasure-cascade-<id>.json.

Pack-specific time bounds:
- GDPR: 30d cap; consent-graph targets 7d.
- KR PIPA: 10d.
- HIPAA: not erasure-mandated, but breach-notification cascade applies if violation.

## 7. Data minimization at retention end

When an agreement reaches retention end:
- Class A row tombstoned (not hard-deleted; tombstone is audit-citable).
- Class B Pulsar topic destroyed; partitions deleted.
- Class C projection cache tombstoned by grantee.
- Class D audit entries retained per audit-chain retention (default 7y from event date, not from
  agreement end).
- Class F compiled policy expired from cache; row deleted from compiled_policies table.
- Class G OpenBao secrets revoked; key versions destroyed after 90d.

Tombstones survive 1y for audit-chain forensic reconstruction; hard-delete after 1y unless legal
hold.

## 8. Data residency reporting

Quarterly compliance report:
- All Class A/B/C/D/F locations enumerated by region, by pack.
- Sovereignty-violation-zero SLO confirmed (any breach is regulatory disclosure).
- Cross-border agreement count + lawful-basis distribution.

Generated by `consent_graph_residency_report_worker` and sealed in audit-chain.

## 9. Verification

- Manifest validates: pack overlay rules exist for every supported pack.
- CI lint `oya-check-pack-overlay-residency-complete`.
- Quarterly residency report tabulated in `evidence/`.

## 10. Cross-references

- `compliance.md` per-regulation map.
- `dpia.md` for risk treatment.
- `multi-region.md` for geographic topology.
- ADR-SVC-CG-004 grantor-region topic ownership.
- ADR-0064 canonical-base + pack overlay neutrality.
