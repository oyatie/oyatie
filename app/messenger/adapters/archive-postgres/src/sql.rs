pub(crate) const CREATE_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS messenger_archive_events (
    room TEXT NOT NULL,
    id TEXT NOT NULL,
    sender TEXT NOT NULL,
    event_timestamp BIGINT NOT NULL,
    event_type TEXT NOT NULL,
    content TEXT NOT NULL,
    decryption TEXT NOT NULL,
    PRIMARY KEY (room, id)
)";

pub(crate) const CREATE_PAGE_INDEX_SQL: &str = "CREATE INDEX IF NOT EXISTS messenger_archive_events_page \
     ON messenger_archive_events (room, event_timestamp, id)";

pub(crate) const INSERT_SQL: &str = "INSERT INTO messenger_archive_events \
     (room, id, sender, event_timestamp, event_type, content, decryption) \
     VALUES ($1, $2, $3, $4, $5, $6, $7)";

pub(crate) const SELECT_ONE_SQL: &str = "SELECT id, sender, event_timestamp, event_type, content, decryption \
     FROM messenger_archive_events WHERE room = $1 AND id = $2";

pub(crate) const PAGE_SQL: &str = "SELECT id, sender, event_timestamp, event_type, content, decryption \
     FROM messenger_archive_events \
     WHERE room = $1 AND (event_timestamp, id) > ($2, $3) \
     ORDER BY event_timestamp ASC, id ASC LIMIT $4";

#[cfg(test)]
mod tests {
    #[test]
    fn schema_sql_is_embedded() {
        assert!(super::CREATE_TABLE_SQL.contains("messenger_archive_events"));
        assert!(super::INSERT_SQL.contains("event_timestamp"));
        assert!(super::PAGE_SQL.contains("ORDER BY event_timestamp"));
    }
}
