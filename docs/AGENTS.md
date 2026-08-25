---
doc_class: Redirect
shape: redirect
length_cap: 90
authority_tier: 3
status: Accepted
purpose: Route legacy docs/AGENTS readers to the live ADR-0719 authority surfaces.
redirect_to: ../AGENTS.md
---

# Moved

The live session hubs are root [`AGENTS.md`](../AGENTS.md) and [`CLAUDE.md`](../CLAUDE.md); they define the trust boundary and merge path.

For a capability or `app/<product>/`, follow those hubs and open the owner's `ADR.md`, `PRD.md`, `SPEC.md`, and `PLAN.md`.

The owner-law migration is staged. If any of those four files is absent, do
not substitute the old root wiki or deleted `specs/` corpus. Load the owner's
files that exist plus the root hubs and applicable accepted 07xx decisions; add missing law only in that owner's structural lane.

[ADR-0719 D-17](decisions/ADR-0719-eac-serving-control-north-star.md#d-17--presubmit-is-the-graph-not-a-json-product)
deleted `specs/` with no replacement JSON hub; D-27 retains this file only as a thin operating-contract pointer.

## Compatibility anchors

These headings route legacy references; they do not restore the old contract.

## Doctrine survival

Live law survives in the [root hubs](../AGENTS.md) and migrated owner files, not in a global plan, catalog, or chat transcript.

## Repository topology

Use the owner-root and four-file discovery rule in [root `AGENTS.md`](../AGENTS.md#work).

## Pre-flight checklist

Use items 1–2 under [Work](../AGENTS.md#work). The former numbered checklist is retired.

## During-change discipline

Use items 3–6 under [Work](../AGENTS.md#work).

## Sanctioned primitives

Use the fenced required sequence in [root `AGENTS.md`](../AGENTS.md#merge).

## Per-change-class reviewer agents

The named reviewer registry is retired. Select an independent reviewer for the
risk surface; observation is not APPROVE.

## PR shape

Use the live [pull-request template](../.github/PULL_REQUEST_TEMPLATE.md).

## Boundaries

Stay within the user's authorized scope and the [root work and merge contract](../AGENTS.md).

## Done-Definition

The former D1–D18 labels are retired. Done means exact-head cargo evidence,
independent APPROVE, resolved threads, green `presubmit`, and protected squash merge.

## RFC-2119

Every load-bearing MUST uses the five fields required by [root `AGENTS.md`](../AGENTS.md#work).

## Per-agent appendices

There is no separate repo-local agent appendix; use the two root session hubs.

## changeset

Changeset evidence follows [Work](../AGENTS.md#work) and [Merge](../AGENTS.md#merge).

## Authority survival

- **achieves:** one discoverable authority route with no deleted intermediate
  hub or duplicated operating contract.
- **origin:** the former contract continued to route agents through deleted
  `specs/` and `registry/` files after ADR-0719 D-17 removed them.
- **rule:** agents MUST load the root hubs, then each migrated owner-law file;
  when one is absent they MUST use the staged fallback above. This pointer
  MUST NOT duplicate law or recreate a machine-readable authority tree.
- **ensure:** review resolves every link in this file against the protected
  tree and rejects references to the deleted authority corpus.
- **overturn_when:** a founder-accepted ADR replaces D-17 and updates both root
  session hubs and this pointer in the same change.
