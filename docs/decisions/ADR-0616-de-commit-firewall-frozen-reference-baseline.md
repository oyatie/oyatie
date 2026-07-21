---
id: ADR-0616
title: "Proposed de-commit of the firewall frozen-reference baseline"
status: Proposed
planning_impact: false
deciders: founder
date: 2026-07-09
door: one-way
owner: council-architecture
supersedes: []
superseded_by: []
depends_on: [ADR-0515, ADR-0551, ADR-0596, ADR-0613]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0616: Proposed de-commit of the firewall frozen-reference baseline

## Status

**Proposed — nonbinding.** This record does not amend, reverse, supersede, or otherwise
displace Accepted ADR-0613. ADR-0596 is likewise Proposed and nonbinding provenance; this
record does not give it authority. No implementation, materialization-mode change, baseline
removal, signer/custody claim, or dispatch authority follows from this proposal.

## Context

`ci/facade/artifact-inventory-registry/gate-baseline.generated.json` is the firewall
ratchet's frozen reference. Proposed ADR-0596 records the historical rationale for keeping
that reference committed. Accepted ADR-0613 is the controlling committed-baseline authority
while it de-commits only pure projection faces.

## Proposal

The proposed future direction is to evaluate whether a trusted controller could derive the
frozen reference from immutable merge-base source without weakening the existing ratchet.
This document records a question for review only. It does not authorize a candidate-built
analyzer, a controller change, or a replacement for the committed baseline.

## Current authority and hold

Until a separate decision is Accepted through the repository's governance process, ADR-0613
remains controlling; ADR-0596 and this record remain nonbinding provenance. The committed
frozen baseline is read from the merge-base by the accepted firewall path. Existing HOLD and
no-dispatch conditions are unchanged.
