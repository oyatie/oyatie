---
id: ADR-0038
status: Accepted
doc_status: published
---

# ADR-0038: Trust framework — cross-microservice lineage, DSR cascade across all all microservices, Cosign-signed proof-of-erasure, tenant trust portal

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `council-architecture`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0003, ADR-0007, ADR-0008, ADR-0028, ADR-0029, ADR-0030, ADR-0031, ADR-0033, ADR-0034, ADR-0039, ADR-0043, ADR-0049

---

## Context

The cohesion thesis (ADR-0001) commits us to *one* audit chain, *one* identity surface, *one* consent store. The Data Use Boundary (ADR-0008) commits us to enforced cross-microservice flow gating. The per-vertical override pack (ADR-0034) commits us to hard-deny floors. None of those commitments are visible to tenants unless we expose them — and none survive a regulator request unless we can prove cross-microservice erasure on demand.

The DUBO § 2.2.9 (referenced from PRD/DESIGN) defines the **DSR cascade** model: a Data Subject Request (PIPA Art 36 correction/deletion, GDPR Art 17 erasure, CCPA delete, CPRA correction) must propagate across SaaS / Workspace / Vertical / Foundry / Cloud / Search / Ads / Analytics — and produce a **proof-of-erasure record per affected store**, signed by Cosign. The pack-of-19 foundation ADRs decided this in principle but did not pin the cascade mechanism, the SLA, the proof shape, or the tenant-visible trust portal. This ADR pins all four.

---

## Decision

We adopt the **trust framework** as the cross-microservice lineage + DSR cascade + proof-of-erasure spine. Per-tenant trust portal is the customer-visible surface; cross-microservice lineage tracks per-data-class flow across all all microservices; DSR cascade walks the lineage on every request; per-store proof-of-erasure is Cosign-signed and audit-chained.

### Cross-axis trust framework

```rust
// crates/oya-identity-trust-framework-kernel
pub struct TrustFramework {
    pub lineage: CrossAxisLineageGraph,
    pub dsr_orchestrator: DsrOrchestrator,
    pub proof_emitter: ProofOfErasureEmitter,
    pub trust_portal: TrustPortalSurface,
}

pub struct CrossAxisLineageGraph {
    /// Per data class, the set of axes/stores currently holding it for a tenant
    pub class_to_stores: BTreeMap<(TenantId, DataClass), BTreeSet<StoreRef>>,
}

pub enum StoreRef {
    SaasTenantTable { table: String, cell: CellId },
    WorkspaceMail { mailbox: MailboxId, cell: CellId },
    WorkspaceDriveObject { object: ObjectId, cell: CellId },
    VerticalRecord { vertical: VerticalId, record: RecordId, cell: CellId },
    FoundryAgentMemory { agent: AgentId, cell: CellId },
    SearchIndex { index: IndexId, cell: CellId },
    AdsAttribution { campaign: CampaignId, cell: CellId },
    AnalyticsWarehouse { warehouse: WarehouseId, cell: CellId },
    /* ... */
}
```

The lineage is updated on every write that creates or moves data of a tracked class. The lineage is itself stored per-tenant per-cell with replication via the audit chain (ADR-0003) so it survives single-cell failure.

### DSR cascade per Data Use Boundary § 2.2.9

A DSR is initiated via the trust portal (or per-tenant API endpoint). The orchestrator:

1. Resolves the data subject identity → set of (tenant_id, data_class) tuples.
2. For each tuple, walks the lineage graph → set of `StoreRef`.
3. For each `StoreRef`, dispatches a per-microservice erase / correct / export action via the capability registry (ADR-0011).
4. Awaits per-store proof-of-erasure (or proof-of-correction / proof-of-export).
5. Aggregates proofs → Cosign-signed DSR completion record → audit chain.
6. Notifies the data subject + tenant DPO via Workspace mail (ADR-0029).

### SLA per stability tier

| Tier | DSR cascade SLA |
|---|---|
| Preview | 30 days |
| Stable | 14 days |
| GA | 7 days |

The SLA reflects the maturity of the per-microservice erase implementations; GA-tier means every axis has a measured, audited erase path.

### Per-store proof-of-erasure

```rust
pub struct ProofOfErasure {
    pub store: StoreRef,
    pub data_class: DataClass,
    pub subject_id: SubjectId,
    pub erase_method: EraseMethod,   // KMS-shred / record-delete / index-rebuild / cold-storage-purge
    pub erased_at: DateTime<Utc>,
    pub witness: WitnessIdentity,    // operator / system
    pub evidence_hash: Sha256,       // hash of pre-erase + post-erase state
    pub signature: CosignSignature,  // ADR-0039 keyless signing
    pub rekor_log_index: u64,        // ADR-0039 Rekor reference
}
```

A proof is emitted **per affected store**. A single DSR may produce dozens of proofs (one per Workspace mail mailbox, per Drive object, per Vertical record, per Search index shard, per Analytics warehouse partition).

### Trust portal access for tenants

The trust portal is a Workspace-rendered surface (`crates/oya-workspace-trust-portal-*`) that exposes per-tenant:

- **Lineage view.** Which data classes flow to which stores, today.
- **DSR queue.** Open DSRs, in-progress, completed.
- **Proof archive.** All historical proofs of erasure / correction / export, downloadable.
- **API stability mirror** (per ADR-0037).
- **Per-axis SLA + recent uptime** (per ADR-0042).
- **Override pack view** (per ADR-0034) — which classes are hard-denied for this tenant's vertical.
- **Consent receipt archive** (per ADR-0008).
- **Sub-processor list** (per ADR-0028 cloud + per third-party adapter).
- **Per-residency class declaration** (per ADR-0049).
- **Plugin trust tier matrix** (per ADR-0036).

### Per-DSR audit-chain emission

Every DSR step emits to the audit chain (ADR-0003): submission, per-store dispatch, per-store proof receipt, completion, notification.

### DSR types covered

- **Erase / Right-to-be-forgotten** (PIPA Art 36, GDPR Art 17, CCPA Delete).
- **Correct / Rectification** (PIPA Art 36, GDPR Art 16, CCPA Correct).
- **Export / Right-to-portability** (GDPR Art 20, CCPA Right-to-know structured).
- **Restrict / Right-to-restrict** (GDPR Art 18).
- **Object / Right-to-object** (GDPR Art 21).
- **Automated-decision opt-out** (PIPA Art 22-2, GDPR Art 22).

### Cross-axis erase semantics

- **SaaS tenant tables.** Row-level delete + WAL purge on schedule.
- **Workspace mail.** Per-message KMS-shred (ADR-0029); per-mailbox key rotation eventually purges.
- **Workspace Drive.** Per-object KMS-shred (ADR-0029).
- **Workspace Meet recordings.** Per-recording KMS-shred (ADR-0029).
- **Vertical records.** Per-record delete + audit-chain seal (the audit chain itself is append-only; the seal records the fact of erasure but cannot be retroactively redacted).
- **Foundry agent memory.** Per-agent memory rebuild without subject content (per ADR-0007 cross-session memory).
- **Search indexes.** Per-shard rebuild excluding the subject (scheduled rebuild; index updated immediately to suppress).
- **Ads attribution.** Per-subject attribution record purge; campaign aggregates regenerated.
- **Analytics warehouse.** Per-subject row delete + per-cohort recount.
- **Cloud cells.** Per-cell ephemeral state purge; per-cell HSM partition key rotation if applicable.

### Sub-processor disclosure

Per-tenant per-region sub-processor list (per ADR-0028 cloud trajectory: OCI / AWS / KT / LG U+ / Equinix / per-vendor BMS adapters / per-vendor SaaS adapters). Sub-processor changes notified to tenant in advance (per regional pack — KR PIPA Art 26 mandates 7-day advance notice).

### Anti-scope

Trust framework does not own per-class data definition (per ADR-0008 DUBO). Does not own per-vertical hard-deny (per ADR-0034). Does not own audit chain primitives (per ADR-0003). Does not own signing chain (per ADR-0039).

---

## Consequences

### Positive

- DSR completion is mechanical and auditable: cascade is the same regardless of which axis holds the data.
- Cosign-signed per-store proof-of-erasure is the strongest credibly-defensible posture against PIPA Art 39 enforcement / GDPR Art 83 fines.
- Trust portal is the single customer-facing surface for *all* trust-relevant claims; we never have to send tenants on a hunt across multiple consoles.
- Cross-axis lineage updates also support DUBO enforcement (ADR-0008) — the same graph answers "where can this class flow" and "where does this class live".

### Negative

- Lineage maintenance has overhead per write; the SaaS / Vertical hot paths must batch updates.
- Per-store proof-of-erasure is per-store, which means dozens of proofs per DSR; the proof archive grows quickly.
- Per-axis erase implementations are heavy engineering investments — Search index rebuild, Analytics cohort recount, Foundry memory rebuild are all non-trivial.
- 7-day GA SLA is ambitious; any per-microservice regression slips the SLA.

### Operational

- Trust portal SLA: 99.99% (it is the regulator-facing surface).
- Per-DSR cascade SLA dashboard; per-microservice breakdown of latency contribution.
- Per-quarter DSR completion audit by external auditor (per ISO 27701 / SOC 2 alignment).
- Per-axis erase regression test set runs nightly with synthetic DSRs.
- Sub-processor list change notification ships via Workspace mail per ADR-0029.
- Per-region regulator-facing report (KR PIPA, EU GDPR, CA CCPA) generated quarterly from trust portal.

---

## Alternatives considered

### Alternative A — Per-axis DSR endpoint, no cascade orchestrator

- **Pros:** simpler per-microservice implementation.
- **Cons:** tenants chase the DSR across N axes; per-microservice SLA drift; regulator sees fragmented compliance posture.
- **Rejected because:** the cascade is the trust moat.

### Alternative B — DSR cascade in a queue with no per-store proof

- **Pros:** simpler proof model.
- **Cons:** "trust us, it's done" is not an audit-chain claim; regulators routinely demand per-store evidence.
- **Rejected because:** the proof is the differentiator.

### Alternative C — Trust portal as per-microservice dashboard (no unified portal)

- **Pros:** microservice-team independence.
- **Cons:** customer-facing fragmentation; the cohesion-thesis-promise becomes invisible.
- **Rejected because:** the portal is the cohesion-thesis customer-facing artifact.

### Alternative D — Defer trust portal to W+12 (DSR API only at GA)

- **Pros:** lighter day-1 build.
- **Cons:** KR PIPA Art 38 (data subject rights notice) requires customer-visible surface; without portal we ship a worse compliance posture than competitors.
- **Rejected because:** the portal is gating for KR launch.

---

## Open questions

1. **Q1.** Lineage graph update cadence — synchronous on write or async batched? Default: async batched (per-second flush) for hot paths; synchronous for sensitive classes (PHI / PCI / minor-subject). → ADR-0034.
2. **Q2.** Trust portal localization at GA — KR + EN only or full multi-locale? Default: KR + EN at GA; JP + ZH + others at W+12. → ADR-0029.
3. **Q3.** Per-DSR cost model — bundled in tenant subscription or per-DSR fee? Default: bundled at GA; revisit if abuse pattern emerges. → ADR-0028.
4. **Q4.** Sub-processor change notification UX — email + portal or portal only? Default: email + portal (PIPA mandates writing). → ADR-0029.
5. **Q5.** Proof archive retention — per-tenant lifetime or capped (e.g. 7 years)? Default: per-tenant lifetime + per-region statutory minimum overlay. → ADR-0049.

---

## References

- `docs/PRD.md` §11 (data use boundary), §11 (DSR cascade)
- `docs/DESIGN.md` §11 (trust framework), §11 (cross-microservice contradictions), §10 (cross-microservice contracts)
- KR 「개인정보보호법」 Art 26 (sub-processor notice), Art 36 (correction/deletion), Art 38 (data subject rights notice), Art 39 (penalties)
- EU GDPR Art 16, 17, 18, 20, 21, 22, 26, 28, 30, 33, 34, 83
- US: CCPA / CPRA (Cal. Civ. Code §1798.105 et seq); HIPAA Privacy Rule
- ISO 27701; SOC 2 Type II; ISO 27018
- Sigstore Cosign + Rekor specs
- ADR-0001 (cohesion), ADR-0003 (audit), ADR-0007 (Cedar + persona tier), ADR-0008 (DUBO), ADR-0028 (cloud), ADR-0029 (workspace), ADR-0030 (search), ADR-0031 (ads), ADR-0033 (vertical pack), ADR-0034 (per-vertical override), ADR-0037 (API stability), ADR-0039 (supply chain), ADR-0042 (observability), ADR-0043 (HSM + KMS), ADR-0049 (residency)
