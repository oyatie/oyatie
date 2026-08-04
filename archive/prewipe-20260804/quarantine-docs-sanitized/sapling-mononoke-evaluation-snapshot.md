# Sapling and Mononoke evaluation snapshot

- **Snapshot date:** 2026-07-31
- **Status:** historical design research, non-authoritative, and not a dependency decision

## Preserved conclusion

The source separated three questions that should not be collapsed:

| Question | Historical conclusion | Revalidation needed |
|---|---|---|
| Mononoke as a hosted SCM server | Reject for the evaluated destination: external operability was unsupported, the identity model did not match the content-only work-area identity, and licensing prevented copying code into the owned stack. | Recheck upstream support, deployment artifacts, architecture, and license. |
| Sapling client as a local tool | Optional developer ergonomics only; it did not discharge server, pipeline, evidence, or work-area-identity requirements. | Recheck current client behavior and repository policy. |
| Design harvest | Preserve selected architecture patterns without adopting or vendoring the system. | Validate each pattern against current Oyatie contracts before implementation. |

## Identity mismatch

The evaluated Mononoke changeset identifier was derived from a serialized history node containing parents, author and committer metadata, timestamps, message, and file changes. The evaluated Oyatie work-area identity excluded author identity, timestamps, and moving-parent state.

The durable distinction is categorical:

- a VCS changeset identifier names a history record;
- a work-area identifier names a canonical content frame.

Carrying the content-frame hash as extra metadata would not make it byte-identical to the VCS primary key. A separate mapping may be useful for provenance, but it is not identifier equivalence.

## Design patterns worth retaining

1. **Canonical record with derived projections.** Keep one owned canonical model; treat Git or other VCS representations as derived projections with explicit mappings.
2. **Immutable content plus mutable metadata.** Separate a content-addressed blob store from mutable metadata such as refs, indexes, and mappings.
3. **Move integration serialization off frontends.** Queue conflicting land/integration operations by target so stateless frontends do not repeat doomed work.
4. **Scale service tiers independently.** Keep protocol frontends stateless and separate them from integration, derived-data, and metadata services.

## Cautions retained from the source

- “Self-hostable source code” is not the same as a supported, operable external server distribution.
- A client release does not prove the server path is deployable.
- Production serialization at a narrow integration key can be safer and cheaper than a blanket “no leader” requirement; evaluate measured contention before rejecting it.
- Design reading is not permission to copy code. Confirm current licenses before reuse.
- Negative ecosystem findings decay quickly and must be repeated before a present-day decision.

## Primary references named by the source

These links are provenance, not verification of current state:

- <https://github.com/facebook/sapling>
- <https://github.com/facebook/sapling/tree/main/eden/mononoke>
- <https://github.com/facebook/sapling/issues/812>
- <https://github.com/facebook/sapling/issues/922>
- <https://sapling-scm.com/docs/git/git_support_modes/>
- <https://sapling-scm.com/docs/scale/overview/>
- <https://cacm.acm.org/research/why-google-stores-billions-of-lines-of-code-in-a-single-repository/>
- <https://arxiv.org/html/2604.11977v1>
- <https://epicgames.github.io/lore/explanation/system-design/>
