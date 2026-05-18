---
doc_class: PhasePlan
template_id: TPL-PHASE
milestone: M02-foundation
phase: P01-recordings-foundation
status: pending
owner_team: axis-recordings
related_adrs: [ADR-0130, ADR-0131, ADR-0132, ADR-0133, ADR-RECORDINGS-0001, ADR-RECORDINGS-0002, ADR-RECORDINGS-0003, ADR-RECORDINGS-0004, ADR-RECORDINGS-0005, ADR-RECORDINGS-0006, ADR-RECORDINGS-0007]
related_artifacts:
  - microservices/recordings/PRD.md
  - microservices/recordings/multi-region.md
  - microservices/recordings/capacity-model.md
  - microservices/recordings/IP-001-iac-bootstrap.md
date: 2026-05-17
doc_status: published
---

# PHASE-01: recordings µservice foundation

## Intent

Ship the recordings µservice from zero to **HG-RECORDINGS gate green at p99
SLOs sustained 30d** across dev → staging → production, with parallel
Strangler migration of the legacy `oya-connect-recordings-domain` crate.

The phase is the canonical M02 instance of the per-µservice agentic SLO-gated
promotion pattern (ADR-0130) + per-µservice flat layout (ADR-0131) +
industry-best-practice conformance (ADR-0133) + Connect dissolution Strangler
migration (ADR-0134).

## ChangeSet sequencing

Per ADR-0110 changeset state-machine + ADR-0111 merge queue. Each IP below is
a single claimable-verifiable-bundleable-promotable ChangeSet.

| # | ChangeSet | Depends on | Reviewer-agent verdict required |
|---|---|---|---|
| IP-001 | IaC bootstrap (Helm + Kustomize + Terraform) | — | governance-iac-conformance |
| IP-002 | Cargo workspace bootstrap (22 BC crate families, kernel layer) | IP-001 | layer-correctness + port-location |
| IP-003 | Recording-ingest BC: kernel + domain + usecase + ingest contract | IP-002 | port-location + lean-a1 |
| IP-004 | Recording BC: kernel + domain + usecase + REST (read-side) | IP-003 | port-location + lean-a1 |
| IP-005 | Media-segment BC: HLS multi-bitrate adapter-ffmpeg | IP-004 | layer-correctness |
| IP-006 | Transcript BC: Whisper + pyannote adapter (foundry-runtime gVisor) | IP-005 | lean-a1 + lean-a2 |
| IP-007 | Search BC: Meilisearch adapter + transcript indexing | IP-006 | shardability + statelessness |
| IP-008 | Redaction BC: overlay model (no source mutation) | IP-006 | port-location + ADR-RECORDINGS-0003 conformance |
| IP-009 | Chapter-marker BC + summary BC | IP-006 | port-location |
| IP-010 | Retention-policy BC + legal-hold BC (load-bearing) | IP-004 | retention-policy-correctness + legal-hold-chain-of-custody-correctness |
| IP-011 | Playback BC + share-link BC + watermarking BC | IP-005 | port-location |
| IP-012 | Export BC + eDiscovery BC | IP-010 | port-location + audit-chain-integrity |
| IP-013 | Translation BC (cross-µservice → translate) | IP-006 | lean-a2 (cross-product through Workflow) |
| IP-014 | Strangler migration adapter shim (oya-connect-recordings-*) | IP-004 | strangler-conformance |
| IP-015 | HG-RECORDINGS authority-cohesion gate registration | IP-002..IP-014 | authority-cohesion |

## Exit criteria

- All 14 CI lanes green (lean-a1..a10 + port-location + layer-correctness +
  per-microservice-layout + statelessness + shardability +
  authority-cohesion + retention-policy-correctness +
  legal-hold-chain-of-custody-correctness).
- HG-RECORDINGS authority-cohesion gate accepts at p99 SLOs sustained 30d
  per ADR-0135 retirement trigger.
- Phase-3 canary (10 % → 50 % → 100 % traffic) on the Strangler adapter
  reaches 100 % traffic on the new µservice sustained 7d.
- All 10 SLOs publishing burn-rate to promotion ledger per ADR-0130.

## Halt conditions

- Any LEAN-A lane regression — block; investigate.
- Whisper / pyannote / ffmpeg upstream CVE — block; sunset to next pinned LTS.
- Legal-hold engagement SLO breach — Sev-1; rollback last ChangeSet; engage
  council-privacy + ops-compliance.
- Retention-policy correctness < 100 % — Sev-1; rollback last ChangeSet;
  engage council-privacy + ops-compliance + ops-security.

## References

- ADR-0130, ADR-0131, ADR-0132, ADR-0133, ADR-0134.
- ADR-RECORDINGS-0001..0007.
- PRD.md.
- IP-001-iac-bootstrap.md..IP-015-hg-recordings.md.
