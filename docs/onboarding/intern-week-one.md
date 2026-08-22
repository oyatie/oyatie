---
doc_class: Onboarding
shape: Tutorial
status: Proposed
date: 2026-05-21
authority_tier: 2
length_cap: 2400
planned_enforcement_ref: governance-doc-rigor
purpose: |
  Week-one onboarding for a programming-capable intern with zero prior Oyatie knowledge. Every step ends in a verifiable artifact and cites glossary or doctrine.
related_adrs:
  - ADR-0212
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0317
  - ADR-0320
companion_docs:
  - docs/GLOSSARY.md
  - docs/standards/documentation-rigor.md
  - docs/architecture/keystone-bundle-2026-05-20-synthesis.md
  - docs/personas/MASTER-ROSTER-2026-05-21.md
inbound_citations:
  - docs/AGENTS.md
  - docs/DOC-CATALOG.md
  - docs/standards/documentation-rigor.md
---

# Intern Week-One Onboarding

## A. Operating frame

The intern is a programming-capable principal with no prior Oyatie doctrine.
The week-one path teaches one identity, one tenant primitive, one Cedar gate, one workflow engine, one ontology, and one audit chain before feature work.
Every step ends in an artifact that a reviewer can inspect without guessing.
Escalation channels are `doc-style-reviewer`, `council-architecture`, `axis-foundry`, and the assigned pull-request reviewer.
The intern MUST preserve continuity of identity across personal learning context, supervised tenant membership, and repository contributor role.

## B. Glossary anchors

Glossary file token (GLOSSARY) means the canonical `docs/GLOSSARY.md` reference surface.

- `Tenant`: resolve in [GLOSSARY.md](../GLOSSARY.md) before using it in an issue, branch, or PR.
- `Principal`: resolve in [GLOSSARY.md](../GLOSSARY.md) before using it in an issue, branch, or PR.
- `Cedar permit`: resolve in [GLOSSARY.md](../GLOSSARY.md) before using it in an issue, branch, or PR.
- `Workflow`: resolve in [GLOSSARY.md](../GLOSSARY.md) before using it in an issue, branch, or PR.
- `Ontology`: resolve in [GLOSSARY.md](../GLOSSARY.md) before using it in an issue, branch, or PR.
- `Audit-chain`: resolve in [GLOSSARY.md](../GLOSSARY.md) before using it in an issue, branch, or PR.
- `Role projection`: resolve in [GLOSSARY.md](../GLOSSARY.md) before using it in an issue, branch, or PR.
- `Capability tier`: resolve in [GLOSSARY.md](../GLOSSARY.md) before using it in an issue, branch, or PR.
- `Dual-tenant boundary`: resolve in [GLOSSARY.md](../GLOSSARY.md) before using it in an issue, branch, or PR.
- `Transient identity`: resolve in [GLOSSARY.md](../GLOSSARY.md) before using it in an issue, branch, or PR.

## Day 0. Laptop bring-up and safety rails

### Day 0 step 01
Goal: Learn the laptop bring-up and safety rails path by tying `Tenant` to ADR-0242 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-01` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0242](../decisions/ADR-0242-oyatie-is-a-tenant-doctrine.md) in the artifact.

### Day 0 step 02
Goal: Learn the laptop bring-up and safety rails path by tying `Principal` to ADR-0243 and `specs/root-hub-pointers.json`.
Read: Open `specs/root-hub-pointers.json` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-02` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0243](../decisions/ADR-0243-cedar-as-universal-gate.md) in the artifact.

### Day 0 step 03
Goal: Learn the laptop bring-up and safety rails path by tying `Cedar permit` to ADR-0244 and `docs/standards/doc-style.md`.
Read: Open `docs/standards/doc-style.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-03` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0244](../decisions/ADR-0244-tenant-as-universal-scoping-primitive.md) in the artifact.

### Day 0 step 04
Goal: Learn the laptop bring-up and safety rails path by tying `Workflow` to ADR-0245 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-04` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0245](../decisions/ADR-0245-substrate-vs-product-layering.md) in the artifact.

### Day 0 step 05
Goal: Learn the laptop bring-up and safety rails path by tying `Ontology` to ADR-0246 and `specs/root-hub-pointers.json`.
Read: Open `specs/root-hub-pointers.json` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-05` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0246](../decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 0 step 06
Goal: Learn the laptop bring-up and safety rails path by tying `Audit-chain` to ADR-0247 and `docs/standards/doc-style.md`.
Read: Open `docs/standards/doc-style.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-06` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0247](../decisions/ADR-0247-self-hosting-self-modification-doctrine.md) in the artifact.

### Day 0 step 07
Goal: Learn the laptop bring-up and safety rails path by tying `Role projection` to ADR-0248 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-07` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0248](../decisions/ADR-0248-amazon-shape-cellular-architecture.md) in the artifact.

### Day 0 step 08
Goal: Learn the laptop bring-up and safety rails path by tying `Capability tier` to ADR-0249 and `specs/root-hub-pointers.json`.
Read: Open `specs/root-hub-pointers.json` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-08` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0249](../decisions/ADR-0249-multi-category-marketplace-doctrine.md) in the artifact.

### Day 0 step 09
Goal: Learn the laptop bring-up and safety rails path by tying `Dual-tenant boundary` to ADR-0250 and `docs/standards/doc-style.md`.
Read: Open `docs/standards/doc-style.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-09` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0250](../decisions/ADR-0250-build-ahead-of-certification-doctrine.md) in the artifact.

### Day 0 step 10
Goal: Learn the laptop bring-up and safety rails path by tying `Transient identity` to ADR-0251 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-10` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0251](../decisions/ADR-0251-compliance-pack-cell-certification-levels.md) in the artifact.

### Day 0 step 11
Goal: Learn the laptop bring-up and safety rails path by tying `Tenant` to ADR-0252 and `specs/root-hub-pointers.json`.
Read: Open `specs/root-hub-pointers.json` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-11` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0252](../decisions/ADR-0252-time-coordination-distributed-consistency.md) in the artifact.

### Day 0 step 12
Goal: Learn the laptop bring-up and safety rails path by tying `Principal` to ADR-0253 and `docs/standards/doc-style.md`.
Read: Open `docs/standards/doc-style.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-12` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0253](../decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) in the artifact.

### Day 0 step 13
Goal: Learn the laptop bring-up and safety rails path by tying `Cedar permit` to ADR-0254 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-13` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0254](../decisions/ADR-0254-deployment-model-spectrum.md) in the artifact.

### Day 0 step 14
Goal: Learn the laptop bring-up and safety rails path by tying `Workflow` to ADR-0255 and `specs/root-hub-pointers.json`.
Read: Open `specs/root-hub-pointers.json` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-14` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0255](../decisions/ADR-0355-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 0 step 15
Goal: Learn the laptop bring-up and safety rails path by tying `Ontology` to ADR-0257 and `docs/standards/doc-style.md`.
Read: Open `docs/standards/doc-style.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-15` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0257](../decisions/ADR-0356-amendment-library-first-ontology-read-path.md) in the artifact.

### Day 0 step 16
Goal: Learn the laptop bring-up and safety rails path by tying `Audit-chain` to ADR-0258 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-16` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0258](../decisions/ADR-0258-api-versioning-model.md) in the artifact.

### Day 0 step 17
Goal: Learn the laptop bring-up and safety rails path by tying `Role projection` to ADR-0263 and `specs/root-hub-pointers.json`.
Read: Open `specs/root-hub-pointers.json` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-17` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0263](../decisions/ADR-0263-observability-emission-contract.md) in the artifact.

### Day 0 step 18
Goal: Learn the laptop bring-up and safety rails path by tying `Capability tier` to ADR-0273 and `docs/standards/doc-style.md`.
Read: Open `docs/standards/doc-style.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-18` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0273](../decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md) in the artifact.

### Day 0 step 19
Goal: Learn the laptop bring-up and safety rails path by tying `Dual-tenant boundary` to ADR-0276 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-19` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0276](../decisions/ADR-0276-backup-portability-format-gdpr-article-20.md) in the artifact.

### Day 0 step 20
Goal: Learn the laptop bring-up and safety rails path by tying `Transient identity` to ADR-0280 and `specs/root-hub-pointers.json`.
Read: Open `specs/root-hub-pointers.json` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-20` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0280](../decisions/ADR-0280-substrate-of-substrate-dependency-doctrine.md) in the artifact.

### Day 0 step 21
Goal: Learn the laptop bring-up and safety rails path by tying `Tenant` to ADR-0284 and `docs/standards/doc-style.md`.
Read: Open `docs/standards/doc-style.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-21` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0284](../decisions/ADR-0284-platform-owner-name-indirection.md) in the artifact.

### Day 0 step 22
Goal: Learn the laptop bring-up and safety rails path by tying `Principal` to ADR-0292 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-22` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0292](../decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md) in the artifact.

### Day 0 step 23
Goal: Learn the laptop bring-up and safety rails path by tying `Cedar permit` to ADR-0293 and `specs/root-hub-pointers.json`.
Read: Open `specs/root-hub-pointers.json` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-23` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0293](../decisions/ADR-0293-governance-meta-trust-root.md) in the artifact.

### Day 0 step 24
Goal: Learn the laptop bring-up and safety rails path by tying `Workflow` to ADR-0294 and `docs/standards/doc-style.md`.
Read: Open `docs/standards/doc-style.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-24` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0294](../decisions/ADR-0294-cedar-fragment-soak-anomaly-rollback.md) in the artifact.

### Day 0 step 25
Goal: Learn the laptop bring-up and safety rails path by tying `Ontology` to ADR-0295 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-25` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0295](../decisions/ADR-0295-bootstrap-ci-spiffe-kill-switch.md) in the artifact.

### Day 0 step 26
Goal: Learn the laptop bring-up and safety rails path by tying `Audit-chain` to ADR-0296 and `specs/root-hub-pointers.json`.
Read: Open `specs/root-hub-pointers.json` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-26` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0296](../decisions/ADR-0296-library-first-credential-sidecar.md) in the artifact.

### Day 0 step 27
Goal: Learn the laptop bring-up and safety rails path by tying `Role projection` to ADR-0311 and `docs/standards/doc-style.md`.
Read: Open `docs/standards/doc-style.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-27` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0311](../decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md) in the artifact.

### Day 0 step 28
Goal: Learn the laptop bring-up and safety rails path by tying `Capability tier` to ADR-0313 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-28` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0313](../decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md) in the artifact.

### Day 0 step 29
Goal: Learn the laptop bring-up and safety rails path by tying `Dual-tenant boundary` to ADR-0316 and `specs/root-hub-pointers.json`.
Read: Open `specs/root-hub-pointers.json` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-29` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0316](../decisions/ADR-0316-capability-tier-over-product-fragmentation.md) in the artifact.

### Day 0 step 30
Goal: Learn the laptop bring-up and safety rails path by tying `Transient identity` to ADR-0317 and `docs/standards/doc-style.md`.
Read: Open `docs/standards/doc-style.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-30` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0317](../decisions/ADR-0317-role-based-projection-unified-ux-shell.md) in the artifact.

### Day 0 step 31
Goal: Learn the laptop bring-up and safety rails path by tying `Tenant` to ADR-0242 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-31` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0242](../decisions/ADR-0242-oyatie-is-a-tenant-doctrine.md) in the artifact.

### Day 0 step 32
Goal: Learn the laptop bring-up and safety rails path by tying `Principal` to ADR-0243 and `specs/root-hub-pointers.json`.
Read: Open `specs/root-hub-pointers.json` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-32` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0243](../decisions/ADR-0243-cedar-as-universal-gate.md) in the artifact.

### Day 0 step 33
Goal: Learn the laptop bring-up and safety rails path by tying `Cedar permit` to ADR-0244 and `docs/standards/doc-style.md`.
Read: Open `docs/standards/doc-style.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-33` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0244](../decisions/ADR-0244-tenant-as-universal-scoping-primitive.md) in the artifact.

### Day 0 step 34
Goal: Learn the laptop bring-up and safety rails path by tying `Workflow` to ADR-0245 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-34` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0245](../decisions/ADR-0245-substrate-vs-product-layering.md) in the artifact.

### Day 0 step 35
Goal: Learn the laptop bring-up and safety rails path by tying `Ontology` to ADR-0246 and `specs/root-hub-pointers.json`.
Read: Open `specs/root-hub-pointers.json` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-35` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0246](../decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 0 step 36
Goal: Learn the laptop bring-up and safety rails path by tying `Audit-chain` to ADR-0247 and `docs/standards/doc-style.md`.
Read: Open `docs/standards/doc-style.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-36` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0247](../decisions/ADR-0247-self-hosting-self-modification-doctrine.md) in the artifact.

### Day 0 step 37
Goal: Learn the laptop bring-up and safety rails path by tying `Role projection` to ADR-0248 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-37` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0248](../decisions/ADR-0248-amazon-shape-cellular-architecture.md) in the artifact.

### Day 0 step 38
Goal: Learn the laptop bring-up and safety rails path by tying `Capability tier` to ADR-0249 and `specs/root-hub-pointers.json`.
Read: Open `specs/root-hub-pointers.json` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-38` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0249](../decisions/ADR-0249-multi-category-marketplace-doctrine.md) in the artifact.

### Day 0 step 39
Goal: Learn the laptop bring-up and safety rails path by tying `Dual-tenant boundary` to ADR-0250 and `docs/standards/doc-style.md`.
Read: Open `docs/standards/doc-style.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-39` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0250](../decisions/ADR-0250-build-ahead-of-certification-doctrine.md) in the artifact.

### Day 0 step 40
Goal: Learn the laptop bring-up and safety rails path by tying `Transient identity` to ADR-0251 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-40` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0251](../decisions/ADR-0251-compliance-pack-cell-certification-levels.md) in the artifact.

### Day 0 step 41
Goal: Learn the laptop bring-up and safety rails path by tying `Tenant` to ADR-0252 and `specs/root-hub-pointers.json`.
Read: Open `specs/root-hub-pointers.json` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-41` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0252](../decisions/ADR-0252-time-coordination-distributed-consistency.md) in the artifact.

### Day 0 step 42
Goal: Learn the laptop bring-up and safety rails path by tying `Principal` to ADR-0253 and `docs/standards/doc-style.md`.
Read: Open `docs/standards/doc-style.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-42` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0253](../decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) in the artifact.

### Day 0 step 43
Goal: Learn the laptop bring-up and safety rails path by tying `Cedar permit` to ADR-0254 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-43` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0254](../decisions/ADR-0254-deployment-model-spectrum.md) in the artifact.

### Day 0 step 44
Goal: Learn the laptop bring-up and safety rails path by tying `Workflow` to ADR-0255 and `specs/root-hub-pointers.json`.
Read: Open `specs/root-hub-pointers.json` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-44` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0255](../decisions/ADR-0355-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 0 step 45
Goal: Learn the laptop bring-up and safety rails path by tying `Ontology` to ADR-0257 and `docs/standards/doc-style.md`.
Read: Open `docs/standards/doc-style.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-45` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0257](../decisions/ADR-0356-amendment-library-first-ontology-read-path.md) in the artifact.

### Day 0 step 46
Goal: Learn the laptop bring-up and safety rails path by tying `Audit-chain` to ADR-0258 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-46` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0258](../decisions/ADR-0258-api-versioning-model.md) in the artifact.

### Day 0 step 47
Goal: Learn the laptop bring-up and safety rails path by tying `Role projection` to ADR-0263 and `specs/root-hub-pointers.json`.
Read: Open `specs/root-hub-pointers.json` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-47` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0263](../decisions/ADR-0263-observability-emission-contract.md) in the artifact.

### Day 0 step 48
Goal: Learn the laptop bring-up and safety rails path by tying `Capability tier` to ADR-0273 and `docs/standards/doc-style.md`.
Read: Open `docs/standards/doc-style.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-48` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0273](../decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md) in the artifact.

### Day 0 step 49
Goal: Learn the laptop bring-up and safety rails path by tying `Dual-tenant boundary` to ADR-0276 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-49` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0276](../decisions/ADR-0276-backup-portability-format-gdpr-article-20.md) in the artifact.

### Day 0 step 50
Goal: Learn the laptop bring-up and safety rails path by tying `Transient identity` to ADR-0280 and `specs/root-hub-pointers.json`.
Read: Open `specs/root-hub-pointers.json` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-0-50` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0280](../decisions/ADR-0280-substrate-of-substrate-dependency-doctrine.md) in the artifact.

## Day 1. Repo orientation, master roster, documentation rigor, and keystone synthesis

### Day 1 step 01
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Tenant` to ADR-0242 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-01` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0242](../decisions/ADR-0242-oyatie-is-a-tenant-doctrine.md) in the artifact.

### Day 1 step 02
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Principal` to ADR-0243 and `docs/standards/documentation-rigor.md`.
Read: Open `docs/standards/documentation-rigor.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-02` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0243](../decisions/ADR-0243-cedar-as-universal-gate.md) in the artifact.

### Day 1 step 03
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Cedar permit` to ADR-0244 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-03` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0244](../decisions/ADR-0244-tenant-as-universal-scoping-primitive.md) in the artifact.

### Day 1 step 04
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Workflow` to ADR-0245 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-04` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0245](../decisions/ADR-0245-substrate-vs-product-layering.md) in the artifact.

### Day 1 step 05
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Ontology` to ADR-0246 and `docs/standards/documentation-rigor.md`.
Read: Open `docs/standards/documentation-rigor.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-05` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0246](../decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 1 step 06
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Audit-chain` to ADR-0247 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-06` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0247](../decisions/ADR-0247-self-hosting-self-modification-doctrine.md) in the artifact.

### Day 1 step 07
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Role projection` to ADR-0248 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-07` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0248](../decisions/ADR-0248-amazon-shape-cellular-architecture.md) in the artifact.

### Day 1 step 08
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Capability tier` to ADR-0249 and `docs/standards/documentation-rigor.md`.
Read: Open `docs/standards/documentation-rigor.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-08` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0249](../decisions/ADR-0249-multi-category-marketplace-doctrine.md) in the artifact.

### Day 1 step 09
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Dual-tenant boundary` to ADR-0250 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-09` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0250](../decisions/ADR-0250-build-ahead-of-certification-doctrine.md) in the artifact.

### Day 1 step 10
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Transient identity` to ADR-0251 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-10` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0251](../decisions/ADR-0251-compliance-pack-cell-certification-levels.md) in the artifact.

### Day 1 step 11
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Tenant` to ADR-0252 and `docs/standards/documentation-rigor.md`.
Read: Open `docs/standards/documentation-rigor.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-11` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0252](../decisions/ADR-0252-time-coordination-distributed-consistency.md) in the artifact.

### Day 1 step 12
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Principal` to ADR-0253 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-12` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0253](../decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) in the artifact.

### Day 1 step 13
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Cedar permit` to ADR-0254 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-13` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0254](../decisions/ADR-0254-deployment-model-spectrum.md) in the artifact.

### Day 1 step 14
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Workflow` to ADR-0255 and `docs/standards/documentation-rigor.md`.
Read: Open `docs/standards/documentation-rigor.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-14` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0255](../decisions/ADR-0355-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 1 step 15
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Ontology` to ADR-0257 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-15` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0257](../decisions/ADR-0356-amendment-library-first-ontology-read-path.md) in the artifact.

### Day 1 step 16
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Audit-chain` to ADR-0258 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-16` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0258](../decisions/ADR-0258-api-versioning-model.md) in the artifact.

### Day 1 step 17
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Role projection` to ADR-0263 and `docs/standards/documentation-rigor.md`.
Read: Open `docs/standards/documentation-rigor.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-17` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0263](../decisions/ADR-0263-observability-emission-contract.md) in the artifact.

### Day 1 step 18
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Capability tier` to ADR-0273 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-18` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0273](../decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md) in the artifact.

### Day 1 step 19
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Dual-tenant boundary` to ADR-0276 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-19` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0276](../decisions/ADR-0276-backup-portability-format-gdpr-article-20.md) in the artifact.

### Day 1 step 20
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Transient identity` to ADR-0280 and `docs/standards/documentation-rigor.md`.
Read: Open `docs/standards/documentation-rigor.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-20` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0280](../decisions/ADR-0280-substrate-of-substrate-dependency-doctrine.md) in the artifact.

### Day 1 step 21
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Tenant` to ADR-0284 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-21` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0284](../decisions/ADR-0284-platform-owner-name-indirection.md) in the artifact.

### Day 1 step 22
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Principal` to ADR-0292 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-22` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0292](../decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md) in the artifact.

### Day 1 step 23
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Cedar permit` to ADR-0293 and `docs/standards/documentation-rigor.md`.
Read: Open `docs/standards/documentation-rigor.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-23` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0293](../decisions/ADR-0293-governance-meta-trust-root.md) in the artifact.

### Day 1 step 24
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Workflow` to ADR-0294 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-24` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0294](../decisions/ADR-0294-cedar-fragment-soak-anomaly-rollback.md) in the artifact.

### Day 1 step 25
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Ontology` to ADR-0295 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-25` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0295](../decisions/ADR-0295-bootstrap-ci-spiffe-kill-switch.md) in the artifact.

### Day 1 step 26
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Audit-chain` to ADR-0296 and `docs/standards/documentation-rigor.md`.
Read: Open `docs/standards/documentation-rigor.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-26` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0296](../decisions/ADR-0296-library-first-credential-sidecar.md) in the artifact.

### Day 1 step 27
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Role projection` to ADR-0311 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-27` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0311](../decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md) in the artifact.

### Day 1 step 28
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Capability tier` to ADR-0313 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-28` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0313](../decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md) in the artifact.

### Day 1 step 29
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Dual-tenant boundary` to ADR-0316 and `docs/standards/documentation-rigor.md`.
Read: Open `docs/standards/documentation-rigor.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-29` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0316](../decisions/ADR-0316-capability-tier-over-product-fragmentation.md) in the artifact.

### Day 1 step 30
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Transient identity` to ADR-0317 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-30` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0317](../decisions/ADR-0317-role-based-projection-unified-ux-shell.md) in the artifact.

### Day 1 step 31
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Tenant` to ADR-0242 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-31` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0242](../decisions/ADR-0242-oyatie-is-a-tenant-doctrine.md) in the artifact.

### Day 1 step 32
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Principal` to ADR-0243 and `docs/standards/documentation-rigor.md`.
Read: Open `docs/standards/documentation-rigor.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-32` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0243](../decisions/ADR-0243-cedar-as-universal-gate.md) in the artifact.

### Day 1 step 33
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Cedar permit` to ADR-0244 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-33` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0244](../decisions/ADR-0244-tenant-as-universal-scoping-primitive.md) in the artifact.

### Day 1 step 34
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Workflow` to ADR-0245 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-34` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0245](../decisions/ADR-0245-substrate-vs-product-layering.md) in the artifact.

### Day 1 step 35
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Ontology` to ADR-0246 and `docs/standards/documentation-rigor.md`.
Read: Open `docs/standards/documentation-rigor.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-35` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0246](../decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 1 step 36
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Audit-chain` to ADR-0247 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-36` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0247](../decisions/ADR-0247-self-hosting-self-modification-doctrine.md) in the artifact.

### Day 1 step 37
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Role projection` to ADR-0248 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-37` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0248](../decisions/ADR-0248-amazon-shape-cellular-architecture.md) in the artifact.

### Day 1 step 38
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Capability tier` to ADR-0249 and `docs/standards/documentation-rigor.md`.
Read: Open `docs/standards/documentation-rigor.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-38` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0249](../decisions/ADR-0249-multi-category-marketplace-doctrine.md) in the artifact.

### Day 1 step 39
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Dual-tenant boundary` to ADR-0250 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-39` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0250](../decisions/ADR-0250-build-ahead-of-certification-doctrine.md) in the artifact.

### Day 1 step 40
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Transient identity` to ADR-0251 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-40` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0251](../decisions/ADR-0251-compliance-pack-cell-certification-levels.md) in the artifact.

### Day 1 step 41
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Tenant` to ADR-0252 and `docs/standards/documentation-rigor.md`.
Read: Open `docs/standards/documentation-rigor.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-41` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0252](../decisions/ADR-0252-time-coordination-distributed-consistency.md) in the artifact.

### Day 1 step 42
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Principal` to ADR-0253 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-42` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0253](../decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) in the artifact.

### Day 1 step 43
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Cedar permit` to ADR-0254 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-43` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0254](../decisions/ADR-0254-deployment-model-spectrum.md) in the artifact.

### Day 1 step 44
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Workflow` to ADR-0255 and `docs/standards/documentation-rigor.md`.
Read: Open `docs/standards/documentation-rigor.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-44` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0255](../decisions/ADR-0355-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 1 step 45
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Ontology` to ADR-0257 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-45` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0257](../decisions/ADR-0356-amendment-library-first-ontology-read-path.md) in the artifact.

### Day 1 step 46
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Audit-chain` to ADR-0258 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-46` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0258](../decisions/ADR-0258-api-versioning-model.md) in the artifact.

### Day 1 step 47
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Role projection` to ADR-0263 and `docs/standards/documentation-rigor.md`.
Read: Open `docs/standards/documentation-rigor.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-47` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0263](../decisions/ADR-0263-observability-emission-contract.md) in the artifact.

### Day 1 step 48
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Capability tier` to ADR-0273 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-48` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0273](../decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md) in the artifact.

### Day 1 step 49
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Dual-tenant boundary` to ADR-0276 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-49` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0276](../decisions/ADR-0276-backup-portability-format-gdpr-article-20.md) in the artifact.

### Day 1 step 50
Goal: Learn the repo orientation, master roster, documentation rigor, and keystone synthesis path by tying `Transient identity` to ADR-0280 and `docs/standards/documentation-rigor.md`.
Read: Open `docs/standards/documentation-rigor.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-1-50` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0280](../decisions/ADR-0280-substrate-of-substrate-dependency-doctrine.md) in the artifact.

## Day 2. Tenant primitive, Cedar gate, workflow engine, and first PR

### Day 2 step 01
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Tenant` to ADR-0242 and `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
Read: Open `docs/decisions/ADR-0702-identity-authz-live-apex.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-01` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0242](../decisions/ADR-0242-oyatie-is-a-tenant-doctrine.md) in the artifact.

### Day 2 step 02
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Principal` to ADR-0243 and `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
Read: Open `docs/decisions/ADR-0700-ci-admission-live-apex.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-02` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0243](../decisions/ADR-0243-cedar-as-universal-gate.md) in the artifact.

### Day 2 step 03
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Cedar permit` to ADR-0244 and `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
Read: Open `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-03` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0244](../decisions/ADR-0244-tenant-as-universal-scoping-primitive.md) in the artifact.

### Day 2 step 04
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Workflow` to ADR-0245 and `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
Read: Open `docs/decisions/ADR-0702-identity-authz-live-apex.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-04` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0245](../decisions/ADR-0245-substrate-vs-product-layering.md) in the artifact.

### Day 2 step 05
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Ontology` to ADR-0246 and `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
Read: Open `docs/decisions/ADR-0700-ci-admission-live-apex.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-05` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0246](../decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 2 step 06
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Audit-chain` to ADR-0247 and `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
Read: Open `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-06` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0247](../decisions/ADR-0247-self-hosting-self-modification-doctrine.md) in the artifact.

### Day 2 step 07
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Role projection` to ADR-0248 and `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
Read: Open `docs/decisions/ADR-0702-identity-authz-live-apex.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-07` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0248](../decisions/ADR-0248-amazon-shape-cellular-architecture.md) in the artifact.

### Day 2 step 08
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Capability tier` to ADR-0249 and `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
Read: Open `docs/decisions/ADR-0700-ci-admission-live-apex.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-08` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0249](../decisions/ADR-0249-multi-category-marketplace-doctrine.md) in the artifact.

### Day 2 step 09
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Dual-tenant boundary` to ADR-0250 and `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
Read: Open `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-09` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0250](../decisions/ADR-0250-build-ahead-of-certification-doctrine.md) in the artifact.

### Day 2 step 10
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Transient identity` to ADR-0251 and `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
Read: Open `docs/decisions/ADR-0702-identity-authz-live-apex.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-10` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0251](../decisions/ADR-0251-compliance-pack-cell-certification-levels.md) in the artifact.

### Day 2 step 11
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Tenant` to ADR-0252 and `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
Read: Open `docs/decisions/ADR-0700-ci-admission-live-apex.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-11` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0252](../decisions/ADR-0252-time-coordination-distributed-consistency.md) in the artifact.

### Day 2 step 12
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Principal` to ADR-0253 and `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
Read: Open `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-12` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0253](../decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) in the artifact.

### Day 2 step 13
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Cedar permit` to ADR-0254 and `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
Read: Open `docs/decisions/ADR-0702-identity-authz-live-apex.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-13` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0254](../decisions/ADR-0254-deployment-model-spectrum.md) in the artifact.

### Day 2 step 14
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Workflow` to ADR-0255 and `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
Read: Open `docs/decisions/ADR-0700-ci-admission-live-apex.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-14` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0255](../decisions/ADR-0355-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 2 step 15
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Ontology` to ADR-0257 and `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
Read: Open `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-15` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0257](../decisions/ADR-0356-amendment-library-first-ontology-read-path.md) in the artifact.

### Day 2 step 16
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Audit-chain` to ADR-0258 and `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
Read: Open `docs/decisions/ADR-0702-identity-authz-live-apex.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-16` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0258](../decisions/ADR-0258-api-versioning-model.md) in the artifact.

### Day 2 step 17
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Role projection` to ADR-0263 and `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
Read: Open `docs/decisions/ADR-0700-ci-admission-live-apex.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-17` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0263](../decisions/ADR-0263-observability-emission-contract.md) in the artifact.

### Day 2 step 18
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Capability tier` to ADR-0273 and `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
Read: Open `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-18` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0273](../decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md) in the artifact.

### Day 2 step 19
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Dual-tenant boundary` to ADR-0276 and `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
Read: Open `docs/decisions/ADR-0702-identity-authz-live-apex.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-19` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0276](../decisions/ADR-0276-backup-portability-format-gdpr-article-20.md) in the artifact.

### Day 2 step 20
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Transient identity` to ADR-0280 and `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
Read: Open `docs/decisions/ADR-0700-ci-admission-live-apex.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-20` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0280](../decisions/ADR-0280-substrate-of-substrate-dependency-doctrine.md) in the artifact.

### Day 2 step 21
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Tenant` to ADR-0284 and `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
Read: Open `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-21` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0284](../decisions/ADR-0284-platform-owner-name-indirection.md) in the artifact.

### Day 2 step 22
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Principal` to ADR-0292 and `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
Read: Open `docs/decisions/ADR-0702-identity-authz-live-apex.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-22` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0292](../decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md) in the artifact.

### Day 2 step 23
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Cedar permit` to ADR-0293 and `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
Read: Open `docs/decisions/ADR-0700-ci-admission-live-apex.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-23` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0293](../decisions/ADR-0293-governance-meta-trust-root.md) in the artifact.

### Day 2 step 24
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Workflow` to ADR-0294 and `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
Read: Open `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-24` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0294](../decisions/ADR-0294-cedar-fragment-soak-anomaly-rollback.md) in the artifact.

### Day 2 step 25
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Ontology` to ADR-0295 and `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
Read: Open `docs/decisions/ADR-0702-identity-authz-live-apex.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-25` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0295](../decisions/ADR-0295-bootstrap-ci-spiffe-kill-switch.md) in the artifact.

### Day 2 step 26
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Audit-chain` to ADR-0296 and `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
Read: Open `docs/decisions/ADR-0700-ci-admission-live-apex.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-26` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0296](../decisions/ADR-0296-library-first-credential-sidecar.md) in the artifact.

### Day 2 step 27
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Role projection` to ADR-0311 and `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
Read: Open `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-27` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0311](../decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md) in the artifact.

### Day 2 step 28
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Capability tier` to ADR-0313 and `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
Read: Open `docs/decisions/ADR-0702-identity-authz-live-apex.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-28` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0313](../decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md) in the artifact.

### Day 2 step 29
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Dual-tenant boundary` to ADR-0316 and `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
Read: Open `docs/decisions/ADR-0700-ci-admission-live-apex.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-29` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0316](../decisions/ADR-0316-capability-tier-over-product-fragmentation.md) in the artifact.

### Day 2 step 30
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Transient identity` to ADR-0317 and `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
Read: Open `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-30` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0317](../decisions/ADR-0317-role-based-projection-unified-ux-shell.md) in the artifact.

### Day 2 step 31
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Tenant` to ADR-0242 and `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
Read: Open `docs/decisions/ADR-0702-identity-authz-live-apex.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-31` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0242](../decisions/ADR-0242-oyatie-is-a-tenant-doctrine.md) in the artifact.

### Day 2 step 32
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Principal` to ADR-0243 and `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
Read: Open `docs/decisions/ADR-0700-ci-admission-live-apex.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-32` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0243](../decisions/ADR-0243-cedar-as-universal-gate.md) in the artifact.

### Day 2 step 33
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Cedar permit` to ADR-0244 and `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
Read: Open `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-33` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0244](../decisions/ADR-0244-tenant-as-universal-scoping-primitive.md) in the artifact.

### Day 2 step 34
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Workflow` to ADR-0245 and `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
Read: Open `docs/decisions/ADR-0702-identity-authz-live-apex.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-34` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0245](../decisions/ADR-0245-substrate-vs-product-layering.md) in the artifact.

### Day 2 step 35
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Ontology` to ADR-0246 and `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
Read: Open `docs/decisions/ADR-0700-ci-admission-live-apex.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-35` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0246](../decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 2 step 36
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Audit-chain` to ADR-0247 and `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
Read: Open `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-36` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0247](../decisions/ADR-0247-self-hosting-self-modification-doctrine.md) in the artifact.

### Day 2 step 37
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Role projection` to ADR-0248 and `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
Read: Open `docs/decisions/ADR-0702-identity-authz-live-apex.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-37` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0248](../decisions/ADR-0248-amazon-shape-cellular-architecture.md) in the artifact.

### Day 2 step 38
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Capability tier` to ADR-0249 and `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
Read: Open `docs/decisions/ADR-0700-ci-admission-live-apex.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-38` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0249](../decisions/ADR-0249-multi-category-marketplace-doctrine.md) in the artifact.

### Day 2 step 39
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Dual-tenant boundary` to ADR-0250 and `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
Read: Open `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-39` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0250](../decisions/ADR-0250-build-ahead-of-certification-doctrine.md) in the artifact.

### Day 2 step 40
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Transient identity` to ADR-0251 and `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
Read: Open `docs/decisions/ADR-0702-identity-authz-live-apex.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-40` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0251](../decisions/ADR-0251-compliance-pack-cell-certification-levels.md) in the artifact.

### Day 2 step 41
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Tenant` to ADR-0252 and `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
Read: Open `docs/decisions/ADR-0700-ci-admission-live-apex.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-41` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0252](../decisions/ADR-0252-time-coordination-distributed-consistency.md) in the artifact.

### Day 2 step 42
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Principal` to ADR-0253 and `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
Read: Open `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-42` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0253](../decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) in the artifact.

### Day 2 step 43
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Cedar permit` to ADR-0254 and `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
Read: Open `docs/decisions/ADR-0702-identity-authz-live-apex.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-43` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0254](../decisions/ADR-0254-deployment-model-spectrum.md) in the artifact.

### Day 2 step 44
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Workflow` to ADR-0255 and `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
Read: Open `docs/decisions/ADR-0700-ci-admission-live-apex.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-44` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0255](../decisions/ADR-0355-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 2 step 45
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Ontology` to ADR-0257 and `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
Read: Open `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-45` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0257](../decisions/ADR-0356-amendment-library-first-ontology-read-path.md) in the artifact.

### Day 2 step 46
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Audit-chain` to ADR-0258 and `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
Read: Open `docs/decisions/ADR-0702-identity-authz-live-apex.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-46` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0258](../decisions/ADR-0258-api-versioning-model.md) in the artifact.

### Day 2 step 47
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Role projection` to ADR-0263 and `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
Read: Open `docs/decisions/ADR-0700-ci-admission-live-apex.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-47` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0263](../decisions/ADR-0263-observability-emission-contract.md) in the artifact.

### Day 2 step 48
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Capability tier` to ADR-0273 and `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
Read: Open `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-48` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0273](../decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md) in the artifact.

### Day 2 step 49
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Dual-tenant boundary` to ADR-0276 and `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
Read: Open `docs/decisions/ADR-0702-identity-authz-live-apex.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-49` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0276](../decisions/ADR-0276-backup-portability-format-gdpr-article-20.md) in the artifact.

### Day 2 step 50
Goal: Learn the tenant primitive, cedar gate, workflow engine, and first pr path by tying `Transient identity` to ADR-0280 and `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
Read: Open `docs/decisions/ADR-0700-ci-admission-live-apex.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-2-50` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0280](../decisions/ADR-0280-substrate-of-substrate-dependency-doctrine.md) in the artifact.

## Day 3. Personas, journeys, and continuity of identity

### Day 3 step 01
Goal: Learn the personas, journeys, and continuity of identity path by tying `Tenant` to ADR-0242 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-01` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0242](../decisions/ADR-0242-oyatie-is-a-tenant-doctrine.md) in the artifact.

### Day 3 step 02
Goal: Learn the personas, journeys, and continuity of identity path by tying `Principal` to ADR-0243 and `docs/user-journeys/CATALOG-j126-j150-ecosystem.md`.
Read: Open `docs/user-journeys/CATALOG-j126-j150-ecosystem.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-02` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0243](../decisions/ADR-0243-cedar-as-universal-gate.md) in the artifact.

### Day 3 step 03
Goal: Learn the personas, journeys, and continuity of identity path by tying `Cedar permit` to ADR-0244 and `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md`.
Read: Open `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-03` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0244](../decisions/ADR-0244-tenant-as-universal-scoping-primitive.md) in the artifact.

### Day 3 step 04
Goal: Learn the personas, journeys, and continuity of identity path by tying `Workflow` to ADR-0245 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-04` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0245](../decisions/ADR-0245-substrate-vs-product-layering.md) in the artifact.

### Day 3 step 05
Goal: Learn the personas, journeys, and continuity of identity path by tying `Ontology` to ADR-0246 and `docs/user-journeys/CATALOG-j126-j150-ecosystem.md`.
Read: Open `docs/user-journeys/CATALOG-j126-j150-ecosystem.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-05` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0246](../decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 3 step 06
Goal: Learn the personas, journeys, and continuity of identity path by tying `Audit-chain` to ADR-0247 and `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md`.
Read: Open `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-06` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0247](../decisions/ADR-0247-self-hosting-self-modification-doctrine.md) in the artifact.

### Day 3 step 07
Goal: Learn the personas, journeys, and continuity of identity path by tying `Role projection` to ADR-0248 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-07` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0248](../decisions/ADR-0248-amazon-shape-cellular-architecture.md) in the artifact.

### Day 3 step 08
Goal: Learn the personas, journeys, and continuity of identity path by tying `Capability tier` to ADR-0249 and `docs/user-journeys/CATALOG-j126-j150-ecosystem.md`.
Read: Open `docs/user-journeys/CATALOG-j126-j150-ecosystem.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-08` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0249](../decisions/ADR-0249-multi-category-marketplace-doctrine.md) in the artifact.

### Day 3 step 09
Goal: Learn the personas, journeys, and continuity of identity path by tying `Dual-tenant boundary` to ADR-0250 and `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md`.
Read: Open `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-09` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0250](../decisions/ADR-0250-build-ahead-of-certification-doctrine.md) in the artifact.

### Day 3 step 10
Goal: Learn the personas, journeys, and continuity of identity path by tying `Transient identity` to ADR-0251 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-10` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0251](../decisions/ADR-0251-compliance-pack-cell-certification-levels.md) in the artifact.

### Day 3 step 11
Goal: Learn the personas, journeys, and continuity of identity path by tying `Tenant` to ADR-0252 and `docs/user-journeys/CATALOG-j126-j150-ecosystem.md`.
Read: Open `docs/user-journeys/CATALOG-j126-j150-ecosystem.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-11` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0252](../decisions/ADR-0252-time-coordination-distributed-consistency.md) in the artifact.

### Day 3 step 12
Goal: Learn the personas, journeys, and continuity of identity path by tying `Principal` to ADR-0253 and `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md`.
Read: Open `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-12` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0253](../decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) in the artifact.

### Day 3 step 13
Goal: Learn the personas, journeys, and continuity of identity path by tying `Cedar permit` to ADR-0254 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-13` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0254](../decisions/ADR-0254-deployment-model-spectrum.md) in the artifact.

### Day 3 step 14
Goal: Learn the personas, journeys, and continuity of identity path by tying `Workflow` to ADR-0255 and `docs/user-journeys/CATALOG-j126-j150-ecosystem.md`.
Read: Open `docs/user-journeys/CATALOG-j126-j150-ecosystem.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-14` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0255](../decisions/ADR-0355-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 3 step 15
Goal: Learn the personas, journeys, and continuity of identity path by tying `Ontology` to ADR-0257 and `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md`.
Read: Open `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-15` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0257](../decisions/ADR-0356-amendment-library-first-ontology-read-path.md) in the artifact.

### Day 3 step 16
Goal: Learn the personas, journeys, and continuity of identity path by tying `Audit-chain` to ADR-0258 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-16` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0258](../decisions/ADR-0258-api-versioning-model.md) in the artifact.

### Day 3 step 17
Goal: Learn the personas, journeys, and continuity of identity path by tying `Role projection` to ADR-0263 and `docs/user-journeys/CATALOG-j126-j150-ecosystem.md`.
Read: Open `docs/user-journeys/CATALOG-j126-j150-ecosystem.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-17` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0263](../decisions/ADR-0263-observability-emission-contract.md) in the artifact.

### Day 3 step 18
Goal: Learn the personas, journeys, and continuity of identity path by tying `Capability tier` to ADR-0273 and `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md`.
Read: Open `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-18` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0273](../decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md) in the artifact.

### Day 3 step 19
Goal: Learn the personas, journeys, and continuity of identity path by tying `Dual-tenant boundary` to ADR-0276 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-19` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0276](../decisions/ADR-0276-backup-portability-format-gdpr-article-20.md) in the artifact.

### Day 3 step 20
Goal: Learn the personas, journeys, and continuity of identity path by tying `Transient identity` to ADR-0280 and `docs/user-journeys/CATALOG-j126-j150-ecosystem.md`.
Read: Open `docs/user-journeys/CATALOG-j126-j150-ecosystem.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-20` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0280](../decisions/ADR-0280-substrate-of-substrate-dependency-doctrine.md) in the artifact.

### Day 3 step 21
Goal: Learn the personas, journeys, and continuity of identity path by tying `Tenant` to ADR-0284 and `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md`.
Read: Open `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-21` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0284](../decisions/ADR-0284-platform-owner-name-indirection.md) in the artifact.

### Day 3 step 22
Goal: Learn the personas, journeys, and continuity of identity path by tying `Principal` to ADR-0292 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-22` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0292](../decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md) in the artifact.

### Day 3 step 23
Goal: Learn the personas, journeys, and continuity of identity path by tying `Cedar permit` to ADR-0293 and `docs/user-journeys/CATALOG-j126-j150-ecosystem.md`.
Read: Open `docs/user-journeys/CATALOG-j126-j150-ecosystem.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-23` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0293](../decisions/ADR-0293-governance-meta-trust-root.md) in the artifact.

### Day 3 step 24
Goal: Learn the personas, journeys, and continuity of identity path by tying `Workflow` to ADR-0294 and `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md`.
Read: Open `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-24` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0294](../decisions/ADR-0294-cedar-fragment-soak-anomaly-rollback.md) in the artifact.

### Day 3 step 25
Goal: Learn the personas, journeys, and continuity of identity path by tying `Ontology` to ADR-0295 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-25` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0295](../decisions/ADR-0295-bootstrap-ci-spiffe-kill-switch.md) in the artifact.

### Day 3 step 26
Goal: Learn the personas, journeys, and continuity of identity path by tying `Audit-chain` to ADR-0296 and `docs/user-journeys/CATALOG-j126-j150-ecosystem.md`.
Read: Open `docs/user-journeys/CATALOG-j126-j150-ecosystem.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-26` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0296](../decisions/ADR-0296-library-first-credential-sidecar.md) in the artifact.

### Day 3 step 27
Goal: Learn the personas, journeys, and continuity of identity path by tying `Role projection` to ADR-0311 and `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md`.
Read: Open `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-27` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0311](../decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md) in the artifact.

### Day 3 step 28
Goal: Learn the personas, journeys, and continuity of identity path by tying `Capability tier` to ADR-0313 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-28` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0313](../decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md) in the artifact.

### Day 3 step 29
Goal: Learn the personas, journeys, and continuity of identity path by tying `Dual-tenant boundary` to ADR-0316 and `docs/user-journeys/CATALOG-j126-j150-ecosystem.md`.
Read: Open `docs/user-journeys/CATALOG-j126-j150-ecosystem.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-29` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0316](../decisions/ADR-0316-capability-tier-over-product-fragmentation.md) in the artifact.

### Day 3 step 30
Goal: Learn the personas, journeys, and continuity of identity path by tying `Transient identity` to ADR-0317 and `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md`.
Read: Open `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-30` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0317](../decisions/ADR-0317-role-based-projection-unified-ux-shell.md) in the artifact.

### Day 3 step 31
Goal: Learn the personas, journeys, and continuity of identity path by tying `Tenant` to ADR-0242 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-31` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0242](../decisions/ADR-0242-oyatie-is-a-tenant-doctrine.md) in the artifact.

### Day 3 step 32
Goal: Learn the personas, journeys, and continuity of identity path by tying `Principal` to ADR-0243 and `docs/user-journeys/CATALOG-j126-j150-ecosystem.md`.
Read: Open `docs/user-journeys/CATALOG-j126-j150-ecosystem.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-32` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0243](../decisions/ADR-0243-cedar-as-universal-gate.md) in the artifact.

### Day 3 step 33
Goal: Learn the personas, journeys, and continuity of identity path by tying `Cedar permit` to ADR-0244 and `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md`.
Read: Open `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-33` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0244](../decisions/ADR-0244-tenant-as-universal-scoping-primitive.md) in the artifact.

### Day 3 step 34
Goal: Learn the personas, journeys, and continuity of identity path by tying `Workflow` to ADR-0245 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-34` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0245](../decisions/ADR-0245-substrate-vs-product-layering.md) in the artifact.

### Day 3 step 35
Goal: Learn the personas, journeys, and continuity of identity path by tying `Ontology` to ADR-0246 and `docs/user-journeys/CATALOG-j126-j150-ecosystem.md`.
Read: Open `docs/user-journeys/CATALOG-j126-j150-ecosystem.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-35` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0246](../decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 3 step 36
Goal: Learn the personas, journeys, and continuity of identity path by tying `Audit-chain` to ADR-0247 and `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md`.
Read: Open `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-36` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0247](../decisions/ADR-0247-self-hosting-self-modification-doctrine.md) in the artifact.

### Day 3 step 37
Goal: Learn the personas, journeys, and continuity of identity path by tying `Role projection` to ADR-0248 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-37` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0248](../decisions/ADR-0248-amazon-shape-cellular-architecture.md) in the artifact.

### Day 3 step 38
Goal: Learn the personas, journeys, and continuity of identity path by tying `Capability tier` to ADR-0249 and `docs/user-journeys/CATALOG-j126-j150-ecosystem.md`.
Read: Open `docs/user-journeys/CATALOG-j126-j150-ecosystem.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-38` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0249](../decisions/ADR-0249-multi-category-marketplace-doctrine.md) in the artifact.

### Day 3 step 39
Goal: Learn the personas, journeys, and continuity of identity path by tying `Dual-tenant boundary` to ADR-0250 and `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md`.
Read: Open `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-39` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0250](../decisions/ADR-0250-build-ahead-of-certification-doctrine.md) in the artifact.

### Day 3 step 40
Goal: Learn the personas, journeys, and continuity of identity path by tying `Transient identity` to ADR-0251 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-40` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0251](../decisions/ADR-0251-compliance-pack-cell-certification-levels.md) in the artifact.

### Day 3 step 41
Goal: Learn the personas, journeys, and continuity of identity path by tying `Tenant` to ADR-0252 and `docs/user-journeys/CATALOG-j126-j150-ecosystem.md`.
Read: Open `docs/user-journeys/CATALOG-j126-j150-ecosystem.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-41` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0252](../decisions/ADR-0252-time-coordination-distributed-consistency.md) in the artifact.

### Day 3 step 42
Goal: Learn the personas, journeys, and continuity of identity path by tying `Principal` to ADR-0253 and `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md`.
Read: Open `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-42` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0253](../decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) in the artifact.

### Day 3 step 43
Goal: Learn the personas, journeys, and continuity of identity path by tying `Cedar permit` to ADR-0254 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-43` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0254](../decisions/ADR-0254-deployment-model-spectrum.md) in the artifact.

### Day 3 step 44
Goal: Learn the personas, journeys, and continuity of identity path by tying `Workflow` to ADR-0255 and `docs/user-journeys/CATALOG-j126-j150-ecosystem.md`.
Read: Open `docs/user-journeys/CATALOG-j126-j150-ecosystem.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-44` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0255](../decisions/ADR-0355-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 3 step 45
Goal: Learn the personas, journeys, and continuity of identity path by tying `Ontology` to ADR-0257 and `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md`.
Read: Open `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-45` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0257](../decisions/ADR-0356-amendment-library-first-ontology-read-path.md) in the artifact.

### Day 3 step 46
Goal: Learn the personas, journeys, and continuity of identity path by tying `Audit-chain` to ADR-0258 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-46` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0258](../decisions/ADR-0258-api-versioning-model.md) in the artifact.

### Day 3 step 47
Goal: Learn the personas, journeys, and continuity of identity path by tying `Role projection` to ADR-0263 and `docs/user-journeys/CATALOG-j126-j150-ecosystem.md`.
Read: Open `docs/user-journeys/CATALOG-j126-j150-ecosystem.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-47` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0263](../decisions/ADR-0263-observability-emission-contract.md) in the artifact.

### Day 3 step 48
Goal: Learn the personas, journeys, and continuity of identity path by tying `Capability tier` to ADR-0273 and `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md`.
Read: Open `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-48` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0273](../decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md) in the artifact.

### Day 3 step 49
Goal: Learn the personas, journeys, and continuity of identity path by tying `Dual-tenant boundary` to ADR-0276 and `docs/personas/MASTER-ROSTER-2026-05-21.md`.
Read: Open `docs/personas/MASTER-ROSTER-2026-05-21.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-49` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0276](../decisions/ADR-0276-backup-portability-format-gdpr-article-20.md) in the artifact.

### Day 3 step 50
Goal: Learn the personas, journeys, and continuity of identity path by tying `Transient identity` to ADR-0280 and `docs/user-journeys/CATALOG-j126-j150-ecosystem.md`.
Read: Open `docs/user-journeys/CATALOG-j126-j150-ecosystem.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-3-50` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0280](../decisions/ADR-0280-substrate-of-substrate-dependency-doctrine.md) in the artifact.

## Day 4. Keystone ADR deep dive and decision-trace discipline

### Day 4 step 01
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Tenant` to ADR-0242 and `docs/architecture/keystone-bundle-reading-order.md`.
Read: Open `docs/architecture/keystone-bundle-reading-order.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-01` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0242](../decisions/ADR-0242-oyatie-is-a-tenant-doctrine.md) in the artifact.

### Day 4 step 02
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Principal` to ADR-0243 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-02` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0243](../decisions/ADR-0243-cedar-as-universal-gate.md) in the artifact.

### Day 4 step 03
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Cedar permit` to ADR-0244 and `docs/onboarding/doctrine-bootcamp-2026-05-21.md`.
Read: Open `docs/onboarding/doctrine-bootcamp-2026-05-21.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-03` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0244](../decisions/ADR-0244-tenant-as-universal-scoping-primitive.md) in the artifact.

### Day 4 step 04
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Workflow` to ADR-0245 and `docs/architecture/keystone-bundle-reading-order.md`.
Read: Open `docs/architecture/keystone-bundle-reading-order.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-04` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0245](../decisions/ADR-0245-substrate-vs-product-layering.md) in the artifact.

### Day 4 step 05
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Ontology` to ADR-0246 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-05` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0246](../decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 4 step 06
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Audit-chain` to ADR-0247 and `docs/onboarding/doctrine-bootcamp-2026-05-21.md`.
Read: Open `docs/onboarding/doctrine-bootcamp-2026-05-21.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-06` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0247](../decisions/ADR-0247-self-hosting-self-modification-doctrine.md) in the artifact.

### Day 4 step 07
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Role projection` to ADR-0248 and `docs/architecture/keystone-bundle-reading-order.md`.
Read: Open `docs/architecture/keystone-bundle-reading-order.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-07` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0248](../decisions/ADR-0248-amazon-shape-cellular-architecture.md) in the artifact.

### Day 4 step 08
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Capability tier` to ADR-0249 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-08` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0249](../decisions/ADR-0249-multi-category-marketplace-doctrine.md) in the artifact.

### Day 4 step 09
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Dual-tenant boundary` to ADR-0250 and `docs/onboarding/doctrine-bootcamp-2026-05-21.md`.
Read: Open `docs/onboarding/doctrine-bootcamp-2026-05-21.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-09` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0250](../decisions/ADR-0250-build-ahead-of-certification-doctrine.md) in the artifact.

### Day 4 step 10
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Transient identity` to ADR-0251 and `docs/architecture/keystone-bundle-reading-order.md`.
Read: Open `docs/architecture/keystone-bundle-reading-order.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-10` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0251](../decisions/ADR-0251-compliance-pack-cell-certification-levels.md) in the artifact.

### Day 4 step 11
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Tenant` to ADR-0252 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-11` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0252](../decisions/ADR-0252-time-coordination-distributed-consistency.md) in the artifact.

### Day 4 step 12
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Principal` to ADR-0253 and `docs/onboarding/doctrine-bootcamp-2026-05-21.md`.
Read: Open `docs/onboarding/doctrine-bootcamp-2026-05-21.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-12` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0253](../decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) in the artifact.

### Day 4 step 13
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Cedar permit` to ADR-0254 and `docs/architecture/keystone-bundle-reading-order.md`.
Read: Open `docs/architecture/keystone-bundle-reading-order.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-13` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0254](../decisions/ADR-0254-deployment-model-spectrum.md) in the artifact.

### Day 4 step 14
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Workflow` to ADR-0255 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-14` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0255](../decisions/ADR-0355-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 4 step 15
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Ontology` to ADR-0257 and `docs/onboarding/doctrine-bootcamp-2026-05-21.md`.
Read: Open `docs/onboarding/doctrine-bootcamp-2026-05-21.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-15` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0257](../decisions/ADR-0356-amendment-library-first-ontology-read-path.md) in the artifact.

### Day 4 step 16
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Audit-chain` to ADR-0258 and `docs/architecture/keystone-bundle-reading-order.md`.
Read: Open `docs/architecture/keystone-bundle-reading-order.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-16` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0258](../decisions/ADR-0258-api-versioning-model.md) in the artifact.

### Day 4 step 17
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Role projection` to ADR-0263 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-17` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0263](../decisions/ADR-0263-observability-emission-contract.md) in the artifact.

### Day 4 step 18
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Capability tier` to ADR-0273 and `docs/onboarding/doctrine-bootcamp-2026-05-21.md`.
Read: Open `docs/onboarding/doctrine-bootcamp-2026-05-21.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-18` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0273](../decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md) in the artifact.

### Day 4 step 19
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Dual-tenant boundary` to ADR-0276 and `docs/architecture/keystone-bundle-reading-order.md`.
Read: Open `docs/architecture/keystone-bundle-reading-order.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-19` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0276](../decisions/ADR-0276-backup-portability-format-gdpr-article-20.md) in the artifact.

### Day 4 step 20
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Transient identity` to ADR-0280 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-20` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0280](../decisions/ADR-0280-substrate-of-substrate-dependency-doctrine.md) in the artifact.

### Day 4 step 21
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Tenant` to ADR-0284 and `docs/onboarding/doctrine-bootcamp-2026-05-21.md`.
Read: Open `docs/onboarding/doctrine-bootcamp-2026-05-21.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-21` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0284](../decisions/ADR-0284-platform-owner-name-indirection.md) in the artifact.

### Day 4 step 22
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Principal` to ADR-0292 and `docs/architecture/keystone-bundle-reading-order.md`.
Read: Open `docs/architecture/keystone-bundle-reading-order.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-22` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0292](../decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md) in the artifact.

### Day 4 step 23
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Cedar permit` to ADR-0293 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-23` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0293](../decisions/ADR-0293-governance-meta-trust-root.md) in the artifact.

### Day 4 step 24
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Workflow` to ADR-0294 and `docs/onboarding/doctrine-bootcamp-2026-05-21.md`.
Read: Open `docs/onboarding/doctrine-bootcamp-2026-05-21.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-24` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0294](../decisions/ADR-0294-cedar-fragment-soak-anomaly-rollback.md) in the artifact.

### Day 4 step 25
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Ontology` to ADR-0295 and `docs/architecture/keystone-bundle-reading-order.md`.
Read: Open `docs/architecture/keystone-bundle-reading-order.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-25` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0295](../decisions/ADR-0295-bootstrap-ci-spiffe-kill-switch.md) in the artifact.

### Day 4 step 26
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Audit-chain` to ADR-0296 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-26` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0296](../decisions/ADR-0296-library-first-credential-sidecar.md) in the artifact.

### Day 4 step 27
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Role projection` to ADR-0311 and `docs/onboarding/doctrine-bootcamp-2026-05-21.md`.
Read: Open `docs/onboarding/doctrine-bootcamp-2026-05-21.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-27` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0311](../decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md) in the artifact.

### Day 4 step 28
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Capability tier` to ADR-0313 and `docs/architecture/keystone-bundle-reading-order.md`.
Read: Open `docs/architecture/keystone-bundle-reading-order.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-28` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0313](../decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md) in the artifact.

### Day 4 step 29
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Dual-tenant boundary` to ADR-0316 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-29` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0316](../decisions/ADR-0316-capability-tier-over-product-fragmentation.md) in the artifact.

### Day 4 step 30
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Transient identity` to ADR-0317 and `docs/onboarding/doctrine-bootcamp-2026-05-21.md`.
Read: Open `docs/onboarding/doctrine-bootcamp-2026-05-21.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-30` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0317](../decisions/ADR-0317-role-based-projection-unified-ux-shell.md) in the artifact.

### Day 4 step 31
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Tenant` to ADR-0242 and `docs/architecture/keystone-bundle-reading-order.md`.
Read: Open `docs/architecture/keystone-bundle-reading-order.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-31` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0242](../decisions/ADR-0242-oyatie-is-a-tenant-doctrine.md) in the artifact.

### Day 4 step 32
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Principal` to ADR-0243 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-32` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0243](../decisions/ADR-0243-cedar-as-universal-gate.md) in the artifact.

### Day 4 step 33
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Cedar permit` to ADR-0244 and `docs/onboarding/doctrine-bootcamp-2026-05-21.md`.
Read: Open `docs/onboarding/doctrine-bootcamp-2026-05-21.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-33` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0244](../decisions/ADR-0244-tenant-as-universal-scoping-primitive.md) in the artifact.

### Day 4 step 34
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Workflow` to ADR-0245 and `docs/architecture/keystone-bundle-reading-order.md`.
Read: Open `docs/architecture/keystone-bundle-reading-order.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-34` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0245](../decisions/ADR-0245-substrate-vs-product-layering.md) in the artifact.

### Day 4 step 35
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Ontology` to ADR-0246 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-35` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0246](../decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 4 step 36
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Audit-chain` to ADR-0247 and `docs/onboarding/doctrine-bootcamp-2026-05-21.md`.
Read: Open `docs/onboarding/doctrine-bootcamp-2026-05-21.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-36` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0247](../decisions/ADR-0247-self-hosting-self-modification-doctrine.md) in the artifact.

### Day 4 step 37
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Role projection` to ADR-0248 and `docs/architecture/keystone-bundle-reading-order.md`.
Read: Open `docs/architecture/keystone-bundle-reading-order.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-37` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0248](../decisions/ADR-0248-amazon-shape-cellular-architecture.md) in the artifact.

### Day 4 step 38
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Capability tier` to ADR-0249 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-38` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0249](../decisions/ADR-0249-multi-category-marketplace-doctrine.md) in the artifact.

### Day 4 step 39
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Dual-tenant boundary` to ADR-0250 and `docs/onboarding/doctrine-bootcamp-2026-05-21.md`.
Read: Open `docs/onboarding/doctrine-bootcamp-2026-05-21.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-39` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0250](../decisions/ADR-0250-build-ahead-of-certification-doctrine.md) in the artifact.

### Day 4 step 40
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Transient identity` to ADR-0251 and `docs/architecture/keystone-bundle-reading-order.md`.
Read: Open `docs/architecture/keystone-bundle-reading-order.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-40` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0251](../decisions/ADR-0251-compliance-pack-cell-certification-levels.md) in the artifact.

### Day 4 step 41
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Tenant` to ADR-0252 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-41` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0252](../decisions/ADR-0252-time-coordination-distributed-consistency.md) in the artifact.

### Day 4 step 42
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Principal` to ADR-0253 and `docs/onboarding/doctrine-bootcamp-2026-05-21.md`.
Read: Open `docs/onboarding/doctrine-bootcamp-2026-05-21.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-42` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0253](../decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) in the artifact.

### Day 4 step 43
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Cedar permit` to ADR-0254 and `docs/architecture/keystone-bundle-reading-order.md`.
Read: Open `docs/architecture/keystone-bundle-reading-order.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-43` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0254](../decisions/ADR-0254-deployment-model-spectrum.md) in the artifact.

### Day 4 step 44
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Workflow` to ADR-0255 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-44` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0255](../decisions/ADR-0355-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 4 step 45
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Ontology` to ADR-0257 and `docs/onboarding/doctrine-bootcamp-2026-05-21.md`.
Read: Open `docs/onboarding/doctrine-bootcamp-2026-05-21.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-45` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0257](../decisions/ADR-0356-amendment-library-first-ontology-read-path.md) in the artifact.

### Day 4 step 46
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Audit-chain` to ADR-0258 and `docs/architecture/keystone-bundle-reading-order.md`.
Read: Open `docs/architecture/keystone-bundle-reading-order.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-46` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0258](../decisions/ADR-0258-api-versioning-model.md) in the artifact.

### Day 4 step 47
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Role projection` to ADR-0263 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-47` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0263](../decisions/ADR-0263-observability-emission-contract.md) in the artifact.

### Day 4 step 48
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Capability tier` to ADR-0273 and `docs/onboarding/doctrine-bootcamp-2026-05-21.md`.
Read: Open `docs/onboarding/doctrine-bootcamp-2026-05-21.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-48` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0273](../decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md) in the artifact.

### Day 4 step 49
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Dual-tenant boundary` to ADR-0276 and `docs/architecture/keystone-bundle-reading-order.md`.
Read: Open `docs/architecture/keystone-bundle-reading-order.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-49` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0276](../decisions/ADR-0276-backup-portability-format-gdpr-article-20.md) in the artifact.

### Day 4 step 50
Goal: Learn the keystone adr deep dive and decision-trace discipline path by tying `Transient identity` to ADR-0280 and `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`.
Read: Open `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-4-50` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0280](../decisions/ADR-0280-substrate-of-substrate-dependency-doctrine.md) in the artifact.

## Day 5. Run a slice end-to-end and ship a contribution

### Day 5 step 01
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Tenant` to ADR-0242 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-01` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0242](../decisions/ADR-0242-oyatie-is-a-tenant-doctrine.md) in the artifact.

### Day 5 step 02
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Principal` to ADR-0243 and `templates/checklists/done-definition-checklist.md`.
Read: Open `templates/checklists/done-definition-checklist.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-02` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0243](../decisions/ADR-0243-cedar-as-universal-gate.md) in the artifact.

### Day 5 step 03
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Cedar permit` to ADR-0244 and `docs/GLOSSARY.md`.
Read: Open `docs/GLOSSARY.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-03` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0244](../decisions/ADR-0244-tenant-as-universal-scoping-primitive.md) in the artifact.

### Day 5 step 04
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Workflow` to ADR-0245 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-04` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0245](../decisions/ADR-0245-substrate-vs-product-layering.md) in the artifact.

### Day 5 step 05
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Ontology` to ADR-0246 and `templates/checklists/done-definition-checklist.md`.
Read: Open `templates/checklists/done-definition-checklist.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-05` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0246](../decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 5 step 06
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Audit-chain` to ADR-0247 and `docs/GLOSSARY.md`.
Read: Open `docs/GLOSSARY.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-06` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0247](../decisions/ADR-0247-self-hosting-self-modification-doctrine.md) in the artifact.

### Day 5 step 07
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Role projection` to ADR-0248 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-07` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0248](../decisions/ADR-0248-amazon-shape-cellular-architecture.md) in the artifact.

### Day 5 step 08
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Capability tier` to ADR-0249 and `templates/checklists/done-definition-checklist.md`.
Read: Open `templates/checklists/done-definition-checklist.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-08` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0249](../decisions/ADR-0249-multi-category-marketplace-doctrine.md) in the artifact.

### Day 5 step 09
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Dual-tenant boundary` to ADR-0250 and `docs/GLOSSARY.md`.
Read: Open `docs/GLOSSARY.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-09` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0250](../decisions/ADR-0250-build-ahead-of-certification-doctrine.md) in the artifact.

### Day 5 step 10
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Transient identity` to ADR-0251 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-10` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0251](../decisions/ADR-0251-compliance-pack-cell-certification-levels.md) in the artifact.

### Day 5 step 11
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Tenant` to ADR-0252 and `templates/checklists/done-definition-checklist.md`.
Read: Open `templates/checklists/done-definition-checklist.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-11` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0252](../decisions/ADR-0252-time-coordination-distributed-consistency.md) in the artifact.

### Day 5 step 12
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Principal` to ADR-0253 and `docs/GLOSSARY.md`.
Read: Open `docs/GLOSSARY.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-12` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0253](../decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) in the artifact.

### Day 5 step 13
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Cedar permit` to ADR-0254 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-13` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0254](../decisions/ADR-0254-deployment-model-spectrum.md) in the artifact.

### Day 5 step 14
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Workflow` to ADR-0255 and `templates/checklists/done-definition-checklist.md`.
Read: Open `templates/checklists/done-definition-checklist.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-14` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0255](../decisions/ADR-0355-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 5 step 15
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Ontology` to ADR-0257 and `docs/GLOSSARY.md`.
Read: Open `docs/GLOSSARY.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-15` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0257](../decisions/ADR-0356-amendment-library-first-ontology-read-path.md) in the artifact.

### Day 5 step 16
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Audit-chain` to ADR-0258 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-16` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0258](../decisions/ADR-0258-api-versioning-model.md) in the artifact.

### Day 5 step 17
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Role projection` to ADR-0263 and `templates/checklists/done-definition-checklist.md`.
Read: Open `templates/checklists/done-definition-checklist.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-17` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0263](../decisions/ADR-0263-observability-emission-contract.md) in the artifact.

### Day 5 step 18
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Capability tier` to ADR-0273 and `docs/GLOSSARY.md`.
Read: Open `docs/GLOSSARY.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-18` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0273](../decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md) in the artifact.

### Day 5 step 19
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Dual-tenant boundary` to ADR-0276 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-19` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0276](../decisions/ADR-0276-backup-portability-format-gdpr-article-20.md) in the artifact.

### Day 5 step 20
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Transient identity` to ADR-0280 and `templates/checklists/done-definition-checklist.md`.
Read: Open `templates/checklists/done-definition-checklist.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-20` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0280](../decisions/ADR-0280-substrate-of-substrate-dependency-doctrine.md) in the artifact.

### Day 5 step 21
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Tenant` to ADR-0284 and `docs/GLOSSARY.md`.
Read: Open `docs/GLOSSARY.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-21` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0284](../decisions/ADR-0284-platform-owner-name-indirection.md) in the artifact.

### Day 5 step 22
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Principal` to ADR-0292 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-22` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0292](../decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md) in the artifact.

### Day 5 step 23
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Cedar permit` to ADR-0293 and `templates/checklists/done-definition-checklist.md`.
Read: Open `templates/checklists/done-definition-checklist.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-23` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0293](../decisions/ADR-0293-governance-meta-trust-root.md) in the artifact.

### Day 5 step 24
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Workflow` to ADR-0294 and `docs/GLOSSARY.md`.
Read: Open `docs/GLOSSARY.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-24` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0294](../decisions/ADR-0294-cedar-fragment-soak-anomaly-rollback.md) in the artifact.

### Day 5 step 25
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Ontology` to ADR-0295 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-25` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0295](../decisions/ADR-0295-bootstrap-ci-spiffe-kill-switch.md) in the artifact.

### Day 5 step 26
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Audit-chain` to ADR-0296 and `templates/checklists/done-definition-checklist.md`.
Read: Open `templates/checklists/done-definition-checklist.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-26` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0296](../decisions/ADR-0296-library-first-credential-sidecar.md) in the artifact.

### Day 5 step 27
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Role projection` to ADR-0311 and `docs/GLOSSARY.md`.
Read: Open `docs/GLOSSARY.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-27` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0311](../decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md) in the artifact.

### Day 5 step 28
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Capability tier` to ADR-0313 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-28` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0313](../decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md) in the artifact.

### Day 5 step 29
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Dual-tenant boundary` to ADR-0316 and `templates/checklists/done-definition-checklist.md`.
Read: Open `templates/checklists/done-definition-checklist.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-29` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0316](../decisions/ADR-0316-capability-tier-over-product-fragmentation.md) in the artifact.

### Day 5 step 30
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Transient identity` to ADR-0317 and `docs/GLOSSARY.md`.
Read: Open `docs/GLOSSARY.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-30` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0317](../decisions/ADR-0317-role-based-projection-unified-ux-shell.md) in the artifact.

### Day 5 step 31
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Tenant` to ADR-0242 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-31` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0242](../decisions/ADR-0242-oyatie-is-a-tenant-doctrine.md) in the artifact.

### Day 5 step 32
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Principal` to ADR-0243 and `templates/checklists/done-definition-checklist.md`.
Read: Open `templates/checklists/done-definition-checklist.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-32` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0243](../decisions/ADR-0243-cedar-as-universal-gate.md) in the artifact.

### Day 5 step 33
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Cedar permit` to ADR-0244 and `docs/GLOSSARY.md`.
Read: Open `docs/GLOSSARY.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-33` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0244](../decisions/ADR-0244-tenant-as-universal-scoping-primitive.md) in the artifact.

### Day 5 step 34
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Workflow` to ADR-0245 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-34` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0245](../decisions/ADR-0245-substrate-vs-product-layering.md) in the artifact.

### Day 5 step 35
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Ontology` to ADR-0246 and `templates/checklists/done-definition-checklist.md`.
Read: Open `templates/checklists/done-definition-checklist.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-35` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0246](../decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 5 step 36
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Audit-chain` to ADR-0247 and `docs/GLOSSARY.md`.
Read: Open `docs/GLOSSARY.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-36` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0247](../decisions/ADR-0247-self-hosting-self-modification-doctrine.md) in the artifact.

### Day 5 step 37
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Role projection` to ADR-0248 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-37` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0248](../decisions/ADR-0248-amazon-shape-cellular-architecture.md) in the artifact.

### Day 5 step 38
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Capability tier` to ADR-0249 and `templates/checklists/done-definition-checklist.md`.
Read: Open `templates/checklists/done-definition-checklist.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-38` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0249](../decisions/ADR-0249-multi-category-marketplace-doctrine.md) in the artifact.

### Day 5 step 39
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Dual-tenant boundary` to ADR-0250 and `docs/GLOSSARY.md`.
Read: Open `docs/GLOSSARY.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-39` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0250](../decisions/ADR-0250-build-ahead-of-certification-doctrine.md) in the artifact.

### Day 5 step 40
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Transient identity` to ADR-0251 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-40` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0251](../decisions/ADR-0251-compliance-pack-cell-certification-levels.md) in the artifact.

### Day 5 step 41
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Tenant` to ADR-0252 and `templates/checklists/done-definition-checklist.md`.
Read: Open `templates/checklists/done-definition-checklist.md` and record one sentence that explains how it constrains `Tenant`.
Do: Run `rg -n "Tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-41` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Tenant` and [ADR-0252](../decisions/ADR-0252-time-coordination-distributed-consistency.md) in the artifact.

### Day 5 step 42
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Principal` to ADR-0253 and `docs/GLOSSARY.md`.
Read: Open `docs/GLOSSARY.md` and record one sentence that explains how it constrains `Principal`.
Do: Run `rg -n "Principal" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-42` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Principal` and [ADR-0253](../decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) in the artifact.

### Day 5 step 43
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Cedar permit` to ADR-0254 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Cedar permit`.
Do: Run `rg -n "Cedar" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-43` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Cedar permit` and [ADR-0254](../decisions/ADR-0254-deployment-model-spectrum.md) in the artifact.

### Day 5 step 44
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Workflow` to ADR-0255 and `templates/checklists/done-definition-checklist.md`.
Read: Open `templates/checklists/done-definition-checklist.md` and record one sentence that explains how it constrains `Workflow`.
Do: Run `rg -n "Workflow" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-44` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Workflow` and [ADR-0255](../decisions/ADR-0355-amendment-library-first-network-opt-in-clarification.md) in the artifact.

### Day 5 step 45
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Ontology` to ADR-0257 and `docs/GLOSSARY.md`.
Read: Open `docs/GLOSSARY.md` and record one sentence that explains how it constrains `Ontology`.
Do: Run `rg -n "Ontology" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-45` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Ontology` and [ADR-0257](../decisions/ADR-0356-amendment-library-first-ontology-read-path.md) in the artifact.

### Day 5 step 46
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Audit-chain` to ADR-0258 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Audit-chain`.
Do: Run `rg -n "Audit-chain" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-46` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Audit-chain` and [ADR-0258](../decisions/ADR-0258-api-versioning-model.md) in the artifact.

### Day 5 step 47
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Role projection` to ADR-0263 and `templates/checklists/done-definition-checklist.md`.
Read: Open `templates/checklists/done-definition-checklist.md` and record one sentence that explains how it constrains `Role projection`.
Do: Run `rg -n "Role" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-47` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Role projection` and [ADR-0263](../decisions/ADR-0263-observability-emission-contract.md) in the artifact.

### Day 5 step 48
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Capability tier` to ADR-0273 and `docs/GLOSSARY.md`.
Read: Open `docs/GLOSSARY.md` and record one sentence that explains how it constrains `Capability tier`.
Do: Run `rg -n "Capability" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-48` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Capability tier` and [ADR-0273](../decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md) in the artifact.

### Day 5 step 49
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Dual-tenant boundary` to ADR-0276 and `docs/AGENTS.md`.
Read: Open `docs/AGENTS.md` and record one sentence that explains how it constrains `Dual-tenant boundary`.
Do: Run `rg -n "Dual-tenant" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-49` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Dual-tenant boundary` and [ADR-0276](../decisions/ADR-0276-backup-portability-format-gdpr-article-20.md) in the artifact.

### Day 5 step 50
Goal: Learn the run a slice end-to-end and ship a contribution path by tying `Transient identity` to ADR-0280 and `templates/checklists/done-definition-checklist.md`.
Read: Open `templates/checklists/done-definition-checklist.md` and record one sentence that explains how it constrains `Transient identity`.
Do: Run `rg -n "Transient" docs specs crates | sed -n '1,20p'` and choose one concrete source location.
Artifact: Commit-note draft `week-one-day-5-50` with source path, glossary term, binding ADR, and expected reviewer.
Verification: Reviewer can open the cited path, confirm the term is used consistently, and reproduce the command without hidden setup.
Escalation: If the source path contradicts the glossary, pause that step and file a doc-drift note to `doc-style-reviewer` plus `council-architecture`.
Glossary cross-reference: cite [GLOSSARY.md](../GLOSSARY.md) row `Transient identity` and [ADR-0280](../decisions/ADR-0280-substrate-of-substrate-dependency-doctrine.md) in the artifact.

## C. Week-one exit checklist

1. Can explain tenant, principal, Cedar permit, workflow, ontology, and audit-chain without inventing synonyms.
   Artifact: `week-one-exit-1` note with path, command, observed output, reviewer, and glossary rows used.
2. Can trace one persona through personal and work tenant contexts without breaking the dual-tenant boundary.
   Artifact: `week-one-exit-2` note with path, command, observed output, reviewer, and glossary rows used.
3. Can prepare a first documentation PR with binding ADR citations and a verifiable artifact per step.
   Artifact: `week-one-exit-3` note with path, command, observed output, reviewer, and glossary rows used.
4. Can run targeted glossary and line-count checks and paste evidence into a review note.
   Artifact: `week-one-exit-4` note with path, command, observed output, reviewer, and glossary rows used.
5. Can identify whether a question belongs to the glossary, ADR, standard, or implementation spec.
   Artifact: `week-one-exit-5` note with path, command, observed output, reviewer, and glossary rows used.
