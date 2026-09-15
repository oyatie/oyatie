//! The query grammar of `GET /v1/objects`.
//!
//! Parsed from the RAW query string inside the handler, after the
//! credential, for the reason `RevisionPin` documents: a typed extractor
//! refuses a malformed value before anyone is asked to authenticate, which
//! would answer a stranger with a description of the API.

use data_ontology_kernel::PropertyValue;
use foundry_projection_draft::{PageRequest, PredicateError, ProjectionCursor, PropertyPredicate};

/// The page size a caller that names none is served.
const DEFAULT_PAGE_LIMIT: usize = 100;
/// The largest page this surface will serve, so one request cannot ask the
/// store for a tenant's whole type.
const MAX_PAGE_LIMIT: usize = 1000;

/// `?type=&revision=&limit=&cursor=&property=&equals=&from=&to=`.
///
/// A key this route does not define is refused rather than ignored, so a
/// parameter the surface drops can never be mistaken for one it honoured.
pub(crate) struct ListingQuery {
    pub(crate) entity_type: String,
    pub(crate) revision: u32,
    pub(crate) limit: usize,
    pub(crate) cursor: Option<ProjectionCursor>,
    pub(crate) filter: Option<PropertyPredicate>,
}

/// A filter literal is `<kind>:<value>`, so the kind a caller compares
/// under is stated rather than guessed from the spelling: `int:7` and
/// `text:7` are different questions and a bare `7` does not say which.
/// `text`, `int` and `bool` only — the kinds a literal can carry
/// unambiguously without a second parser for formats or precision.
fn typed_literal(raw: &str) -> Option<PropertyValue> {
    let (kind, value) = raw.split_once(':')?;
    match kind {
        "text" => Some(PropertyValue::String(value.to_owned())),
        "int" => value.parse().ok().map(PropertyValue::Integer),
        "bool" => match value {
            "true" => Some(PropertyValue::Boolean(true)),
            "false" => Some(PropertyValue::Boolean(false)),
            _ => None,
        },
        _ => None,
    }
}

/// Why a query string cannot be served. Each names a RULE the request broke, so
/// two requests that break only one rule answer alike however differently they
/// broke it; a request that breaks two is answered by whichever is reached
/// first, here or inside the predicate the parser builds.
pub(crate) enum QueryRefusal {
    UnknownParameter,
    /// The property is not a name the projection could hold. Through a query
    /// string only the empty case arrives: values are not decoded, so an
    /// escaped space is a literal name and a raw one is not a legal request
    /// target. The space case is the port's rule, for a caller that can spell
    /// one.
    UnusableProperty,
    RepeatedParameter,
    MissingType,
    UnusableRevision,
    UnusableLimit,
    UnusableCursor,
    /// A filter clause without the property it constrains, a property with
    /// no clause, or one end of a range: no half of a filter is a question
    /// on its own, and one bound is not a range.
    IncompleteFilter,
    /// `equals` together with a range.
    ConflictingFilter,
    UnusableLiteral,
    /// `from` sorts after `to`, or the two name different kinds.
    UnusableRange,
}

impl QueryRefusal {
    pub(crate) fn cause(&self) -> &'static str {
        match self {
            Self::UnusableProperty => {
                "?property= must name the property to filter on, with no surrounding space"
            }
            Self::UnknownParameter => {
                "this listing defines type, revision, limit, cursor, property, equals, from and to only"
            }
            Self::RepeatedParameter => "a parameter given twice has no single honest answer",
            Self::MissingType => "a listing must name the entity type it lists: ?type=ety_...",
            Self::UnusableRevision => "a read must pin the revision it understands: ?revision=N",
            Self::UnusableLimit => "?limit= must be a whole number from 1 to 1000",
            Self::UnusableCursor => "?cursor= must be the object reference a page ended on",
            Self::IncompleteFilter => {
                "a filter is ?property= with either ?equals= or both ?from= and ?to="
            }
            Self::ConflictingFilter => "a filter is either an equality or a range, not both",
            Self::UnusableLiteral => "a filter literal is text:, int: or bool: and its value",
            Self::UnusableRange => {
                "?from= and ?to= must be the same kind, and from may not sort after to"
            }
        }
    }
}

impl ListingQuery {
    /// Canonical form only: keys and values are matched literally and are
    /// NOT percent-decoded, the narrowing `RevisionPin::parse` documents.
    pub(crate) fn parse(raw: Option<&str>) -> Result<Self, QueryRefusal> {
        let (mut entity_type, mut revision, mut limit, mut cursor) = (None, None, None, None);
        let (mut property, mut equals, mut from, mut to) = (None, None, None, None);
        for pair in raw.unwrap_or_default().split('&').filter(|p| !p.is_empty()) {
            let Some((key, value)) = pair.split_once('=') else {
                return Err(QueryRefusal::UnknownParameter);
            };
            let slot = match key {
                "type" => &mut entity_type,
                "revision" => &mut revision,
                "limit" => &mut limit,
                "cursor" => &mut cursor,
                "property" => &mut property,
                "equals" => &mut equals,
                "from" => &mut from,
                "to" => &mut to,
                _ => return Err(QueryRefusal::UnknownParameter),
            };
            if slot.replace(value.to_owned()).is_some() {
                return Err(QueryRefusal::RepeatedParameter);
            }
        }
        let entity_type = entity_type.ok_or(QueryRefusal::MissingType)?;
        let revision = revision
            .ok_or(QueryRefusal::UnusableRevision)?
            .parse::<u32>()
            .map_err(|_| QueryRefusal::UnusableRevision)?;
        let limit = match limit {
            None => DEFAULT_PAGE_LIMIT,
            Some(text) => match text.parse::<usize>() {
                Ok(limit) if (1..=MAX_PAGE_LIMIT).contains(&limit) => limit,
                _ => return Err(QueryRefusal::UnusableLimit),
            },
        };
        let cursor = match cursor {
            None => None,
            Some(after_object_ref) if after_object_ref.starts_with("ent_") => {
                Some(ProjectionCursor { after_object_ref })
            }
            Some(_) => return Err(QueryRefusal::UnusableCursor),
        };
        let filter = filter_of(property, equals, from, to)?;
        Ok(Self {
            entity_type,
            revision,
            limit,
            cursor,
            filter,
        })
    }
}

/// The predicate the four filter parameters spell, or `None` when none of
/// them is present. Each incoherent combination is refused by the rule it
/// breaks.
fn filter_of(
    property: Option<String>,
    equals: Option<String>,
    from: Option<String>,
    to: Option<String>,
) -> Result<Option<PropertyPredicate>, QueryRefusal> {
    let clause = equals.is_some() || from.is_some() || to.is_some();
    let Some(property) = property else {
        return if clause {
            Err(QueryRefusal::IncompleteFilter)
        } else {
            Ok(None)
        };
    };
    match (equals, from, to) {
        (None, None, None) => Err(QueryRefusal::IncompleteFilter),
        (Some(_), Some(_), _) | (Some(_), _, Some(_)) => Err(QueryRefusal::ConflictingFilter),
        (Some(value), None, None) => {
            let value = typed_literal(&value).ok_or(QueryRefusal::UnusableLiteral)?;
            PropertyPredicate::equals(property, value)
                .map(Some)
                .map_err(refusal_of)
        }
        // One bound is not a range: the same rule an absent clause breaks.
        (None, Some(_), None) | (None, None, Some(_)) => Err(QueryRefusal::IncompleteFilter),
        (None, Some(from), Some(to)) => {
            let from = typed_literal(&from).ok_or(QueryRefusal::UnusableLiteral)?;
            let to = typed_literal(&to).ok_or(QueryRefusal::UnusableLiteral)?;
            PropertyPredicate::range(property, from, to)
                .map(Some)
                .map_err(refusal_of)
        }
    }
}

/// The refusal a predicate the port rejected is reported as. Every variant
/// the port can return is named, so no refusal reports a measurement that
/// was not made.
fn refusal_of(error: PredicateError) -> QueryRefusal {
    match error {
        PredicateError::BlankProperty => QueryRefusal::UnusableProperty,
        PredicateError::MixedValueKinds | PredicateError::InvertedRange => {
            QueryRefusal::UnusableRange
        }
        // Unreachable from this grammar: `typed_literal` yields text,
        // integers and booleans, and all three are ranked. Named rather than
        // discarded so a new port variant forces a decision here.
        PredicateError::UnrankedValueKind => QueryRefusal::UnusableLiteral,
    }
}

impl ListingQuery {
    pub(crate) fn page(&self) -> PageRequest {
        match &self.cursor {
            None => PageRequest::first(self.limit),
            Some(cursor) => PageRequest::after(self.limit, cursor.clone()),
        }
    }
}
