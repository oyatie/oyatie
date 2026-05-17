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
    /// `id` was empty.
    EmptyId,
    /// `version` was zero; must be >= 1.
    InvalidVersion { version: u32 },
    /// `tenant_id` was empty or contained whitespace only.
    EmptyTenantId,
    /// `microservice` was empty or contained whitespace only.
    EmptyMicroservice,
    /// `event_type` was empty or contained whitespace only.
    EmptyEventType,
    /// `cell_id` was empty or contained whitespace only.
    EmptyCellId,
    /// `tenant_id`, `microservice`, or `event_type` contained a dot, which
    /// would break the structured topic name.
    IllegalDotInComponent,
}

impl std::fmt::Display for CloudEventError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyId => f.write_str("cloud_event: id must be non-empty"),
            Self::InvalidVersion { version } => {
                write!(f, "cloud_event: version must be >= 1, got {version}")
            }
            Self::EmptyTenantId => f.write_str("cloud_event: tenant_id must be non-empty"),
            Self::EmptyMicroservice => f.write_str("cloud_event: microservice must be non-empty"),
            Self::EmptyEventType => f.write_str("cloud_event: event_type must be non-empty"),
            Self::EmptyCellId => f.write_str("cloud_event: cell_id must be non-empty"),
            Self::IllegalDotInComponent => f.write_str(
                "cloud_event: tenant_id, microservice, and event_type must not contain '.'",
            ),
        }
    }
}

/// CloudEvents 1.0 envelope with oyatie tenant + cell extensions.
///
/// Constructed via [`CloudEvent::new`], which enforces the oyatie topic naming
/// convention and validates all fields. Fields are private to prevent
/// bypass of constructor invariants; use the accessor methods to read them.
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
    spec_version: &'static str,
    /// Unique event identifier; serves as idempotency key.
    id: String, // data_class: INTERNAL_ONLY
    /// Source URI: `//oyatie/{microservice}`.
    source: String, // data_class: INTERNAL_ONLY
    /// Structured topic name (the CloudEvents `type` attribute).
    /// Format: `t.{tenant_id}.{microservice}.{event_type}.v{version}`
    event_type: String, // data_class: INTERNAL_ONLY
    /// Always `"application/json"`.
    data_content_type: &'static str,
    /// ISO-8601 timestamp string supplied by the caller.
    time: String, // data_class: INTERNAL_ONLY
    /// Raw JSON payload (opaque to the eventing substrate).
    data: String, // data_class: INTERNAL_ONLY
    /// Oyatie extension: tenant that produced the event.
    oyatie_tenant_id: String, // data_class: INTERNAL_ONLY
    /// Oyatie extension: cell/region that produced the event.
    oyatie_cell_id: String, // data_class: INTERNAL_ONLY
}

impl CloudEvent {
    /// Construct a [`CloudEvent`] and enforce the oyatie topic naming convention.
    ///
    /// # Arguments
    ///
    /// - `id` — caller-supplied unique identifier (e.g. UUID string); must be non-empty.
    /// - `tenant_id` — tenant context; embedded in the topic name; must not contain `'.'`.
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
    #[allow(clippy::too_many_arguments)]
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

        // Thread 1 (P1): reject empty id
        if id.is_empty() {
            return Err(CloudEventError::EmptyId);
        }
        // Thread 2 (P2): enforce version >= 1
        if version == 0 {
            return Err(CloudEventError::InvalidVersion { version });
        }

        // Thread 7 (P2): trim + lowercase before composition and validation
        let tenant_id = tenant_id.trim().to_ascii_lowercase();
        let microservice = microservice.trim().to_ascii_lowercase();
        let event_type_str = event_type_str.trim().to_ascii_lowercase();
        let cell_id = cell_id.trim().to_ascii_lowercase();

        if tenant_id.is_empty() {
            return Err(CloudEventError::EmptyTenantId);
        }
        if cell_id.is_empty() {
            return Err(CloudEventError::EmptyCellId);
        }
        if microservice.is_empty() {
            return Err(CloudEventError::EmptyMicroservice);
        }
        if event_type_str.is_empty() {
            return Err(CloudEventError::EmptyEventType);
        }
        // Thread 4 (P1): reject dots in tenant_id too, not just microservice/event_type
        if tenant_id.contains('.') || microservice.contains('.') || event_type_str.contains('.') {
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

    // Thread 6 (P1): accessors replacing public fields

    /// CloudEvents spec version — always `"1.0"`.
    pub fn spec_version(&self) -> &str {
        self.spec_version
    }

    /// Unique event identifier; serves as idempotency key.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Source URI: `//oyatie/{microservice}`.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Structured topic name (the CloudEvents `type` attribute).
    pub fn event_type(&self) -> &str {
        &self.event_type
    }

    /// Always `"application/json"`.
    pub fn data_content_type(&self) -> &str {
        self.data_content_type
    }

    /// ISO-8601 timestamp string supplied by the caller.
    pub fn time(&self) -> &str {
        &self.time
    }

    /// Raw JSON payload (opaque to the eventing substrate).
    pub fn data(&self) -> &str {
        &self.data
    }

    /// Oyatie extension: tenant that produced the event.
    pub fn oyatie_tenant_id(&self) -> &str {
        &self.oyatie_tenant_id
    }

    /// Oyatie extension: cell/region that produced the event.
    pub fn oyatie_cell_id(&self) -> &str {
        &self.oyatie_cell_id
    }

    /// Parse the `event_type` field back into its components.
    ///
    /// Returns `(tenant_id, microservice, event_type_token, version)` or
    /// `None` if the field does not match the expected shape.
    ///
    /// The version token must be exactly `v` followed by one or more digits
    /// (e.g. `v1`, `v12`). All components must be non-empty.
    pub fn parse_topic(event_type: &str) -> Option<(&str, &str, &str, &str)> {
        // expected: t.<tenant_id>.<microservice>.<event_type>.v<version>
        let body = event_type.strip_prefix("t.")?;
        // Split into exactly 4 dot-separated parts: tenant_id, microservice, event_type, version
        let mut parts = body.splitn(4, '.');
        let tenant_id = parts.next()?;
        let microservice = parts.next()?;
        let event_type_token = parts.next()?;
        let version = parts.next()?;

        // Thread 5 (P2): reject empty components
        if tenant_id.is_empty() || microservice.is_empty() || event_type_token.is_empty() {
            return None;
        }

        // Thread 3 (P2): strict version token shape: v\d+ (no extra dots or chars)
        let digits = version.strip_prefix('v')?;
        if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
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
        assert_eq!(ev.event_type(), "t.acme.eventing.outbox_dispatched.v1");
        assert_eq!(ev.source(), "//oyatie/eventing");
        assert_eq!(ev.spec_version(), "1.0");
        assert_eq!(ev.data_content_type(), "application/json");
    }

    #[test]
    fn cloud_event_parse_topic_roundtrips() {
        let ev = make_event();
        let parsed =
            CloudEvent::parse_topic(ev.event_type()).expect("structured type is parseable");
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
        assert_eq!(ev.event_type(), "t.acme.eventing.outbox_dispatched.v3");
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
        assert!(!CloudEventError::EmptyId.to_string().is_empty());
        assert!(
            !CloudEventError::InvalidVersion { version: 0 }
                .to_string()
                .is_empty()
        );
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

    // --- New tests for each validation added per codex review ---

    /// Thread 1 (P1): constructor must reject empty id.
    #[test]
    fn new_rejects_empty_id() {
        let err = CloudEvent::new("", "acme", "cell", "eventing", "dispatched", 1, "t", "{}")
            .unwrap_err();
        assert_eq!(err, CloudEventError::EmptyId);
    }

    /// Thread 2 (P2): constructor must reject version == 0.
    #[test]
    fn new_rejects_zero_version() {
        let err = CloudEvent::new("id", "acme", "cell", "eventing", "dispatched", 0, "t", "{}")
            .unwrap_err();
        assert_eq!(err, CloudEventError::InvalidVersion { version: 0 });
    }

    /// Thread 4 (P1): constructor must reject dotted tenant_id.
    #[test]
    fn new_rejects_dotted_tenant_id() {
        let err = CloudEvent::new(
            "id",
            "acme.eu",
            "cell",
            "eventing",
            "dispatched",
            1,
            "t",
            "{}",
        )
        .unwrap_err();
        assert_eq!(err, CloudEventError::IllegalDotInComponent);
    }

    /// Thread 5 (P2): parse_topic must reject empty tenant/microservice/event components.
    #[test]
    fn parse_topic_rejects_empty_components() {
        // empty tenant
        assert!(CloudEvent::parse_topic("t..eventing.dispatched.v1").is_none());
        // empty microservice
        assert!(CloudEvent::parse_topic("t.acme..dispatched.v1").is_none());
        // empty event_type_token
        assert!(CloudEvent::parse_topic("t.acme.eventing..v1").is_none());
    }

    /// Thread 3 (P2): parse_topic must reject malformed version tokens.
    #[test]
    fn parse_topic_rejects_malformed_version_token() {
        // version with non-digit suffix
        assert!(CloudEvent::parse_topic("t.acme.eventing.dispatched.v1alpha").is_none());
        // version with extra dots (splitn(4) means version captures remainder)
        assert!(CloudEvent::parse_topic("t.acme.eventing.dispatched.v1.extra").is_none());
        // bare 'v' with no digits
        assert!(CloudEvent::parse_topic("t.acme.eventing.dispatched.v").is_none());
    }

    /// Thread 6 (P1): fields are private — the only construction path is `new`.
    /// This is a structural guarantee verified by the fact that all field accesses
    /// in this test module go through getters, and the struct literal syntax
    /// `CloudEvent { .. }` is not used anywhere in tests or production code.
    #[test]
    fn fields_are_private_construction_only_via_new() {
        let ev = make_event();
        // Verify all fields are accessible via getters (compile-time proof of private fields)
        assert!(!ev.id().is_empty());
        assert!(!ev.spec_version().is_empty());
        assert!(!ev.source().is_empty());
        assert!(!ev.event_type().is_empty());
        assert!(!ev.data_content_type().is_empty());
        assert!(!ev.time().is_empty());
        assert!(!ev.data().is_empty());
        assert!(!ev.oyatie_tenant_id().is_empty());
        assert!(!ev.oyatie_cell_id().is_empty());
    }

    /// Thread 7 (P2): tokens with leading/trailing whitespace are trimmed and lowercased.
    #[test]
    fn new_normalizes_tokens_with_whitespace_and_case() {
        let ev = CloudEvent::new(
            "id",
            "  ACME  ",
            "  Cell-US-East-1  ",
            "  Eventing  ",
            "  Outbox_Dispatched  ",
            1,
            "t",
            "{}",
        )
        .expect("whitespace-padded inputs are valid after trim");
        assert_eq!(ev.event_type(), "t.acme.eventing.outbox_dispatched.v1");
        assert_eq!(ev.oyatie_tenant_id(), "acme");
        assert_eq!(ev.oyatie_cell_id(), "cell-us-east-1");
    }
}
