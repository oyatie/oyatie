let meta = #{
    name: "parallel-delivery-bun",
    description: "Bun-discipline parallel delivery: trial gates first, then bounded fan-out for PR maintain + reorg lanes",
    when_to_use: "multi-PR CI babysit + concurrent reorg/capacity lanes under maxRunners pressure",
    phases: [
        #{ title: "Trial", detail: "capacity + path-filter + ADR trial status; expand gate" },
        #{ title: "Fan-out", detail: "bounded parallel PR fix; capacity expansion only if trial green" },
        #{ title: "Synthesize", detail: "merge readiness, process_edits, blockers" },
    ],
};

// Bun lessons: (1) prep contract first (2) one trial before fan-out (3) dual-critic
// (4) fail-closed missing evidence (5) fix process when systematic (6) no expand while trial red
//
// PROCESS DEFINITION: "fix the process" = edit this workflow / .grok/bin / drive.v1 /
// multi-model-roles so the next agent cannot repeat the mistake. Not a one-off rebase
// or chat reminder. See BUN-PARALLEL-DISCIPLINE.md.

let PROCESS_RULES =
    "PROCESS: fix process = edit harness/workflow/tool (not only the symptom). "
    + "Preflight only at PR head. No passive WAIT without armed re-poll. "
    + "No fan-out expansion while trial red. Return process_edits when systematic.\n";

phase("Trial");
let trial = agent(
    PROCESS_RULES
    + "Re-query Oyatie CI capacity and trial PRs only. Use gh/mm-drive. "
    + "Return a JSON object (in a fenced json block) with EXACT keys:\n"
    + "{\n"
    + "  \"runners_online\": <int>,\n"
    + "  \"runners_busy\": <int>,\n"
    + "  \"pr_1564_head\": \"<sha or null>\",\n"
    + "  \"pr_1564_oya_ci\": \"SUCCESS|FAILURE|null\",\n"
    + "  \"pr_1562_oya_ci\": \"SUCCESS|FAILURE|null\",\n"
    + "  \"pr_1569_exists\": <bool>,\n"
    + "  \"pr_1569_oya_ci\": \"SUCCESS|FAILURE|null\",\n"
    + "  \"trial_ready_to_expand\": <bool>,\n"
    + "  \"blockers\": [\"...\"],\n"
    + "  \"process_edits\": [\"file: rule\"]\n"
    + "}\n"
    + "trial_ready_to_expand=true ONLY if: (dual-worker PR 1564 not red on latest head "
    + "OR capacity already maxRunners>=2 live) AND no hard capacity blocker. "
    + "Do not merge anything. Fail-closed: if gh fails, trial_ready_to_expand=false.\n"
    + "Repo /Users/jasonlee/Developer/oyatie.",
    #{ label: "trial-capacity", capability_mode: "execute", agent_type: "explore" }
);

let expand = false;
let trial_note = "trial failed or missing";
if trial != () && trial.success && trial.output != () {
    trial_note = trial.output;
    // Strict expand markers only (process: fail closed on ambiguous trial)
    if type_of(trial.output) == "string" {
        if trial.output.contains("\"trial_ready_to_expand\": true")
            || trial.output.contains("\"trial_ready_to_expand\":true")
            || trial.output.contains("trial_ready_to_expand=true") {
            expand = true;
        }
        // Explicit false wins over true if both appear (agent contradiction)
        if trial.output.contains("\"trial_ready_to_expand\": false")
            || trial.output.contains("\"trial_ready_to_expand\":false")
            || trial.output.contains("trial_ready_to_expand=false") {
            expand = false;
        }
    }
}
log("trial expand=" + expand.to_string());

phase("Fan-out");
// Always maintain open PRs (bounded). Capacity/reorg expansion only if expand==true
// except inventory-only docs (always-safe).
let jobs = [];

// Always-on: PR maintain (bounded) — still under PROCESS_RULES
jobs.push(#{
    prompt: PROCESS_RULES
        + "PR maintenance for open PRs against dev. For each open PR: gh pr checks; "
        + "if red diagnose authoritative job log root cause; fix only that PR worktree AT PR head; "
        + "preflight-ci-infra --pr N before push if CI paths; dual-critic re-head; ONE push. "
        + "Priority: capacity/path-filter trial PRs first, then behind-tip PRs (tip-sync), then others. "
        + "Use mm-drive merge when merge-check ok (agent dual-critic). "
        + "Bun: do not expand scope on red PRs. "
        + "Return list of PRs fixed/merged/waiting + process_edits (harness path + rule) if systematic.",
    label: "pr-maintain",
    capability_mode: "all",
    agent_type: "general-purpose",
});

// Always-on: docs/inventory reorg (safe without capacity trial)
jobs.push(#{
    prompt: PROCESS_RULES
        + "Oyatie reorg concurrent lanes (REORG-DOCTRINE: move|refactor|rewrite|delete; multi-cap ok): "
        + "Babysit inventory/docs reorg PRs only; dual-critic re-head if needed; "
        + "if CI red fix only. Optionally open ONE inventory-only PR under evidence/reorg/. "
        + "No bulk multi-cap moves. Bun: trial capacity not required for docs inventory. "
        + "Return process_edits if doctrine/workflow gap found.",
    label: "reorg-inventory",
    capability_mode: "all",
    agent_type: "general-purpose",
});

// Gated: cancel-in-progress / path-filter expansion only when trial ready
if expand {
    jobs.push(#{
        prompt: PROCESS_RULES
            + "TRIAL GREEN — expand allowed. Oyatie cancel-in-progress Wave4: "
            + "if ADR-0639 / PR 1569 path exists, implement or babysit concurrency group "
            + "cancel-in-progress on oya-ci-required; dual-critic; preflight at PR head; ONE push. "
            + "If trial was borderline, prefer babysit over new scope. "
            + "Return status + process_edits.",
        label: "cancel-in-progress",
        capability_mode: "all",
        agent_type: "general-purpose",
    });
} else {
    jobs.push(#{
        prompt: PROCESS_RULES
            + "TRIAL NOT READY TO EXPAND — do NOT open new capacity/path-filter legs or cancel-in-progress "
            + "implementation PRs. Only: re-query trial PRs; document blockers; propose process_edits "
            + "if agents previously expanded while red. Return hold + blockers. No code push for expansion.",
        label: "expand-hold",
        capability_mode: "read-only",
        agent_type: "explore",
    });
}

// Always-on: k8s packet discipline (no W0-B expand while G001 incomplete)
jobs.push(#{
    prompt: PROCESS_RULES
        + "Oyatie k8s port G001: if PR open and oya-ci-required green, mm-drive merge; then write "
        + "post-merge packet with mm-packet after trunk green. If not green, fix only new reds. "
        + "Do not start W0-B engine crates until G001 packet complete (Bun: no expand while trial incomplete). "
        + "Return status + process_edits if packet/process gap.",
    label: "k8s-g001",
    capability_mode: "all",
    agent_type: "general-purpose",
});

let results = parallel(jobs);

phase("Synthesize");
let synth = agent(
    PROCESS_RULES
    + "Synthesize parallel delivery status from agent results (fail-closed if missing). "
    + "Produce markdown:\n"
    + "(1) trial capacity + expand=" + expand.to_string() + "\n"
    + "(2) each PR status merge/red/wait\n"
    + "(3) reorg\n"
    + "(4) process_edits: concrete file+rule changes that prevent recurrence "
    + "(definition: harness edit, not one-off ops)\n"
    + "(5) next Bun-gated steps\n"
    + "Do not claim ultragoal complete.\n"
    + "Trial note:\n" + trial_note + "\nResults:\n" + json_encode(results),
    #{ label: "synth", capability_mode: "read-only", agent_type: "explore" }
);

let report = "Synthesis failed or empty — fail-closed. Check parallel panel results.";
if synth != () && synth.success && synth.output != () {
    report = synth.output;
}
complete(#{
    report: report,
    expand: expand,
    trial_note: trial_note,
    process_definition: "edit harness/workflow/tool so agents cannot repeat the mistake",
});
