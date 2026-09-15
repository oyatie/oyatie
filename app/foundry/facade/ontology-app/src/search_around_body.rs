//! The wire form of `POST /v1/search-around`.
//!
//! The body is taken as a string and parsed inside the handler, after the
//! credential, the ordering `RevisionPin` states and the set route follows.
//!
//! The seed is an object set, spelled exactly as `POST /v1/object-sets/page`
//! spells one, because a set is what the domain's `with_additional_roots`
//! documents as its seam: the walk starts from every member the seed holds.

use foundry_ontology_query_domain::{
    EdgeConsent, KnowledgeGraphQueryError, KnowledgeGraphQueryRequest, TraversalDirection,
};
use foundry_spine::SetDefinition;
use serde::Deserialize;

use crate::object_set_body::{SetRefusal, seed_definition};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SeedBody {
    #[serde(rename = "type")]
    entity_type: String, // data_class: INTERNAL_ONLY
    revision: serde_json::Value, // data_class: INTERNAL_ONLY
    set: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SearchAroundBody {
    idempotency_key: String, // data_class: INTERNAL_ONLY
    seed: SeedBody,
    edge_types: Vec<String>,      // data_class: INTERNAL_ONLY
    max_depth: serde_json::Value, // data_class: INTERNAL_ONLY
    direction: String,            // data_class: INTERNAL_ONLY
    consent: serde_json::Value,
    freshness_floor_epoch_seconds: serde_json::Value, // data_class: INTERNAL_ONLY
    observed_at_epoch_seconds: serde_json::Value,     // data_class: INTERNAL_ONLY
}

/// Why a search-around cannot be served, before the domain sees it.
pub(crate) enum SearchRefusal {
    /// The body is not the shape this route defines.
    UnusableBody,
    /// The idempotency key is blank; a replay could not be recognised.
    BlankIdempotencyKey,
    /// A whole-number field is not one.
    UnusableNumber,
    /// `direction` is not one of the three the domain defines.
    UnusableDirection,
    /// `consent` is neither "unrestricted" nor a grant list.
    UnusableConsent,
    /// The seed is not a set this surface can spell.
    Seed(SetRefusal),
}

impl SearchRefusal {
    pub(crate) fn cause(&self) -> &'static str {
        match self {
            Self::UnusableBody => {
                "a search-around is {\"idempotency_key\",\"seed\",\"edge_types\",\"max_depth\",\
                 \"direction\",\"consent\",\"freshness_floor_epoch_seconds\",\
                 \"observed_at_epoch_seconds\"}"
            }
            Self::BlankIdempotencyKey => "an idempotency key must name the attempt it repeats",
            Self::UnusableNumber => "a whole number is required, and must fit the field's width",
            Self::UnusableDirection => "a direction is \"outbound\", \"inbound\" or \"both\"",
            Self::UnusableConsent => {
                "consent is \"unrestricted\" or {\"granted\":[\"lty_...\"]}, and an empty grant \
                 list traverses nothing"
            }
            Self::Seed(refusal) => refusal.cause(),
        }
    }
}

/// What one request asks for: the seed to walk from, and the walk itself.
pub(crate) struct SearchAroundRequest {
    pub(crate) idempotency_key: String,
    pub(crate) entity_type: String,
    pub(crate) revision: u32,
    pub(crate) seed: SetDefinition,
    pub(crate) edge_types: Vec<String>,
    pub(crate) max_depth: u32,
    pub(crate) direction: TraversalDirection,
    pub(crate) consent: EdgeConsent,
    pub(crate) freshness_floor_epoch_seconds: u64,
    pub(crate) observed_at_epoch_seconds: u64,
}

impl SearchAroundRequest {
    pub(crate) fn parse(body: &str) -> Result<Self, SearchRefusal> {
        let body: SearchAroundBody =
            serde_json::from_str(body).map_err(|_| SearchRefusal::UnusableBody)?;
        if body.idempotency_key.trim().is_empty() {
            return Err(SearchRefusal::BlankIdempotencyKey);
        }
        Ok(Self {
            idempotency_key: body.idempotency_key,
            entity_type: body.seed.entity_type,
            revision: narrow(&body.seed.revision)?,
            seed: seed_definition(body.seed.set).map_err(SearchRefusal::Seed)?,
            edge_types: body.edge_types,
            max_depth: narrow(&body.max_depth)?,
            direction: direction_of(&body.direction)?,
            consent: consent_of(&body.consent)?,
            freshness_floor_epoch_seconds: whole(&body.freshness_floor_epoch_seconds)?,
            observed_at_epoch_seconds: whole(&body.observed_at_epoch_seconds)?,
        })
    }

    /// The domain request this asks for, seeded from `roots`. The first root is
    /// the walk's own root and the rest are the set's remaining members; the
    /// domain deduplicates them and starts every one at depth zero.
    pub(crate) fn into_query(
        self,
        tenant_id: &str,
        roots: Vec<String>,
    ) -> Result<KnowledgeGraphQueryRequest, KnowledgeGraphQueryError> {
        let query_id = format!("kgq_{}", self.idempotency_key);
        let mut roots = roots.into_iter();
        // The caller materialized a non-empty seed, so a root is present; an
        // empty one would be refused before this is reached.
        let root = roots.next().unwrap_or_default();
        KnowledgeGraphQueryRequest::new(
            tenant_id,
            query_id,
            root,
            self.edge_types,
            self.max_depth,
            self.freshness_floor_epoch_seconds,
            self.observed_at_epoch_seconds,
            self.consent,
            self.direction,
        )?
        .with_additional_roots(roots.collect())
    }
}

fn whole(value: &serde_json::Value) -> Result<u64, SearchRefusal> {
    value.as_u64().ok_or(SearchRefusal::UnusableNumber)
}

fn narrow(value: &serde_json::Value) -> Result<u32, SearchRefusal> {
    u32::try_from(whole(value)?).map_err(|_| SearchRefusal::UnusableNumber)
}

fn direction_of(raw: &str) -> Result<TraversalDirection, SearchRefusal> {
    match raw {
        "outbound" => Ok(TraversalDirection::Outbound),
        "inbound" => Ok(TraversalDirection::Inbound),
        "both" => Ok(TraversalDirection::Both),
        _ => Err(SearchRefusal::UnusableDirection),
    }
}

/// Consent has no silent default: a surface where it does not govern must say
/// "unrestricted", and a grant list that names nothing traverses nothing.
fn consent_of(value: &serde_json::Value) -> Result<EdgeConsent, SearchRefusal> {
    if value.as_str() == Some("unrestricted") {
        return Ok(EdgeConsent::Unrestricted);
    }
    let grants = value
        .get("granted")
        .and_then(serde_json::Value::as_array)
        .ok_or(SearchRefusal::UnusableConsent)?;
    let grants: Option<Vec<String>> = grants
        .iter()
        .map(|grant| grant.as_str().map(str::to_owned))
        .collect();
    Ok(EdgeConsent::Granted(
        grants.ok_or(SearchRefusal::UnusableConsent)?,
    ))
}
