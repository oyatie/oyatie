use std::collections::{BTreeMap, BTreeSet};

pub mod families;
pub mod store;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Object {
    pub object_type: &'static str,
    pub object_id: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Subject {
    Concrete(Object),
    Userset {
        object: Object,
        relation: &'static str,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tuple {
    pub object: Object,
    pub relation: &'static str,
    pub subject: Subject,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr {
    This,
    Computed(&'static str),
    TupleToUserset {
        tupleset: &'static str,
        computed: &'static str,
    },
    Union(Vec<Expr>),
    Intersection(Vec<Expr>),
    Difference(Box<Expr>, Box<Expr>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Relation {
    pub object_type: &'static str,
    pub name: &'static str,
    pub stratum: usize,
    pub expr: Expr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Model {
    pub objects: Vec<Object>,
    pub users: Vec<Object>,
    pub relations: Vec<Relation>,
    pub tuples: Vec<Tuple>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Member {
    pub subject: Object,
    pub relation: &'static str,
    pub object: Object,
}

pub type Membership = BTreeSet<Member>;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SetKey {
    object: Object,
    relation: &'static str,
}

type Sets = BTreeMap<SetKey, BTreeSet<Object>>;

pub fn reference(model: &Model) -> Membership {
    let mut sets = Sets::new();
    for relation in &model.relations {
        for object in objects_for(model, relation.object_type) {
            sets.insert(
                SetKey {
                    object: object.clone(),
                    relation: relation.name,
                },
                BTreeSet::new(),
            );
        }
    }
    let membership_limit = sets.len().saturating_mul(model.users.len());
    let strata: BTreeSet<usize> = model
        .relations
        .iter()
        .map(|relation| relation.stratum)
        .collect();
    for stratum in strata {
        let mut stabilized = false;
        for _ in 0..=membership_limit {
            let mut next = sets.clone();
            for relation in model
                .relations
                .iter()
                .filter(|relation| relation.stratum == stratum)
            {
                for object in objects_for(model, relation.object_type) {
                    let key = SetKey {
                        object: object.clone(),
                        relation: relation.name,
                    };
                    let derived = evaluate(&relation.expr, relation, object, model, &sets);
                    assert!(
                        sets[&key].is_subset(&derived),
                        "{}#{} stratum {} was not monotone",
                        object.object_id,
                        relation.name,
                        stratum
                    );
                    next.insert(key, derived);
                }
            }
            if next == sets {
                stabilized = true;
                break;
            }
            sets = next;
        }
        assert!(
            stabilized,
            "stratum {stratum} exceeded the finite membership bound {membership_limit}"
        );
    }
    sets.into_iter()
        .flat_map(|(key, subjects)| {
            subjects.into_iter().map(move |subject| Member {
                subject,
                relation: key.relation,
                object: key.object.clone(),
            })
        })
        .collect()
}

fn objects_for<'a>(model: &'a Model, object_type: &str) -> impl Iterator<Item = &'a Object> {
    model
        .objects
        .iter()
        .filter(move |object| object.object_type == object_type)
}

fn lookup(sets: &Sets, object: &Object, relation: &'static str) -> BTreeSet<Object> {
    sets.get(&SetKey {
        object: object.clone(),
        relation,
    })
    .cloned()
    .unwrap_or_default()
}

fn evaluate(
    expr: &Expr,
    relation: &Relation,
    object: &Object,
    model: &Model,
    sets: &Sets,
) -> BTreeSet<Object> {
    match expr {
        Expr::This => direct(relation.name, object, model, sets),
        Expr::Computed(computed) => lookup(sets, object, computed),
        Expr::TupleToUserset { tupleset, computed } => model
            .tuples
            .iter()
            .filter(|tuple| &tuple.object == object && tuple.relation == *tupleset)
            .filter_map(|tuple| match &tuple.subject {
                Subject::Concrete(target) => Some(lookup(sets, target, computed)),
                Subject::Userset { .. } => None,
            })
            .flatten()
            .collect(),
        Expr::Union(children) => children
            .iter()
            .flat_map(|child| evaluate(child, relation, object, model, sets))
            .collect(),
        Expr::Intersection(children) => {
            let mut children = children.iter();
            let Some(first) = children.next() else {
                return BTreeSet::new();
            };
            children.fold(
                evaluate(first, relation, object, model, sets),
                |held, child| {
                    held.intersection(&evaluate(child, relation, object, model, sets))
                        .cloned()
                        .collect()
                },
            )
        }
        Expr::Difference(base, subtract) => {
            assert_lower(base, relation, model);
            assert_lower(subtract, relation, model);
            evaluate(base, relation, object, model, sets)
                .difference(&evaluate(subtract, relation, object, model, sets))
                .cloned()
                .collect()
        }
    }
}

fn direct(relation: &'static str, object: &Object, model: &Model, sets: &Sets) -> BTreeSet<Object> {
    model
        .tuples
        .iter()
        .filter(|tuple| &tuple.object == object && tuple.relation == relation)
        .flat_map(|tuple| match &tuple.subject {
            Subject::Concrete(subject) if model.users.contains(subject) => {
                BTreeSet::from([subject.clone()])
            }
            Subject::Concrete(_) => BTreeSet::new(),
            Subject::Userset { object, relation } => lookup(sets, object, relation),
        })
        .collect()
}

fn assert_lower(expr: &Expr, relation: &Relation, model: &Model) {
    match expr {
        Expr::This => panic!("difference may only read finalized lower-stratum relations"),
        Expr::Computed(name) => {
            let dependency = model
                .relations
                .iter()
                .find(|candidate| {
                    candidate.object_type == relation.object_type && candidate.name == *name
                })
                .expect("computed relation is declared");
            assert!(dependency.stratum < relation.stratum);
        }
        Expr::TupleToUserset { computed, .. } => {
            assert!(
                model
                    .relations
                    .iter()
                    .filter(|item| item.name == *computed)
                    .all(|dependency| dependency.stratum < relation.stratum)
            );
        }
        Expr::Union(children) | Expr::Intersection(children) => {
            for child in children {
                assert_lower(child, relation, model);
            }
        }
        Expr::Difference(base, subtract) => {
            assert_lower(base, relation, model);
            assert_lower(subtract, relation, model);
        }
    }
}

pub fn member_ids<'a>(
    membership: &'a Membership,
    object_type: &str,
    object_id: &str,
    relation: &str,
) -> BTreeSet<&'a str> {
    membership
        .iter()
        .filter(|member| {
            member.object.object_type == object_type
                && member.object.object_id == object_id
                && member.relation == relation
        })
        .map(|member| member.subject.object_id)
        .collect()
}
