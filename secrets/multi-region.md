---
doc_class: MultiRegion
microservice: cloud-secrets
status: Accepted
date: 2026-05-17
owner_team: axis-cloud-secrets + ops-sre
related_adrs: [ADR-0117, ADR-0131]
related_artifacts:
  - microservices/cloud-secrets/policy/data-residency.md
  - microservices/cloud-secrets/capacity-model.md
  - microservices/cloud-secrets/incident-response.md
review_cadence: annually + on every pack activation
doc_status: published
---

# Multi-Region: cloud-secrets µservice

## Posture

cloud-secrets follows oyatie's **per-pack residency model** (per ADR-0117 + `policy/data-residency.md`): each pack runs its own OpenBao cluster + HSM partition + Postgres-HA backend in a pack-pinned region or DR-pair. **Cross-pack replication is forbidden by default for secrets, KEK material, and audit events.**

## Region Topology by Pack

| Pack | Primary region | DR region | Replication mode | Notes |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | none (single-region) | within-region 5-node Raft + Patroni sync replication | KR-FSS may require dual-region; defer to first FSS tenant |
| pack-eu | OCI eu-frankfurt-1 | OCI eu-amsterdam-1 | DR-pair sync Raft + sync Patroni | Schrems-II supplementary measures applied |
| pack-us | OCI us-ashburn-1 | OCI us-phoenix-1 | DR-pair sync | |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | none initially | within-region; DR pair eligible region only | BAA conditional |
| pack-jp | OCI ap-tokyo-1 | none initially | within-region | |
| pack-sg | OCI ap-singapore-1 | none initially | within-region | |
| pack-au | OCI ap-sydney-1 | OCI ap-melbourne-1 | DR-pair sync | APRA-CPS 234 alignment |
| pack-in | OCI ap-hyderabad-1 | OCI ap-mumbai-1 | DR-pair sync | DPDPA + RBI |
| pack-br | OCI sa-saopaulo-1 | OCI sa-vinhedo-1 | DR-pair sync | LGPD |
| pack-ae | OCI me-abudhabi-1 | OCI me-dubai-1 | DR-pair sync | UAE PDPL |
| pack-ksa | OCI me-jeddah-1 | OCI me-riyadh-1 | DR-pair sync | KSA PDPL + SAMA + NCA |

## Single-Region Posture

For packs without a DR-pair (pack-kr initial, pack-jp, pack-sg, pack-us-healthcare initial):
- 5-node OpenBao Raft cluster spread across ≥3 availability domains (ADs) within the region.
- Patroni-HA Postgres with synchronous replication across ≥3 ADs.
- HSM partition + HA replica in same region.
- Object storage for backups in same region with versioning + lifecycle rules.
- **Availability target: 99.99 %** (achievable with AD-level HA within OCI region).
- **RTO: ≤2 min for AD failure; ≤30 min for region failure (manual provider escalation; data preserved cross-AD)**.
- **RPO: ≤1s within AD; up to ≤5 min across AD (sync replication lag)**.

## DR-Pair Posture

For packs with a DR-pair (pack-eu, pack-us, pack-au, pack-in, pack-br, pack-ae, pack-ksa):
- Primary region runs active OpenBao + HSM + Postgres.
- DR region runs warm standby OpenBao + HSM + Postgres replica.
- **Sync replication** between primary + DR Patroni: zero data loss for Postgres backend.
- **HSM in DR region**: each pack has a separate HSM partition in the DR region. KEK material is replicated to the DR HSM partition via the vendor's HSM-side replication protocol (OCI Cloud-HSM and Thales Luna both support this; KEK never exits cleartext).
- **Availability target: 99.99 %** within pack (AD + region resilience).
- **RTO: ≤30 min for region failure (controlled failover; not auto)**.
- **RPO: ≤1s (sync replication)**.

## Cross-Pack: Forbidden

Per `policy/data-residency.md`:
- No cross-pack secret replication.
- No cross-pack KEK movement.
- No cross-pack audit replication (each pack has its own audit-chain instance).

Exceptions: SCC-authorised processing of personal data per Art. 44–46 applies only to *processed personal data*, NOT to raw KEK or secret material.

## Failover Modes

### Mode 1: Pod / Node failure (intra-AD)
- **Trigger**: Kubernetes node failure or pod crash.
- **Mechanism**: kube-scheduler reschedules; OpenBao Raft tolerates ≤2 peer loss out of 5.
- **Detection latency**: <30s.
- **Recovery**: automatic; no human intervention.

### Mode 2: AD failure (intra-region)
- **Trigger**: full AD outage.
- **Mechanism**: OpenBao Raft survives loss of 2 of 5 nodes (1 AD); Patroni promotes replica in a healthy AD.
- **Detection latency**: <60s.
- **Recovery**: automatic; service continues at reduced capacity until AD restored.
- **RTO: ≤2 min**.

### Mode 3: Region failure (cross-region for DR-pair packs)
- **Trigger**: full region outage.
- **Mechanism**: controlled failover to DR region — manual ops-security + axis-cloud-secrets decision; not automatic.
- **Steps**:
  1. Confirm primary region unrecoverable.
  2. Promote DR Postgres to primary.
  3. Promote DR OpenBao Raft cluster.
  4. Update DNS to point to DR endpoints.
  5. Notify consumer µservices (push config update; SDKs re-resolve OpenBao endpoint via service discovery).
- **Detection latency**: depends on outage scope; typically <5 min.
- **Recovery**: manual; **RTO: ≤30 min**.

### Mode 4: HSM partition failure
- **Trigger**: HSM partition unreachable (vendor-side or network).
- **Mechanism**: PKCS#11 client retries against HA partition; OpenBao auto-unseal uses fallback partition.
- **Detection latency**: <60s.
- **Recovery**: automatic (HA partition); manual failover if both partitions affected.
- **RTO: ≤5 min** (HA); ≤24h if both partitions compromised (vendor escalation).

### Mode 5: Audit-chain bridge failure
- **Trigger**: audit-chain µservice degraded.
- **Mechanism**: audit-emitter writes to local durable file; bridges asynchronously.
- **Detection latency**: <60s.
- **Recovery**: local file drains when audit-chain recovers; no data loss (local file durable).

## Failover Drills (cadence)

| Drill | Cadence | Acceptance |
|---|---|---|
| Pod-kill chaos | weekly (automated) | Raft re-elects within 5s |
| AD-failure simulation | monthly | service continues; verdict latency ≤2× normal |
| Region failover (DR-pair packs) | quarterly | manual failover completes within 30 min |
| HSM partition failover | quarterly | automatic to HA partition within 5 min |
| Audit-chain bridge degradation | monthly | local file drains; no audit loss |
| Postgres backup-restore | quarterly | restored cluster reaches healthy in ≤30 min |

## Federation: Not Performed

cloud-secrets does NOT federate across packs:
- No global view of secrets across packs.
- No cross-pack identity federation (each pack's OpenBao is its own authority).
- No cross-pack policy authoring (per-pack policy values in Helm overlays).

Federation, if ever required (open question), would be added under a separate ADR and would require:
- Tenant DPA consent for cross-pack federation.
- Federated identity model (which pack issues the canonical token).
- Federated audit (cross-pack audit-chain reconciliation).
- Regulatory review (Schrems II + per-pack equivalent).

## Verification

```bash
cargo run -p dev-cli -- gate validate multi-region-topology --microservice cloud-secrets
cargo run -p dev-cli -- gate validate dr-pair-conformance --microservice cloud-secrets
cargo run -p dev-cli -- gate validate cross-pack-replication-forbidden --microservice cloud-secrets
```

Annual ops-sre review: verify each active pack's DR posture matches this document.

## References

- ADR-0117 (Cloud-native infrastructure)
- ADR-0131 (Cloud split)
- `microservices/cloud-secrets/policy/data-residency.md`
- `microservices/cloud-secrets/capacity-model.md`
- `microservices/cloud-secrets/incident-response.md`
- `microservices/cloud-secrets/runbooks/openbao-restart.md`
- `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`
- OCI region documentation
- OpenBao Raft consensus model
- Patroni HA documentation

---

## ADR-0158 Multi-Region Disposition Statement

**Disposition: `single_region` per cell (secret material does not cross region).**

Per ADR-0158, the cloud-secrets µservice is declared `single_region`. Secret material never leaves the cell. Cross-region replication of secrets is forbidden by construction (HSM partition is per-region; OpenBao seal key is per-region).

| Property | Value |
|---|---|
| Disposition | `single_region` |
| RPO (intra-cell) | ≤ 5 seconds (OpenBao Raft + Patroni HA) |
| RTO (intra-cell) | ≤ 60 seconds (Raft leader-election) |
| Cross-region replication | FORBIDDEN |
| Failover model | intra-region only (KR-Seoul1 ↔ KR-Chuncheon for `pack-kr`; analogous per pack) |

## ADR-0164 Sovereign Cloud / Air-Gapped Deployment Variant

Per ADR-0164, the cloud-secrets µservice ships a per-pack air-gap variant. The variant ensures:

### No external KMS dependency

- All KMS code paths replaced by OpenBao Transit secrets-engine.
- Per-tenant secret keys live in the in-cell HSM partition only (per ADR-0043).
- The cloud KMS adapter (AWS KMS / GCP KMS / Azure Key Vault) is ABSENT from air-gap pack image builds.

### HSM-backed OpenBao seal

- OpenBao auto-unseal uses the in-cell HSM partition (PKCS#11 interface; HSM choice per pack — Thales Luna in sovereign packs; Marvell LiquidSec in some on-prem packs).
- Recovery keys split per ADR-0043 quorum (Shamir 5-of-9 default; per-pack overlay).
- Air-gap seal recovery requires regulator-witnessed quorum (per `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`).

### encryption-key BYOK + sovereign tenant key custody (ADR-0251 §D-10)

- A sovereign tenant in `pack-ksa` / `pack-uae` / `pack-eu-sovereign` / `pack-us-gov` may bring its own HSM-generated KEK; cloud-secrets accepts the KEK wrapped under the cell's KEK-of-KEKs.
- encryption-key BYOK material (ADR-0251 §D-10) is HSM-stored in the cell HSM partition; never exported.

### Pack matrix (cloud-secrets perspective)

| Pack | `air_gap` | OpenBao | HSM partition |
|---|---|---|---|
| `pack-eu-sovereign-airgap` | true | in-cell | EU-region HSM (Thales Luna) |
| `pack-kr-fsc` | true | in-cell | KR-region HSM (financial-grade) |
| `pack-kr-public` | true | in-cell | KR-region HSM |
| `pack-ksa` | true | in-cell | KSA-region HSM (Thales Luna) |
| `pack-uae` | true | in-cell | UAE-region HSM |
| `pack-us-gov` | true | in-cell | US-Gov HSM (FIPS 140-3 L4) |
| `pack-us-shared` | false | in-cell + AWS KMS adapter | per-cell HSM partition |
| `pack-eu` | false | in-cell | EU-region HSM |
| `pack-kr` | false | in-cell | KR-region HSM |
| `pack-jp` | false | in-cell | JP-region HSM |

### CI gates

CI lane `oya gate validate air-gap-overlay` enforces (a) air-gap packs contain no external KMS adapter binary, (b) OpenBao auto-unseal binds to in-cell HSM (no cloud-KMS unseal path), (c) encryption-key BYOK paths use HSM-wrapped material only (ADR-0251 §D-10).

See `/specs/sovereign-cloud-air-gapped-canonical.json` for the canonical declaration.
