---
doc_class: JudgmentNote
title: Phase0 merge-admission + path ownership (additive on naming_sweep tip)
status: Accepted
date: 2026-08-11
ssot: domain_stack_integ_model_9b63d851.plan.md#merge-admission--domain-green-phase0-absorb
---

# Phase0 encode (additive; does not fight Phase1 dual-emit)

## Forever public status string (ONE)

`merge-admission-required` — matches forever workflow filename and dual-emit job.
Legacy `oya-ci-required` remains event-trigger host + BP context until PAUSE-AND-PAIR.

## Name map

| Old | Forever |
| --- | --- |
| oya-ci-required | merge-admission-required |
| firewall / cloud-ci-firewall | admission.policy |
| affected-set lane | admission.graph-affected |
| product-protocol-policy-gate | admission.manifest-inventory |
| (domain) | domain.<envelope>.stabilize |
| (trunk) | trunk.postsubmit → trunk.health |

## Path ownership

Envelope `envelope_globs` are prefix allow. Evidence taxes: #1660 compute/**, #1680 iac/governance/**.
Machine doctrine: `specs/integ-branch-envelopes.json#path_ownership` (1.16.22+).
Transitional reachability `compute/` registered on integ/specs until admission.policy consumes envelopes.

## Explicit non-goals this commit

- Protection flip
- Moving event triggers onto merge-admission-required.yml
- Full envelope-prefix gate implementation (Phase1+)
- OWNERS generation / Buck2 domain rewire / ship ritual automation
