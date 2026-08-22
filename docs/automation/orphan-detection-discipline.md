---
doc_class: DisciplineSpec
shape: discipline
length_cap: 150
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Every file/dir/JSON in the repo carries a `purpose:` field. The
  `governance-orphan-detection` lane (HIGH severity) sweeps every
  `**/*.md`, `**/*.json`, `**/*.toml` and fails any PR that introduces a file
  without a declared purpose, or that leaves a file referenced nowhere. No
  orphans in `main`.
planned_enforcement_ref: governance-orphan-detection
extends_crates:
  - governance-readme-doc-coverage-kernel
  - intelligence-catalog-kernel
  - governance-placeholder-debt-kernel
companion_docs:
  - INDEX.md
  - cross-reference-index-spec.md
doc_status: published
---

# Discipline: orphan detection

> **ADRs:** ADR-0052, ADR-0053, ADR-0054.

## 1. Purpose

An "orphan" is a file or directory that exists in `main` but is referenced by nothing (no INDEX entry, no doc link, no catalog row, no Cargo manifest, no test fixture path). Orphans degrade navigability and signal abandonware. Per MASTERPLAN Directive 10 ("every directory has an INDEX.md or .json; every artifact has a declared `purpose:` in frontmatter"), orphans are a discipline failure.

## 2. The `purpose:` contract

Every file in scope declares a `purpose:` in machine-readable form:

- Markdown (`*.md`): YAML frontmatter `purpose: |` block.
- JSON (`*.json`): top-level `"purpose"` key (or sibling `<filename>.purpose.md`).
- TOML (`*.toml`): top-level `purpose = "..."` key in a `[meta]` table.
- Other files: a sibling `<filename>.purpose.md` is permitted.

## 3. Scope (the lane sweeps)

Every path matching:
- `**/*.md`
- `**/*.json` (excluding `target/`, `node_modules/`, generated `package-lock.json`)
- `**/*.toml` (excluding `Cargo.lock`)

Excluded by ADR-tracked exemption: vendored dirs (`vendor/`, `third_party/`), generated sidecars under `*/generated/`, and Cargo's own files (`Cargo.toml` are catalogued via `governance-cargo-prefix-kernel` and need no per-file `purpose:` field provided the catalog row exists).

## 4. Orphan-detection algorithm

Given the file inventory under scope:

1. Build a reference graph by scanning every Markdown link, every JSON cross-reference (`"path":` fields containing repo-relative paths), every Cargo workspace member entry, and every README/INDEX child link.
2. For each in-scope file, check that it is referenced by ≥ 1 other file OR is itself a root anchor (`MASTERPLAN.md`, `README.md`, `docs/INDEX.md`, repo-root `Cargo.toml`).
3. Orphans = in-scope files with zero inbound references and not a root anchor.

## 5. Validation gates (`governance-orphan-detection`)

1. **Purpose declaration.** Every in-scope file declares `purpose:` (HIGH).
2. **Orphan ban.** Every in-scope file has ≥ 1 inbound reference (HIGH).
3. **Per-directory INDEX presence.** Every directory under `docs/`, `.omc/`, `crates/`, `contracts/`, `infra/` contains an `INDEX.md` or `INDEX.json` (HIGH).
4. **Per-directory `purpose:` coverage.** Each INDEX declares the directory's `purpose:` (HIGH).
5. **Stale-reference detection.** A reference pointing to a non-existent path → BLOCKER (broken link is worse than orphan).

## 6. Trigger matrix

| Event | Action |
|---|---|
| Per-PR | Run sweep on the PR's changed files; PR fails on new orphans or missing `purpose:`. |
| Nightly | Full repo sweep; emit `docs/machine-readable/orphans.json` report. |
| On directory creation | A new dir without an INDEX → BLOCKER. |

## 7. PR-time UX

When the lane fails on a PR, the bot comments:

```
governance-orphan-detection: HIGH

The following files lack a `purpose:` frontmatter field:
- crates/intelligence-new-feature-kernel/Cargo.toml
- docs/runbooks/new-runbook.md

The following files have zero inbound references:
- .omc/drafts/abandoned-draft.md

Action: add `purpose:` to each, or delete the orphan, or open an
ADR-tracked exemption.
```

## 8. Exemptions

Three categories may be exempted via ADR row:

1. **Snapshot archives** (`docs/site/archive/**`): write-once, no inbound references by design.
2. **Vendored deps** (`vendor/`, `third_party/**`): governed by upstream.
3. **Generated sidecars** (`*/generated/**`): tracked under their generator's pipeline.

Every exemption resolves to an ADR id; the ADR cites the exemption rule.

## 9. Cross-reference with cross-reference-index

`cross-reference-index-spec.md` builds the per-doc cross-reference table; this discipline ensures every file lands in that table by having a `purpose:`. The two specs are codependent.

## 10. Out-of-scope

- Quality of `purpose:` field (a one-word `purpose: stub` is allowed; reviewer enforces meaning).
- Cycle detection in reference graph (handled by `dependency-graph-spec.md`).
- Per-axis orphan ownership routing (covered by `docs/RACI-OWNERSHIP.md`).
