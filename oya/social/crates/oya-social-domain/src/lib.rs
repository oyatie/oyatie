//! Service-local social workplace domain slice.
//!
//! This crate owns the first `oya/social/**` implementation seam for the Social PRD RED
//! fixtures: immutable context/pillar binding for `SocialPost`, feed/search/Ontology
//! filtering by context, and a replayable `community.social.post.v1` event envelope.
//! It is deliberately in-memory and framework-free; no storage, network, UI, Workflow,
//! Ontology runtime, deployment, production-readiness, or hyperscaler claim is made.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` / `panic!()`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextKind {
    Personal,
    Professional,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnershipPillar {
    Personal,
    Professional,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediaFlavor {
    Text,
    Photo,
    ShortVideo,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SocialError {
    Invalid,
    ContextPillarMismatch,
    DuplicatePost,
    ProfessionalPostRequiresConsent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SocialPostCreate {
    pub post_id: String,                   // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub creator_ref: String,               // data_class: PII_IDENTIFYING
    pub context_kind: ContextKind,         // data_class: INTERNAL_ONLY
    pub ownership_pillar: OwnershipPillar, // data_class: INTERNAL_ONLY
    pub body: String,                      // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub media_flavor: MediaFlavor,         // data_class: INTERNAL_ONLY
    pub idempotency_key: String,           // data_class: INTERNAL_ONLY
    pub consent_token: Option<String>,     // data_class: DECLARED_PREFERENCE
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SocialPost {
    post_id: String,                   // data_class: INTERNAL_ONLY
    tenant_id: String,                 // data_class: INTERNAL_ONLY
    creator_ref: String,               // data_class: PII_IDENTIFYING
    context_kind: ContextKind,         // data_class: INTERNAL_ONLY
    ownership_pillar: OwnershipPillar, // data_class: INTERNAL_ONLY
    body: String,                      // data_class: BEHAVIORAL_TENANT_PRODUCT
    media_flavor: MediaFlavor,         // data_class: INTERNAL_ONLY
    idempotency_key: String,           // data_class: INTERNAL_ONLY
    consent_token: Option<String>,     // data_class: DECLARED_PREFERENCE
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SocialPostPublishedEvent {
    pub contract_family: &'static str,
    pub event_name: &'static str,
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub post_id: String,                   // data_class: INTERNAL_ONLY
    pub creator_ref: String,               // data_class: PII_IDENTIFYING
    pub context_kind: ContextKind,         // data_class: INTERNAL_ONLY
    pub ownership_pillar: OwnershipPillar, // data_class: INTERNAL_ONLY
    pub media_flavor: MediaFlavor,         // data_class: INTERNAL_ONLY
    pub idempotency_key: String,           // data_class: INTERNAL_ONLY
    pub consent_token: Option<String>,     // data_class: DECLARED_PREFERENCE
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SocialFeedIndex {
    posts: Vec<SocialPost>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractReplayReport {
    pub contract_family: &'static str,
    pub event_name: &'static str,
    pub event_count: usize,
    pub no_cross_context_feed_leak: bool,
    pub no_cross_context_search_leak: bool,
    pub no_cross_context_ontology_leak: bool,
}

impl SocialPost {
    pub fn publish(input: SocialPostCreate) -> Result<Self, SocialError> {
        non_empty(&input.post_id)?;
        non_empty(&input.tenant_id)?;
        non_empty(&input.creator_ref)?;
        non_empty(&input.body)?;
        non_empty(&input.idempotency_key)?;
        if input.context_kind.expected_pillar() != input.ownership_pillar {
            return Err(SocialError::ContextPillarMismatch);
        }
        if input.context_kind == ContextKind::Professional
            && input
                .consent_token
                .as_deref()
                .map(str::trim)
                .unwrap_or("")
                .is_empty()
        {
            return Err(SocialError::ProfessionalPostRequiresConsent);
        }
        Ok(Self {
            post_id: input.post_id,
            tenant_id: input.tenant_id,
            creator_ref: input.creator_ref,
            context_kind: input.context_kind,
            ownership_pillar: input.ownership_pillar,
            body: input.body,
            media_flavor: input.media_flavor,
            idempotency_key: input.idempotency_key,
            consent_token: input.consent_token,
        })
    }

    pub fn post_id(&self) -> &str {
        &self.post_id
    }

    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    pub fn creator_ref(&self) -> &str {
        &self.creator_ref
    }

    pub fn context_kind(&self) -> ContextKind {
        self.context_kind
    }

    pub fn ownership_pillar(&self) -> OwnershipPillar {
        self.ownership_pillar
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn media_flavor(&self) -> MediaFlavor {
        self.media_flavor
    }

    pub fn service_authority(&self) -> &'static str {
        "social"
    }

    pub fn retired_standalone_shorts_authority(&self) -> Option<&'static str> {
        None
    }

    pub fn to_post_published_event(&self) -> SocialPostPublishedEvent {
        SocialPostPublishedEvent {
            contract_family: "community.social.post.v1",
            event_name: "SocialContentPublished",
            tenant_id: self.tenant_id.clone(),
            post_id: self.post_id.clone(),
            creator_ref: self.creator_ref.clone(),
            context_kind: self.context_kind,
            ownership_pillar: self.ownership_pillar,
            media_flavor: self.media_flavor,
            idempotency_key: self.idempotency_key.clone(),
            consent_token: self.consent_token.clone(),
        }
    }
}

impl ContextKind {
    pub fn expected_pillar(self) -> OwnershipPillar {
        match self {
            Self::Personal => OwnershipPillar::Personal,
            Self::Professional => OwnershipPillar::Professional,
        }
    }
}

impl SocialFeedIndex {
    pub fn publish(&mut self, post: SocialPost) -> Result<(), SocialError> {
        if self.posts.iter().any(|existing| {
            existing.tenant_id == post.tenant_id
                && (existing.post_id == post.post_id
                    || existing.idempotency_key == post.idempotency_key)
        }) {
            return Err(SocialError::DuplicatePost);
        }
        self.posts.push(post);
        Ok(())
    }

    pub fn feed_for(
        &self,
        tenant_id: &str,
        context_kind: ContextKind,
        ownership_pillar: OwnershipPillar,
    ) -> Vec<&SocialPost> {
        let tenant_id = tenant_id.trim();
        if tenant_id.is_empty() {
            return Vec::new();
        }
        self.posts
            .iter()
            .filter(|post| {
                post.tenant_id == tenant_id
                    && post.context_kind == context_kind
                    && post.ownership_pillar == ownership_pillar
            })
            .collect()
    }

    pub fn search(
        &self,
        query: &str,
        tenant_id: &str,
        context_kind: ContextKind,
        ownership_pillar: OwnershipPillar,
    ) -> Vec<&SocialPost> {
        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return Vec::new();
        }
        self.feed_for(tenant_id, context_kind, ownership_pillar)
            .into_iter()
            .filter(|post| post.body.to_lowercase().contains(&query))
            .collect()
    }

    pub fn ontology_projection_for(
        &self,
        tenant_id: &str,
        context_kind: ContextKind,
        ownership_pillar: OwnershipPillar,
    ) -> Vec<String> {
        self.feed_for(tenant_id, context_kind, ownership_pillar)
            .into_iter()
            .map(|post| post.post_id.clone())
            .collect()
    }

    pub fn replay_post_contracts(&self) -> ContractReplayReport {
        let events: Vec<SocialPostPublishedEvent> = self
            .posts
            .iter()
            .map(SocialPost::to_post_published_event)
            .collect();
        ContractReplayReport {
            contract_family: "community.social.post.v1",
            event_name: "SocialContentPublished",
            event_count: events.len(),
            no_cross_context_feed_leak: self.no_cross_context_feed_leak(),
            no_cross_context_search_leak: self.no_cross_context_search_leak(),
            no_cross_context_ontology_leak: self.no_cross_context_ontology_leak(),
        }
    }

    fn no_cross_context_feed_leak(&self) -> bool {
        self.posts.iter().all(|post| {
            self.feed_for(&post.tenant_id, post.context_kind, post.ownership_pillar)
                .iter()
                .all(|candidate| {
                    candidate.tenant_id == post.tenant_id
                        && candidate.context_kind == post.context_kind
                        && candidate.ownership_pillar == post.ownership_pillar
                })
        })
    }

    fn no_cross_context_search_leak(&self) -> bool {
        self.posts.iter().all(|post| {
            self.search(
                &post.body,
                &post.tenant_id,
                post.context_kind,
                post.ownership_pillar,
            )
            .iter()
            .all(|candidate| {
                candidate.tenant_id == post.tenant_id
                    && candidate.context_kind == post.context_kind
                    && candidate.ownership_pillar == post.ownership_pillar
            })
        })
    }

    fn no_cross_context_ontology_leak(&self) -> bool {
        self.posts.iter().all(|post| {
            let projection = self.ontology_projection_for(
                &post.tenant_id,
                post.context_kind,
                post.ownership_pillar,
            );
            projection.contains(&post.post_id)
                && projection.iter().all(|post_id| {
                    self.posts.iter().any(|candidate| {
                        candidate.post_id == *post_id
                            && candidate.tenant_id == post.tenant_id
                            && candidate.context_kind == post.context_kind
                            && candidate.ownership_pillar == post.ownership_pillar
                    })
                })
        })
    }
}

fn non_empty(value: &str) -> Result<(), SocialError> {
    if value.trim().is_empty() {
        Err(SocialError::Invalid)
    } else {
        Ok(())
    }
}
