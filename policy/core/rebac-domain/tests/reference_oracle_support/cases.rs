use std::collections::BTreeSet;

use super::model::{Model, Object, Query, Rewrite, Subject, Tuple};

pub const TENANT: &str = "tenant_a";

pub struct GeneratedCase {
    pub name: String,
    pub model: Model,
    pub operators: BTreeSet<&'static str>,
}

#[derive(Clone)]
struct Atom {
    name: &'static str,
    rewrite: Rewrite,
    operators: BTreeSet<&'static str>,
}

pub fn generated_cases() -> Vec<GeneratedCase> {
    let atoms = atoms();
    let mut cases: Vec<GeneratedCase> = atoms
        .iter()
        .map(|atom| {
            make_case(
                atom.name.to_owned(),
                atom.rewrite.clone(),
                atom.operators.clone(),
            )
        })
        .collect();
    for (operator, label) in [
        ("union", "union"),
        ("intersection", "intersection"),
        ("difference", "difference"),
    ] {
        for left in &atoms {
            for right in &atoms {
                let rewrite = composite(operator, left.rewrite.clone(), right.rewrite.clone());
                let mut operators = left.operators.clone();
                operators.extend(&right.operators);
                operators.insert(label);
                cases.push(make_case(
                    format!("{label}_{}_{}", left.name, right.name),
                    rewrite,
                    operators,
                ));
            }
        }
    }
    cases
}

pub fn tuple_universe() -> Vec<Tuple> {
    vec![
        tuple(
            TENANT,
            "doc",
            "one",
            "target",
            Subject::object("user", "alice"),
        ),
        tuple(
            TENANT,
            "doc",
            "one",
            "left",
            Subject::object("user", "alice"),
        ),
        tuple(
            TENANT,
            "doc",
            "one",
            "right",
            Subject::object("user", "bob"),
        ),
        tuple(
            TENANT,
            "doc",
            "one",
            "parent",
            Subject::object("folder", "one"),
        ),
        tuple(
            TENANT,
            "folder",
            "one",
            "member",
            Subject::object("user", "alice"),
        ),
    ]
}

pub fn queries() -> Vec<Query> {
    ["alice", "bob"]
        .into_iter()
        .map(|subject| query(subject, "doc", "one", "target"))
        .collect()
}

pub fn direct_model(object_type: &str, relation: &str) -> Model {
    Model::default().define(object_type, relation, Rewrite::This)
}

pub fn query(subject_id: &str, object_type: &str, object_id: &str, relation: &str) -> Query {
    Query::new(
        TENANT,
        Subject::object("user", subject_id),
        Object::new(object_type, object_id),
        relation,
    )
}

pub fn tuple(
    tenant: &str,
    object_type: &str,
    object_id: &str,
    relation: &str,
    subject: Subject,
) -> Tuple {
    Tuple::new(
        tenant,
        Object::new(object_type, object_id),
        relation,
        subject,
    )
}

fn atoms() -> Vec<Atom> {
    vec![
        Atom {
            name: "this",
            rewrite: Rewrite::This,
            operators: BTreeSet::from(["this"]),
        },
        Atom {
            name: "computed_left",
            rewrite: Rewrite::Computed("left".to_owned()),
            operators: BTreeSet::from(["computed_userset"]),
        },
        Atom {
            name: "computed_right",
            rewrite: Rewrite::Computed("right".to_owned()),
            operators: BTreeSet::from(["computed_userset"]),
        },
        Atom {
            name: "tuple_parent_member",
            rewrite: Rewrite::TupleToUserset {
                tupleset: "parent".to_owned(),
                computed: "member".to_owned(),
            },
            operators: BTreeSet::from(["tuple_to_userset"]),
        },
    ]
}

fn make_case(name: String, rewrite: Rewrite, operators: BTreeSet<&'static str>) -> GeneratedCase {
    let model = direct_model("doc", "left")
        .define("doc", "right", Rewrite::This)
        .define("folder", "member", Rewrite::This)
        .define("doc", "target", rewrite);
    GeneratedCase {
        name,
        model,
        operators,
    }
}

fn composite(operator: &str, left: Rewrite, right: Rewrite) -> Rewrite {
    match operator {
        "union" => Rewrite::Union(vec![left, right]),
        "intersection" => Rewrite::Intersection(vec![left, right]),
        "difference" => Rewrite::Difference(Box::new(left), Box::new(right)),
        _ => unreachable!("the generator has a closed operator alphabet"),
    }
}
