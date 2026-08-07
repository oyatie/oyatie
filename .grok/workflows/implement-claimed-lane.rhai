let meta = #{
    name: "implement-claimed-lane",
    description: "Claim ready board lane; drive mm-pipeline TDD path; open/update PR; dual-critic; heartbeat W2",
    when_to_use: "Always-on fabric W2: drain task-board ready lanes via mechanical mm-* implement path",
    phases: [
        #{ title: "Claim", detail: "pick highest priority ready lane" },
        #{ title: "Implement", detail: "mm-pipeline + TDD until dual-critic APPROVE path ready" },
        #{ title: "Hand off", detail: "board pr field + heartbeat for babysit" },
    ],
};

// W2 Implement — full autonomy under human supervise.
// Path: CAPTURE→…→RED→IMPLEMENT→GREEN→false-green; dual-critic APPROVE or fix until APPROVE.
// Soft reds on board are first-class work. Not merge authority (babysit merges).

phase("Claim");
let claim = agent(
    "Repo /Users/jasonlee/Developer/oyatie. Fabric W2 implementer.\n"
    + "1) python3 .grok/bin/mm-queue-ingest --cwd /Users/jasonlee/Developer/oyatie --base dev\n"
    + "2) Read .grok/programs/delivery-fabric/task-board.v1.json\n"
    + "3) Pick ONE ready lane: prefer source=ci_red, then soft_red, then reorg (W0/W1 ready not blocked), "
    + "then block, then other. Highest priority first (P0>P1>P2>P3).\n"
    + "3b) ALWAYS run python3 .grok/bin/mm-dev-status --cwd /Users/jasonlee/Developer/oyatie --fetch "
    + "and create/use worktree from origin/dev tip when starting new reorg work (stale execute is defect).\n"
    + "4) Mark it status=claimed, claimed_by=implement-claimed-lane, claimed_at=UTC ISO; write board.\n"
    + "5) python3 .grok/bin/mm-fabric-heartbeat --cwd /Users/jasonlee/Developer/oyatie "
    + "--class implement-claimed-lane --note claimed\n"
    + "If no ready lanes: heartbeat with note empty-board and stop (success with empty).\n"
    + "Return JSON: lane_id, pr, source, title, paths.",
    #{ label: "w2-claim", capability_mode: "all", agent_type: "general-purpose" }
);

phase("Implement");
let impl = agent(
    "Repo /Users/jasonlee/Developer/oyatie. Fabric W2 implement the CLAIMED lane from task-board "
    + "(status=claimed, claimed_by=implement-claimed-lane).\n"
    + "PROCESS (mechanical mm-*):\n"
    + "A) If lane has pr number: work in that PR worktree/branch only (fetch tip). "
    + "Else create isolated worktree from origin/dev, branch agent/fabric-<lane_id>.\n"
    + "B) python3 .grok/bin/mm-pipeline start --cwd <worktree_or_repo> --objective \"<lane title>\" "
    + "--risk medium --allow-diverged only if required; prefer clean worktree.\n"
    + "C) TDD: write failing test (RED proof), implement minimal fix, green same suite. "
    + "For soft_red/ci_red: diagnose gh run failed logs for that check and fix root cause.\n"
    + "D) Never hand-edit *.generated.json; use materialize producers.\n"
    + "E0) MANDATORY before push/pr-create: python3 .grok/bin/mm-preflight-pr --base origin/dev\n"
    + "    Exit nonzero → fix local FAILs; do NOT open a red PR. See CI-RED-PROCESS.md.\n"
    + "E) Commit + SSH-signed push to PR branch (or open PR against dev). ONE push after local verify.\n"
    + "F) Dual-critic: write .grok/programs/**/evidence/pr-N-dual-critic.json (local pin preferred until CI green; "
    + "no tip thrash). verdict APPROVE only if earned; independence honest (same_family ok if noted).\n"
    + "G) Board: status=ready_for_babysit, pr=N, branch=..., head_sha=... — HAND OFF to W3. "
    + "FORBIDDEN after push: sleep+gh pr checks loops, merge-on-green arms, multi-worker poll for this PR "
    + "(see BABYSIT-SINGLE-FLIGHT.md). W3 is sole babysit owner.\n"
    + "H) Heartbeat implement-claimed-lane note=handoff-pr-N (not waiting_ci idle).\n"
    + "Autonomy: W3 merges on dual-critic APPROVE + oya-ci-required; human GH APPROVE not required.\n"
    + "Return: pr, head sha, dual_critic path/verdict, handoff=ready_for_babysit.",
    #{ label: "w2-implement", capability_mode: "all", agent_type: "general-purpose", isolation_worktree: false }
);

phase("Hand off");
let handoff = agent(
    "Repo /Users/jasonlee/Developer/oyatie. W2 HANDOFF (BABYSIT-SINGLE-FLIGHT.md).\n"
    + "1) Board: set claimed lane status=ready_for_babysit with pr=N branch=... head_sha=...\n"
    + "2) FORBIDDEN: multi-minute gh pr checks / sleep poll / merge-on-green arm — W3 is sole babysit owner.\n"
    + "3) One-shot merge-check is optional; do NOT loop. If not green, leave for W3.\n"
    + "4) python3 .grok/bin/mm-fabric-heartbeat --cwd /Users/jasonlee/Developer/oyatie "
    + "--class implement-claimed-lane --note \"handoff pr=N ready_for_babysit\"\n"
    + "5) python3 .grok/bin/mm-fabric-status --cwd /Users/jasonlee/Developer/oyatie\n"
    + "6) Next productive act: claim a DIFFERENT path-disjoint ready lane or empty-board success.\n"
    + "Summarize handoff to W3 only.",
    #{ label: "w2-handoff", capability_mode: "execute", agent_type: "general-purpose" }
);

complete(#{
    workflow: "implement-claimed-lane",
    fabric_class: "implement-claimed-lane",
    claim_ok: claim != () && claim.success,
    impl_ok: impl != () && impl.success,
    handoff_ok: handoff != () && handoff.success,
});
