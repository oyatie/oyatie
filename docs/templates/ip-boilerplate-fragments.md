---
doc_class: Template
template_id: TPL-IP-BOILERPLATE-FRAGMENTS
canonical_authority: ADR-0064 (SWEEP-I Slice 6)
status: active
created_at: 2026-05-18
---

# Implementation Plan canonical boilerplate fragments

Every per-µservice `IP-NNN-<slug>.md` MAY import these canonical sections by
reference rather than re-authoring boilerplate. Each fragment below is keyed
by `## <section>` heading; per-µservice IPs cite the section then add the
µservice-specific content.

## Fragment: ChangeSet boundary

Every IP defines exactly ONE ChangeSet. Per ADR-0111 changeset state machine,
the ChangeSet is the atomic unit of:

- **Claimable** — exactly one agent holds it at any time
- **Verifiable** — passes all acceptance lanes in frontmatter
- **Bundleable** — can be admitted to the merge queue as a unit
- **Promotable** — passes the completion gate (reviewer-agent APPROVE + CI green)

Per-µservice IPs add: which files/crates the ChangeSet touches.

## Fragment: Halt Conditions

A ChangeSet MUST halt and request human review when ANY of:

1. Any acceptance lane in the IP frontmatter exits non-zero AFTER the IP
   author claims green (i.e. honesty violation — `check-honest-claims`).
2. Any irreversible operation appears in the diff (per
   `/specs/forbidden-operations.json`): destructive git, force-push to
   protected branch, schema-breaking change without ADR + version bump.
3. Any cross-µservice direct call (per ADR-0064 Workflow + Ontology adapter
   layer rule — products NEVER call each other directly).
4. Any new dependency on a non-LTS upstream (per
   `docs/standards/lts-versions-verified.md`).
5. Any line of code that downgrades a public-contract guarantee without
   ADR + version bump + sunset notice (per the no-silent-regression
   doctrine).

Per-µservice IPs add: µservice-specific halt conditions (e.g.
"halt if cell statefulness invariant breaks").

## Fragment: Verification

Every IP author collects ALL of these BEFORE marking the IP complete:

1. `buck2 build <touched-build-targets>` exits 0
2. `buck2 test <touched-test-targets>` exits 0
3. the relevant Buck2/cloud-ci lint/static-analysis gate exits 0
4. the relevant Buck2/cloud-ci formatting gate exits 0
5. the relevant cloud-ci acceptance gate packet is green in `presubmit`
6. New / modified files match canonical schemas:
   - `*.cedar` → `specs/policy/cedar-scope-schema.md`
   - `iac/helm/*/Chart.yaml` → depends on `_oya-helpers`
   - `iac/kustomize/*/kustomization.yaml` → composes canonical component
   - `*.openslo.yaml` → `specs/openslo/canonical-envelope-schema.json`
   - `capabilities/*.yaml` → `specs/capabilities/canonical-tier-schema.json`
   - `catalog/*.yaml` → `specs/catalog/canonical-crate-record-schema.json`
7. Evidence emitted at `evidence/<ip-id>-acceptance.json`

Per-µservice IPs add: µservice-specific eval-corpus / golden-trace runs.

## Fragment: Next IP

Every IP closes with a `## Next IP` section naming the IP that succeeds it
(or "none — terminal IP for this phase"). Per the milestone > phase > IP
hierarchy, IPs chain within a phase, phases chain within a milestone.

Per-µservice IPs add: concrete next-IP identifier.

## Fragment: Concrete File Targets

Every IP lists concrete file targets in a table:

| Path | Action | Description |
|---|---|---|
| `<absolute path under microservices/<ms>/>` | create / modify / delete | one-line intent |

Per ADR-0131 per-microservice flat layout, all paths land under
`microservices/<ms>/src/...` for code, `microservices/<ms>/catalog/` for
catalog, etc.

## Fragment: Evidence emission

Every IP emits per-acceptance-lane evidence at:

```
evidence/<ip-id>-acceptance.json
evidence/<ip-id>-lanes/<lane>.json   # per-lane drill-down
```

Per-µservice IPs add: µservice-specific evidence topic
(`oya.<microservice>.<bc>.<event>`) the IP's changes will start emitting.

## Fragment: References

Every IP closes with a `## References` section citing:

- The phase document at `microservices/<ms>/PHASE-NN-<slug>.md`
- The PRD section that motivates this IP (FR-N)
- Source ADRs (must include at least one)
- Source specs
- Related IPs (preceded_by / succeeded_by from frontmatter)

## How to use these fragments in a per-µservice IP

In your per-µservice IP markdown, reference the fragment then add per-µservice
specifics:

```markdown
## ChangeSet boundary

(Per canonical fragment in docs/templates/ip-boilerplate-fragments.md#ChangeSet-boundary)

This IP's ChangeSet touches:
- `microservices/workflow-studio/src/crates/workflow-studio-visual-canvas-kernel/`
- `microservices/workflow-studio/src/crates/workflow-studio-visual-canvas-domain/`

## Halt Conditions

(Per canonical fragment) plus µservice-specific:
- Halt if visual-canvas layout algebra produces non-deterministic output
  across two runs over the same input set.
```
