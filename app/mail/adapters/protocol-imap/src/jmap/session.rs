use super::Jmap;
use axum::Json;
use axum::http::StatusCode;
use serde_json::{Value, json};

pub(super) const CORE: &str = "urn:ietf:params:jmap:core";
pub(super) const MAIL: &str = "urn:ietf:params:jmap:mail";
pub(super) const SUBMISSION: &str = "urn:ietf:params:jmap:submission";
pub(super) const VACATION: &str = "urn:ietf:params:jmap:vacationresponse";
pub(super) const BLOB: &str = "urn:ietf:params:jmap:blob";
pub(super) const MAX_REQUEST: usize = 1024 * 1024;

pub(super) fn known(capability: &Value) -> bool {
    matches!(
        capability.as_str(),
        Some(CORE | MAIL | BLOB | SUBMISSION | VACATION)
    )
}

pub(super) fn capability(name: &str) -> &'static str {
    if matches!(name, "Core/echo" | "Blob/copy") {
        CORE
    } else if name.starts_with("Identity/") || name.starts_with("EmailSubmission/") {
        SUBMISSION
    } else if name.starts_with("VacationResponse/") {
        VACATION
    } else {
        MAIL
    }
}

pub(super) fn response(state: Jmap, token: &str) -> Result<Json<Value>, StatusCode> {
    let principal = state
        .service
        .principal(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let account = state
        .service
        .authorize(token, &principal.account, mail_api::Action::Read)
        .map_err(|_| StatusCode::FORBIDDEN)?;
    let write = super::mailbox::writable(&state.service, token, &account.id)
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(Json(json!({
        "capabilities": { CORE: {"maxSizeUpload":26214400,"maxConcurrentUpload":1,"maxSizeRequest":MAX_REQUEST,"maxConcurrentRequests":4,"maxCallsInRequest":16,"maxObjectsInGet":256,"maxObjectsInSet":256,"collationAlgorithms":["i;ascii-casemap"]}, MAIL:{}, SUBMISSION:{}, VACATION:{} },
        "accounts": { &account.id: { "name":account.address,"isPersonal":true,"isReadOnly":!write,"accountCapabilities": { MAIL: {"maxMailboxesPerEmail":null,"maxMailboxDepth":null,"maxSizeMailboxName":255,"maxSizeAttachmentsPerEmail":26214400,"emailQuerySortOptions":["receivedAt"],"mayCreateTopLevelMailbox":write}, SUBMISSION:{"maxDelayedSend":mail_service::MAX_DELAYED_SEND,"submissionExtensions":{"FUTURERELEASE":[]}}, VACATION:{} } } },
        "primaryAccounts": {MAIL:account.id,SUBMISSION:account.id,VACATION:account.id}, "username":account.address,
        "apiUrl":format!("{}/jmap",state.base),
        "downloadUrl":format!("{}/download/{{accountId}}/{{blobId}}/{{name}}?type={{type}}",state.base),
        "uploadUrl":format!("{}/upload/{{accountId}}",state.base),
        "eventSourceUrl":format!("{}/events?types={{types}}&closeafter={{closeafter}}&ping={{ping}}",state.base),
        "state":"1"
    })))
}
