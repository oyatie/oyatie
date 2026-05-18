---
id: ADR-ANON-0007
status: Accepted
date: 2026-05-17
microservice: anonymous
deciders: axis-anonymous, council-privacy, council-architecture
owner: axis-anonymous + council-privacy
supersedes: []
superseded_by: []
related:
  - ADR-ANON-0002
  - ADR-ANON-0006
related_artifacts:
  - microservices/anonymous/PRD.md (FR-16, FR-21, FR-22, FR-23)
  - microservices/anonymous/runbooks/geo-affinity-cluster-rebalance.md
  - microservices/anonymous/runbooks/employer-affinity-employer-domain-takeover.md
purpose: |
  Define the cardinality + minimum-population (k-anonymity) floor for affinity
  clusters (geo / employer / university / workspace / industry) and the
  hierarchical anonymisation-fallback procedure when a cluster falls below
  the floor.
---

# ADR-ANON-0007: Affinity-cluster design — k=50 geo / k=20 employer / k=10 small-employer fallback; hierarchical anonymisation-fallback per Sweeney 2002 k-anonymity

## Status

Accepted — 2026-05-17.

## Context

Affinity clusters (employer / university / geographic region / workspace / industry) are the unit of feed-scoping in the anonymous µservice. PRD invariant I2 (affinity-not-identity) requires that affinity attestation reveal the affinity but not the identity. But there is an emergent failure mode: if a cluster has too few members, "I am a Bominal employee" can effectively reveal "I am Jane Smith" (the only Bominal employee on the platform).

This is the k-anonymity problem (Sweeney 2002). The decision:

1. **What k-floor for each cluster kind?**
2. **What happens when a cluster falls below floor?**
3. **How do we handle small employers / sparse geographic regions?**
4. **Hierarchical fallback or refusal?**

Industry precedent:

- **Blind**: permits single-employee verification (k=1 effectively); this is documented as a Blind weakness in `competitor-parity-matrix.md`.
- **Fishbowl**: industry-bound; k floor is large by construction (whole industry).
- **YikYak (early)**: 5-mile geo radius, sometimes k=1 on small campuses; harassment cascade was the consequence.
- **Sidechat**: university-bound; k floor varies by university size.
- **Jodel**: hyperlocal geo; k floor varies.

Academic anchor: **Sweeney L. (2002), "k-anonymity: A model for protecting privacy"**, IEEE Trans. Knowledge & Data Engineering. The foundational work; recommends k ≥ 5 for any release; larger k for higher-sensitivity contexts.

## Decision

Adopt **per-cluster-kind k-floors** with **hierarchical anonymisation-fallback**:

### k-floors

| Cluster kind | k-floor | Rationale |
|---|---|---|
| Geographic | **k=50** | Higher floor for geo because geo + post-pattern fingerprinting is strong (Sweeney's classic re-identification result); larger floor needed for safety |
| Employer | **k=20** | Lower floor than geo because employer cluster has smaller universe (employer-attestation is rarer event); 20 is the Blind-class minimum for safety |
| University | **k=20** | Same as employer; small universities are handled via fallback |
| Workspace (tenant-internal) | **k=20** | Same |
| Industry | **k=50** | Larger universe; can sustain larger floor |
| Small-employer fallback | **k=10** | Only acceptable with anonymisation-fallback (merge into industry or geo) |

### Hierarchical anonymisation-fallback

When a cluster falls below its k-floor (or never reaches it), it is **merged into a parent cluster** in a hierarchy:

```
Geographic:    locality → metro → state/province → country → "global" (last resort)
Employer:      employer → industry → "all-employed" (last resort)
University:    department → university → "all-students-in-region" (last resort)
Workspace:     workspace-team → workspace-org → "all-workspace-members"
Industry:      industry-niche → industry → "all-employed"
```

The merge is **mandatory at the k-floor boundary**; members of the sub-floor cluster are migrated to the parent cluster. The migration is a credential re-issuance (BBS+ re-credential per ADR-ANON-0001) with audit-chain seal.

### Cluster-creation refusal

A tenant operator cannot create a cluster with cardinality below the k-floor at creation time. New clusters must reach k-floor (via member-binding) before posts can be published. The verifier endpoint refuses bindings into a sub-floor cluster.

### Cluster rebalance procedure

Per `runbooks/geo-affinity-cluster-rebalance.md`:

- **Planned (Sev-3)**: 14-day notice + member-side migration option
- **Emergency (Sev-2; k=10)**: 24-hour notice + auto-merge
- **k=5 emergency (Sev-1)**: immediate pause + immediate auto-merge to grandparent

### Cluster cardinality monitoring

Per `dashboards/anonymity-health.json` + Prometheus alert `AffinityClusterCardinalityBelowFloor` (Sev-2): any cluster with cardinality < floor fires alert.

## Alternatives Considered

### A. k=5 universal (Sweeney baseline)

- **Pros**: Sweeney's recommended floor; conservative.
- **Cons**: Too low for geographic clusters where geo + post-pattern fingerprinting is strong; insufficient for affinity-bound posting where the attestation itself carries information.
- **Rejected because**: Sweeney's k=5 is a minimum, not a recommendation for high-sensitivity contexts.

### B. k=100 universal

- **Pros**: Maximum safety.
- **Cons**: Excludes too many small employers + universities + rural regions; product utility regression.
- **Rejected because**: Disproportionate to risk; hierarchical fallback achieves similar safety with better product utility.

### C. No floor (let tenants choose)

- **Pros**: Tenant flexibility.
- **Cons**: Defeats the privacy promise; tenants cannot reason about k-anonymity; Blind-precedent failure.
- **Rejected because**: Privacy floor is a structural promise, not a tenant configuration.

### D. Per-tenant floor (tenant can opt above default)

- **Pros**: Tenant flexibility while keeping baseline.
- **Cons**: Useful in theory; rarely exercised; complicates configuration.
- **Rejected because**: Default + fallback is sufficient; tenant-tunable floor can be added later if needed.

### E. Floor by absolute count (e.g., k=20 in all sizes) without hierarchical fallback

- **Pros**: Simpler.
- **Cons**: Small clusters (small employers, rural regions, small departments) cannot exist at all → product utility regression.
- **Rejected because**: Hierarchical fallback is the elegant resolution.

### F. No hierarchical fallback (refuse below-floor clusters)

- **Pros**: Simpler conceptually.
- **Cons**: Small clusters cannot exist; members of small clusters cannot use the platform; product reach regression.
- **Rejected because**: Hierarchical merge is the canonical k-anonymity-with-utility approach.

## Consequences

### Positive

- **I2 invariant flowed through k-floor.** "Affinity-not-identity" holds because affinity cluster size meets the k-floor.
- **Product reach** preserved via hierarchical fallback (small employers + sparse regions still usable through parent cluster).
- **Academic alignment** with Sweeney 2002; defensible to auditors.
- **Operational clarity** via 3-tier severity (planned / emergency / Sev-1 immediate).

### Negative

- **Forced cluster merges** can frustrate small-employer / sparse-region members. Mitigated: 14-day notice; member-side opt-out; per `runbooks/geo-affinity-cluster-rebalance.md`.
- **Cluster cardinality monitoring** is operational overhead. Mitigated: Prometheus alerting + dashboard.
- **Cross-cluster post-merging on rebalance** creates audit-trail complexity. Mitigated: every merge sealed to audit-chain.

### Operational

- IP-006 (affinity-attestation BC) implements floor enforcement.
- `runbooks/geo-affinity-cluster-rebalance.md` documents the rebalance.
- `runbooks/employer-affinity-employer-domain-takeover.md` documents employer-side change-of-ownership.
- LEAN lane `oya-check-k-anonymity-floor` verifies no binding into sub-floor cluster.
- Quarterly review: cluster cardinality distribution + rebalance frequency.

### Regulatory

- **GDPR Recital 26 + Art. 11** pseudonymisation: k-anonymity floor is the canonical pseudonymisation pattern for sparse-data scenarios.
- **GDPR Art. 25 privacy-by-design**: floor + fallback is the design.
- **KR PIPA Art. 24-2** alternative-pseudonymous-processing: floor + fallback is part of the alternative-identifier scheme.
- **APPI Art. 18**: purpose-limitation respected.

### Invariant Preservation

I2 structurally satisfied; I1 protected from emergent re-identification.

## References

- Sweeney, L. (2002). "k-anonymity: A model for protecting privacy". International Journal of Uncertainty, Fuzziness and Knowledge-Based Systems.
- Sweeney, L. (2000). "Simple demographics often identify people uniquely". Carnegie Mellon University.
- Machanavajjhala, A., Kifer, D., Gehrke, J., Venkitasubramaniam, M. (2007). "l-diversity: Privacy beyond k-anonymity". ACM TKDD.
- ADR-ANON-0002 (affinity-attestation — feeds clusters)
- ADR-ANON-0006 (federation refusal — hashtag corpus federation would defeat k-anonymity)
- GDPR Recital 26 + Art. 25
- KR PIPA Art. 24-2
- NUTS classification (EU regional units used in geo-hierarchy)
- `microservices/anonymous/competitor-parity-matrix.md` (Blind / YikYak / Burnbook anti-patterns)
