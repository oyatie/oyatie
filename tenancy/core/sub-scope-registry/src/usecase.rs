//! The registrar: the only sanctioned way to mutate a tenant's sub-scope
//! tree.
//!
//! Every entry point takes an explicit `tenant_id` and resolves through the
//! tenant-scoped port methods, so no operation can read or write another
//! tenant's namespace. The registrar owns validation; the port stays a dumb
//! store.

use crate::domain::{
    MAX_DEPTH, canonical_path, normalize_segment, reroot_path, validate_new_edge,
    validate_path_consistency, validate_tenant_id,
};
use crate::kernel::{
    HierarchyEdge, SubScope, SubScopeHierarchyReadPort, SubScopeId, SubScopeKernelError,
    SubScopeKind, SubScopePath, SubScopeRegistryPort,
};

/// Validating façade over a [`SubScopeHierarchyReadPort`] store.
#[derive(Debug, Clone)]
pub struct SubScopeRegistrar<R> {
    registry: R,
}

impl<R: SubScopeHierarchyReadPort> SubScopeRegistrar<R> {
    /// Wrap a store.
    pub const fn new(registry: R) -> Self {
        Self { registry }
    }

    /// Borrow the underlying store (read-only escape hatch for adapters).
    pub const fn registry(&self) -> &R {
        &self.registry
    }

    /// Create the tenant root: the single parentless scope of a tenant.
    ///
    /// # Errors
    /// [`SubScopeKernelError::RootImmutable`] when the tenant already has a
    /// root, [`SubScopeKernelError::NamespaceMalformed`] for a bad segment,
    /// [`SubScopeKernelError::TenantBoundaryViolation`] for a blank tenant id.
    pub fn register_root(
        &self,
        tenant_id: &str,
        id: SubScopeId,
        kind: SubScopeKind,
        segment: &str,
    ) -> Result<SubScope, SubScopeKernelError> {
        validate_tenant_id(tenant_id)?;
        let leaf = normalize_segment(segment)?;
        let scope = SubScope {
            id,
            tenant_id: tenant_id.to_owned(),
            kind,
            parent: None,
            path: SubScopePath::new(vec![leaf]),
        };
        self.register(&scope)
    }

    /// Create a child scope under `parent`, deriving its canonical path.
    ///
    /// # Errors
    /// Any variant of [`SubScopeKernelError`] raised by edge validation,
    /// namespace validation or the store.
    pub fn register_child(
        &self,
        tenant_id: &str,
        id: SubScopeId,
        kind: SubScopeKind,
        parent: &SubScopeId,
        segment: &str,
    ) -> Result<SubScope, SubScopeKernelError> {
        validate_tenant_id(tenant_id)?;
        let parent_scope = self.resolve_parent(tenant_id, parent)?;
        let path = canonical_path(&parent_scope.path, segment)?;
        let scope = SubScope {
            id,
            tenant_id: tenant_id.to_owned(),
            kind,
            parent: Some(parent_scope.id.clone()),
            path,
        };
        self.register(&scope)
    }

    /// Register a fully-formed record, including a caller-supplied path.
    ///
    /// The path is checked against the parent chain and refused — never
    /// stored — when the two disagree.
    ///
    /// # Errors
    /// [`SubScopeKernelError::PathInconsistent`] when the record's path is not
    /// the derived one; [`SubScopeKernelError::RootImmutable`] for a second
    /// root; [`SubScopeKernelError::TenantBoundaryViolation`] when the parent
    /// belongs to another tenant; plus cycle, depth, kind, namespace and
    /// duplicate refusals.
    pub fn register(&self, scope: &SubScope) -> Result<SubScope, SubScopeKernelError> {
        validate_tenant_id(&scope.tenant_id)?;
        match scope.parent.clone() {
            None => {
                match self.root(&scope.tenant_id) {
                    Ok(_existing) => return Err(SubScopeKernelError::RootImmutable),
                    // The tenant genuinely has no root: the ONE case in which
                    // a second parentless scope may be created.
                    Err(SubScopeKernelError::NotFound) => {}
                    // A store that could not answer is not evidence of an
                    // absent root. Collapsing the two into `is_ok()` would
                    // make a read outage mint a duplicate root and silently
                    // lose the immutable-root invariant, so the store error
                    // is surfaced instead.
                    Err(other) => return Err(other),
                }
                validate_path_consistency(None, scope)?;
            }
            Some(parent_id) => {
                let parent = self.resolve_parent(&scope.tenant_id, &parent_id)?;
                let parent_ancestors = self
                    .registry
                    .ancestors_in_tenant(&scope.tenant_id, &parent.id)?;
                validate_new_edge(&parent, scope, &parent_ancestors)?;
                validate_path_consistency(Some(&parent), scope)?;
            }
        }
        self.registry.insert(scope)?;
        Ok(scope.clone())
    }

    /// The tenant root record.
    ///
    /// # Errors
    /// [`SubScopeKernelError::NotFound`] when the tenant has no root yet.
    pub fn root(&self, tenant_id: &str) -> Result<SubScope, SubScopeKernelError> {
        self.registry
            .list_tenant(tenant_id)?
            .into_iter()
            .find(SubScope::is_root)
            .ok_or(SubScopeKernelError::NotFound)
    }

    /// Tenant-scoped read of one scope.
    ///
    /// # Errors
    /// [`SubScopeKernelError::NotFound`] when the id is absent OR belongs to
    /// another tenant — the two are deliberately indistinguishable.
    pub fn resolve(
        &self,
        tenant_id: &str,
        id: &SubScopeId,
    ) -> Result<SubScope, SubScopeKernelError> {
        validate_tenant_id(tenant_id)?;
        self.registry
            .get_in_tenant(tenant_id, id)?
            .ok_or(SubScopeKernelError::NotFound)
    }

    /// Tenant-scoped ancestors, nearest parent first.
    ///
    /// # Errors
    /// [`SubScopeKernelError::NotFound`] outside the tenant; store errors.
    pub fn ancestors(
        &self,
        tenant_id: &str,
        id: &SubScopeId,
    ) -> Result<Vec<SubScopeId>, SubScopeKernelError> {
        validate_tenant_id(tenant_id)?;
        self.registry.ancestors_in_tenant(tenant_id, id)
    }

    /// Tenant-scoped descendants, pre-order by materialized path.
    ///
    /// # Errors
    /// [`SubScopeKernelError::NotFound`] outside the tenant; store errors.
    pub fn descendants(
        &self,
        tenant_id: &str,
        id: &SubScopeId,
    ) -> Result<Vec<SubScopeId>, SubScopeKernelError> {
        validate_tenant_id(tenant_id)?;
        self.registry.descendants_in_tenant(tenant_id, id)
    }

    /// The tenant's whole adjacency edge set, ordered by (parent, child).
    ///
    /// This is the projection IP-023 persists as `sub_scope_hierarchy_closure`
    /// seed rows without re-implementing any validation.
    ///
    /// # Errors
    /// Store errors from the underlying port.
    pub fn edges(&self, tenant_id: &str) -> Result<Vec<HierarchyEdge>, SubScopeKernelError> {
        validate_tenant_id(tenant_id)?;
        let mut edges: Vec<HierarchyEdge> = self
            .registry
            .list_tenant(tenant_id)?
            .into_iter()
            .filter_map(|scope| {
                scope
                    .parent
                    .map(|parent| HierarchyEdge::new(parent, scope.id))
            })
            .collect();
        edges.sort();
        Ok(edges)
    }

    /// Move a scope (with its whole subtree) under a new parent.
    ///
    /// # Errors
    /// [`SubScopeKernelError::RootImmutable`] for the tenant root,
    /// [`SubScopeKernelError::CycleRefused`] when the new parent is the scope
    /// itself or one of its descendants, [`SubScopeKernelError::DepthExceeded`]
    /// when the deepest moved descendant would pass [`MAX_DEPTH`],
    /// [`SubScopeKernelError::TenantBoundaryViolation`] for a foreign parent,
    /// plus kind and duplicate refusals.
    pub fn reparent(
        &self,
        tenant_id: &str,
        id: &SubScopeId,
        new_parent: &SubScopeId,
    ) -> Result<SubScope, SubScopeKernelError> {
        let scope = self.resolve(tenant_id, id)?;
        if scope.is_root() {
            return Err(SubScopeKernelError::RootImmutable);
        }
        let parent = self.resolve_parent(tenant_id, new_parent)?;
        let parent_ancestors = self.registry.ancestors_in_tenant(tenant_id, &parent.id)?;
        validate_new_edge(&parent, &scope, &parent_ancestors)?;

        // A stored record always carries a non-empty path; an empty one is a
        // corrupt record handed back by the store, and a corrupt record is
        // refused rather than moved. `PathInconsistent` names that honestly —
        // nothing about the caller's namespace was malformed.
        let leaf = scope
            .path
            .leaf()
            .ok_or(SubScopeKernelError::PathInconsistent)?
            .to_owned();
        let new_path = canonical_path(&parent.path, &leaf)?;
        let subtree = self.subtree(tenant_id, &scope)?;
        self.assert_subtree_fits(&new_path, &scope, &subtree)?;
        self.assert_path_free(tenant_id, &new_path, &scope.id)?;

        let moved = SubScope {
            parent: Some(parent.id),
            path: new_path.clone(),
            ..scope.clone()
        };
        let batch = Self::rewritten_batch(moved.clone(), &subtree, &scope.path, &new_path)?;
        self.registry.replace_all(&batch)?;
        Ok(moved)
    }

    /// Rename a scope's own path segment, rewriting its subtree's paths.
    ///
    /// # Errors
    /// [`SubScopeKernelError::RootImmutable`] for the tenant root,
    /// [`SubScopeKernelError::NamespaceMalformed`] for a bad segment,
    /// [`SubScopeKernelError::DuplicateScope`] when a sibling already holds
    /// the new path; store errors.
    pub fn rename(
        &self,
        tenant_id: &str,
        id: &SubScopeId,
        new_segment: &str,
    ) -> Result<SubScope, SubScopeKernelError> {
        let scope = self.resolve(tenant_id, id)?;
        // `SubScope::is_root` IS `parent.is_none()`, so this destructuring is
        // the whole root guard. An `if is_root()` check followed by an
        // `ok_or(RootImmutable)` on the same field would leave a second,
        // permanently unreachable error path in a public signature.
        let Some(parent_id) = scope.parent.clone() else {
            return Err(SubScopeKernelError::RootImmutable);
        };
        let parent = self.resolve_parent(tenant_id, &parent_id)?;
        let new_path = canonical_path(&parent.path, new_segment)?;
        if new_path == scope.path {
            return Ok(scope);
        }
        self.assert_path_free(tenant_id, &new_path, &scope.id)?;

        let subtree = self.subtree(tenant_id, &scope)?;
        let renamed = SubScope {
            path: new_path.clone(),
            ..scope.clone()
        };
        let batch = Self::rewritten_batch(renamed.clone(), &subtree, &scope.path, &new_path)?;
        self.registry.replace_all(&batch)?;
        Ok(renamed)
    }

    /// Delete a scope and its whole subtree; returns how many records went.
    ///
    /// Deletion is deepest-first, which is why it needs no batch: a delete
    /// that stops part-way has removed a suffix of the subtree and left a
    /// smaller, still fully consistent tree behind. Every surviving record
    /// still sits at its derived path under a live parent.
    ///
    /// # Errors
    /// [`SubScopeKernelError::RootImmutable`] for the tenant root,
    /// [`SubScopeKernelError::NotFound`] outside the tenant; store errors.
    pub fn remove(&self, tenant_id: &str, id: &SubScopeId) -> Result<usize, SubScopeKernelError> {
        let scope = self.resolve(tenant_id, id)?;
        if scope.is_root() {
            return Err(SubScopeKernelError::RootImmutable);
        }
        let subtree = self.subtree(tenant_id, &scope)?;
        // Deepest first, so no record is ever orphaned mid-delete.
        for record in subtree.iter().rev() {
            self.registry.delete(tenant_id, &record.id)?;
        }
        self.registry.delete(tenant_id, &scope.id)?;
        Ok(subtree.len() + 1)
    }

    /// Resolve a parent id inside `tenant_id`.
    ///
    /// A parent that belongs to ANOTHER tenant is reported as
    /// [`SubScopeKernelError::NotFound`], exactly like an id that exists
    /// nowhere. `TenantBoundaryViolation` would be the more specific answer,
    /// but the difference is observable to any authenticated caller and turns
    /// `register_child` into an oracle: guess an id, and the response says
    /// whether some other tenant owns it. Write paths get the same "absent"
    /// answer as read paths, so there is no shape of request that enumerates
    /// a foreign namespace.
    fn resolve_parent(
        &self,
        tenant_id: &str,
        parent: &SubScopeId,
    ) -> Result<SubScope, SubScopeKernelError> {
        self.registry
            .get_in_tenant(tenant_id, parent)?
            .ok_or(SubScopeKernelError::NotFound)
    }

    /// The scope's descendant records, pre-order (parents before children).
    fn subtree(
        &self,
        tenant_id: &str,
        scope: &SubScope,
    ) -> Result<Vec<SubScope>, SubScopeKernelError> {
        let ids = self.registry.descendants_in_tenant(tenant_id, &scope.id)?;
        let mut records = Vec::with_capacity(ids.len());
        for id in ids {
            records.push(self.resolve(tenant_id, &id)?);
        }
        Ok(records)
    }

    /// Refuse a move whose deepest descendant would pass [`MAX_DEPTH`].
    fn assert_subtree_fits(
        &self,
        new_path: &SubScopePath,
        scope: &SubScope,
        subtree: &[SubScope],
    ) -> Result<(), SubScopeKernelError> {
        let deepest_relative = subtree
            .iter()
            .map(|record| record.depth().saturating_sub(scope.depth()))
            .max()
            .unwrap_or(0);
        if new_path.depth() + deepest_relative > MAX_DEPTH {
            return Err(SubScopeKernelError::DepthExceeded);
        }
        Ok(())
    }

    /// Refuse a move or rename onto a path some other scope already holds.
    fn assert_path_free(
        &self,
        tenant_id: &str,
        path: &SubScopePath,
        except: &SubScopeId,
    ) -> Result<(), SubScopeKernelError> {
        let taken = self
            .registry
            .list_tenant(tenant_id)?
            .into_iter()
            .any(|record| &record.path == path && &record.id != except);
        if taken {
            return Err(SubScopeKernelError::DuplicateScope);
        }
        Ok(())
    }

    /// The complete write set for a move or a rename: the subject at its new
    /// path, followed by every descendant re-rooted onto it.
    ///
    /// Built as ONE batch on purpose. Writing the subject and then each
    /// descendant as independent calls is exactly what lets a mid-flight
    /// store outage commit the subject's move and strand a descendant at its
    /// old path — a record whose materialized path no longer agrees with its
    /// parent chain, which the crate contract says is never stored. Handing
    /// the whole set to [`SubScopeRegistryPort::replace_all`] makes the store
    /// responsible for applying it all-or-nothing.
    fn rewritten_batch(
        subject: SubScope,
        subtree: &[SubScope],
        old_prefix: &SubScopePath,
        new_prefix: &SubScopePath,
    ) -> Result<Vec<SubScope>, SubScopeKernelError> {
        let mut batch = Vec::with_capacity(subtree.len() + 1);
        batch.push(subject);
        for record in subtree {
            let path = reroot_path(&record.path, old_prefix, new_prefix)
                .ok_or(SubScopeKernelError::PathInconsistent)?;
            batch.push(SubScope {
                path,
                ..record.clone()
            });
        }
        Ok(batch)
    }
}
