use messenger_domain::{ArchiveEvent, Decryption, Error};
use sqlx::{Row, postgres::PgRow};

pub(crate) fn unavailable(_error: sqlx::Error) -> Error {
    Error::Unavailable("archive store unavailable".into())
}

pub(crate) fn unique_violation(error: &sqlx::Error) -> bool {
    matches!(
        error,
        sqlx::Error::Database(database) if database.code().as_deref() == Some("23505")
    )
}

pub(crate) fn timestamp_sql(timestamp: u64) -> Result<i64, Error> {
    i64::try_from(timestamp).map_err(|_| Error::Invalid("invalid timestamp".into()))
}

pub(crate) fn encode_content(content: &serde_json::Value) -> Result<String, Error> {
    serde_json::to_string(content).map_err(|_| Error::Invalid("invalid event content".into()))
}

pub(crate) fn decryption_name(value: &Decryption) -> &'static str {
    match value {
        Decryption::Decrypted => "decrypted",
        Decryption::MissingKey => "missing_key",
        Decryption::Plaintext => "plaintext",
    }
}

pub(crate) fn encode_cursor(timestamp: u64, id: &str) -> String {
    format!("{timestamp}:{id}")
}

pub(crate) fn decode_cursor(cursor: &str) -> Result<(i64, String), Error> {
    let (timestamp, id) = cursor
        .split_once(':')
        .ok_or_else(|| Error::Invalid("invalid archive cursor".into()))?;
    if id.is_empty() {
        return Err(Error::Invalid("invalid archive cursor".into()));
    }
    let timestamp = timestamp
        .parse::<u64>()
        .map_err(|_| Error::Invalid("invalid archive cursor".into()))?;
    Ok((timestamp_sql(timestamp)?, id.to_owned()))
}

pub(crate) fn row_to_event(row: &PgRow) -> Result<ArchiveEvent, Error> {
    let timestamp: i64 = row.try_get("event_timestamp").map_err(unavailable)?;
    let content: String = row.try_get("content").map_err(unavailable)?;
    let decryption: String = row.try_get("decryption").map_err(unavailable)?;
    Ok(ArchiveEvent {
        id: row.try_get("id").map_err(unavailable)?,
        sender: row.try_get("sender").map_err(unavailable)?,
        timestamp: u64::try_from(timestamp)
            .map_err(|_| Error::Unavailable("archive store unavailable".into()))?,
        event_type: row.try_get("event_type").map_err(unavailable)?,
        content: serde_json::from_str(&content)
            .map_err(|_| Error::Unavailable("archive store unavailable".into()))?,
        decryption: decryption_from_name(&decryption)?,
    })
}

fn decryption_from_name(name: &str) -> Result<Decryption, Error> {
    match name {
        "decrypted" => Ok(Decryption::Decrypted),
        "missing_key" => Ok(Decryption::MissingKey),
        "plaintext" => Ok(Decryption::Plaintext),
        _ => Err(Error::Unavailable("archive store unavailable".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn cursor_round_trips() {
        let encoded = encode_cursor(42, "$event:1");
        let (timestamp, id) = decode_cursor(&encoded).unwrap();
        assert_eq!(timestamp, 42);
        assert_eq!(id, "$event:1");
        assert!(decode_cursor("bad").is_err());
        assert!(decode_cursor("1:").is_err());
    }

    #[test]
    fn decryption_and_content_round_trip() {
        assert_eq!(decryption_name(&Decryption::MissingKey), "missing_key");
        assert_eq!(
            decryption_from_name("plaintext").unwrap(),
            Decryption::Plaintext
        );
        assert!(decryption_from_name("other").is_err());
        let content = json!({"body": "hello"});
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&encode_content(&content).unwrap()).unwrap(),
            content
        );
        assert!(timestamp_sql(u64::MAX).is_err());
    }
}
