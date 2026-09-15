//! The wire form of `POST /v1/object-sets/page` and its mapping to the
//! domain's [`ObjectSet`].
//!
//! The body is taken as a string and parsed inside the handler, after the
//! credential, for the reason `RevisionPin` states: a typed extractor refuses a
//! malformed body before anyone is asked to authenticate, which would answer a
//! stranger with a description of the API. One condition: taking it as a string
//! still requires it to be UTF-8, so a body that is not gets the runtime's own
//! refusal before this handler runs — as on every other body route here.
//!
//! A set is a tree, which no query string can carry, so this route reads a
//! body where the listing reads a query. The filter clause of a `matching`
//! leaf is the listing's own grammar, parsed by the listing's own function,
//! so one predicate spelling serves both routes.

use foundry_projection_draft::{PageRequest, ProjectionCursor};
use foundry_spine::{ObjectSet, SetDefinition};
use serde::Deserialize;

use crate::listing_query::{DEFAULT_PAGE_LIMIT, MAX_PAGE_LIMIT, QueryRefusal, filter_of};

/// One node of a set definition. A node names its own kind, so an operand is
/// read the same way at any depth.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SetNode {
    Every,
    Named(Vec<String>), // data_class: INTERNAL_ONLY
    Matching(MatchingBody),
    Union(Box<SetNode>, Box<SetNode>),
    Intersect(Box<SetNode>, Box<SetNode>),
    Subtract(Box<SetNode>, Box<SetNode>),
}

/// A filter leaf, in the four names `GET /v1/objects` gives the same clause.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MatchingBody {
    property: String,       // data_class: INTERNAL_ONLY
    equals: Option<String>, // data_class: PII_IDENTIFYING
    from: Option<String>,   // data_class: PII_IDENTIFYING
    to: Option<String>,     // data_class: PII_IDENTIFYING
}

/// The page bounds arrive as JSON values rather than as typed fields, so a
/// value of the wrong shape is refused by the rule it breaks — the same cause
/// the listing gives — instead of by the body's shape sentence.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SetPageBody {
    #[serde(rename = "type")]
    entity_type: String, // data_class: INTERNAL_ONLY
    revision: serde_json::Value, // data_class: INTERNAL_ONLY
    limit: Option<serde_json::Value>,
    cursor: Option<serde_json::Value>, // data_class: INTERNAL_ONLY
    set: SetNode,
}

/// Why a set page cannot be served. Every reason a filter clause can fail is
/// the listing's, reported with the listing's own cause.
pub(crate) enum SetRefusal {
    /// The body is not the shape this route defines.
    UnusableBody,
    Query(QueryRefusal),
}

impl SetRefusal {
    pub(crate) fn cause(&self) -> &'static str {
        match self {
            Self::UnusableBody => {
                "a set page is {\"type\",\"revision\",\"set\"} with optional limit and cursor, \
                 and a set is \"every\", {\"named\":[..]}, {\"matching\":{..}}, \
                 or {\"union\"|\"intersect\"|\"subtract\":[set,set]}"
            }
            Self::Query(refusal) => refusal.cause(),
        }
    }
}

/// What one request asks for: the set, the pin it is read at, and the page.
pub(crate) struct SetPageRequest {
    pub(crate) entity_type: String,
    pub(crate) revision: u32,
    pub(crate) definition: SetDefinition,
    pub(crate) page: PageRequest,
}

impl SetPageRequest {
    pub(crate) fn parse(body: &str) -> Result<Self, SetRefusal> {
        let body: SetPageBody = serde_json::from_str(body).map_err(|_| SetRefusal::UnusableBody)?;
        let revision = whole_number(&body.revision)
            .and_then(|revision| u32::try_from(revision).ok())
            .ok_or(SetRefusal::Query(QueryRefusal::UnusableRevision))?;
        let limit = match &body.limit {
            None => DEFAULT_PAGE_LIMIT,
            Some(value) => whole_number(value)
                .and_then(|limit| usize::try_from(limit).ok())
                .filter(|limit| (1..=MAX_PAGE_LIMIT).contains(limit))
                .ok_or(SetRefusal::Query(QueryRefusal::UnusableLimit))?,
        };
        let page = match &body.cursor {
            None => PageRequest::first(limit),
            Some(value) => {
                let after_object_ref = value
                    .as_str()
                    .filter(|cursor| cursor.starts_with("ent_"))
                    .ok_or(SetRefusal::Query(QueryRefusal::UnusableCursor))?;
                PageRequest::after(
                    limit,
                    ProjectionCursor {
                        after_object_ref: after_object_ref.to_owned(),
                    },
                )
            }
        };
        Ok(Self {
            entity_type: body.entity_type,
            revision,
            definition: definition_of(body.set)?,
            page,
        })
    }

    /// The set this request names, over `entity_type` as the tenant declares
    /// it. The type is resolved by the handler, so a set is never built over
    /// a type the caller's tenant does not have.
    pub(crate) fn set(self, entity_type: data_ontology_kernel::EntityTypeId) -> ObjectSet {
        ObjectSet {
            entity_type,
            definition: self.definition,
        }
    }
}

/// The whole number `value` holds, if it holds one. A JSON number with a
/// fraction or a sign the bound cannot carry is not a whole number.
fn whole_number(value: &serde_json::Value) -> Option<u64> {
    value.as_u64()
}

fn definition_of(node: SetNode) -> Result<SetDefinition, SetRefusal> {
    Ok(match node {
        SetNode::Every => SetDefinition::Every,
        SetNode::Named(object_refs) => SetDefinition::Named(object_refs),
        SetNode::Matching(clause) => {
            let filter = filter_of(Some(clause.property), clause.equals, clause.from, clause.to)
                .map_err(SetRefusal::Query)?
                .ok_or(SetRefusal::Query(QueryRefusal::IncompleteFilter))?;
            SetDefinition::Matching(filter)
        }
        SetNode::Union(left, right) => SetDefinition::Union(operand(*left)?, operand(*right)?),
        SetNode::Intersect(left, right) => {
            SetDefinition::Intersect(operand(*left)?, operand(*right)?)
        }
        SetNode::Subtract(left, right) => {
            SetDefinition::Subtract(operand(*left)?, operand(*right)?)
        }
    })
}

fn operand(node: SetNode) -> Result<Box<SetDefinition>, SetRefusal> {
    definition_of(node).map(Box::new)
}
