import json, os
from collections import Counter, defaultdict

out_path = "/Users/jasonlee/oyatie/registry/stub-audit/2026-05-17/standards-and-plans.jsonl"
os.makedirs(os.path.dirname(out_path), exist_ok=True)

findings = []

def emit(file, section, line, pattern, snippet, severity, fix_hint):
    findings.append({
        "file": file,
        "section": section,
        "line": line,
        "pattern": pattern,
        "snippet": snippet[:80],
        "severity": severity,
        "fix_hint": fix_hint[:120]
    })

# ── PATTERN 3: Standards referencing lanes that don't exist in .github/workflows/ ──
MISSING_LANES = [
    ("docs/standards/agent-instructions-discipline.md", "§3 Banned-token grep scope", 62, "oya-governance-agent-instructions-fence"),
    ("docs/standards/agent-instructions-discipline.md", "§4 Dual-audience requirement", 125, "oya-governance-dual-audience"),
    ("docs/standards/agent-instructions-discipline.md", "§3 Banned-token grep scope", 85, "oya-governance-banned-primitives"),
    ("docs/standards/autonomy-ceiling.md", "§2 The capability record", 90, "oya-governance-capability-publish"),
    ("docs/standards/autonomy-ceiling.md", "§5 Tier-uplift PR shape", 165, "oya-governance-autonomy-ceiling"),
    ("docs/standards/clean-architecture.md", "§3 Dependency-direction enforcement", 159, "oya-governance-architecture-conventions"),
    ("docs/standards/crate-naming-convention.md", "frontmatter enforced_by", 19, "oya-governance-naming-convention"),
    ("docs/standards/code-style-rust.md", "§2 Workspace lint inheritance", 22, "oya-governance-clippy-pedantic"),
    ("docs/standards/code-style-rust.md", "§4 unsafe policy", 133, "oya-governance-unsafe-kani"),
    ("docs/standards/error-handling.md", "§1 The boundary rule", 59, "oya-governance-error-boundary"),
    ("docs/standards/error-handling.md", "§4 No unwrap()", 117, "oya-governance-no-unwrap-prod"),
    ("docs/standards/error-handling.md", "§6 Silent-failure prevention", 176, "oya-governance-silent-failure"),
    ("docs/standards/error-handling.md", "§7 Audit-chain integration", 192, "oya-governance-audit-emission"),
    ("docs/standards/testing.md", "frontmatter enforced_by", 23, "oya-governance-test-evidence"),
    ("docs/standards/data-class.md", "§1 The class taxonomy", 60, "oya-governance-data-class"),
    ("docs/standards/data-class.md", "§5 DSR cascade integration", 155, "oya-governance-dsr-cascade"),
    ("docs/standards/dependency-policy.md", "§1 LTS pinning", 55, "oya-governance-lts-dependency"),
    ("docs/standards/dependency-policy.md", "§4 Renovate baseline", 146, "oya-governance-renovate-config"),
    ("docs/standards/image-discipline.md", "§2 Forbidden bases", 73, "oya-governance-container-base"),
    ("docs/standards/image-discipline.md", "§5 Image-size budget", 140, "oya-governance-image-discipline"),
    ("docs/standards/observability.md", "§1 OpenTelemetry mandatory", 60, "oya-governance-otel-emit"),
    ("docs/standards/on-call.md", "frontmatter enforced_by", 20, "oya-governance-runbook-index-resolves"),
    ("docs/standards/release-management.md", "frontmatter enforced_by", 20, "oya-governance-flag-debt"),
    ("docs/standards/release-management.md", "§1 Trunk-based development", 52, "oya-governance-branch-age"),
    ("docs/standards/multi-agent-tool-map.md", "frontmatter enforced_by", 22, "oya-governance-tool-map-cohesion"),
    ("docs/standards/git-workflow.md", "§4 Revised lane semantics", 98, "oya-governance-banned-primitives"),
    ("docs/standards/git-workflow.md", "§5 Migration-candidate flow", 125, "oya-governance-direct-tool-rationale"),
    ("docs/standards/claude-code-harness.md", "§8 Boundaries", 200, "oya-governance-user-machine-guard"),
    ("docs/standards/claude-code-harness.md", "§5 Active hooks", 142, "oya-governance-hook-self-test"),
]

for (file, sec, line, lane) in MISSING_LANES:
    emit(file, sec, line, "3",
         "enforced_by: " + lane + " — no .github/workflows/ file exists",
         "critical",
         "Create .github/workflows/" + lane + ".yml or wire into existing aggregator workflow")

# ── PATTERN 4: Standards referencing oya-check-* kernels missing from tree ──
MISSING_KERNELS = [
    ("docs/standards/m02-exit-gate-validators.md", "§ The 14 lanes", 34, "oya-check-foundation-bypass"),
    ("docs/standards/m02-exit-gate-validators.md", "§ The 14 lanes", 36, "oya-check-audit-chain-replay"),
    ("docs/standards/m02-exit-gate-validators.md", "§ The 14 lanes", 37, "oya-check-foundry-capability-schema"),
    ("docs/standards/m02-exit-gate-validators.md", "§ The 14 lanes", 38, "oya-check-foundry-eval"),
    ("docs/standards/m02-exit-gate-validators.md", "§ The 14 lanes", 39, "oya-check-cross-tenant-access-fuzz"),
    ("docs/standards/m02-exit-gate-validators.md", "§ The 14 lanes", 40, "oya-check-api-semver"),
    ("docs/standards/m02-exit-gate-validators.md", "§ The 14 lanes", 41, "oya-check-documentation"),
    ("docs/standards/autonomy-ceiling.md", "§8 Eval-set requirements", 215, "oya-check-foundry-eval (eval lane kernel missing)"),
    ("docs/standards/data-class.md", "§8 Cross-tenant isolation", 184, "oya-check-cross-tenant-access-fuzz (nightly kernel missing)"),
]

for (file, sec, line, crate) in MISSING_KERNELS:
    emit(file, sec, line, "4",
         "kernel crate " + crate[:55] + " not on tree",
         "critical",
         "Scaffold crates/" + crate.split()[0] + "/src/lib.rs — lane listed as wired but kernel absent is OP-11 violation")

# ── PATTERN 6: Master-plan items with status: stub or scaffolded ──
STUB_PHASES = [
    "P06-distroless-lts-image status: stub",
    "M01-P09 P09-doc-automation-freshness status: stub",
    "M01-P10 P10-purpose-orphan-detection status: stub",
    "M01-P11 P11-agentic-navigability status: stub",
    "M01-P12 P12-provider-agnosticism status: stub",
    "M01-P13 P13-distroless-lts-image status: stub",
    "M01-P14 P14-hyperscaler-practices status: stub",
]
for snippet in STUB_PHASES:
    emit("specs/masterplan.json", "live_implementation_index.milestones[0].phases", 0, "6",
         snippet, "critical",
         "Phase must be implemented or explicitly deferred with named exit-gate blocker per OP-11")

STUB_IPS = [
    ("M01-P08-IP-001", "M01-P08 adr-cutover IP-001 status: stub"),
    ("M01-P08-IP-002", "M01-P08 invent IP-002 status: stub"),
    ("M01-P08-IP-003", "M01-P08 oya-tooling IP-003 status: stub"),
    ("M01-P08-IP-004", "M01-P08 bidirectional IP-004 status: stub"),
    ("M01-P08-IP-005", "M01-P08 foundry IP-005 status: stub"),
    ("M01-P08-IP-006", "M01-P08 agent-fence IP-006 status: stub"),
    ("M01-P08-IP-007", "M01-P08 hook-self-test IP-007 status: stub"),
    ("M01-P09-IP-001", "M01-P09 mdbook IP-001 status: stub"),
    ("M01-P09-IP-002", "M01-P09 doc-freshness IP-002 status: stub"),
    ("M01-P10-IP-001", "M01-P10 purpose-orphan IP-001 status: stub"),
]
for (ip_id, snippet) in STUB_IPS:
    emit("specs/masterplan.json", "live_implementation_index " + ip_id, 0, "6",
         snippet, "high",
         "Implement IP or mark blocked with evidence — stub IPs block oya-governance-master-plan-completion lane")

# ── PATTERN 9: forbidden_primitives with no CI guard ──
for primitive in ["git", "gh", "manual-branch", "manual-rebase", "manual-merge", "manual-push"]:
    emit("specs/master-plan-sequencing.json", "forbidden_primitives", 0, "9",
         'forbidden_primitives "' + primitive + '" declared but no pre-commit/CI guard enforces it',
         "critical",
         "Add pre-commit hook or CI step grepping agent fences for undocumented `" + primitive + "` invocations")

# ── PATTERN 1: TBD / placeholder / draft language ──
emit("docs/standards/migration-playbook.md", "§3 Per-migration phases", 43, "1",
     "per `templates/migration-runbook-template.md` (planned)",
     "medium",
     "Replace '(planned)' with actual file path — no stub references in normative docs per OP-11")

emit("docs/standards/migration-playbook.md", "§3 Pre-flight", 50, "1",
     "per `checklists/tenant-onboarding.md` (planned)",
     "medium",
     "Create checklists/tenant-onboarding.md — normative flow references non-existent file")

emit("docs/standards/code-review.md", "§ Status header", 8, "1",
     "**Status:** Draft v0.1 — 2026-05-09",
     "high",
     "Promote to Accepted — Draft status in a normative standards doc violates OP-11")

emit("docs/standards/api-design.md", "§ Status header", 8, "1",
     "**Status:** Draft v0.1 — 2026-05-09",
     "high",
     "Promote to Accepted — Draft status in a normative standards doc violates OP-11")

emit("docs/standards/m02-exit-gate-validators.md", "§ Status header", 9, "1",
     "Remaining lanes wired in CLI but lack integration tests; follow-on PRs add one each",
     "critical",
     "12 of 14 exit-gate lanes lack tests — BLOCKER flip explicitly deferred; direct OP-11 contradiction")

emit("docs/standards/logging-tracing.md", "§8 Forwarding + storage", 70, "1",
     "In-house Leptos UI for visualization (long-horizon)",
     "medium",
     "Move 'long-horizon' plan to ROADMAP.md — normative standard must describe current required state only")

# ── PATTERN 5: Aspirational / guidance without normative force ──
emit("docs/standards/clean-architecture.md", "§5 Testing posture per layer", 294, "5",
     "These rules are advisory inside crates but are checked at lane time",
     "high",
     "Remove 'advisory' qualifier — rules either have lane enforcement or are not normative per OP-11")

emit("docs/standards/image-discipline.md", "§8 SLSA L2 provenance", 199, "5",
     "SLSA L3 is the next milestone after GitHub Actions reusable workflow lands per ADR-SUP-002",
     "low",
     "Move roadmap statements to ROADMAP.md — normative standard should only describe current requirements")

emit("docs/standards/INDEX.md", "§ Forward-reference resolution map", 78, "5",
     "prevention-doctrine.md — deferred — see §Out-of-scope",
     "medium",
     "Resolve deferred forward-reference — OP-11 requires complete normative surface without deferrals")

# ── PATTERN 7: exit_gate references with no CI lane trace ──
emit("specs/masterplan.json", "milestones[0].phases exit_gate fields", 0, "7",
     "Phase exit_gate fields present but no corresponding .github/workflows/ job enforces them",
     "critical",
     "Map every exit_gate to an active CI lane by name; create workflow or cite existing enforcer")

emit("docs/MASTERPLAN.md", "§1 Vision — milestone gate", 58, "7",
     "Bominal Proof Ladder L0..L7 + 9 architecture planes green at every milestone gate",
     "high",
     "No CI lane enforces Proof Ladder gate — create oya-governance-proof-ladder lane or cite enforcer")

# ── PATTERN 2: enforcement_mode effectively cultural (no mechanical block) ──
emit("docs/standards/git-workflow.md", "§4 Revised lane semantics", 107, "2",
     "git outside any fence (plain prose, human-facing) — PASS (advisory only)",
     "critical",
     "Human git/gh outside fences is advisory — add pre-commit grep or CI gate per OP-11 mechanical requirement")

emit("docs/standards/claude-code-harness.md", "§3 grit claim->work->done lifecycle", 114, "2",
     "grit done MUST run AFTER cargo nextest ... are green locally",
     "high",
     "Local pre-grit-done rule is cultural (no hook enforces it) — wire pre-push hook to mechanically enforce")

# ── PATTERN 8: Master-plan IPs listed but IP files may be missing from .omc/plans/ ──
emit("specs/masterplan.json", "live_implementation_index P01-agentic-pipeline-cutover IPs", 0, "8",
     "IPs reference phases/P01-agentic-pipeline-cutover/* paths — file existence unverified",
     "high",
     "Verify .omc/plans/milestones/M01-foundation/phases/P01*/IP-*.md exists; scaffold if absent per OP-11")

# Write JSONL
with open(out_path, "w") as out:
    for finding in findings:
        out.write(json.dumps(finding) + "\n")

print("Wrote", len(findings), "findings to", out_path)

critical = sum(1 for x in findings if x["severity"] == "critical")
high     = sum(1 for x in findings if x["severity"] == "high")
medium   = sum(1 for x in findings if x["severity"] == "medium")
low      = sum(1 for x in findings if x["severity"] == "low")
print("  critical=%d  high=%d  medium=%d  low=%d  total=%d" % (critical, high, medium, low, len(findings)))

pat_counts = Counter(x["pattern"] for x in findings)
print("  By pattern:", dict(sorted(pat_counts.items())))

file_score = defaultdict(int)
for x in findings:
    if x["severity"] in ("critical", "high"):
        file_score[x["file"]] += 1
top5 = sorted(file_score.items(), key=lambda kv: -kv[1])[:5]
print("  Top 5 worst files (critical+high):", top5)
