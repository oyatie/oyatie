//! Sub-scope registry kernel — the typed substrate for every scope BELOW the
//! tenant (business unit, workspace, engagement, project, investigation,
//! counterparty), implementing `tenancy/IP-016-sub-scope-registry-kernel.md`.
//!
//! IP-016 specifies four crates (`-kernel`, `-domain`, `-usecase`,
//! `-adapter`). The tenancy capability is capped at 12 crates and the
//! workspace lockfile is frozen, so the layering is collapsed into this one
//! crate as a module tree, face for face:
//!
//! - [`kernel`] — ids, kinds, the scope record, the materialized path, the
//!   hierarchy edge, both ports, and the closed error enum;
//! - [`domain`] — namespace normalization, path derivation, the parent-kind
//!   table, and edge validation (pure functions, no I/O);
//! - [`usecase`] — [`SubScopeRegistrar`], the only sanctioned mutator;
//! - [`inmemory`] — the reference store implementing both ports.
//!
//! Everything public is re-exported at the crate root, so the published
//! contract is a flat namespace regardless of the internal split.
//!
//! # The invariants this kernel owns
//!
//! The hierarchy model is an adjacency edge (`SubScope::parent`) plus a
//! materialized path projection (`SubScope::path`), exactly as IP-016 §B
//! requires for the future closure-table adapter. Both representations are
//! written through one derivation rule, so they cannot drift: a record whose
//! path disagrees with its parent chain is refused, never stored.
//!
//! - **Tenant isolation.** Ids are tenant-local: two tenants may legitimately
//!   mint the same id string, and doing so degrades neither tenant's tree.
//!   Every application read and every write keys by `(tenant_id, id)`, so a
//!   foreign scope is reported as ABSENT — on reads and on writes alike, with
//!   no carve-out — and no shape of request enumerates another tenant's
//!   namespace. The raw, tenant-less [`SubScopeRegistryPort::get`] refuses an
//!   ambiguous id rather than guessing a winner, and no tenant-scoped path
//!   depends on it.
//! - **No cycles.** An edge that would make a scope its own ancestor is
//!   refused, direct or arbitrarily indirect.
//! - **Bounded depth.** [`MAX_DEPTH`] edges from the tenant root, enforced on
//!   creation and on a subtree move by its DEEPEST member.
//! - **Immutable root.** The one parentless scope per tenant cannot be
//!   reparented, renamed, deleted or duplicated. A store that cannot answer
//!   "is there a root?" fails the write; it never counts as "there is none".
//! - **All-or-nothing subtree rewrites.** A move or rename rewrites the
//!   subject and every descendant as one batch through
//!   [`SubScopeRegistryPort::replace_all`], so an outage part-way cannot
//!   commit half of it. Where a store cannot be transactional, the failure is
//!   compensated and, if that compensation also fails, reported as
//!   [`SubScopeKernelError::PartialWriteUnresolved`] — never as a bare
//!   "nothing happened".
//! - **Namespace discipline.** Segments are normalized then validated against
//!   a documented ASCII rule; see [`normalize_segment`]. Tenant ids are held
//!   to their own character set by [`validate_tenant_id`], because this crate
//!   is the namespace validator its SQL and policy consumers trust.
//!
//! # Determinism
//!
//! No function here reads a clock, draws randomness, or performs I/O.
//! Traversal order is part of the contract (ancestors nearest-first,
//! descendants pre-order by path), so results are reproducible.
//!
//! # Gaps (deliberately deferred, not overlooked)
//!
//! - **Postgres adapter.** `sub_scopes` + `sub_scope_hierarchy_closure`
//!   persistence is IP-023 and is explicitly out of scope. Only the
//!   process-local [`inmemory`] store ships here; [`SubScopeRegistrar::edges`]
//!   is the projection IP-023 will seed its closure table from.
//! - **No dependencies.** The workspace lockfile is frozen for this wave, so
//!   this crate takes zero dependencies: no `serde` derives (wire encoding is
//!   the adapter's job), no `uuid`/`ulid` (ids arrive as opaque strings minted
//!   upstream), no `proptest` (the property-style cases IP-016 §D.6 asks for
//!   are written as explicit, enumerated tests instead).
//! - **Sync ports.** The ports are synchronous. The async/`tokio` boundary
//!   belongs to the adapter slice that owns the connection pool; making the
//!   kernel async would pull a runtime into a pure crate.
//! - **No policy evaluation.** `tenancy/policy/*.cedar` consumes these scopes;
//!   deciding permissions over them is not this crate's job.
//! - **No cross-tenant sharing.** IP-journey-j123 (shared workspace scope)
//!   needs a grant model that deliberately crosses the boundary this crate
//!   currently refuses outright; that is a later, explicit feature.
//! - **No per-tenant quota.** Nothing here caps how many scopes one tenant
//!   may hold, so the process has no bound on memory. Admission control and
//!   quota are a platform concern, not a hierarchy kernel's. What this crate
//!   DOES own — that one tenant's volume must not be felt in another
//!   tenant's latency — is handled: [`InMemorySubScopeRegistry`] keys and
//!   range-scans by tenant, so no read walks a foreign tenant's records.
//! - **Best-effort compensation, not distributed transactions.** The default
//!   [`SubScopeRegistryPort::replace_all`] emulates atomicity with a
//!   compensating restore. A real transaction is the adapter's job; the
//!   reference store already commits a batch under one lock.
//!
//! # Example
//!
//! ```ignore
//! let registrar = SubScopeRegistrar::new(InMemorySubScopeRegistry::new());
//! registrar.register_root("ten_acme", SubScopeId("sub_root".into()),
//!                         SubScopeKind::BusinessUnit, "acme")?;
//! ```
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod domain;
pub mod inmemory;
pub mod kernel;
pub mod usecase;

pub use domain::{
    MAX_DEPTH, MAX_SEGMENT_LEN, MAX_TENANT_ID_LEN, PATH_SEPARATOR, canonical_path, kind_may_parent,
    normalize_path, normalize_segment, reroot_path, validate_new_edge, validate_path_consistency,
    validate_tenant_id,
};
pub use inmemory::InMemorySubScopeRegistry;
pub use kernel::{
    HierarchyEdge, SubScope, SubScopeHierarchyReadPort, SubScopeId, SubScopeKernelError,
    SubScopeKind, SubScopePath, SubScopeRegistryPort,
};
pub use usecase::SubScopeRegistrar;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_normalization_folds_case_and_trims() {
        assert_eq!(normalize_segment("  Atlas  ").unwrap(), "atlas");
        assert_eq!(normalize_segment("EMEA-North").unwrap(), "emea-north");
        assert_eq!(normalize_segment("a1").unwrap(), "a1");
    }

    #[test]
    fn segment_validation_refuses_the_documented_character_set() {
        for bad in [
            "",               // empty
            "   ",            // whitespace only
            "a/b",            // the path separator
            "a.b",            // dotted
            "a_b",            // underscore
            "a b",            // inner space
            "1atlas",         // leading digit
            "-atlas",         // leading dash
            "atlas-",         // trailing dash
            "at--las",        // doubled dash
            "atlas\u{e9}",    // non-ASCII
            "\u{41c}\u{438}", // non-ASCII script
        ] {
            assert_eq!(
                normalize_segment(bad),
                Err(SubScopeKernelError::NamespaceMalformed),
                "segment {bad:?} must be refused"
            );
        }
        let too_long = "a".repeat(MAX_SEGMENT_LEN + 1);
        assert_eq!(
            normalize_segment(&too_long),
            Err(SubScopeKernelError::NamespaceMalformed)
        );
        assert!(normalize_segment(&"a".repeat(MAX_SEGMENT_LEN)).is_ok());
    }

    #[test]
    fn path_rendering_uses_the_single_separator() {
        let path = normalize_path(&["Acme".to_owned(), "EMEA".to_owned()]).unwrap();
        assert_eq!(path.to_string(), "acme/emea");
        assert_eq!(PATH_SEPARATOR, '/');
        assert_eq!(path.depth(), 1);
        assert_eq!(path.leaf(), Some("emea"));
    }

    #[test]
    fn parent_kind_table_matches_its_documentation() {
        assert!(kind_may_parent(
            SubScopeKind::BusinessUnit,
            SubScopeKind::BusinessUnit
        ));
        assert!(kind_may_parent(
            SubScopeKind::Engagement,
            SubScopeKind::Project
        ));
        // Rule 1: an engagement may not own a business unit.
        assert!(!kind_may_parent(
            SubScopeKind::Engagement,
            SubScopeKind::BusinessUnit
        ));
        // Rule 2: a counterparty is terminal.
        assert!(!kind_may_parent(
            SubScopeKind::Counterparty,
            SubScopeKind::Custom
        ));
    }

    #[test]
    fn errors_render_a_stable_message_and_are_std_errors() {
        let error: &dyn std::error::Error = &SubScopeKernelError::CycleRefused;
        assert_eq!(
            error.to_string(),
            "edge refused: a scope may not become its own ancestor"
        );
        assert_eq!(
            SubScopeKernelError::RootImmutable.message(),
            "refused: the tenant root scope is immutable"
        );
        // A half-written tree and a store that did nothing are DIFFERENT
        // answers; the messages must not blur them.
        assert_ne!(
            SubScopeKernelError::PartialWriteUnresolved.message(),
            SubScopeKernelError::PersistenceUnavailable.message()
        );
        assert_eq!(
            SubScopeKernelError::PartialWriteUnresolved.message(),
            "sub-scope subtree rewrite failed and could not be rolled back"
        );
    }

    #[test]
    fn kind_labels_are_stable() {
        assert_eq!(SubScopeKind::BusinessUnit.label(), "business_unit");
        assert_eq!(SubScopeKind::Investigation.to_string(), "investigation");
    }

    #[test]
    fn reroot_rewrites_only_matching_prefixes() {
        let path = SubScopePath::new(vec!["acme".into(), "emea".into(), "atlas".into()]);
        let old = SubScopePath::new(vec!["acme".into(), "emea".into()]);
        let new = SubScopePath::new(vec!["acme".into(), "apac".into()]);
        assert_eq!(
            reroot_path(&path, &old, &new).unwrap().to_string(),
            "acme/apac/atlas"
        );
        let unrelated = SubScopePath::new(vec!["globex".into()]);
        assert_eq!(reroot_path(&path, &unrelated, &new), None);
    }
}
