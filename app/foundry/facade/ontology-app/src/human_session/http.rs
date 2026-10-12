use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use iam_identity_user_session::{
    ActiveUserMembership, SessionRefusal, SessionView, UserSessionAuthorizer,
};

use super::config::{HumanSessionConfig, HumanSessionConfigError};
use super::cookie::{SESSION_COOKIE, valid_token};

pub struct HttpUserSessionAuthorizer {
    config: HumanSessionConfig,
    client: reqwest::Client,
}

impl HttpUserSessionAuthorizer {
    pub const TIMEOUT: Duration = Duration::from_secs(2);
    pub const MAX_BODY_BYTES: usize = 16 * 1024;

    pub fn new(config: HumanSessionConfig) -> Result<Self, HumanSessionConfigError> {
        let client = reqwest::Client::builder()
            .timeout(Self::TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .build()
            .map_err(|_| HumanSessionConfigError)?;
        Ok(Self { config, client })
    }

    async fn fetch(
        &self,
        token: &str,
        tenant: &str,
        now: i64,
    ) -> Result<ActiveUserMembership, SessionRefusal> {
        let mut url = self.config.base.clone();
        url.path_segments_mut()
            .map_err(|_| SessionRefusal::MembershipUnavailable)?
            .clear()
            .extend(["identity", "v1", tenant, "session"]);
        let mut response = self
            .client
            .get(url)
            .header(reqwest::header::COOKIE, format!("{SESSION_COOKIE}={token}"))
            .send()
            .await
            .map_err(|_| SessionRefusal::MembershipUnavailable)?;
        match response.status() {
            reqwest::StatusCode::OK => {}
            reqwest::StatusCode::UNAUTHORIZED => return Err(SessionRefusal::Unauthenticated),
            reqwest::StatusCode::FORBIDDEN => return Err(SessionRefusal::InactiveMembership),
            _ => return Err(SessionRefusal::MembershipUnavailable),
        }
        if response
            .content_length()
            .is_some_and(|len| len > Self::MAX_BODY_BYTES as u64)
        {
            return Err(SessionRefusal::MembershipUnavailable);
        }
        let mut body = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| SessionRefusal::MembershipUnavailable)?
        {
            if chunk.len() > Self::MAX_BODY_BYTES - body.len() {
                return Err(SessionRefusal::MembershipUnavailable);
            }
            body.extend_from_slice(&chunk);
        }
        let view: SessionView =
            serde_json::from_slice(&body).map_err(|_| SessionRefusal::MembershipUnavailable)?;
        view.into_active_membership(tenant, now.max(super::now_epoch_seconds()?))
    }
}

impl UserSessionAuthorizer for HttpUserSessionAuthorizer {
    fn authorize<'a>(
        &'a self,
        presented: Option<&'a str>,
        expected_tenant: &'a str,
        now_epoch_seconds: i64,
    ) -> Pin<Box<dyn Future<Output = Result<ActiveUserMembership, SessionRefusal>> + Send + 'a>>
    {
        Box::pin(async move {
            let token = presented.ok_or(SessionRefusal::Unauthenticated)?;
            if !valid_token(token) {
                return Err(SessionRefusal::InvalidCredential);
            }
            self.fetch(token, expected_tenant, now_epoch_seconds).await
        })
    }
}
