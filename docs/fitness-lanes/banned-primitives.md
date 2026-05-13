# Fitness Lane: banned-primitives

- status: Accepted
- date: 2026-05-12
- purpose: Verify agent-instruction sections only invoke sanctioned primitives (grit, icm) and any direct git/gh usage carries a documented-rationale via icm.
- enforces: Directive 12 (MASTERPLAN) — sanctioned-primitives policy + ADR-0054 + ADR-0055.
- adr_citations: ADR-0053 (sanctioned primitives — defines {grit, icm, oya-tooling-agent-read} as the closed set; direct git/gh requires icm-documented rationale)
- kernel_crate: `oya-foundry-fitness-banned-primitives-kernel` — `PrimitiveUsage { path, line, primitive, has_icm_rationale }`, verdict `BannedPrimitivesFitnessReport { usages_checked }`.
- runner_path: `tools/oya-foundry-fitness-banned-primitives`
- inputs: `docs/agents/**/*.md`, `AGENTS.md`, `.omc/sessions/**/*.md`, icm rationale store dump.
- failure_modes:
  - agent doc says `Run: git commit ...` with no icm rationale id
  - agent doc invokes `gh pr merge` (always banned)
  - rationale id not in icm store
- ci_invocation: `cargo run -p oya-foundry-fitness-banned-primitives`
- runtime_budget: 500 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct PrimitiveUsage {
    pub path: String,                // data_class: INTERNAL_ONLY
    pub line: u32,                   // data_class: INTERNAL_ONLY
    pub primitive: String,           // data_class: INTERNAL_ONLY
    pub icm_rationale: Option<String>, // data_class: INTERNAL_ONLY
}

pub struct BannedPrimitivesFitnessReport { pub usages_checked: usize }

pub enum BannedPrimitivesFitnessError {
    HardBannedPrimitive { path: String, line: u32, primitive: String },
    UnjustifiedDirectGit { path: String, line: u32 },
    UnknownRationale { path: String, rationale: String },
}

pub fn validate_banned_primitives_fitness(
    usages: &[PrimitiveUsage],
    hard_banned: &[String],
    known_rationales: &[String],
) -> Result<BannedPrimitivesFitnessReport, BannedPrimitivesFitnessError> {
    let banned: std::collections::BTreeSet<&str> = hard_banned.iter().map(|s| s.as_str()).collect();
    let rats: std::collections::BTreeSet<&str> = known_rationales.iter().map(|s| s.as_str()).collect();
    for u in usages {
        if banned.contains(u.primitive.as_str()) {
            return Err(BannedPrimitivesFitnessError::HardBannedPrimitive {
                path: u.path.clone(), line: u.line, primitive: u.primitive.clone(),
            });
        }
        if u.primitive == "git" || u.primitive == "gh" {
            let r = u.icm_rationale.as_ref().ok_or_else(|| BannedPrimitivesFitnessError::UnjustifiedDirectGit {
                path: u.path.clone(), line: u.line,
            })?;
            if !rats.contains(r.as_str()) {
                return Err(BannedPrimitivesFitnessError::UnknownRationale {
                    path: u.path.clone(), rationale: r.clone(),
                });
            }
        }
    }
    Ok(BannedPrimitivesFitnessReport { usages_checked: usages.len() })
}
```
