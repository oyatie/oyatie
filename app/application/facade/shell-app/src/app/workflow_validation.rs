use super::*;
use std::collections::{HashMap, HashSet, VecDeque};

pub(super) fn validate_workflow_sample(
    nodes: &[WorkflowNode],
    edges: &[crate::render_envelope::WorkflowEdge],
) -> Vec<String> {
    let mut errors = Vec::new();
    if nodes.is_empty() {
        errors.push("sample graph has no nodes".into());
    }
    let mut ids = HashSet::new();
    for node in nodes {
        if node.id.trim().is_empty() {
            errors.push("sample node ID is empty".into());
        }
        if !ids.insert(node.id.as_str()) {
            errors.push(format!("duplicate sample node ID: {}", node.id));
        }
        if node.label.trim().is_empty() {
            errors.push(format!("sample node {} has an empty label", node.id));
        }
        if node.kind.trim().is_empty() {
            errors.push(format!("sample node {} has an empty kind", node.id));
        }
    }
    let mut seen = HashSet::new();
    let mut degree: HashMap<&str, usize> = ids.iter().map(|id| (*id, 0)).collect();
    let mut next: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in edges {
        if !ids.contains(edge.from.as_str()) || !ids.contains(edge.to.as_str()) {
            errors.push(format!(
                "sample link {} → {} references a missing node",
                edge.from, edge.to
            ));
            continue;
        }
        if !seen.insert((edge.from.as_str(), edge.to.as_str())) {
            errors.push(format!(
                "duplicate sample link: {} → {}",
                edge.from, edge.to
            ));
        }
        *degree
            .get_mut(edge.to.as_str())
            .expect("checked sample node") += 1;
        next.entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
    }
    let mut ready: VecDeque<&str> = degree
        .iter()
        .filter_map(|(id, count)| (*count == 0).then_some(*id))
        .collect();
    let mut visited = 0;
    while let Some(id) = ready.pop_front() {
        visited += 1;
        for target in next.get(id).into_iter().flatten() {
            let count = degree.get_mut(target).expect("checked sample edge");
            *count -= 1;
            if *count == 0 {
                ready.push_back(target);
            }
        }
    }
    if visited != ids.len() {
        errors.push("sample links contain a cycle".into());
    }
    errors
}

#[cfg(test)]
#[path = "workflow_validation_test.rs"]
mod tests;
