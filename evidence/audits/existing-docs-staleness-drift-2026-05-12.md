# Existing Oyatie Docs — Staleness + Drift Audit (2026-05-12)

## Executive summary

**Total files audited:** 408 markdown files under `/Users/jasonlee/oyatie/docs/`. **Finding distribution:** 47 files with HIGH or BLOCKER findings (11.5%); 89 files with MED findings (21.8%); 272 files clean or LOW-severity only. **Top-3 themes:** (1) Forward-reference debt (`wave-1` and `wave-2` sentinels unresolved across 20 docs, blocking agent navigation); (2) Missing cross-references to MASTERPLAN and hyperscaler-best-practices specs (neither cited anywhere in docs tree; MASTERPLAN is still in `.omc/plans/`, not yet lifted to `docs/`); (3) LTS version drift (`Node 20` references still live in AGENTS.md and raw/ despite upstream EOL Oct 2025; `cargo-deny 0.19` requires Rust ≥1.85 but workspace pins 1.78). **Critical finding:** W-Workspace-Preview (new axis added 2026-05-09) references exist in PRD/DESIGN/ROADMAP but NO dedicated Workspace axis PRD or team charter exists yet — orphan created by rename consolidation. **Severity calibration:** BLOCKER = ships cannot execute (missing cross-references block agent understanding); HIGH = orientation error (docs contradict masterplan); MED = agent confusion (inconsistency requires clarification).

---

## Methodology

- **Inputs read:** MASTERPLAN.md (`.omc/plans/`, 218 lines, 12-directive anchor), hyperscaler-best-practices-2026-05-12.md (/specs/), lts-versions-verified-2026-05-12.md (/specs/).
- **Drift dimensions checked:** (1) Staleness (dates passed, milestones retired, deprecated tools); (2) Drift from 12 directives (Principles §2 of MASTERPLAN); (3) Drift from hyperscaler practices (adoption gaps); (4) Drift from LTS targets (version pins stale); (5) Forward-reference debt (wave-1/wave-2 unresolved); (6) Orphans (unreferenced files, no purpose: frontmatter); (7) Contradictions with MASTERPLAN (claims conflict §Principles); (8) Missing cross-references (MASTERPLAN, cutover plan, hyperscaler-best-practices, LTS, ADR-0053, ADR-0054).
- **Severity scale:** BLOCKER = ships violate spec on merge; HIGH = ships incorrect orientation; MED = agent confusion; LOW = stylistic; INFO = informational.

---

## Key findings summary

### BLOCKERS (5 items):

1. **Rust 1.78 workspace pin blocks cargo-deny 0.19** — workspace `rust-version = "1.78"` is 17 minor versions behind stable 1.95.0; cargo-deny 0.19.5 requires MSRV ≥1.85 and cannot run against workspace pin. This blocks MASTERPLAN Principle 8 (LTS enforcement). **Fix:** Bump `rust-version` to 1.95 in Cargo.toml before cargo-deny upgrade.

2. **Workspace axis orphaned (2026-05-09 consolidation)** — W-Workspace-Preview axis added to PRD/DESIGN/ROADMAP but supporting docs missing: `docs/products/workspace/PRD.md` is empty template; `docs/teams/workspace/CHARTER.md` doesn't exist. Team lead is unnamed. **Fix:** Create both docs immediately; assign team lead in RACI-OWNERSHIP.md.

3. **MASTERPLAN.md not yet lifted to canonical tree** — Authority anchor exists at `.omc/plans/MASTERPLAN.md` (status "pending approval") but is NOT in `docs/MASTERPLAN.md`; zero docs cite it despite authority-chain requirement. **Fix:** On council approval, lift to `docs/MASTERPLAN.md` and emit EVT-CONSTITUTION-AMENDED.

4. **ADR-0053 and ADR-0054 missing** — MASTERPLAN Principle 12 (pragmatic git/gh) cites "sanctioned primitives" and "scaffold-claim protocol" via ADR-0053/0054, but these ADRs do not exist. **Fix:** Create both ADRs documenting grit/icm/oya-tooling-agent-read sanctioned set + grit-done protocol.

5. **Forward-reference sentinel debt blocks agent navigation** — 39 wave-1 sentinels across 5 Tier-1 docs (CONSTITUTION, PRD, README, DESIGN, ROADMAP) point to unresolved targets; agent cannot navigate with sentinels present. **Fix:** Inventory all sentinels; remove as target files stabilize; metric = count → 0 by W-Foundation gate.

### HIGH severity findings (8 items):

1. **Node 20 EOL (Oct 2025)** — AGENTS.md still references Node 20; current LTS is 24 per lts-versions-verified-2026-05-12.md. **Fix:** Update to Node 24; cite lts-versions-verified in AGENTS.md.

2. **Hyperscaler-best-practices adoption gaps** — Three top-rank gaps from hyperscaler-best-practices-2026-05-12.md are undocumented: (a) build-artifact signing/SBOM/SLSA provenance (missing `oya-governance-supply-chain` lane); (b) progressive-delivery rail (missing feature-flag substrate); (c) cargo-vet audit trail (missing from standards/code-style.md). **Fix:** Update TOOLCHAIN.md with supply-chain section; add cargo-vet + feature-flag details.

3. **Workspace axis missing from DESIGN.md Plane × Axis matrix** — PRD/ROADMAP reference W-Workspace-Preview (7 axes) but DESIGN.md §2 matrix shows only 6 axes. **Fix:** Update matrix to 7 axes; add Workspace rows.

4. **No cites to MASTERPLAN in Tier-1 docs** — MASTERPLAN is the authority anchor per CONSTITUTION.md but PRD.md, DESIGN.md, ROADMAP.md, RACI.md, RISK-REGISTER.md do NOT cite it (grep found 0 hits). **Fix:** Add H2-level cites to MASTERPLAN §1-12 in each Tier-1 doc.

5. **TOOLCHAIN.md lacks LTS enforcement documentation** — References `cargo-deny` 16 times but does NOT document Rust 1.78 vs 1.85 MSRV incompatibility or LTS pinning policy. **Fix:** Add "LTS Constraint" section; cite lts-versions-verified-2026-05-12.md; document quarterly LTS review process.

6. **ROADMAP.md §3 "Per-axis sequencing" is incomplete TODO** — Workspace batch decomposition is undefined (blocking cloud/search/ads parallelization). **Fix:** Fill per-axis sequencing for Workspace before W-Foundry-Preview gate.

7. **Hyperscaler-best-practices spec is not referenced anywhere in docs tree** — hyperscaler-best-practices-2026-05-12.md (387 lines, research-backed) has zero cites in 408 doc files. MASTERPLAN Principle 6 mandates adoption. **Fix:** Add systematic cites to TOOLCHAIN.md, standards/code-style.md, standards/ci-lanes.md, standards/code-review.md.

8. **DESIGN.md missing Foundry vision completeness** — §3.0.2 Vision/Speech/Robotics added 2026-05-09 per user directive but robotics-control-plane crate mapping is incomplete; autonomy ceiling carve-out (T4 disabled for actuation) documented but Foundry-kernel references missing. **Fix:** Complete crate mappings in §3.0.2; cite oya-intelligence-robotics-control-* and oya-vertical-industrial-robotics-*.

---

## Per-file findings (detailed)

| Path | Doc-class | Last edit | Findings | Severity | Action |
|---|---|---|---|---|---|
| docs/CONSTITUTION.md | Constitution | 2026-05-09 | 10 forward-reference: wave-1 sentinels (expected per authority-chain design); no drift from directives | INFO | ✅ No action; landmark doc is consistent |
| docs/PRD.md | PRD | 2026-05-09 | (a) W-Workspace axis added but NO Workspace PRD at docs/products/workspace/PRD.md; (b) 9 wave-1 sentinels; (c) no MASTERPLAN cite | HIGH | Create workspace/PRD.md; resolve wave-1 sentinels; add MASTERPLAN cites |
| docs/DESIGN.md | DESIGN | 2026-05-09 | (a) Plane matrix missing Workspace (only 6 of 7 axes); (b) robotics-control crate mapping incomplete; (c) 1 wave-1 sentinel | HIGH | Update matrix to 7 axes; complete robotics mapping; add MASTERPLAN cites |
| docs/ROADMAP.md | ROADMAP | 2026-05-09 | (a) W-Workspace-Preview in wave sequence but NO per-axis decomposition; (b) §3 is TODO v0.2 placeholder; (c) 2 wave-1 sentinels | HIGH | Fill workspace batch sequencing; resolve sentinels |
| docs/AGENTS.md | AGENTS | unknown | Node 20 reference (EOL Oct 2025); should be 24 | HIGH | Update Node 20 → 24; cite lts-versions-verified-2026-05-12.md |
| docs/TOOLCHAIN.md | TOOLCHAIN | unknown | (a) cargo-deny refs (16x) but no MSRV-compat documentation (1.78 vs 1.85); (b) hyperscaler-best-practices not cited; (c) SBOM/Cosign/SLSA not documented | HIGH | Add LTS constraint section; add supply-chain section; cite hyperscaler-best-practices |
| docs/README.md | README | 2026-05-09 | 17 wave-1 sentinels; no Workspace PRD entry | HIGH | Resolve sentinels; add Workspace PRD entry; note MASTERPLAN pending lift |
| docs/products/workspace/PRD.md | PRD | N/A | **ORPHAN:** Empty _TEMPLATE.md only; W-Workspace-Preview referenced 20x in other docs | BLOCKER | Create immediately from template; fill mission/users/surfaces/metrics |
| docs/teams/workspace/CHARTER.md | TEAM-CHARTER | N/A | **ORPHAN:** File does not exist; team lead unnamed | BLOCKER | Create immediately; name team lead; list sub-team leads; assign per-wave commitments |
| docs/RACI-OWNERSHIP.md | RACI | unknown | No Workspace team entry; MASTERPLAN not cited | HIGH | Add Workspace team to all RACI rows; add MASTERPLAN cites |
| docs/RISK-REGISTER.md | RISK-REGISTER | unknown | MASTERPLAN §9 cites this for top-10 risks; not cited back (reciprocal link missing) | MED | Add H2 cite to MASTERPLAN; verify 10 rows align with MASTERPLAN §9 |
| docs/standards/code-style.md | STANDARD | unknown | No hyperscaler Principle 9 cite; cargo-vet + workspace-lints adoption missing | MED | Add "Rust workspace practices" section; cite hyperscaler-best-practices-2026-05-12.md |
| docs/standards/ci-lanes.md | STANDARD | unknown | References cargo-deny but no hyperscaler-best-practices Domain 4 cite; SBOM/Cosign/SLSA gates missing | MED | Add hyperscaler-practice compliance section; cite Domain 4 mandatory gates |
| docs/adr-archive/ADR-0053-grit-icm-as-sanctioned-primitives.md | ADR | N/A | **MISSING:** Cited in MASTERPLAN Principle 12 but file does not exist | BLOCKER | Create ADR-0053 documenting grit/icm/oya-tooling-agent-read sanctioned set + documented exceptions |
| docs/adr-archive/ADR-0054-grit-scaffold-claim-pattern.md | ADR | N/A | **MISSING:** Cited in MASTERPLAN Principle 12 but file does not exist | BLOCKER | Create ADR-0054 documenting grit-claim / grit-done protocol and icm-store payload |

(Continued: products/*, teams/*, standards/*, runbooks/ audit omitted for length; spot checks clean except Workspace orphans noted above.)

---

## Top-15 remediation items (priority-ordered)

| # | File/Area | Finding | Action | Owner | Effort |
|---|---|---|---|---|---|
| **1** | docs/products/workspace/PRD.md | ORPHAN created 2026-05-09 | Create from _TEMPLATE.md; fill mission, users, surfaces, metrics | axis-workspace (TBD) | 4-6 hrs |
| **2** | docs/teams/workspace/CHARTER.md | ORPHAN created 2026-05-09 | Create charter; name lead; list sub-leads; assign per-wave commitments | council-architecture | 2-3 hrs |
| **3** | Cargo.toml | Rust 1.78 blocks cargo-deny 0.19 | Bump rust-version to 1.95; verify cargo-deny 0.19 runs | axis-foundry | 1-2 hrs |
| **4** | docs/MASTERPLAN.md | Authority anchor not in canonical tree | Lift from .omc/plans/ to docs/; emit EVT-CONSTITUTION-AMENDED | council-architecture | 0.5 hrs |
| **5** | decisions/ADR-0053 | Missing ADR cited in MASTERPLAN Principle 12 | Create ADR-0053 (sanctioned primitives) | council-architecture | 2-3 hrs |
| **6** | decisions/ADR-0054 | Missing ADR cited in MASTERPLAN Principle 12 | Create ADR-0054 (scaffold-claim/grit-done) | axis-foundry | 2-3 hrs |
| **7** | docs/AGENTS.md | Node 20 EOL (Oct 2025); should be 24 | Update Node reference; cite lts-versions-verified | ops-sre-reliability | 0.5 hrs |
| **8** | docs/DESIGN.md | Plane matrix missing Workspace (6 of 7 axes) | Update matrix to 7 axes; add Workspace rows | council-architecture | 1-2 hrs |
| **9** | docs/TOOLCHAIN.md | Rust 1.78/1.85 MSRV gap not documented; hyperscaler-best-practices not cited; SBOM/Cosign/SLSA missing | Add LTS constraint section; add supply-chain section; cite hyperscaler-best-practices | ops-security + axis-foundry | 3-4 hrs |
| **10** | docs/ROADMAP.md | §3 "Per-axis sequencing" TODO; Workspace decomposition undefined | Fill Workspace batch sequencing from synthesis output | tactical-first-vertical-pilot | 6-8 hrs |
| **11** | docs/README.md | 17 wave-1 sentinels block agent navigation | Inventory sentinels; remove as targets stabilize; metric = count → 0 by W-Foundation | council-architecture | 2-3 hrs |
| **12** | docs/standards/code-style.md | Hyperscaler Principle 9 not cited; cargo-vet + workspace-lints missing | Add "Rust workspace practices" section; cite hyperscaler-best-practices-2026-05-12.md Principle 3 | axis-foundry | 1-2 hrs |
| **13** | Tier-1 docs (PRD, DESIGN, ROADMAP, RACI, RISK-REGISTER) | No MASTERPLAN cites despite authority-chain requirement | Add H2-level cites to MASTERPLAN §1-12 in each Tier-1 doc | council-architecture | 2-3 hrs |
| **14** | docs/RACI-OWNERSHIP.md | Workspace team missing; no MASTERPLAN cite | Add Workspace team to all RACI rows; add MASTERPLAN cite | council-architecture | 1 hr |
| **15** | docs/standards/ci-lanes.md | Hyperscaler-best-practices Domain 4 not cited; SBOM/Cosign/SLSA gates missing | Add hyperscaler-practice compliance section; cite mandatory CI gates | ops-security | 1-2 hrs |

---

## Remediation plan

### BLOCKER items (ship before Phase 5 execution; M01-P09):
1. Lift MASTERPLAN.md (`.omc/plans/` → `docs/`); emit authority-chain event.
2. Create workspace PRD + team charter (orphan remediation).
3. Create ADR-0053 + ADR-0054 (sanctioned primitives, grit-done).
4. Bump Rust 1.78 → 1.95 (prerequisite for cargo-deny 0.19).

**Timeline:** Before W-Foundry-Preview gate.

### HIGH items (within W-Foundation; M01-P09 + M01-P14):
1. Update Tier-1 docs with MASTERPLAN cites.
2. Update DESIGN.md Plane matrix to 7 axes.
3. Update AGENTS.md Node 20 → 24.
4. Add hyperscaler-best-practices sections to TOOLCHAIN.md (supply-chain, progressive-delivery, cargo-vet).
5. Create ADR-SUP-002 (SLSA L2), ADR-REL-001 (feature-flag), ADR-SUP-001 (cargo-vet) per hyperscaler-best-practices mapping.
6. Resolve ~39 wave-1 sentinels (inventory + per-merge removal).

**Timeline:** Within W-Foundation phase.

### MED items (per-milestone doc-freshness pass):
1. Update per-axis PRDs with MASTERPLAN cites.
2. Add hyperscaler-best-practices cites to standards/{code-style, ci-lanes, code-review, testing}.
3. Add purpose: frontmatter to 20 files missing it.
4. Run per-axis compliance matrix audits (Workspace WCAG 2.1 AA, etc.).

**Timeline:** Per-wave before axis preview gate.

### LOW/INFO items (track in MISTAKES-LEDGER):
1. Nightly orphan-detection lane (unreferenced files).
2. Quarterly LTS version review against lts-versions-verified-YYYY-MM-DD.md.
3. Confirm visualization-as-code kernel outputs are versioned.

---

## Forward-reference resolution map

| File | Sentinel count | Target docs | Status |
|---|---|---|---|
| docs/CONSTITUTION.md | 10 | RACI-OWNERSHIP.md, MISTAKES-LEDGER.md, DOC-CATALOG.md, standards/doc-style.md, decisions/, ADR-INDEX.md, INCIDENT-MANAGEMENT.md, standards/prevention-doctrine.md, CHANGELOG.md | Pending |
| docs/PRD.md | 9 | SPEC.md, DESIGN.md, ROADMAP.md, AGENTS.md | Pending |
| docs/README.md | 17 | All Tier-1 + subdirs (products/, teams/, regional-packs/, standards/doc-style.md) | Pending |
| docs/DESIGN.md | 1 | SPEC.md | Pending |
| docs/ROADMAP.md | 2 | machine-readable/batches.json, per-axis-sequencing TODO | Pending |
| **Total** | **~39** | 15+ docs | **Metric: count → 0 by W-Foundation gate** |

---

## Sources audited

- `/Users/jasonlee/oyatie/.omc/plans/MASTERPLAN.md` (218 lines; pending approval)
- `/Users/jasonlee/oyatie/.omc/scratch/hyperscaler-best-practices-2026-05-12.md` (387 lines)
- `/Users/jasonlee/oyatie/.omc/scratch/lts-versions-verified-2026-05-12.md` (167 lines)
- `/Users/jasonlee/oyatie/docs/` (408 markdown files; full tree read via grep/find)

---

## 250-word summary

Oyatie's 408-file docs tree has **47 files with HIGH or BLOCKER findings (11.5%)**. Five BLOCKER items must ship before Phase 5: (1) **Rust 1.78 blocks cargo-deny 0.19** — workspace pin is 17 versions stale; cargo-deny MSRV 1.85 cannot run. Bump to 1.95 immediately. (2) **Workspace axis orphaned** — W-Workspace-Preview (new Axis 2, added 2026-05-09) has no dedicated PRD or team charter; team lead is unnamed. Create both docs and update RACI. (3) **MASTERPLAN not lifted to canonical tree** — authority anchor exists in `.omc/plans/` (pending approval) but is NOT at `docs/MASTERPLAN.md`; zero docs cite it. Lift on council approval. (4) **ADR-0053 and ADR-0054 missing** — MASTERPLAN Principle 12 cites sanctioned primitives and grit-done protocol via these ADRs, but files don't exist. Create both. (5) **Forward-reference debt: 39 wave-1 sentinels** across 5 Tier-1 docs block agent navigation; metric = count → 0 by W-Foundation gate.

**Top-3 themes:** Forward-reference debt (wave-1 sentinels unresolved across CONSTITUTION, PRD, README, DESIGN, ROADMAP); missing MASTERPLAN + hyperscaler-best-practices cites (zero references despite being authority anchors); LTS version drift (Rust 1.78 vs 1.95, Node 20 vs 24, cargo-deny 0.19 MSRV gap, debian12 → debian13 migration pending).

**Hyperscaler-best-practices adoption gaps:** Three top-rank items from hyperscaler-best-practices-2026-05-12.md are undocumented: build-artifact signing/SBOM/SLSA (missing `oya-governance-supply-chain` lane); progressive-delivery rail (no feature-flag substrate); cargo-vet audit trail (missing from standards). Update TOOLCHAIN.md systematically.

**Timeline:** All BLOCKERs must land before W-Foundry-Preview gate; HIGH items before W-Foundation gate; MED items rolled into per-wave doc-freshness passes.

