---
doc_class: Standard
shape: ~
length_cap: 250
authority_tier: 2
status: pending approval
purpose: |
  Canonical house style for every doc under `docs/`. Defines the Diátaxis quadrants,
  RFC-2119 normative-language discipline, the dual-audience rule, frontmatter shapes
  per doc-class, heading hierarchy, and line-length guidance. Resolves the
  `standards/doc-style.md` forward-reference sentinel in
  `docs/CONSTITUTION.md` §Documentation, `docs/AGENTS.md` canonical doc map, and
  `docs/README.md`.
lift_target: oyatie/docs/standards/doc-style.md
canonical_authority: docs/CONSTITUTION.md
enforced_by: oya-foundry-fitness-doc-style
companion_docs:
  - docs/DOC-CATALOG.md
  - docs/STANDARDS-AND-TEMPLATES.md
  - docs/GLOSSARY.md
---

# Doc Style

## Constitutional authority — [CONSTITUTION.md](../CONSTITUTION.md)

This standard operates within the [`CONSTITUTION.md`](../CONSTITUTION.md)
frame (§Documentation) and downstream of [`AGENTS.md`](../AGENTS.md). It
defines *how* to write canonical docs; [`DOC-CATALOG.md`](../DOC-CATALOG.md)
defines *when* and *why* to update them.

## 1. Diátaxis — the four quadrants

Every doc page serves **one** of four audience needs. Mixing types is the
antipattern. (Source: [Diátaxis](https://diataxis.fr/),
adopted by Python docs, Canonical, Cloudflare.)

| Quadrant | Audience question | Voice | Length cap | Examples in `docs/` |
|---|---|---|---|---|
| **Tutorial** (learning-oriented) | "I am new, walk me through it." | imperative, hand-holding, complete | ≤ 500 lines | `README.md` quickstart, onboarding tracks |
| **How-to** (task-oriented) | "I know the system; how do I do X?" | imperative, terse, goal-focused | ≤ 300 lines | runbooks, `RUNBOOKS-INDEX.md` rows, `checklists/` |
| **Reference** (info-oriented) | "What is the exact contract?" | declarative, exhaustive, no narrative | ≤ 600 lines | `SPEC.md`, `ADR-INDEX.md`, `GLOSSARY.md`, `contracts/` |
| **Explanation** (understanding-oriented) | "Why does it work this way?" | narrative, comparative, opinionated | ≤ 400 lines | `DESIGN.md`, `CONSTITUTION.md`, this file |

A canonical doc MUST declare its quadrant in frontmatter `doc_class:` (one of
`Tutorial`, `HowTo`, `Reference`, `Explanation`) OR a `doc_class` from the
catalog (e.g., `Standard`, `Operating-Contract`, `Constitution`, `RunbookIndex`)
that maps unambiguously to one quadrant. The lane
`oya-foundry-fitness-doc-class-diataxis` rejects pages whose body shape
contradicts the declared quadrant (e.g., a `Reference` doc with a tutorial
preamble).

## 2. RFC-2119 normative discipline

The keywords **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**,
**SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and
**OPTIONAL** carry the meanings of [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119)
as updated by [RFC 8174](https://www.rfc-editor.org/rfc/rfc8174) **only when
written in all-caps**.

Rules:

1. Every standard MUST include an RFC-2119 normative-language statement
   identical to the one in [`AGENTS.md`](../AGENTS.md) §"RFC-2119
   normative-language statement". A reference link is sufficient if the doc
   is ≤ 100 lines.
2. Lowercase forms ("you must", "should consider") carry no normative force.
3. Authors MUST NOT introduce non-RFC-2119 "must"-words ("required to",
   "needed to", "have to") in normative positions; rewrite as "MUST" or
   downgrade to lowercase advisory prose.
4. Reference docs (Diátaxis quadrant 3) MAY omit RFC-2119 keywords entirely
   if they are pure surface enumeration.

The `doc-style` lane greps for lowercase "must" / "should" in normative
positions and flags candidates for human review (advisory, not blocking).

## 3. Dual-audience rule

Per [`CONSTITUTION.md`](../CONSTITUTION.md) §Decision principles — Do, Item 4:
every directive read by an agent is also readable to a human, and vice versa.
The mechanical contract:

- **Agent-actionable sections** (commands, JSON, structured arguments) are
  fenced `<!-- agent-instructions:start --> ... <!-- agent-instructions:end -->`
  per [`agent-instructions-discipline.md`](agent-instructions-discipline.md).
- **Plain-English explanation** for the same directive sits **outside** the
  fence and MUST stand alone (a human reading only the prose understands the
  intent without the fenced commands).
- **Terminal commands shown to humans** MUST use the `rtk` prefix per the
  project CLAUDE.md / RTK convention; agents inside `<!-- agent-instructions
  -->` blocks call the same primitive through their harness.

The lane `oya-foundry-fitness-dual-audience` checks that every fenced block has
adjacent plain-English prose of ≥ 2 sentences explaining the intent.

## 4. Frontmatter shape per doc-class

Every canonical doc starts with YAML frontmatter. The minimum shape is below;
add doc-class-specific fields as documented in `STANDARDS-AND-TEMPLATES.md`.

```yaml
---
doc_class: <Standard | Constitution | Operating-Contract | Reference | RunbookIndex | ...>
shape: <~ | anchor | index | redirect>
length_cap: <integer line count>
authority_tier: <0 | 1 | 2 | 3>
status: <pending approval | accepted | superseded | retired>
purpose: |
  One paragraph: what this doc answers; who reads it.
lift_target: <path/under/docs/...>   # only for working drafts under .omc/
canonical_authority: docs/CONSTITUTION.md
enforced_by: <ci-lane-name>          # standards/runbooks only
companion_docs:
  - <path>
authority_chain_declaration: |
  docs/CONSTITUTION.md > rest of docs/ > catalog records > Redirect-class files > working drafts
---
```

Per-doc-class additions:

- **Tier-1 strategy docs** (CONSTITUTION, AGENTS, PRD, DESIGN, SPEC, ROADMAP):
  `excludes:` block (each row `path:` + `reason:`).
- **Standards**: `enforced_by: <lane-name>` (REQUIRED).
- **Runbooks**: `last_verified: <ISO-date>`, `severity_tier: <Sev-1|2|3|4>`,
  `slo_links: [<slo-ids>]`.
- **ADRs**: `adr_id:`, `status: <Proposed|Accepted|Superseded|Retired>`,
  `supersedes:`, `superseded_by:`, `decision_drivers:`, `consequences:`.

The lane `oya-foundry-fitness-frontmatter-shape` validates per-doc-class
required keys.

## 5. Heading hierarchy

1. Every doc has exactly one **H1** matching the doc title.
2. **H2** sections are stable anchors used by RACI / dependent-doc references;
   renaming an H2 is a breaking change and triggers `EVT-DOC-UPDATED` cascade
   per [`DOC-CATALOG.md`](../DOC-CATALOG.md) §3.5.
3. **H3** for sub-sections; **H4** discouraged (collapses readability). If a
   section needs an H5 it usually wants its own H2 or a child doc.
4. Skipping levels (H2 → H4) is forbidden.
5. Reference docs MAY use H2 per "row" (e.g., per ADR, per capability) when
   the row count is ≤ 50; otherwise prefer a table.

## 6. Line-length guidance

- **Prose**: SHOULD wrap at 100 columns. MUST NOT exceed 120 columns. URLs
  and code blocks are exempt.
- **Tables**: MUST NOT wrap inside a cell; if a cell exceeds 120 cols, break
  into multiple rows or use a footnote.
- **Code blocks**: SHOULD wrap at 100 cols; long shell pipelines MAY use
  backslash continuation.

The `oya-foundry-fitness-line-length` lane is **advisory** (warn-only) — it
surfaces violations as PR comments but does not block merge.

## 7. Voice and tone

- Prefer present-tense declarative ("the lane refuses X") over future or
  conditional ("the lane will refuse X if Y").
- No first-person plural ("we should") in normative prose; use "MUST" /
  "SHOULD" / "MAY" or third-person declarative.
- No marketing language. No emoji in canonical docs (per user-machine
  global instruction).
- Cite sources with their canonical URL inline at first mention. Do not
  paraphrase a hyperscaler practice without naming the source.

## 8. Anti-patterns

1. **Mixing Diátaxis quadrants** in one doc — e.g., a reference doc with a
   tutorial preamble; split into two docs.
2. **Lowercase normative "must"** — either uppercase it or downgrade to
   advisory prose.
3. **Inline duplication of authority-tier content** — link, don't restate.
4. **Renaming H2 anchors silently** — trigger a CHANGELOG row and dependent-doc
   re-walk.
5. **Frontmatter drift** — adding undocumented top-level keys; either add to
   this standard first or use a per-doc-class extension block.

## 9. Sources scanned

- [Diátaxis](https://diataxis.fr/) — content-type quadrants.
- [I'd Rather Be Writing — Diátaxis](https://idratherbewriting.com/blog/what-is-diataxis-documentation-framework).
- [Ubuntu — Diátaxis foundation](https://ubuntu.com/blog/diataxis-a-new-foundation-for-canonical-documentation).
- [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119),
  [RFC 8174](https://www.rfc-editor.org/rfc/rfc8174).
- [Google Developer Documentation Style Guide](https://developers.google.com/style)
  (voice + tone reference).
- [Linux kernel `Documentation/process/`](https://www.kernel.org/doc/html/latest/process/)
  precedent — terse, declarative, machine-friendly.
- [`.omc/specs/hyperscaler-best-practices-2026-05-12.md`](../specs/hyperscaler-best-practices-2026-05-12.md)
  Domain 2 "Documentation" section.
