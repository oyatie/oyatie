# ARCHITECT VERDICT (fable lane): SOUND-WITH-AMENDMENTS

Antithesis highlights: A1 leader = non-durable orchestrator (controller state not reconstructable; violates P5 at orchestration layer); A2 worker-embedded review is a pre-filter, not Tricorder (leader pass is the real authority; all real defects this session were caught by fresh-context review, never self-review); A3 one-shot mispriced — old nudges were broken plumbing, not needless steering; premise errors burn full runs; A4 manual merge train serializes away parallelism (FRIC-007; Tide is the answer; rebase choreography already gutted PR #643 once).

Amendments:
1. **Dispatch ledger** (.omc/ultragoal/dispatch-ledger.jsonl, append-only: lane_id, brief, worktree, branch, base, expected_surfaces, status transitions dispatched→pr-open→reviewed→merged|escalated) + reconciliation rule (PR state = ground truth) + new acceptance criterion A5 (fresh leader reconstructs all in-flight state from ledger + gh alone).
2. **Embedded review = mandatory pre-open FILTER, never authority**; verdict written to tracked review-verdict.txt pinned to the PR head SHA; PR without SHA-pinned verdict is rejected at intake, not reviewed.
3. **Premise gate**: worker's FIRST action = verify the brief's premise against current dev and write premise.txt (claim, verification, the one invalidating fact); stop+escalate immediately on failure (converts wrong-premise cost from hours to minutes).
4. **Mechanical disjointness check + concurrency bound K**: expected_surfaces in the ledger; new lane dispatched only if surfaces disjoint from in-flight lanes; colliding lanes sequenced.
5. **Tide-survivable merge contract**: admission contract = "green on projected post-merge state"; manual rebase = the current adapter; explicit cutover assertion (interface unchanged at ADR-0515 Tide).
6. **Sixth escalation trigger** (security-sensitive discovery = immediate stop+escalate) + meta-skill load attestation in final message (R5 spot-checkable).

Hyperscaler verdict: keep one-shot stateless workers, fixed-rubric review, projected-state goal, file contracts; change non-durable leader, serial train, convention disjointness, review-as-latency-win framing. Owned-stack: GitHub coupling adapter-shaped IFF Amendment 5 lands.
