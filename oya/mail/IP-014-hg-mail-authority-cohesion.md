---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-mail-dissolution-from-connect
impl_plan_id: IP-014-hg-mail-authority-cohesion
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-mail
acceptance_lanes: [oya-governance-authority-cohesion, oya-governance-hyperscaler-maturity-claims, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-014: HG-MAIL registration + authority-cohesion + branch-protection

## Intent

Register the `HG-MAIL` hyperscaler-maturity gate per ADR-0123. Extend `.github/branch-protection.yaml` to enforce mail's per-microservice promotion lane. Bind `mail` µservice to authority-cohesion contract per ADR-0123.

## ChangeSet boundary

Repo-wide IaC + spec updates. No Rust code.

## Concrete File Targets

| Path | Action |
|---|---|
| `/specs/hyperscaler-gates.json` | update | register HG-MAIL with claim boundary + verification method |
| `.github/branch-protection.yaml` | update | extend `release/mail/staging` + `release/mail/production` patterns with promotion-readiness lane |
| `.github/workflows/promote-mail-dev-to-staging.yml` | create | event-driven (eligibility-changed) promotion workflow |
| `.github/workflows/promote-mail-staging-to-production.yml` | create | event-driven promotion workflow |
| `registry/artifact-capabilities-registry.json` | update | add mail-send, mailbox-search, ediscovery-export, dlp-quarantine capabilities |
| `registry/knowledge-graph-semantic.json` | update | mail nodes + edges |
| `microservices/mail/competitor-parity-matrix.md` | reference | claim boundaries audited |

## HG-MAIL claim boundary

```json
{
  "gate_id": "HG-MAIL",
  "microservice": "mail",
  "claims_permitted": [
    "Dual-context (Personal/Professional) isolation at the kernel layer",
    "Four-eyes legal hold + eDiscovery with re-derivable chain-of-custody digest",
    "Per-tenant SMTP IP reputation as a first-class FinOps surface",
    "SMTP / IMAP4rev2 / JMAP / MIME / DKIM / SPF / DMARC / ARC / S/MIME / OpenPGP / MTA-STS / TLS-RPT standards-compatible at the edge",
    "Encrypted-token search; no plaintext indexing",
    "Per-pack data residency (11 packs) with cross-pack replication forbidden",
    "Self-hosted; no Exchange / Gmail dependency",
    "Mail-to-Workflow handoff requires explicit consent or tenant policy basis"
  ],
  "claims_forbidden": [
    "We beat Gmail / Outlook / Proton on any unsourced metric",
    "HIPAA-compliant out of the box (conditional on BAA + pack-us-healthcare activation)",
    "GDPR-compliant out of the box (requires DPA + per-pack overlay)",
    "AI-Act compliant for high-risk classifiers (requires conformity assessment)",
    "Cost-advantage over hosted providers (depends on workload shape)"
  ],
  "verification": "microservices/mail/competitor-parity-matrix.md + bi-annual benchmarks"
}
```

## branch-protection.yaml diff

```yaml
branches:
  dev:
    required_status_checks:
      # ADDED:
      - oya-governance-dual-context-cross-boundary
      - oya-governance-retention-floor-conformance
      - oya-governance-dkim-key-rotation-conformance
      - oya-governance-ediscovery-chain-of-custody
      - oya-governance-mail-encryption-tenant-dek
      - oya-governance-mta-sts-conformance
      - oya-governance-jmap-conformance
      - oya-governance-imap-conformance
      - oya-governance-search-index-context-partition
      - oya-governance-encrypted-token-conformance
      - oya-governance-personal-pillar-hold-forbidden

  ? release/mail/staging
  :
    require_pull_request: false
    require_linear_history: true
    disallow_force_push: true
    require_signed_commits: true
    required_status_checks: [oya-governance-promotion-readiness]

  ? release/mail/production
  :
    require_pull_request: false
    require_linear_history: true
    disallow_force_push: true
    require_signed_commits: true
    required_status_checks: [oya-governance-promotion-readiness]
```

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate authority-cohesion
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice mail
```

## Test Plan

- HG-MAIL registers; gate-validate exit 0.
- Branch protection emulation: PR to dev without all new status-checks → blocked.
- Promotion workflow: eligibility-changed event triggers `promote-mail-dev-to-staging` workflow start.

## Halt Conditions

- Forbidden claim string found in marketing materials → fail.
- Branch protection regression (any required check removed) → block.


## DR posture (per ADR-0343)
- Manifest target source: `microservices/mail/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/mail/IP-014-hg-mail-authority-cohesion.md` matched `SLO`; anchors `microservices/mail/runbooks/mailbox-restore-from-backup.md, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.

## Next IP

[`IP-015-pack-kr-overlay.md`](IP-015-pack-kr-overlay.md)

## References

- ADR-0123 (hyperscaler-maturity-claim-gate)
- ADR-0139 (agentic SLO-gated promotion)
- ADR-0131 (per-microservice flat layout)
- `microservices/mail/competitor-parity-matrix.md`
- Bominal ADR-0208/0210/0215 (mail inheritance)
