//! CloudEvent 1.0 envelope with oyatie topic-naming-convention enforcement.
//!
//! The CloudEvents 1.0 spec (<https://cloudevents.io>) defines a minimal envelope
//! for event interoperability. Oyatie adds two extension attributes:
//! - `oyatie_tenant_id`: tenant that produced the event
//! - `oyatie_cell_id`: cell/region that produced the event
//!
//! Topic naming convention (enforced at construction):
//! `t.{tenant_id}.{microservice}.{event_type}.v{version}`
//!
//! ADR reference: IP-P05-eventing-substrate (M02b/P05)
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// Error returned when a [`CloudEvent`] cannot be constructed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudEventError {
    /// `tenant_id` was empty or contained whitespace only.
    EmptyTenantId,
    /// `microservice` was empty or contained whitespace only.
    EmptyMicroservice,
    /// `event_type` was empty or contained whitespace only.
    EmptyEventType,
    /// `cell_id` was empty or contained whitespace only.
    EmptyCellId,
    /// `microservice` or `event_type` contained a dot, which would break the
    /// structured topic name.
    IllegalDotInComponent,
}

impl std::fmt::Display for CloudEventError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyTenantId => f.write_str("cloud_event: tenant_id must be non-empty"),
            Self::EmptyMicroservice => f.write_str("cloud_event: microservice must be non-empty"),
            Self::EmptyEventType => f.write_str("cloud_event: event_type must be non-empty"),
            Self::EmptyCellId => f.write_str("cloud_event: cell_id must be non-empty"),
            Self::IllegalDotInComponent => {
                f.write_str("cloud_event: microservice and event_type must not contain '.'")
            }
        }
    }
}

/// CloudEvents 1.0 envelope with oyatie tenant + cell extensions.
///
/// Constructed via [`CloudEvent::new`], which enforces the oyatie topic naming
/// convention and validates all fields.
///
/// The `spec_version` attribute is always `"1.0"`.
/// The `data_content_type` attribute is always `"application/json"`.
/// The `id` field is a caller-supplied opaque string (typically a UUID) used
/// as the idempotency key.
///
/// # Topic naming convention
///
/// ```text
/// t.{tenant_id}.{microservice}.{event_type}.v{version}
/// ```
///
/// Example: `t.acme.eventing.outbox_dispatched.v1`
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudEvent {
    /// CloudEvents spec version — always `"1.0"`.
    pub spec_version: &'static str,
    /// Unique event identifier; serves as idempotency key.
    pub id: String, // data_class: INTERNAL_ONLY
    /// Source URI: `//oyatie/{microservice}`.
    pub source: String, // data_class: INTERNAL_ONLY
    /// Structured topic name (the CloudEvents `type` attribute).
    /// Format: `t.{tenant_id}.{microservice}.{event_type}.v{version}`
    pub event_type: String, // data_class: INTERNAL_ONLY
    /// Always `"application/json"`.
    pub data_content_type: &'static str,
    /// ISO-8601 timestamp string supplied by the caller.
    pub time: String, // data_class: INTERNAL_ONLY
    /// Raw JSON payload (opaque to the eventing substrate).
    pub data: String, // data_class: INTERNAL_ONLY
    /// Oyatie extension: tenant that produced the event.
    pub oyatie_tenant_id: String, // data_class: INTERNAL_ONLY
    /// Oyatie extension: cell/region that produced the event.
    pub oyatie_cell_id: String, // data_class: INTERNAL_ONLY
}

impl CloudEvent {
    /// Construct a [`CloudEvent`] and enforce the oyatie topic naming convention.
    ///
    /// # Arguments
    ///
    /// - `id` — caller-supplied unique identifier (e.g. UUID string).
    /// - `tenant_id` — tenant context; embedded in the topic name.
    /// - `cell_id` — cell/region identifier.
    /// - `microservice` — producing microservice name; must not contain `'.'`.
    /// - `event_type` — event type token; must not contain `'.'`.
    /// - `version` — semantic version integer (≥ 1).
    /// - `time` — ISO-8601 timestamp string.
    /// - `data` — JSON payload string.
    ///
    /// # Errors
    ///
    /// Returns [`CloudEventError`] if any field fails validation.
    pub fn new(
        id: impl Into<String>,
        tenant_id: impl Into<String>,
        cell_id: impl Into<String>,
        microservice: impl Into<String>,
        event_type: impl Into<String>,
        version: u32,
        time: impl Into<String>,
        data: impl Into<String>,
    ) -> Result<Self, CloudEventError> {
        let id = id.into();
        let tenant_id = tenant_id.into();
        let cell_id = cell_id.into();
        let microservice = microservice.into();
        let event_type_str = event_type.into();
        let time = time.into();
        let data = data.into();

        if tenant_id.trim().is_empty() {
            return Err(CloudEventError::EmptyTenantId);
        }
        if cell_id.trim().is_empty() {
            return Err(CloudEventError::EmptyCellId);
        }
        if microservice.trim().is_empty() {
            return Err(CloudEventError::EmptyMicroservice);
        }
        if event_type_str.trim().is_empty() {
            return Err(CloudEventError::EmptyEventType);
        }
        if microservice.contains('.') || event_type_str.contains('.') {
            return Err(CloudEventError::IllegalDotInComponent);
        }

        let structured_type = format!("t.{tenant_id}.{microservice}.{event_type_str}.v{version}");
        let source = format!("//oyatie/{microservice}");

        Ok(Self {
            spec_version: "1.0",
            id,
            source,
            event_type: structured_type,
            data_content_type: "application/json",
            time,
            data,
            oyatie_tenant_id: tenant_id,
            oyatie_cell_id: cell_id,
        })
    }

    /// Parse the `event_type` field back into its components.
    ///
    /// Returns `(tenant_id, microservice, event_type_token, version)` or
    /// `None` if the field does not match the expected shape.
    pub fn parse_topic(event_type: &str) -> Option<(&str, &str, &str, &str)> {
        // expected: t.<tenant_id>.<microservice>.<event_type>.v<version>
        let body = event_type.strip_prefix("t.")?;
        // Split into exactly 4 dot-separated parts: tenant_id, microservice, event_type, version
        let mut parts = body.splitn(4, '.');
        let tenant_id = parts.next()?;
        let microservice = parts.next()?;
        let event_type_token = parts.next()?;
        let version = parts.next()?;
        if !version.starts_with('v') {
            return None;
        }
        Some((tenant_id, microservice, event_type_token, version))
    }
}

#[cfg(test)]
mod tests {
    use super::{CloudEvent, CloudEventError};

    fn make_event() -> CloudEvent {
        CloudEvent::new(
            "evt-id-1",
            "acme",
            "cell-us-east-1",
            "eventing",
            "outbox_dispatched",
            1,
            "2026-05-17T00:00:00Z",
            r#"{"key":"value"}"#,
        )
        .expect("valid event")
    }

    #[test]
    fn cloud_event_topic_naming_convention_enforced() {
        let ev = make_event();
        assert_eq!(ev.event_type, "t.acme.eventing.outbox_dispatched.v1");
        assert_eq!(ev.source, "//oyatie/eventing");
        assert_eq!(ev.spec_version, "1.0");
        assert_eq!(ev.data_content_type, "application/json");
    }

    #[test]
    fn cloud_event_parse_topic_roundtrips() {
        let ev = make_event();
        let parsed = CloudEvent::parse_topic(&ev.event_type).expect("structured type is parseable");
        assert_eq!(parsed.0, "acme");
        assert_eq!(parsed.1, "eventing");
        assert_eq!(parsed.2, "outbox_dispatched");
        assert_eq!(parsed.3, "v1");
    }

    #[test]
    fn cloud_event_version_increments_in_topic() {
        let ev = CloudEvent::new(
            "evt-id-2",
            "acme",
            "cell-us-east-1",
            "eventing",
            "outbox_dispatched",
            3,
            "2026-05-17T00:00:00Z",
            "{}",
        )
        .expect("valid event");
        assert_eq!(ev.event_type, "t.acme.eventing.outbox_dispatched.v3");
    }

    #[test]
    fn cloud_event_rejects_empty_tenant_id() {
        let err =
            CloudEvent::new("id", "", "cell", "eventing", "dispatched", 1, "t", "{}").unwrap_err();
        assert_eq!(err, CloudEventError::EmptyTenantId);
    }

    #[test]
    fn cloud_event_rejects_empty_microservice() {
        let err =
            CloudEvent::new("id", "acme", "cell", "", "dispatched", 1, "t", "{}").unwrap_err();
        assert_eq!(err, CloudEventError::EmptyMicroservice);
    }

    #[test]
    fn cloud_event_rejects_empty_event_type() {
        let err = CloudEvent::new("id", "acme", "cell", "eventing", "", 1, "t", "{}").unwrap_err();
        assert_eq!(err, CloudEventError::EmptyEventType);
    }

    #[test]
    fn cloud_event_rejects_empty_cell_id() {
        let err =
            CloudEvent::new("id", "acme", "", "eventing", "dispatched", 1, "t", "{}").unwrap_err();
        assert_eq!(err, CloudEventError::EmptyCellId);
    }

    #[test]
    fn cloud_event_rejects_dot_in_microservice() {
        let err = CloudEvent::new(
            "id",
            "acme",
            "cell",
            "eventing.sub",
            "dispatched",
            1,
            "t",
            "{}",
        )
        .unwrap_err();
        assert_eq!(err, CloudEventError::IllegalDotInComponent);
    }

    #[test]
    fn cloud_event_rejects_dot_in_event_type() {
        let err = CloudEvent::new(
            "id",
            "acme",
            "cell",
            "eventing",
            "out.dispatched",
            1,
            "t",
            "{}",
        )
        .unwrap_err();
        assert_eq!(err, CloudEventError::IllegalDotInComponent);
    }

    #[test]
    fn cloud_event_error_display_is_human_readable() {
        assert!(!CloudEventError::EmptyTenantId.to_string().is_empty());
        assert!(!CloudEventError::EmptyMicroservice.to_string().is_empty());
        assert!(!CloudEventError::EmptyEventType.to_string().is_empty());
        assert!(!CloudEventError::EmptyCellId.to_string().is_empty());
        assert!(
            !CloudEventError::IllegalDotInComponent
                .to_string()
                .is_empty()
        );
    }

    #[test]
    fn cloud_event_parse_topic_rejects_non_v_version() {
        assert!(CloudEvent::parse_topic("t.acme.eventing.dispatched.1").is_none());
    }

    #[test]
    fn cloud_event_parse_topic_rejects_wrong_prefix() {
        assert!(CloudEvent::parse_topic("x.acme.eventing.dispatched.v1").is_none());
    }
}
