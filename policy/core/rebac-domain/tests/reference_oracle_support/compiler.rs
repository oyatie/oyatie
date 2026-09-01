use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::model::{Definition, Model, Refusal, Rewrite};

type Node = (String, String);

#[derive(Clone)]
pub struct CompiledModel<'a> {
    model: &'a Model,
}

impl<'a> CompiledModel<'a> {
    pub fn rewrite(&self, object_type: &str, relation: &str) -> Option<&'a Rewrite> {
        self.model.rewrite(object_type, relation)
    }
}

pub fn compile(model: &Model) -> Result<CompiledModel<'_>, Refusal> {
    validate_stratification(&dependency_graph(&model.effective_definitions()))?;
    Ok(CompiledModel { model })
}

fn dependency_graph(definitions: &[&Definition]) -> BTreeMap<Node, Vec<(Node, bool)>> {
    let object_types: BTreeSet<&str> = definitions
        .iter()
        .map(|definition| definition.object_type.as_str())
        .collect();
    let mut graph = BTreeMap::new();
    for definition in definitions {
        let from = (definition.object_type.clone(), definition.relation.clone());
        let mut edges = Vec::new();
        collect_edges(
            &definition.object_type,
            &definition.rewrite,
            false,
            &object_types,
            &mut edges,
        );
        graph.insert(from, edges);
    }
    graph
}

fn validate_stratification(graph: &BTreeMap<Node, Vec<(Node, bool)>>) -> Result<(), Refusal> {
    for (from, edges) in graph {
        for (to, negative) in edges {
            if *negative && reaches(to, from, graph) {
                return Err(Refusal::NonStratified {
                    object_type: to.0.clone(),
                    relation: to.1.clone(),
                });
            }
        }
    }
    Ok(())
}

fn collect_edges(
    object_type: &str,
    rewrite: &Rewrite,
    negative: bool,
    object_types: &BTreeSet<&str>,
    edges: &mut Vec<(Node, bool)>,
) {
    let mut pending = vec![(rewrite, negative)];
    while let Some((rewrite, negative)) = pending.pop() {
        match rewrite {
            Rewrite::This => {}
            Rewrite::Computed(relation) => {
                edges.push(((object_type.to_owned(), relation.clone()), negative));
            }
            Rewrite::TupleToUserset { computed, .. } => {
                edges.extend(
                    object_types
                        .iter()
                        .map(|kind| (((*kind).to_owned(), computed.clone()), negative)),
                );
            }
            Rewrite::Union(children) | Rewrite::Intersection(children) => {
                pending.extend(children.iter().map(|child| (child, negative)));
            }
            Rewrite::Difference(base, subtract) => {
                pending.push((base, negative));
                pending.push((subtract, true));
            }
        }
    }
}

fn reaches(start: &Node, target: &Node, graph: &BTreeMap<Node, Vec<(Node, bool)>>) -> bool {
    let mut queue = VecDeque::from([start.clone()]);
    let mut seen = BTreeSet::new();
    while let Some(node) = queue.pop_front() {
        if &node == target {
            return true;
        }
        if !seen.insert(node.clone()) {
            continue;
        }
        if let Some(edges) = graph.get(&node) {
            queue.extend(edges.iter().map(|(next, _)| next.clone()));
        }
    }
    false
}
