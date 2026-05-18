---
microservice: compliance
ip: IP-002
title: SOC 2 control mapping (Trust Services Criteria → artifact kinds → emission cadence)
status: Drafting
authority_tier: 3
owner: axis-compliance
date: 2026-05-18
related_adrs: [ADR-0209]
---

# IP-002 — SOC 2 control mapping

## Purpose

Codify the AICPA Trust Services Criteria → required artifact kinds → emission cadence → owning µservice mapping. Drives the per-framework scorecard at `scorecards/soc2-type-2.json` and the `coverage.md` rollup.

## Acceptance criteria

1. `policy/soc2-control-mapping.json` lists each of CC1..CC9, A1, C1, PI1, P1..P8 with:
   - Required artifact kinds.
   - Emission cadence (per-build, per-deploy, weekly, monthly, quarterly, yearly, continuous).
   - Owning µservice (or "fleet-wide" for cross-cutting).
   - Cedar capability fragment (for auditor read).
2. `scorecards/soc2-type-2.json` references the mapping and reports per-control status.
3. Coverage gate (`oya-check-compliance-evidence-coverage`) reads the mapping; gap reports cite the violated criterion.
4. Auditor portal renders the per-criterion control mapping.
5. ≥ 5 integration tests asserting mapping round-trips through `oya-shared-compliance-evidence-kernel`.

## Trust Services Criteria coverage

(Per `compliance.md` matrix — replicated for IP-local reference.)

| Criterion | Description | Required artifact kinds |
|---|---|---|
| CC1 | Control environment | access-review-snapshot |
| CC2 | Communication & information | Runbooks index (informational) |
| CC3 | Risk assessment | vuln-scan-report + pen-test-report |
| CC4 | Monitoring | minimum-necessary-access-log + audit-chain seal coverage |
| CC5 | Control activities | deploy-receipt + ci-artifact-hash |
| CC6 | Logical & physical access | access-review-snapshot |
| CC7 | System operations | backup-restore-drill-receipt |
| CC8 | Change management | deploy-receipt + ci-artifact-hash |
| CC9 | Risk mitigation | vuln-scan-report + pen-test-report |
| A1 | Availability | backup-restore-drill-receipt + SLO burn-down |
| C1 | Confidentiality | access-review-snapshot + audit-chain seal coverage |
| PI1 | Processing integrity | audit-chain seal coverage |
| P1-P8 | Privacy | dsar-completion-record + Cedar policy snapshot |

## Cedar capability fragments

Each criterion maps to a Cedar capability:

```cedar
// capabilities/auditor-soc2-read.cedar
permit (
  principal in Auditor::"current-engagement",
  action in Action::"read-artifact",
  resource in EvidenceArtifact::"soc2-type-2"
) when {
  resource.tenant_id in principal.tenants_in_scope &&
  resource.emitted_unix_ms >= principal.engagement_window_open_unix_ms
};
```

## Risk + mitigation

- **Risk:** Trust Services Criteria revisions (AICPA updates the standard every few years). **Mitigation:** versioned mapping file (`policy/soc2-control-mapping.v2017.json` vs `.v2025.json`); migration ADR per revision.
- **Risk:** drift between mapping file and Cedar capability fragments. **Mitigation:** integration test asserts every criterion in mapping has a matching Cedar fragment.

## Acceptance evidence

`evidence/ip-002-soc2-control-mapping-acceptance.json`.

## Cross-references

- IP-001 — collector bootstrap.
- ADR-0209 — substrate authority.
- AICPA TSC 2017 (with 2022 points of focus update).
