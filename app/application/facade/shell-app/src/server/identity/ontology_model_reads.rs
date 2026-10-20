use super::*;

pub(super) async fn drafts(
    State(c): State<Configuration>,
    path: ModelPath<String>,
    RawQuery(q): RawQuery,
    h: HeaderMap,
) -> Response {
    response(
        async {
            let t = extract_path(path)?;
            let (s, m) = source(c, &t, &h, false).await?;
            let url =
                reqwest::Url::parse(&format!("https://query.invalid/?{}", q.unwrap_or_default()))
                    .map_err(|_| refused(ModelRefusalCode::MalformedRequest))?;
            let mut after = None;
            let mut limit = None;
            for (k, v) in url.query_pairs() {
                match k.as_ref() {
                    "after" if after.is_none() => after = Some(number(&v)?),
                    "limit"
                        if limit.is_none()
                            && !v.starts_with('0')
                            && v.bytes().all(|b| b.is_ascii_digit()) =>
                    {
                        limit = Some(
                            v.parse::<usize>()
                                .map_err(|_| refused(ModelRefusalCode::MalformedRequest))?,
                        )
                    }
                    _ => return Err(refused(ModelRefusalCode::MalformedRequest)),
                }
            }
            Ok((s.owned_drafts(&t, after, limit.unwrap_or(25)).await?, m))
        }
        .await,
        StatusCode::OK,
    )
}

fn expected(query: Option<&str>) -> Result<PrepareModelDraft, ModelSourceError> {
    let malformed = || refused(ModelRefusalCode::MalformedRequest);
    let mut version = None;
    let mut base = None;
    for pair in query.ok_or_else(malformed)?.split('&') {
        let (key, value) = pair.split_once('=').ok_or_else(malformed)?;
        let value = number(value)?;
        match key {
            "expected_version" if version.is_none() => version = Some(value),
            "expected_base" if base.is_none() => base = Some(value),
            _ => return Err(malformed()),
        }
    }
    Ok(PrepareModelDraft {
        expected_version: version.ok_or_else(malformed)?,
        expected_base: base.ok_or_else(malformed)?,
    })
}

pub(super) async fn base(
    State(c): State<Configuration>,
    path: ModelPath<(String, String)>,
    RawQuery(query): RawQuery,
    headers: HeaderMap,
) -> Response {
    response(
        async {
            let (tenant, id) = extract_path(path)?;
            let (source, member) = source(c, &tenant, &headers, false).await?;
            Ok((
                source
                    .draft_base(&tenant, number(&id)?, &expected(query.as_deref())?)
                    .await?,
                member,
            ))
        }
        .await,
        StatusCode::OK,
    )
}
