---
doc_class: IncidentResponse
microservice: cloud-secrets
status: Accepted
date: 2026-05-17
owner_team: ops-security + axis-cloud-secrets + ops-sre
deciders: ops-security, ops-legal, council-privacy, axis-cloud-secrets
related_adrs: [ADR-0028, ADR-0117, ADR-0131]
related_artifacts:
  - microservices/cloud-secrets/threat-model.md
  - microservices/cloud-secrets/failure-modes.md
  - microservices/cloud-secrets/runbooks/*.md
review_cadence: quarterly + after every Sev-1/Sev-2
doc_status: published
---

# Incident Response: cloud-secrets µservice

This document defines the severity ladder, response timeline, communication tree, and tenant + regulator notification SLAs for incidents in cloud-secrets. The two **always-Sev-1** categories are:

- **Raw-secret-leak detected** (anywhere — repo, chat, checkpoint, log, exfiltration confirmed)
- **HSM compromise** (attestation failure, vendor-confirmed supply-chain incident, key extraction confirmed/suspected)

## Severity Ladder

| Sev | Definition | Page | Tenant notification | Regulator notification | Post-mortem |
|---|---|---|---|---|---|
| Sev-1 | Secret-resolution unavailable cluster-wide OR raw-secret-leak detected OR HSM compromise OR cross-pack data movement of `SECRET` material | grafana-oncall page; ops-security + axis-cloud-secrets + on-call exec | within 72h (GDPR Art. 33); 24h (LGPD Art. 48); 24h (KSA PDPL); per BAA (HIPAA) | within statutory cadence per pack (24-72h) | mandatory; due 2 weeks |
| Sev-2 | Pack-scoped degradation; SLO burn >14.4× over 1h; rotation backlog >SLA | grafana-oncall page; axis-cloud-secrets + ops-sre | only if SLA breach impacts tenant operations | only if regulator-applicable threshold | mandatory; due 1 week |
| Sev-3 | Tenant-scoped or microservice-scoped degradation | ticket; axis-cloud-secrets | if tenant-impacting | n/a | optional |
| Sev-4 | Cosmetic, dashboard, latency hiccup within budget | observability lane | n/a | n/a | n/a |

## Detection → Page → Response Flow

```text
[Detection]
    ↓
[Auto-classify Sev]
    ↓
[Page grafana-oncall]
    ↓
[Incident commander assumes role]
    ↓
[Containment → Eradication → Recovery → Communication → Post-mortem]
```

## Sev-1 Response: Raw-Secret-Leak Detected

### Timeline

| Phase | SLA | Action |
|---|---|---|
| Detection | t=0 | LEAN-A11 lane fail OR retroactive scanner finding OR external responsible-disclosure |
| Triage | t+5min | Confirm true positive; classify Sev-1 |
| Containment | t+10min | Identify all affected secrets via path + scope; mark for revocation |
| Eradication | t+15min | `openbao secrets revoke <path>` for every affected secret; cascade-rotate dependents |
| Recovery | t+30min | Confirm all consumer SDKs flushed cache (revocation push propagated); verify with chaos drill snapshot |
| Communication | t+72h max | Tenant notification (Art. 33); regulator notification per pack |
| Post-mortem | t+2 weeks | Root cause; pattern-update for LEAN-A11; team review |

### Containment Actions

1. **Immediate revocation**: `cargo run -p oya-cloud-secrets-secret-reference-resolver-app -- admin revoke <path>`; OpenBao emits SecretRevoked; revocation push to consumers.
2. **Cascade-rotate dependents**: identify secrets depending on the revoked secret (e.g., DEKs encrypted by a revoked KEK); rotation-scheduler triggered with `priority=immediate`.
3. **Audit-emit**: every revocation + rotation event sealed in audit-chain with `incident_id` correlation.
4. **Forensic**: examine logs, agent transcripts, git history (find the emission origin); fix the gap (pattern update, training, tool change).

### Communication Tree

| Stakeholder | Channel | Window |
|---|---|---|
| Incident commander (ops-security on-call) | grafana-oncall page → Slack incident-channel | t=0 |
| axis-cloud-secrets lead | Slack | t+5min |
| ops-legal | Slack | t+30min |
| Tenant DPO (if tenant data affected) | tenant DPA-listed contact | within 72h max (GDPR Art. 33); earlier per pack SLA |
| Regulators (per pack) | per-pack regulator contact list at `legal/regulator-contacts.md` (Slice D) | per pack statutory cadence |
| oyatie status page | external status page | per Sev-1 escalation policy |

### Tenant Notification Template

See `runbooks/secret-leak-detected.md` §"Tenant notification template" for the canonical text.

### Post-Mortem Mandate

Sev-1 post-mortem is mandatory. Format: blameless, 5-whys + contributing-factors. Outcomes:
- LEAN-A11 pattern update (if new pattern caused the leak).
- Process update (operator training, tool change).
- Tenant DPA addendum (if needed).
- ADR if architectural change is warranted.

## Sev-1 Response: HSM Compromise

### Timeline

| Phase | SLA | Action |
|---|---|---|
| Detection | t=0 | Daily attestation verification failure OR vendor disclosure |
| Triage | t+10min | Confirm via cross-check vendor attestation chain |
| Containment | t+30min | Halt new HSM ops to the affected partition; switch to HA partition for routine ops |
| Eradication | t+24h | KEK ceremony in alternate HSM (different vendor partition); KEK-of-KEKs rotation; cascade re-wrap all DEKs (extensive — may take 24h+) |
| Recovery | t+48h | New KEK fully propagated; old partition decommissioned |
| Communication | t+24h max (vendor) + 72h max (tenants) | per Sev-1 protocol; vendor incident review |
| Post-mortem | t+2 weeks | Root cause; vendor switch if warranted |

### Vendor Switch Decision

If HSM compromise is vendor-side and impacts trust in the vendor:
- Move affected pack to alternate vendor (pack-kr: switch OCI ↔ Luna; other packs: alternate OCI region or vendor).
- Re-run KEK ceremony.
- Tenant communication includes vendor change.

## Sev-1 Response: Cross-Pack Data Movement

If `SECRET`-class material moves across packs (residency breach):
- Sev-1 immediately.
- Quarantine the destination pack's affected namespace.
- Forensic: identify the movement mechanism (policy mis-author, operator action, code bug).
- Cryptographic-erase any cross-pack copies.
- Regulator notification per pack (KR PIPA, GDPR, LGPD, etc.).
- DPA + ROPA update.

## Sev-1 Response: Cluster-Wide Resolution Unavailable

See `runbooks/openbao-restart.md` for the operational sequence:
1. Confirm scope (single AD, region-wide, pack-wide).
2. Apply containment (cache TTL extension to 60s if not already).
3. Failover (intra-AD or DR if region).
4. Recovery + verification.

## Sev-2 Response

Sev-2 incidents follow same flow as Sev-1 but with reduced SLA pressure:
- Page within 5min.
- Containment + recovery within 1h.
- Tenant notification only if SLA breach.
- Post-mortem within 1 week.

Examples:
- Rotation backlog > 30min.
- Audit emission backlog > 60s but < 300s.
- Single-pack OpenBao quorum loss (recovered via auto-restart).

## Sev-3 + Sev-4 Response

- Sev-3: ticket; investigated next business day; tenant-comm if tenant-impacting.
- Sev-4: tracked in observability lane; no page.

## Regulator Contact Cadence

| Pack | Regulator | Notification cadence | Mechanism |
|---|---|---|---|
| pack-kr | KISA + PIPC | 24h for personal-data breach | KISA online portal + PIPC email |
| pack-eu | DPA of lead member state (per Art. 56) | 72h (Art. 33) | DPA breach portal |
| pack-us-healthcare | HHS OCR | 60 days (HIPAA Breach Notification Rule) for ≥500 affected | HHS OCR breach portal |
| pack-us (PCI) | acquiring bank + card brand | per merchant agreement (typically 24h) | per merchant agreement |
| pack-au | OAIC | 30 days (Notifiable Data Breaches scheme) | OAIC notification form |
| pack-in | DPB (Data Protection Board) | 72h (DPDPA §10) | DPB portal |
| pack-br | ANPD | 24h (LGPD Art. 48) | ANPD portal |
| pack-ae | UAE Data Office | per UAE PDPL Art. 9 | UAE Data Office |
| pack-ksa | SDAIA + SAMA (if financial) | per KSA PDPL + SAMA cadence | SDAIA portal + SAMA |
| pack-jp | PPC (Personal Information Protection Commission) | per APPI | PPC online |
| pack-sg | PDPC | per PDPA notification | PDPC portal |

Full regulator contact list at `microservices/cloud-secrets/legal/regulator-contacts.md` (Slice D).

## Drills (cadence)

| Drill | Cadence | Acceptance |
|---|---|---|
| Sev-1 raw-secret-leak tabletop | quarterly | full timeline executed; all stakeholders responded |
| Sev-1 HSM compromise tabletop | semi-annually | KEK ceremony rehearsal; vendor-switch decision exercised |
| Sev-1 cluster-wide unavailability | quarterly | failover within RTO |
| Sev-1 cross-pack data movement detection | quarterly | Cedar deny intercepts; audit emits |
| Communication tree test | semi-annually | all stakeholders reachable; backup contacts validated |
| Regulator-notification dry-run | annually | template + portal access verified per pack |

## Incident Records

| Field | Captured at |
|---|---|
| incident_id | ULID assigned at triage |
| sev_class | Sev-1 / Sev-2 / Sev-3 / Sev-4 |
| detected_at | timestamp |
| contained_at | timestamp |
| recovered_at | timestamp |
| tenants_notified | list + timestamps |
| regulators_notified | list + timestamps |
| post_mortem_id | URL to post-mortem doc |
| linked_failure_modes | FM-xx list per `failure-modes.md` |

Records sealed in audit-chain with Ed25519 + Merkle.

## References

- `microservices/cloud-secrets/threat-model.md`
- `microservices/cloud-secrets/failure-modes.md`
- `microservices/cloud-secrets/runbooks/secret-leak-detected.md`
- `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`
- `microservices/cloud-secrets/runbooks/openbao-restart.md`
- `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`
- `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`
- `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`
- `microservices/cloud-secrets/legal/regulator-contacts.md` (Slice D)
- GDPR Art. 33 (breach notification)
- KR PIPA Art. 34
- HIPAA Breach Notification Rule
- LGPD Art. 48
