---
doc_class: FoundrySpec
title: "Foundry Agent Types and Roles"
status: Draft
date: 2026-05-20
owner: "axis-foundry + council-foundry-vcs"
related_oyatie_adrs:
  - ADR-0221
  - ADR-0110
  - ADR-0111
  - ADR-0112
  - ADR-0113
  - ADR-0116
  - ADR-0136
  - ADR-0220
  - ADR-0263
audience: RETIRED — historical foundry internal agentic-development pipeline (see ADR-0335 Wave 15I)
consumer_facing: false
canonical_path: microservices/foundry/spec
---

# Foundry Agent Types and Roles

## Purpose

This spec defines the Foundry agent role model for Claude Opus, Codex, planner, executor, reviewer, verifier, and supporting services in the internal agentic-development pipeline.

These are internal development agents. They are not consumer assistants, tenant copilots, or Intelligence personas.

Foundry was the internal agentic-development pipeline (RETIRED per ADR-0335 Wave 15I; absorbed into intelligence): it coordinates source changes, CI evidence, reviewer decisions, merge-queue state, and promotion state for the oyatie repository.
Foundry is not a consumer-facing AI product, not a tenant assistant, and not the place where B2B or B2C prompt history is stored.
This spec turns the ADR-level decision into an operator-readable and agent-executable substrate contract.
The contract is intentionally written with RFC 2119 and RFC 8174 keywords so policy, CI, and agent prompts can parse the same text.
The control plane treats tenant_id=oyatie as the internal tenant for internal agentic-development pipeline emissions, consistent with ADR-0263.
Every state-changing step emits an audit event and a correlated observability record before a downstream state may rely on it.
The implementation boundary is the single Foundry microservice with internal bounded contexts from ADR-0136.
The repository boundary is microservices/foundry plus repo-level governance registries and docs referenced in cross-references.
The user-facing Intelligence substrate remains separate and MUST NOT be conflated with this pipeline.
The stop condition for this spec is a changeset whose evidence, policy, CI, review, queue, and promotion state can be replayed deterministically.

## Normative Requirements

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, MAY, and OPTIONAL in this section are to be interpreted as described in RFC 2119 and RFC 8174.

1. Foundry Agent Types and Roles MUST ensure the state transition be written before downstream consumers act.
2. Foundry Agent Types and Roles MUST ensure the state transition carry a deterministic identifier.
3. Foundry Agent Types and Roles MUST ensure the state transition declare its actor, action, resource, and reason.
4. Foundry Agent Types and Roles MUST ensure the state transition fail closed when required evidence is absent.
5. Foundry Agent Types and Roles MUST ensure the Cedar decision be written before downstream consumers act.
6. Foundry Agent Types and Roles MUST ensure the Cedar decision carry a deterministic identifier.
7. Foundry Agent Types and Roles MUST ensure the Cedar decision declare its actor, action, resource, and reason.
8. Foundry Agent Types and Roles MUST ensure the Cedar decision fail closed when required evidence is absent.
9. Foundry Agent Types and Roles MUST ensure the audit event be written before downstream consumers act.
10. Foundry Agent Types and Roles MUST ensure the audit event carry a deterministic identifier.
11. Foundry Agent Types and Roles MUST ensure the audit event declare its actor, action, resource, and reason.
12. Foundry Agent Types and Roles MUST ensure the audit event fail closed when required evidence is absent.
13. Foundry Agent Types and Roles MUST ensure the observability emission be written before downstream consumers act.
14. Foundry Agent Types and Roles MUST ensure the observability emission carry a deterministic identifier.
15. Foundry Agent Types and Roles MUST ensure the observability emission declare its actor, action, resource, and reason.
16. Foundry Agent Types and Roles MUST ensure the observability emission fail closed when required evidence is absent.
17. Foundry Agent Types and Roles MUST ensure the evidence bundle be written before downstream consumers act.
18. Foundry Agent Types and Roles MUST ensure the evidence bundle carry a deterministic identifier.
19. Foundry Agent Types and Roles MUST ensure the evidence bundle declare its actor, action, resource, and reason.
20. Foundry Agent Types and Roles MUST ensure the evidence bundle fail closed when required evidence is absent.
21. Foundry Agent Types and Roles MUST ensure the cost budget be written before downstream consumers act.
22. Foundry Agent Types and Roles MUST ensure the cost budget carry a deterministic identifier.
23. Foundry Agent Types and Roles MUST ensure the cost budget declare its actor, action, resource, and reason.
24. Foundry Agent Types and Roles MUST ensure the cost budget fail closed when required evidence is absent.
25. Foundry Agent Types and Roles MUST ensure the idempotency key be written before downstream consumers act.
26. Foundry Agent Types and Roles MUST ensure the idempotency key carry a deterministic identifier.
27. Foundry Agent Types and Roles MUST ensure the idempotency key declare its actor, action, resource, and reason.
28. Foundry Agent Types and Roles MUST ensure the idempotency key fail closed when required evidence is absent.
29. Foundry Agent Types and Roles MUST ensure the retry branch be written before downstream consumers act.
30. Foundry Agent Types and Roles MUST ensure the retry branch carry a deterministic identifier.
31. Foundry Agent Types and Roles MUST ensure the retry branch declare its actor, action, resource, and reason.
32. Foundry Agent Types and Roles MUST ensure the retry branch fail closed when required evidence is absent.
33. Foundry Agent Types and Roles MUST ensure the reviewer verdict be written before downstream consumers act.
34. Foundry Agent Types and Roles MUST ensure the reviewer verdict carry a deterministic identifier.
35. Foundry Agent Types and Roles MUST ensure the reviewer verdict declare its actor, action, resource, and reason.
36. Foundry Agent Types and Roles MUST ensure the reviewer verdict fail closed when required evidence is absent.
37. Foundry Agent Types and Roles MUST ensure the CI status be written before downstream consumers act.
38. Foundry Agent Types and Roles MUST ensure the CI status carry a deterministic identifier.
39. Foundry Agent Types and Roles MUST ensure the CI status declare its actor, action, resource, and reason.
40. Foundry Agent Types and Roles MUST ensure the CI status fail closed when required evidence is absent.
41. Foundry Agent Types and Roles MUST ensure the worktree lane be written before downstream consumers act.
42. Foundry Agent Types and Roles MUST ensure the worktree lane carry a deterministic identifier.
43. Foundry Agent Types and Roles MUST ensure the worktree lane declare its actor, action, resource, and reason.
44. Foundry Agent Types and Roles MUST ensure the worktree lane fail closed when required evidence is absent.
45. Foundry Agent Types and Roles MUST ensure the branch reference be written before downstream consumers act.
46. Foundry Agent Types and Roles MUST ensure the branch reference carry a deterministic identifier.
47. Foundry Agent Types and Roles MUST ensure the branch reference declare its actor, action, resource, and reason.
48. Foundry Agent Types and Roles MUST ensure the branch reference fail closed when required evidence is absent.
49. Foundry Agent Types and Roles MUST ensure the merge-queue position be written before downstream consumers act.
50. Foundry Agent Types and Roles MUST ensure the merge-queue position carry a deterministic identifier.
51. Foundry Agent Types and Roles MUST ensure the merge-queue position declare its actor, action, resource, and reason.
52. Foundry Agent Types and Roles MUST ensure the merge-queue position fail closed when required evidence is absent.
53. Foundry Agent Types and Roles MUST ensure the promotion target be written before downstream consumers act.
54. Foundry Agent Types and Roles MUST ensure the promotion target carry a deterministic identifier.
55. Foundry Agent Types and Roles MUST ensure the promotion target declare its actor, action, resource, and reason.
56. Foundry Agent Types and Roles MUST ensure the promotion target fail closed when required evidence is absent.
57. Foundry Agent Types and Roles MUST ensure the human override be written before downstream consumers act.
58. Foundry Agent Types and Roles MUST ensure the human override carry a deterministic identifier.
59. Foundry Agent Types and Roles MUST ensure the human override declare its actor, action, resource, and reason.
60. Foundry Agent Types and Roles MUST ensure the human override fail closed when required evidence is absent.
61. Foundry Agent Types and Roles MUST ensure the safe-path predicate be written before downstream consumers act.
62. Foundry Agent Types and Roles MUST ensure the safe-path predicate carry a deterministic identifier.
63. Foundry Agent Types and Roles MUST ensure the safe-path predicate declare its actor, action, resource, and reason.
64. Foundry Agent Types and Roles MUST ensure the safe-path predicate fail closed when required evidence is absent.
65. Foundry Agent Types and Roles MUST ensure the OpenBao secret reference be written before downstream consumers act.
66. Foundry Agent Types and Roles MUST ensure the OpenBao secret reference carry a deterministic identifier.
67. Foundry Agent Types and Roles MUST ensure the OpenBao secret reference declare its actor, action, resource, and reason.
68. Foundry Agent Types and Roles MUST ensure the OpenBao secret reference fail closed when required evidence is absent.
69. Foundry Agent Types and Roles MUST ensure the GitHub post-back be written before downstream consumers act.
70. Foundry Agent Types and Roles MUST ensure the GitHub post-back carry a deterministic identifier.
71. Foundry Agent Types and Roles MUST ensure the GitHub post-back declare its actor, action, resource, and reason.
72. Foundry Agent Types and Roles MUST ensure the GitHub post-back fail closed when required evidence is absent.
73. Foundry Agent Types and Roles MUST ensure the multispectrum evidence be written before downstream consumers act.
74. Foundry Agent Types and Roles MUST ensure the multispectrum evidence carry a deterministic identifier.
75. Foundry Agent Types and Roles MUST ensure the multispectrum evidence declare its actor, action, resource, and reason.
76. Foundry Agent Types and Roles MUST ensure the multispectrum evidence fail closed when required evidence is absent.
77. Foundry Agent Types and Roles MUST ensure the trace context be written before downstream consumers act.
78. Foundry Agent Types and Roles MUST ensure the trace context carry a deterministic identifier.
79. Foundry Agent Types and Roles MUST ensure the trace context declare its actor, action, resource, and reason.
80. Foundry Agent Types and Roles MUST ensure the trace context fail closed when required evidence is absent.
81. The `intake` phase SHOULD expose a machine-readable status row with actor, at, evidence_uri, trace_id, and audit_id.
82. The `planner` phase SHOULD expose a machine-readable status row with actor, at, evidence_uri, trace_id, and audit_id.
83. The `claude_opus` phase SHOULD expose a machine-readable status row with actor, at, evidence_uri, trace_id, and audit_id.
84. The `codex` phase SHOULD expose a machine-readable status row with actor, at, evidence_uri, trace_id, and audit_id.
85. The `executor` phase SHOULD expose a machine-readable status row with actor, at, evidence_uri, trace_id, and audit_id.
86. The `reviewer` phase SHOULD expose a machine-readable status row with actor, at, evidence_uri, trace_id, and audit_id.
87. The `verifier` phase SHOULD expose a machine-readable status row with actor, at, evidence_uri, trace_id, and audit_id.
88. The `merge_queue` phase SHOULD expose a machine-readable status row with actor, at, evidence_uri, trace_id, and audit_id.
89. The `human_override` phase SHOULD expose a machine-readable status row with actor, at, evidence_uri, trace_id, and audit_id.
90. The `terminal` phase SHOULD expose a machine-readable status row with actor, at, evidence_uri, trace_id, and audit_id.
91. Action `foundry.agent.plan` MUST be denied unless a matching Cedar permit binds the principal, resource, and changeset intent.
92. Action `foundry.agent.execute` MUST be denied unless a matching Cedar permit binds the principal, resource, and changeset intent.
93. Action `foundry.agent.review` MUST be denied unless a matching Cedar permit binds the principal, resource, and changeset intent.
94. Action `foundry.agent.verify` MUST be denied unless a matching Cedar permit binds the principal, resource, and changeset intent.
95. Action `foundry.agent.invoke_claude_opus` MUST be denied unless a matching Cedar permit binds the principal, resource, and changeset intent.
96. Action `foundry.agent.invoke_codex` MUST be denied unless a matching Cedar permit binds the principal, resource, and changeset intent.
97. Action `foundry.agent.escalate_human` MUST be denied unless a matching Cedar permit binds the principal, resource, and changeset intent.
98. Action `foundry.agent.record_handoff` MUST be denied unless a matching Cedar permit binds the principal, resource, and changeset intent.
99. Implementations MAY add stricter product-local gates when they do not weaken any requirement in ADR-0221.

## State Machine / Sequence Diagram

```mermaid
flowchart TD
  intake["intake"]
  planner["planner"]
  claude_opus["claude_opus"]
  codex["codex"]
  executor["executor"]
  reviewer["reviewer"]
  verifier["verifier"]
  merge_queue["merge_queue"]
  human_override["human_override"]
  terminal["terminal"]
  request -->|intake_route: workflow event or human task arrives| intake
  intake -->|plan_slice: planner decomposes and names gates| planner
  planner -->|deep_reason: complex design or adversarial review required| claude_opus
  planner -->|code_edit: repo-local implementation required| codex
  codex -->|execute_slice: bounded implementation lane runs| executor
  executor -->|review_slice: change class reviewer evaluates| reviewer
  reviewer -->|verify_claim: evidence and checks validated| verifier
  verifier -->|queue_release: ready changeset enters queue| merge_queue
  reviewer -->|override: signed human override handles rare mismatch| human_override
  merge_queue -->|finish: produced or terminal fail emitted| terminal
```

### Named Transitions

| Transition | From | To | Guard summary | Audit event | Recovery if denied |
|---|---|---|---|---|---|
| intake_route | request | intake | workflow event or human task arrives; Cedar permit required; evidence hash present | EVT-FOUNDRY-AGENT-EXECUTION-STARTED | Hold at request; append refusal reason; request fix or human override |
| plan_slice | intake | planner | planner decomposes and names gates; Cedar permit required; evidence hash present | EVT-FOUNDRY-AGENT-EXECUTION-STARTED | Hold at intake; append refusal reason; request fix or human override |
| deep_reason | planner | claude_opus | complex design or adversarial review required; Cedar permit required; evidence hash present | EVT-FOUNDRY-AGENT-EXECUTION-STARTED | Hold at planner; append refusal reason; request fix or human override |
| code_edit | planner | codex | repo-local implementation required; Cedar permit required; evidence hash present | EVT-FOUNDRY-HUMAN-OVERRIDE-USED | Hold at planner; append refusal reason; request fix or human override |
| execute_slice | codex | executor | bounded implementation lane runs; Cedar permit required; evidence hash present | EVT-FOUNDRY-AGENT-REVIEW-RECORDED | Hold at codex; append refusal reason; request fix or human override |
| review_slice | executor | reviewer | change class reviewer evaluates; Cedar permit required; evidence hash present | EVT-FOUNDRY-HUMAN-OVERRIDE-USED | Hold at executor; append refusal reason; request fix or human override |
| verify_claim | reviewer | verifier | evidence and checks validated; Cedar permit required; evidence hash present | EVT-FOUNDRY-AGENT-EXECUTION-STARTED | Hold at reviewer; append refusal reason; request fix or human override |
| queue_release | verifier | merge_queue | ready changeset enters queue; Cedar permit required; evidence hash present | EVT-FOUNDRY-HUMAN-OVERRIDE-USED | Hold at verifier; append refusal reason; request fix or human override |
| override | reviewer | human_override | signed human override handles rare mismatch; Cedar permit required; evidence hash present | EVT-FOUNDRY-AGENT-REVIEW-RECORDED | Hold at reviewer; append refusal reason; request fix or human override |
| finish | merge_queue | terminal | produced or terminal fail emitted; Cedar permit required; evidence hash present | EVT-FOUNDRY-AGENT-ROLE-ROUTED | Hold at merge_queue; append refusal reason; request fix or human override |
| replay-check-01 | request | intake | Replay validates intake ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-ROLE-ROUTED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-02 | intake | planner | Replay validates planner ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-03 | planner | claude_opus | Replay validates claude_opus ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-EXECUTION-STARTED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-04 | planner | codex | Replay validates codex ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-REVIEW-RECORDED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-05 | codex | executor | Replay validates executor ordering, signature, budget, and trace context | EVT-FOUNDRY-HUMAN-OVERRIDE-USED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-06 | executor | reviewer | Replay validates reviewer ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-ROLE-ROUTED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-07 | reviewer | verifier | Replay validates verifier ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-08 | verifier | merge_queue | Replay validates merge_queue ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-EXECUTION-STARTED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-09 | reviewer | human_override | Replay validates human_override ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-REVIEW-RECORDED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-10 | merge_queue | terminal | Replay validates terminal ordering, signature, budget, and trace context | EVT-FOUNDRY-HUMAN-OVERRIDE-USED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-11 | request | intake | Replay validates intake ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-ROLE-ROUTED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-12 | intake | planner | Replay validates planner ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-13 | planner | claude_opus | Replay validates claude_opus ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-EXECUTION-STARTED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-14 | planner | codex | Replay validates codex ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-REVIEW-RECORDED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-15 | codex | executor | Replay validates executor ordering, signature, budget, and trace context | EVT-FOUNDRY-HUMAN-OVERRIDE-USED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-16 | executor | reviewer | Replay validates reviewer ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-ROLE-ROUTED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-17 | reviewer | verifier | Replay validates verifier ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-18 | verifier | merge_queue | Replay validates merge_queue ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-EXECUTION-STARTED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-19 | reviewer | human_override | Replay validates human_override ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-REVIEW-RECORDED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-20 | merge_queue | terminal | Replay validates terminal ordering, signature, budget, and trace context | EVT-FOUNDRY-HUMAN-OVERRIDE-USED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-21 | request | intake | Replay validates intake ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-ROLE-ROUTED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-22 | intake | planner | Replay validates planner ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-23 | planner | claude_opus | Replay validates claude_opus ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-EXECUTION-STARTED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-24 | planner | codex | Replay validates codex ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-REVIEW-RECORDED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-25 | codex | executor | Replay validates executor ordering, signature, budget, and trace context | EVT-FOUNDRY-HUMAN-OVERRIDE-USED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-26 | executor | reviewer | Replay validates reviewer ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-ROLE-ROUTED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-27 | reviewer | verifier | Replay validates verifier ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-28 | verifier | merge_queue | Replay validates merge_queue ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-EXECUTION-STARTED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-29 | reviewer | human_override | Replay validates human_override ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-REVIEW-RECORDED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-30 | merge_queue | terminal | Replay validates terminal ordering, signature, budget, and trace context | EVT-FOUNDRY-HUMAN-OVERRIDE-USED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-31 | request | intake | Replay validates intake ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-ROLE-ROUTED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-32 | intake | planner | Replay validates planner ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-33 | planner | claude_opus | Replay validates claude_opus ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-EXECUTION-STARTED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-34 | planner | codex | Replay validates codex ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-REVIEW-RECORDED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-35 | codex | executor | Replay validates executor ordering, signature, budget, and trace context | EVT-FOUNDRY-HUMAN-OVERRIDE-USED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-36 | executor | reviewer | Replay validates reviewer ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-ROLE-ROUTED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-37 | reviewer | verifier | Replay validates verifier ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-38 | verifier | merge_queue | Replay validates merge_queue ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-EXECUTION-STARTED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-39 | reviewer | human_override | Replay validates human_override ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-REVIEW-RECORDED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-40 | merge_queue | terminal | Replay validates terminal ordering, signature, budget, and trace context | EVT-FOUNDRY-HUMAN-OVERRIDE-USED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-41 | request | intake | Replay validates intake ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-ROLE-ROUTED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-42 | intake | planner | Replay validates planner ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-43 | planner | claude_opus | Replay validates claude_opus ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-EXECUTION-STARTED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-44 | planner | codex | Replay validates codex ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-REVIEW-RECORDED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-45 | codex | executor | Replay validates executor ordering, signature, budget, and trace context | EVT-FOUNDRY-HUMAN-OVERRIDE-USED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-46 | executor | reviewer | Replay validates reviewer ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-ROLE-ROUTED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-47 | reviewer | verifier | Replay validates verifier ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-48 | verifier | merge_queue | Replay validates merge_queue ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-EXECUTION-STARTED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-49 | reviewer | human_override | Replay validates human_override ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-REVIEW-RECORDED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-50 | merge_queue | terminal | Replay validates terminal ordering, signature, budget, and trace context | EVT-FOUNDRY-HUMAN-OVERRIDE-USED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-51 | request | intake | Replay validates intake ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-ROLE-ROUTED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-52 | intake | planner | Replay validates planner ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-53 | planner | claude_opus | Replay validates claude_opus ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-EXECUTION-STARTED | Recompute from append-only log and refuse non-monotonic row |
| replay-check-54 | planner | codex | Replay validates codex ordering, signature, budget, and trace context | EVT-FOUNDRY-AGENT-REVIEW-RECORDED | Recompute from append-only log and refuse non-monotonic row |

## Cedar Policy Bindings

The bindings below use specific internal principals, actions, and resources. A deny from any narrower policy wins over these permits.

| Binding | Principal | Action | Resource | Required context | Decision |
|---|---|---|---|---|---|
| B-001 | Principal::"oyatie.agent.codex" | Action::"foundry.agent.plan" | Resource::"agent-role:executor" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-002 | Principal::"oyatie.agent.codex" | Action::"foundry.agent.execute" | Resource::"agent-role:claude-opus" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-003 | Principal::"oyatie.agent.codex" | Action::"foundry.agent.review" | Resource::"agent-role:codex" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-004 | Principal::"oyatie.agent.codex" | Action::"foundry.agent.verify" | Resource::"agent-role:reviewer" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-005 | Principal::"oyatie.agent.codex" | Action::"foundry.agent.invoke_claude_opus" | Resource::"agent-role:verifier" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-006 | Principal::"oyatie.agent.claude-opus" | Action::"foundry.agent.plan" | Resource::"repo:oyatie/microservices/foundry" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-007 | Principal::"oyatie.agent.claude-opus" | Action::"foundry.agent.execute" | Resource::"branch:dev" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-008 | Principal::"oyatie.agent.claude-opus" | Action::"foundry.agent.review" | Resource::"queue:foundry-dev" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-009 | Principal::"oyatie.agent.claude-opus" | Action::"foundry.agent.verify" | Resource::"event-log:registry/vcs/changeset-event-log.json" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-010 | Principal::"oyatie.agent.claude-opus" | Action::"foundry.agent.invoke_claude_opus" | Resource::"event-router:registry/vcs/event-router.yaml" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-011 | Principal::"oyatie.agent.planner" | Action::"foundry.agent.plan" | Resource::"safe-paths:registry/vcs/concurrent-safe-paths.yaml" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-012 | Principal::"oyatie.agent.planner" | Action::"foundry.agent.execute" | Resource::"evidence:evidence/multispectrum" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-013 | Principal::"oyatie.agent.planner" | Action::"foundry.agent.review" | Resource::"audit:event-class/foundry" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-014 | Principal::"oyatie.agent.planner" | Action::"foundry.agent.verify" | Resource::"agent-role:planner" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-015 | Principal::"oyatie.agent.planner" | Action::"foundry.agent.invoke_claude_opus" | Resource::"agent-role:executor" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-016 | Principal::"oyatie.agent.executor" | Action::"foundry.agent.plan" | Resource::"agent-role:claude-opus" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-017 | Principal::"oyatie.agent.executor" | Action::"foundry.agent.execute" | Resource::"agent-role:codex" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-018 | Principal::"oyatie.agent.executor" | Action::"foundry.agent.review" | Resource::"agent-role:reviewer" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-019 | Principal::"oyatie.agent.executor" | Action::"foundry.agent.verify" | Resource::"agent-role:verifier" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-020 | Principal::"oyatie.agent.executor" | Action::"foundry.agent.invoke_claude_opus" | Resource::"repo:oyatie/microservices/foundry" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-021 | Principal::"oyatie.service.vcs-orchestrator" | Action::"foundry.agent.plan" | Resource::"branch:dev" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-022 | Principal::"oyatie.service.vcs-orchestrator" | Action::"foundry.agent.execute" | Resource::"queue:foundry-dev" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-023 | Principal::"oyatie.service.vcs-orchestrator" | Action::"foundry.agent.review" | Resource::"event-log:registry/vcs/changeset-event-log.json" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-024 | Principal::"oyatie.service.vcs-orchestrator" | Action::"foundry.agent.verify" | Resource::"event-router:registry/vcs/event-router.yaml" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-025 | Principal::"oyatie.service.vcs-orchestrator" | Action::"foundry.agent.invoke_claude_opus" | Resource::"safe-paths:registry/vcs/concurrent-safe-paths.yaml" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-026 | Principal::"oyatie.service.webhook-receiver" | Action::"foundry.agent.plan" | Resource::"evidence:evidence/multispectrum" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-027 | Principal::"oyatie.service.webhook-receiver" | Action::"foundry.agent.execute" | Resource::"audit:event-class/foundry" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-028 | Principal::"oyatie.service.webhook-receiver" | Action::"foundry.agent.review" | Resource::"agent-role:planner" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-029 | Principal::"oyatie.service.webhook-receiver" | Action::"foundry.agent.verify" | Resource::"agent-role:executor" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-030 | Principal::"oyatie.service.webhook-receiver" | Action::"foundry.agent.invoke_claude_opus" | Resource::"agent-role:claude-opus" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-031 | Principal::"oyatie.service.merge-queue" | Action::"foundry.agent.plan" | Resource::"agent-role:codex" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-032 | Principal::"oyatie.service.merge-queue" | Action::"foundry.agent.execute" | Resource::"agent-role:reviewer" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-033 | Principal::"oyatie.service.merge-queue" | Action::"foundry.agent.review" | Resource::"agent-role:verifier" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-034 | Principal::"oyatie.service.merge-queue" | Action::"foundry.agent.verify" | Resource::"repo:oyatie/microservices/foundry" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-035 | Principal::"oyatie.service.merge-queue" | Action::"foundry.agent.invoke_claude_opus" | Resource::"branch:dev" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-036 | Principal::"oyatie.service.admission-gate" | Action::"foundry.agent.plan" | Resource::"queue:foundry-dev" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-037 | Principal::"oyatie.service.admission-gate" | Action::"foundry.agent.execute" | Resource::"event-log:registry/vcs/changeset-event-log.json" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-038 | Principal::"oyatie.service.admission-gate" | Action::"foundry.agent.review" | Resource::"event-router:registry/vcs/event-router.yaml" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-039 | Principal::"oyatie.service.admission-gate" | Action::"foundry.agent.verify" | Resource::"safe-paths:registry/vcs/concurrent-safe-paths.yaml" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-040 | Principal::"oyatie.service.admission-gate" | Action::"foundry.agent.invoke_claude_opus" | Resource::"evidence:evidence/multispectrum" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-041 | Principal::"oyatie.service.completion-gate" | Action::"foundry.agent.plan" | Resource::"audit:event-class/foundry" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-042 | Principal::"oyatie.service.completion-gate" | Action::"foundry.agent.execute" | Resource::"agent-role:planner" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-043 | Principal::"oyatie.service.completion-gate" | Action::"foundry.agent.review" | Resource::"agent-role:executor" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-044 | Principal::"oyatie.service.completion-gate" | Action::"foundry.agent.verify" | Resource::"agent-role:claude-opus" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-045 | Principal::"oyatie.service.completion-gate" | Action::"foundry.agent.invoke_claude_opus" | Resource::"agent-role:codex" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-046 | Principal::"oyatie.human.reviewer" | Action::"foundry.agent.plan" | Resource::"agent-role:reviewer" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-047 | Principal::"oyatie.human.reviewer" | Action::"foundry.agent.execute" | Resource::"agent-role:verifier" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-048 | Principal::"oyatie.human.reviewer" | Action::"foundry.agent.review" | Resource::"repo:oyatie/microservices/foundry" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-049 | Principal::"oyatie.human.reviewer" | Action::"foundry.agent.verify" | Resource::"branch:dev" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |
| B-050 | Principal::"oyatie.human.reviewer" | Action::"foundry.agent.invoke_claude_opus" | Resource::"queue:foundry-dev" | workflow=foundry_pipeline; tenant_id=oyatie; intent=agent-types-and-roles | permit when evidence_hash and changeset_id are present |

```cedar
permit(
  principal in Principal::"oyatie.agent.codex",
  action == Action::"foundry.agent.plan",
  resource in Resource::"repo:oyatie/microservices/foundry"
) when {
  context.tenant_id == "oyatie" &&
  context.workflow == "foundry_pipeline" &&
  context.related_adr == "ADR-0221" &&
  context.evidence_hash != "" &&
  context.trace_id != ""
};

permit(
  principal in Principal::"oyatie.agent.codex",
  action == Action::"foundry.agent.execute",
  resource in Resource::"repo:oyatie/microservices/foundry"
) when {
  context.tenant_id == "oyatie" &&
  context.workflow == "foundry_pipeline" &&
  context.related_adr == "ADR-0221" &&
  context.evidence_hash != "" &&
  context.trace_id != ""
};

permit(
  principal in Principal::"oyatie.agent.codex",
  action == Action::"foundry.agent.review",
  resource in Resource::"repo:oyatie/microservices/foundry"
) when {
  context.tenant_id == "oyatie" &&
  context.workflow == "foundry_pipeline" &&
  context.related_adr == "ADR-0221" &&
  context.evidence_hash != "" &&
  context.trace_id != ""
};

permit(
  principal in Principal::"oyatie.agent.codex",
  action == Action::"foundry.agent.verify",
  resource in Resource::"repo:oyatie/microservices/foundry"
) when {
  context.tenant_id == "oyatie" &&
  context.workflow == "foundry_pipeline" &&
  context.related_adr == "ADR-0221" &&
  context.evidence_hash != "" &&
  context.trace_id != ""
};

forbid(
  principal,
  action,
  resource in Resource::"repo:oyatie/microservices/foundry/decisions"
) when {
  context.intent == "agent-types-and-roles" &&
  context.owner != "per-msvc-adrs-batch-d"
};
```

### Cedar Guard Catalog

- Guard 01: Principal::"oyatie.agent.codex" may invoke foundry.agent.plan on Resource::"agent-role:planner" only while `intake` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 02: Principal::"oyatie.agent.claude-opus" may invoke foundry.agent.execute on Resource::"agent-role:executor" only while `planner` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 03: Principal::"oyatie.agent.planner" may invoke foundry.agent.review on Resource::"agent-role:claude-opus" only while `claude_opus` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 04: Principal::"oyatie.agent.executor" may invoke foundry.agent.verify on Resource::"agent-role:codex" only while `codex` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 05: Principal::"oyatie.service.vcs-orchestrator" may invoke foundry.agent.invoke_claude_opus on Resource::"agent-role:reviewer" only while `executor` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 06: Principal::"oyatie.service.webhook-receiver" may invoke foundry.agent.invoke_codex on Resource::"agent-role:verifier" only while `reviewer` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 07: Principal::"oyatie.service.merge-queue" may invoke foundry.agent.escalate_human on Resource::"repo:oyatie/microservices/foundry" only while `verifier` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 08: Principal::"oyatie.service.admission-gate" may invoke foundry.agent.record_handoff on Resource::"branch:dev" only while `merge_queue` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 09: Principal::"oyatie.service.completion-gate" may invoke foundry.agent.plan on Resource::"queue:foundry-dev" only while `human_override` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 10: Principal::"oyatie.human.reviewer" may invoke foundry.agent.execute on Resource::"event-log:registry/vcs/changeset-event-log.json" only while `terminal` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 11: Principal::"oyatie.agent.codex" may invoke foundry.agent.review on Resource::"event-router:registry/vcs/event-router.yaml" only while `intake` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 12: Principal::"oyatie.agent.claude-opus" may invoke foundry.agent.verify on Resource::"safe-paths:registry/vcs/concurrent-safe-paths.yaml" only while `planner` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 13: Principal::"oyatie.agent.planner" may invoke foundry.agent.invoke_claude_opus on Resource::"evidence:evidence/multispectrum" only while `claude_opus` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 14: Principal::"oyatie.agent.executor" may invoke foundry.agent.invoke_codex on Resource::"audit:event-class/foundry" only while `codex` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 15: Principal::"oyatie.service.vcs-orchestrator" may invoke foundry.agent.escalate_human on Resource::"agent-role:planner" only while `executor` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 16: Principal::"oyatie.service.webhook-receiver" may invoke foundry.agent.record_handoff on Resource::"agent-role:executor" only while `reviewer` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 17: Principal::"oyatie.service.merge-queue" may invoke foundry.agent.plan on Resource::"agent-role:claude-opus" only while `verifier` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 18: Principal::"oyatie.service.admission-gate" may invoke foundry.agent.execute on Resource::"agent-role:codex" only while `merge_queue` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 19: Principal::"oyatie.service.completion-gate" may invoke foundry.agent.review on Resource::"agent-role:reviewer" only while `human_override` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 20: Principal::"oyatie.human.reviewer" may invoke foundry.agent.verify on Resource::"agent-role:verifier" only while `terminal` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 21: Principal::"oyatie.agent.codex" may invoke foundry.agent.invoke_claude_opus on Resource::"repo:oyatie/microservices/foundry" only while `intake` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 22: Principal::"oyatie.agent.claude-opus" may invoke foundry.agent.invoke_codex on Resource::"branch:dev" only while `planner` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 23: Principal::"oyatie.agent.planner" may invoke foundry.agent.escalate_human on Resource::"queue:foundry-dev" only while `claude_opus` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 24: Principal::"oyatie.agent.executor" may invoke foundry.agent.record_handoff on Resource::"event-log:registry/vcs/changeset-event-log.json" only while `codex` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 25: Principal::"oyatie.service.vcs-orchestrator" may invoke foundry.agent.plan on Resource::"event-router:registry/vcs/event-router.yaml" only while `executor` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 26: Principal::"oyatie.service.webhook-receiver" may invoke foundry.agent.execute on Resource::"safe-paths:registry/vcs/concurrent-safe-paths.yaml" only while `reviewer` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 27: Principal::"oyatie.service.merge-queue" may invoke foundry.agent.review on Resource::"evidence:evidence/multispectrum" only while `verifier` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 28: Principal::"oyatie.service.admission-gate" may invoke foundry.agent.verify on Resource::"audit:event-class/foundry" only while `merge_queue` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 29: Principal::"oyatie.service.completion-gate" may invoke foundry.agent.invoke_claude_opus on Resource::"agent-role:planner" only while `human_override` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 30: Principal::"oyatie.human.reviewer" may invoke foundry.agent.invoke_codex on Resource::"agent-role:executor" only while `terminal` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 31: Principal::"oyatie.agent.codex" may invoke foundry.agent.escalate_human on Resource::"agent-role:claude-opus" only while `intake` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 32: Principal::"oyatie.agent.claude-opus" may invoke foundry.agent.record_handoff on Resource::"agent-role:codex" only while `planner` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 33: Principal::"oyatie.agent.planner" may invoke foundry.agent.plan on Resource::"agent-role:reviewer" only while `claude_opus` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 34: Principal::"oyatie.agent.executor" may invoke foundry.agent.execute on Resource::"agent-role:verifier" only while `codex` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 35: Principal::"oyatie.service.vcs-orchestrator" may invoke foundry.agent.review on Resource::"repo:oyatie/microservices/foundry" only while `executor` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 36: Principal::"oyatie.service.webhook-receiver" may invoke foundry.agent.verify on Resource::"branch:dev" only while `reviewer` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 37: Principal::"oyatie.service.merge-queue" may invoke foundry.agent.invoke_claude_opus on Resource::"queue:foundry-dev" only while `verifier` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 38: Principal::"oyatie.service.admission-gate" may invoke foundry.agent.invoke_codex on Resource::"event-log:registry/vcs/changeset-event-log.json" only while `merge_queue` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 39: Principal::"oyatie.service.completion-gate" may invoke foundry.agent.escalate_human on Resource::"event-router:registry/vcs/event-router.yaml" only while `human_override` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 40: Principal::"oyatie.human.reviewer" may invoke foundry.agent.record_handoff on Resource::"safe-paths:registry/vcs/concurrent-safe-paths.yaml" only while `terminal` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 41: Principal::"oyatie.agent.codex" may invoke foundry.agent.plan on Resource::"evidence:evidence/multispectrum" only while `intake` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 42: Principal::"oyatie.agent.claude-opus" may invoke foundry.agent.execute on Resource::"audit:event-class/foundry" only while `planner` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 43: Principal::"oyatie.agent.planner" may invoke foundry.agent.review on Resource::"agent-role:planner" only while `claude_opus` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 44: Principal::"oyatie.agent.executor" may invoke foundry.agent.verify on Resource::"agent-role:executor" only while `codex` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 45: Principal::"oyatie.service.vcs-orchestrator" may invoke foundry.agent.invoke_claude_opus on Resource::"agent-role:claude-opus" only while `executor` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 46: Principal::"oyatie.service.webhook-receiver" may invoke foundry.agent.invoke_codex on Resource::"agent-role:codex" only while `reviewer` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 47: Principal::"oyatie.service.merge-queue" may invoke foundry.agent.escalate_human on Resource::"agent-role:reviewer" only while `verifier` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 48: Principal::"oyatie.service.admission-gate" may invoke foundry.agent.record_handoff on Resource::"agent-role:verifier" only while `merge_queue` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 49: Principal::"oyatie.service.completion-gate" may invoke foundry.agent.plan on Resource::"repo:oyatie/microservices/foundry" only while `human_override` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 50: Principal::"oyatie.human.reviewer" may invoke foundry.agent.execute on Resource::"branch:dev" only while `terminal` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 51: Principal::"oyatie.agent.codex" may invoke foundry.agent.review on Resource::"queue:foundry-dev" only while `intake` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 52: Principal::"oyatie.agent.claude-opus" may invoke foundry.agent.verify on Resource::"event-log:registry/vcs/changeset-event-log.json" only while `planner` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 53: Principal::"oyatie.agent.planner" may invoke foundry.agent.invoke_claude_opus on Resource::"event-router:registry/vcs/event-router.yaml" only while `claude_opus` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 54: Principal::"oyatie.agent.executor" may invoke foundry.agent.invoke_codex on Resource::"safe-paths:registry/vcs/concurrent-safe-paths.yaml" only while `codex` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 55: Principal::"oyatie.service.vcs-orchestrator" may invoke foundry.agent.escalate_human on Resource::"evidence:evidence/multispectrum" only while `executor` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 56: Principal::"oyatie.service.webhook-receiver" may invoke foundry.agent.record_handoff on Resource::"audit:event-class/foundry" only while `reviewer` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 57: Principal::"oyatie.service.merge-queue" may invoke foundry.agent.plan on Resource::"agent-role:planner" only while `verifier` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 58: Principal::"oyatie.service.admission-gate" may invoke foundry.agent.execute on Resource::"agent-role:executor" only while `merge_queue` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 59: Principal::"oyatie.service.completion-gate" may invoke foundry.agent.review on Resource::"agent-role:claude-opus" only while `human_override` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 60: Principal::"oyatie.human.reviewer" may invoke foundry.agent.verify on Resource::"agent-role:codex" only while `terminal` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 61: Principal::"oyatie.agent.codex" may invoke foundry.agent.invoke_claude_opus on Resource::"agent-role:reviewer" only while `intake` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 62: Principal::"oyatie.agent.claude-opus" may invoke foundry.agent.invoke_codex on Resource::"agent-role:verifier" only while `planner` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 63: Principal::"oyatie.agent.planner" may invoke foundry.agent.escalate_human on Resource::"repo:oyatie/microservices/foundry" only while `claude_opus` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 64: Principal::"oyatie.agent.executor" may invoke foundry.agent.record_handoff on Resource::"branch:dev" only while `codex` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 65: Principal::"oyatie.service.vcs-orchestrator" may invoke foundry.agent.plan on Resource::"queue:foundry-dev" only while `executor` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 66: Principal::"oyatie.service.webhook-receiver" may invoke foundry.agent.execute on Resource::"event-log:registry/vcs/changeset-event-log.json" only while `reviewer` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 67: Principal::"oyatie.service.merge-queue" may invoke foundry.agent.review on Resource::"event-router:registry/vcs/event-router.yaml" only while `verifier` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 68: Principal::"oyatie.service.admission-gate" may invoke foundry.agent.verify on Resource::"safe-paths:registry/vcs/concurrent-safe-paths.yaml" only while `merge_queue` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 69: Principal::"oyatie.service.completion-gate" may invoke foundry.agent.invoke_claude_opus on Resource::"evidence:evidence/multispectrum" only while `human_override` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.
- Guard 70: Principal::"oyatie.human.reviewer" may invoke foundry.agent.invoke_codex on Resource::"audit:event-class/foundry" only while `terminal` is current, the changeset id is stable, the event is signed, and the ADR-0221 invariant is cited.

## Audit Event Classes Emitted (per ADR-0263)

Every event class below emits structured JSON logs with schema=oyatie/log/v1, tenant_id=oyatie, microservice=foundry, trace_id, span_id, and audit_id when the operation changes state.

| Event class | Emitted when | Required fields | Retention | Cardinality guard |
|---|---|---|---|---|
| EVT-FOUNDRY-AGENT-ROLE-ROUTED | Foundry Agent Types and Roles changes a durable pipeline fact | changeset_id, actor, action, resource, from_state, to_state, trace_id, span_id, audit_id, evidence_hash | 7 years for audit chain; 90 days hot observability | event_class + changeset_id only; no raw prompt or secret values |
| EVT-FOUNDRY-AGENT-HANDOFF-RECORDED | Foundry Agent Types and Roles changes a durable pipeline fact | changeset_id, actor, action, resource, from_state, to_state, trace_id, span_id, audit_id, evidence_hash | 7 years for audit chain; 90 days hot observability | event_class + changeset_id only; no raw prompt or secret values |
| EVT-FOUNDRY-AGENT-EXECUTION-STARTED | Foundry Agent Types and Roles changes a durable pipeline fact | changeset_id, actor, action, resource, from_state, to_state, trace_id, span_id, audit_id, evidence_hash | 7 years for audit chain; 90 days hot observability | event_class + changeset_id only; no raw prompt or secret values |
| EVT-FOUNDRY-AGENT-REVIEW-RECORDED | Foundry Agent Types and Roles changes a durable pipeline fact | changeset_id, actor, action, resource, from_state, to_state, trace_id, span_id, audit_id, evidence_hash | 7 years for audit chain; 90 days hot observability | event_class + changeset_id only; no raw prompt or secret values |
| EVT-FOUNDRY-HUMAN-OVERRIDE-USED | Foundry Agent Types and Roles changes a durable pipeline fact | changeset_id, actor, action, resource, from_state, to_state, trace_id, span_id, audit_id, evidence_hash | 7 years for audit chain; 90 days hot observability | event_class + changeset_id only; no raw prompt or secret values |
| EVT-FOUNDRY-AGENT-ROLE-ROUTED-001 | claim path observes intake | tenant_id=oyatie, sub_scope=oyatie.foundry.claim, policy_id=ADR-0221.intake, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-HANDOFF-RECORDED-002 | verify path observes planner | tenant_id=oyatie, sub_scope=oyatie.foundry.verify, policy_id=ADR-0221.planner, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-EXECUTION-STARTED-003 | done path observes claude_opus | tenant_id=oyatie, sub_scope=oyatie.foundry.done, policy_id=ADR-0221.claude_opus, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-REVIEW-RECORDED-004 | admission path observes codex | tenant_id=oyatie, sub_scope=oyatie.foundry.admission, policy_id=ADR-0221.codex, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-HUMAN-OVERRIDE-USED-005 | completion path observes executor | tenant_id=oyatie, sub_scope=oyatie.foundry.completion, policy_id=ADR-0221.executor, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-ROLE-ROUTED-006 | merge_queue path observes reviewer | tenant_id=oyatie, sub_scope=oyatie.foundry.merge_queue, policy_id=ADR-0221.reviewer, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-HANDOFF-RECORDED-007 | webhook path observes verifier | tenant_id=oyatie, sub_scope=oyatie.foundry.webhook, policy_id=ADR-0221.verifier, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-EXECUTION-STARTED-008 | review path observes merge_queue | tenant_id=oyatie, sub_scope=oyatie.foundry.review, policy_id=ADR-0221.merge_queue, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-REVIEW-RECORDED-009 | promotion path observes human_override | tenant_id=oyatie, sub_scope=oyatie.foundry.promotion, policy_id=ADR-0221.human_override, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-HUMAN-OVERRIDE-USED-010 | override path observes terminal | tenant_id=oyatie, sub_scope=oyatie.foundry.override, policy_id=ADR-0221.terminal, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-ROLE-ROUTED-011 | claim path observes intake | tenant_id=oyatie, sub_scope=oyatie.foundry.claim, policy_id=ADR-0221.intake, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-HANDOFF-RECORDED-012 | verify path observes planner | tenant_id=oyatie, sub_scope=oyatie.foundry.verify, policy_id=ADR-0221.planner, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-EXECUTION-STARTED-013 | done path observes claude_opus | tenant_id=oyatie, sub_scope=oyatie.foundry.done, policy_id=ADR-0221.claude_opus, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-REVIEW-RECORDED-014 | admission path observes codex | tenant_id=oyatie, sub_scope=oyatie.foundry.admission, policy_id=ADR-0221.codex, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-HUMAN-OVERRIDE-USED-015 | completion path observes executor | tenant_id=oyatie, sub_scope=oyatie.foundry.completion, policy_id=ADR-0221.executor, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-ROLE-ROUTED-016 | merge_queue path observes reviewer | tenant_id=oyatie, sub_scope=oyatie.foundry.merge_queue, policy_id=ADR-0221.reviewer, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-HANDOFF-RECORDED-017 | webhook path observes verifier | tenant_id=oyatie, sub_scope=oyatie.foundry.webhook, policy_id=ADR-0221.verifier, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-EXECUTION-STARTED-018 | review path observes merge_queue | tenant_id=oyatie, sub_scope=oyatie.foundry.review, policy_id=ADR-0221.merge_queue, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-REVIEW-RECORDED-019 | promotion path observes human_override | tenant_id=oyatie, sub_scope=oyatie.foundry.promotion, policy_id=ADR-0221.human_override, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-HUMAN-OVERRIDE-USED-020 | override path observes terminal | tenant_id=oyatie, sub_scope=oyatie.foundry.override, policy_id=ADR-0221.terminal, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-ROLE-ROUTED-021 | claim path observes intake | tenant_id=oyatie, sub_scope=oyatie.foundry.claim, policy_id=ADR-0221.intake, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-HANDOFF-RECORDED-022 | verify path observes planner | tenant_id=oyatie, sub_scope=oyatie.foundry.verify, policy_id=ADR-0221.planner, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-EXECUTION-STARTED-023 | done path observes claude_opus | tenant_id=oyatie, sub_scope=oyatie.foundry.done, policy_id=ADR-0221.claude_opus, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-REVIEW-RECORDED-024 | admission path observes codex | tenant_id=oyatie, sub_scope=oyatie.foundry.admission, policy_id=ADR-0221.codex, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-HUMAN-OVERRIDE-USED-025 | completion path observes executor | tenant_id=oyatie, sub_scope=oyatie.foundry.completion, policy_id=ADR-0221.executor, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-ROLE-ROUTED-026 | merge_queue path observes reviewer | tenant_id=oyatie, sub_scope=oyatie.foundry.merge_queue, policy_id=ADR-0221.reviewer, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-HANDOFF-RECORDED-027 | webhook path observes verifier | tenant_id=oyatie, sub_scope=oyatie.foundry.webhook, policy_id=ADR-0221.verifier, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-EXECUTION-STARTED-028 | review path observes merge_queue | tenant_id=oyatie, sub_scope=oyatie.foundry.review, policy_id=ADR-0221.merge_queue, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-REVIEW-RECORDED-029 | promotion path observes human_override | tenant_id=oyatie, sub_scope=oyatie.foundry.promotion, policy_id=ADR-0221.human_override, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-HUMAN-OVERRIDE-USED-030 | override path observes terminal | tenant_id=oyatie, sub_scope=oyatie.foundry.override, policy_id=ADR-0221.terminal, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-ROLE-ROUTED-031 | claim path observes intake | tenant_id=oyatie, sub_scope=oyatie.foundry.claim, policy_id=ADR-0221.intake, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-HANDOFF-RECORDED-032 | verify path observes planner | tenant_id=oyatie, sub_scope=oyatie.foundry.verify, policy_id=ADR-0221.planner, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-EXECUTION-STARTED-033 | done path observes claude_opus | tenant_id=oyatie, sub_scope=oyatie.foundry.done, policy_id=ADR-0221.claude_opus, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-REVIEW-RECORDED-034 | admission path observes codex | tenant_id=oyatie, sub_scope=oyatie.foundry.admission, policy_id=ADR-0221.codex, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-HUMAN-OVERRIDE-USED-035 | completion path observes executor | tenant_id=oyatie, sub_scope=oyatie.foundry.completion, policy_id=ADR-0221.executor, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-ROLE-ROUTED-036 | merge_queue path observes reviewer | tenant_id=oyatie, sub_scope=oyatie.foundry.merge_queue, policy_id=ADR-0221.reviewer, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-HANDOFF-RECORDED-037 | webhook path observes verifier | tenant_id=oyatie, sub_scope=oyatie.foundry.webhook, policy_id=ADR-0221.verifier, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-EXECUTION-STARTED-038 | review path observes merge_queue | tenant_id=oyatie, sub_scope=oyatie.foundry.review, policy_id=ADR-0221.merge_queue, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-REVIEW-RECORDED-039 | promotion path observes human_override | tenant_id=oyatie, sub_scope=oyatie.foundry.promotion, policy_id=ADR-0221.human_override, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-HUMAN-OVERRIDE-USED-040 | override path observes terminal | tenant_id=oyatie, sub_scope=oyatie.foundry.override, policy_id=ADR-0221.terminal, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-ROLE-ROUTED-041 | claim path observes intake | tenant_id=oyatie, sub_scope=oyatie.foundry.claim, policy_id=ADR-0221.intake, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-HANDOFF-RECORDED-042 | verify path observes planner | tenant_id=oyatie, sub_scope=oyatie.foundry.verify, policy_id=ADR-0221.planner, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-EXECUTION-STARTED-043 | done path observes claude_opus | tenant_id=oyatie, sub_scope=oyatie.foundry.done, policy_id=ADR-0221.claude_opus, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-REVIEW-RECORDED-044 | admission path observes codex | tenant_id=oyatie, sub_scope=oyatie.foundry.admission, policy_id=ADR-0221.codex, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-HUMAN-OVERRIDE-USED-045 | completion path observes executor | tenant_id=oyatie, sub_scope=oyatie.foundry.completion, policy_id=ADR-0221.executor, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-ROLE-ROUTED-046 | merge_queue path observes reviewer | tenant_id=oyatie, sub_scope=oyatie.foundry.merge_queue, policy_id=ADR-0221.reviewer, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-HANDOFF-RECORDED-047 | webhook path observes verifier | tenant_id=oyatie, sub_scope=oyatie.foundry.webhook, policy_id=ADR-0221.verifier, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-EXECUTION-STARTED-048 | review path observes merge_queue | tenant_id=oyatie, sub_scope=oyatie.foundry.review, policy_id=ADR-0221.merge_queue, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-REVIEW-RECORDED-049 | promotion path observes human_override | tenant_id=oyatie, sub_scope=oyatie.foundry.promotion, policy_id=ADR-0221.human_override, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-HUMAN-OVERRIDE-USED-050 | override path observes terminal | tenant_id=oyatie, sub_scope=oyatie.foundry.override, policy_id=ADR-0221.terminal, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-ROLE-ROUTED-051 | claim path observes intake | tenant_id=oyatie, sub_scope=oyatie.foundry.claim, policy_id=ADR-0221.intake, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-HANDOFF-RECORDED-052 | verify path observes planner | tenant_id=oyatie, sub_scope=oyatie.foundry.verify, policy_id=ADR-0221.planner, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-EXECUTION-STARTED-053 | done path observes claude_opus | tenant_id=oyatie, sub_scope=oyatie.foundry.done, policy_id=ADR-0221.claude_opus, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-REVIEW-RECORDED-054 | admission path observes codex | tenant_id=oyatie, sub_scope=oyatie.foundry.admission, policy_id=ADR-0221.codex, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-HUMAN-OVERRIDE-USED-055 | completion path observes executor | tenant_id=oyatie, sub_scope=oyatie.foundry.completion, policy_id=ADR-0221.executor, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-ROLE-ROUTED-056 | merge_queue path observes reviewer | tenant_id=oyatie, sub_scope=oyatie.foundry.merge_queue, policy_id=ADR-0221.reviewer, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-HANDOFF-RECORDED-057 | webhook path observes verifier | tenant_id=oyatie, sub_scope=oyatie.foundry.webhook, policy_id=ADR-0221.verifier, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-EXECUTION-STARTED-058 | review path observes merge_queue | tenant_id=oyatie, sub_scope=oyatie.foundry.review, policy_id=ADR-0221.merge_queue, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-REVIEW-RECORDED-059 | promotion path observes human_override | tenant_id=oyatie, sub_scope=oyatie.foundry.promotion, policy_id=ADR-0221.human_override, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-HUMAN-OVERRIDE-USED-060 | override path observes terminal | tenant_id=oyatie, sub_scope=oyatie.foundry.override, policy_id=ADR-0221.terminal, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-ROLE-ROUTED-061 | claim path observes intake | tenant_id=oyatie, sub_scope=oyatie.foundry.claim, policy_id=ADR-0221.intake, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-HANDOFF-RECORDED-062 | verify path observes planner | tenant_id=oyatie, sub_scope=oyatie.foundry.verify, policy_id=ADR-0221.planner, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-EXECUTION-STARTED-063 | done path observes claude_opus | tenant_id=oyatie, sub_scope=oyatie.foundry.done, policy_id=ADR-0221.claude_opus, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-REVIEW-RECORDED-064 | admission path observes codex | tenant_id=oyatie, sub_scope=oyatie.foundry.admission, policy_id=ADR-0221.codex, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-HUMAN-OVERRIDE-USED-065 | completion path observes executor | tenant_id=oyatie, sub_scope=oyatie.foundry.completion, policy_id=ADR-0221.executor, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-ROLE-ROUTED-066 | merge_queue path observes reviewer | tenant_id=oyatie, sub_scope=oyatie.foundry.merge_queue, policy_id=ADR-0221.reviewer, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-HANDOFF-RECORDED-067 | webhook path observes verifier | tenant_id=oyatie, sub_scope=oyatie.foundry.webhook, policy_id=ADR-0221.verifier, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-EXECUTION-STARTED-068 | review path observes merge_queue | tenant_id=oyatie, sub_scope=oyatie.foundry.review, policy_id=ADR-0221.merge_queue, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-REVIEW-RECORDED-069 | promotion path observes human_override | tenant_id=oyatie, sub_scope=oyatie.foundry.promotion, policy_id=ADR-0221.human_override, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-HUMAN-OVERRIDE-USED-070 | override path observes terminal | tenant_id=oyatie, sub_scope=oyatie.foundry.override, policy_id=ADR-0221.terminal, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-ROLE-ROUTED-071 | claim path observes intake | tenant_id=oyatie, sub_scope=oyatie.foundry.claim, policy_id=ADR-0221.intake, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-HANDOFF-RECORDED-072 | verify path observes planner | tenant_id=oyatie, sub_scope=oyatie.foundry.verify, policy_id=ADR-0221.planner, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-EXECUTION-STARTED-073 | done path observes claude_opus | tenant_id=oyatie, sub_scope=oyatie.foundry.done, policy_id=ADR-0221.claude_opus, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-AGENT-REVIEW-RECORDED-074 | admission path observes codex | tenant_id=oyatie, sub_scope=oyatie.foundry.admission, policy_id=ADR-0221.codex, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |
| EVT-FOUNDRY-HUMAN-OVERRIDE-USED-075 | completion path observes executor | tenant_id=oyatie, sub_scope=oyatie.foundry.completion, policy_id=ADR-0221.executor, correlation_id, audit_id | hot 90d + sealed audit chain | bounded by changeset_id and delivery_id; free text is scrubbed |

## Failure Modes + Recovery

| Failure mode | Detection | Immediate recovery | Durable prevention | Audit event |
|---|---|---|---|---|
| missing_evidence-1 | evidence bundle or multispectrum file absent during intake | hold current state and request fix | admission refuses until evidence hash exists; oya-governance-changeset-state-monotonicity records the check | EVT-FOUNDRY-AGENT-ROLE-ROUTED |
| cedar_deny-1 | policy evaluation denies actor/action/resource during planner | refuse transition and expose denial reason | add regression fixture for the denied scenario; oya-governance-changeset-state-enum-closed records the check | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED |
| signature_invalid-1 | Ed25519/HMAC signature mismatch during claude_opus | drop event before state mutation | rotate secret/key and run signature replay tests; oya-governance-merge-queue-ref-hygiene records the check | EVT-FOUNDRY-AGENT-EXECUTION-STARTED |
| idempotency_collision-1 | same dedup key maps to different payload during codex | quarantine event and require human review | dedup log monotonic lane fails; oya-governance-event-router-completeness records the check | EVT-FOUNDRY-AGENT-REVIEW-RECORDED |
| budget_exhausted-1 | cost budget counter reaches zero during executor | transition to cost_exhausted | monthly budget lane alerts owning team; oya-governance-webhook-stuck records the check | EVT-FOUNDRY-HUMAN-OVERRIDE-USED |
| trace_missing-1 | ADR-0263 trace_id/span_id missing during reviewer | reject observability emission and fail check | shared observability client fixture; oya-governance-webhook-delivery-log-monotonic records the check | EVT-FOUNDRY-AGENT-ROLE-ROUTED |
| unsafe_path_overlap-1 | concurrent-safe-paths predicate denies during verifier | park or refuse later changeset | safe path registry requires explicit annotation; oya-governance-changeset-cost-budget-monthly records the check | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED |
| ci_red-1 | required status check fails during merge_queue | route to fix-loop if budget remains | completion gate blocks auto-merge; oya-governance-override-justification records the check | EVT-FOUNDRY-AGENT-EXECUTION-STARTED |
| review_reject-1 | reviewer-agent REQUEST CHANGES during human_override | return to work state | review evidence must list resolved items; oya-governance-override-frequency-alarming records the check | EVT-FOUNDRY-AGENT-REVIEW-RECORDED |
| stale_projection-1 | projected base differs from tested base during terminal | rerun projected-state CI | merge queue invalidates positions >= i; oya-governance-audit-event-emission records the check | EVT-FOUNDRY-HUMAN-OVERRIDE-USED |
| missing_evidence-2 | evidence bundle or multispectrum file absent during intake | hold current state and request fix | admission refuses until evidence hash exists; oya-governance-doc-catalog records the check | EVT-FOUNDRY-AGENT-ROLE-ROUTED |
| cedar_deny-2 | policy evaluation denies actor/action/resource during planner | refuse transition and expose denial reason | add regression fixture for the denied scenario; oya-governance-glossary records the check | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED |
| signature_invalid-2 | Ed25519/HMAC signature mismatch during claude_opus | drop event before state mutation | rotate secret/key and run signature replay tests; oya-governance-changeset-state-monotonicity records the check | EVT-FOUNDRY-AGENT-EXECUTION-STARTED |
| idempotency_collision-2 | same dedup key maps to different payload during codex | quarantine event and require human review | dedup log monotonic lane fails; oya-governance-changeset-state-enum-closed records the check | EVT-FOUNDRY-AGENT-REVIEW-RECORDED |
| budget_exhausted-2 | cost budget counter reaches zero during executor | transition to cost_exhausted | monthly budget lane alerts owning team; oya-governance-merge-queue-ref-hygiene records the check | EVT-FOUNDRY-HUMAN-OVERRIDE-USED |
| trace_missing-2 | ADR-0263 trace_id/span_id missing during reviewer | reject observability emission and fail check | shared observability client fixture; oya-governance-event-router-completeness records the check | EVT-FOUNDRY-AGENT-ROLE-ROUTED |
| unsafe_path_overlap-2 | concurrent-safe-paths predicate denies during verifier | park or refuse later changeset | safe path registry requires explicit annotation; oya-governance-webhook-stuck records the check | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED |
| ci_red-2 | required status check fails during merge_queue | route to fix-loop if budget remains | completion gate blocks auto-merge; oya-governance-webhook-delivery-log-monotonic records the check | EVT-FOUNDRY-AGENT-EXECUTION-STARTED |
| review_reject-2 | reviewer-agent REQUEST CHANGES during human_override | return to work state | review evidence must list resolved items; oya-governance-changeset-cost-budget-monthly records the check | EVT-FOUNDRY-AGENT-REVIEW-RECORDED |
| stale_projection-2 | projected base differs from tested base during terminal | rerun projected-state CI | merge queue invalidates positions >= i; oya-governance-override-justification records the check | EVT-FOUNDRY-HUMAN-OVERRIDE-USED |
| missing_evidence-3 | evidence bundle or multispectrum file absent during intake | hold current state and request fix | admission refuses until evidence hash exists; oya-governance-override-frequency-alarming records the check | EVT-FOUNDRY-AGENT-ROLE-ROUTED |
| cedar_deny-3 | policy evaluation denies actor/action/resource during planner | refuse transition and expose denial reason | add regression fixture for the denied scenario; oya-governance-audit-event-emission records the check | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED |
| signature_invalid-3 | Ed25519/HMAC signature mismatch during claude_opus | drop event before state mutation | rotate secret/key and run signature replay tests; oya-governance-doc-catalog records the check | EVT-FOUNDRY-AGENT-EXECUTION-STARTED |
| idempotency_collision-3 | same dedup key maps to different payload during codex | quarantine event and require human review | dedup log monotonic lane fails; oya-governance-glossary records the check | EVT-FOUNDRY-AGENT-REVIEW-RECORDED |
| budget_exhausted-3 | cost budget counter reaches zero during executor | transition to cost_exhausted | monthly budget lane alerts owning team; oya-governance-changeset-state-monotonicity records the check | EVT-FOUNDRY-HUMAN-OVERRIDE-USED |
| trace_missing-3 | ADR-0263 trace_id/span_id missing during reviewer | reject observability emission and fail check | shared observability client fixture; oya-governance-changeset-state-enum-closed records the check | EVT-FOUNDRY-AGENT-ROLE-ROUTED |
| unsafe_path_overlap-3 | concurrent-safe-paths predicate denies during verifier | park or refuse later changeset | safe path registry requires explicit annotation; oya-governance-merge-queue-ref-hygiene records the check | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED |
| ci_red-3 | required status check fails during merge_queue | route to fix-loop if budget remains | completion gate blocks auto-merge; oya-governance-event-router-completeness records the check | EVT-FOUNDRY-AGENT-EXECUTION-STARTED |
| review_reject-3 | reviewer-agent REQUEST CHANGES during human_override | return to work state | review evidence must list resolved items; oya-governance-webhook-stuck records the check | EVT-FOUNDRY-AGENT-REVIEW-RECORDED |
| stale_projection-3 | projected base differs from tested base during terminal | rerun projected-state CI | merge queue invalidates positions >= i; oya-governance-webhook-delivery-log-monotonic records the check | EVT-FOUNDRY-HUMAN-OVERRIDE-USED |
| missing_evidence-4 | evidence bundle or multispectrum file absent during intake | hold current state and request fix | admission refuses until evidence hash exists; oya-governance-changeset-cost-budget-monthly records the check | EVT-FOUNDRY-AGENT-ROLE-ROUTED |
| cedar_deny-4 | policy evaluation denies actor/action/resource during planner | refuse transition and expose denial reason | add regression fixture for the denied scenario; oya-governance-override-justification records the check | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED |
| signature_invalid-4 | Ed25519/HMAC signature mismatch during claude_opus | drop event before state mutation | rotate secret/key and run signature replay tests; oya-governance-override-frequency-alarming records the check | EVT-FOUNDRY-AGENT-EXECUTION-STARTED |
| idempotency_collision-4 | same dedup key maps to different payload during codex | quarantine event and require human review | dedup log monotonic lane fails; oya-governance-audit-event-emission records the check | EVT-FOUNDRY-AGENT-REVIEW-RECORDED |
| budget_exhausted-4 | cost budget counter reaches zero during executor | transition to cost_exhausted | monthly budget lane alerts owning team; oya-governance-doc-catalog records the check | EVT-FOUNDRY-HUMAN-OVERRIDE-USED |
| trace_missing-4 | ADR-0263 trace_id/span_id missing during reviewer | reject observability emission and fail check | shared observability client fixture; oya-governance-glossary records the check | EVT-FOUNDRY-AGENT-ROLE-ROUTED |
| unsafe_path_overlap-4 | concurrent-safe-paths predicate denies during verifier | park or refuse later changeset | safe path registry requires explicit annotation; oya-governance-changeset-state-monotonicity records the check | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED |
| ci_red-4 | required status check fails during merge_queue | route to fix-loop if budget remains | completion gate blocks auto-merge; oya-governance-changeset-state-enum-closed records the check | EVT-FOUNDRY-AGENT-EXECUTION-STARTED |
| review_reject-4 | reviewer-agent REQUEST CHANGES during human_override | return to work state | review evidence must list resolved items; oya-governance-merge-queue-ref-hygiene records the check | EVT-FOUNDRY-AGENT-REVIEW-RECORDED |
| stale_projection-4 | projected base differs from tested base during terminal | rerun projected-state CI | merge queue invalidates positions >= i; oya-governance-event-router-completeness records the check | EVT-FOUNDRY-HUMAN-OVERRIDE-USED |
| missing_evidence-5 | evidence bundle or multispectrum file absent during intake | hold current state and request fix | admission refuses until evidence hash exists; oya-governance-webhook-stuck records the check | EVT-FOUNDRY-AGENT-ROLE-ROUTED |
| cedar_deny-5 | policy evaluation denies actor/action/resource during planner | refuse transition and expose denial reason | add regression fixture for the denied scenario; oya-governance-webhook-delivery-log-monotonic records the check | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED |
| signature_invalid-5 | Ed25519/HMAC signature mismatch during claude_opus | drop event before state mutation | rotate secret/key and run signature replay tests; oya-governance-changeset-cost-budget-monthly records the check | EVT-FOUNDRY-AGENT-EXECUTION-STARTED |
| idempotency_collision-5 | same dedup key maps to different payload during codex | quarantine event and require human review | dedup log monotonic lane fails; oya-governance-override-justification records the check | EVT-FOUNDRY-AGENT-REVIEW-RECORDED |
| budget_exhausted-5 | cost budget counter reaches zero during executor | transition to cost_exhausted | monthly budget lane alerts owning team; oya-governance-override-frequency-alarming records the check | EVT-FOUNDRY-HUMAN-OVERRIDE-USED |
| trace_missing-5 | ADR-0263 trace_id/span_id missing during reviewer | reject observability emission and fail check | shared observability client fixture; oya-governance-audit-event-emission records the check | EVT-FOUNDRY-AGENT-ROLE-ROUTED |
| unsafe_path_overlap-5 | concurrent-safe-paths predicate denies during verifier | park or refuse later changeset | safe path registry requires explicit annotation; oya-governance-doc-catalog records the check | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED |
| ci_red-5 | required status check fails during merge_queue | route to fix-loop if budget remains | completion gate blocks auto-merge; oya-governance-glossary records the check | EVT-FOUNDRY-AGENT-EXECUTION-STARTED |
| review_reject-5 | reviewer-agent REQUEST CHANGES during human_override | return to work state | review evidence must list resolved items; oya-governance-changeset-state-monotonicity records the check | EVT-FOUNDRY-AGENT-REVIEW-RECORDED |
| stale_projection-5 | projected base differs from tested base during terminal | rerun projected-state CI | merge queue invalidates positions >= i; oya-governance-changeset-state-enum-closed records the check | EVT-FOUNDRY-HUMAN-OVERRIDE-USED |
| missing_evidence-6 | evidence bundle or multispectrum file absent during intake | hold current state and request fix | admission refuses until evidence hash exists; oya-governance-merge-queue-ref-hygiene records the check | EVT-FOUNDRY-AGENT-ROLE-ROUTED |
| cedar_deny-6 | policy evaluation denies actor/action/resource during planner | refuse transition and expose denial reason | add regression fixture for the denied scenario; oya-governance-event-router-completeness records the check | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED |
| signature_invalid-6 | Ed25519/HMAC signature mismatch during claude_opus | drop event before state mutation | rotate secret/key and run signature replay tests; oya-governance-webhook-stuck records the check | EVT-FOUNDRY-AGENT-EXECUTION-STARTED |
| idempotency_collision-6 | same dedup key maps to different payload during codex | quarantine event and require human review | dedup log monotonic lane fails; oya-governance-webhook-delivery-log-monotonic records the check | EVT-FOUNDRY-AGENT-REVIEW-RECORDED |
| budget_exhausted-6 | cost budget counter reaches zero during executor | transition to cost_exhausted | monthly budget lane alerts owning team; oya-governance-changeset-cost-budget-monthly records the check | EVT-FOUNDRY-HUMAN-OVERRIDE-USED |
| trace_missing-6 | ADR-0263 trace_id/span_id missing during reviewer | reject observability emission and fail check | shared observability client fixture; oya-governance-override-justification records the check | EVT-FOUNDRY-AGENT-ROLE-ROUTED |
| unsafe_path_overlap-6 | concurrent-safe-paths predicate denies during verifier | park or refuse later changeset | safe path registry requires explicit annotation; oya-governance-override-frequency-alarming records the check | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED |
| ci_red-6 | required status check fails during merge_queue | route to fix-loop if budget remains | completion gate blocks auto-merge; oya-governance-audit-event-emission records the check | EVT-FOUNDRY-AGENT-EXECUTION-STARTED |
| review_reject-6 | reviewer-agent REQUEST CHANGES during human_override | return to work state | review evidence must list resolved items; oya-governance-doc-catalog records the check | EVT-FOUNDRY-AGENT-REVIEW-RECORDED |
| stale_projection-6 | projected base differs from tested base during terminal | rerun projected-state CI | merge queue invalidates positions >= i; oya-governance-glossary records the check | EVT-FOUNDRY-HUMAN-OVERRIDE-USED |
| missing_evidence-7 | evidence bundle or multispectrum file absent during intake | hold current state and request fix | admission refuses until evidence hash exists; oya-governance-changeset-state-monotonicity records the check | EVT-FOUNDRY-AGENT-ROLE-ROUTED |
| cedar_deny-7 | policy evaluation denies actor/action/resource during planner | refuse transition and expose denial reason | add regression fixture for the denied scenario; oya-governance-changeset-state-enum-closed records the check | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED |
| signature_invalid-7 | Ed25519/HMAC signature mismatch during claude_opus | drop event before state mutation | rotate secret/key and run signature replay tests; oya-governance-merge-queue-ref-hygiene records the check | EVT-FOUNDRY-AGENT-EXECUTION-STARTED |
| idempotency_collision-7 | same dedup key maps to different payload during codex | quarantine event and require human review | dedup log monotonic lane fails; oya-governance-event-router-completeness records the check | EVT-FOUNDRY-AGENT-REVIEW-RECORDED |
| budget_exhausted-7 | cost budget counter reaches zero during executor | transition to cost_exhausted | monthly budget lane alerts owning team; oya-governance-webhook-stuck records the check | EVT-FOUNDRY-HUMAN-OVERRIDE-USED |
| trace_missing-7 | ADR-0263 trace_id/span_id missing during reviewer | reject observability emission and fail check | shared observability client fixture; oya-governance-webhook-delivery-log-monotonic records the check | EVT-FOUNDRY-AGENT-ROLE-ROUTED |
| unsafe_path_overlap-7 | concurrent-safe-paths predicate denies during verifier | park or refuse later changeset | safe path registry requires explicit annotation; oya-governance-changeset-cost-budget-monthly records the check | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED |
| ci_red-7 | required status check fails during merge_queue | route to fix-loop if budget remains | completion gate blocks auto-merge; oya-governance-override-justification records the check | EVT-FOUNDRY-AGENT-EXECUTION-STARTED |
| review_reject-7 | reviewer-agent REQUEST CHANGES during human_override | return to work state | review evidence must list resolved items; oya-governance-override-frequency-alarming records the check | EVT-FOUNDRY-AGENT-REVIEW-RECORDED |
| stale_projection-7 | projected base differs from tested base during terminal | rerun projected-state CI | merge queue invalidates positions >= i; oya-governance-audit-event-emission records the check | EVT-FOUNDRY-HUMAN-OVERRIDE-USED |
| missing_evidence-8 | evidence bundle or multispectrum file absent during intake | hold current state and request fix | admission refuses until evidence hash exists; oya-governance-doc-catalog records the check | EVT-FOUNDRY-AGENT-ROLE-ROUTED |
| cedar_deny-8 | policy evaluation denies actor/action/resource during planner | refuse transition and expose denial reason | add regression fixture for the denied scenario; oya-governance-glossary records the check | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED |
| signature_invalid-8 | Ed25519/HMAC signature mismatch during claude_opus | drop event before state mutation | rotate secret/key and run signature replay tests; oya-governance-changeset-state-monotonicity records the check | EVT-FOUNDRY-AGENT-EXECUTION-STARTED |
| idempotency_collision-8 | same dedup key maps to different payload during codex | quarantine event and require human review | dedup log monotonic lane fails; oya-governance-changeset-state-enum-closed records the check | EVT-FOUNDRY-AGENT-REVIEW-RECORDED |
| budget_exhausted-8 | cost budget counter reaches zero during executor | transition to cost_exhausted | monthly budget lane alerts owning team; oya-governance-merge-queue-ref-hygiene records the check | EVT-FOUNDRY-HUMAN-OVERRIDE-USED |
| trace_missing-8 | ADR-0263 trace_id/span_id missing during reviewer | reject observability emission and fail check | shared observability client fixture; oya-governance-event-router-completeness records the check | EVT-FOUNDRY-AGENT-ROLE-ROUTED |
| unsafe_path_overlap-8 | concurrent-safe-paths predicate denies during verifier | park or refuse later changeset | safe path registry requires explicit annotation; oya-governance-webhook-stuck records the check | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED |
| ci_red-8 | required status check fails during merge_queue | route to fix-loop if budget remains | completion gate blocks auto-merge; oya-governance-webhook-delivery-log-monotonic records the check | EVT-FOUNDRY-AGENT-EXECUTION-STARTED |
| review_reject-8 | reviewer-agent REQUEST CHANGES during human_override | return to work state | review evidence must list resolved items; oya-governance-changeset-cost-budget-monthly records the check | EVT-FOUNDRY-AGENT-REVIEW-RECORDED |
| stale_projection-8 | projected base differs from tested base during terminal | rerun projected-state CI | merge queue invalidates positions >= i; oya-governance-override-justification records the check | EVT-FOUNDRY-HUMAN-OVERRIDE-USED |
| missing_evidence-9 | evidence bundle or multispectrum file absent during intake | hold current state and request fix | admission refuses until evidence hash exists; oya-governance-override-frequency-alarming records the check | EVT-FOUNDRY-AGENT-ROLE-ROUTED |
| cedar_deny-9 | policy evaluation denies actor/action/resource during planner | refuse transition and expose denial reason | add regression fixture for the denied scenario; oya-governance-audit-event-emission records the check | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED |
| signature_invalid-9 | Ed25519/HMAC signature mismatch during claude_opus | drop event before state mutation | rotate secret/key and run signature replay tests; oya-governance-doc-catalog records the check | EVT-FOUNDRY-AGENT-EXECUTION-STARTED |
| idempotency_collision-9 | same dedup key maps to different payload during codex | quarantine event and require human review | dedup log monotonic lane fails; oya-governance-glossary records the check | EVT-FOUNDRY-AGENT-REVIEW-RECORDED |
| budget_exhausted-9 | cost budget counter reaches zero during executor | transition to cost_exhausted | monthly budget lane alerts owning team; oya-governance-changeset-state-monotonicity records the check | EVT-FOUNDRY-HUMAN-OVERRIDE-USED |
| trace_missing-9 | ADR-0263 trace_id/span_id missing during reviewer | reject observability emission and fail check | shared observability client fixture; oya-governance-changeset-state-enum-closed records the check | EVT-FOUNDRY-AGENT-ROLE-ROUTED |
| unsafe_path_overlap-9 | concurrent-safe-paths predicate denies during verifier | park or refuse later changeset | safe path registry requires explicit annotation; oya-governance-merge-queue-ref-hygiene records the check | EVT-FOUNDRY-AGENT-HANDOFF-RECORDED |
| ci_red-9 | required status check fails during merge_queue | route to fix-loop if budget remains | completion gate blocks auto-merge; oya-governance-event-router-completeness records the check | EVT-FOUNDRY-AGENT-EXECUTION-STARTED |
| review_reject-9 | reviewer-agent REQUEST CHANGES during human_override | return to work state | review evidence must list resolved items; oya-governance-webhook-stuck records the check | EVT-FOUNDRY-AGENT-REVIEW-RECORDED |
| stale_projection-9 | projected base differs from tested base during terminal | rerun projected-state CI | merge queue invalidates positions >= i; oya-governance-webhook-delivery-log-monotonic records the check | EVT-FOUNDRY-HUMAN-OVERRIDE-USED |

## Worked Examples

### Example 1: Planner decomposes a docs substrate slice and Codex authors it.

1. Intake: Planner decomposes a docs substrate slice and Codex authors it. starts with a bounded Foundry changeset and a stable changeset_id.
2. Actor: Principal::"oyatie.agent.codex" invokes foundry.agent.plan.
3. Resource: Resource::"agent-role:planner" is the primary target.
4. Policy: Cedar evaluates tenant_id=oyatie, workflow=foundry_pipeline, related_adr=ADR-0221, and evidence_hash presence.
5. State: The active phase is intake; no downstream phase observes partial state.
6. Evidence: The evidence bundle stores the command, stdout summary, exit code, trace_id, and audit_id.
7. Audit: EVT-FOUNDRY-AGENT-ROLE-ROUTED seals the state-changing fact before observability emission finalizes.
8. Recovery: If the action fails, the changeset remains at intake and the denial reason is appended.
9. Verification: oya-foundry-vcs-changeset-state-kernel and oya-governance-changeset-state-monotonicity cover this branch.
10. Stop condition: the example is complete only when replay from append-only logs reproduces the same decision.

- Example 1.01: intake_route moves request to intake only after workflow event or human task arrives, with EVT-FOUNDRY-AGENT-ROLE-ROUTED emitted and Cedar denial staying terminal for that attempt.
- Example 1.02: plan_slice moves intake to planner only after planner decomposes and names gates, with EVT-FOUNDRY-AGENT-HANDOFF-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 1.03: deep_reason moves planner to claude_opus only after complex design or adversarial review required, with EVT-FOUNDRY-AGENT-EXECUTION-STARTED emitted and Cedar denial staying terminal for that attempt.
- Example 1.04: code_edit moves planner to codex only after repo-local implementation required, with EVT-FOUNDRY-AGENT-REVIEW-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 1.05: execute_slice moves codex to executor only after bounded implementation lane runs, with EVT-FOUNDRY-HUMAN-OVERRIDE-USED emitted and Cedar denial staying terminal for that attempt.
- Example 1.06: review_slice moves executor to reviewer only after change class reviewer evaluates, with EVT-FOUNDRY-AGENT-ROLE-ROUTED emitted and Cedar denial staying terminal for that attempt.
- Example 1.07: verify_claim moves reviewer to verifier only after evidence and checks validated, with EVT-FOUNDRY-AGENT-HANDOFF-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 1.08: queue_release moves verifier to merge_queue only after ready changeset enters queue, with EVT-FOUNDRY-AGENT-EXECUTION-STARTED emitted and Cedar denial staying terminal for that attempt.
- Example 1.09: override moves reviewer to human_override only after signed human override handles rare mismatch, with EVT-FOUNDRY-AGENT-REVIEW-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 1.10: finish moves merge_queue to terminal only after produced or terminal fail emitted, with EVT-FOUNDRY-HUMAN-OVERRIDE-USED emitted and Cedar denial staying terminal for that attempt.
- Example 1.11: intake_route moves request to intake only after workflow event or human task arrives, with EVT-FOUNDRY-AGENT-ROLE-ROUTED emitted and Cedar denial staying terminal for that attempt.
- Example 1.12: plan_slice moves intake to planner only after planner decomposes and names gates, with EVT-FOUNDRY-AGENT-HANDOFF-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 1.13: deep_reason moves planner to claude_opus only after complex design or adversarial review required, with EVT-FOUNDRY-AGENT-EXECUTION-STARTED emitted and Cedar denial staying terminal for that attempt.
- Example 1.14: code_edit moves planner to codex only after repo-local implementation required, with EVT-FOUNDRY-AGENT-REVIEW-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 1.15: execute_slice moves codex to executor only after bounded implementation lane runs, with EVT-FOUNDRY-HUMAN-OVERRIDE-USED emitted and Cedar denial staying terminal for that attempt.
- Example 1.16: review_slice moves executor to reviewer only after change class reviewer evaluates, with EVT-FOUNDRY-AGENT-ROLE-ROUTED emitted and Cedar denial staying terminal for that attempt.
- Example 1.17: verify_claim moves reviewer to verifier only after evidence and checks validated, with EVT-FOUNDRY-AGENT-HANDOFF-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 1.18: queue_release moves verifier to merge_queue only after ready changeset enters queue, with EVT-FOUNDRY-AGENT-EXECUTION-STARTED emitted and Cedar denial staying terminal for that attempt.

### Example 2: Claude Opus performs adversarial architecture review for a broad change.

1. Intake: Claude Opus performs adversarial architecture review for a broad change. starts with a bounded Foundry changeset and a stable changeset_id.
2. Actor: Principal::"oyatie.agent.claude-opus" invokes foundry.agent.execute.
3. Resource: Resource::"agent-role:executor" is the primary target.
4. Policy: Cedar evaluates tenant_id=oyatie, workflow=foundry_pipeline, related_adr=ADR-0221, and evidence_hash presence.
5. State: The active phase is planner; no downstream phase observes partial state.
6. Evidence: The evidence bundle stores the command, stdout summary, exit code, trace_id, and audit_id.
7. Audit: EVT-FOUNDRY-AGENT-HANDOFF-RECORDED seals the state-changing fact before observability emission finalizes.
8. Recovery: If the action fails, the changeset remains at planner and the denial reason is appended.
9. Verification: oya-foundry-vcs-changeset-state-app and oya-governance-changeset-state-enum-closed cover this branch.
10. Stop condition: the example is complete only when replay from append-only logs reproduces the same decision.

- Example 2.01: plan_slice moves intake to planner only after planner decomposes and names gates, with EVT-FOUNDRY-AGENT-HANDOFF-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 2.02: deep_reason moves planner to claude_opus only after complex design or adversarial review required, with EVT-FOUNDRY-AGENT-EXECUTION-STARTED emitted and Cedar denial staying terminal for that attempt.
- Example 2.03: code_edit moves planner to codex only after repo-local implementation required, with EVT-FOUNDRY-AGENT-REVIEW-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 2.04: execute_slice moves codex to executor only after bounded implementation lane runs, with EVT-FOUNDRY-HUMAN-OVERRIDE-USED emitted and Cedar denial staying terminal for that attempt.
- Example 2.05: review_slice moves executor to reviewer only after change class reviewer evaluates, with EVT-FOUNDRY-AGENT-ROLE-ROUTED emitted and Cedar denial staying terminal for that attempt.
- Example 2.06: verify_claim moves reviewer to verifier only after evidence and checks validated, with EVT-FOUNDRY-AGENT-HANDOFF-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 2.07: queue_release moves verifier to merge_queue only after ready changeset enters queue, with EVT-FOUNDRY-AGENT-EXECUTION-STARTED emitted and Cedar denial staying terminal for that attempt.
- Example 2.08: override moves reviewer to human_override only after signed human override handles rare mismatch, with EVT-FOUNDRY-AGENT-REVIEW-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 2.09: finish moves merge_queue to terminal only after produced or terminal fail emitted, with EVT-FOUNDRY-HUMAN-OVERRIDE-USED emitted and Cedar denial staying terminal for that attempt.
- Example 2.10: intake_route moves request to intake only after workflow event or human task arrives, with EVT-FOUNDRY-AGENT-ROLE-ROUTED emitted and Cedar denial staying terminal for that attempt.
- Example 2.11: plan_slice moves intake to planner only after planner decomposes and names gates, with EVT-FOUNDRY-AGENT-HANDOFF-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 2.12: deep_reason moves planner to claude_opus only after complex design or adversarial review required, with EVT-FOUNDRY-AGENT-EXECUTION-STARTED emitted and Cedar denial staying terminal for that attempt.
- Example 2.13: code_edit moves planner to codex only after repo-local implementation required, with EVT-FOUNDRY-AGENT-REVIEW-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 2.14: execute_slice moves codex to executor only after bounded implementation lane runs, with EVT-FOUNDRY-HUMAN-OVERRIDE-USED emitted and Cedar denial staying terminal for that attempt.
- Example 2.15: review_slice moves executor to reviewer only after change class reviewer evaluates, with EVT-FOUNDRY-AGENT-ROLE-ROUTED emitted and Cedar denial staying terminal for that attempt.
- Example 2.16: verify_claim moves reviewer to verifier only after evidence and checks validated, with EVT-FOUNDRY-AGENT-HANDOFF-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 2.17: queue_release moves verifier to merge_queue only after ready changeset enters queue, with EVT-FOUNDRY-AGENT-EXECUTION-STARTED emitted and Cedar denial staying terminal for that attempt.
- Example 2.18: override moves reviewer to human_override only after signed human override handles rare mismatch, with EVT-FOUNDRY-AGENT-REVIEW-RECORDED emitted and Cedar denial staying terminal for that attempt.

### Example 3: Executor owns bounded files and verifier owns final evidence.

1. Intake: Executor owns bounded files and verifier owns final evidence. starts with a bounded Foundry changeset and a stable changeset_id.
2. Actor: Principal::"oyatie.agent.planner" invokes foundry.agent.review.
3. Resource: Resource::"agent-role:claude-opus" is the primary target.
4. Policy: Cedar evaluates tenant_id=oyatie, workflow=foundry_pipeline, related_adr=ADR-0221, and evidence_hash presence.
5. State: The active phase is claude_opus; no downstream phase observes partial state.
6. Evidence: The evidence bundle stores the command, stdout summary, exit code, trace_id, and audit_id.
7. Audit: EVT-FOUNDRY-AGENT-EXECUTION-STARTED seals the state-changing fact before observability emission finalizes.
8. Recovery: If the action fails, the changeset remains at claude_opus and the denial reason is appended.
9. Verification: oya-foundry-vcs-merge-queue-conflict-kernel and oya-governance-merge-queue-ref-hygiene cover this branch.
10. Stop condition: the example is complete only when replay from append-only logs reproduces the same decision.

- Example 3.01: deep_reason moves planner to claude_opus only after complex design or adversarial review required, with EVT-FOUNDRY-AGENT-EXECUTION-STARTED emitted and Cedar denial staying terminal for that attempt.
- Example 3.02: code_edit moves planner to codex only after repo-local implementation required, with EVT-FOUNDRY-AGENT-REVIEW-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 3.03: execute_slice moves codex to executor only after bounded implementation lane runs, with EVT-FOUNDRY-HUMAN-OVERRIDE-USED emitted and Cedar denial staying terminal for that attempt.
- Example 3.04: review_slice moves executor to reviewer only after change class reviewer evaluates, with EVT-FOUNDRY-AGENT-ROLE-ROUTED emitted and Cedar denial staying terminal for that attempt.
- Example 3.05: verify_claim moves reviewer to verifier only after evidence and checks validated, with EVT-FOUNDRY-AGENT-HANDOFF-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 3.06: queue_release moves verifier to merge_queue only after ready changeset enters queue, with EVT-FOUNDRY-AGENT-EXECUTION-STARTED emitted and Cedar denial staying terminal for that attempt.
- Example 3.07: override moves reviewer to human_override only after signed human override handles rare mismatch, with EVT-FOUNDRY-AGENT-REVIEW-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 3.08: finish moves merge_queue to terminal only after produced or terminal fail emitted, with EVT-FOUNDRY-HUMAN-OVERRIDE-USED emitted and Cedar denial staying terminal for that attempt.
- Example 3.09: intake_route moves request to intake only after workflow event or human task arrives, with EVT-FOUNDRY-AGENT-ROLE-ROUTED emitted and Cedar denial staying terminal for that attempt.
- Example 3.10: plan_slice moves intake to planner only after planner decomposes and names gates, with EVT-FOUNDRY-AGENT-HANDOFF-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 3.11: deep_reason moves planner to claude_opus only after complex design or adversarial review required, with EVT-FOUNDRY-AGENT-EXECUTION-STARTED emitted and Cedar denial staying terminal for that attempt.
- Example 3.12: code_edit moves planner to codex only after repo-local implementation required, with EVT-FOUNDRY-AGENT-REVIEW-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 3.13: execute_slice moves codex to executor only after bounded implementation lane runs, with EVT-FOUNDRY-HUMAN-OVERRIDE-USED emitted and Cedar denial staying terminal for that attempt.
- Example 3.14: review_slice moves executor to reviewer only after change class reviewer evaluates, with EVT-FOUNDRY-AGENT-ROLE-ROUTED emitted and Cedar denial staying terminal for that attempt.
- Example 3.15: verify_claim moves reviewer to verifier only after evidence and checks validated, with EVT-FOUNDRY-AGENT-HANDOFF-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 3.16: queue_release moves verifier to merge_queue only after ready changeset enters queue, with EVT-FOUNDRY-AGENT-EXECUTION-STARTED emitted and Cedar denial staying terminal for that attempt.
- Example 3.17: override moves reviewer to human_override only after signed human override handles rare mismatch, with EVT-FOUNDRY-AGENT-REVIEW-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 3.18: finish moves merge_queue to terminal only after produced or terminal fail emitted, with EVT-FOUNDRY-HUMAN-OVERRIDE-USED emitted and Cedar denial staying terminal for that attempt.

### Example 4: Reviewer-agent requests changes and Codex returns to implementation.

1. Intake: Reviewer-agent requests changes and Codex returns to implementation. starts with a bounded Foundry changeset and a stable changeset_id.
2. Actor: Principal::"oyatie.agent.executor" invokes foundry.agent.verify.
3. Resource: Resource::"agent-role:codex" is the primary target.
4. Policy: Cedar evaluates tenant_id=oyatie, workflow=foundry_pipeline, related_adr=ADR-0221, and evidence_hash presence.
5. State: The active phase is codex; no downstream phase observes partial state.
6. Evidence: The evidence bundle stores the command, stdout summary, exit code, trace_id, and audit_id.
7. Audit: EVT-FOUNDRY-AGENT-REVIEW-RECORDED seals the state-changing fact before observability emission finalizes.
8. Recovery: If the action fails, the changeset remains at codex and the denial reason is appended.
9. Verification: oya-foundry-vcs-review-mergequeue-kernel and oya-governance-event-router-completeness cover this branch.
10. Stop condition: the example is complete only when replay from append-only logs reproduces the same decision.

- Example 4.01: code_edit moves planner to codex only after repo-local implementation required, with EVT-FOUNDRY-AGENT-REVIEW-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 4.02: execute_slice moves codex to executor only after bounded implementation lane runs, with EVT-FOUNDRY-HUMAN-OVERRIDE-USED emitted and Cedar denial staying terminal for that attempt.
- Example 4.03: review_slice moves executor to reviewer only after change class reviewer evaluates, with EVT-FOUNDRY-AGENT-ROLE-ROUTED emitted and Cedar denial staying terminal for that attempt.
- Example 4.04: verify_claim moves reviewer to verifier only after evidence and checks validated, with EVT-FOUNDRY-AGENT-HANDOFF-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 4.05: queue_release moves verifier to merge_queue only after ready changeset enters queue, with EVT-FOUNDRY-AGENT-EXECUTION-STARTED emitted and Cedar denial staying terminal for that attempt.
- Example 4.06: override moves reviewer to human_override only after signed human override handles rare mismatch, with EVT-FOUNDRY-AGENT-REVIEW-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 4.07: finish moves merge_queue to terminal only after produced or terminal fail emitted, with EVT-FOUNDRY-HUMAN-OVERRIDE-USED emitted and Cedar denial staying terminal for that attempt.
- Example 4.08: intake_route moves request to intake only after workflow event or human task arrives, with EVT-FOUNDRY-AGENT-ROLE-ROUTED emitted and Cedar denial staying terminal for that attempt.
- Example 4.09: plan_slice moves intake to planner only after planner decomposes and names gates, with EVT-FOUNDRY-AGENT-HANDOFF-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 4.10: deep_reason moves planner to claude_opus only after complex design or adversarial review required, with EVT-FOUNDRY-AGENT-EXECUTION-STARTED emitted and Cedar denial staying terminal for that attempt.
- Example 4.11: code_edit moves planner to codex only after repo-local implementation required, with EVT-FOUNDRY-AGENT-REVIEW-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 4.12: execute_slice moves codex to executor only after bounded implementation lane runs, with EVT-FOUNDRY-HUMAN-OVERRIDE-USED emitted and Cedar denial staying terminal for that attempt.
- Example 4.13: review_slice moves executor to reviewer only after change class reviewer evaluates, with EVT-FOUNDRY-AGENT-ROLE-ROUTED emitted and Cedar denial staying terminal for that attempt.
- Example 4.14: verify_claim moves reviewer to verifier only after evidence and checks validated, with EVT-FOUNDRY-AGENT-HANDOFF-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 4.15: queue_release moves verifier to merge_queue only after ready changeset enters queue, with EVT-FOUNDRY-AGENT-EXECUTION-STARTED emitted and Cedar denial staying terminal for that attempt.
- Example 4.16: override moves reviewer to human_override only after signed human override handles rare mismatch, with EVT-FOUNDRY-AGENT-REVIEW-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 4.17: finish moves merge_queue to terminal only after produced or terminal fail emitted, with EVT-FOUNDRY-HUMAN-OVERRIDE-USED emitted and Cedar denial staying terminal for that attempt.
- Example 4.18: intake_route moves request to intake only after workflow event or human task arrives, with EVT-FOUNDRY-AGENT-ROLE-ROUTED emitted and Cedar denial staying terminal for that attempt.

### Example 5: Human override is rare, signed, justified, and alarmed.

1. Intake: Human override is rare, signed, justified, and alarmed. starts with a bounded Foundry changeset and a stable changeset_id.
2. Actor: Principal::"oyatie.service.vcs-orchestrator" invokes foundry.agent.invoke_claude_opus.
3. Resource: Resource::"agent-role:reviewer" is the primary target.
4. Policy: Cedar evaluates tenant_id=oyatie, workflow=foundry_pipeline, related_adr=ADR-0221, and evidence_hash presence.
5. State: The active phase is executor; no downstream phase observes partial state.
6. Evidence: The evidence bundle stores the command, stdout summary, exit code, trace_id, and audit_id.
7. Audit: EVT-FOUNDRY-HUMAN-OVERRIDE-USED seals the state-changing fact before observability emission finalizes.
8. Recovery: If the action fails, the changeset remains at executor and the denial reason is appended.
9. Verification: oya-foundry-webhook-receiver-kernel and oya-governance-webhook-stuck cover this branch.
10. Stop condition: the example is complete only when replay from append-only logs reproduces the same decision.

- Example 5.01: execute_slice moves codex to executor only after bounded implementation lane runs, with EVT-FOUNDRY-HUMAN-OVERRIDE-USED emitted and Cedar denial staying terminal for that attempt.
- Example 5.02: review_slice moves executor to reviewer only after change class reviewer evaluates, with EVT-FOUNDRY-AGENT-ROLE-ROUTED emitted and Cedar denial staying terminal for that attempt.
- Example 5.03: verify_claim moves reviewer to verifier only after evidence and checks validated, with EVT-FOUNDRY-AGENT-HANDOFF-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 5.04: queue_release moves verifier to merge_queue only after ready changeset enters queue, with EVT-FOUNDRY-AGENT-EXECUTION-STARTED emitted and Cedar denial staying terminal for that attempt.
- Example 5.05: override moves reviewer to human_override only after signed human override handles rare mismatch, with EVT-FOUNDRY-AGENT-REVIEW-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 5.06: finish moves merge_queue to terminal only after produced or terminal fail emitted, with EVT-FOUNDRY-HUMAN-OVERRIDE-USED emitted and Cedar denial staying terminal for that attempt.
- Example 5.07: intake_route moves request to intake only after workflow event or human task arrives, with EVT-FOUNDRY-AGENT-ROLE-ROUTED emitted and Cedar denial staying terminal for that attempt.
- Example 5.08: plan_slice moves intake to planner only after planner decomposes and names gates, with EVT-FOUNDRY-AGENT-HANDOFF-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 5.09: deep_reason moves planner to claude_opus only after complex design or adversarial review required, with EVT-FOUNDRY-AGENT-EXECUTION-STARTED emitted and Cedar denial staying terminal for that attempt.
- Example 5.10: code_edit moves planner to codex only after repo-local implementation required, with EVT-FOUNDRY-AGENT-REVIEW-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 5.11: execute_slice moves codex to executor only after bounded implementation lane runs, with EVT-FOUNDRY-HUMAN-OVERRIDE-USED emitted and Cedar denial staying terminal for that attempt.
- Example 5.12: review_slice moves executor to reviewer only after change class reviewer evaluates, with EVT-FOUNDRY-AGENT-ROLE-ROUTED emitted and Cedar denial staying terminal for that attempt.
- Example 5.13: verify_claim moves reviewer to verifier only after evidence and checks validated, with EVT-FOUNDRY-AGENT-HANDOFF-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 5.14: queue_release moves verifier to merge_queue only after ready changeset enters queue, with EVT-FOUNDRY-AGENT-EXECUTION-STARTED emitted and Cedar denial staying terminal for that attempt.
- Example 5.15: override moves reviewer to human_override only after signed human override handles rare mismatch, with EVT-FOUNDRY-AGENT-REVIEW-RECORDED emitted and Cedar denial staying terminal for that attempt.
- Example 5.16: finish moves merge_queue to terminal only after produced or terminal fail emitted, with EVT-FOUNDRY-HUMAN-OVERRIDE-USED emitted and Cedar denial staying terminal for that attempt.
- Example 5.17: intake_route moves request to intake only after workflow event or human task arrives, with EVT-FOUNDRY-AGENT-ROLE-ROUTED emitted and Cedar denial staying terminal for that attempt.
- Example 5.18: plan_slice moves intake to planner only after planner decomposes and names gates, with EVT-FOUNDRY-AGENT-HANDOFF-RECORDED emitted and Cedar denial staying terminal for that attempt.

## Verification

Named checks below are the required evidence vocabulary for CI, local agent verification, and future Oya governance ports.

| Check | Command or lane | Required crate | Claim proved |
|---|---|---|---|
| state monotonicity | oya gate validate changeset-state-monotonicity | oya-governance-changeset-state-monotonicity | event log replay never regresses |
| closed enum | oya gate validate changeset-state-enum-closed | oya-governance-changeset-state-enum-closed | only accepted states appear |
| merge projection | cargo test -p oya-foundry-vcs-merge-queue-conflict-kernel | oya-foundry-vcs-merge-queue-conflict-kernel | projected merge state is deterministic |
| review merge queue | cargo test -p oya-foundry-vcs-review-mergequeue-kernel | oya-foundry-vcs-review-mergequeue-kernel | fairness and parked state work |
| webhook receiver | cargo test -p oya-foundry-webhook-receiver-kernel | oya-foundry-webhook-receiver-kernel | HMAC and dedup paths are valid |
| admission gate | cargo test -p oya-foundry-vcs-admission-gate-kernel | oya-foundry-vcs-admission-gate-kernel | policy and evidence gate refuses bad bundles |
| changebundle | cargo test -p oya-foundry-vcs-changebundle-kernel | oya-foundry-vcs-changebundle-kernel | bundle shape is stable |
| promotion controller | cargo test -p oya-foundry-vcs-promotion-controller-kernel | oya-foundry-vcs-promotion-controller-kernel | environment promotion respects state |
| cli ratchet | cargo test -p oya-foundry-vcs-cli-ratchet-kernel | oya-foundry-vcs-cli-ratchet-kernel | claim/verify/done/promote CLI grammar holds |
| audit emission | cargo test -p oya-governance-audit-event-emission | oya-governance-audit-event-emission | ADR-0263 audit linkage exists |
| doc catalog | oya gate validate doc-catalog | oya-governance-doc-catalog | spec is discoverable and owned |
| glossary | oya gate validate glossary | oya-governance-glossary | Foundry internal vs Intelligence consumer vocabulary is preserved |
| agent-types-and-roles-matrix-01 | oya gate validate changeset-state-monotonicity --scope intake --adr ADR-0221 | oya-governance-changeset-state-monotonicity | proves intake cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-02 | oya gate validate changeset-state-enum-closed --scope planner --adr ADR-0221 | oya-governance-changeset-state-enum-closed | proves planner cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-03 | cargo test -p oya-foundry-vcs-merge-queue-conflict-kernel --scope claude_opus --adr ADR-0221 | oya-foundry-vcs-merge-queue-conflict-kernel | proves claude_opus cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-04 | cargo test -p oya-foundry-vcs-review-mergequeue-kernel --scope codex --adr ADR-0221 | oya-foundry-vcs-review-mergequeue-kernel | proves codex cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-05 | cargo test -p oya-foundry-webhook-receiver-kernel --scope executor --adr ADR-0221 | oya-foundry-webhook-receiver-kernel | proves executor cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-06 | cargo test -p oya-foundry-vcs-admission-gate-kernel --scope reviewer --adr ADR-0221 | oya-foundry-vcs-admission-gate-kernel | proves reviewer cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-07 | cargo test -p oya-foundry-vcs-changebundle-kernel --scope verifier --adr ADR-0221 | oya-foundry-vcs-changebundle-kernel | proves verifier cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-08 | cargo test -p oya-foundry-vcs-promotion-controller-kernel --scope merge_queue --adr ADR-0221 | oya-foundry-vcs-promotion-controller-kernel | proves merge_queue cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-09 | cargo test -p oya-foundry-vcs-cli-ratchet-kernel --scope human_override --adr ADR-0221 | oya-foundry-vcs-cli-ratchet-kernel | proves human_override cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-10 | cargo test -p oya-governance-audit-event-emission --scope terminal --adr ADR-0221 | oya-governance-audit-event-emission | proves terminal cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-11 | oya gate validate doc-catalog --scope intake --adr ADR-0221 | oya-governance-doc-catalog | proves intake cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-12 | oya gate validate glossary --scope planner --adr ADR-0221 | oya-governance-glossary | proves planner cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-13 | oya gate validate changeset-state-monotonicity --scope claude_opus --adr ADR-0221 | oya-governance-changeset-state-monotonicity | proves claude_opus cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-14 | oya gate validate changeset-state-enum-closed --scope codex --adr ADR-0221 | oya-governance-changeset-state-enum-closed | proves codex cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-15 | cargo test -p oya-foundry-vcs-merge-queue-conflict-kernel --scope executor --adr ADR-0221 | oya-foundry-vcs-merge-queue-conflict-kernel | proves executor cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-16 | cargo test -p oya-foundry-vcs-review-mergequeue-kernel --scope reviewer --adr ADR-0221 | oya-foundry-vcs-review-mergequeue-kernel | proves reviewer cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-17 | cargo test -p oya-foundry-webhook-receiver-kernel --scope verifier --adr ADR-0221 | oya-foundry-webhook-receiver-kernel | proves verifier cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-18 | cargo test -p oya-foundry-vcs-admission-gate-kernel --scope merge_queue --adr ADR-0221 | oya-foundry-vcs-admission-gate-kernel | proves merge_queue cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-19 | cargo test -p oya-foundry-vcs-changebundle-kernel --scope human_override --adr ADR-0221 | oya-foundry-vcs-changebundle-kernel | proves human_override cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-20 | cargo test -p oya-foundry-vcs-promotion-controller-kernel --scope terminal --adr ADR-0221 | oya-foundry-vcs-promotion-controller-kernel | proves terminal cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-21 | cargo test -p oya-foundry-vcs-cli-ratchet-kernel --scope intake --adr ADR-0221 | oya-foundry-vcs-cli-ratchet-kernel | proves intake cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-22 | cargo test -p oya-governance-audit-event-emission --scope planner --adr ADR-0221 | oya-governance-audit-event-emission | proves planner cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-23 | oya gate validate doc-catalog --scope claude_opus --adr ADR-0221 | oya-governance-doc-catalog | proves claude_opus cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-24 | oya gate validate glossary --scope codex --adr ADR-0221 | oya-governance-glossary | proves codex cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-25 | oya gate validate changeset-state-monotonicity --scope executor --adr ADR-0221 | oya-governance-changeset-state-monotonicity | proves executor cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-26 | oya gate validate changeset-state-enum-closed --scope reviewer --adr ADR-0221 | oya-governance-changeset-state-enum-closed | proves reviewer cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-27 | cargo test -p oya-foundry-vcs-merge-queue-conflict-kernel --scope verifier --adr ADR-0221 | oya-foundry-vcs-merge-queue-conflict-kernel | proves verifier cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-28 | cargo test -p oya-foundry-vcs-review-mergequeue-kernel --scope merge_queue --adr ADR-0221 | oya-foundry-vcs-review-mergequeue-kernel | proves merge_queue cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-29 | cargo test -p oya-foundry-webhook-receiver-kernel --scope human_override --adr ADR-0221 | oya-foundry-webhook-receiver-kernel | proves human_override cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-30 | cargo test -p oya-foundry-vcs-admission-gate-kernel --scope terminal --adr ADR-0221 | oya-foundry-vcs-admission-gate-kernel | proves terminal cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-31 | cargo test -p oya-foundry-vcs-changebundle-kernel --scope intake --adr ADR-0221 | oya-foundry-vcs-changebundle-kernel | proves intake cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-32 | cargo test -p oya-foundry-vcs-promotion-controller-kernel --scope planner --adr ADR-0221 | oya-foundry-vcs-promotion-controller-kernel | proves planner cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-33 | cargo test -p oya-foundry-vcs-cli-ratchet-kernel --scope claude_opus --adr ADR-0221 | oya-foundry-vcs-cli-ratchet-kernel | proves claude_opus cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-34 | cargo test -p oya-governance-audit-event-emission --scope codex --adr ADR-0221 | oya-governance-audit-event-emission | proves codex cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-35 | oya gate validate doc-catalog --scope executor --adr ADR-0221 | oya-governance-doc-catalog | proves executor cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-36 | oya gate validate glossary --scope reviewer --adr ADR-0221 | oya-governance-glossary | proves reviewer cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-37 | oya gate validate changeset-state-monotonicity --scope verifier --adr ADR-0221 | oya-governance-changeset-state-monotonicity | proves verifier cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-38 | oya gate validate changeset-state-enum-closed --scope merge_queue --adr ADR-0221 | oya-governance-changeset-state-enum-closed | proves merge_queue cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-39 | cargo test -p oya-foundry-vcs-merge-queue-conflict-kernel --scope human_override --adr ADR-0221 | oya-foundry-vcs-merge-queue-conflict-kernel | proves human_override cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-40 | cargo test -p oya-foundry-vcs-review-mergequeue-kernel --scope terminal --adr ADR-0221 | oya-foundry-vcs-review-mergequeue-kernel | proves terminal cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-41 | cargo test -p oya-foundry-webhook-receiver-kernel --scope intake --adr ADR-0221 | oya-foundry-webhook-receiver-kernel | proves intake cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-42 | cargo test -p oya-foundry-vcs-admission-gate-kernel --scope planner --adr ADR-0221 | oya-foundry-vcs-admission-gate-kernel | proves planner cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-43 | cargo test -p oya-foundry-vcs-changebundle-kernel --scope claude_opus --adr ADR-0221 | oya-foundry-vcs-changebundle-kernel | proves claude_opus cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-44 | cargo test -p oya-foundry-vcs-promotion-controller-kernel --scope codex --adr ADR-0221 | oya-foundry-vcs-promotion-controller-kernel | proves codex cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-45 | cargo test -p oya-foundry-vcs-cli-ratchet-kernel --scope executor --adr ADR-0221 | oya-foundry-vcs-cli-ratchet-kernel | proves executor cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-46 | cargo test -p oya-governance-audit-event-emission --scope reviewer --adr ADR-0221 | oya-governance-audit-event-emission | proves reviewer cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-47 | oya gate validate doc-catalog --scope verifier --adr ADR-0221 | oya-governance-doc-catalog | proves verifier cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-48 | oya gate validate glossary --scope merge_queue --adr ADR-0221 | oya-governance-glossary | proves merge_queue cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-49 | oya gate validate changeset-state-monotonicity --scope human_override --adr ADR-0221 | oya-governance-changeset-state-monotonicity | proves human_override cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-50 | oya gate validate changeset-state-enum-closed --scope terminal --adr ADR-0221 | oya-governance-changeset-state-enum-closed | proves terminal cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-51 | cargo test -p oya-foundry-vcs-merge-queue-conflict-kernel --scope intake --adr ADR-0221 | oya-foundry-vcs-merge-queue-conflict-kernel | proves intake cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-52 | cargo test -p oya-foundry-vcs-review-mergequeue-kernel --scope planner --adr ADR-0221 | oya-foundry-vcs-review-mergequeue-kernel | proves planner cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-53 | cargo test -p oya-foundry-webhook-receiver-kernel --scope claude_opus --adr ADR-0221 | oya-foundry-webhook-receiver-kernel | proves claude_opus cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-54 | cargo test -p oya-foundry-vcs-admission-gate-kernel --scope codex --adr ADR-0221 | oya-foundry-vcs-admission-gate-kernel | proves codex cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-55 | cargo test -p oya-foundry-vcs-changebundle-kernel --scope executor --adr ADR-0221 | oya-foundry-vcs-changebundle-kernel | proves executor cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-56 | cargo test -p oya-foundry-vcs-promotion-controller-kernel --scope reviewer --adr ADR-0221 | oya-foundry-vcs-promotion-controller-kernel | proves reviewer cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-57 | cargo test -p oya-foundry-vcs-cli-ratchet-kernel --scope verifier --adr ADR-0221 | oya-foundry-vcs-cli-ratchet-kernel | proves verifier cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-58 | cargo test -p oya-governance-audit-event-emission --scope merge_queue --adr ADR-0221 | oya-governance-audit-event-emission | proves merge_queue cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-59 | oya gate validate doc-catalog --scope human_override --adr ADR-0221 | oya-governance-doc-catalog | proves human_override cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-60 | oya gate validate glossary --scope terminal --adr ADR-0221 | oya-governance-glossary | proves terminal cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-61 | oya gate validate changeset-state-monotonicity --scope intake --adr ADR-0221 | oya-governance-changeset-state-monotonicity | proves intake cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-62 | oya gate validate changeset-state-enum-closed --scope planner --adr ADR-0221 | oya-governance-changeset-state-enum-closed | proves planner cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-63 | cargo test -p oya-foundry-vcs-merge-queue-conflict-kernel --scope claude_opus --adr ADR-0221 | oya-foundry-vcs-merge-queue-conflict-kernel | proves claude_opus cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-64 | cargo test -p oya-foundry-vcs-review-mergequeue-kernel --scope codex --adr ADR-0221 | oya-foundry-vcs-review-mergequeue-kernel | proves codex cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-65 | cargo test -p oya-foundry-webhook-receiver-kernel --scope executor --adr ADR-0221 | oya-foundry-webhook-receiver-kernel | proves executor cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-66 | cargo test -p oya-foundry-vcs-admission-gate-kernel --scope reviewer --adr ADR-0221 | oya-foundry-vcs-admission-gate-kernel | proves reviewer cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-67 | cargo test -p oya-foundry-vcs-changebundle-kernel --scope verifier --adr ADR-0221 | oya-foundry-vcs-changebundle-kernel | proves verifier cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-68 | cargo test -p oya-foundry-vcs-promotion-controller-kernel --scope merge_queue --adr ADR-0221 | oya-foundry-vcs-promotion-controller-kernel | proves merge_queue cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-69 | cargo test -p oya-foundry-vcs-cli-ratchet-kernel --scope human_override --adr ADR-0221 | oya-foundry-vcs-cli-ratchet-kernel | proves human_override cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-70 | cargo test -p oya-governance-audit-event-emission --scope terminal --adr ADR-0221 | oya-governance-audit-event-emission | proves terminal cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-71 | oya gate validate doc-catalog --scope intake --adr ADR-0221 | oya-governance-doc-catalog | proves intake cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-72 | oya gate validate glossary --scope planner --adr ADR-0221 | oya-governance-glossary | proves planner cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-73 | oya gate validate changeset-state-monotonicity --scope claude_opus --adr ADR-0221 | oya-governance-changeset-state-monotonicity | proves claude_opus cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-74 | oya gate validate changeset-state-enum-closed --scope codex --adr ADR-0221 | oya-governance-changeset-state-enum-closed | proves codex cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-75 | cargo test -p oya-foundry-vcs-merge-queue-conflict-kernel --scope executor --adr ADR-0221 | oya-foundry-vcs-merge-queue-conflict-kernel | proves executor cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-76 | cargo test -p oya-foundry-vcs-review-mergequeue-kernel --scope reviewer --adr ADR-0221 | oya-foundry-vcs-review-mergequeue-kernel | proves reviewer cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-77 | cargo test -p oya-foundry-webhook-receiver-kernel --scope verifier --adr ADR-0221 | oya-foundry-webhook-receiver-kernel | proves verifier cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-78 | cargo test -p oya-foundry-vcs-admission-gate-kernel --scope merge_queue --adr ADR-0221 | oya-foundry-vcs-admission-gate-kernel | proves merge_queue cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-79 | cargo test -p oya-foundry-vcs-changebundle-kernel --scope human_override --adr ADR-0221 | oya-foundry-vcs-changebundle-kernel | proves human_override cannot advance without policy, evidence, trace, and audit correlation |
| agent-types-and-roles-matrix-80 | cargo test -p oya-foundry-vcs-promotion-controller-kernel --scope terminal --adr ADR-0221 | oya-foundry-vcs-promotion-controller-kernel | proves terminal cannot advance without policy, evidence, trace, and audit correlation |

## Cross-References

| Reference | Path | Use in this spec |
|---|---|---|
| Foundry single microservice | docs/decisions/ADR-0136-foundry-as-single-microservice.md | Foundry is one microservice with internal bounded contexts. |
| Consumer split | docs/decisions/ADR-0220-consumer-intelligence-substrate.md | Consumer AI belongs to Intelligence, not Foundry. |
| Pipeline hardening | docs/decisions/ADR-0221-agentic-development-pipeline-hardening.md | Pre-dispatch templates, scope locks, and CI gates harden agentic work. |
| Changeset state | docs/decisions/ADR-0110-changeset-state-machine.md | Closed state enum and event-sourced log. |
| Merge queue | docs/decisions/ADR-0111-merge-queue-projected-state-fix-at-any-stage.md | Projected merge state and fix-at-any-stage algorithm. |
| Webhook receiver | docs/decisions/ADR-0112-webhook-driven-foundry-agent-invocation.md | HMAC, dedup, event router, and agent invocation. |
| VCS orchestrator | docs/decisions/ADR-0113-vcs-orchestrator-end-to-end.md | claim to promote lifecycle. |
| Retired tooling | docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md | grit, rtk, icm, and vox are retired. |
| Observability contract | docs/decisions/ADR-0263-observability-emission-contract.md | trace, log, metric, audit_id, and tenant_id requirements. |
| Foundry PRD | microservices/foundry/PRD.md | Product-of-record Foundry surface. |
| Foundry architecture | microservices/foundry/ARCHITECTURE.md | Top-level architecture for the single Foundry microservice. |
| Foundry policy | microservices/foundry/policy/ | Cedar policy and markdown policy corpus. |
| Peer spec | microservices/foundry/spec/changeset-state-machine.md | Cross-checks Foundry Changeset State Machine against Foundry Agent Types and Roles. |
| Peer spec | microservices/foundry/spec/merge-queue-projected-state.md | Cross-checks Foundry Merge Queue Projected State against Foundry Agent Types and Roles. |
| Peer spec | microservices/foundry/spec/webhook-driven-agent-invocation.md | Cross-checks Foundry Webhook Driven Agent Invocation against Foundry Agent Types and Roles. |
| Peer spec | microservices/foundry/spec/vcs-orchestrator-end-to-end.md | Cross-checks Foundry VCS Orchestrator End to End against Foundry Agent Types and Roles. |
| Peer spec | microservices/foundry/spec/agent-pipeline-isolation-worktree.md | Cross-checks Foundry Agent Pipeline Isolation Worktree against Foundry Agent Types and Roles. |
| Peer spec | microservices/foundry/spec/admission-gate-policy-and-evidence.md | Cross-checks Foundry Admission Gate Policy and Evidence against Foundry Agent Types and Roles. |
| Peer spec | microservices/foundry/spec/completion-gate-reviewer-and-ci.md | Cross-checks Foundry Completion Gate Reviewer and CI against Foundry Agent Types and Roles. |
| Peer spec | microservices/foundry/spec/agent-types-and-roles.md | Cross-checks Foundry Agent Types and Roles against Foundry Agent Types and Roles. |

## Implementation Control Ledger

The rows below are intentionally concrete so agents can convert this spec into tests, gates, fixtures, and runbooks without inventing missing control names.

| Row | Phase | Control | Owner | Evidence | Stop condition |
|---|---|---|---|---|---|

