//! In-memory registry adapter: the reference implementation of both ports.
//!
//! It is process-local and non-durable — the durable closure-table adapter is
//! IP-023 (Postgres) and is out of scope here. Records are keyed by
//! `(tenant_id, sub_scope_id)`, which is what makes tenant isolation a
//! property of the storage layout rather than of caller discipline.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Bound;
use std::sync::{Mutex, MutexGuard};

use crate::domain::MAX_DEPTH;
use crate::kernel::{
    SubScope, SubScopeHierarchyReadPort, SubScopeId, SubScopeKernelError, SubScopePath,
    SubScopeRegistryPort,
};

type ScopeMap = BTreeMap<(String, SubScopeId), SubScope>;

/// A process-local sub-scope registry.
#[derive(Debug, Default)]
pub struct InMemorySubScopeRegistry {
    scopes: Mutex<ScopeMap>,
}

impl InMemorySubScopeRegistry {
    /// An empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of stored records across all tenants.
    ///
    /// # Errors
    /// [`SubScopeKernelError::PersistenceUnavailable`] if the lock is poisoned.
    pub fn len(&self) -> Result<usize, SubScopeKernelError> {
        Ok(self.guard()?.len())
    }

    /// Whether the registry holds no records at all.
    ///
    /// # Errors
    /// [`SubScopeKernelError::PersistenceUnavailable`] if the lock is poisoned.
    pub fn is_empty(&self) -> Result<bool, SubScopeKernelError> {
        Ok(self.len()? == 0)
    }

    /// A poisoned lock is an unavailable store, never a panic.
    fn guard(&self) -> Result<MutexGuard<'_, ScopeMap>, SubScopeKernelError> {
        self.scopes
            .lock()
            .map_err(|_poisoned| SubScopeKernelError::PersistenceUnavailable)
    }

    fn key(scope: &SubScope) -> (String, SubScopeId) {
        (scope.tenant_id.clone(), scope.id.clone())
    }

    /// Resolve an id the way [`SubScopeRegistryPort::get`] does, from an
    /// already-held guard.
    fn resolve(map: &ScopeMap, id: &SubScopeId) -> Result<Option<SubScope>, SubScopeKernelError> {
        let mut found: Option<&SubScope> = None;
        for ((_tenant, scope_id), scope) in map {
            if scope_id == id {
                if found.is_some() {
                    // The same tenant-local id in two tenants: refuse loudly
                    // instead of handing back an arbitrary tenant's scope.
                    return Err(SubScopeKernelError::TenantBoundaryViolation);
                }
                found = Some(scope);
            }
        }
        Ok(found.cloned())
    }

    fn path_taken(
        map: &ScopeMap,
        tenant_id: &str,
        path: &SubScopePath,
        except: &SubScopeId,
    ) -> bool {
        Self::tenant_range(map, tenant_id)
            .any(|((_tenant, scope_id), scope)| scope_id != except && &scope.path == path)
    }

    /// Every entry of one tenant, keyed-range rather than whole-map scanned.
    ///
    /// Keys are `(tenant_id, id)` and compare lexicographically, so one
    /// tenant's records are a contiguous run: the walk costs O(log n + k) in
    /// that tenant's own size, never in the size of the whole store.
    fn tenant_range<'map>(
        map: &'map ScopeMap,
        tenant_id: &str,
    ) -> impl Iterator<Item = (&'map (String, SubScopeId), &'map SubScope)> {
        let low = (tenant_id.to_owned(), SubScopeId(String::new()));
        map.range((Bound::Included(low), Bound::Unbounded))
            .take_while(move |((tenant, _id), _scope)| tenant == tenant_id)
    }

    /// Whether two records of one tenant claim the same materialized path.
    fn has_duplicate_path(map: &ScopeMap, tenant_id: &str) -> bool {
        let mut seen: BTreeSet<&SubScopePath> = BTreeSet::new();
        Self::tenant_range(map, tenant_id).any(|(_key, scope)| !seen.insert(&scope.path))
    }
}

impl SubScopeRegistryPort for InMemorySubScopeRegistry {
    fn insert(&self, scope: &SubScope) -> Result<(), SubScopeKernelError> {
        let mut map = self.guard()?;
        let key = Self::key(scope);
        if map.contains_key(&key)
            || Self::path_taken(&map, &scope.tenant_id, &scope.path, &scope.id)
        {
            return Err(SubScopeKernelError::DuplicateScope);
        }
        map.insert(key, scope.clone());
        Ok(())
    }

    fn get(&self, id: &SubScopeId) -> Result<Option<SubScope>, SubScopeKernelError> {
        let map = self.guard()?;
        Self::resolve(&map, id)
    }

    fn replace(&self, scope: &SubScope) -> Result<(), SubScopeKernelError> {
        let mut map = self.guard()?;
        let key = Self::key(scope);
        if !map.contains_key(&key) {
            return Err(SubScopeKernelError::NotFound);
        }
        if Self::path_taken(&map, &scope.tenant_id, &scope.path, &scope.id) {
            return Err(SubScopeKernelError::DuplicateScope);
        }
        map.insert(key, scope.clone());
        Ok(())
    }

    fn delete(&self, tenant_id: &str, id: &SubScopeId) -> Result<(), SubScopeKernelError> {
        let mut map = self.guard()?;
        map.remove(&(tenant_id.to_owned(), id.clone()))
            .map(|_removed| ())
            .ok_or(SubScopeKernelError::NotFound)
    }

    fn list_tenant(&self, tenant_id: &str) -> Result<Vec<SubScope>, SubScopeKernelError> {
        let map = self.guard()?;
        Ok(Self::tenant_range(&map, tenant_id)
            .map(|(_key, scope)| scope.clone())
            .collect())
    }

    /// Tenant-keyed read: an O(log n) hit on `(tenant_id, id)`.
    ///
    /// Overriding the default matters for more than speed. The default routes
    /// through the raw [`SubScopeRegistryPort::get`], which refuses an id two
    /// tenants both minted; keying the lookup by tenant means that legitimate
    /// collision is never even observed, and one tenant's volume can no
    /// longer be felt in another tenant's read latency.
    fn get_in_tenant(
        &self,
        tenant_id: &str,
        id: &SubScopeId,
    ) -> Result<Option<SubScope>, SubScopeKernelError> {
        let map = self.guard()?;
        Ok(map.get(&(tenant_id.to_owned(), id.clone())).cloned())
    }

    /// Atomic batch replace: every record lands, or none does.
    ///
    /// The reference store holds one lock for the whole batch, so the
    /// partially rewritten subtree the default emulation can only compensate
    /// for is unrepresentable here. `(tenant_id, path)` uniqueness is judged
    /// on the POST-state, because a subtree move legitimately passes through
    /// paths that were occupied a moment earlier in the same batch.
    fn replace_all(&self, scopes: &[SubScope]) -> Result<(), SubScopeKernelError> {
        let mut map = self.guard()?;
        // Validate every key BEFORE touching the map, so the common rejection
        // never mutates at all.
        for scope in scopes {
            if !map.contains_key(&Self::key(scope)) {
                return Err(SubScopeKernelError::NotFound);
            }
        }
        let mut undo: Vec<((String, SubScopeId), SubScope)> = Vec::with_capacity(scopes.len());
        let mut tenants: BTreeSet<&str> = BTreeSet::new();
        for scope in scopes {
            let key = Self::key(scope);
            tenants.insert(scope.tenant_id.as_str());
            if let Some(prior) = map.insert(key.clone(), scope.clone()) {
                undo.push((key, prior));
            }
        }
        let collided = tenants
            .into_iter()
            .any(|tenant| Self::has_duplicate_path(&map, tenant));
        if collided {
            for (key, prior) in undo.into_iter().rev() {
                map.insert(key, prior);
            }
            return Err(SubScopeKernelError::DuplicateScope);
        }
        Ok(())
    }
}

impl SubScopeHierarchyReadPort for InMemorySubScopeRegistry {
    fn ancestors(&self, id: &SubScopeId) -> Result<Vec<SubScopeId>, SubScopeKernelError> {
        let map = self.guard()?;
        let scope = Self::resolve(&map, id)?.ok_or(SubScopeKernelError::NotFound)?;
        let tenant = scope.tenant_id.clone();
        let mut chain = Vec::new();
        let mut cursor = scope.parent.clone();
        // Bounded walk: corrupted adjacency must surface as CycleRefused, not
        // as a hang.
        while let Some(parent_id) = cursor {
            if chain.len() > MAX_DEPTH + 1 {
                return Err(SubScopeKernelError::CycleRefused);
            }
            let parent = map
                .get(&(tenant.clone(), parent_id.clone()))
                .ok_or(SubScopeKernelError::NotFound)?;
            chain.push(parent_id);
            cursor = parent.parent.clone();
        }
        Ok(chain)
    }

    fn descendants(&self, id: &SubScopeId) -> Result<Vec<SubScopeId>, SubScopeKernelError> {
        let map = self.guard()?;
        let scope = Self::resolve(&map, id)?.ok_or(SubScopeKernelError::NotFound)?;
        let mut found: Vec<&SubScope> = Self::tenant_range(&map, &scope.tenant_id)
            .filter(|(_key, candidate)| scope.path.is_strict_prefix_of(&candidate.path))
            .map(|(_key, candidate)| candidate)
            .collect();
        // Pre-order: a prefix path sorts before every path that extends it,
        // so ordering by (path, id) puts every parent before its children.
        found.sort_by(|left, right| {
            left.path
                .cmp(&right.path)
                .then_with(|| left.id.cmp(&right.id))
        });
        Ok(found
            .into_iter()
            .map(|candidate| candidate.id.clone())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::kernel::SubScopeKind;

    fn scope(tenant: &str, id: &str, parent: Option<&str>, path: &[&str]) -> SubScope {
        SubScope {
            id: SubScopeId(id.to_owned()),
            tenant_id: tenant.to_owned(),
            kind: SubScopeKind::Workspace,
            parent: parent.map(|value| SubScopeId(value.to_owned())),
            path: SubScopePath::new(path.iter().map(|s| (*s).to_owned()).collect()),
        }
    }

    #[test]
    fn raw_get_refuses_an_id_that_exists_in_two_tenants() {
        let registry = InMemorySubScopeRegistry::new();
        registry
            .insert(&scope("ten_a", "sub_x", None, &["acme"]))
            .unwrap();
        registry
            .insert(&scope("ten_b", "sub_x", None, &["globex"]))
            .unwrap();

        // The archetypal isolation bug would be returning one of them here.
        assert_eq!(
            registry.get(&SubScopeId("sub_x".to_owned())),
            Err(SubScopeKernelError::TenantBoundaryViolation)
        );
        // The tenant-scoped read still resolves each tenant's own record.
        let a = registry
            .get_in_tenant("ten_a", &SubScopeId("sub_x".to_owned()))
            .unwrap()
            .unwrap();
        let b = registry
            .get_in_tenant("ten_b", &SubScopeId("sub_x".to_owned()))
            .unwrap()
            .unwrap();
        assert_eq!(a.path.to_string(), "acme");
        assert_eq!(b.path.to_string(), "globex");
    }

    #[test]
    fn a_poisoned_lock_reports_persistence_unavailable() {
        let registry = Arc::new(InMemorySubScopeRegistry::new());
        let holder = Arc::clone(&registry);
        let poisoner = std::thread::spawn(move || {
            let _guard = holder.scopes.lock().unwrap();
            panic!("poison the registry lock on purpose");
        });
        assert!(poisoner.join().is_err());

        assert_eq!(
            registry.get(&SubScopeId("sub_x".to_owned())),
            Err(SubScopeKernelError::PersistenceUnavailable)
        );
        assert_eq!(
            registry.list_tenant("ten_a"),
            Err(SubScopeKernelError::PersistenceUnavailable)
        );
    }

    #[test]
    fn the_raw_traversals_refuse_an_ambiguous_id_and_no_tenant_read_needs_them() {
        let registry = InMemorySubScopeRegistry::new();
        // Both tenants mint the identical two-record tree.
        for tenant in ["ten_a", "ten_b"] {
            registry
                .insert(&scope(tenant, "sub_root", None, &["acme"]))
                .unwrap();
            registry
                .insert(&scope(tenant, "sub_ws", Some("sub_root"), &["acme", "ws"]))
                .unwrap();
        }
        let ws = SubScopeId("sub_ws".to_owned());
        let root = SubScopeId("sub_root".to_owned());

        // The raw traversals are storage primitives and inherit `get`'s
        // ambiguity; that is their documented contract.
        assert_eq!(
            registry.ancestors(&ws),
            Err(SubScopeKernelError::TenantBoundaryViolation)
        );
        assert_eq!(
            registry.descendants(&root),
            Err(SubScopeKernelError::TenantBoundaryViolation)
        );
        // The tenant-scoped reads do not depend on them, so each tenant's own
        // hierarchy stays perfectly readable.
        assert_eq!(
            registry.ancestors_in_tenant("ten_a", &ws).unwrap(),
            vec![root.clone()]
        );
        assert_eq!(
            registry.descendants_in_tenant("ten_b", &root).unwrap(),
            vec![ws.clone()]
        );
        assert_eq!(
            registry.descendants_in_tenant("ten_a", &ws).unwrap(),
            Vec::<SubScopeId>::new()
        );
    }

    #[test]
    fn a_tenant_listing_covers_that_tenant_exactly() {
        let registry = InMemorySubScopeRegistry::new();
        // Tenant names that bracket "ten_b" lexically, to pin the keyed range
        // scan: a sloppy bound would spill a neighbour's records into it.
        for tenant in ["ten_a", "ten_b", "ten_ba", "ten_c"] {
            registry
                .insert(&scope(tenant, &format!("sub_{tenant}"), None, &["acme"]))
                .unwrap();
        }
        registry
            .insert(&scope(
                "ten_b",
                "sub_zz",
                Some("sub_ten_b"),
                &["acme", "ws"],
            ))
            .unwrap();

        let listed: Vec<String> = registry
            .list_tenant("ten_b")
            .unwrap()
            .into_iter()
            .map(|record| record.id.0)
            .collect();
        assert_eq!(listed, vec!["sub_ten_b".to_owned(), "sub_zz".to_owned()]);
        assert_eq!(registry.list_tenant("ten_nope").unwrap(), Vec::new());
        assert_eq!(registry.len().unwrap(), 5);
    }

    #[test]
    fn delete_and_replace_report_not_found_for_absent_records() {
        let registry = InMemorySubScopeRegistry::new();
        let missing = scope("ten_a", "sub_ghost", None, &["acme"]);
        assert_eq!(
            registry.delete("ten_a", &missing.id),
            Err(SubScopeKernelError::NotFound)
        );
        assert_eq!(
            registry.replace(&missing),
            Err(SubScopeKernelError::NotFound)
        );
    }
}
