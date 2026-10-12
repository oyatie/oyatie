use application_ontology_read::{
    AuditRow, EntityType, HistoryRow, ObjectDetail, ObjectListQuery, ObjectPage, OntologyReadError,
    OntologyReadSource, PinnedObject,
};
use reqwest::{StatusCode, Url, header::HeaderValue};
use serde::{Deserialize, de::DeserializeOwned};

use super::{HumanOntologyCardSource, query, transport};

const MAXIMUM_READ_BODY: usize = 1024 * 1024;

/// Request-scoped server transport. No credential or membership result is serializable.
/// Create from the freshly guarded shell cookie; Foundry reauthorizes every read.
pub struct HumanOntologyReadSource {
    transport: HumanOntologyCardSource,
    cookie: HeaderValue, // data_class: SECRET
}

impl HumanOntologyCardSource {
    pub fn read_source(
        &self,
        session_cookie: &str,
    ) -> Result<HumanOntologyReadSource, OntologyReadError> {
        let token = session_cookie
            .strip_prefix("__Host-oya_session=v1.")
            .ok_or(OntologyReadError::Unauthenticated)?;
        if token.len() != 43
            || !token
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
            || !token
                .as_bytes()
                .last()
                .is_some_and(|b| b"AEIMQUYcgkosw048".contains(b))
        {
            return Err(OntologyReadError::Unauthenticated);
        }
        let mut cookie = HeaderValue::from_str(session_cookie)
            .map_err(|_| OntologyReadError::Unauthenticated)?;
        cookie.set_sensitive(true);
        Ok(HumanOntologyReadSource {
            transport: self.clone(),
            cookie,
        })
    }
}

impl HumanOntologyReadSource {
    fn url(&self, tenant: &str, suffix: &[&str]) -> Result<Url, OntologyReadError> {
        if !tenant.starts_with("ten_") || tenant.len() <= 4 {
            return Err(OntologyReadError::InvalidQuery);
        }
        let mut url = self.transport.base.clone();
        url.path_segments_mut()
            .map_err(|_| OntologyReadError::SourceUnavailable)?
            .extend(["v1", "tenants", tenant])
            .extend(suffix);
        Ok(url)
    }

    fn object_url(
        &self,
        tenant: &str,
        object_ref: &str,
        history: bool,
    ) -> Result<Url, OntologyReadError> {
        if !object_ref.starts_with("ent_") || object_ref.len() <= 4 {
            return Err(OntologyReadError::InvalidQuery);
        }
        if history {
            self.url(tenant, &["objects", object_ref, "history"])
        } else {
            self.url(tenant, &["objects", object_ref])
        }
    }

    async fn get<T: DeserializeOwned>(&self, url: Url) -> Result<T, OntologyReadError> {
        let response = self.transport.request(url, &self.cookie).await?;
        match response.status() {
            StatusCode::OK => transport::decode(response, MAXIMUM_READ_BODY).await,
            StatusCode::UNAUTHORIZED => Err(OntologyReadError::Unauthenticated),
            StatusCode::FORBIDDEN => Err(OntologyReadError::Forbidden),
            status @ (StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND | StatusCode::CONFLICT) => {
                let refusal: Refusal = transport::decode(response, 16 * 1024).await?;
                Err(refusal.error(status))
            }
            _ => Err(OntologyReadError::SourceUnavailable),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativePage {
    objects: Vec<PinnedObject>,
    next: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Refusal {
    gate: String,
    cause: String,
}

impl Refusal {
    fn error(&self, status: StatusCode) -> OntologyReadError {
        if self.gate != "surface" {
            return OntologyReadError::SourceUnavailable;
        }
        match (status, self.cause.as_str()) {
            (StatusCode::BAD_REQUEST, _) => OntologyReadError::InvalidQuery,
            (StatusCode::NOT_FOUND, "the durable projection holds no object at this reference") => {
                OntologyReadError::AbsentObject
            }
            (StatusCode::CONFLICT, "that revision was never accepted for this entity type") => {
                OntologyReadError::RevisionConflict
            }
            (StatusCode::CONFLICT, "the stored values of that property are of another kind") => {
                OntologyReadError::FilterConflict
            }
            _ => OntologyReadError::SourceUnavailable,
        }
    }
}

impl OntologyReadSource for HumanOntologyReadSource {
    async fn types(&self, tenant: &str) -> Result<Vec<EntityType>, OntologyReadError> {
        self.get(self.url(tenant, &["types"])?).await
    }

    async fn objects(
        &self,
        tenant: &str,
        query: &ObjectListQuery,
    ) -> Result<ObjectPage, OntologyReadError> {
        let mut url = self.url(tenant, &["objects"])?;
        query::listing(&mut url, query)?;
        let page: NativePage = self.get(url).await?;
        Ok(ObjectPage {
            requested_revision: query.revision,
            objects: page.objects,
            next: page.next,
        })
    }

    async fn object(
        &self,
        tenant: &str,
        object_ref: &str,
        revision: u32,
    ) -> Result<ObjectDetail, OntologyReadError> {
        let mut url = self.object_url(tenant, object_ref, false)?;
        url.set_query(Some(&format!("revision={revision}")));
        Ok(ObjectDetail {
            requested_revision: revision,
            object: self.get(url).await?,
        })
    }

    async fn history(
        &self,
        tenant: &str,
        object_ref: &str,
    ) -> Result<Vec<HistoryRow>, OntologyReadError> {
        self.get(self.object_url(tenant, object_ref, true)?).await
    }

    async fn audit(&self, tenant: &str) -> Result<Vec<AuditRow>, OntologyReadError> {
        self.get(self.url(tenant, &["audit"])?).await
    }
}
