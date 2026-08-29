//! The foundation aggregate and its construction.

use crate::*;

#[derive(Clone, Debug)]
pub struct Foundation {
    pub(crate) tenants: BTreeMap<String, Tenant>,
    pub(crate) tenant_policies: BTreeMap<String, TenantPolicy>,
    pub(crate) users: BTreeMap<(String, String), User>,
    pub(crate) capabilities: CapabilityRegistry,
    pub(crate) regional_packs: BTreeMap<String, RegionalPack>,
    pub(crate) object_entities: BTreeMap<(String, String), ObjectEntity>,
    pub(crate) outbox: Outbox,
    pub(crate) consent_scopes: BTreeMap<String, ConsentScope>,
    pub(crate) policies: PolicySet,
    pub(crate) eval_gate: EvalGate,
    pub(crate) cost_budgets: BudgetLedger,
    pub(crate) foundation_bypass_ledger: BypassLedger,
    pub(crate) foundry_runs: RunLedger,
    pub(crate) foundry_steps: StepLedger,
    pub(crate) foundry_evidence: EvidenceChain,
    pub(crate) mcp_rate_limiter: McpRateLimiter,
    pub(crate) cells: CellRouter,
    pub(crate) audit_chain: AuditChain,
    pub(crate) observability: FoundationObservability,
}

impl Default for Foundation {
    fn default() -> Self {
        Self {
            tenants: BTreeMap::new(),
            tenant_policies: BTreeMap::new(),
            users: BTreeMap::new(),
            capabilities: CapabilityRegistry::default(),
            regional_packs: BTreeMap::new(),
            object_entities: BTreeMap::new(),
            outbox: Outbox::default(),
            consent_scopes: BTreeMap::new(),
            policies: PolicySet::default(),
            eval_gate: EvalGate::default(),
            cost_budgets: BudgetLedger::default(),
            foundation_bypass_ledger: BypassLedger::default(),
            foundry_runs: RunLedger::default(),
            foundry_steps: StepLedger::default(),
            foundry_evidence: EvidenceChain::default(),
            mcp_rate_limiter: McpRateLimiter::default(),
            cells: CellRouter::default(),
            audit_chain: AuditChain::multi_tenant_shards(),
            observability: FoundationObservability::default(),
        }
    }
}
