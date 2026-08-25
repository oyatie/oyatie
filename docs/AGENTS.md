---
doc_class: Redirect
shape: redirect
length_cap: 30
authority_tier: 3
status: Accepted
purpose: Route legacy docs/AGENTS readers to the live ADR-0719 authority surfaces.
redirect_to: ../AGENTS.md
---

# Moved

The live session hubs are the repository-root [`AGENTS.md`](../AGENTS.md) and
[`CLAUDE.md`](../CLAUDE.md). They define the trust boundary, delivery ritual,
verification commands, and protected merge path.

For work inside a capability or `app/<product>/`, follow those hubs and open
that owner's `ADR.md`, `PRD.md`, `SPEC.md`, and `PLAN.md`. Those four files are
the owner-local law for the path.

[ADR-0719 D-17](decisions/ADR-0719-eac-serving-control-north-star.md#d-17--presubmit-is-the-graph-not-a-json-product)
deleted the former `specs/` authority corpus, including
`root-hub-pointers.json`, and requires no replacement JSON hub. D-27 retains
this file only as a thin operating-contract pointer.

## Authority survival

- **achieves:** one discoverable authority route with no deleted intermediate
  hub or duplicated operating contract.
- **origin:** the former contract continued to route agents through deleted
  `specs/` and `registry/` files after ADR-0719 D-17 removed them.
- **rule:** agents MUST load the root session hubs, then the exact four law
  files for the owner they are changing; this redirect MUST NOT grow a second
  copy of that law or recreate a catch-all machine-readable authority tree.
- **ensure:** review resolves every link in this file against the protected
  tree and rejects references to the deleted authority corpus.
- **overturn_when:** a founder-accepted ADR replaces D-17 and updates both root
  session hubs and this pointer in the same change.
