---
microservice: compliance
ip: IP-006
title: Evidence storage on SeaweedFS (per-framework bucket + WORM tier + cold archive)
status: Drafting
authority_tier: 3
owner: axis-compliance
co_owners: [ops-sre-reliability]
date: 2026-05-18
related_adrs: [ADR-0145, ADR-0184, ADR-0209]
---

# IP-006 — Evidence storage on SeaweedFS

## Purpose

Replace the IP-001 in-memory `EvidenceLedger` with a SeaweedFS-backed adapter (per ADR-0145 + ADR-0184). Provide per-framework bucket isolation + WORM (write-once-read-many) hot tier + cold-archive tier with cosign re-seal on tier transition.

## Acceptance criteria

1. `oya-compliance-storage-adapter` crate connects to SeaweedFS filer endpoint (`oya-seaweedfs-filer.shared-storage.svc:8888`).
2. Per-framework bucket: `oya-compliance-evidence-{soc2|gdpr|hipaa|pci}`.
3. Hot tier (0-90 days) WORM-enforced: rejects PUT with same key + different content.
4. Cold tier (90 days - 7 years) gzip-compressed; cosign re-seal on tier transition.
5. Retention enforcement per `policy/retention-tier-policy.json` (drives IP-009).
6. ≥ 6 integration tests: PUT-then-immutable + cold-transition + cold-seal-verify + cross-framework-bucket-isolation + retention-cutoff + WORM-violation-Sev-1.

## Bucket layout

```
oya-compliance-evidence-soc2/
  hot/<year>/<month>/<artifact_id>.json  ← 0-90 days
  cold/<year>/<month>/<artifact_id>.gz   ← 90 days - 7 years
oya-compliance-evidence-gdpr/
  ... same ...
oya-compliance-evidence-hipaa/
  ... same; retention 6 years (HIPAA statutory) ...
oya-compliance-evidence-pci/
  ... same ...
```

## Tier transition

A cron job runs daily:

1. Scan hot tier for artifacts older than 90 days.
2. Gzip + write to cold tier.
3. Cosign re-seal the gzipped blob (new seal hex; chain links old seal → new seal).
4. Verify cold-tier read-back.
5. Delete hot-tier copy.
6. Emit `EVT-EVIDENCE-COLD-ARCHIVED`.

## WORM enforcement

SeaweedFS filer with `worm` flag enabled on hot bucket. PUT returns 409 if key exists. Modification = create-new-version (separate artifact_id); never overwrite.

## Risk + mitigation

- **Risk:** SeaweedFS filer outage stalls evidence emission. **Mitigation:** producer-side buffer queue (24-hour persistence); auto-resume.
- **Risk:** cold-tier re-seal chain breaks (re-seal step fails after gzip). **Mitigation:** transactional 2-phase: gzip + re-seal in temp location → verify → atomic move.

## Acceptance evidence

`evidence/ip-006-evidence-storage-seaweedfs-acceptance.json`.

## Cross-references

- ADR-0145 — SeaweedFS as inter-µservice storage substrate.
- ADR-0184 — storage tier layering.
- ADR-0209 — substrate authority.
- IP-005 — audit-chain seal.
- IP-009 — retention tier policy.
