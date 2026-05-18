---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-sites-foundation
impl_plan_id: IP-012-policy-dpia-threat-model
status: pending
execution_unit: ChangeSet
owner: ops-security + council-privacy + axis-sites
acceptance_lanes: [cedar-policy-lint, oya-governance-cedar-enforcement]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: Cedar policy + DPIA + threat-model sign-off

## Intent

Finalise Cedar policies (`tenant-scope.cedar`, `ci-scope.cedar`, `auditor-scope.cedar`, `public-read.cedar`), DPIA, threat-model, compliance.md, multi-region.md, incident-response.md, failure-modes.md, backfill-replay.md. Council-privacy + ops-security sign-off recorded in audit-chain.

## ChangeSet boundary

6 Cedar policy files + 9 markdown policy/governance files. No Rust code.

## Acceptance Gates

```bash
cedar validate microservices/sites/policy/tenant-scope.cedar
cedar validate microservices/sites/policy/ci-scope.cedar
cedar validate microservices/sites/policy/auditor-scope.cedar
cedar validate microservices/sites/policy/public-read.cedar
cargo run -p oya-dev-cli -- gate validate cedar-enforcement --microservice sites
cargo run -p oya-dev-cli -- gate validate dpia-completeness --microservice sites
cargo run -p oya-dev-cli -- gate validate threat-model-completeness --microservice sites
```

## ChangeSet metadata

```yaml
changeset_id: CS-SITES-IP-012-policy-dpia-threat-model
depends_on_changesets: [CS-SITES-IP-003-site-and-page-bcs]
parallel_safe_with_changesets: [CS-SITES-IP-013-contracts-and-capabilities, CS-SITES-IP-014-dashboards-runbooks-slos]
enables: [CS-SITES-IP-015-hg-sites-maturity-claim]
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | `tenant-scope.cedar` validates and refuses unguarded action surface | `cedar validate microservices/sites/policy/tenant-scope.cedar` |
| AC-02 | `ci-scope.cedar` / `auditor-scope.cedar` / `public-read.cedar` validate | `cedar validate` on each |
| AC-03 | DPIA covers all 11 BCs + 11 packs + rights-of-data-subject sections | `cargo run -p oya-dev-cli -- gate validate dpia-completeness --microservice sites` |
| AC-04 | Threat-model enumerates STRIDE per BC + each mitigation cross-referenced | `cargo run -p oya-dev-cli -- gate validate threat-model-completeness --microservice sites` |
| AC-05 | Council-privacy + ops-security sign-off recorded in audit-chain | manual + audit-chain seal |

## Build Sequence

1. Author `policy/*.cedar` (4 policies); lint via `cedar validate`.
2. Author `compliance.md`, `multi-region.md`, `incident-response.md`, `failure-modes.md`, `backfill-replay.md`, DPIA, threat-model.
3. Run `oya gate validate cedar-enforcement --microservice sites`.
4. Run `dpia-completeness` + `threat-model-completeness` gates.
5. Council-privacy + ops-security review; audit-chain seal.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-sites NFR | Security, Audit + Compliance, Data residency |
| PRD-sites AC | AC-12, AC-13, AC-14 |
| ADR | ADR-0140 (Cedar pack overlays), ADR-SITES-0007 |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Cedar policy regression silently broadens access | Coverage gate refuses unguarded action surface; CI lane |
| DPIA goes stale as new BCs land | Per-BC DPIA section enforced by `dpia-completeness` gate |
| Threat-model misses cross-µservice trust boundary | STRIDE-per-trust-boundary enumeration; covered by gate |

## References

- ADR-0140 (Cedar policy + pack overlays).
- Cedar Policy Language reference (`docs.cedarpolicy.com`).
- ICO DPIA template (UK Information Commissioner's Office — "Data Protection Impact Assessments").
- Microsoft STRIDE threat-modelling guidance ("Threat modeling tool 2016").
- OWASP Application Security Verification Standard (ASVS) v4.
- EU DSA Arts. 14 + 27 (transparency).
- agent-skills documentation-and-adrs SKILL.md.
- agent-skills security-and-hardening SKILL.md.
