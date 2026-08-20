let meta = #{
    name: "open-pr-fleet",
    description: "Babysit every open PR vs dev: diagnose, fix, re-run hosted CI, dual-critic, tip-sync, merge when ready",
    when_to_use: "After public GHA flip or anytime open PR queue must be drained under one orchestration",
    phases: [
        #{ title: "Census", detail: "list all open PRs and ground truth" },
        #{ title: "Critical path", detail: "land hosted-CI / blocking PR first" },
        #{ title: "Fleet fan-out", detail: "one agent per remaining open PR" },
        #{ title: "Tip-sync", detail: "after merges, rebase remaining PRs onto new origin/dev" },
        #{ title: "Synthesize", detail: "status board, process_edits, next actions" },
    ],
};

// Ground truth: repo PUBLIC; free GHA hosted; lab ARC may be 0 runners.
// Merge authority: dual-critic APPROVE + oya-ci-required green; mm-drive merge when ok.
// Hard stops: no hand-edit *.generated.json; no warm CAS flip; no ultragoal complete claim;
// no re-private; do not force-push shared branches without need.
//
// ── PROCESS DEFINITION (Bun: fix process = edit harness, not hero ops) ──
// "Fix the process" means change this workflow / .grok/bin tools / drive.v1 so
// the next agent cannot repeat the mistake. It does NOT mean only rebasing one
// PR, writing a chat reminder, or waiting passively on CI.
// See: .grok/programs/hyperscaler-delivery-lanes/BUN-PARALLEL-DISCIPLINE.md

let PROCESS_RULES =
    "PROCESS (MANDATORY — systematic, not one-off):\n"
    + "Definition: fix the process = edit workflow/harness/tool so agents cannot "
    + "repeat the same mistake. Not: one-off rebase, memory note, or passive WAIT.\n"
    + "A) PREFLIGHT AT PR HEAD: never run preflight/merge-check from clean tip, dirty "
    + "primary, or unrelated worktree. Tool: .grok/bin/preflight-ci-infra --pr N fails "
    + "closed if cwd HEAD != gh pr headRefOid.\n"
    + "B) ANTI-PASSIVE: waiting_ci is not idle_complete. Arm a re-poll monitor or "
    + "schedule re-tick; never leave fleet in WAIT without a next action timestamp.\n"
    + "C) TIP-SYNC AFTER MERGE: when any PR merges to dev, remaining open PRs must "
    + "fetch origin/dev and rebase in their own worktrees before claiming green.\n"
    + "D) ONE PUSH: batch all local reds; never push mid-run (cancel-in-progress kills FULL).\n"
    + "E) PROCESS_EDITS: if you hit a repeated class of failure, append a concrete "
    + "harness edit (file + rule) to your return — do not only fix the symptom.\n";

// R4 preflight (agents cannot skip for CI-infra paths):
//   Tool: .grok/bin/preflight-ci-infra  (NOT under infra/* — reorg target)
//   Wrapper: mm-drive preflight-ci --pr N
//   Receipt: .grok/programs/hyperscaler-delivery-lanes/evidence/preflight-ci-infra-pr-N.json
//   mm-drive merge-check fails closed without matching-head ok receipt on CI-infra PRs.

let PREFLIGHT_RULE =
    "PREFLIGHT (MANDATORY — do not skip):\n"
    + "Before ANY git push that touches .github/workflows/**, infra/ci/**, infra/arc/**, "
    + "ci/facade/**, cloud/cloud-ci/**, or oya-ci.toml:\n"
    + "  1) From the PR worktree AT THAT PR's head SHA:  .grok/bin/preflight-ci-infra --pr <N>\n"
    + "     (or: mm-drive preflight-ci --pr <N> --cwd <worktree>)\n"
    + "     Fail-closed if cwd HEAD != PR headRefOid (process gate; do not waive for receipts).\n"
    + "  2) Must exit 0 and write receipt under\n"
    + "     .grok/programs/hyperscaler-delivery-lanes/evidence/preflight-ci-infra-pr-<N>.json\n"
    + "     with ok:true and head matching git rev-parse HEAD.\n"
    + "  3) Only then dual-critic re-head → ONE push → wait for COMPLETE oya-ci-required "
    + "(never mid-run push; cancel-in-progress kills FULL cone).\n"
    + "  4) Soft multi-arch (windows) continue-on-error is NON-binding — ignore for merge.\n"
    + "  5) mm-drive merge-check / merge will FAIL without the receipt for CI-infra PRs.\n"
    + "Tool lives under .grok/bin only — do NOT create scripts under infra/*.\n";

phase("Census");
let census = agent(
    "Repo jason931225/oyatie is PUBLIC. Work from /Users/jasonlee/Developer/oyatie or existing worktrees.\n"
    + PROCESS_RULES
    + "Use gh only. List ALL open PRs against base dev (json): number, title, draft, mergeable, headRefName, "
    + "behind-by vs origin/dev (commits), oya-ci-required conclusion if any, failure check names, whether runs-on is still oya-arm64 vs ubuntu-latest "
    + "on tip (sample workflow if needed). Mark which need rebase onto origin/dev after recent merges. "
    + "Return a markdown table covering EVERY open PR number — do not omit any. Fail-closed if gh fails.\n"
    + "Note for synthesis: any CI-infra PR must use PREFLIGHT rule (tool .grok/bin/preflight-ci-infra).",
    #{ label: "census", capability_mode: "execute", agent_type: "explore" }
);

phase("Critical path");
// Prefer hosted multi-arch / CAS / capacity blockers first when open; else first greenable CI-infra.
let p_critical = agent(
    "CRITICAL PATH: land the highest-blocking open PR first (historically #1573 hosted multi-arch + CAS; "
    + "if already merged, pick next CI-infra/capacity PR that unblocks the fleet).\n"
    + "Repo /Users/jasonlee/Developer/oyatie. Use existing worktrees; never dirty primary as receipt source.\n"
    + PROCESS_RULES
    + PREFLIGHT_RULE
    + "Steps:\n"
    + "1) gh pr checks + gh run view for latest oya-ci-required.\n"
    + "2) If CONTENT reds: fix batch locally IN THE PR WORKTREE; run preflight --pr N until exit 0 "
    + "BEFORE any push. Re-run failed jobs only if no code change needed (flake).\n"
    + "3) Soft multi-arch legs may continue-on-error — binding linux-amd64 must green.\n"
    + "4) Dual-critic re-head if code changes; ONE push; wait for complete run; "
    + "mm-drive merge-check --pr N; if ok, mm-drive merge --pr N.\n"
    + "5) After merge: git fetch origin dev (tip-sync seed for next phase).\n"
    + "6) Return: final check state, merge yes/no, blockers, head SHA, preflight receipt path, "
    + "and any process_edits (harness file + rule) if you hit a systematic failure class.\n"
    + "Hard stops: no force-push origin/dev; no warm license flip; no secrets in git; no mid-run push.",
    #{ label: "pr-critical", capability_mode: "all", agent_type: "general-purpose" }
);

phase("Fleet fan-out");
// One agent per remaining open PR so none is orphaned. Agents must re-query if PR already merged.
let jobs = [];

jobs.push(#{
    prompt: "OWN PR #1566 only (docs harness RR-HARNESS-0619). Oyatie PUBLIC, free GHA.\n"
        + PROCESS_RULES + PREFLIGHT_RULE
        + "gh pr view/checks 1566; if red: root-cause from job logs; fix in that PR worktree only; "
        + "rebase onto origin/dev if behind; dual-critic; push only after preflight if CI paths; "
        + "merge-check; merge if ok.\n"
        + "Return: status fixed|waiting|blocked|merged + one-line reason + process_edits if any. No other PRs.",
    label: "pr-1566",
    capability_mode: "all",
    agent_type: "general-purpose",
});

jobs.push(#{
    prompt: "OWN PR #1567 only (reorg move-plan hygiene). Oyatie PUBLIC, free GHA.\n"
        + PROCESS_RULES + PREFLIGHT_RULE
        + "gh pr checks; fix content reds only; rebase onto origin/dev if needed; "
        + "dual-critic; push only after preflight if CI paths; merge-check; merge if ok. "
        + "Return fixed|waiting|blocked|merged + process_edits if any. No other PRs.",
    label: "pr-1567",
    capability_mode: "all",
    agent_type: "general-purpose",
});

jobs.push(#{
    prompt: "OWN PR #1568 only (reorg dual-home inventory R-DUAL-0615). Oyatie PUBLIC.\n"
        + PROCESS_RULES + PREFLIGHT_RULE
        + "Verify tip vs parent-check; fix reds; rebase if needed; dual-critic; "
        + "merge if merge-check ok.\n"
        + "Return fixed|waiting|blocked|merged + process_edits if any. No other PRs.",
    label: "pr-1568",
    capability_mode: "all",
    agent_type: "general-purpose",
});

jobs.push(#{
    prompt: "OWN PR #1569 only (ADR-0639 path/event-optional legs). Oyatie PUBLIC.\n"
        + PROCESS_RULES + PREFLIGHT_RULE
        + "If CONFLICTING or behind origin/dev — rebase carefully; resolve conflicts; "
        + "do not drop ADR content. Fix CI reds; preflight at PR head before push if CI paths; dual-critic; "
        + "merge-check; merge if ok.\n"
        + "Return fixed|waiting|blocked|merged + process_edits if any. No other PRs.",
    label: "pr-1569",
    capability_mode: "all",
    agent_type: "general-purpose",
});

jobs.push(#{
    prompt: "OWN PR #1570 only (cancel-in-progress docs + Tide note). Oyatie PUBLIC.\n"
        + PROCESS_RULES + PREFLIGHT_RULE
        + "Prefer merge after 1569 if dependent; else merge when green. Fix reds; "
        + "preflight at PR head before push if CI paths; dual-critic; merge-check; merge if ok. "
        + "Return fixed|waiting|blocked|merged + process_edits if any. No other PRs.",
    label: "pr-1570",
    capability_mode: "all",
    agent_type: "general-purpose",
});

jobs.push(#{
    prompt: "OWN PR #1571 only (draft PLAN residual ci-move-plan). Oyatie PUBLIC.\n"
        + PROCESS_RULES + PREFLIGHT_RULE
        + "Keep draft unless ready. Diagnose CI; fix or document hold reason. Dual-critic if undrafting.\n"
        + "Return fixed|waiting|blocked|merged|draft-hold + process_edits if any. No other PRs.",
    label: "pr-1571",
    capability_mode: "all",
    agent_type: "general-purpose",
});

jobs.push(#{
    prompt: "OWN PR #1572 only (GH-standard runner units). Oyatie PUBLIC; hosted is primary now.\n"
        + PROCESS_RULES + PREFLIGHT_RULE
        + "Triage: close as superseded by hosted-first OR retarget as lab-overflow-only docs, "
        + "OR fix and merge if still valuable. "
        + "If code push: preflight --pr 1572 required (infra/arc paths). Dual-critic if code push.\n"
        + "Return closed|merged|waiting|blocked + rationale + process_edits if any. No other PRs.",
    label: "pr-1572",
    capability_mode: "all",
    agent_type: "general-purpose",
});

// Catch-all: any open PR number not in 1566-1573 (e.g. newly opened)
jobs.push(#{
    prompt: "CATCH-ALL open PRs: gh pr list --base dev --state open. For any number NOT in "
        + "{1566,1567,1568,1569,1570,1571,1572,1573}, diagnose and either babysit to merge-check "
        + "or file explicit blocked reason. If none, say none. Repo jason931225/oyatie PUBLIC free GHA.\n"
        + PROCESS_RULES + PREFLIGHT_RULE,
    label: "pr-catchall",
    capability_mode: "all",
    agent_type: "general-purpose",
});

let fleet = parallel(jobs);

phase("Tip-sync");
// Process fix: merges advance origin/dev; remaining PRs go red on scm-facts/parent unless rebased.
// Agents previously treated this as one-off rebases; make it a mandatory fleet phase.
let tip_sync = agent(
    "POST-MERGE TIP-SYNC (process phase — do not skip):\n"
    + PROCESS_RULES
    + "1) git fetch origin dev; record tip SHA.\n"
    + "2) gh pr list --base dev --state open --json number,headRefName,mergeable,statusCheckRollup.\n"
    + "3) For EACH open PR behind origin/dev: in THAT PR's worktree only, rebase onto origin/dev; "
    + "resolve conflicts; dual-critic only if behavior-changing; preflight --pr N if CI paths; ONE push.\n"
    + "4) Do not push from primary checkout or clean tip worktree.\n"
    + "5) If a PR is already up to date, say so — do not force empty rebases.\n"
    + "6) Return table: pr | was_behind | rebased | new_head | blockers | process_edits.\n"
    + "Fail-closed: if any open PR remains behind without an explicit blocked reason, report FAIL.\n"
    + "Critical-path result (may include merges):\n" + json_encode(p_critical)
    + "\nFleet panel:\n" + json_encode(fleet),
    #{ label: "tip-sync", capability_mode: "all", agent_type: "general-purpose" }
);

phase("Synthesize");
let synth = agent(
    "Synthesize open-PR fleet status. Inputs: census + critical + fleet + tip-sync.\n"
    + PROCESS_RULES
    + "Output markdown:\n"
    + "1) Table: every open PR at start → final status (merged/waiting/blocked/draft/closed)\n"
    + "2) Critical-path outcome (must be explicit)\n"
    + "3) Tip-sync: which PRs rebased onto new origin/dev; any still behind\n"
    + "4) Hosted GHA vs ARC dependency remaining\n"
    + "5) Preflight compliance: which CI-infra PRs had head-matching receipt / which skipped wrongly\n"
    + "6) process_edits: concrete harness/workflow/tool changes proposed or applied this run "
    + "(file path + rule). Empty only if zero systematic issues.\n"
    + "7) Anti-passive: any waiting_ci lanes MUST list armed monitor or re-tick time\n"
    + "8) Blockers requiring human\n"
    + "9) Next product spine only if capacity hard-stop cleared\n"
    + "Fail-closed: list any PR number from census missing from results.\n"
    + "Census:\n" + json_encode(census) + "\nCritical:\n" + json_encode(p_critical)
    + "\nFleet:\n" + json_encode(fleet) + "\nTip-sync:\n" + json_encode(tip_sync),
    #{ label: "synth", capability_mode: "read-only", agent_type: "explore" }
);

let report = "Synthesis failed — inspect workflow panels.";
if synth != () && synth.success && synth.output != () {
    report = synth.output;
}
complete(#{
    report: report,
    prs: "open-fleet+tip-sync",
    preflight: ".grok/bin/preflight-ci-infra",
    process_definition: "edit harness/workflow/tool so agents cannot repeat the mistake",
});
