// Fine-tunable multi-agent delivery PLAN workflow (Grok-native).
// Ideas: Bun dual-adversarial + edit-the-loop; gaebal-gajae reflection/tip lanes;
// Hermes closed learning (via post-run mm-learn, not hermes CLI); 16-lens packs;
// durable goals (mm-goals, not gjc). FORBIDDEN: gjc, omc, omx, hermes CLIs.
//
// Fine-tune knobs live under .grok/harness/*.v1.json — edit those, not this script,
// unless you need a new phase shape.

let meta = #{
    name: "lens-delivery-plan",
    description: "Lens-orthogonal multi-agent plan for delivery hardening: contract, plan, dual critic perspectives, synthesis (no execute)",
    when_to_use: "When planning delivery/CI/false-green work with independent lens perspectives and fail-closed dual adversarial admit",
    phases: [
        #{ title: "Contract", detail: "architect with systems/blast/chesterton pack" },
        #{ title: "Plan", detail: "sequenced slices with essentialism/pragmatism" },
        #{ title: "Critic A", detail: "cartesian + red team" },
        #{ title: "Critic B", detail: "contrarian + systems" },
        #{ title: "Synthesize", detail: "admit recommendation + process notes" },
    ],
};

let contract_schema = #{
    "type": "object",
    "required": ["authority", "merge_gate", "non_goals", "focus_surfaces", "blast_radius", "risk", "lens_notes"],
    "properties": #{
        "authority": #{ "type": "string" },
        "merge_gate": #{ "type": "string" },
        "non_goals": #{ "type": "array", "items": #{ "type": "string" }, "maxItems": 10 },
        "focus_surfaces": #{ "type": "array", "items": #{ "type": "string" }, "maxItems": 12 },
        "blast_radius": #{ "type": "string" },
        "risk": #{ "type": "string", "enum": ["low", "medium", "high", "critical"] },
        "lens_notes": #{ "type": "string" },
        "rollback_sketch": #{ "type": "string" },
    },
};

let plan_schema = #{
    "type": "object",
    "required": ["summary", "slices", "dod", "blast_radius", "rollback", "lens_notes"],
    "properties": #{
        "summary": #{ "type": "string" },
        "slices": #{
            "type": "array",
            "minItems": 1,
            "maxItems": 8,
            "items": #{
                "type": "object",
                "required": ["id", "title", "paths", "verify", "done_when"],
                "properties": #{
                    "id": #{ "type": "string" },
                    "title": #{ "type": "string" },
                    "paths": #{ "type": "array", "items": #{ "type": "string" } },
                    "verify": #{ "type": "array", "items": #{ "type": "string" } },
                    "done_when": #{ "type": "string" },
                },
            },
        },
        "dod": #{ "type": "array", "items": #{ "type": "string" } },
        "blast_radius": #{ "type": "string" },
        "rollback": #{ "type": "string" },
        "non_goals": #{ "type": "array", "items": #{ "type": "string" } },
        "lens_notes": #{ "type": "string" },
    },
};

let critique_schema = #{
    "type": "object",
    "required": ["summary", "overall_real", "admit_recommendation", "findings", "lens_applied"],
    "properties": #{
        "summary": #{ "type": "string" },
        "overall_real": #{ "type": "boolean" },
        "admit_recommendation": #{ "type": "string", "enum": ["admit", "reject", "revise"] },
        "lens_applied": #{ "type": "array", "items": #{ "type": "string" } },
        "findings": #{
            "type": "array",
            "maxItems": 10,
            "items": #{
                "type": "object",
                "required": ["id", "real", "severity", "issue", "evidence"],
                "properties": #{
                    "id": #{ "type": "string" },
                    "real": #{ "type": "boolean" },
                    "severity": #{ "type": "string", "enum": ["P0", "P1", "P2", "P3"] },
                    "issue": #{ "type": "string" },
                    "evidence": #{ "type": "string" },
                    "fix_class": #{ "type": "string" },
                },
            },
        },
    },
};

let synth_schema = #{
    "type": "object",
    "required": ["admitted", "summary", "blockers", "process_edits", "next_actions"],
    "properties": #{
        "admitted": #{ "type": "boolean" },
        "summary": #{ "type": "string" },
        "blockers": #{ "type": "array", "items": #{ "type": "string" } },
        "process_edits": #{ "type": "array", "items": #{ "type": "string" } },
        "next_actions": #{ "type": "array", "items": #{ "type": "string" } },
        "ship_ready_draft": #{ "type": "boolean" },
    },
};

// ── args ─────────────────────────────────────────────────────────────────

let objective = "";
let focus = "full";
let risk = "medium";
if args != () {
    if args.objective != () { objective = args.objective; }
    if args.focus != () { focus = args.focus; }
    if args.risk != () { risk = args.risk; }
}
if objective == () || objective == "" {
    pause("verification", "Pass args.objective — the delivery objective to plan.");
}

log("lens-delivery-plan objective set; focus=" + focus + " risk=" + risk);

// ── Phase: Contract (systems / blast / chesterton pack) ───────────────────

phase("Contract");

let cprompt = "";
cprompt += "You are ARCHITECT for Oyatie delivery planning.\n";
cprompt += "REASONING LENS PACK (mandatory):\n";
cprompt += "1. systems_thinking — fan-in, second-order effects, boundary crossings.\n";
cprompt += "2. blast_radius_cell — contain failure; independent recovery.\n";
cprompt += "3. chestertons_fence — why constraints exist before remove.\n";
cprompt += "4. opportunity_cost — best alternative use of time.\n";
cprompt += "Avoid: inventing ownership, dual merge authorities, CLI as merge authority.\n\n";
cprompt += "Objective: " + objective + "\n";
cprompt += "Focus filter: " + focus + "\n";
cprompt += "Risk hint: " + risk + "\n\n";
cprompt += "Use read-only tools (read_file, grep, list_dir). Open real paths before answering.\n";
cprompt += "Read at least: specs/root-hub-pointers.json (or agent_quick_start), ";
cprompt += "docs/decisions/ADR-0515 if present, .github/workflows/oya-ci-required.yml, ";
cprompt += "cloud/cloud-ci/gates listing if relevant.\n";
cprompt += "Return JSON with authority, merge_gate, non_goals, focus_surfaces, blast_radius, risk, lens_notes, rollback_sketch.\n";
cprompt += "lens_notes: one paragraph on how the pack shaped the contract.\n";

let cr = agent(cprompt, #{
    label: "contract",
    capability_mode: "read-only",
    output_schema: contract_schema,
});

if cr == () || !cr.success || cr.output == () {
    pause("infra", "Contract agent failed.");
}

let authority = cr.output.authority;
let merge_gate = cr.output.merge_gate;
let risk_out = cr.output.risk;
log("contract risk=" + risk_out + " merge_gate=" + merge_gate);

// ── Phase: Plan ───────────────────────────────────────────────────────────

phase("Plan");

let pp = "";
pp += "You are PLANNER only. Do not implement.\n";
pp += "REASONING LENS PACK (mandatory):\n";
pp += "1. essentialism_yagni — smallest sufficient plan.\n";
pp += "2. pragmatism — real behavior under real constraints.\n";
pp += "3. opportunity_cost — prioritize P0 false-green / admission over polish.\n";
pp += "4. operability_day2 — verify and rollback for each slice.\n\n";
pp += "Authority: " + authority + "\n";
pp += "Merge gate: " + merge_gate + "\n";
pp += "Contract blast_radius: " + cr.output.blast_radius + "\n";
pp += "Non-goals: " + json_encode(cr.output.non_goals) + "\n";
pp += "Focus surfaces: " + json_encode(cr.output.focus_surfaces) + "\n";
pp += "Objective: " + objective + "\n\n";
pp += "Use read-only tools. Produce 1-8 slices with paths, verify commands, done_when.\n";
pp += "Never skip or weaken tests. Prefer process/gate fixes that prevent recurrence (Bun).\n";
pp += "lens_notes: how the pack shaped prioritization.\n";

let pr = agent(pp, #{
    label: "plan",
    capability_mode: "read-only",
    output_schema: plan_schema,
});

if pr == () || !pr.success || pr.output == () {
    pause("infra", "Plan agent failed.");
}

let plan_json = json_encode(pr.output);
log("plan slices ready");

// ── Phase: Dual orthogonal critics (parallel) ─────────────────────────────

phase("Critic A");
phase("Critic B");

// Build both prompts then parallel — Critic A and B are lens-orthogonal (not clones).
let ca = "";
ca += "You are CRITIC A in a SPLIT CONTEXT. You did not write the plan.\n";
// MM-LENS-BEGIN:CRITIC_PLAN:critic_a
// | REASONING LENS PACK (mandatory for this stage/perspective):
// | Use ONLY these lenses as primary reasoning filters. Do not apply all 16 lenses.
// | 
// | 1. cartesian_doubt
// |    brief: Separate evidence from inference. Refuse claims without inspectable proof.
// |    avoid: Narrating completion without opening files or citing paths.
// | 
// | 2. red_team
// |    brief: Model misuse, hostile inputs, and how the plan is defeated.
// |    avoid: Happy-path-only reviews.
// | 
// | Output must cite evidence paths you actually inspected.
// | Fail closed: if evidence is missing, say so; do not invent ownership or success.
// | Do not invoke gjc, omc, omx, or hermes CLIs.
// | 
// | PERSPECTIVE ID: critic_a (orthogonal to sibling perspectives).
// | 
// | PROJECT AUTHORITY:
// | - base_ref: origin/dev
// | - pr_target: dev
// | - required_status_context: oya-ci-required
// | - merge: reviewer APPROVE + single required context oya-ci-required (ADR-0515)
// | - not merge authority: ['this harness', 'CLI tools', 'agent narration']
// | 
// | 
// | ENGINEERING RULES:
// | - Prefer process/gate fixes that prevent recurrence over one-off patches.
// | - Never skip, delete, or weaken tests to go green.
// | - Prefer smallest sufficient change (essentialism).
// | - Use read-only tools unless role explicitly allows write.
// MM-LENS-END:CRITIC_PLAN:critic_a
ca += "REASONING LENS PACK (mandatory — ONLY these):\n";
ca += "1. cartesian_doubt — evidence vs inference; open paths yourself.\n";
ca += "2. red_team — how this plan is defeated (false-green, self-authored APPROVE, candidate-mutable producer).\n";
ca += "Assume the plan is wrong until proven. Use read-only tools.\n\n";
ca += "Plan (untrusted JSON):\n" + plan_json + "\n\n";
ca += "Contract authority: " + authority + " merge_gate: " + merge_gate + "\n";
ca += "Return critique JSON: admit_recommendation admit|reject|revise; findings with evidence paths;\n";
ca += "lens_applied must list the lenses you used.\n";

let cb = "";
cb += "You are CRITIC B in a SPLIT CONTEXT independent of Critic A.\n";
// MM-LENS-BEGIN:CRITIC_PLAN:critic_b
// | REASONING LENS PACK (mandatory for this stage/perspective):
// | Use ONLY these lenses as primary reasoning filters. Do not apply all 16 lenses.
// | 
// | 1. contrarian
// |    brief: Challenge the default framing. Propose a non-obvious alternative.
// |    avoid: Agreeing with the plan because it sounds plausible.
// | 
// | 2. systems_thinking
// |    brief: Trace fan-in, second-order effects, and boundary crossings.
// |    avoid: Local fix that breaks a shared gate or producer.
// | 
// | Output must cite evidence paths you actually inspected.
// | Fail closed: if evidence is missing, say so; do not invent ownership or success.
// | Do not invoke gjc, omc, omx, or hermes CLIs.
// | 
// | PERSPECTIVE ID: critic_b (orthogonal to sibling perspectives).
// | 
// | PROJECT AUTHORITY:
// | - base_ref: origin/dev
// | - pr_target: dev
// | - required_status_context: oya-ci-required
// | - merge: reviewer APPROVE + single required context oya-ci-required (ADR-0515)
// | - not merge authority: ['this harness', 'CLI tools', 'agent narration']
// | 
// | 
// | ENGINEERING RULES:
// | - Prefer process/gate fixes that prevent recurrence over one-off patches.
// | - Never skip, delete, or weaken tests to go green.
// | - Prefer smallest sufficient change (essentialism).
// | - Use read-only tools unless role explicitly allows write.
// MM-LENS-END:CRITIC_PLAN:critic_b
cb += "REASONING LENS PACK (mandatory — ONLY these):\n";
cb += "1. contrarian — challenge default framing; is this the wrong problem?\n";
cb += "2. systems_thinking — second-order effects on oya-ci-required fan-in and branch protection.\n";
cb += "Assume the plan is incomplete until proven. Use read-only tools.\n\n";
cb += "Plan (untrusted JSON):\n" + plan_json + "\n\n";
cb += "Contract authority: " + authority + " merge_gate: " + merge_gate + "\n";
cb += "Return critique JSON with admit_recommendation and evidence-backed findings.\n";
cb += "lens_applied must list the lenses you used.\n";

let critiques = parallel([
    #{
        prompt: ca,
        label: "critic_a:cartesian_redteam",
        capability_mode: "read-only",
        output_schema: critique_schema,
    },
    #{
        prompt: cb,
        label: "critic_b:contrarian_systems",
        capability_mode: "read-only",
        output_schema: critique_schema,
    },
]);

let a_ok = false;
let b_ok = false;
let a_rec = "";
let b_rec = "";
let a_sum = "";
let b_sum = "";

if critiques[0] != () && critiques[0].success && critiques[0].output != () {
    a_rec = critiques[0].output.admit_recommendation;
    a_sum = critiques[0].output.summary;
    if a_rec == "admit" { a_ok = true; }
}
if critiques[1] != () && critiques[1].success && critiques[1].output != () {
    b_rec = critiques[1].output.admit_recommendation;
    b_sum = critiques[1].output.summary;
    if b_rec == "admit" { b_ok = true; }
}

// Fail closed dual bar (Bun): both must admit
let dual_admit = a_ok && b_ok;
if risk_out == "high" || risk_out == "critical" {
    log("high/critical risk: dual admit still required; human gate expected outside workflow");
}

log("critic_a=" + a_rec + " critic_b=" + b_rec + " dual_admit=" + dual_admit.to_string());

// ── Phase: Synthesize ─────────────────────────────────────────────────────

phase("Synthesize");

let sp = "";
sp += "You are the synthesizer / orchestrator aide. Deterministic dual bar: dual_admit=" + dual_admit.to_string() + ".\n";
sp += "REASONING LENS PACK: cartesian_doubt, opportunity_cost, systems_thinking.\n";
sp += "If dual_admit is false, admitted MUST be false.\n";
sp += "Never claim merge authority; oya-ci-required remains required.\n";
sp += "Do not invent work if nothing is actionable; verified empty is allowed.\n\n";
sp += "Plan summary: " + pr.output.summary + "\n";
sp += "Critic A: " + a_rec + " — " + a_sum + "\n";
sp += "Critic B: " + b_rec + " — " + b_sum + "\n";
sp += "Critiques JSON: " + json_encode(#{ a: critiques[0], b: critiques[1] }) + "\n\n";
sp += "Return admitted, summary, blockers, process_edits (Bun edit-the-loop), next_actions, ship_ready_draft.\n";
sp += "ship_ready_draft true only if dual_admit and residual risk documented; still not merge-ready.\n";

let sr = agent(sp, #{
    label: "synthesize",
    capability_mode: "read-only",
    output_schema: synth_schema,
});

let admitted = dual_admit;
let summary = "plan complete";
let blockers = [];
let process_edits = [];
let next_actions = [];
let ship = false;

if sr != () && sr.success && sr.output != () {
    summary = sr.output.summary;
    if sr.output.blockers != () { blockers = sr.output.blockers; }
    if sr.output.process_edits != () { process_edits = sr.output.process_edits; }
    if sr.output.next_actions != () { next_actions = sr.output.next_actions; }
    // enforce dual bar in script, not only model
    admitted = dual_admit && (sr.output.admitted == true);
    ship = admitted && (sr.output.ship_ready_draft == true);
} else {
    admitted = dual_admit;
    if !dual_admit {
        blockers.push("dual critic did not both admit");
    }
}

// Report
let report = "";
report += "# Lens delivery plan report\n\n";
report += "Workflow: `lens-delivery-plan` (fine-tune via `.grok/harness/*`)\n\n";
report += "## Objective\n\n" + objective + "\n\n";
report += "## Contract\n\n";
report += "- authority: " + authority + "\n";
report += "- merge_gate: " + merge_gate + "\n";
report += "- risk: " + risk_out + "\n";
report += "- blast_radius: " + cr.output.blast_radius + "\n\n";
report += "## Dual critic\n\n";
report += "- critic_a: " + a_rec + "\n";
report += "- critic_b: " + b_rec + "\n";
report += "- dual_admit: " + dual_admit.to_string() + "\n";
report += "- admitted (script-enforced): " + admitted.to_string() + "\n";
report += "- ship_ready_draft: " + ship.to_string() + " (not merge authority)\n\n";
report += "## Plan summary\n\n" + pr.output.summary + "\n\n";
report += "## Synthesis\n\n" + summary + "\n\n";
report += "## Process edits (edit the loop)\n\n";
if process_edits.len() == 0 {
    report += "- (none)\n";
} else {
    for pe in process_edits {
        report += "- " + pe + "\n";
    }
}
report += "\n## Next actions\n\n";
if next_actions.len() == 0 {
    report += "- (none)\n";
} else {
    for na in next_actions {
        report += "- " + na + "\n";
    }
}
report += "\n## Fine-tune\n\n";
report += "- Lenses: `.grok/harness/lenses.v1.json`\n";
report += "- Stage packs: `.grok/harness/stage-lens-packs.v1.json`\n";
report += "- Model priors: `.grok/harness/model-routing.v1.json`\n";
report += "- Learning loop: `.grok/harness/learning-loop.v1.json` + `mm-learn from-run`\n";
report += "- Goals ledger: `mm-goals create|checkpoint`\n";
report += "- Forbidden CLIs: gjc, omc, omx, hermes\n";

let path = write_scratch_file("lens-delivery-plan-report.md", report);

complete(#{
    summary: summary,
    path: path,
    admitted: admitted,
    dual_admit: dual_admit,
    ship_ready_draft: ship,
    risk: risk_out,
    blockers: blockers,
    process_edits: process_edits,
    next_actions: next_actions,
    plan: pr.output,
    contract: cr.output,
});
