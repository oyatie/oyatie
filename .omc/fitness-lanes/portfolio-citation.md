# Fitness Lane: portfolio-citation

- purpose: Verify bidirectional cross-citations between `bominal/` and `oyatie/` PRDs.
- enforces: Directive A1 (MASTERPLAN) — bidirectional bominal<->oyatie PRD cite plus P3.5 Foundry corpus cross-cite.
- kernel_crate: `oya-governance-portfolio-citation-kernel` — `CitationBlock { target_path, role, anchor }`, verdicts `PortfolioCitationVerdict { oyatie_cites_bominal, bominal_cites_oyatie, citations_checked }` and `FoundryCorpusCitationVerdict { required_sources_present, required_sources_total, missing_sources }`.
- runner_path: `tools/oya-foundry-fitness-portfolio-citation`
- inputs: `../bominal/docs/consolidated/PRD.md` (or `BOMINAL_PRD_PATH` / `--bominal-prd`), `docs/PRD.md` (or `OYATIE_PRD_PATH` / `--oyatie-prd`), and `docs/products/foundry/PRD.md` (or `FOUNDRY_PRD_PATH` / `--foundry-prd`).
- failure_modes:
  - oyatie PRD references a bominal PRD that does not reciprocate
  - bominal PRD missing oyatie back-cite
  - cited PRD path unresolved
  - Foundry PRD missing one of the KEEP-classified Bominal foundry corpus source citations
- ci_invocation: `cargo run -p oya-foundry-fitness-portfolio-citation`
- runtime_budget: 800 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct CitationBlock {
    pub target_path: String,       // data_class: INTERNAL_ONLY
    pub role: CitationRole,        // data_class: INTERNAL_ONLY
    pub anchor: Option<String>,    // data_class: INTERNAL_ONLY
}

pub const REQUIRED_FOUNDRY_CORPUS_SOURCES: [&str; 3] = [/* Bominal Foundry corpus source paths */];

pub enum CitationRole { PortfolioParent, CanonicalImplHome, FoundryCorpusSource }

pub struct PortfolioCitationVerdict {
    pub oyatie_cites_bominal: bool, // data_class: INTERNAL_ONLY
    pub bominal_cites_oyatie: bool, // data_class: INTERNAL_ONLY
    pub citations_checked: usize,   // data_class: INTERNAL_ONLY
}

pub fn verify(
    oyatie_prd_citations: &[CitationBlock],
    bominal_prd_citations: &[CitationBlock],
) -> PortfolioCitationVerdict { /* role-specific bidirectional check */ }

pub fn verify_foundry_corpus(
    foundry_prd_citations: &[CitationBlock],
) -> FoundryCorpusCitationVerdict { /* required Foundry source corpus coverage */ }
```
