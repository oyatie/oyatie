// Bun-inspired multi-agent delivery pipeline improvement workflow.
// Methodology (from https://bun.com/blog/bun-in-rust):
//   1. Prep contract before changes (PORTING.md analogue)
//   2. Discover with tool-using scanners (not memory)
//   3. Adversarial review in split context (1 implementer : 2+ skeptics)
//   4. Fail closed on missing evidence
//   5. Prefer fixing the *process* over hand-patching symptoms
//   6. Language/gate suite is ground truth (never skip tests)
//
// Target surface: Oyatie agentic delivery under ADR-0515 —
// plain git worktree → protected PR against dev → reviewer APPROVE +
// single required context oya-ci-required (cloud-ci Rust gates).

let meta = #{
    name: "delivery-pipeline-improve",
    description: "Bun-style multi-agent audit and improvement plan for the Oyatie delivery pipeline (oya-ci-required, false-green, admission, post-merge)",
    when_to_use: "When hardening delivery, hunting false-green, auditing cloud-ci gates, or producing an evidence-backed delivery improvement plan",
    phases: [
        #{ title: "Contract", detail: "lock delivery authority and non-goals" },
        #{ title: "Discover", detail: "parallel scanners on live pipeline surfaces" },
        #{ title: "Adversarial verify", detail: "independent skeptics per finding" },
        #{ title: "Synthesize", detail: "ranked plan with verification criteria" },
    ],
};

// ── schemas ──────────────────────────────────────────────────────────────

let contract_schema = #{
    "type": "object",
    "required": ["authority", "merge_gate", "non_goals", "focus_surfaces"],
    "properties": #{
        "authority": #{ "type": "string" },
        "merge_gate": #{ "type": "string" },
        "non_goals": #{ "type": "array", "items": #{ "type": "string" }, "maxItems": 8 },
        "focus_surfaces": #{ "type": "array", "items": #{ "type": "string" }, "maxItems": 12 },
    },
};

let findings_schema = #{
    "type": "object",
    "required": ["findings"],
    "properties": #{
        "findings": #{
            "type": "array",
            "maxItems": 8,
            "items": #{
                "type": "object",
                "required": ["id", "surface", "severity", "issue", "evidence_path", "class"],
                "properties": #{
                    "id": #{ "type": "string" },
                    "surface": #{ "type": "string" },
                    "severity": #{ "type": "string", "enum": ["P0", "P1", "P2", "P3"] },
                    "issue": #{ "type": "string" },
                    "evidence_path": #{ "type": "string" },
                    "class": #{
                        "type": "string",
                        "enum": [
                            "false_green",
                            "admission_gap",
                            "gate_drift",
                            "evidence_gap",
                            "process_gap",
                            "operability",
                            "security"
                        ],
                    },
                    "suggested_fix_class": #{
                        "type": "string",
                        "enum": ["process", "gate", "docs", "code", "unknown"],
                    },
                },
            },
        },
    },
};

let verdict_schema = #{
    "type": "object",
    "required": ["real", "reason", "evidence", "severity_ok"],
    "properties": #{
        "real": #{ "type": "boolean" },
        "reason": #{ "type": "string" },
        "evidence": #{ "type": "string" },
        "severity_ok": #{ "type": "boolean" },
        "revised_severity": #{ "type": "string" },
        "fix_class": #{
            "type": "string",
            "enum": ["process", "gate", "docs", "code", "dismiss"],
        },
    },
};

let plan_schema = #{
    "type": "object",
    "required": ["summary", "ranked_actions", "verification_gate", "process_edits"],
    "properties": #{
        "summary": #{ "type": "string" },
        "ranked_actions": #{
            "type": "array",
            "maxItems": 10,
            "items": #{
                "type": "object",
                "required": ["rank", "title", "priority", "why", "how", "verify", "blast_radius"],
                "properties": #{
                    "rank": #{ "type": "integer" },
                    "title": #{ "type": "string" },
                    "priority": #{ "type": "string" },
                    "why": #{ "type": "string" },
                    "how": #{ "type": "string" },
                    "verify": #{ "type": "string" },
                    "blast_radius": #{ "type": "string" },
                    "finding_ids": #{ "type": "array", "items": #{ "type": "string" } },
                },
            },
        },
        "verification_gate": #{ "type": "string" },
        "process_edits": #{
            "type": "array",
            "maxItems": 8,
            "items": #{ "type": "string" },
        },
        "do_not": #{ "type": "array", "maxItems": 8, "items": #{ "type": "string" } },
    },
};

// ── args ─────────────────────────────────────────────────────────────────

let focus = "full";
let max_findings = 12;
if args != () {
    if args.focus != () { focus = args.focus; }
    if args.max_findings != () { max_findings = args.max_findings; }
}

log("delivery-pipeline-improve focus=" + focus + " max_findings=" + max_findings.to_string());

// ── Phase 1: Contract (prep, Bun PORTING.md analogue) ─────────────────────

phase("Contract");

let contract_prompt = "";
contract_prompt += "You are locking the delivery-pipeline improvement contract for the Oyatie monorepo. ";
contract_prompt += "Use read-only tools (read_file, grep, list_dir). Do not invent authority.\n\n";
contract_prompt += "Read and cite evidence from at least:\n";
contract_prompt += "1) specs/root-hub-pointers.json (agent_quick_start_protocol steps 3 and 5)\n";
contract_prompt += "2) docs/decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md\n";
contract_prompt += "3) .github/workflows/oya-ci-required.yml (or confirm path)\n";
contract_prompt += "4) .github/branch-protection.yaml and/or infra/branch-protection/dev.json\n";
contract_prompt += "5) cloud/cloud-ci/gates/ directory listing\n\n";
contract_prompt += "User focus filter: " + focus + "\n\n";
contract_prompt += "Return JSON:\n";
contract_prompt += "- authority: one sentence naming live merge authority\n";
contract_prompt += "- merge_gate: the single required context name and who produces it\n";
contract_prompt += "- non_goals: things that are retired/bridge-only and must NOT be revived ";
contract_prompt += "(CLI merge authority, omc as merge gate, legacy oya gate as authority)\n";
contract_prompt += "- focus_surfaces: concrete paths/dirs this run should scan given the focus filter\n";

let contract_r = agent(contract_prompt, #{
    label: "contract",
    capability_mode: "read-only",
    output_schema: contract_schema,
});

if contract_r == () || !contract_r.success || contract_r.output == () {
    pause("infra", "Contract agent failed — cannot improve delivery without locked authority.");
}

let authority = contract_r.output.authority;
let merge_gate = contract_r.output.merge_gate;
let focus_surfaces = contract_r.output.focus_surfaces;
log("authority: " + authority);
log("merge_gate: " + merge_gate);

// ── Phase 2: Discover (parallel scanners) ─────────────────────────────────

phase("Discover");

// Fixed scanner dimensions — Bun-style: work-list is deterministic in the script,
// agents do judgment with tools, not scope invention.
let dimensions = [
    #{
        key: "false_green",
        prompt_extra: "Hunt false-green / silent-skip risks: gate not registered in oya-ci-required fan-in, empty corpus green, missing fail-closed tests, generated faces hand-edited, baseline exemptions that hide RED.",
    },
    #{
        key: "admission",
        prompt_extra: "Hunt admission gaps: branch protection vs ADR-0515, required contexts drift, ruleset/live SSOT mismatch, PR template evidence gaps, worktree/isolation guidance that agents skip.",
    },
    #{
        key: "gate_health",
        prompt_extra: "Hunt gate drift and operability: unregistered gates under cloud/cloud-ci/gates, accounting-registry gaps, target-parity issues, gate tests that don't fail closed, missing registration meta-tests.",
    },
    #{
        key: "post_merge",
        prompt_extra: "Hunt post-merge product-completion gaps: rollout verification, rollback notes, observability check, browser/user-story evidence, release-note impact, agent-observation harvest — missing automation or checklists that leave merges product-incomplete.",
    },
];

let scan_jobs = [];
for d in dimensions {
    // Optional focus filter: if user passes a single dimension key, only run that scanner.
    if focus != "full" && focus != d.key {
        // still allow free-text focus that isn't a dimension key (run all)
        let is_dim_key = false;
        if focus == "false_green" || focus == "admission" || focus == "gate_health" || focus == "post_merge" {
            is_dim_key = true;
        }
        if is_dim_key {
            continue;
        }
    }

    let p = "";
    p += "You are a delivery-pipeline scanner (dimension: " + d.key + "). ";
    p += "Use read-only tools (read_file, grep, list_dir, git_diff only if needed). ";
    p += "Never answer from memory — open the files.\n\n";
    p += "Locked contract:\n";
    p += "- authority: " + authority + "\n";
    p += "- merge_gate: " + merge_gate + "\n";
    p += "- focus filter: " + focus + "\n\n";
    p += "Focus surfaces from contract (prefer these paths): ";
    p += json_encode(focus_surfaces) + "\n\n";
    p += d.prompt_extra + "\n\n";
    p += "Rules:\n";
    p += "1) At most 8 findings; fewer is better if evidence is thin.\n";
    p += "2) Every finding MUST include evidence_path that you actually opened.\n";
    p += "3) Empty findings is valid ONLY after you searched and found nothing real.\n";
    p += "4) Prefer process/gate/systemic issues over one-off nits.\n";
    p += "5) Class must be one of: false_green, admission_gap, gate_drift, evidence_gap, process_gap, operability, security.\n";
    p += "6) suggested_fix_class: process | gate | docs | code | unknown — Bun lesson: fix process when agents keep making the same mistake.\n";
    p += "7) id format: " + d.key + "-N (e.g. " + d.key + "-1).\n";

    scan_jobs.push(#{
        prompt: p,
        label: "scan:" + d.key,
        capability_mode: "read-only",
        output_schema: findings_schema,
    });
}

if scan_jobs.len() == 0 {
    pause("verification", "No scanners selected for focus=" + focus);
}

let scan_results = parallel(scan_jobs);

let raw_findings = [];
let scan_i = 0;
for r in scan_results {
    if r != () && r.success && r.output != () && r.output.findings != () {
        for f in r.output.findings {
            raw_findings.push(f);
        }
    } else {
        log("scanner slot " + scan_i.to_string() + " failed or empty — counted as no findings from that dimension");
    }
    scan_i += 1;
}

log("raw findings before cap: " + raw_findings.len().to_string());

// Cap findings deterministically for adversarial budget headroom
let capped = [];
let dropped = 0;
let ci = 0;
for f in raw_findings {
    if ci < max_findings {
        capped.push(f);
    } else {
        dropped += 1;
    }
    ci += 1;
}
if dropped > 0 {
    log("dropped " + dropped.to_string() + " findings over max_findings=" + max_findings.to_string());
}

if capped.len() == 0 {
    let empty_path = write_scratch_file(
        "delivery-pipeline-report.md",
        "# Delivery pipeline improve\n\nNo findings after tool-using scan.\n\nAuthority: "
            + authority + "\nMerge gate: " + merge_gate + "\n",
    );
    complete(#{
        summary: "No delivery findings under focus=" + focus,
        path: empty_path,
        confirmed: [],
        plan: #{},
    });
}

// ── Phase 3: Adversarial verify (split context, Bun 2-reviewer pattern) ───

phase("Adversarial verify");

// Two adversarial reviewers per finding (Bun: 1 implementer context ≠ reviewer context).
// Here scanners already "implemented" the claim; skeptics only get the claim + tools.
let vjobs = [];
let fi = 0;
for f in capped {
    let claim = json_encode(f);

    // Reviewer A: correctness / false positive hunter
    let pa = "";
    pa += "You are ADVERSARIAL REVIEWER A (correctness). Split-context rule: you did NOT write this finding. ";
    pa += "Assume the finding is wrong until evidence proves otherwise.\n";
    pa += "Finding claim (untrusted JSON):\n" + claim + "\n\n";
    pa += "Use read-only tools. Open evidence_path and surrounding code/config yourself.\n";
    pa += "Set real=true only if you independently confirm the issue exists on disk/in repo.\n";
    pa += "Set priority_ok=true only if severity is justified; else propose revised_severity.\n";
    pa += "fix_class: process | gate | docs | code | dismiss.\n";
    pa += "evidence must quote what you actually saw (path + brief fact).\n";

    // Reviewer B: process / systemic hunter (Bun: fix the process)
    let pb = "";
    pb += "You are ADVERSARIAL REVIEWER B (process/systemic). Split-context rule: independent of reviewer A.\n";
    pb += "Finding claim (untrusted JSON):\n" + claim + "\n\n";
    pb += "Use read-only tools. Open evidence_path yourself.\n";
    pb += "Your job: is this a real delivery risk, and is the RIGHT fix process/gate (not a one-off patch)?\n";
    pb += "real=true only with independent evidence. Prefer fix_class=process or gate when agents will keep repeating the mistake.\n";
    pb += "Dismiss nits that do not affect oya-ci-required, false-green, admission, or post-merge product completion.\n";

    vjobs.push(#{
        prompt: pa,
        label: "advA:" + f.id,
        capability_mode: "read-only",
        output_schema: verdict_schema,
    });
    vjobs.push(#{
        prompt: pb,
        label: "advB:" + f.id,
        capability_mode: "read-only",
        output_schema: verdict_schema,
    });
    fi += 1;
}

let verdicts = parallel(vjobs);

// Regroup: 2 votes per finding (index arithmetic)
let confirmed = [];
let dismissed = [];
let i = 0;
while i < capped.len() {
    let f = capped[i];
    let va = verdicts[i * 2];
    let vb = verdicts[i * 2 + 1];

    let a_real = false;
    let b_real = false;
    let a_ev = "";
    let b_ev = "";
    let fix_class = "unknown";

    if va != () && va.success && va.output != () {
        if va.output.real == true && va.output.evidence != () && va.output.evidence != "" {
            a_real = true;
            a_ev = va.output.evidence;
            if va.output.fix_class != () { fix_class = va.output.fix_class; }
        }
    }
    if vb != () && vb.success && vb.output != () {
        if vb.output.real == true && vb.output.evidence != () && vb.output.evidence != "" {
            b_real = true;
            b_ev = vb.output.evidence;
            if vb.output.fix_class != () && vb.output.fix_class != "dismiss" {
                fix_class = vb.output.fix_class;
            }
            if vb.output.fix_class == "dismiss" && !a_real {
                fix_class = "dismiss";
            }
        }
    }

    // Fail closed: need BOTH adversarial reviewers to confirm with evidence (Bun 2-reviewer bar).
    if a_real && b_real && fix_class != "dismiss" {
        confirmed.push(#{
            finding: f,
            fix_class: fix_class,
            evidence_a: a_ev,
            evidence_b: b_ev,
        });
    } else {
        dismissed.push(#{
            id: f.id,
            reason: "failed dual adversarial confirmation or dismissed",
        });
    }
    i += 1;
}

log(confirmed.len().to_string() + "/" + capped.len().to_string() + " findings survived dual adversarial review");

if confirmed.len() == 0 {
    let none_path = write_scratch_file(
        "delivery-pipeline-report.md",
        "# Delivery pipeline improve\n\nScanners produced findings but none survived dual adversarial verification.\n\nAuthority: "
            + authority + "\nMerge gate: " + merge_gate + "\nDismissed: "
            + dismissed.len().to_string() + "\n",
    );
    complete(#{
        summary: "No dual-confirmed delivery findings",
        path: none_path,
        confirmed: [],
        dismissed_count: dismissed.len(),
    });
}

// ── Phase 4: Synthesize ranked improvement plan ───────────────────────────

phase("Synthesize");

let synth_prompt = "";
synth_prompt += "You are the delivery-pipeline synthesizer for Oyatie. ";
synth_prompt += "Produce a RANKED improvement plan from dual-confirmed findings only.\n\n";
synth_prompt += "Authority: " + authority + "\n";
synth_prompt += "Merge gate: " + merge_gate + "\n\n";
synth_prompt += "Confirmed findings (JSON):\n" + json_encode(confirmed) + "\n\n";
synth_prompt += "Methodology constraints (Bun-inspired):\n";
synth_prompt += "1) Prefer process/gate fixes that prevent recurrence over one-off code patches.\n";
synth_prompt += "2) Never propose reviving CLI merge authority or dual required contexts.\n";
synth_prompt += "3) Every action needs a verify step that fails closed (test, gate, or live check).\n";
synth_prompt += "4) Do not skip or weaken tests to go green.\n";
synth_prompt += "5) Keep blast radius explicit (cell/worktree/gate scope).\n";
synth_prompt += "6) process_edits: changes to agent prompts, registration meta-tests, or workflow rules that stop the class of mistake.\n";
synth_prompt += "7) do_not: anti-patterns (hand-edit *.generated.json, bypass oya-ci-required, etc.).\n";
synth_prompt += "8) At most 10 ranked_actions; order by delivery risk then effort.\n";
synth_prompt += "You may re-read files with tools if a finding citation needs clarification.\n";

let plan_r = agent(synth_prompt, #{
    label: "synthesize",
    capability_mode: "read-only",
    output_schema: plan_schema,
});

let plan_obj = #{};
let summary = "dual-confirmed findings without synthesized plan";
if plan_r != () && plan_r.success && plan_r.output != () {
    plan_obj = plan_r.output;
    if plan_r.output.summary != () {
        summary = plan_r.output.summary;
    }
} else {
    log("synthesize agent failed — returning confirmed findings without plan");
}

// ── Report artifact ───────────────────────────────────────────────────────

let report = "";
report += "# Delivery pipeline improvement report\n\n";
report += "Generated by Grok workflow `delivery-pipeline-improve` ";
report += "(Bun multi-agent methodology: contract → discover → dual adversarial → synthesize).\n\n";
report += "## Contract\n\n";
report += "- **Authority:** " + authority + "\n";
report += "- **Merge gate:** " + merge_gate + "\n";
report += "- **Focus:** " + focus + "\n\n";
report += "## Confirmed findings (" + confirmed.len().to_string() + ")\n\n";

let ri = 0;
for c in confirmed {
    let f = c.finding;
    report += "### " + f.id + " — " + f.severity + " / " + f.class + "\n\n";
    report += "- **Surface:** " + f.surface + "\n";
    report += "- **Issue:** " + f.issue + "\n";
    report += "- **Evidence path:** `" + f.evidence_path + "`\n";
    report += "- **Fix class:** " + c.fix_class + "\n";
    report += "- **Reviewer A:** " + c.evidence_a + "\n";
    report += "- **Reviewer B:** " + c.evidence_b + "\n\n";
    ri += 1;
}

report += "## Ranked actions\n\n";
if plan_obj.ranked_actions != () {
    for a in plan_obj.ranked_actions {
        report += a.rank.to_string() + ". **[" + a.priority + "] " + a.title + "**\n";
        report += "   - Why: " + a.why + "\n";
        report += "   - How: " + a.how + "\n";
        report += "   - Verify: " + a.verify + "\n";
        report += "   - Blast radius: " + a.blast_radius + "\n\n";
    }
} else {
    report += "_Synthesizer did not return ranked_actions._\n\n";
}

if plan_obj.verification_gate != () {
    report += "## Verification gate\n\n" + plan_obj.verification_gate + "\n\n";
}
if plan_obj.process_edits != () {
    report += "## Process edits (fix the loop, not only the symptom)\n\n";
    for pe in plan_obj.process_edits {
        report += "- " + pe + "\n";
    }
    report += "\n";
}
if plan_obj.do_not != () {
    report += "## Do not\n\n";
    for dn in plan_obj.do_not {
        report += "- " + dn + "\n";
    }
    report += "\n";
}

report += "## Dismissed / failed confirmation: " + dismissed.len().to_string() + "\n";

let path = write_scratch_file("delivery-pipeline-report.md", report);

complete(#{
    summary: summary,
    path: path,
    confirmed_count: confirmed.len(),
    dismissed_count: dismissed.len(),
    confirmed: confirmed,
    plan: plan_obj,
});
