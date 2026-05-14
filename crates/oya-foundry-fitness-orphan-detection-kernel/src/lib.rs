//! Orphan-detection fitness kernel — flag plan/spec nodes that no
//! other node points to (i.e., unreachable from any declared root).
//!
//! I/O-free. Runners enumerate the plan tree, build typed
//! [`PlanReference`] edges, and call [`detect_orphans`].

/// A reference edge: `from` declares a pointer to `to` (a path).
/// Both are repo-relative paths.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanReference {
    pub from: String, // data_class: INTERNAL_ONLY
    pub to: String,   // data_class: INTERNAL_ONLY
}

/// A single node in the plan tree, identified by its repo-relative path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanNodePath {
    pub path: String,  // data_class: INTERNAL_ONLY
    pub is_root: bool, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrphanDetectionReport {
    pub nodes_checked: usize, // data_class: INTERNAL_ONLY
    pub roots_checked: usize, // data_class: INTERNAL_ONLY
    pub orphans: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OrphanDetectionError {
    NoRoots,
    EmptyPath,
    DuplicateNode { path: String },
    DanglingReference { from: String, to: String },
}

impl OrphanDetectionError {
    pub fn message(&self) -> String {
        match self {
            Self::NoRoots => "no nodes marked as root".to_owned(),
            Self::EmptyPath => "empty path in plan node".to_owned(),
            Self::DuplicateNode { path } => format!("duplicate plan node: {path}"),
            Self::DanglingReference { from, to } => {
                format!("reference from {from} points to unknown node {to}")
            }
        }
    }
}

/// Detect orphans: nodes unreachable from any declared root.
///
/// `references` are edges `from → to`; nodes are reachable iff there
/// is a path from some root to them via these edges.
pub fn detect_orphans(
    nodes: &[PlanNodePath],
    references: &[PlanReference],
) -> Result<OrphanDetectionReport, OrphanDetectionError> {
    use std::collections::{BTreeMap, BTreeSet, VecDeque};

    let mut by_path: BTreeMap<&str, &PlanNodePath> = BTreeMap::new();
    let mut roots: Vec<&str> = Vec::new();

    for node in nodes {
        if node.path.is_empty() {
            return Err(OrphanDetectionError::EmptyPath);
        }
        if by_path.insert(node.path.as_str(), node).is_some() {
            return Err(OrphanDetectionError::DuplicateNode {
                path: node.path.clone(),
            });
        }
        if node.is_root {
            roots.push(node.path.as_str());
        }
    }

    if roots.is_empty() {
        return Err(OrphanDetectionError::NoRoots);
    }

    let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for r in references {
        if !by_path.contains_key(r.to.as_str()) {
            return Err(OrphanDetectionError::DanglingReference {
                from: r.from.clone(),
                to: r.to.clone(),
            });
        }
        adjacency
            .entry(r.from.as_str())
            .or_default()
            .push(r.to.as_str());
    }

    let mut visited: BTreeSet<&str> = BTreeSet::new();
    let mut queue: VecDeque<&str> = VecDeque::new();
    for r in &roots {
        if visited.insert(*r) {
            queue.push_back(*r);
        }
    }
    while let Some(cur) = queue.pop_front() {
        if let Some(targets) = adjacency.get(cur) {
            for t in targets {
                if visited.insert(*t) {
                    queue.push_back(*t);
                }
            }
        }
    }

    let mut orphans: Vec<String> = nodes
        .iter()
        .filter(|n| !visited.contains(n.path.as_str()))
        .map(|n| n.path.clone())
        .collect();
    orphans.sort();

    Ok(OrphanDetectionReport {
        nodes_checked: nodes.len(),
        roots_checked: roots.len(),
        orphans,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(path: &str, is_root: bool) -> PlanNodePath {
        PlanNodePath {
            path: path.into(),
            is_root,
        }
    }
    fn edge(from: &str, to: &str) -> PlanReference {
        PlanReference {
            from: from.into(),
            to: to.into(),
        }
    }

    #[test]
    fn root_only_no_orphans() {
        let r = detect_orphans(&[node("root.md", true)], &[]).unwrap();
        assert!(r.orphans.is_empty());
        assert_eq!(r.roots_checked, 1);
    }

    #[test]
    fn reachable_chain_no_orphans() {
        let nodes = vec![
            node("root.md", true),
            node("a.md", false),
            node("b.md", false),
        ];
        let edges = vec![edge("root.md", "a.md"), edge("a.md", "b.md")];
        let r = detect_orphans(&nodes, &edges).unwrap();
        assert!(r.orphans.is_empty());
    }

    #[test]
    fn unreachable_node_is_orphan() {
        let nodes = vec![
            node("root.md", true),
            node("a.md", false),
            node("orphan.md", false),
        ];
        let edges = vec![edge("root.md", "a.md")];
        let r = detect_orphans(&nodes, &edges).unwrap();
        assert_eq!(r.orphans, vec!["orphan.md".to_owned()]);
    }

    #[test]
    fn multiple_orphans_sorted() {
        let nodes = vec![
            node("root.md", true),
            node("z.md", false),
            node("a.md", false),
        ];
        let r = detect_orphans(&nodes, &[]).unwrap();
        assert_eq!(r.orphans, vec!["a.md".to_owned(), "z.md".to_owned()]);
    }

    #[test]
    fn two_roots_both_reach_their_subtrees() {
        let nodes = vec![
            node("root-a.md", true),
            node("root-b.md", true),
            node("child-a.md", false),
            node("child-b.md", false),
        ];
        let edges = vec![
            edge("root-a.md", "child-a.md"),
            edge("root-b.md", "child-b.md"),
        ];
        let r = detect_orphans(&nodes, &edges).unwrap();
        assert!(r.orphans.is_empty());
    }

    #[test]
    fn cycle_is_handled() {
        let nodes = vec![
            node("root.md", true),
            node("a.md", false),
            node("b.md", false),
        ];
        let edges = vec![
            edge("root.md", "a.md"),
            edge("a.md", "b.md"),
            edge("b.md", "a.md"),
        ];
        let r = detect_orphans(&nodes, &edges).unwrap();
        assert!(r.orphans.is_empty());
    }

    #[test]
    fn no_roots_errors() {
        let err = detect_orphans(&[node("a.md", false)], &[]).unwrap_err();
        assert!(matches!(err, OrphanDetectionError::NoRoots));
    }

    #[test]
    fn empty_path_errors() {
        let err = detect_orphans(&[node("", true)], &[]).unwrap_err();
        assert!(matches!(err, OrphanDetectionError::EmptyPath));
    }

    #[test]
    fn duplicate_node_errors() {
        let err = detect_orphans(&[node("root.md", true), node("root.md", true)], &[]).unwrap_err();
        assert!(matches!(err, OrphanDetectionError::DuplicateNode { .. }));
    }

    #[test]
    fn dangling_reference_errors() {
        let nodes = vec![node("root.md", true)];
        let edges = vec![edge("root.md", "ghost.md")];
        let err = detect_orphans(&nodes, &edges).unwrap_err();
        assert!(matches!(
            err,
            OrphanDetectionError::DanglingReference { .. }
        ));
    }

    #[test]
    fn diamond_graph_no_orphans() {
        let nodes = vec![
            node("root.md", true),
            node("a.md", false),
            node("b.md", false),
            node("c.md", false),
        ];
        let edges = vec![
            edge("root.md", "a.md"),
            edge("root.md", "b.md"),
            edge("a.md", "c.md"),
            edge("b.md", "c.md"),
        ];
        let r = detect_orphans(&nodes, &edges).unwrap();
        assert!(r.orphans.is_empty());
    }

    #[test]
    fn empty_input_with_root_only() {
        let r = detect_orphans(&[node("r.md", true)], &[]).unwrap();
        assert_eq!(r.nodes_checked, 1);
        assert_eq!(r.roots_checked, 1);
    }
}
