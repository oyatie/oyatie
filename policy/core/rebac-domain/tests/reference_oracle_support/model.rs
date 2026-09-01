use std::collections::BTreeSet;

use policy_cedar_domain::rebac::{
    RebacObjectRef, RebacRelation, RebacSubjectRef, RebacTenantScope, RebacTuple, UsersetRewrite,
};
use policy_rebac_domain::{ExpansionBounds, ExpansionError, NamespaceConfig, ValidatedNamespace};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Object {
    pub object_type: String,
    pub object_id: String,
}

impl Object {
    pub fn new(object_type: &str, object_id: &str) -> Self {
        Self {
            object_type: object_type.to_owned(),
            object_id: object_id.to_owned(),
        }
    }

    pub fn native(&self) -> RebacObjectRef {
        RebacObjectRef::new(self.object_type.clone(), self.object_id.clone())
            .expect("reference-oracle objects use valid tokens")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Subject {
    Object(Object),
    Userset { object: Object, relation: String },
}

impl Subject {
    pub fn object(object_type: &str, object_id: &str) -> Self {
        Self::Object(Object::new(object_type, object_id))
    }

    pub fn userset(object_type: &str, object_id: &str, relation: &str) -> Self {
        Self::Userset {
            object: Object::new(object_type, object_id),
            relation: relation.to_owned(),
        }
    }

    pub fn native(&self) -> RebacSubjectRef {
        match self {
            Self::Object(object) => RebacSubjectRef::object(object.native()),
            Self::Userset { object, relation } => RebacSubjectRef::userset(
                object.native(),
                RebacRelation::new(relation.clone()).expect("oracle relation is valid"),
            ),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tuple {
    pub tenant: String,
    pub object: Object,
    pub relation: String,
    pub subject: Subject,
}

impl Tuple {
    pub fn new(tenant: &str, object: Object, relation: &str, subject: Subject) -> Self {
        Self {
            tenant: tenant.to_owned(),
            object,
            relation: relation.to_owned(),
            subject,
        }
    }

    pub fn native(&self) -> RebacTuple {
        RebacTuple::new(
            RebacTenantScope::new(self.tenant.clone()).expect("oracle tenant is valid"),
            self.object.native(),
            RebacRelation::new(self.relation.clone()).expect("oracle relation is valid"),
            self.subject.native(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Rewrite {
    This,
    Computed(String),
    TupleToUserset { tupleset: String, computed: String },
    Union(Vec<Self>),
    Intersection(Vec<Self>),
    Difference(Box<Self>, Box<Self>),
}

impl Rewrite {
    fn native(&self) -> UsersetRewrite {
        match self {
            Self::This => UsersetRewrite::This,
            Self::Computed(relation) => UsersetRewrite::ComputedUserset {
                relation: RebacRelation::new(relation.clone()).expect("oracle relation is valid"),
            },
            Self::TupleToUserset { tupleset, computed } => UsersetRewrite::TupleToUserset {
                tupleset_relation: RebacRelation::new(tupleset.clone())
                    .expect("oracle relation is valid"),
                computed_userset_relation: RebacRelation::new(computed.clone())
                    .expect("oracle relation is valid"),
            },
            Self::Union(children) => UsersetRewrite::Union {
                children: children.iter().map(Self::native).collect(),
            },
            Self::Intersection(children) => UsersetRewrite::Intersection {
                children: children.iter().map(Self::native).collect(),
            },
            Self::Difference(base, subtract) => UsersetRewrite::Difference {
                base: Box::new(base.native()),
                subtract: Box::new(subtract.native()),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Definition {
    pub object_type: String,
    pub relation: String,
    pub rewrite: Rewrite,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Model {
    pub definitions: Vec<Definition>,
}

impl Model {
    pub fn define(mut self, object_type: &str, relation: &str, rewrite: Rewrite) -> Self {
        self.definitions.push(Definition {
            object_type: object_type.to_owned(),
            relation: relation.to_owned(),
            rewrite,
        });
        self
    }

    pub fn rewrite(&self, object_type: &str, relation: &str) -> Option<&Rewrite> {
        self.definitions
            .iter()
            .rev()
            .find(|definition| {
                definition.object_type == object_type && definition.relation == relation
            })
            .map(|definition| &definition.rewrite)
    }

    pub fn effective_definitions(&self) -> Vec<&Definition> {
        let mut seen = BTreeSet::new();
        let mut definitions = Vec::new();
        for definition in self.definitions.iter().rev() {
            let key = (&definition.object_type, &definition.relation);
            if seen.insert(key) {
                definitions.push(definition);
            }
        }
        definitions.reverse();
        definitions
    }

    pub fn native(&self) -> Result<ValidatedNamespace, ExpansionError> {
        let mut namespace = NamespaceConfig::new();
        for definition in &self.definitions {
            let relation =
                RebacRelation::new(definition.relation.clone()).expect("oracle relation is valid");
            namespace = namespace.define(
                definition.object_type.clone(),
                &relation,
                definition.rewrite.native(),
            );
        }
        namespace.validated()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Query {
    pub tenant: String,
    pub subject: Subject,
    pub object: Object,
    pub relation: String,
}

impl Query {
    pub fn new(tenant: &str, subject: Subject, object: Object, relation: &str) -> Self {
        Self {
            tenant: tenant.to_owned(),
            subject,
            object,
            relation: relation.to_owned(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Bounds {
    pub candidates: usize,
    pub depth: u32,
    pub tuples: usize,
    pub pages: usize,
}

impl Bounds {
    pub const GENEROUS: Self = Self {
        candidates: 32,
        depth: 16,
        tuples: 128,
        pages: 32,
    };

    pub fn native(self) -> ExpansionBounds {
        ExpansionBounds {
            max_candidates: self.candidates,
            max_depth: self.depth,
            max_tuples_read: self.tuples,
            max_pages_per_tupleset: self.pages,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Refusal {
    UnknownRelation {
        object_type: String,
        relation: String,
    },
    NonStratified {
        object_type: String,
        relation: String,
    },
    NegatedCycleInData {
        object_type: String,
        relation: String,
    },
    CandidateBudgetExceeded(usize),
    DepthExceeded(u32),
    TupleBudgetExceeded(usize),
    PageBudgetExceeded(usize),
    TenantScope,
    Cancelled,
    Store(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    Allow,
    Deny,
    Refuse(Refusal),
}
