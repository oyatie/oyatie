#![forbid(unsafe_code)]
//! Versioned Drive REST, gRPC, streaming, event, webhook, and SDK contract types.
//!
//! These contracts are pure Rust and dependency-light so backend, web, worker,
//! SDK, and benchmark lanes can develop in parallel against stable types.

use oya_office_authz_domain::{AclRole, DriveAction};
use oya_office_drive_domain::{DriveObjectBinding, DriveObjectKind, DriveObjectPublicView};
use oya_office_kernel::{DataClass, ObjectId, PrincipalId, RequestId, TenantId};

/// Stable crate identifier used by workspace and Buck2 scaffold verification.
pub const CRATE_NAME: &str = "oya-office-drive-api";

/// Product vertical slice owned by this crate.
pub const VERTICAL_SLICE: &str = "drive";

/// Canonical ADR-0056 architectural layer represented by this crate: the
/// protocol-neutral typed contract surface is the `api` layer.
pub const ARCHITECTURE_LAYER: &str = "api";

/// Versioned Drive API surface.
pub const DRIVE_API_VERSION: &str = "v1";

/// Versioned Drive event schema surface.
pub const DRIVE_EVENT_SCHEMA_VERSION: &str = "drive.events.v1";

/// Stable Drive route contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DriveRoute {
    /// `GET /api/drive/v1/objects`
    ListObjects,
    /// `POST /api/drive/v1/objects`
    CreateObject,
    /// `GET /api/drive/v1/objects/{objectId}`
    GetObject,
    /// `PATCH /api/drive/v1/objects/{objectId}`
    UpdateObject,
    /// `POST /api/drive/v1/objects/{objectId}:authorize`
    AuthorizeObject,
    /// `POST /api/drive/v1/objects/{objectId}:launch`
    LaunchObject,
    /// `POST /api/drive/v1/objects/{objectId}:trash`
    TrashObject,
    /// `POST /api/drive/v1/objects/{objectId}:beginUpload`
    BeginUpload,
    /// `GET /api/drive/v1/objects/{objectId}:download`
    DownloadObject,
    /// `POST /api/drive/v1/objects/{objectId}:share`
    ShareObject,
    /// `POST /api/drive/v1/objects/{objectId}:revoke`
    RevokeObjectAccess,
    /// `GET /api/drive/v1/objects/{objectId}/versions`
    ListObjectVersions,
    /// `POST /api/drive/v1/objects/{objectId}:preview`
    CreatePreview,
    /// `GET /api/drive/v1/search`
    SearchObjects,
}

impl DriveRoute {
    /// Returns the HTTP method and stable path template.
    #[must_use]
    pub const fn method_and_path(self) -> (&'static str, &'static str) {
        match self {
            Self::ListObjects => ("GET", "/api/drive/v1/objects"),
            Self::CreateObject => ("POST", "/api/drive/v1/objects"),
            Self::GetObject => ("GET", "/api/drive/v1/objects/{objectId}"),
            Self::UpdateObject => ("PATCH", "/api/drive/v1/objects/{objectId}"),
            Self::AuthorizeObject => ("POST", "/api/drive/v1/objects/{objectId}:authorize"),
            Self::LaunchObject => ("POST", "/api/drive/v1/objects/{objectId}:launch"),
            Self::TrashObject => ("POST", "/api/drive/v1/objects/{objectId}:trash"),
            Self::BeginUpload => ("POST", "/api/drive/v1/objects/{objectId}:beginUpload"),
            Self::DownloadObject => ("GET", "/api/drive/v1/objects/{objectId}:download"),
            Self::ShareObject => ("POST", "/api/drive/v1/objects/{objectId}:share"),
            Self::RevokeObjectAccess => ("POST", "/api/drive/v1/objects/{objectId}:revoke"),
            Self::ListObjectVersions => ("GET", "/api/drive/v1/objects/{objectId}/versions"),
            Self::CreatePreview => ("POST", "/api/drive/v1/objects/{objectId}:preview"),
            Self::SearchObjects => ("GET", "/api/drive/v1/search"),
        }
    }
}

/// Returns all currently declared Drive routes.
#[must_use]
pub const fn drive_routes() -> [DriveRoute; 14] {
    [
        DriveRoute::ListObjects,
        DriveRoute::CreateObject,
        DriveRoute::GetObject,
        DriveRoute::UpdateObject,
        DriveRoute::AuthorizeObject,
        DriveRoute::LaunchObject,
        DriveRoute::TrashObject,
        DriveRoute::BeginUpload,
        DriveRoute::DownloadObject,
        DriveRoute::ShareObject,
        DriveRoute::RevokeObjectAccess,
        DriveRoute::ListObjectVersions,
        DriveRoute::CreatePreview,
        DriveRoute::SearchObjects,
    ]
}

/// Bounded page limit for Drive list/search endpoints.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PageLimit(u16);

impl PageLimit {
    /// Maximum allowed page size.
    pub const MAX: u16 = 200;

    /// Creates a bounded page limit. Values outside range are clamped safely.
    #[must_use]
    pub const fn new(value: u16) -> Self {
        if value == 0 {
            Self(50)
        } else if value > Self::MAX {
            Self(Self::MAX)
        } else {
            Self(value)
        }
    }

    /// Returns the numeric limit.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

/// Request to create a Drive object shell before content upload/conversion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateDriveObjectRequest {
    tenant_id: TenantId,
    name: String,
    kind: DriveObjectKind,
    data_class: DataClass,
}

impl CreateDriveObjectRequest {
    /// Creates a Drive object creation request.
    pub fn new(
        tenant_id: TenantId,
        name: impl Into<String>,
        kind: DriveObjectKind,
        data_class: DataClass,
    ) -> Result<Self, DriveApiContractError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(DriveApiContractError::new(
                DriveApiErrorCode::ValidationFailed,
                "drive object name must not be empty",
            ));
        }
        Ok(Self {
            tenant_id,
            name,
            kind,
            data_class,
        })
    }

    /// Returns tenant id.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns object display name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns object kind.
    #[must_use]
    pub const fn kind(&self) -> DriveObjectKind {
        self.kind
    }

    /// Returns requested data class.
    #[must_use]
    pub const fn data_class(&self) -> DataClass {
        self.data_class
    }
}

/// Bounded Drive list request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveListObjectsRequest {
    tenant_id: TenantId,
    page_limit: PageLimit,
    page_cursor: Option<String>,
}

impl DriveListObjectsRequest {
    /// Creates a list request with bounded page size.
    #[must_use]
    pub fn new(tenant_id: TenantId, page_limit: PageLimit, page_cursor: Option<String>) -> Self {
        Self {
            tenant_id,
            page_limit,
            page_cursor,
        }
    }

    /// Returns tenant id.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns bounded page limit.
    #[must_use]
    pub const fn page_limit(&self) -> PageLimit {
        self.page_limit
    }

    /// Returns optional cursor.
    #[must_use]
    pub fn page_cursor(&self) -> Option<&str> {
        self.page_cursor.as_deref()
    }
}

/// Drive object response safe for API clients.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveObjectResponse {
    object: DriveObjectPublicView,
}

impl DriveObjectResponse {
    /// Creates a response from a redacted public view.
    #[must_use]
    pub const fn new(object: DriveObjectPublicView) -> Self {
        Self { object }
    }

    /// Returns the redacted public object view.
    #[must_use]
    pub const fn object(&self) -> &DriveObjectPublicView {
        &self.object
    }
}

/// Drive list response.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveListObjectsResponse {
    objects: Vec<DriveObjectResponse>,
    next_cursor: Option<String>,
}

/// Upload/download content operation kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DriveContentOperation {
    /// Begin upload for object content.
    Upload,
    /// Download object content.
    Download,
}

impl DriveContentOperation {
    /// Returns the route for this content operation.
    #[must_use]
    pub const fn route(self) -> DriveRoute {
        match self {
            Self::Upload => DriveRoute::BeginUpload,
            Self::Download => DriveRoute::DownloadObject,
        }
    }
}

/// Content intent for upload/download API lanes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveObjectContentIntent {
    binding: DriveObjectBinding,
    operation: DriveContentOperation,
}

impl DriveObjectContentIntent {
    /// Creates content intent from a Drive binding.
    #[must_use]
    pub const fn new(binding: DriveObjectBinding, operation: DriveContentOperation) -> Self {
        Self { binding, operation }
    }

    /// Returns binding.
    #[must_use]
    pub const fn binding(&self) -> &DriveObjectBinding {
        &self.binding
    }

    /// Returns operation.
    #[must_use]
    pub const fn operation(&self) -> DriveContentOperation {
        self.operation
    }

    /// Returns route.
    #[must_use]
    pub const fn route(&self) -> DriveRoute {
        self.operation.route()
    }
}

/// Share request contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShareDriveObjectRequest {
    object_id: ObjectId,
    principal_id: PrincipalId,
    role: AclRole,
}

impl ShareDriveObjectRequest {
    /// Creates a share request. Owner transfers are not share grants.
    pub fn new(
        object_id: ObjectId,
        principal_id: PrincipalId,
        role: AclRole,
    ) -> Result<Self, DriveApiContractError> {
        if role == AclRole::Owner {
            return Err(DriveApiContractError::new(
                DriveApiErrorCode::ValidationFailed,
                "drive share request cannot grant owner role",
            ));
        }
        Ok(Self {
            object_id,
            principal_id,
            role,
        })
    }

    /// Returns object id.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    /// Returns principal id.
    #[must_use]
    pub const fn principal_id(&self) -> &PrincipalId {
        &self.principal_id
    }

    /// Returns role.
    #[must_use]
    pub const fn role(&self) -> AclRole {
        self.role
    }
}

/// Share revocation request contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RevokeDriveObjectAccessRequest {
    object_id: ObjectId,
    principal_id: PrincipalId,
}

impl RevokeDriveObjectAccessRequest {
    /// Creates a revocation request.
    #[must_use]
    pub const fn new(object_id: ObjectId, principal_id: PrincipalId) -> Self {
        Self {
            object_id,
            principal_id,
        }
    }

    /// Returns object id.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    /// Returns principal id.
    #[must_use]
    pub const fn principal_id(&self) -> &PrincipalId {
        &self.principal_id
    }
}

/// Drive search request contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveSearchObjectsRequest {
    tenant_id: TenantId,
    query: String,
    page_limit: PageLimit,
}

impl DriveSearchObjectsRequest {
    /// Creates a bounded Drive search request.
    pub fn new(
        tenant_id: TenantId,
        query: impl Into<String>,
        page_limit: PageLimit,
    ) -> Result<Self, DriveApiContractError> {
        let query = query.into();
        if query.trim().is_empty() {
            return Err(DriveApiContractError::new(
                DriveApiErrorCode::ValidationFailed,
                "drive search query must not be empty",
            ));
        }
        Ok(Self {
            tenant_id,
            query: query.trim().to_owned(),
            page_limit,
        })
    }

    /// Returns tenant id.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns query.
    #[must_use]
    pub fn query(&self) -> &str {
        self.query.as_str()
    }

    /// Returns page limit.
    #[must_use]
    pub const fn page_limit(&self) -> PageLimit {
        self.page_limit
    }
}

impl DriveListObjectsResponse {
    /// Creates a list response.
    #[must_use]
    pub fn new(objects: Vec<DriveObjectResponse>, next_cursor: Option<String>) -> Self {
        Self {
            objects,
            next_cursor,
        }
    }

    /// Returns objects.
    #[must_use]
    pub fn objects(&self) -> &[DriveObjectResponse] {
        self.objects.as_slice()
    }

    /// Returns optional next cursor.
    #[must_use]
    pub fn next_cursor(&self) -> Option<&str> {
        self.next_cursor.as_deref()
    }
}

/// Launch target for opening a Drive object in the suite.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DriveLaunchTarget {
    /// Oya Docs editor.
    Docs,
    /// Oya Sheets editor.
    Sheets,
    /// Oya Slides editor.
    Slides,
    /// File preview surface.
    Preview,
}

impl DriveLaunchTarget {
    /// Selects a launch target from object kind.
    #[must_use]
    pub const fn from_object_kind(kind: DriveObjectKind) -> Self {
        match kind {
            DriveObjectKind::Document => Self::Docs,
            DriveObjectKind::Spreadsheet => Self::Sheets,
            DriveObjectKind::Presentation => Self::Slides,
            DriveObjectKind::Folder | DriveObjectKind::Binary => Self::Preview,
        }
    }
}

/// Request to launch a Drive object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveLaunchRequest {
    binding: DriveObjectBinding,
    requested_action: DriveAction,
}

impl DriveLaunchRequest {
    /// Creates a launch request.
    #[must_use]
    pub const fn new(binding: DriveObjectBinding, requested_action: DriveAction) -> Self {
        Self {
            binding,
            requested_action,
        }
    }

    /// Returns the Drive binding.
    #[must_use]
    pub const fn binding(&self) -> &DriveObjectBinding {
        &self.binding
    }

    /// Returns the requested action.
    #[must_use]
    pub const fn requested_action(&self) -> DriveAction {
        self.requested_action
    }

    /// Returns the target editor/surface.
    #[must_use]
    pub const fn target(&self) -> DriveLaunchTarget {
        DriveLaunchTarget::from_object_kind(self.binding.kind())
    }
}

/// API error code contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DriveApiErrorCode {
    /// Request failed validation.
    ValidationFailed,
    /// Caller is authenticated but unauthorized.
    Forbidden,
    /// Object was not found or not visible to caller.
    NotFound,
    /// Write/update conflict.
    Conflict,
    /// Internal server error; never exposes internals.
    Internal,
}

/// API error envelope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveApiContractError {
    code: DriveApiErrorCode,
    message: String,
}

impl DriveApiContractError {
    /// Creates an API error envelope.
    #[must_use]
    pub fn new(code: DriveApiErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Returns machine-readable code.
    #[must_use]
    pub const fn code(&self) -> DriveApiErrorCode {
        self.code
    }

    /// Returns user-safe message.
    #[must_use]
    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

impl core::fmt::Display for DriveApiContractError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for DriveApiContractError {}

/// Drive event name for future worker/webhook streams.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DriveEventKind {
    /// Object created.
    ObjectCreated,
    /// Object metadata updated.
    ObjectUpdated,
    /// Object ACL changed.
    AclChanged,
    /// Object lifecycle changed.
    LifecycleChanged,
    /// Object version pointer created.
    VersionCreated,
    /// Object moved to trash.
    ObjectTrashed,
    /// Tenant quota evaluated for a Drive object mutation.
    QuotaEvaluated,
    /// Object indexed or re-indexed for search.
    SearchIndexed,
}

impl DriveEventKind {
    /// Returns the stable event name emitted by future streams/webhooks.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ObjectCreated => "drive.object.created",
            Self::ObjectUpdated => "drive.object.updated",
            Self::AclChanged => "drive.acl.changed",
            Self::LifecycleChanged => "drive.lifecycle.changed",
            Self::VersionCreated => "drive.version.created",
            Self::ObjectTrashed => "drive.object.trashed",
            Self::QuotaEvaluated => "drive.quota.evaluated",
            Self::SearchIndexed => "drive.search.indexed",
        }
    }
}

/// Returns all Drive event kinds covered by the G080 event contract.
#[must_use]
pub const fn drive_event_kinds() -> [DriveEventKind; 8] {
    [
        DriveEventKind::ObjectCreated,
        DriveEventKind::ObjectUpdated,
        DriveEventKind::AclChanged,
        DriveEventKind::LifecycleChanged,
        DriveEventKind::VersionCreated,
        DriveEventKind::ObjectTrashed,
        DriveEventKind::QuotaEvaluated,
        DriveEventKind::SearchIndexed,
    ]
}

/// Tenant/object-scoped Drive event envelope for streams, workers, and webhooks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveEventEnvelope {
    schema_version: &'static str,
    event_id: RequestId,
    tenant_id: TenantId,
    object_id: ObjectId,
    kind: DriveEventKind,
    data_class: DataClass,
    sequence_number: u64,
}

impl DriveEventEnvelope {
    /// Creates a sequenced Drive event envelope.
    pub fn new(
        event_id: RequestId,
        tenant_id: TenantId,
        object_id: ObjectId,
        kind: DriveEventKind,
        data_class: DataClass,
        sequence_number: u64,
    ) -> Result<Self, DriveApiContractError> {
        if sequence_number == 0 {
            return Err(DriveApiContractError::new(
                DriveApiErrorCode::ValidationFailed,
                "drive event sequence number must be greater than zero",
            ));
        }
        Ok(Self {
            schema_version: DRIVE_EVENT_SCHEMA_VERSION,
            event_id,
            tenant_id,
            object_id,
            kind,
            data_class,
            sequence_number,
        })
    }

    /// Returns the event schema version.
    #[must_use]
    pub const fn schema_version(&self) -> &'static str {
        self.schema_version
    }

    /// Returns the event identifier.
    #[must_use]
    pub const fn event_id(&self) -> &RequestId {
        &self.event_id
    }

    /// Returns the owning tenant.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns the Drive object id.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    /// Returns the event kind.
    #[must_use]
    pub const fn kind(&self) -> DriveEventKind {
        self.kind
    }

    /// Returns the object's data classification at event time.
    #[must_use]
    pub const fn data_class(&self) -> DataClass {
        self.data_class
    }

    /// Returns the monotonically increasing object-scoped sequence number.
    #[must_use]
    pub const fn sequence_number(&self) -> u64 {
        self.sequence_number
    }
}

#[cfg(test)]
mod tests {
    use oya_office_authz_domain::DriveAction;
    use oya_office_drive_domain::{DriveObjectBinding, DriveObjectKind};
    use oya_office_kernel::{DataClass, ObjectId, RequestId, TenantId};

    use super::{
        ARCHITECTURE_LAYER, CRATE_NAME, CreateDriveObjectRequest, DRIVE_API_VERSION,
        DRIVE_EVENT_SCHEMA_VERSION, DriveContentOperation, DriveEventEnvelope, DriveEventKind,
        DriveLaunchRequest, DriveLaunchTarget, DriveObjectContentIntent, DriveRoute, PageLimit,
        ShareDriveObjectRequest, VERTICAL_SLICE, drive_event_kinds, drive_routes,
    };

    #[test]
    fn scaffold_identity_is_declared() {
        assert!(!ARCHITECTURE_LAYER.is_empty());
        assert!(!CRATE_NAME.is_empty());
        assert!(!VERTICAL_SLICE.is_empty());
        assert_eq!(DRIVE_API_VERSION, "v1");
    }

    #[test]
    fn drive_routes_include_launch_contract() {
        let routes = drive_routes();
        assert!(routes.contains(&DriveRoute::LaunchObject));
        assert!(routes.contains(&DriveRoute::ShareObject));
        assert!(routes.contains(&DriveRoute::DownloadObject));
        assert_eq!(DriveRoute::ListObjects.method_and_path().0, "GET");
    }

    #[test]
    fn page_limit_is_bounded() {
        assert_eq!(PageLimit::new(0).get(), 50);
        assert_eq!(PageLimit::new(999).get(), PageLimit::MAX);
    }

    #[test]
    fn launch_target_follows_drive_object_kind() {
        let binding = DriveObjectBinding::new(
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            ObjectId::new("sheet-1").expect("valid object id"),
            DriveObjectKind::Spreadsheet,
            DataClass::Confidential,
        );
        let request = DriveLaunchRequest::new(binding, DriveAction::Read);
        assert_eq!(request.target(), DriveLaunchTarget::Sheets);
    }

    #[test]
    fn create_request_rejects_blank_name() {
        let result = CreateDriveObjectRequest::new(
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            "   ",
            DriveObjectKind::Document,
            DataClass::Internal,
        );
        assert!(result.is_err());
    }

    #[test]
    fn content_and_share_contracts_expose_expected_routes() {
        let binding = DriveObjectBinding::new(
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            ObjectId::new("doc-1").expect("valid object id"),
            DriveObjectKind::Document,
            DataClass::Confidential,
        );
        let intent = DriveObjectContentIntent::new(binding, DriveContentOperation::Upload);
        assert_eq!(intent.route(), DriveRoute::BeginUpload);

        let share = ShareDriveObjectRequest::new(
            ObjectId::new("doc-1").expect("valid object id"),
            oya_office_kernel::PrincipalId::new("viewer-1").expect("valid principal"),
            oya_office_authz_domain::AclRole::Viewer,
        )
        .expect("valid share");
        assert_eq!(share.role(), oya_office_authz_domain::AclRole::Viewer);
    }

    #[test]
    fn g080_drive_event_envelope_is_tenant_object_scoped_and_covers_required_events() {
        let event = DriveEventEnvelope::new(
            RequestId::new("evt-drive-1").expect("valid event id"),
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            ObjectId::new("doc-1").expect("valid object id"),
            DriveEventKind::AclChanged,
            DataClass::Confidential,
            7,
        )
        .expect("valid event");
        let event_kinds = drive_event_kinds();

        assert_eq!(event.schema_version(), DRIVE_EVENT_SCHEMA_VERSION);
        assert_eq!(event.event_id().as_str(), "evt-drive-1");
        assert_eq!(event.tenant_id().as_str(), "tenant-alpha");
        assert_eq!(event.object_id().as_str(), "doc-1");
        assert_eq!(event.kind().as_str(), "drive.acl.changed");
        assert_eq!(event.data_class(), DataClass::Confidential);
        assert_eq!(event.sequence_number(), 7);
        assert_eq!(event_kinds.len(), 8);
        assert!(event_kinds.contains(&DriveEventKind::ObjectCreated));
        assert!(event_kinds.contains(&DriveEventKind::AclChanged));
        assert!(event_kinds.contains(&DriveEventKind::LifecycleChanged));
        assert!(event_kinds.contains(&DriveEventKind::VersionCreated));
        assert!(event_kinds.contains(&DriveEventKind::ObjectTrashed));
        assert!(event_kinds.contains(&DriveEventKind::QuotaEvaluated));
        assert!(event_kinds.contains(&DriveEventKind::SearchIndexed));
        assert!(
            DriveEventEnvelope::new(
                RequestId::new("evt-drive-2").expect("valid event id"),
                TenantId::new("tenant-alpha").expect("valid tenant id"),
                ObjectId::new("doc-1").expect("valid object id"),
                DriveEventKind::ObjectUpdated,
                DataClass::Internal,
                0,
            )
            .is_err()
        );
    }
}
