---
doc_class: Phase
template_id: TPL-PHASE
phase_id: PHASE-01-ANONYMOUS-FOUNDATION
microservice: anonymous
status: Active
milestone_parent: M02-foundation
date: 2026-05-17
owner_team: axis-anonymous
deciders: council-architecture, ops-security, council-privacy, axis-anonymous
related_adrs: [ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-ANON-0001, ADR-ANON-0002, ADR-ANON-0003, ADR-ANON-0004, ADR-ANON-0005, ADR-ANON-0006, ADR-ANON-0007]
doc_status: published
---

# PHASE-01-ANONYMOUS-FOUNDATION: Pseudonymous-Identity + Affinity-Attestation + Post + Vote + Feed + Moderation + Legal-Process Substrate

## Phase Scope

PHASE-01 ships the audit-grade *substrate* for the anonymous µservice. It does NOT ship: anonymous-DM (P02 IP-016+), trending (P02), algorithmic-ranking (P02; chronological-only in P01). It DOES ship:

- IP-001 → IP-015 (see below): IaC bootstrap → BC scaffolding → REST surface → workflow event emission → catalog records → SLO manifests → Helm chart → Kustomize overlays → CI lanes → branch protection.
- Seven design invariants I1–I7 enforced by code + Cedar + LEAN lanes from PR-1.
- Legal-process disclosure workflow operational from day-1 (no soft-launch without it).
- 30-day default retention; hard-delete propagation p99 ≤ 5s correctness 100% target.
- NCMEC CyberTipline integration operational from day-1 (18 USC §2258A is non-deferrable).

## Phase Exit Criteria

| EC-ID | Criterion | Verification |
|---|---|---|
| EC-01 | All 15 IPs (IP-001 → IP-015) merged + green-lane passing | `gh pr list --search "is:merged label:phase-01-anonymous"` |
| EC-02 | All 9 OpenSLO manifests authored + green | `oya gate validate openslo --microservice anonymous` |
| EC-03 | All 11 Cedar policies authored + Cedar v4.2 schema-valid | `cedar validate --schema policy/schema.cedarschema policy/*.cedar` |
| EC-04 | All 7 design invariants I1–I7 backed by code-level enforcement + at least one e2e test each | `cargo test -p oya-anonymous-* --features invariant-tests` |
| EC-05 | HG-ANONYMOUS authority cohesion gate green | `oya gate validate authority-cohesion --microservice anonymous` |
| EC-06 | Branch protection registered for `anonymous/*` paths via IP-015 | `gh api repos/{owner}/{repo}/branches/dev/protection` |
| EC-07 | All 7 ADRs (ADR-ANON-0001 → ADR-ANON-0007) Accepted + linked from decisions/README.md | review `microservices/anonymous/decisions/README.md` |
| EC-08 | Threat-model + DPIA + compliance + cost-budget + multi-region + incident-response + capacity-model + failure-modes + sdk-plan + competitor-parity-matrix + backfill-replay reviewed by council-privacy | review checklist |
| EC-09 | Catalog records authored for every kernel + adapter + rest + worker + sdk crate (~16 records) | `oya catalog list --microservice anonymous` |
| EC-10 | NCMEC CyberTipline integration smoke-tested in dev pack-us; CSAM-suspect verdict → reporter queue within 48h | `tests/e2e/ncmec-reporting.rs` |
| EC-11 | Legal-process disclosure rehearsed via tabletop exercise (court-order receipt → dual-control → audit-chain seal → transparency-report inclusion) | `runbooks/legal-process-court-order-receipt.md` tabletop log |
| EC-12 | LEAN lane `oya-check-third-party-tracker-refused` green (zero trackers in client SDK or app bundle) | LEAN lane CI |

## IP Sequence

See individual IP-NNN files in this directory. Dependency graph:

```
IP-001 IaC bootstrap
  └── IP-002 Cargo workspace bootstrap
        ├── IP-003 pseudonymous-identity kernel + domain
        │     └── IP-004 pseudonymous-identity adapter-postgres + adapter-blind-signatures
        ├── IP-005 affinity-attestation kernel + domain
        │     └── IP-006 affinity-attestation adapter-bbs-plus + adapter-postgres
        ├── IP-007 post-thread kernel + domain
        │     └── IP-008 post-thread adapter-postgres + REST + worker
        ├── IP-009 upvote-downvote BC end-to-end
        ├── IP-010 feed-timeline + redis cache
        ├── IP-011 content-moderation + foundry-runtime client + NCMEC reporter
        ├── IP-012 legal-process-disclosure end-to-end (dual-control + audit-chain)
        ├── IP-013 retention-policy + hard-delete worker
        └── IP-014 REST API surface + OpenAPI 3.2.0 + Cedar wiring
              └── IP-015 HG-ANONYMOUS registration + branch protection + Helm + Kustomize overlays + dashboards
```

## Phase Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Blind-signature library (`ring 0.17` / `rust-bls`) crypto-correctness | Low | Sev-1 (I1 violated) | ADR-ANON-0001 cites NIST SP 800-186; library audited; integration test with known-answer vectors |
| Affinity attestation BBS+ implementation drift | Medium | Sev-1 (I2 violated) | ADR-ANON-0002 pins library version + W3C VC 2.0 conformance test vectors |
| Legal-process disclosure flow not rehearsed before first court-order received | Medium | Sev-1 (chain-of-custody breach) | EC-11 mandatory tabletop before phase exit |
| Retention worker drift (hard-delete misses replica or backup) | Medium | Sev-1 (I3 violated) | EC-04 + dedicated SLO `hard-delete-propagation-correctness` 100% target |
| Third-party tracker accidentally added via transitive dependency | Medium | Sev-2 (I4 violated) | EC-12 LEAN lane mandatory from PR-1 |
| Federation BC accidentally scaffolded by template copy from `social` | Low | Sev-2 (I5 violated) | Code review + `decisions/ADR-ANON-0006` invariant doc; no `federation-gateway` directory exists |

## Sequencing with Other Wave-B Phases

- `anonymous` PHASE-01 may proceed in parallel with `social`, `community`, `mail`, `messenger` PHASE-01.
- `anonymous` PHASE-01 depends on `audit-chain` (sealing), `tenancy` (RLS), `cell` (per-affinity boundary), `ontology` (Affinity reads), `foundry-runtime` (T2 classifier), `cloud-secrets` (OpenBao for blind-signature private keys), `cloud-k8s` (substrate).
- `anonymous` PHASE-02 (P02): trending + algorithmic ranking + anonymous-DM MLS + T2 attachments + transparency-report exporter.
