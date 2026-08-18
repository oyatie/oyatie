use std::collections::BTreeMap;

use work_area_tree_kernel::{
    NodeContentHash, NodeKind, NodeLocator, SourceSpan, WorkAreaHash, WorkAreaNode, WorkAreaNodeId,
    WorkAreaTree, WorkAreaTreeError,
};

fn digest(seed: u8) -> [u8; 32] {
    let mut bytes = [0_u8; 32];
    bytes[0] = seed;
    bytes[31] = seed.wrapping_add(1);
    bytes
}

#[test]
fn node_identity_names_work_area_hash_node_hash_and_source_locator() {
    let work_area_hash = WorkAreaHash::from_bytes(digest(1));
    let node_hash = NodeContentHash::from_bytes(digest(2));
    let span = SourceSpan::new(10, 42).expect("valid span");
    let locator = NodeLocator::new("docs/decisions/ADR-0700-ci-admission-live-apex.md", span)
        .expect("valid locator");

    let node_id = WorkAreaNodeId::new(work_area_hash, node_hash, locator.clone());
    let duplicate_content_at_different_locator = WorkAreaNodeId::new(
        work_area_hash,
        node_hash,
        NodeLocator::new(
            "docs/decisions/ADR-0701-monorepo-capability-live-apex.md",
            SourceSpan::new(50, 72).expect("valid duplicate-content span"),
        )
        .expect("valid duplicate-content locator"),
    );

    assert_eq!(WorkAreaHash::ALGORITHM, "sha256");
    assert_eq!(NodeContentHash::ALGORITHM, "sha256");
    assert_eq!(node_id.work_area_hash().as_bytes(), &digest(1));
    assert_eq!(node_id.node_hash().as_bytes(), &digest(2));
    assert_eq!(node_id.locator(), &locator);
    assert_eq!(node_id.locator().span().start_byte(), 10);
    assert_eq!(node_id.locator().span().end_byte(), 42);
    assert_eq!(node_id.work_area_hash().to_hex().len(), 64);
    assert_eq!(
        duplicate_content_at_different_locator.work_area_hash(),
        node_id.work_area_hash()
    );
    assert_eq!(
        duplicate_content_at_different_locator.node_hash(),
        node_id.node_hash()
    );
    assert_ne!(duplicate_content_at_different_locator, node_id);
}

#[test]
fn source_span_rejects_empty_or_reversed_ranges() {
    assert_eq!(
        SourceSpan::new(7, 7),
        Err(WorkAreaTreeError::InvalidSpan {
            start_byte: 7,
            end_byte: 7,
        })
    );
    assert_eq!(
        SourceSpan::new(9, 8),
        Err(WorkAreaTreeError::InvalidSpan {
            start_byte: 9,
            end_byte: 8,
        })
    );
}

#[test]
fn work_area_tree_trait_exposes_consumer_seam_without_parser_impl() {
    let work_area_hash = WorkAreaHash::from_bytes(digest(10));
    let root = node_id(work_area_hash, 20, "Cargo.toml", 0, 120);
    let child = node_id(work_area_hash, 21, "Cargo.toml", 1, 20);
    let root_node = WorkAreaNode::new(root.clone(), NodeKind::Root);
    let child_node = WorkAreaNode::new(child.clone(), NodeKind::Syntax);

    let tree = FixtureTree {
        work_area_hash,
        root_id: root.clone(),
        nodes: BTreeMap::from([(root.clone(), root_node), (child.clone(), child_node)]),
        children: BTreeMap::from([(root.clone(), vec![child.clone()])]),
    };

    assert_eq!(tree.work_area_hash(), work_area_hash);
    assert_eq!(tree.root_id(), root);
    assert_eq!(
        tree.child_ids(&root).expect("root children"),
        vec![child.clone()]
    );
    assert_eq!(
        tree.node(&child).expect("child node").kind(),
        NodeKind::Syntax
    );
    assert_eq!(
        tree.node(&node_id(work_area_hash, 99, "missing.rs", 1, 2)),
        Err(WorkAreaTreeError::NodeNotFound)
    );
    assert_eq!(
        tree.child_ids(&node_id(work_area_hash, 100, "missing.rs", 3, 4)),
        Err(WorkAreaTreeError::NodeNotFound)
    );
}

fn node_id(
    work_area_hash: WorkAreaHash,
    node_seed: u8,
    path: &str,
    start_byte: u64,
    end_byte: u64,
) -> WorkAreaNodeId {
    let span = SourceSpan::new(start_byte, end_byte).expect("valid span");
    let locator = NodeLocator::new(path, span).expect("valid locator");
    WorkAreaNodeId::new(
        work_area_hash,
        NodeContentHash::from_bytes(digest(node_seed)),
        locator,
    )
}

struct FixtureTree {
    work_area_hash: WorkAreaHash,
    root_id: WorkAreaNodeId,
    nodes: BTreeMap<WorkAreaNodeId, WorkAreaNode>,
    children: BTreeMap<WorkAreaNodeId, Vec<WorkAreaNodeId>>,
}

impl WorkAreaTree for FixtureTree {
    fn work_area_hash(&self) -> WorkAreaHash {
        self.work_area_hash
    }

    fn root_id(&self) -> WorkAreaNodeId {
        self.root_id.clone()
    }

    fn node(&self, id: &WorkAreaNodeId) -> Result<WorkAreaNode, WorkAreaTreeError> {
        self.nodes
            .get(id)
            .cloned()
            .ok_or(WorkAreaTreeError::NodeNotFound)
    }

    fn child_ids(&self, id: &WorkAreaNodeId) -> Result<Vec<WorkAreaNodeId>, WorkAreaTreeError> {
        if !self.nodes.contains_key(id) {
            return Err(WorkAreaTreeError::NodeNotFound);
        }
        Ok(self.children.get(id).cloned().unwrap_or_default())
    }
}
