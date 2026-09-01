use std::collections::BTreeMap;

use super::compiler::{CompiledModel, compile};
use super::expression::{Formula, Node};
use super::model::{Model, Outcome, Query, Refusal, Rewrite, Subject, Tuple};

pub struct FiniteEvaluator<'a> {
    model: CompiledModel<'a>,
    tuples: &'a [Tuple],
}

impl<'a> FiniteEvaluator<'a> {
    pub fn new(model: &'a Model, tuples: &'a [Tuple]) -> Result<Self, Refusal> {
        Ok(Self {
            model: compile(model)?,
            tuples,
        })
    }

    pub fn evaluate(&self, query: &Query) -> Outcome {
        let root = Node::new(query.object.clone(), query.relation.clone());
        let formula = self.unfold_node(query, &root, &mut BTreeMap::new(), None);
        match formula.evaluate() {
            Ok(true) => Outcome::Allow,
            Ok(false) => Outcome::Deny,
            Err(refusal) => Outcome::Refuse(refusal),
        }
    }

    fn unfold_node(
        &self,
        query: &Query,
        node: &Node,
        active: &mut BTreeMap<Node, usize>,
        subtraction_boundary: Option<usize>,
    ) -> Formula {
        if let Some(position) = active.get(node) {
            return if subtraction_boundary.is_some_and(|boundary| *position < boundary) {
                Formula::Refuse(Refusal::NegatedCycleInData {
                    object_type: node.object.object_type.clone(),
                    relation: node.relation.clone(),
                })
            } else {
                Formula::Literal(false)
            };
        }
        let Some(rewrite) = self.model.rewrite(&node.object.object_type, &node.relation) else {
            return Formula::Refuse(Refusal::UnknownRelation {
                object_type: node.object.object_type.clone(),
                relation: node.relation.clone(),
            });
        };

        let position = active.len();
        active.insert(node.clone(), position);
        let formula = self.unfold_rewrite(query, node, rewrite, active, subtraction_boundary);
        let removed = active.remove(node);
        debug_assert_eq!(removed, Some(position));
        formula
    }

    fn unfold_rewrite(
        &self,
        query: &Query,
        node: &Node,
        rewrite: &Rewrite,
        active: &mut BTreeMap<Node, usize>,
        subtraction_boundary: Option<usize>,
    ) -> Formula {
        match rewrite {
            Rewrite::This => self.unfold_direct(query, node, active, subtraction_boundary),
            Rewrite::Computed(relation) => self.unfold_node(
                query,
                &Node::new(node.object.clone(), relation.clone()),
                active,
                subtraction_boundary,
            ),
            Rewrite::TupleToUserset { tupleset, computed } => self.unfold_tupleset(
                query,
                node,
                tupleset,
                computed,
                active,
                subtraction_boundary,
            ),
            Rewrite::Union(children) => Formula::Union(
                children
                    .iter()
                    .map(|child| {
                        self.unfold_rewrite(query, node, child, active, subtraction_boundary)
                    })
                    .collect(),
            ),
            Rewrite::Intersection(children) => Formula::Intersection(
                children
                    .iter()
                    .map(|child| {
                        self.unfold_rewrite(query, node, child, active, subtraction_boundary)
                    })
                    .collect(),
            ),
            Rewrite::Difference(base, subtract) => Formula::Difference(
                Box::new(self.unfold_rewrite(query, node, base, active, subtraction_boundary)),
                Box::new(self.unfold_rewrite(query, node, subtract, active, Some(active.len()))),
            ),
        }
    }

    fn unfold_direct(
        &self,
        query: &Query,
        node: &Node,
        active: &mut BTreeMap<Node, usize>,
        subtraction_boundary: Option<usize>,
    ) -> Formula {
        let mut alternatives = Vec::new();
        for tuple in self.matching_tuples(query, node, &node.relation) {
            if tuple.subject == query.subject {
                alternatives.push(Formula::Literal(true));
            } else if let Subject::Userset { object, relation } = &tuple.subject {
                alternatives.push(self.unfold_node(
                    query,
                    &Node::new(object.clone(), relation.clone()),
                    active,
                    subtraction_boundary,
                ));
            }
        }
        Formula::Union(alternatives)
    }

    fn unfold_tupleset(
        &self,
        query: &Query,
        node: &Node,
        tupleset: &str,
        computed: &str,
        active: &mut BTreeMap<Node, usize>,
        subtraction_boundary: Option<usize>,
    ) -> Formula {
        let mut alternatives = Vec::new();
        for tuple in self.matching_tuples(query, node, tupleset) {
            if let Subject::Object(object) = &tuple.subject {
                alternatives.push(self.unfold_node(
                    query,
                    &Node::new(object.clone(), computed),
                    active,
                    subtraction_boundary,
                ));
            }
        }
        Formula::Union(alternatives)
    }

    fn matching_tuples<'tuple>(
        &'tuple self,
        query: &'tuple Query,
        node: &'tuple Node,
        relation: &'tuple str,
    ) -> impl Iterator<Item = &'tuple Tuple> {
        self.tuples.iter().filter(move |tuple| {
            tuple.tenant == query.tenant
                && tuple.object == node.object
                && tuple.relation == relation
        })
    }
}
