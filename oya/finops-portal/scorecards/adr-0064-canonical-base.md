---
scorecard_id: finops-portal/adr-0064-canonical-base
authored: 2026-05-18
authority: ADR-0064 canonical-base + localization (seam/adapter/pack)
status: ready
---

# Scorecard — ADR-0064 Canonical-base + localization

ADR-0064 mandates that every µservice ships a canonical global base
plus regulatory-pack overlays via seam OR adapter OR pack patterns,
and that canonical-base neutrality is CI-enforced.

## Compliance evidence

| Criterion                                                  | Status | Evidence                                            |
|-----------------------------------------------------------|--------|-----------------------------------------------------|
| Canonical base exists                                      | ✓      | `iac/helm/finops-portal/values.yaml`                |
| Base is regulatory-pack neutral                            | ✓      | `costAttribution.regulatoryPack: generic` (default) |
| KR pack overlay present                                    | ✓      | `iac/helm/finops-portal/values-kr.yaml`             |
| EU pack overlay present                                    | ✓      | `iac/helm/finops-portal/values-eu.yaml`             |
| US-healthcare pack overlay present                         | ✓      | `iac/helm/finops-portal/values-us-healthcare.yaml`  |
| Seam pattern used for regulator-evidence emit              | ✓      | IP-015 + Cedar `regulator-evidence-emit.cedar`      |
| Adapter pattern used for FOCUS export storage              | ✓      | `oya-finops-portal-focus-export-adapter-seaweedfs`  |
| Pack pattern used for compliance overlays                  | ✓      | Cedar policies use `regulatory_pack` context field  |
| No hardcoded pack-specific logic in base                   | ✓      | Audited; only `features.*` toggles, no hardcoded   |
| Cedar policies use `context.regulatory_pack` discriminator | ✓      | 4 policy files at `policy/cedar/`                   |
| CI gate `oya-check-canonical-base-neutrality` enforced     | pending| Gate exists in workspace; wire-up planned per IP-003|

## Citations

- `iac/helm/finops-portal/Chart.yaml` — references `_oya-helpers` per
  canonical-base library.
- `policy/cedar/*.cedar` — every `permit` clause carries the
  `context.regulatory_pack == resource.regulatory_pack` discriminator.
- `compliance-matrix.md` — per-pack obligation tracker.

## Gaps + remediation

- **Gap**: CI gate `oya-check-canonical-base-neutrality` not yet wired
  for this µservice. **Remediation**: add the gate config in the next
  IP-003 follow-up.

## Verdict

**PASS** (with one tracked gap on CI gate wire-up).

## References

- ADR-0064 canonical-base + localization.
- `multi-region-strategy.md`.
- `compliance-matrix.md`.
