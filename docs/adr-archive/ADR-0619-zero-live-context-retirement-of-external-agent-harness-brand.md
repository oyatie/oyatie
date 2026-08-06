---
id: ADR-0619
title: "Zero-live-context retirement of an external agent-harness brand"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-07-20
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-709]
amends: [ADR-0069, ADR-0211, ADR-0220, ADR-0239, ADR-0328, ADR-0335, ADR-0609]
depends_on: [ADR-0335, ADR-0363, ADR-0515]
related: [ADR-0516]
related_specs:
  - /specs/masterplan.json
  - /specs/root-hub-pointers.json
  - /specs/agent-operating-contract.json
milestone: W0
---

# ADR-0619: Zero-live-context retirement of an external agent-harness brand

## Status

**Accepted — 2026-07-20.** The founder directed immediate cessation and removal. This ADR
records that ruling, closes the readable-history exceptions in ADR-0335 D-30, D-34, and D-81,
and removes the source-specific ingest introduced by Proposed ADR-0609. No later Accepted ADR
reverses ADR-0335's retirement.

The forbidden lower-case ASCII token is identified by SHA-256
8cfde6efdfc4ed5ab1f6acbbd1ba49bf31932f84d0a4c090eb41c7d151e8b180.
The plaintext token is intentionally absent from this decision.

## Context

ADR-0335 retired an inherited external agent-harness brand as a canonical primitive, but allowed
the exact name to remain in historical ADRs, memories, and forensic prose. Proposed ADR-0609 later
introduced a source-specific local-board reader, schema keys, 934 imported completion claims, an
evidence packet, build targets, and live plan references. Those surfaces re-established the retired
system as an active dependency even though all imported claims remained non-authoritative:

- 934 imported completion claims;
- 34 claims carrying evidence references;
- 900 claims with no evidence reference;
- zero verified claims;
- zero imports mapped to native MPV2 work items.

There are no product users, user data, or outcomes requiring compatibility preservation. Keeping a
dual-read alias would add migration debt and invite future agents to treat stale claims as authority.

## Decision

### 1. Absolute active-tree absence

The current protected branch must contain zero case-insensitive occurrences of the forbidden token
in tracked pathnames or tracked blob bytes. There are no exemptions for ADRs, audit evidence,
archives, binary blobs, symlink payloads, fixtures, or generated projections.

A born-blocking frozen-empty cloud-ci rule scans the complete tracked candidate tree before
historical/archive filtering. The rule fails closed on missing or unreadable tracked blobs. Its own
source constructs the forbidden bytes numerically and therefore does not reintroduce the token.

### 2. Capability-neutral coordination

The surviving operating model is a portfolio/architecture coordinator plus dispatcher-assigned
implementation and review workers. The coordinator/worker separation, isolated worktrees,
independent review, and protected-PR admission contract remain. No role, source path, skill,
profile, CLI, board, or runtime is coupled to a vendor or agent harness.

### 3. Source-specific ingest is deleted, not renamed

The local-board reader, its build/package registrations, its evidence packet, its live masterplan
array and summary, and every source-specific validation branch are removed atomically.

The generic invariant survives: any future external completion claim that is represented in a live
plan must be provider-neutral, schema-versioned, and unable to attain verified completion without
recorded completion evidence. Absence of an external-claim array is valid; a malformed present
array fails closed. No compatibility alias or dual-read period is allowed.

### 4. Provenance lives outside live agent context

Git object history is the repository provenance archive. Historical tracked projections are
sanitized or removed from HEAD; they are not copied to another readable in-tree archive.

Append-only evidence uses an epoch rollover rather than pretending an in-place rewrite is an
append. The successor epoch records the predecessor Git blob OID and SHA-256, then contains only
neutral current records. Exact predecessor bytes remain reachable through authorized Git history,
not through agent entry surfaces or default corpus scans.

### 5. Existing evidence is not promoted

Evidence references carried by the retired import are not automatically mapped into native
masterplan work items. Mapping requires an independently justified MPV2 identity and the normal
recorded-evidence checks. The migration records counts and predecessor digests, then deletes all 934
claims without upgrading any status.

### 6. Runtime and context cutover

Installed executables, home-directory configuration, credentials, skills, prompts, temporary
artifacts, and active project bindings for the retired harness are removed from the declared local
runtime boundary. Existing agents that received the old context cannot certify the cutover; final
review is performed by fresh-context reviewers after the protected branch and runtime scans pass.

Memory systems that are externally managed receive an explicit invalidation request through their
supported update mechanism. The repository does not preserve a readable duplicate.

## Consequences

- Stale external-board claims cannot influence roadmap, priority, or completion truth.
- Future work-source adapters must enter through a provider-neutral API/capability boundary and
  qualification lifecycle; no local database path becomes plan authority.
- Git history remains available for authorized forensic recovery, while normal agents receive a
  smaller, contradiction-free context.
- The change is one-way. Reintroduction requires a new Accepted ADR, a demonstrated capability gap,
  an independently reviewed threat model, and a nonzero born-blocking gate migration.

## Alternatives considered

### Rename the source-specific importer

Rejected. A rename would preserve the dependency, schema, and stale data under a neutral label.

### Keep historical exceptions in the tracked tree

Rejected. Agents repeatedly ingest tracked ADRs and evidence; labels such as historical do not
prevent context leakage or accidental authority laundering.

### Rewrite all Git history

Rejected for this cutover. Current-tree and runtime absence achieves the requested operational
boundary without invalidating every existing commit signature, clone, pull request, and external
reference. A later legal erasure order may authorize a separately governed history rewrite.

### Delete evidence without a receipt

Rejected. Predecessor object identities, counts, disposition, and the absence of status promotion
remain independently checkable through the neutral closure receipt and Git history.

## Verification

1. Candidate-tree path scan returns zero matches.
2. Raw tracked-blob scan returns zero matches, including binary and symlink payload fixtures.
3. No source-specific importer crate, package, target, schema key, source path, or compatibility
   alias remains.
4. Canonical JSON parses and targeted cross-artifact, brand-residue, baseline-ratchet,
   generated-artifact, and gate-self-conformance tests pass.
5. No file matching the generated-JSON suffix was hand-edited.
6. Runtime/configuration/temporary-root scans are empty within the declared boundary.
7. A fresh-context reviewer independently reproduces the scans and confirms the architecture is
   reconstructable without the retired system.
8. Protected pull-request review is complete and the single required cloud-ci context is green.
