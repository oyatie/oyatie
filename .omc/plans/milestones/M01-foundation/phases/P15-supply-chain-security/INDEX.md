---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M01-P15
title: Supply-Chain Security (Cosign + Rekor + SLSA + SBOM)
status: complete
purpose: Every artifact Cosign-signed + Rekor-anchored; SBOM per build; SLSA level published.
---

# M01-P15 — Supply-Chain Security

## Purpose
Per MASTERPLAN §2 Directives 3 (final-shape from day one — supply-chain rigor cannot be retrofitted), 9 (hyperscaler bar), and per ADR-0039.

## Acceptance
- `oya-governance-supply-chain` lane CI-blocks: any artifact not Cosign-signed + Rekor-anchored.
- SBOM (CycloneDX or SPDX) at `releases/<tag>/sbom.json`.
- SLSA level ≥ 3 published per release.
- `oya-governance-license-policy` lane CI-blocks AGPL/GPL/SSPL/BUSL/RSAL in product code.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Cosign + Rekor signing pipeline | complete | [`IP-001-cosign-rekor.md`](IP-001-cosign-rekor.md) |
| IP-002 | SBOM generation per build (CycloneDX / SPDX) | complete | [`IP-002-sbom-pipeline.md`](IP-002-sbom-pipeline.md) |
| IP-003 | License-policy lane (AGPL/GPL/SSPL/BUSL/RSAL hard-deny) | complete | [`IP-003-license-policy-lane.md`](IP-003-license-policy-lane.md) |
| IP-004 | SLSA level ≥3 attestation publishing | complete | [`IP-004-slsa-attestation.md`](IP-004-slsa-attestation.md) |

## Estimated parallelism
4 agents.

## Symbols-touched
`crates/oya-governance-{supply-chain,license-policy}-kernel`, `.github/workflows/cosign.yml`, `.github/workflows/sbom.yml`, `.github/workflows/slsa.yml`.

## Agent-handoff
```
icm store -t context-oyatie -c "M01-P15 complete: Cosign + Rekor + SBOM + SLSA-3 + license-policy lane green" -i critical -k "M-CC,P08,supply-chain,complete"
```
