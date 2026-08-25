---
purpose: Current thin documentation entry point after ADR-0719.
doc_status: published
status: current
---

# Docs hub

Root documentation is an index into law that lives once. Do not restore the
former root wiki, catch-all specs tree, or machine-readable document catalog.

## Start

1. Load the repository-root [`AGENTS.md`](../AGENTS.md) and
   [`CLAUDE.md`](../CLAUDE.md).
2. For a capability or `app/<product>/`, use the owner-law route and staged
   migration fallback in [`AGENTS.md`](AGENTS.md).
3. Read applicable accepted 07xx decisions under [`decisions/`](decisions/).
   [ADR-0719](decisions/ADR-0719-eac-serving-control-north-star.md) defines the
   current repository and documentation shape.
4. Use a standard only when a root hub, an accepted 07xx decision, or migrated
   owner law cites that exact file. D-27 retains [`standards/`](standards/),
   but the directory is not a blanket current authority set.

## Scope

Owner engineering material belongs under `<owner>/docs/`. Root `docs/` retains
the operating-contract pointer, the unique live decision home, and direct
standards. Historical files still awaiting disposition are not authority or
entry points.
