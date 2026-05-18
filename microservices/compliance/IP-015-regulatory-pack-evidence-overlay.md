---
microservice: compliance
ip: IP-015
title: Regulatory pack evidence overlay (per-pack additional framework + artifact requirements)
status: Drafting
authority_tier: 3
owner: axis-compliance
co_owners: [axis-regional-pack]
date: 2026-05-18
related_adrs: [ADR-0064, ADR-0179, ADR-0209]
---

# IP-015 — Regulatory pack evidence overlay

## Purpose

Per ADR-0064 + ADR-0179, every µservice ships a canonical-base + per-regional-pack overlay. For compliance, each pack adds jurisdiction-specific frameworks + artifact kinds:

- **KR pack** — Personal Information Protection Act (PIPA); Financial Services Commission electronic-financial-supervisory regulation.
- **UAE / SA pack** — PDPL (UAE Federal Decree-Law No. 45 of 2021) + Saudi Personal Data Protection Law.
- **EU pack** — GDPR is canonical, but EU pack adds EU AI Act high-risk evidence (per ADR-0118).
- **US-Federal pack** — FedRAMP Moderate / FedRAMP High (when government-tenant demand drives).
- **JP pack** — APPI (Act on the Protection of Personal Information).

## Acceptance criteria

1. `microservices/compliance/packs/<pack>/manifest.json` declares pack's additional frameworks + artifact kinds + collectors.
2. Pack overlay merges with canonical at runtime (kernel + domain pick up overlay).
3. Per-pack auditor portal filter renders pack-specific frameworks.
4. ≥ 5 integration tests: KR-pack-pipa-coverage + UAE-pack-pdpl-coverage + EU-pack-ai-act-coverage + pack-non-overlap + canonical-still-passes.

## KR pack — example

```json
{
  "pack": "kr",
  "additional_frameworks": ["pipa", "fsc-efs-supervisory"],
  "additional_artifact_kinds": [
    "pipa-data-broker-registration-receipt",
    "fsc-efs-quarterly-self-audit"
  ],
  "additional_collectors": [
    {"kind": "pipa-data-broker-registration-receipt", "cadence": "yearly", "manual_upload": true}
  ]
}
```

## UAE / SA pack — example

```json
{
  "pack": "uae-sa",
  "additional_frameworks": ["uae-pdpl", "saudi-pdpl"],
  "additional_artifact_kinds": [
    "uae-data-office-registration-receipt",
    "saudi-sdaia-controller-registration"
  ]
}
```

## Risk + mitigation

- **Risk:** pack overlay drift (canonical changes, pack misses). **Mitigation:** quarterly pack-canonical-diff review; advisory gate flags missing pack updates.
- **Risk:** jurisdiction frameworks change (new laws). **Mitigation:** per-pack quarterly legal-review cadence; ADR-tracked changes.

## Acceptance evidence

`evidence/ip-015-regulatory-pack-evidence-overlay-acceptance.json`.

## Cross-references

- ADR-0064 — canonical base + localization.
- ADR-0179 — sovereign cloud per-regional pack.
- ADR-0118 — EU AI Act Annex III refusal.
- ADR-0209 — substrate authority.
- IP-002 — SOC 2 control mapping (canonical analogue).
