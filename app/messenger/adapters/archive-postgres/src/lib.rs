#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

mod map;
mod sql;

use map::{
    decode_cursor, decryption_name, encode_content, encode_cursor, row_to_event, timestamp_sql,
    unavailable, unique_violation,
};
use messenger_archive_api::{Archive, same_archive_event, validate_capture, validate_page};
use messenger_domain::{ArchiveEvent, ArchiveEventPage, Error};
use sql::{CREATE_PAGE_INDEX_SQL, CREATE_TABLE_SQL, INSERT_SQL, PAGE_SQL, SELECT_ONE_SQL};
use sqlx::PgPool;

#[derive(Clone)]
pub struct PostgresArchive {
    pool: PgPool,
}

impl PostgresArchive {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn ensure_schema(&self) -> Result<(), Error> {
        sqlx::query(CREATE_TABLE_SQL)
            .execute(&self.pool)
            .await
            .map_err(unavailable)?;
        sqlx::query(CREATE_PAGE_INDEX_SQL)
            .execute(&self.pool)
            .await
            .map_err(unavailable)?;
        Ok(())
    }

    async fn load(&self, room: &str, id: &str) -> Result<Option<ArchiveEvent>, Error> {
        let row = sqlx::query(SELECT_ONE_SQL)
            .bind(room)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(unavailable)?;
        row.map(|row| row_to_event(&row)).transpose()
    }
}

impl Archive for PostgresArchive {
    async fn capture(&self, room: &str, event: ArchiveEvent) -> Result<(), Error> {
        validate_capture(room, &event)?;
        let timestamp = timestamp_sql(event.timestamp)?;
        let content = encode_content(&event.content)?;
        let result = sqlx::query(INSERT_SQL)
            .bind(room)
            .bind(&event.id)
            .bind(&event.sender)
            .bind(timestamp)
            .bind(&event.event_type)
            .bind(&content)
            .bind(decryption_name(&event.decryption))
            .execute(&self.pool)
            .await;
        match result {
            Ok(_) => Ok(()),
            Err(error) if unique_violation(&error) => match self.load(room, &event.id).await? {
                Some(stored) if same_archive_event(&stored, &event) => Ok(()),
                _ => Err(Error::Invalid("archive event conflict".into())),
            },
            Err(error) => Err(unavailable(error)),
        }
    }

    async fn page(
        &self,
        room: &str,
        after: Option<&str>,
        limit: u16,
    ) -> Result<ArchiveEventPage, Error> {
        validate_page(room, limit)?;
        let (after_ts, after_id) = match after {
            Some(cursor) => decode_cursor(cursor)?,
            None => (-1, String::new()),
        };
        let rows = sqlx::query(PAGE_SQL)
            .bind(room)
            .bind(after_ts)
            .bind(&after_id)
            .bind(i64::from(limit) + 1)
            .fetch_all(&self.pool)
            .await
            .map_err(unavailable)?;
        let mut events = Vec::with_capacity(rows.len().min(usize::from(limit)));
        for row in rows.iter().take(usize::from(limit)) {
            events.push(row_to_event(row)?);
        }
        let next = (rows.len() > usize::from(limit))
            .then(|| {
                events
                    .last()
                    .map(|event| encode_cursor(event.timestamp, &event.id))
            })
            .flatten();
        Ok(ArchiveEventPage { events, next })
    }
}
