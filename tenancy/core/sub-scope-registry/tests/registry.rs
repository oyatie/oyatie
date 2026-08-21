//! Behavioral pins for the sub-scope registry: every `SubScopeKernelError`
//! variant, both depth boundaries, path derivation, and — above all — the
//! tenant boundary on every read and write path.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use tenancy_sub_scope_registry::{
    InMemorySubScopeRegistry, MAX_DEPTH, MAX_TENANT_ID_LEN, SubScope, SubScopeHierarchyReadPort,
    SubScopeId, SubScopeKernelError, SubScopeKind, SubScopePath, SubScopeRegistrar,
    SubScopeRegistryPort,
};

const TEN_A: &str = "ten_acme";
const TEN_B: &str = "ten_globex";

fn id(value: &str) -> SubScopeId {
    SubScopeId(value.to_owned())
}

fn path(segments: &[&str]) -> SubScopePath {
    SubScopePath::new(segments.iter().map(|s| (*s).to_owned()).collect())
}

/// A tenant with a root and one business unit under it.
fn seeded() -> SubScopeRegistrar<InMemorySubScopeRegistry> {
    let registrar = SubScopeRegistrar::new(InMemorySubScopeRegistry::new());
    registrar
        .register_root(TEN_A, id("sub_root"), SubScopeKind::BusinessUnit, "acme")
        .unwrap();
    registrar
        .register_child(
            TEN_A,
            id("sub_emea"),
            SubScopeKind::BusinessUnit,
            &id("sub_root"),
            "EMEA",
        )
        .unwrap();
    registrar
}

/// Chain `count` scopes below the root, returning the deepest id.
fn chain(registrar: &SubScopeRegistrar<InMemorySubScopeRegistry>, count: usize) -> SubScopeId {
    let mut parent = id("sub_root");
    for step in 1..=count {
        let child = id(&format!("sub_d{step}"));
        registrar
            .register_child(
                TEN_A,
                child.clone(),
                SubScopeKind::Workspace,
                &parent,
                &format!("d{step}"),
            )
            .unwrap();
        parent = child;
    }
    parent
}

#[test]
fn child_paths_are_derived_from_the_parent_chain() {
    let registrar = seeded();
    let emea = registrar.resolve(TEN_A, &id("sub_emea")).unwrap();
    // "EMEA" normalized, and the path is the parent path plus the segment.
    assert_eq!(emea.path, path(&["acme", "emea"]));
    assert_eq!(emea.depth(), 1);
    assert_eq!(emea.parent, Some(id("sub_root")));
    assert_eq!(registrar.root(TEN_A).unwrap().id, id("sub_root"));
}

#[test]
fn a_record_whose_path_contradicts_its_parent_is_refused_not_stored() {
    let registrar = seeded();
    let liar = SubScope {
        id: id("sub_liar"),
        tenant_id: TEN_A.to_owned(),
        kind: SubScopeKind::Workspace,
        parent: Some(id("sub_emea")),
        // Real parent path is acme/emea; this claims to hang off the root.
        path: path(&["acme", "liar"]),
    };
    assert_eq!(
        registrar.register(&liar),
        Err(SubScopeKernelError::PathInconsistent)
    );
    assert_eq!(
        registrar.resolve(TEN_A, &id("sub_liar")),
        Err(SubScopeKernelError::NotFound),
        "a refused record must never reach the store"
    );

    // A non-normalized path is equally refused (tenant B has no root yet,
    // so RootImmutable cannot mask the path check).
    let unnormalized = SubScope {
        id: id("sub_case"),
        tenant_id: TEN_B.to_owned(),
        kind: SubScopeKind::Workspace,
        parent: None,
        path: path(&["ACME"]),
    };
    assert_eq!(
        registrar.register(&unnormalized),
        Err(SubScopeKernelError::PathInconsistent)
    );
}

#[test]
fn the_tenant_root_is_immutable() {
    let registrar = seeded();
    let root = id("sub_root");

    // A second root is refused.
    assert_eq!(
        registrar.register_root(TEN_A, id("sub_root2"), SubScopeKind::BusinessUnit, "other"),
        Err(SubScopeKernelError::RootImmutable)
    );
    assert_eq!(
        registrar.reparent(TEN_A, &root, &id("sub_emea")),
        Err(SubScopeKernelError::RootImmutable)
    );
    assert_eq!(
        registrar.rename(TEN_A, &root, "newname"),
        Err(SubScopeKernelError::RootImmutable)
    );
    assert_eq!(
        registrar.remove(TEN_A, &root),
        Err(SubScopeKernelError::RootImmutable)
    );
    assert_eq!(registrar.root(TEN_A).unwrap().path, path(&["acme"]));
}

#[test]
fn a_foreign_parent_is_indistinguishable_from_an_absent_one() {
    let registrar = seeded();
    registrar
        .register_root(TEN_B, id("sub_broot"), SubScopeKind::BusinessUnit, "globex")
        .unwrap();

    // Tenant B tries to hang a scope under tenant A's business unit. The
    // write is refused — and refused with the SAME answer as a parent id
    // that exists nowhere at all. Answering `TenantBoundaryViolation` here
    // would be an oracle: guess ids, and the response tells you which ones
    // some other tenant owns.
    let foreign = registrar.register_child(
        TEN_B,
        id("sub_theft"),
        SubScopeKind::Workspace,
        &id("sub_emea"),
        "theft",
    );
    let unknown = registrar.register_child(
        TEN_B,
        id("sub_theft"),
        SubScopeKind::Workspace,
        &id("sub_no_such_id_anywhere"),
        "theft",
    );
    assert_eq!(foreign, Err(SubScopeKernelError::NotFound));
    assert_eq!(
        foreign, unknown,
        "a live foreign id and a dead id must be indistinguishable to a probe"
    );
    assert_eq!(
        registrar.resolve(TEN_B, &id("sub_theft")),
        Err(SubScopeKernelError::NotFound),
        "a refused record must never reach the store"
    );

    // And tenant B cannot move one of its own scopes under tenant A either —
    // again reported as absent, on the write path as on the read path.
    registrar
        .register_child(
            TEN_B,
            id("sub_bteam"),
            SubScopeKind::Workspace,
            &id("sub_broot"),
            "bteam",
        )
        .unwrap();
    assert_eq!(
        registrar.reparent(TEN_B, &id("sub_bteam"), &id("sub_emea")),
        Err(SubScopeKernelError::NotFound)
    );
    assert_eq!(
        registrar.resolve(TEN_B, &id("sub_bteam")).unwrap().path,
        path(&["globex", "bteam"]),
        "the refused move left the scope where it was"
    );
    // A blank tenant id is unattributable, therefore outside every tenant.
    assert_eq!(
        registrar.resolve("   ", &id("sub_emea")),
        Err(SubScopeKernelError::TenantBoundaryViolation)
    );
}

#[test]
fn every_read_path_is_tenant_scoped() {
    let registrar = seeded();
    registrar
        .register_child(
            TEN_A,
            id("sub_atlas"),
            SubScopeKind::Project,
            &id("sub_emea"),
            "atlas",
        )
        .unwrap();
    registrar
        .register_root(TEN_B, id("sub_broot"), SubScopeKind::BusinessUnit, "globex")
        .unwrap();

    // The archetypal bug: tenant B resolving tenant A's scope.
    for probe in [&id("sub_root"), &id("sub_emea"), &id("sub_atlas")] {
        assert_eq!(
            registrar.resolve(TEN_B, probe),
            Err(SubScopeKernelError::NotFound),
            "tenant B must not resolve {probe}"
        );
        assert_eq!(
            registrar.ancestors(TEN_B, probe),
            Err(SubScopeKernelError::NotFound)
        );
        assert_eq!(
            registrar.descendants(TEN_B, probe),
            Err(SubScopeKernelError::NotFound)
        );
    }
    // Tenant B's own view stays intact.
    assert_eq!(
        registrar.descendants(TEN_B, &id("sub_broot")).unwrap(),
        Vec::<SubScopeId>::new()
    );
}

#[test]
fn a_reused_id_string_never_resolves_across_tenants() {
    let registrar = SubScopeRegistrar::new(InMemorySubScopeRegistry::new());
    // Ids are tenant-local, so both tenants legitimately mint "sub_root".
    registrar
        .register_root(TEN_A, id("sub_root"), SubScopeKind::BusinessUnit, "acme")
        .unwrap();
    registrar
        .register_root(TEN_B, id("sub_root"), SubScopeKind::BusinessUnit, "globex")
        .unwrap();

    assert_eq!(
        registrar.resolve(TEN_A, &id("sub_root")).unwrap().path,
        path(&["acme"])
    );
    assert_eq!(
        registrar.resolve(TEN_B, &id("sub_root")).unwrap().path,
        path(&["globex"])
    );
    // The raw, tenant-less read refuses the ambiguity instead of guessing.
    assert_eq!(
        registrar.registry().get(&id("sub_root")),
        Err(SubScopeKernelError::TenantBoundaryViolation)
    );
}

/// A deliberately leaky store: `get` ignores the tenant and the traversals
/// return foreign ids. The port's tenant-scoped defaults must contain it.
#[derive(Debug)]
struct LeakyStore {
    records: Vec<SubScope>,
}

impl SubScopeRegistryPort for LeakyStore {
    fn insert(&self, _scope: &SubScope) -> Result<(), SubScopeKernelError> {
        Err(SubScopeKernelError::PersistenceUnavailable)
    }

    fn get(&self, id: &SubScopeId) -> Result<Option<SubScope>, SubScopeKernelError> {
        Ok(self.records.iter().find(|record| &record.id == id).cloned())
    }

    fn replace(&self, _scope: &SubScope) -> Result<(), SubScopeKernelError> {
        Err(SubScopeKernelError::PersistenceUnavailable)
    }

    fn delete(&self, _tenant_id: &str, _id: &SubScopeId) -> Result<(), SubScopeKernelError> {
        Err(SubScopeKernelError::PersistenceUnavailable)
    }

    fn list_tenant(&self, tenant_id: &str) -> Result<Vec<SubScope>, SubScopeKernelError> {
        Ok(self
            .records
            .iter()
            .filter(|record| record.tenant_id == tenant_id)
            .cloned()
            .collect())
    }
}

impl SubScopeHierarchyReadPort for LeakyStore {
    fn ancestors(&self, _id: &SubScopeId) -> Result<Vec<SubScopeId>, SubScopeKernelError> {
        Ok(self
            .records
            .iter()
            .map(|record| record.id.clone())
            .collect())
    }

    fn descendants(&self, _id: &SubScopeId) -> Result<Vec<SubScopeId>, SubScopeKernelError> {
        Ok(self
            .records
            .iter()
            .map(|record| record.id.clone())
            .collect())
    }
}

#[test]
fn tenant_scoped_reads_ignore_a_leaky_raw_traversal() {
    let record = |tenant: &str, name: &str, parent: Option<&str>, segments: &[&str]| SubScope {
        id: id(name),
        tenant_id: tenant.to_owned(),
        kind: SubScopeKind::Workspace,
        parent: parent.map(id),
        path: path(segments),
    };
    // Both tenants use the SAME path strings, so a filter that looked at
    // paths instead of tenants would fail this test.
    let mine = record(TEN_A, "sub_mine", None, &["acme"]);
    let child = record(TEN_A, "sub_child", Some("sub_mine"), &["acme", "ws"]);
    let theirs = record(TEN_B, "sub_theirs", None, &["acme"]);
    let their_child = record(TEN_B, "sub_theirchild", Some("sub_theirs"), &["acme", "ws"]);
    let store = LeakyStore {
        records: vec![
            mine.clone(),
            child.clone(),
            theirs.clone(),
            their_child.clone(),
        ],
    };

    // The raw surfaces leak everything, by construction.
    assert_eq!(store.get(&theirs.id).unwrap(), Some(theirs.clone()));
    assert_eq!(store.descendants(&mine.id).unwrap().len(), 4);
    // The tenant-scoped surfaces do not consult them at all.
    assert_eq!(store.get_in_tenant(TEN_A, &theirs.id).unwrap(), None);
    assert_eq!(
        store.descendants_in_tenant(TEN_A, &mine.id).unwrap(),
        vec![child.id.clone()],
        "only this tenant's records, and a scope is not its own descendant"
    );
    assert_eq!(
        store.ancestors_in_tenant(TEN_A, &mine.id).unwrap(),
        Vec::<SubScopeId>::new(),
        "a root has no ancestors, least of all itself"
    );
    assert_eq!(
        store.ancestors_in_tenant(TEN_A, &child.id).unwrap(),
        vec![mine.id.clone()]
    );
    assert_eq!(
        store.descendants_in_tenant(TEN_A, &theirs.id),
        Err(SubScopeKernelError::NotFound)
    );

    // `retain_tenant` stays the boundary filter for an adapter that DOES
    // override the tenant-scoped reads with a native query.
    assert_eq!(
        store
            .retain_tenant(
                TEN_A,
                vec![mine.id.clone(), theirs.id.clone(), child.id.clone()],
            )
            .unwrap(),
        vec![mine.id, child.id]
    );
}

#[test]
fn a_scope_may_not_become_its_own_parent() {
    let registrar = seeded();
    assert_eq!(
        registrar.reparent(TEN_A, &id("sub_emea"), &id("sub_emea")),
        Err(SubScopeKernelError::CycleRefused)
    );
}

#[test]
fn a_long_indirect_cycle_is_refused() {
    let registrar = seeded();
    let deepest = chain(&registrar, 4);
    // sub_d1 .. sub_d4 hang in a line below the root; moving sub_d1 under
    // sub_d4 would make sub_d1 its own ancestor four edges away.
    assert_eq!(
        registrar.reparent(TEN_A, &id("sub_d1"), &deepest),
        Err(SubScopeKernelError::CycleRefused)
    );
    // The tree is untouched by the refusal.
    assert_eq!(
        registrar.ancestors(TEN_A, &deepest).unwrap(),
        vec![id("sub_d3"), id("sub_d2"), id("sub_d1"), id("sub_root")]
    );
}

#[test]
fn depth_is_bounded_at_the_documented_maximum() {
    let registrar = seeded();
    let deepest = chain(&registrar, MAX_DEPTH);
    assert_eq!(
        registrar.resolve(TEN_A, &deepest).unwrap().depth(),
        MAX_DEPTH,
        "MAX_DEPTH edges from the root must be allowed"
    );
    assert_eq!(
        registrar.register_child(
            TEN_A,
            id("sub_toodeep"),
            SubScopeKind::Workspace,
            &deepest,
            "toodeep",
        ),
        Err(SubScopeKernelError::DepthExceeded),
        "MAX_DEPTH + 1 must be refused"
    );
}

#[test]
fn a_move_is_judged_by_its_deepest_descendant() {
    let registrar = seeded();
    chain(&registrar, 4);
    // A three-deep subtree hanging directly off the root: s1/s2/s3.
    let mut parent = id("sub_root");
    for step in 1..=3 {
        let child = id(&format!("sub_s{step}"));
        registrar
            .register_child(
                TEN_A,
                child.clone(),
                SubScopeKind::Workspace,
                &parent,
                &format!("s{step}"),
            )
            .unwrap();
        parent = child;
    }

    // sub_d4 is at depth 4; the subtree adds 2 more below its root -> 4+1+2=7.
    assert_eq!(
        registrar.reparent(TEN_A, &id("sub_s1"), &id("sub_d4")),
        Err(SubScopeKernelError::DepthExceeded)
    );
    // Under sub_d3 (depth 3) the same subtree lands exactly on MAX_DEPTH.
    let moved = registrar
        .reparent(TEN_A, &id("sub_s1"), &id("sub_d3"))
        .unwrap();
    assert_eq!(moved.path, path(&["acme", "d1", "d2", "d3", "s1"]));
    assert_eq!(
        registrar.resolve(TEN_A, &id("sub_s3")).unwrap().depth(),
        MAX_DEPTH
    );
}

#[test]
fn reparent_rewrites_the_whole_subtree_path() {
    let registrar = seeded();
    registrar
        .register_child(
            TEN_A,
            id("sub_apac"),
            SubScopeKind::BusinessUnit,
            &id("sub_root"),
            "apac",
        )
        .unwrap();
    registrar
        .register_child(
            TEN_A,
            id("sub_atlas"),
            SubScopeKind::Project,
            &id("sub_emea"),
            "atlas",
        )
        .unwrap();
    registrar
        .register_child(
            TEN_A,
            id("sub_task"),
            SubScopeKind::Custom,
            &id("sub_atlas"),
            "task",
        )
        .unwrap();

    registrar
        .reparent(TEN_A, &id("sub_atlas"), &id("sub_apac"))
        .unwrap();

    assert_eq!(
        registrar.resolve(TEN_A, &id("sub_atlas")).unwrap().path,
        path(&["acme", "apac", "atlas"])
    );
    assert_eq!(
        registrar.resolve(TEN_A, &id("sub_task")).unwrap().path,
        path(&["acme", "apac", "atlas", "task"]),
        "a descendant path must follow its moved ancestor"
    );
    assert_eq!(
        registrar.ancestors(TEN_A, &id("sub_task")).unwrap(),
        vec![id("sub_atlas"), id("sub_apac"), id("sub_root")]
    );
}

#[test]
fn rename_normalizes_and_rewrites_descendants() {
    let registrar = seeded();
    registrar
        .register_child(
            TEN_A,
            id("sub_atlas"),
            SubScopeKind::Project,
            &id("sub_emea"),
            "atlas",
        )
        .unwrap();

    let renamed = registrar
        .rename(TEN_A, &id("sub_emea"), "  EMEA-North ")
        .unwrap();
    assert_eq!(renamed.path, path(&["acme", "emea-north"]));
    assert_eq!(
        registrar.resolve(TEN_A, &id("sub_atlas")).unwrap().path,
        path(&["acme", "emea-north", "atlas"])
    );
    assert_eq!(
        registrar.rename(TEN_A, &id("sub_emea"), "emea/north"),
        Err(SubScopeKernelError::NamespaceMalformed)
    );
}

#[test]
fn duplicate_ids_and_duplicate_sibling_paths_are_refused() {
    let registrar = seeded();
    assert_eq!(
        registrar.register_child(
            TEN_A,
            id("sub_emea"),
            SubScopeKind::Workspace,
            &id("sub_root"),
            "other",
        ),
        Err(SubScopeKernelError::DuplicateScope),
        "an id may be used once per tenant"
    );
    assert_eq!(
        registrar.register_child(
            TEN_A,
            id("sub_emea2"),
            SubScopeKind::Workspace,
            &id("sub_root"),
            "emea",
        ),
        Err(SubScopeKernelError::DuplicateScope),
        "two siblings may not share a normalized segment"
    );
    // Case folding means EMEA and emea are the same sibling name.
    assert_eq!(
        registrar.register_child(
            TEN_A,
            id("sub_emea3"),
            SubScopeKind::Workspace,
            &id("sub_root"),
            "eMeA",
        ),
        Err(SubScopeKernelError::DuplicateScope)
    );
}

#[test]
fn the_parent_kind_table_is_enforced_on_write() {
    let registrar = seeded();
    registrar
        .register_child(
            TEN_A,
            id("sub_deal"),
            SubScopeKind::Engagement,
            &id("sub_emea"),
            "deal",
        )
        .unwrap();
    registrar
        .register_child(
            TEN_A,
            id("sub_party"),
            SubScopeKind::Counterparty,
            &id("sub_emea"),
            "party",
        )
        .unwrap();

    // A business unit may not hang under an engagement.
    assert_eq!(
        registrar.register_child(
            TEN_A,
            id("sub_dept"),
            SubScopeKind::BusinessUnit,
            &id("sub_deal"),
            "dept",
        ),
        Err(SubScopeKernelError::ParentKindNotAllowed)
    );
    // A counterparty is terminal.
    assert_eq!(
        registrar.register_child(
            TEN_A,
            id("sub_room"),
            SubScopeKind::Investigation,
            &id("sub_party"),
            "room",
        ),
        Err(SubScopeKernelError::ParentKindNotAllowed)
    );
    // The rule is enforced on a move as well, not only on creation: this
    // business unit is neither an ancestor nor a descendant of the
    // engagement, so nothing but the kind table can refuse the move.
    registrar
        .register_child(
            TEN_A,
            id("sub_apac"),
            SubScopeKind::BusinessUnit,
            &id("sub_root"),
            "apac",
        )
        .unwrap();
    assert_eq!(
        registrar.reparent(TEN_A, &id("sub_apac"), &id("sub_deal")),
        Err(SubScopeKernelError::ParentKindNotAllowed)
    );
    // A cycle outranks the kind table when both would refuse.
    assert_eq!(
        registrar.reparent(TEN_A, &id("sub_emea"), &id("sub_deal")),
        Err(SubScopeKernelError::CycleRefused)
    );
}

#[test]
fn malformed_segments_and_unknown_ids_are_refused() {
    let registrar = seeded();
    assert_eq!(
        registrar.register_child(
            TEN_A,
            id("sub_bad"),
            SubScopeKind::Workspace,
            &id("sub_root"),
            "Bad Name",
        ),
        Err(SubScopeKernelError::NamespaceMalformed)
    );
    assert_eq!(
        registrar.register_root(TEN_B, id("sub_broot"), SubScopeKind::BusinessUnit, ""),
        Err(SubScopeKernelError::NamespaceMalformed)
    );
    assert_eq!(
        registrar.resolve(TEN_A, &id("sub_nope")),
        Err(SubScopeKernelError::NotFound)
    );
    assert_eq!(
        registrar.register_child(
            TEN_A,
            id("sub_orphan"),
            SubScopeKind::Workspace,
            &id("sub_nope"),
            "orphan",
        ),
        Err(SubScopeKernelError::NotFound)
    );
    assert_eq!(registrar.root(TEN_B), Err(SubScopeKernelError::NotFound));
}

#[test]
fn traversal_order_is_deterministic() {
    let registrar = seeded();
    for (child, segment) in [("sub_zulu", "zulu"), ("sub_alpha", "alpha")] {
        registrar
            .register_child(
                TEN_A,
                id(child),
                SubScopeKind::Workspace,
                &id("sub_emea"),
                segment,
            )
            .unwrap();
    }
    registrar
        .register_child(
            TEN_A,
            id("sub_deep"),
            SubScopeKind::Custom,
            &id("sub_alpha"),
            "deep",
        )
        .unwrap();

    // Pre-order by materialized path: alpha, alpha/deep, then zulu.
    assert_eq!(
        registrar.descendants(TEN_A, &id("sub_emea")).unwrap(),
        vec![id("sub_alpha"), id("sub_deep"), id("sub_zulu")]
    );
    // Ancestors are nearest-first, up to the root.
    assert_eq!(
        registrar.ancestors(TEN_A, &id("sub_deep")).unwrap(),
        vec![id("sub_alpha"), id("sub_emea"), id("sub_root")]
    );
    // Repeated reads agree.
    assert_eq!(
        registrar.descendants(TEN_A, &id("sub_emea")).unwrap(),
        registrar.descendants(TEN_A, &id("sub_emea")).unwrap()
    );
}

#[test]
fn the_edge_projection_is_the_closure_table_seed() {
    let registrar = seeded();
    registrar
        .register_child(
            TEN_A,
            id("sub_atlas"),
            SubScopeKind::Project,
            &id("sub_emea"),
            "atlas",
        )
        .unwrap();
    let edges = registrar.edges(TEN_A).unwrap();
    assert_eq!(edges.len(), 2, "the root contributes no edge");
    assert_eq!(edges[0].parent, id("sub_emea"));
    assert_eq!(edges[0].child, id("sub_atlas"));
    assert_eq!(edges[1].parent, id("sub_root"));
    assert_eq!(edges[1].child, id("sub_emea"));
    assert_eq!(registrar.edges(TEN_B).unwrap(), Vec::new());
}

#[test]
fn remove_deletes_the_whole_subtree() {
    let registrar = seeded();
    registrar
        .register_child(
            TEN_A,
            id("sub_atlas"),
            SubScopeKind::Project,
            &id("sub_emea"),
            "atlas",
        )
        .unwrap();
    registrar
        .register_child(
            TEN_A,
            id("sub_task"),
            SubScopeKind::Custom,
            &id("sub_atlas"),
            "task",
        )
        .unwrap();

    assert_eq!(registrar.remove(TEN_A, &id("sub_emea")).unwrap(), 3);
    for gone in ["sub_emea", "sub_atlas", "sub_task"] {
        assert_eq!(
            registrar.resolve(TEN_A, &id(gone)),
            Err(SubScopeKernelError::NotFound)
        );
    }
    assert_eq!(registrar.registry().len().unwrap(), 1);
    // Another tenant's scope cannot be removed through this tenant.
    registrar
        .register_root(TEN_B, id("sub_broot"), SubScopeKind::BusinessUnit, "globex")
        .unwrap();
    assert_eq!(
        registrar.remove(TEN_A, &id("sub_broot")),
        Err(SubScopeKernelError::NotFound)
    );
}

/// A store that is simply down.
#[derive(Debug)]
struct DownStore;

impl SubScopeRegistryPort for DownStore {
    fn insert(&self, _scope: &SubScope) -> Result<(), SubScopeKernelError> {
        Err(SubScopeKernelError::PersistenceUnavailable)
    }
    fn get(&self, _id: &SubScopeId) -> Result<Option<SubScope>, SubScopeKernelError> {
        Err(SubScopeKernelError::PersistenceUnavailable)
    }
    fn replace(&self, _scope: &SubScope) -> Result<(), SubScopeKernelError> {
        Err(SubScopeKernelError::PersistenceUnavailable)
    }
    fn delete(&self, _tenant_id: &str, _id: &SubScopeId) -> Result<(), SubScopeKernelError> {
        Err(SubScopeKernelError::PersistenceUnavailable)
    }
    fn list_tenant(&self, _tenant_id: &str) -> Result<Vec<SubScope>, SubScopeKernelError> {
        Err(SubScopeKernelError::PersistenceUnavailable)
    }
}

impl SubScopeHierarchyReadPort for DownStore {
    fn ancestors(&self, _id: &SubScopeId) -> Result<Vec<SubScopeId>, SubScopeKernelError> {
        Err(SubScopeKernelError::PersistenceUnavailable)
    }
    fn descendants(&self, _id: &SubScopeId) -> Result<Vec<SubScopeId>, SubScopeKernelError> {
        Err(SubScopeKernelError::PersistenceUnavailable)
    }
}

#[test]
fn store_outages_surface_as_persistence_unavailable() {
    let registrar = SubScopeRegistrar::new(DownStore);
    assert_eq!(
        registrar.register_root(TEN_A, id("sub_root"), SubScopeKind::BusinessUnit, "acme"),
        Err(SubScopeKernelError::PersistenceUnavailable)
    );
    assert_eq!(
        registrar.resolve(TEN_A, &id("sub_root")),
        Err(SubScopeKernelError::PersistenceUnavailable)
    );
    assert_eq!(
        registrar.descendants(TEN_A, &id("sub_root")),
        Err(SubScopeKernelError::PersistenceUnavailable)
    );
    assert_eq!(
        registrar.remove(TEN_A, &id("sub_root")),
        Err(SubScopeKernelError::PersistenceUnavailable)
    );
}

#[test]
fn a_tenant_local_id_reused_elsewhere_leaves_both_trees_fully_operable() {
    let registrar = SubScopeRegistrar::new(InMemorySubScopeRegistry::new());
    // Ids AND segments are tenant-local, so both tenants legitimately mint
    // the identical tree. Nothing either does may degrade the other.
    for tenant in [TEN_A, TEN_B] {
        registrar
            .register_root(tenant, id("sub_root"), SubScopeKind::BusinessUnit, "acme")
            .unwrap();
        registrar
            .register_child(
                tenant,
                id("sub_ws"),
                SubScopeKind::Workspace,
                &id("sub_root"),
                "ws",
            )
            .unwrap();
    }
    registrar
        .register_child(
            TEN_A,
            id("sub_proj"),
            SubScopeKind::Project,
            &id("sub_ws"),
            "proj",
        )
        .unwrap();
    registrar
        .register_child(
            TEN_A,
            id("sub_apac"),
            SubScopeKind::BusinessUnit,
            &id("sub_root"),
            "apac",
        )
        .unwrap();

    // Every hierarchy operation on a colliding id still works for its owner.
    assert_eq!(
        registrar.ancestors(TEN_A, &id("sub_ws")).unwrap(),
        vec![id("sub_root")]
    );
    assert_eq!(
        registrar.descendants(TEN_A, &id("sub_ws")).unwrap(),
        vec![id("sub_proj")]
    );
    assert_eq!(
        registrar.rename(TEN_A, &id("sub_ws"), "ws2").unwrap().path,
        path(&["acme", "ws2"])
    );
    assert_eq!(
        registrar
            .reparent(TEN_A, &id("sub_ws"), &id("sub_apac"))
            .unwrap()
            .path,
        path(&["acme", "apac", "ws2"])
    );
    assert_eq!(
        registrar.resolve(TEN_A, &id("sub_proj")).unwrap().path,
        path(&["acme", "apac", "ws2", "proj"]),
        "the descendant followed its moved ancestor"
    );
    assert_eq!(registrar.remove(TEN_A, &id("sub_ws")).unwrap(), 2);

    // Tenant B never noticed any of it.
    assert_eq!(
        registrar.resolve(TEN_B, &id("sub_ws")).unwrap().path,
        path(&["acme", "ws"])
    );
    assert_eq!(
        registrar.ancestors(TEN_B, &id("sub_ws")).unwrap(),
        vec![id("sub_root")]
    );
    // ... and can still create children under its own colliding root.
    registrar
        .register_child(
            TEN_B,
            id("sub_bproj"),
            SubScopeKind::Project,
            &id("sub_ws"),
            "proj",
        )
        .unwrap();
    assert_eq!(
        registrar.descendants(TEN_B, &id("sub_root")).unwrap(),
        vec![id("sub_ws"), id("sub_bproj")]
    );
    // The raw, tenant-less read still refuses the ambiguity instead of
    // guessing — it is simply no longer on any tenant-scoped path.
    assert_eq!(
        registrar.registry().get(&id("sub_root")),
        Err(SubScopeKernelError::TenantBoundaryViolation)
    );
}

/// The reference store behind a port whose `replace` fails for a chosen
/// window of calls: a store that goes down in the middle of a batch.
///
/// It deliberately does NOT override `replace_all`, so the port's default
/// best-effort emulation — the code path any non-transactional adapter gets —
/// is what these tests exercise.
#[derive(Debug)]
struct FlakyReplace {
    inner: InMemorySubScopeRegistry,
    calls: AtomicUsize,
    fail_from: AtomicUsize,
    fail_until: AtomicUsize,
}

impl FlakyReplace {
    fn new() -> Self {
        Self {
            inner: InMemorySubScopeRegistry::new(),
            calls: AtomicUsize::new(0),
            fail_from: AtomicUsize::new(usize::MAX),
            fail_until: AtomicUsize::new(usize::MAX),
        }
    }

    /// Fail the replaces numbered `from..until`, counting from the next one.
    fn arm(&self, from: usize, until: usize) {
        self.calls.store(0, Ordering::SeqCst);
        self.fail_from.store(from, Ordering::SeqCst);
        self.fail_until.store(until, Ordering::SeqCst);
    }

    fn down(&self) -> bool {
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        call >= self.fail_from.load(Ordering::SeqCst)
            && call < self.fail_until.load(Ordering::SeqCst)
    }
}

impl SubScopeRegistryPort for FlakyReplace {
    fn insert(&self, scope: &SubScope) -> Result<(), SubScopeKernelError> {
        self.inner.insert(scope)
    }
    fn get(&self, id: &SubScopeId) -> Result<Option<SubScope>, SubScopeKernelError> {
        self.inner.get(id)
    }
    fn get_in_tenant(
        &self,
        tenant_id: &str,
        id: &SubScopeId,
    ) -> Result<Option<SubScope>, SubScopeKernelError> {
        self.inner.get_in_tenant(tenant_id, id)
    }
    fn replace(&self, scope: &SubScope) -> Result<(), SubScopeKernelError> {
        if self.down() {
            return Err(SubScopeKernelError::PersistenceUnavailable);
        }
        self.inner.replace(scope)
    }
    fn delete(&self, tenant_id: &str, id: &SubScopeId) -> Result<(), SubScopeKernelError> {
        self.inner.delete(tenant_id, id)
    }
    fn list_tenant(&self, tenant_id: &str) -> Result<Vec<SubScope>, SubScopeKernelError> {
        self.inner.list_tenant(tenant_id)
    }
}

impl SubScopeHierarchyReadPort for FlakyReplace {
    fn ancestors(&self, id: &SubScopeId) -> Result<Vec<SubScopeId>, SubScopeKernelError> {
        self.inner.ancestors(id)
    }
    fn descendants(&self, id: &SubScopeId) -> Result<Vec<SubScopeId>, SubScopeKernelError> {
        self.inner.descendants(id)
    }
}

/// `acme` > {`emea` > `atlas` > `task`, `apac`} over a flaky store.
fn flaky_tree() -> SubScopeRegistrar<FlakyReplace> {
    let registrar = SubScopeRegistrar::new(FlakyReplace::new());
    registrar
        .register_root(TEN_A, id("sub_root"), SubScopeKind::BusinessUnit, "acme")
        .unwrap();
    for (child, segment) in [("sub_emea", "emea"), ("sub_apac", "apac")] {
        registrar
            .register_child(
                TEN_A,
                id(child),
                SubScopeKind::BusinessUnit,
                &id("sub_root"),
                segment,
            )
            .unwrap();
    }
    registrar
        .register_child(
            TEN_A,
            id("sub_atlas"),
            SubScopeKind::Project,
            &id("sub_emea"),
            "atlas",
        )
        .unwrap();
    registrar
        .register_child(
            TEN_A,
            id("sub_task"),
            SubScopeKind::Custom,
            &id("sub_atlas"),
            "task",
        )
        .unwrap();
    registrar
}

#[test]
fn a_subtree_move_that_fails_part_way_is_rolled_back_whole() {
    let registrar = flaky_tree();
    // The batch is [atlas, task]; kill the write of the descendant, so the
    // subject's move has already been committed when the failure lands.
    registrar.registry().arm(1, 2);

    assert_eq!(
        registrar.reparent(TEN_A, &id("sub_atlas"), &id("sub_apac")),
        Err(SubScopeKernelError::PersistenceUnavailable),
        "an error here promises the caller that nothing happened"
    );

    // ... and nothing did. Both records sit exactly where they started.
    assert_eq!(
        registrar.resolve(TEN_A, &id("sub_atlas")).unwrap().path,
        path(&["acme", "emea", "atlas"])
    );
    assert_eq!(
        registrar.resolve(TEN_A, &id("sub_atlas")).unwrap().parent,
        Some(id("sub_emea"))
    );
    assert_eq!(
        registrar.resolve(TEN_A, &id("sub_task")).unwrap().path,
        path(&["acme", "emea", "atlas", "task"])
    );
    // The decisive one: `descendants` is a materialized-path prefix query, so
    // a drifted child drops out of its own parent's subtree and survives the
    // cascade delete. It has not drifted.
    assert_eq!(
        registrar.descendants(TEN_A, &id("sub_atlas")).unwrap(),
        vec![id("sub_task")]
    );
    assert_eq!(registrar.remove(TEN_A, &id("sub_emea")).unwrap(), 3);
    assert_eq!(
        registrar.resolve(TEN_A, &id("sub_task")),
        Err(SubScopeKernelError::NotFound),
        "no record may survive the deletion of its own ancestor"
    );
}

#[test]
fn a_rename_that_fails_part_way_is_rolled_back_whole() {
    let registrar = flaky_tree();
    // Batch is [emea, atlas, task]; kill the last write.
    registrar.registry().arm(2, 3);

    assert_eq!(
        registrar.rename(TEN_A, &id("sub_emea"), "emea-north"),
        Err(SubScopeKernelError::PersistenceUnavailable)
    );
    for (scope, segments) in [
        ("sub_emea", &["acme", "emea"][..]),
        ("sub_atlas", &["acme", "emea", "atlas"]),
        ("sub_task", &["acme", "emea", "atlas", "task"]),
    ] {
        assert_eq!(
            registrar.resolve(TEN_A, &id(scope)).unwrap().path,
            path(segments),
            "{scope} must be back at its original path"
        );
    }
    assert_eq!(
        registrar.descendants(TEN_A, &id("sub_emea")).unwrap(),
        vec![id("sub_atlas"), id("sub_task")]
    );
}

#[test]
fn a_failed_rollback_is_reported_and_never_disguised_as_no_op() {
    let registrar = flaky_tree();
    // The store goes down at the second write and STAYS down, so the
    // compensating restore cannot land either.
    registrar.registry().arm(1, usize::MAX);

    assert_eq!(
        registrar.reparent(TEN_A, &id("sub_atlas"), &id("sub_apac")),
        Err(SubScopeKernelError::PartialWriteUnresolved),
        "a half-written tree must not be reported as PersistenceUnavailable, \
         which promises that nothing happened"
    );
    assert_eq!(
        SubScopeKernelError::PartialWriteUnresolved.to_string(),
        "sub-scope subtree rewrite failed and could not be rolled back"
    );
    // The drift is real — which is precisely why the caller had to be told.
    assert_eq!(
        registrar.resolve(TEN_A, &id("sub_atlas")).unwrap().path,
        path(&["acme", "apac", "atlas"])
    );
    assert_eq!(
        registrar.resolve(TEN_A, &id("sub_task")).unwrap().path,
        path(&["acme", "emea", "atlas", "task"])
    );
}

#[test]
fn the_reference_store_commits_a_batch_all_or_nothing() {
    let registrar = seeded();
    registrar
        .register_child(
            TEN_A,
            id("sub_apac"),
            SubScopeKind::BusinessUnit,
            &id("sub_root"),
            "apac",
        )
        .unwrap();
    let store = registrar.registry();
    let emea = registrar.resolve(TEN_A, &id("sub_emea")).unwrap();
    let apac = registrar.resolve(TEN_A, &id("sub_apac")).unwrap();

    // One unknown record poisons the whole batch, and the valid record in
    // front of it is not applied.
    let ghost = SubScope {
        id: id("sub_ghost"),
        ..emea.clone()
    };
    let moved_emea = SubScope {
        path: path(&["acme", "emea-north"]),
        ..emea.clone()
    };
    assert_eq!(
        store.replace_all(&[moved_emea, ghost]),
        Err(SubScopeKernelError::NotFound)
    );
    assert_eq!(registrar.resolve(TEN_A, &emea.id).unwrap().path, emea.path);

    // A batch that would leave two records on one path is refused and undone.
    let collide = SubScope {
        path: apac.path.clone(),
        ..emea.clone()
    };
    assert_eq!(
        store.replace_all(&[collide]),
        Err(SubScopeKernelError::DuplicateScope)
    );
    assert_eq!(registrar.resolve(TEN_A, &emea.id).unwrap().path, emea.path);

    // But a straight SWAP is legal: uniqueness is judged on the post-state,
    // not on the transient state each individual write passes through.
    let swap_emea = SubScope {
        path: apac.path.clone(),
        ..emea.clone()
    };
    let swap_apac = SubScope {
        path: emea.path.clone(),
        ..apac.clone()
    };
    store.replace_all(&[swap_emea, swap_apac]).unwrap();
    assert_eq!(registrar.resolve(TEN_A, &emea.id).unwrap().path, apac.path);
    assert_eq!(registrar.resolve(TEN_A, &apac.id).unwrap().path, emea.path);
}

/// A store whose writes work while its tenant listing is down — a read
/// replica lagging behind the primary it is written through.
#[derive(Debug)]
struct ReadOutageStore {
    inner: InMemorySubScopeRegistry,
    blind: AtomicBool,
}

impl ReadOutageStore {
    fn new() -> Self {
        Self {
            inner: InMemorySubScopeRegistry::new(),
            blind: AtomicBool::new(false),
        }
    }
}

impl SubScopeRegistryPort for ReadOutageStore {
    fn insert(&self, scope: &SubScope) -> Result<(), SubScopeKernelError> {
        self.inner.insert(scope)
    }
    fn get(&self, id: &SubScopeId) -> Result<Option<SubScope>, SubScopeKernelError> {
        self.inner.get(id)
    }
    fn get_in_tenant(
        &self,
        tenant_id: &str,
        id: &SubScopeId,
    ) -> Result<Option<SubScope>, SubScopeKernelError> {
        self.inner.get_in_tenant(tenant_id, id)
    }
    fn replace(&self, scope: &SubScope) -> Result<(), SubScopeKernelError> {
        self.inner.replace(scope)
    }
    fn delete(&self, tenant_id: &str, id: &SubScopeId) -> Result<(), SubScopeKernelError> {
        self.inner.delete(tenant_id, id)
    }
    fn list_tenant(&self, tenant_id: &str) -> Result<Vec<SubScope>, SubScopeKernelError> {
        if self.blind.load(Ordering::SeqCst) {
            return Err(SubScopeKernelError::PersistenceUnavailable);
        }
        self.inner.list_tenant(tenant_id)
    }
}

impl SubScopeHierarchyReadPort for ReadOutageStore {
    fn ancestors(&self, id: &SubScopeId) -> Result<Vec<SubScopeId>, SubScopeKernelError> {
        self.inner.ancestors(id)
    }
    fn descendants(&self, id: &SubScopeId) -> Result<Vec<SubScopeId>, SubScopeKernelError> {
        self.inner.descendants(id)
    }
}

#[test]
fn a_read_outage_is_never_mistaken_for_an_absent_root() {
    let registrar = SubScopeRegistrar::new(ReadOutageStore::new());
    registrar
        .register_root(TEN_A, id("sub_root"), SubScopeKind::BusinessUnit, "acme")
        .unwrap();

    // The root exists; the store simply cannot say so right now. "I could not
    // answer" is not "there is none", so the second root is refused.
    registrar.registry().blind.store(true, Ordering::SeqCst);
    assert_eq!(
        registrar.register_root(TEN_A, id("sub_root2"), SubScopeKind::BusinessUnit, "other"),
        Err(SubScopeKernelError::PersistenceUnavailable),
        "a read outage must not be allowed to mint a second tenant root"
    );

    registrar.registry().blind.store(false, Ordering::SeqCst);
    let roots: Vec<SubScopeId> = registrar
        .registry()
        .list_tenant(TEN_A)
        .unwrap()
        .into_iter()
        .filter(SubScope::is_root)
        .map(|scope| scope.id)
        .collect();
    assert_eq!(roots, vec![id("sub_root")], "exactly one parentless root");
    assert_eq!(
        registrar.register_root(TEN_A, id("sub_root2"), SubScopeKind::BusinessUnit, "other"),
        Err(SubScopeKernelError::RootImmutable),
        "with the store back, the honest refusal is RootImmutable"
    );
}

/// A store that hands back corrupt records: an empty materialized path, and a
/// parent chain that loops. Its writes SUCCEED, so any refusal below comes
/// from the registrar's own guards and not from a convenient outage.
#[derive(Debug)]
struct CorruptStore {
    records: Vec<SubScope>,
}

impl SubScopeRegistryPort for CorruptStore {
    fn insert(&self, _scope: &SubScope) -> Result<(), SubScopeKernelError> {
        Ok(())
    }
    fn get(&self, id: &SubScopeId) -> Result<Option<SubScope>, SubScopeKernelError> {
        Ok(self.records.iter().find(|record| &record.id == id).cloned())
    }
    fn replace(&self, _scope: &SubScope) -> Result<(), SubScopeKernelError> {
        Ok(())
    }
    fn replace_all(&self, _scopes: &[SubScope]) -> Result<(), SubScopeKernelError> {
        Ok(())
    }
    fn delete(&self, _tenant_id: &str, _id: &SubScopeId) -> Result<(), SubScopeKernelError> {
        Ok(())
    }
    fn list_tenant(&self, tenant_id: &str) -> Result<Vec<SubScope>, SubScopeKernelError> {
        Ok(self
            .records
            .iter()
            .filter(|record| record.tenant_id == tenant_id)
            .cloned()
            .collect())
    }
}

impl SubScopeHierarchyReadPort for CorruptStore {
    fn ancestors(&self, _id: &SubScopeId) -> Result<Vec<SubScopeId>, SubScopeKernelError> {
        Ok(Vec::new())
    }
    fn descendants(&self, _id: &SubScopeId) -> Result<Vec<SubScopeId>, SubScopeKernelError> {
        Ok(Vec::new())
    }
}

#[test]
fn a_corrupt_record_is_refused_rather_than_moved() {
    let record = |name: &str, parent: Option<&str>, segments: &[&str]| SubScope {
        id: id(name),
        tenant_id: TEN_A.to_owned(),
        kind: SubScopeKind::Workspace,
        parent: parent.map(id),
        path: path(segments),
    };
    let store = CorruptStore {
        records: vec![
            record("sub_root", None, &["acme"]),
            // A stored record can never legally hold an empty path; this one
            // does, and the move must refuse it instead of deriving a leaf.
            record("sub_broken", Some("sub_root"), &[]),
        ],
    };
    let registrar = SubScopeRegistrar::new(store);
    assert_eq!(
        registrar.reparent(TEN_A, &id("sub_broken"), &id("sub_root")),
        Err(SubScopeKernelError::PathInconsistent),
        "the writes of this store all succeed, so only the guard can refuse"
    );
}

#[test]
fn a_looping_parent_chain_surfaces_as_a_cycle_not_a_hang() {
    let record = |name: &str, parent: &str| SubScope {
        id: id(name),
        tenant_id: TEN_A.to_owned(),
        kind: SubScopeKind::Workspace,
        parent: Some(id(parent)),
        path: path(&["acme", name]),
    };
    let store = CorruptStore {
        records: vec![
            record("sub_a", "sub_b"),
            record("sub_b", "sub_a"),
            record("sub_self", "sub_self"),
        ],
    };
    assert_eq!(
        store.ancestors_in_tenant(TEN_A, &id("sub_a")),
        Err(SubScopeKernelError::CycleRefused)
    );
    assert_eq!(
        store.ancestors_in_tenant(TEN_A, &id("sub_self")),
        Err(SubScopeKernelError::CycleRefused)
    );
}

#[test]
fn tenant_ids_are_held_to_a_documented_character_set() {
    let registrar = seeded();
    let long = "t".repeat(MAX_TENANT_ID_LEN + 1);
    for bad in [
        "",
        "   ",
        "ten a",
        "ten/a",
        "ten.a",
        "ten'; DROP TABLE sub_scopes; --",
        "ten\u{7}a",
        "ten\na",
        "ten\"a",
        "ten\\a",
        "_ten",
        "-ten",
        "t\u{e9}nant",
        long.as_str(),
    ] {
        assert_eq!(
            registrar.resolve(bad, &id("sub_root")),
            Err(SubScopeKernelError::TenantBoundaryViolation),
            "tenant id {bad:?} must be refused"
        );
        assert_eq!(
            registrar.register_root(bad, id("sub_x"), SubScopeKind::BusinessUnit, "acme"),
            Err(SubScopeKernelError::TenantBoundaryViolation),
            "tenant id {bad:?} must be refused on the write path too"
        );
    }
    let max = "t".repeat(MAX_TENANT_ID_LEN);
    for good in ["ten_unseeded", "TEN-A1", "t", max.as_str()] {
        assert_eq!(
            registrar.resolve(good, &id("sub_root")),
            Err(SubScopeKernelError::NotFound),
            "tenant id {good:?} is well-formed, merely empty"
        );
    }
    // Case is significant: a tenant id is opaque and is NOT folded.
    registrar
        .register_root(
            "TEN_ACME",
            id("sub_root"),
            SubScopeKind::BusinessUnit,
            "acme",
        )
        .unwrap();
    assert_eq!(
        registrar.resolve(TEN_A, &id("sub_root")).unwrap().tenant_id,
        TEN_A
    );
}
