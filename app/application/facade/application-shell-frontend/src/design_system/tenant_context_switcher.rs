//! DS-TENANT_CONTEXT_SWITCHER (`specs/design-system/tenant-context-switcher.json`).
//!
//! Switch personal/work/admin-audit tenant contexts without leaking cached
//! state across ownership pillars. Spec security invariants:
//!
//! 1. cache namespace includes `context_kind` AND `ownership_pillar`;
//! 2. a switch requires policy re-evaluation before rendering the child app
//!    (modeled as a type-state gate: [`PendingSwitch`] only becomes a
//!    renderable [`ActiveContext`] through [`PendingSwitch::grant`]);
//! 3. the admin-audit variant never appears in a personal context.

use std::collections::BTreeMap;

use leptos::prelude::*;

/// Spec `variants`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ContextKind {
    Personal,
    Work,
    AdminAudit,
}

impl ContextKind {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Personal => "personal",
            Self::Work => "work",
            Self::AdminAudit => "admin-audit",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Personal => "Personal",
            Self::Work => "Work",
            Self::AdminAudit => "Admin audit",
        }
    }
}

/// Ownership pillar the active principal is operating under.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum OwnershipPillar {
    Personal,
    Organization,
}

impl OwnershipPillar {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Personal => "personal",
            Self::Organization => "organization",
        }
    }
}

/// Spec `states`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SwitcherState {
    Default,
    Loading,
    PolicyDenied,
    Offline,
    LegalHoldWarning,
}

impl SwitcherState {
    pub const fn announcement(self) -> &'static str {
        match self {
            Self::Default => "Context ready",
            Self::Loading => "Switching context; queued changes are held",
            Self::PolicyDenied => "Policy denied the requested context switch",
            Self::Offline => "Offline; context switching is unavailable",
            Self::LegalHoldWarning => "Legal hold applies to the requested context",
        }
    }
}

/// One switchable tenant context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantContextOption {
    pub kind: ContextKind,
    pub pillar: OwnershipPillar,
    pub tenant_id: String,
    pub display_name: String,
}

/// Invariant 3: the admin-audit variant never appears in a personal context.
/// Filtering is the single authority for what the switcher offers.
pub fn permitted_options(
    pillar: OwnershipPillar,
    options: &[TenantContextOption],
) -> Vec<TenantContextOption> {
    options
        .iter()
        .filter(|option| {
            !(pillar == OwnershipPillar::Personal && option.kind == ContextKind::AdminAudit)
        })
        .cloned()
        .collect()
}

/// Invariant 1: every cache key is namespaced by context kind AND ownership
/// pillar (plus tenant), so cached state can never collide across pillars.
pub fn cache_namespace(option: &TenantContextOption) -> String {
    format!(
        "ctx/{}/{}/{}",
        option.kind.id(),
        option.pillar.id(),
        option.tenant_id
    )
}

/// Namespaced session cache. All reads and writes go through the namespace
/// derived from the FULL context identity — there is no raw-key API.
#[derive(Debug, Default)]
pub struct ContextScopedCache {
    entries: BTreeMap<String, String>,
}

impl ContextScopedCache {
    pub fn put(&mut self, context: &TenantContextOption, key: &str, value: String) {
        self.entries
            .insert(format!("{}/{key}", cache_namespace(context)), value);
    }

    pub fn get(&self, context: &TenantContextOption, key: &str) -> Option<&String> {
        self.entries
            .get(&format!("{}/{key}", cache_namespace(context)))
    }
}

/// Outcome of policy re-evaluation for a switch request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyDecision {
    Allow,
    Deny,
}

/// Invariant 2 (type-state gate): a requested switch that has NOT passed
/// policy re-evaluation. The only way to obtain a renderable
/// [`ActiveContext`] is [`Self::grant`] with an Allow decision — rendering a
/// child app from an unevaluated switch is unrepresentable.
#[derive(Debug)]
pub struct PendingSwitch {
    target: TenantContextOption,
}

/// A context the policy engine re-evaluated and allowed; the only token the
/// child-app render path accepts.
#[derive(Debug)]
pub struct ActiveContext {
    context: TenantContextOption,
}

impl PendingSwitch {
    pub fn new(target: TenantContextOption) -> Self {
        Self { target }
    }

    pub fn grant(self, decision: PolicyDecision) -> Result<ActiveContext, SwitcherState> {
        match decision {
            PolicyDecision::Allow => Ok(ActiveContext {
                context: self.target,
            }),
            PolicyDecision::Deny => Err(SwitcherState::PolicyDenied),
        }
    }
}

impl ActiveContext {
    pub fn context(&self) -> &TenantContextOption {
        &self.context
    }
}

/// WCAG 2.2 AA switcher: native buttons (keyboard reachable by default),
/// radiogroup semantics for arrow-key segment navigation, and an
/// `aria-live` announcement carrying active context, policy scope, and
/// queued-change state per the spec's screen-reader contract.
#[component]
pub fn TenantContextSwitcher(
    pillar: OwnershipPillar,
    options: Vec<TenantContextOption>,
    active_tenant_id: String,
    state: SwitcherState,
) -> impl IntoView {
    let permitted = permitted_options(pillar, &options);
    let announcement = state.announcement();
    view! {
        <nav class="ds-tenant-context-switcher" aria-label="Tenant context switcher">
            <div role="radiogroup" aria-label="Available tenant contexts">
                {permitted
                    .into_iter()
                    .map(|option| {
                        let selected = option.tenant_id == active_tenant_id;
                        view! {
                            <button
                                type="button"
                                role="radio"
                                aria-checked=selected.to_string()
                                data-context-kind=option.kind.id()
                                data-ownership-pillar=option.pillar.id()
                            >
                                <strong>{option.display_name.clone()}</strong>
                                <span>{option.kind.label()}</span>
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
            <p class="ds-switcher-status" role="status" aria-live="polite">{announcement}</p>
        </nav>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn option(kind: ContextKind, pillar: OwnershipPillar, tenant: &str) -> TenantContextOption {
        TenantContextOption {
            kind,
            pillar,
            tenant_id: tenant.to_owned(),
            display_name: format!("{tenant} ({})", kind.label()),
        }
    }

    #[test]
    fn cache_namespace_includes_kind_and_pillar() {
        // Spec test ref: test_cache_namespace_includes_pillar.
        let work = option(ContextKind::Work, OwnershipPillar::Organization, "acme");
        let ns = cache_namespace(&work);
        assert!(ns.contains("work"), "{ns}");
        assert!(ns.contains("organization"), "{ns}");
        assert!(ns.contains("acme"), "{ns}");
    }

    #[test]
    fn no_cache_leak_across_ownership_pillars() {
        // The G007 acceptance no-cache-leak test: identical tenant + key in
        // two pillars must resolve to disjoint cache entries.
        let personal = option(ContextKind::Personal, OwnershipPillar::Personal, "acme");
        let work = option(ContextKind::Work, OwnershipPillar::Organization, "acme");
        let mut cache = ContextScopedCache::default();
        cache.put(&personal, "inbox-snapshot", "personal-mail".to_owned());

        assert_eq!(
            cache.get(&work, "inbox-snapshot"),
            None,
            "cached state must never leak across ownership pillars"
        );
        assert_eq!(
            cache.get(&personal, "inbox-snapshot").map(String::as_str),
            Some("personal-mail")
        );
    }

    #[test]
    fn admin_audit_never_appears_in_personal_context() {
        let options = vec![
            option(ContextKind::Personal, OwnershipPillar::Personal, "me"),
            option(
                ContextKind::AdminAudit,
                OwnershipPillar::Organization,
                "acme",
            ),
        ];
        let permitted = permitted_options(OwnershipPillar::Personal, &options);
        assert!(
            permitted.iter().all(|o| o.kind != ContextKind::AdminAudit),
            "admin-audit variant must never appear in a personal context"
        );
        let org = permitted_options(OwnershipPillar::Organization, &options);
        assert!(org.iter().any(|o| o.kind == ContextKind::AdminAudit));
    }

    #[test]
    fn switch_requires_policy_grant_before_child_render() {
        let target = option(ContextKind::Work, OwnershipPillar::Organization, "acme");
        let denied = PendingSwitch::new(target.clone()).grant(PolicyDecision::Deny);
        assert_eq!(denied.unwrap_err(), SwitcherState::PolicyDenied);

        let granted = PendingSwitch::new(target).grant(PolicyDecision::Allow);
        assert_eq!(granted.unwrap().context().tenant_id, "acme");
    }
}
