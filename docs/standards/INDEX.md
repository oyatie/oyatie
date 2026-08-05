---
purpose: "Catalog of cross-cutting authoring standards under `docs/standards/`."
doc_status: published
---

---
doc_class: StandardsIndex
shape: index
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Catalog of cross-cutting authoring standards under `docs/standards/`. Resolves every
  `<!-- forward-reference: wave-1 -->` and `<!-- forward-reference: wave-2 -->` sentinel
  in `docs/AGENTS.md` and `docs/README.md` that points at
  `standards/<file>.md`. Each row names the file, its enforcement lane, and its
  authority position.
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json
companion_docs:
  - docs/AGENTS.md
  - docs/DOC-CATALOG.md
  - docs/STANDARDS-AND-TEMPLATES.md
authority_chain_declaration: |
  /specs/decision-principles.json + /specs/forbidden-operations.json > rest of docs/ > catalog records > Redirect-class files > working drafts
related_adrs:
  - ADR-0053
  - ADR-0052
  - ADR-0054
---

# Oyatie Standards Index

This directory holds the cross-cutting **authoring standards** that operate within the
[`decision-principles.json`](../../specs/decision-principles.json) + [`forbidden-operations.json`](../../specs/forbidden-operations.json) frame, downstream of the
[`docs/AGENTS.md`](../AGENTS.md) operating contract, and managed under the
[`docs/DOC-CATALOG.md`](../DOC-CATALOG.md) lifecycle protocol.

Standards are normative (RFC-2119 keywords carry force). Every standard names the
fitness lane that enforces it; every standard cites the upstream hyperscaler practice
that informed it where one exists.

## Catalog

| File | Purpose | Enforcement lane | Authority depth |
|---|---|---|---|
| [`doc-style.md`](doc-style.md) | Diátaxis + RFC-2119 + dual-audience + frontmatter shape + heading hierarchy + line length | `oya-governance-doc-style` | Tier 2 |
| [`code-style-rust.md`](code-style-rust.md) | clippy pedantic, `#![deny(unsafe_code)]`, `[workspace.lints]`, naming, kernel←domain←app←{api,worker,adapter}←runtime layering | `oya-governance-clippy-pedantic`, `-workspace-lints-inherit`, `-flat-crates` | Tier 2 |
| [`error-handling.md`](error-handling.md) | `thiserror` in libraries, `anyhow`/`eyre` at binary edge, no `unwrap` outside tests, silent-failure prevention | `oya-governance-error-boundary`, `-no-unwrap-prod` | Tier 2 |
| [`testing.md`](testing.md) | Test pyramid 2.0 + nextest + proptest/quickcheck + cargo-mutants + cargo-fuzz + coverage budget + 14-day flaky SLA | `oya-governance-test-evidence`, `-fuzz-coverage`, `-flaky-sla` | Tier 2 |
| [`security-review.md`](security-review.md) | OWASP + cargo-deny/audit/vet + Sigstore + SBOM + threat-modeling + data-class boundary + autonomy ceiling | `oya-governance-supply-chain`, `-security-review` | Tier 2 |
| [`on-call.md`](on-call.md) | Rotation cadence + runbook discipline + escalation + blameless postmortem trigger + SLO-burn-rate alerting | `oya-governance-runbook-index-resolves`, `-error-budget-gate` | Tier 2 |
| [`multi-agent-tool-map.md`](multi-agent-tool-map.md) | Claude Code / Codex / Gemini / OMC tool-name mapping + sanctioned tools per agent + delegation patterns | `oya-governance-tool-map-cohesion` | Tier 2 |
| [`observability.md`](observability.md) | OpenTelemetry mandatory + tracing/metrics/logs + `EVT-*` audit-chain emission + structured logging schema + Prometheus 3.11+ + exemplars | `oya-governance-otel-emit`, `-audit-emission` | Tier 2 |
| [`release-management.md`](release-management.md) | Trunk-based + canary + feature flags + progressive delivery + SLO-burn-rate auto-rollback + Sigstore-signed releases | `oya-governance-flag-debt`, `-supply-chain`, `-error-budget-gate` | Tier 2 |
| [`dependency-policy.md`](dependency-policy.md) | LTS pinning + license posture + cargo-vet + cargo-deny + Renovate + provider-SDK ProviderAdapter trait | `oya-governance-lts-dependency`, `-cargo-vet`, `-license` | Tier 2 |
| [`image-discipline.md`](image-discipline.md) | distroless-debian13 + musl static linking + image-size budget + Cosign keyless OIDC + SBOM + SLSA L2 | `oya-governance-image-discipline`, `-container-base`, `-supply-chain` | Tier 2 |
| [`data-class.md`](data-class.md) | Every kernel struct field carries `oyatie.data_class`; cross-pillar flow rules; DSR cascade integration | `oya-governance-data-class`, `-dsr-cascade` | Tier 2 |
| [`autonomy-ceiling.md`](autonomy-ceiling.md) | T1/T2/T3/T4 binding + Cedar policy + per-capability autonomy record + config-flag uplift forbidden | `oya-governance-autonomy-ceiling` | Tier 2 |
| [`agent-instructions-discipline.md`](agent-instructions-discipline.md) | `<!-- agent-instructions:start -->` / `<!-- agent-instructions:end -->` fences + banned-token grep scope + documented-rationale flow | `oya-governance-agent-instructions-fence`, `-banned-primitives` | Tier 2 |
| [`autonomous-kanban-lifecycle.md`](autonomous-kanban-lifecycle.md) | Hermes Kanban lifecycle spine, card fields, Review/fix child-card handoff, bounded active work, and duplicate/no-action handling | `planned: hermes-kanban-lifecycle-steward` | Tier 2 |

## Forward-reference resolution map

The following sentinels in `docs/AGENTS.md` and adjacent canonical files are resolved
by this directory. Each row names the sentinel, its source location, and the file
that satisfies it.

| Sentinel target | Source | Resolved by |
|---|---|---|
| `standards/doc-style.md` | decision-principles.json DP-02 (catalog-first authoring); AGENTS.md canonical doc map | `doc-style.md` |
| `standards/error-handling.md` | AGENTS.md §During-change discipline | `error-handling.md` |
| `standards/testing.md` | AGENTS.md §During-change discipline | `testing.md` |
| `standards/on-call.md` | AGENTS.md canonical doc map | `on-call.md` |
| `standards/claude-code-harness.md` | AGENTS.md §Per-agent appendices (Claude Code) | `claude-code-harness.md` |
| `standards/multi-agent-tool-map.md` | AGENTS.md §Per-agent appendices (Gemini) | `multi-agent-tool-map.md` |
| `standards/prevention-doctrine.md` | forbidden-operations.json (anti-overlap, separate standard; not in this batch) | deferred — see §Out-of-scope |

## Out-of-scope (this batch)

- `prevention-doctrine.md` — referenced in forbidden-operations.json but covers a different concern
  (the mechanical-prevention authoring guide); authored separately.
- `commit-message.md`, `code-review.md` — referenced in
  `.omc/scratch/hyperscaler-best-practices-2026-05-12.md` adoption map; future ADR-PM
  rollout will produce these alongside the small-CL discipline rollout.

## Authoring rules for this directory

Every file in this directory:

1. Carries `status: Accepted` and a `date: <ISO-date>` in frontmatter after lift.
2. Declares `enforced_by: <lane-name>` only for active workflow + quality-lane controls, otherwise `planned_enforcement_ref: <lane-name>`, so the fitness-lane registry has a stable back-reference without overclaiming enforcement.
3. Is ≤ 250 lines (this index included).
4. Cites the hyperscaler source by URL where it adopts an upstream practice.
5. Does NOT duplicate content from `docs/AGENTS.md`,
   `docs/DOC-CATALOG.md`, or `docs/STANDARDS-AND-TEMPLATES.md`; it cross-links.

## Status footer

Status: **Accepted** (landed at `docs/standards/` via Stage 1 Wave 2 lift, 2026-05-12).
Sanctioned-primitive ADRs landing in parallel: ADR-0053 (sanctioned primitives),

## Sources scanned

- [`decision-principles.json`](../../specs/decision-principles.json), [`forbidden-operations.json`](../../specs/forbidden-operations.json), [`docs/AGENTS.md`](../AGENTS.md),
  [`docs/DOC-CATALOG.md`](../DOC-CATALOG.md),
  [`docs/STANDARDS-AND-TEMPLATES.md`](../STANDARDS-AND-TEMPLATES.md).
- Historical `.omc/plans/MASTERPLAN.md` §2 Compound principles (Directives 1-12;
  retired from the tracked tree and preserved as git-history provenance only).
- Historical `.omc/scratch/hyperscaler-best-practices-2026-05-12.md` (Domain 2 +
  Domain 3; retired from the tracked tree and preserved as git-history provenance only).
- Historical `.omc/scratch/lts-versions-verified-2026-05-12.md` (retired from the
  tracked tree and preserved as git-history provenance only).
