//! Kernel vocabulary: ids, kinds, the scope record, the materialized path,
//! hierarchy edges, the persistence ports, and the closed error enum.
//!
//! Nothing here reads a clock, draws randomness, or performs I/O. The ports
//! are SYNC by deliberate design (see the crate-root "Gaps" note).

use core::fmt;

/// A tenant-local, opaque sub-scope identifier.
///
/// Tenant-local means two different tenants MAY legitimately mint the same
/// id string. Every lookup therefore has to carry a tenant, which is why the
/// tenant-scoped port methods — not the raw ones — are the application API.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct SubScopeId(pub String); // data_class: TENANT_SCOPED

impl SubScopeId {
    /// Borrow the id as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SubScopeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The closed set of scope kinds below the tenant.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum SubScopeKind {
    /// A durable slice of the tenant's own org structure.
    BusinessUnit,
    /// A collaboration surface owned by a business unit.
    Workspace,
    /// A time-boxed piece of work, usually with an external party.
    Engagement,
    /// A unit of delivery inside a workspace or engagement.
    Project,
    /// A restricted, evidence-bearing scope (audit / incident / case).
    Investigation,
    /// A scope that mirrors an external party. Terminal by rule.
    Counterparty,
    /// A tenant-defined scope with no platform semantics.
    Custom,
}

impl SubScopeKind {
    /// The stable wire label for this kind.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::BusinessUnit => "business_unit",
            Self::Workspace => "workspace",
            Self::Engagement => "engagement",
            Self::Project => "project",
            Self::Investigation => "investigation",
            Self::Counterparty => "counterparty",
            Self::Custom => "custom",
        }
    }
}

impl fmt::Display for SubScopeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// A scope record: adjacency (`parent`) plus its materialized `path`.
///
/// The two representations are kept consistent by the registrar — a record
/// whose `path` disagrees with its parent chain is refused, never stored.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubScope {
    pub id: SubScopeId,             // data_class: TENANT_SCOPED
    pub tenant_id: String,          // data_class: INTERNAL_ONLY
    pub kind: SubScopeKind,         // data_class: INTERNAL_ONLY
    pub parent: Option<SubScopeId>, // data_class: TENANT_SCOPED
    pub path: SubScopePath,         // data_class: TENANT_SCOPED
}

impl SubScope {
    /// Whether this record is the tenant root (the one scope with no parent).
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    /// Edges from the tenant root to this scope, `0` for the root itself.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.path.depth()
    }
}

/// The materialized path: normalized segments from the tenant root inclusive.
///
/// `SubScopePath(vec!["acme", "emea", "atlas"])` renders as `acme/emea/atlas`.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct SubScopePath(pub Vec<String>); // data_class: TENANT_SCOPED

impl SubScopePath {
    /// A path from already-normalized segments.
    #[must_use]
    pub fn new(segments: Vec<String>) -> Self {
        Self(segments)
    }

    /// The segments, root first.
    #[must_use]
    pub fn segments(&self) -> &[String] {
        &self.0
    }

    /// Edges between the tenant root and this path; the root itself is `0`.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.0.len().saturating_sub(1)
    }

    /// The scope's own (last) segment, `None` for an empty path.
    #[must_use]
    pub fn leaf(&self) -> Option<&str> {
        self.0.last().map(String::as_str)
    }

    /// Whether `self` is an ancestor path of `other` (strict prefix).
    #[must_use]
    pub fn is_strict_prefix_of(&self, other: &Self) -> bool {
        self.0.len() < other.0.len() && other.0.starts_with(&self.0)
    }
}

impl fmt::Display for SubScopePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.join("/"))
    }
}

/// One adjacency edge of the hierarchy; the closure-table adapter (IP-023)
/// persists exactly this shape.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct HierarchyEdge {
    pub parent: SubScopeId, // data_class: TENANT_SCOPED
    pub child: SubScopeId,  // data_class: TENANT_SCOPED
}

impl HierarchyEdge {
    /// A parent -> child edge.
    #[must_use]
    pub fn new(parent: SubScopeId, child: SubScopeId) -> Self {
        Self { parent, child }
    }
}

/// Storage port for scope records.
///
/// The raw methods (`insert`, `get`, `replace`, `delete`, `list_tenant`) are
/// storage primitives. Application code calls [`SubScopeRegistryPort::get_in_tenant`]
/// or the registrar, never the raw `get`: ids are tenant-local, so a bare id
/// is not a resolvable key.
pub trait SubScopeRegistryPort {
    /// Store a new record. Refuses `DuplicateScope` if `(tenant_id, id)` — or
    /// `(tenant_id, path)` — is already taken.
    fn insert(&self, scope: &SubScope) -> Result<(), SubScopeKernelError>;

    /// Raw, tenant-unscoped lookup.
    ///
    /// Implementations MUST return [`SubScopeKernelError::TenantBoundaryViolation`]
    /// when the same id exists in more than one tenant, rather than picking an
    /// arbitrary winner: silently returning the wrong tenant's scope is the
    /// archetypal isolation bug this crate exists to prevent.
    fn get(&self, id: &SubScopeId) -> Result<Option<SubScope>, SubScopeKernelError>;

    /// Overwrite an existing record in place. `NotFound` if absent.
    fn replace(&self, scope: &SubScope) -> Result<(), SubScopeKernelError>;

    /// Delete one record. `NotFound` if absent.
    fn delete(&self, tenant_id: &str, id: &SubScopeId) -> Result<(), SubScopeKernelError>;

    /// Every record of one tenant, ordered deterministically by id.
    fn list_tenant(&self, tenant_id: &str) -> Result<Vec<SubScope>, SubScopeKernelError>;

    /// Tenant-scoped lookup: the safe read.
    ///
    /// Returns `Ok(None)` — not an error — when the id belongs to a different
    /// tenant, so a probe cannot distinguish "not yours" from "does not
    /// exist" and therefore cannot enumerate another tenant's namespace.
    fn get_in_tenant(
        &self,
        tenant_id: &str,
        id: &SubScopeId,
    ) -> Result<Option<SubScope>, SubScopeKernelError> {
        match self.get(id) {
            Ok(Some(scope)) if scope.tenant_id == tenant_id => Ok(Some(scope)),
            Ok(_) => Ok(None),
            // Ambiguous id (same local id in several tenants): resolve it
            // through the tenant-keyed listing instead of failing the caller.
            Err(SubScopeKernelError::TenantBoundaryViolation) => Ok(self
                .list_tenant(tenant_id)?
                .into_iter()
                .find(|candidate| &candidate.id == id)),
            Err(other) => Err(other),
        }
    }

    /// Replace several records as ONE all-or-nothing unit.
    ///
    /// A subtree move rewrites the moved record AND every descendant's
    /// materialized path. Applying those as independent writes lets a
    /// mid-batch outage strand a descendant at its old path — a record whose
    /// path disagrees with its parent chain, the one state this kernel
    /// promises is never stored. The batch, not the record, is therefore the
    /// unit of atomicity.
    ///
    /// An implementation that can write transactionally MUST override this
    /// and commit the batch in one transaction. The default is a best-effort
    /// emulation: it applies the records in order and, on any failure,
    /// restores those it already wrote to their prior values. It reports the
    /// original cause when that restore succeeded, and
    /// [`SubScopeKernelError::PartialWriteUnresolved`] when it did not — so
    /// the caller can always tell "nothing happened" from "this tree needs
    /// repair".
    ///
    /// # Errors
    /// The first failure the batch hit, or
    /// [`SubScopeKernelError::PartialWriteUnresolved`] when the compensating
    /// restore failed as well.
    fn replace_all(&self, scopes: &[SubScope]) -> Result<(), SubScopeKernelError> {
        let mut undo: Vec<SubScope> = Vec::with_capacity(scopes.len());
        for scope in scopes {
            let prior = match self.get_in_tenant(&scope.tenant_id, &scope.id) {
                Ok(Some(prior)) => prior,
                Ok(None) => {
                    return Err(restore_batch(self, &undo, SubScopeKernelError::NotFound));
                }
                Err(error) => return Err(restore_batch(self, &undo, error)),
            };
            if let Err(error) = self.replace(scope) {
                return Err(restore_batch(self, &undo, error));
            }
            undo.push(prior);
        }
        Ok(())
    }
}

/// Best-effort compensating restore for [`SubScopeRegistryPort::replace_all`].
///
/// Rewinds in reverse write order, so the store retraces the states it came
/// through instead of jumping to one it was never in. Returns `cause` when
/// every record was restored, and
/// [`SubScopeKernelError::PartialWriteUnresolved`] when one could not be —
/// the store is then genuinely half-written and the caller must be told so.
fn restore_batch<P: SubScopeRegistryPort + ?Sized>(
    port: &P,
    undo: &[SubScope],
    cause: SubScopeKernelError,
) -> SubScopeKernelError {
    for prior in undo.iter().rev() {
        if port.replace(prior).is_err() {
            return SubScopeKernelError::PartialWriteUnresolved;
        }
    }
    cause
}

/// Hierarchy traversal port.
///
/// Ordering is part of the contract: `ancestors` is nearest-parent first up
/// to the tenant root; `descendants` is pre-order by materialized path, so
/// a parent always precedes its own children.
pub trait SubScopeHierarchyReadPort: SubScopeRegistryPort {
    /// Raw, tenant-unscoped ancestor chain, nearest parent first.
    ///
    /// A storage primitive, NOT the application API — it inherits the
    /// ambiguity of [`SubScopeRegistryPort::get`], so an id two tenants both
    /// minted has no answer here. Application code calls
    /// [`SubScopeHierarchyReadPort::ancestors_in_tenant`].
    ///
    /// # Errors
    /// [`SubScopeKernelError::TenantBoundaryViolation`] for an ambiguous id,
    /// [`SubScopeKernelError::NotFound`] for an unknown one, plus store errors.
    fn ancestors(&self, id: &SubScopeId) -> Result<Vec<SubScopeId>, SubScopeKernelError>;

    /// Raw, tenant-unscoped descendant set, pre-order by materialized path.
    ///
    /// The same storage primitive caveat as
    /// [`SubScopeHierarchyReadPort::ancestors`] applies.
    ///
    /// # Errors
    /// [`SubScopeKernelError::TenantBoundaryViolation`] for an ambiguous id,
    /// [`SubScopeKernelError::NotFound`] for an unknown one, plus store errors.
    fn descendants(&self, id: &SubScopeId) -> Result<Vec<SubScopeId>, SubScopeKernelError>;

    /// Tenant-scoped ancestors, nearest parent first.
    ///
    /// The walk goes through [`SubScopeRegistryPort::get_in_tenant`] and
    /// NEVER through the raw [`SubScopeHierarchyReadPort::ancestors`]. That
    /// is the whole point: a raw, tenant-unscoped lookup has no answer for an
    /// id two tenants both minted, so delegating to it would let one tenant's
    /// unrelated write make another tenant's own chain unreadable. Ids are
    /// tenant-local by contract, so the tenant-keyed read is the only correct
    /// key here — and a leaky raw traversal cannot reach the result at all.
    ///
    /// A chain that revisits an id is corrupt adjacency and surfaces as
    /// [`SubScopeKernelError::CycleRefused`], never as an unbounded walk.
    ///
    /// # Errors
    /// [`SubScopeKernelError::NotFound`] when the scope — or a link of its
    /// chain — is not this tenant's; [`SubScopeKernelError::CycleRefused`]
    /// for corrupt adjacency; store errors.
    fn ancestors_in_tenant(
        &self,
        tenant_id: &str,
        id: &SubScopeId,
    ) -> Result<Vec<SubScopeId>, SubScopeKernelError> {
        let scope = self
            .get_in_tenant(tenant_id, id)?
            .ok_or(SubScopeKernelError::NotFound)?;
        let mut chain: Vec<SubScopeId> = Vec::new();
        let mut cursor = scope.parent;
        while let Some(parent_id) = cursor {
            if parent_id == scope.id || chain.contains(&parent_id) {
                return Err(SubScopeKernelError::CycleRefused);
            }
            let parent = self
                .get_in_tenant(tenant_id, &parent_id)?
                .ok_or(SubScopeKernelError::NotFound)?;
            cursor = parent.parent;
            chain.push(parent_id);
        }
        Ok(chain)
    }

    /// Tenant-scoped descendants, pre-order by materialized path.
    ///
    /// Computed from this tenant's own listing and the materialized-path
    /// prefix, for the same reason as
    /// [`SubScopeHierarchyReadPort::ancestors_in_tenant`]: the tenant-keyed
    /// read is the only correct key for a tenant-local id. The scope itself
    /// is NOT one of its own descendants (the prefix test is strict).
    ///
    /// # Errors
    /// [`SubScopeKernelError::NotFound`] when the scope is not this tenant's;
    /// store errors.
    fn descendants_in_tenant(
        &self,
        tenant_id: &str,
        id: &SubScopeId,
    ) -> Result<Vec<SubScopeId>, SubScopeKernelError> {
        let scope = self
            .get_in_tenant(tenant_id, id)?
            .ok_or(SubScopeKernelError::NotFound)?;
        let mut found: Vec<SubScope> = self
            .list_tenant(tenant_id)?
            .into_iter()
            .filter(|candidate| scope.path.is_strict_prefix_of(&candidate.path))
            .collect();
        // Pre-order: a prefix path sorts before every path that extends it,
        // so ordering by (path, id) puts every parent before its children.
        found.sort_by(|left, right| {
            left.path
                .cmp(&right.path)
                .then_with(|| left.id.cmp(&right.id))
        });
        Ok(found.into_iter().map(|candidate| candidate.id).collect())
    }

    /// Drop every id that does not resolve inside `tenant_id`.
    ///
    /// The tenant-scoped defaults above no longer need this — they never read
    /// an id they did not already key by tenant. It stays part of the port
    /// contract for an adapter that DOES override the tenant-scoped methods
    /// with a native query (a recursive CTE, a closure-table join): running
    /// the result through this filter is what keeps a query bug from becoming
    /// a cross-tenant disclosure.
    ///
    /// # Errors
    /// Store errors raised while re-reading each id.
    fn retain_tenant(
        &self,
        tenant_id: &str,
        ids: Vec<SubScopeId>,
    ) -> Result<Vec<SubScopeId>, SubScopeKernelError> {
        let mut kept = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(scope) = self.get_in_tenant(tenant_id, &id)? {
                kept.push(scope.id);
            }
        }
        Ok(kept)
    }
}

/// The closed failure vocabulary of the sub-scope kernel.
///
/// This enum is the specification: every variant is reachable and pinned by
/// a test.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubScopeKernelError {
    /// The requested edge would make a scope its own ancestor.
    CycleRefused,
    /// The edge would push a scope past `MAX_DEPTH` edges from the root.
    DepthExceeded,
    /// The operation crossed a tenant boundary.
    TenantBoundaryViolation,
    /// The tenant root cannot be reparented, renamed, deleted or replaced.
    RootImmutable,
    /// A path segment failed normalization or validation.
    NamespaceMalformed,
    /// No such scope in this tenant.
    NotFound,
    /// The backing store could not answer.
    PersistenceUnavailable,
    /// The parent's kind may not carry a child of this kind.
    ParentKindNotAllowed,
    /// The tenant already holds a scope with this id or this path.
    DuplicateScope,
    /// The record's materialized path disagrees with its parent chain.
    PathInconsistent,
    /// A multi-record rewrite failed AND the compensating restore failed too,
    /// so the store may hold a partially rewritten subtree.
    ///
    /// Distinct from [`SubScopeKernelError::PersistenceUnavailable`] on
    /// purpose: that one means nothing happened, this one means the tenant's
    /// tree needs repair before it is trusted again.
    PartialWriteUnresolved,
}

impl SubScopeKernelError {
    /// A stable, human-readable reason string.
    #[must_use]
    pub fn message(&self) -> &'static str {
        match self {
            Self::CycleRefused => "edge refused: a scope may not become its own ancestor",
            Self::DepthExceeded => "edge refused: sub-scope depth limit exceeded",
            Self::TenantBoundaryViolation => "refused: the operation crosses a tenant boundary",
            Self::RootImmutable => "refused: the tenant root scope is immutable",
            Self::NamespaceMalformed => "refused: malformed sub-scope path segment",
            Self::NotFound => "no such sub-scope in this tenant",
            Self::PersistenceUnavailable => "sub-scope store unavailable",
            Self::ParentKindNotAllowed => "edge refused: parent kind may not carry this child kind",
            Self::DuplicateScope => "refused: duplicate sub-scope id or path in this tenant",
            Self::PathInconsistent => "refused: materialized path disagrees with the parent chain",
            Self::PartialWriteUnresolved => {
                "sub-scope subtree rewrite failed and could not be rolled back"
            }
        }
    }
}

impl fmt::Display for SubScopeKernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for SubScopeKernelError {}
