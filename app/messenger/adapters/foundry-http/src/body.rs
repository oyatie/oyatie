use messenger_domain::Error;

pub(crate) const MAX_RESPONSE_BYTES: usize = 1_048_576;

pub(crate) async fn bounded_json(
    mut response: reqwest::Response,
) -> Result<serde_json::Value, Error> {
    if !response.status().is_success() {
        return Err(Error::Denied);
    }
    let oversized = || Error::Unavailable("Foundry response exceeds 1 MiB".into());
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return Err(oversized());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| Error::Unavailable("Foundry response unavailable".into()))?
    {
        if chunk.len() > MAX_RESPONSE_BYTES - bytes.len() {
            return Err(oversized());
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes)
        .map_err(|_| Error::Unavailable("Foundry response unavailable".into()))
}
