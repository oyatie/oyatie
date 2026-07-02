// Minimal canonical substrate fixture. ADR-0064 requires the shared base to
// remain pack-neutral; locale-specific controls are supplied by the regional
// pack overlay rather than hard-coded here.

pub fn canonical_residency_intent() -> &'static str {
    "tenant data stays inside the residency class selected by the bound regional pack"
}
