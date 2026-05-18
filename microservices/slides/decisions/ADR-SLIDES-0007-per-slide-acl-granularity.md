---
id: ADR-SLIDES-0007
title: Per-slide ACL granularity — named-block Cedar refinement
microservice: slides
status: Accepted
date: 2026-05-17
owner: axis-workspace + ops-security + council-architecture
deciders: council-architecture, axis-workspace, ops-security, dpo-office
supersedes: []
superseded_by: []
related: [ADR-0105, ADR-0135, ADR-0131, ADR-0140]
related_specs: []
related_artifacts:
  - microservices/slides/PRD.md (FR-30, AC-08, AC-15)
  - microservices/slides/PHASE-01-SLIDES-FOUNDATION.md (IP-013)
  - microservices/slides/policy/tenant-scope.cedar
  - microservices/docs/decisions/ADR-DOCS-0004-per-block-acl.md  # sibling pattern in docs
purpose: Establish per-slide and per-named-block ACL granularity via Cedar v4.2 LTS refinement of deck-level ACL, as a competitive differentiator against Google Slides + PowerPoint Web (which both offer deck-level only).
doc_status: published
---

# ADR-SLIDES-0007: Per-slide ACL — Cedar named-block refinement on top of deck-level

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

Tenants frequently need to share decks where:
- Most slides are visible to a wide audience (e.g., company-wide quarterly review).
- A subset of slides contains sensitive material (e.g., compensation distributions, M&A discussion, anonymous-survey raw data) that should only be visible to a subset of the audience.

Google Slides offers deck-level ACL only. PowerPoint Web offers deck-level + comment-restricted but not per-slide. Keynote offers deck-level. This is a competitive parity gap that the slides µservice can close as a differentiator.

The technical question: how to express + enforce per-slide ACL without exploding the policy evaluation cost or fragmenting the user mental model.

Three granularity options:
- **Slide-level only**: each slide carries an ACL list; deck-level grant inherited unless explicitly overridden.
- **Named-block-level**: a slide can contain "named blocks" (regions of one or more placeholders) that carry independent ACL.
- **Per-placeholder-level**: each placeholder carries its own ACL.

PRD Open Question 5: per-slide vs named-block; bias: named-block.

## Decision

Adopt **per-slide ACL with named-block refinement** in the following shape:

1. **Default**: deck-level ACL grants inherit to every slide and every block within.
2. **Per-slide ACL**: each `Slide` entity may carry a `per_slide_acl` Cedar refinement. The refinement may DENY a principal who would otherwise be granted by deck-level, OR grant a principal who is not granted by deck-level. The deck-level grant is necessary BUT NOT SUFFICIENT to read/edit a slide — the per-slide DENY overrides deck-level GRANT.
3. **Named-block ACL** within a slide: an author may mark a region of one or more placeholders as a "named block" (e.g., `"Compensation table"`) and apply a further Cedar refinement to that block. Named-block DENY overrides slide GRANT.
4. **Cedar policy structure** (per `policy/tenant-scope.cedar`):
   - Three policy levels: deck → slide → named-block.
   - Evaluator visits each level in turn; first DENY wins; if no DENY, PERMIT.
5. **Cardinality**: each deck → 0..N slides; each slide → 0..M named-blocks. M default ≤ 8 per slide (UX simplicity); configurable per-pack.
6. **Cedar preview required before save** (AC-15): every ACL change exercises a Cedar policy preview against the deck's principal set; the API returns `impact_summary` with allowed-count + denied-count + warnings. Save refused if preview returns errors.
7. **CRDT integration** (ADR-SLIDES-0001): per-slide ACL refinement is applied at CRDT projection boundary; ops to non-permitted slides are filtered before fan-out to other peers. ACL change → projection cache invalidation cascade.
8. **Public-share-link interaction** (per `policy/public-read.cedar`): public-share-link reader may NOT access per-slide-restricted content; refused at evaluator. Tenant-visible warning at share-link creation if some slides are per-slide-restricted (data loss vs intended audience).
9. **Audit**: every per-slide and named-block ACL change emits `AclChanged` event with before/after sha + slide_id + block_id (if applicable) + change_type. Ed25519-sealed.
10. **Restore from version-history**: when a version is restored, Cedar re-evaluation is replayed per-slide; the restorer must hold sufficient grants on every slide being restored, else partial restore + tenant-visible warning.

## Alternatives Considered

### A — Deck-level only (competitor baseline)

- **Pros**: Simpler; Google Slides + Keynote pattern.
- **Cons**: Competitive parity gap (named slidesection patterns in enterprise scenarios). Tenants resort to multiple-deck workarounds (split sensitive sections to a separate deck), which fragments their narrative.
- **Rejected reason**: Tenant value Outcome (PRD) + competitive differentiation.

### B — Per-placeholder ACL (finest grain)

- **Pros**: Maximum granularity.
- **Cons**: UX confusion (which placeholder is restricted?); Cedar evaluation cost grows with placeholder count (typical deck has hundreds of placeholders); restoration + version-history complexity multiplies.
- **Rejected reason**: UX + Cedar evaluation cost.

### C — Per-slide ACL but no named-block

- **Pros**: Middle ground; simpler than per-placeholder.
- **Cons**: Misses the "some text within a slide should be hidden from a subset of viewers" use case. Authors resort to splitting one slide into two (one visible, one hidden) — UX confusion.
- **Rejected reason**: Named-block use case unaddressed.

### D — Per-text-region ACL via inline markup (e.g., `${restricted:role-X}`)

- **Pros**: Inline + flexible.
- **Cons**: Markup invades the content model; complicates copy/paste, AI-content-generation, PPTX/ODP export.
- **Rejected reason**: Content-model contamination.

### E — Outside-of-Cedar ACL implementation (homegrown)

- **Pros**: Slides team owns the ACL evaluation.
- **Cons**: Cedar is the cross-µservice ACL substrate (ADR-0140); rolling a homegrown evaluator forfeits cross-µservice consistency + Cedar's formal semantics.
- **Rejected reason**: Cross-µservice consistency.

## Consequences

### Architectural

- `acl` BC crates: `oya-slides-acl-{kernel, domain, usecase, api, adapter, adapter-postgres}`.
- Cedar policy templates in `policy/tenant-scope.cedar`; three levels (deck → slide → named-block).
- Slide entity carries optional `per_slide_acl` field; NamedBlock entity carries optional `block_acl` field.
- Cedar evaluator invoked per (principal, action, resource); resource is the most-specific (NamedBlock > Slide > Deck).
- Per-slide ACL change emits an `AclChanged` event + invalidates CRDT projection cache for the affected slide.
- Per-deck Cedar preview cache: pre-computed per (deck × principal-set) for fast preview API.
- Restore-from-version-history replays Cedar evaluation per slide.

### Downstream impact on other µservices and IPs

1. **IP-013 (acl + comments + version-history + embed-bridge)** — authors the per-slide + named-block ACL.
2. **observability µservice** — slides-specific ACL SLIs (cache hit rate, preview latency, drift counter).
3. **audit-chain µservice** — Ed25519-sealed `AclChanged` event per change.
4. **tenancy µservice** — per-seat licensing interacts with per-slide ACL (per-slide is a Cedar refinement, not a seat-tier feature, but pack-tier may unlock named-block features).
5. **competitor-parity-matrix.md** — per-slide ACL + named-block ACL as unique differentiators.
6. **docs µservice** — has analogous per-block ACL per ADR-DOCS-0004 (sibling pattern); cross-µservice consistency in ACL granularity model.

### SLOs gaining new dimensions

- `slides.acl_evaluation_p99_seconds` — Cedar eval latency; target ≤ 0.03s (30ms; budget within 100ms save-p95).
- `slides.acl_cache_hit_rate` — target ≥ 0.9.
- `slides.acl_drift_detected_count` — must equal 0 over 30min.
- `slides.acl_preview_p95_seconds` — target ≤ 0.5s.

### CI lanes added

- `oya-governance-cedar-preview-required` — verifies every save path exercises Cedar preview.
- `oya-governance-per-slide-acl-no-deck-bypass` — verifies that per-slide DENY overrides deck-level GRANT.
- `oya-governance-named-block-no-bypass` — verifies that named-block DENY overrides slide GRANT.

### Risk register

- **Risk**: Per-slide ACL UX confusion (author unsure why a viewer can't see a slide). **Mitigation**: in-editor per-slide ACL banner + Cedar preview impact summary at save.
- **Risk**: ACL cache + Postgres source-of-truth drift. **Mitigation**: cache invalidation cascade on `AclChanged` event + periodic drift detector cron + alarm.
- **Risk**: Named-block UX overload (too many blocks). **Mitigation**: per-slide block cap 8 (configurable per-pack).
- **Risk**: Public-share-link viewers see per-slide-restricted slides. **Mitigation**: refused at evaluator; tenant warned at share-link creation.
- **Risk**: Version-history restore replay creates partial deck. **Mitigation**: tenant-visible warning; require explicit confirmation for partial restore.
- **Risk**: Cross-µservice ACL granularity divergence (docs uses block; slides uses slide+block; sheets uses cell). **Mitigation**: per-µservice ADRs make granularity explicit; cross-µservice consistency is at Cedar substrate level, not refinement structure.

## References

- Cedar v4.2 LTS — `cedarpolicy.com`, `github.com/cedar-policy/cedar`.
- ADR-0140 (Cedar policy enforcement — repo-wide).
- ADR-DOCS-0004 (sibling — docs per-block ACL).
- PRD FR-30, AC-08, AC-15.
- policy/tenant-scope.cedar.
- failure-modes.md FM-18, FM-19.
- threat-model.md T-E-02.
