<!--
Canonical authority: templates/pull-request-template.md (TPL-PR) + docs/AGENTS.md §PR shape.
Fill all 5 author-owned H2 sections below before requesting review.
CI (`traceability-validator`) fails the gate if any section is missing or empty.
Do NOT add `## Code Review` — the lead reviewer agent adds it at merge time
(guard-pr-merge-review.mjs refuses worker-added review sections).
-->

## Issue

Closes #<n> — change class: `feature | bugfix | refactor | migration | docs | chore | capability | plugin | runbook | ADR | pack-update`

## Summary

- <!-- 1–3 bullets on what + why; the diff already shows the what -->
- <!-- cite the canonical authority read first per docs/AGENTS.md §Pre-flight checklist item 2 -->

## Verification

<!-- Each line MUST carry PASS/FAIL and an actual output excerpt, not a hand-wave. -->

- `buck2 test <targeted test targets>` — `<PASS|FAIL|N/A>` — `<excerpt>`
- `buck2 build <targeted build targets>` — `<PASS|FAIL|N/A>` — `<excerpt>`
- Supplementary cargo feedback (`cargo nextest run` / `cargo clippy` / `cargo deny check`, local-only) — `<PASS|FAIL|N/A>` — `<excerpt>`
- `oya-ci-required` protected status — `<PASS|FAIL>` — `<status URL or cloud-ci packet excerpt>`
- Per-change-class fitness lanes: `<list lanes + PASS|FAIL each>`
- Per-change-class reviewer agent: `<agent-name>` — verdict `<APPROVE|REQUEST CHANGES>`

## Traceability

- Catalog records touched: `<list under registry/catalog/>`
- Cross-axis contracts touched: `<list under contracts/>`
- ADRs cited: `<ADR-NNNN list>`
- `MISTAKES-LEDGER` row referenced (if regression-class): `MFL-NNNN` or `no prior row`
- Cross-axis review label applied (if cross-axis contract change): `<label>`
- Implementation Plan ID (if executing an IP): `IP-NNN-<slug>`
- Changed paths/symbols (agent path): `<repo-relative path or file::Identifier list>`

## Evidence

- Audit-chain emission ID: `EVT-<topic>-<ulid>`
- Foundation-bypass referenced (if any): `<bypass-id>` + renewal date
- Per-pack regulator-watch impact (if any): `<oya-pack-XX.regulator list>`
- Distroless image build (if shipping a binary): `<image:tag>` + Cosign attestation digest
- SBOM artifact: `<path|registry-ref>`
- SLSA provenance level achieved: `L1 | L2 | L3`
- Agent-observation harvest:
  - source contexts reviewed: `<chat|review notes|scratch|PR|Kanban|N/A>`
  - outcome: `<new/linked Kanban card ids | duplicate/no-action rationale>`
