#![forbid(unsafe_code)]
//! Suite-wide REST, gRPC, streaming, error envelope, tenant header, and SDK contract types.
//!
//! This crate keeps suite API contracts provider-neutral. It may depend on
//! Oya Office domain crates for typed identifiers/value objects.

use oya_office_kernel::{PrincipalId, TenantId};
use oya_office_sheet_domain::{CellAddress, CellValue, FormulaExpression, WorkbookCell, WorkbookId};

/// Stable crate identifier used by workspace and Buck2 scaffold verification.
pub const CRATE_NAME: &str = "oya-office-sheets-api";

/// Product vertical slice owned by this crate.
pub const VERTICAL_SLICE: &str = "platform";

/// Canonical ADR-0056 architectural layer represented by this crate: the
/// protocol-neutral typed contract surface is the `api` layer.
pub const ARCHITECTURE_LAYER: &str = "api";

/// Versioned Sheets API surface.
pub const SHEETS_API_VERSION: &str = "v1";

/// Stable Sheets API route contract spanning REST, gRPC-like batch calls, streams, and webhooks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SheetsApiRoute {
    /// `GET /api/sheets/v1/workbooks/{workbookId}`
    GetWorkbook,
    /// `POST /api/sheets/v1/workbooks/{workbookId}:batchUpdate`
    BatchUpdate,
    /// `POST /api/sheets/v1/workbooks/{workbookId}:grpcBatchUpdate`
    GrpcBatchUpdate,
    /// `GET /api/sheets/v1/workbooks/{workbookId}/ranges/{range}`
    ReadRange,
    /// `PUT /api/sheets/v1/workbooks/{workbookId}/ranges/{range}`
    WriteRange,
    /// `POST /api/sheets/v1/workbooks/{workbookId}:calculateFormula`
    CalculateFormula,
    /// `GET /api/sheets/v1/workbooks/{workbookId}:stream`
    OpenChangeStream,
    /// `POST /api/sheets/v1/workbooks/{workbookId}/webhooks`
    RegisterWebhook,
    /// `POST /api/sheets/v1/workbooks/{workbookId}:createEmbedSession`
    CreateEmbedSession,
}

impl SheetsApiRoute {
    /// Returns the HTTP method and stable path template.
    #[must_use]
    pub const fn method_and_path(self) -> (&'static str, &'static str) {
        match self {
            Self::GetWorkbook => ("GET", "/api/sheets/v1/workbooks/{workbookId}"),
            Self::BatchUpdate => ("POST", "/api/sheets/v1/workbooks/{workbookId}:batchUpdate"),
            Self::GrpcBatchUpdate => (
                "POST",
                "/api/sheets/v1/workbooks/{workbookId}:grpcBatchUpdate",
            ),
            Self::ReadRange => (
                "GET",
                "/api/sheets/v1/workbooks/{workbookId}/ranges/{range}",
            ),
            Self::WriteRange => (
                "PUT",
                "/api/sheets/v1/workbooks/{workbookId}/ranges/{range}",
            ),
            Self::CalculateFormula => (
                "POST",
                "/api/sheets/v1/workbooks/{workbookId}:calculateFormula",
            ),
            Self::OpenChangeStream => ("GET", "/api/sheets/v1/workbooks/{workbookId}:stream"),
            Self::RegisterWebhook => ("POST", "/api/sheets/v1/workbooks/{workbookId}/webhooks"),
            Self::CreateEmbedSession => (
                "POST",
                "/api/sheets/v1/workbooks/{workbookId}:createEmbedSession",
            ),
        }
    }

    /// Returns true for streaming routes.
    #[must_use]
    pub const fn is_streaming(self) -> bool {
        matches!(self, Self::OpenChangeStream)
    }
}

/// Returns all currently declared Sheets routes.
#[must_use]
pub const fn sheets_routes() -> [SheetsApiRoute; 9] {
    [
        SheetsApiRoute::GetWorkbook,
        SheetsApiRoute::BatchUpdate,
        SheetsApiRoute::GrpcBatchUpdate,
        SheetsApiRoute::ReadRange,
        SheetsApiRoute::WriteRange,
        SheetsApiRoute::CalculateFormula,
        SheetsApiRoute::OpenChangeStream,
        SheetsApiRoute::RegisterWebhook,
        SheetsApiRoute::CreateEmbedSession,
    ]
}

/// Suite API contract validation error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiContractError {
    message: String,
}

impl ApiContractError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns error message.
    #[must_use]
    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

impl core::fmt::Display for ApiContractError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.message())
    }
}

impl std::error::Error for ApiContractError {}

/// Bounded workbook range reference for Sheets REST/SDK calls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetsRangeRef {
    workbook_id: WorkbookId,
    sheet_id: String,
    start_row: u32,
    start_column: u32,
    end_row: u32,
    end_column: u32,
    cell_count: u32,
}

impl SheetsRangeRef {
    /// Maximum cell count accepted by a single range contract.
    pub const MAX_CELLS: u32 = 100_000;

    /// Creates a bounded 1-based inclusive range reference.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        workbook_id: WorkbookId,
        sheet_id: impl Into<String>,
        start_row: u32,
        start_column: u32,
        end_row: u32,
        end_column: u32,
    ) -> Result<Self, ApiContractError> {
        if start_row == 0 || start_column == 0 || end_row == 0 || end_column == 0 {
            return Err(ApiContractError::new("sheets range bounds must be 1-based"));
        }
        if start_row > end_row || start_column > end_column {
            return Err(ApiContractError::new(
                "sheets range start must not exceed end",
            ));
        }
        let row_count = end_row - start_row + 1;
        let column_count = end_column - start_column + 1;
        let cell_count = row_count.saturating_mul(column_count);
        if cell_count > Self::MAX_CELLS {
            return Err(ApiContractError::new(
                "sheets range exceeds maximum bounded cell count",
            ));
        }
        Ok(Self {
            workbook_id,
            sheet_id: validate_required_text("sheets range worksheet id", sheet_id)?,
            start_row,
            start_column,
            end_row,
            end_column,
            cell_count,
        })
    }

    /// Returns workbook id.
    #[must_use]
    pub const fn workbook_id(&self) -> &WorkbookId {
        &self.workbook_id
    }

    /// Returns worksheet id.
    #[must_use]
    pub fn sheet_id(&self) -> &str {
        self.sheet_id.as_str()
    }

    /// Returns cell count.
    #[must_use]
    pub const fn cell_count(&self) -> u32 {
        self.cell_count
    }

    /// Returns true because the constructor rejects unbounded ranges.
    #[must_use]
    pub const fn is_bounded(&self) -> bool {
        self.cell_count <= Self::MAX_CELLS
    }

    /// Returns inclusive bounds.
    #[must_use]
    pub const fn bounds(&self) -> (u32, u32, u32, u32) {
        (
            self.start_row,
            self.start_column,
            self.end_row,
            self.end_column,
        )
    }
}

/// One batch update operation for Sheets SDK/API callers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SheetsBatchOperation {
    /// Set a literal cell value.
    SetValue {
        /// Target cell address.
        address: CellAddress,
        /// Literal cell value.
        value: CellValue,
    },
    /// Set a formula expression.
    SetFormula {
        /// Target cell address.
        address: CellAddress,
        /// Formula expression.
        formula: FormulaExpression,
    },
    /// Clear a bounded range.
    ClearRange {
        /// Target range.
        range: SheetsRangeRef,
    },
}

impl SheetsBatchOperation {
    /// Creates a set-value batch operation and validates the value through the domain cell.
    pub fn set_value(address: CellAddress, value: CellValue) -> Result<Self, ApiContractError> {
        WorkbookCell::new(address.clone(), value.clone(), None, 1)
            .map_err(|error| ApiContractError::new(error.to_string()))?;
        Ok(Self::SetValue { address, value })
    }

    /// Creates a set-formula batch operation.
    #[must_use]
    pub const fn set_formula(address: CellAddress, formula: FormulaExpression) -> Self {
        Self::SetFormula { address, formula }
    }

    /// Creates a clear-range operation.
    #[must_use]
    pub const fn clear_range(range: SheetsRangeRef) -> Self {
        Self::ClearRange { range }
    }
}

/// Batch update request for REST/gRPC/SDK callers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetsBatchUpdateRequest {
    tenant_id: TenantId,
    workbook_id: WorkbookId,
    actor_id: PrincipalId,
    operations: Vec<SheetsBatchOperation>,
    expected_version_sequence: u64,
}

impl SheetsBatchUpdateRequest {
    /// Maximum operations per request before callers must chunk.
    pub const MAX_OPERATIONS: usize = 1_000;

    /// Creates a bounded batch update request.
    pub fn new(
        tenant_id: TenantId,
        workbook_id: WorkbookId,
        actor_id: PrincipalId,
        operations: Vec<SheetsBatchOperation>,
        expected_version_sequence: u64,
    ) -> Result<Self, ApiContractError> {
        if operations.is_empty() {
            return Err(ApiContractError::new(
                "sheets batch update requires at least one operation",
            ));
        }
        if operations.len() > Self::MAX_OPERATIONS {
            return Err(ApiContractError::new(
                "sheets batch update exceeds maximum operation count",
            ));
        }
        if expected_version_sequence == 0 {
            return Err(ApiContractError::new(
                "sheets batch update expected version must be at least 1",
            ));
        }
        Ok(Self {
            tenant_id,
            workbook_id,
            actor_id,
            operations,
            expected_version_sequence,
        })
    }

    /// Returns tenant id.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns workbook id.
    #[must_use]
    pub const fn workbook_id(&self) -> &WorkbookId {
        &self.workbook_id
    }

    /// Returns actor id.
    #[must_use]
    pub const fn actor_id(&self) -> &PrincipalId {
        &self.actor_id
    }

    /// Returns operations.
    #[must_use]
    pub fn operations(&self) -> &[SheetsBatchOperation] {
        self.operations.as_slice()
    }

    /// Returns operation count.
    #[must_use]
    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }

    /// Returns expected workbook version sequence.
    #[must_use]
    pub const fn expected_version_sequence(&self) -> u64 {
        self.expected_version_sequence
    }
}

/// Bounded range read request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetsRangeReadRequest {
    tenant_id: TenantId,
    range: SheetsRangeRef,
}

impl SheetsRangeReadRequest {
    /// Creates a range read request.
    pub fn new(tenant_id: TenantId, range: SheetsRangeRef) -> Result<Self, ApiContractError> {
        if !range.is_bounded() {
            return Err(ApiContractError::new("sheets range read must be bounded"));
        }
        Ok(Self { tenant_id, range })
    }

    /// Returns tenant id.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns range.
    #[must_use]
    pub const fn range(&self) -> &SheetsRangeRef {
        &self.range
    }
}

/// Formula automation request for SDK/API callers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetsFormulaAutomationRequest {
    tenant_id: TenantId,
    workbook_id: WorkbookId,
    actor_id: PrincipalId,
    target: CellAddress,
    formula: FormulaExpression,
    expected_version_sequence: u64,
}

impl SheetsFormulaAutomationRequest {
    /// Creates a formula automation request.
    pub fn new(
        tenant_id: TenantId,
        workbook_id: WorkbookId,
        actor_id: PrincipalId,
        target: CellAddress,
        formula: FormulaExpression,
        expected_version_sequence: u64,
    ) -> Result<Self, ApiContractError> {
        if expected_version_sequence == 0 {
            return Err(ApiContractError::new(
                "sheets formula automation expected version must be at least 1",
            ));
        }
        Ok(Self {
            tenant_id,
            workbook_id,
            actor_id,
            target,
            formula,
            expected_version_sequence,
        })
    }

    /// Returns route.
    #[must_use]
    pub const fn route(&self) -> SheetsApiRoute {
        SheetsApiRoute::CalculateFormula
    }

    /// Returns tenant id.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns workbook id.
    #[must_use]
    pub const fn workbook_id(&self) -> &WorkbookId {
        &self.workbook_id
    }

    /// Returns actor id.
    #[must_use]
    pub const fn actor_id(&self) -> &PrincipalId {
        &self.actor_id
    }

    /// Returns expected workbook version sequence.
    #[must_use]
    pub const fn expected_version_sequence(&self) -> u64 {
        self.expected_version_sequence
    }

    /// Returns target address.
    #[must_use]
    pub const fn target(&self) -> &CellAddress {
        &self.target
    }

    /// Returns formula.
    #[must_use]
    pub const fn formula(&self) -> &FormulaExpression {
        &self.formula
    }
}

/// Sheets webhook event type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SheetsWebhookEventKind {
    /// A range was updated.
    RangeUpdated,
    /// A formula recalculation finished.
    FormulaRecalculated,
    /// A protected range was changed.
    ProtectedRangeChanged,
    /// A workbook version was committed.
    WorkbookVersionCommitted,
}

/// Sheets webhook subscription request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetsWebhookSubscriptionRequest {
    tenant_id: TenantId,
    workbook_id: WorkbookId,
    callback_url: String,
    event_kinds: Vec<SheetsWebhookEventKind>,
    webhook_secret_ref: String,
}

impl SheetsWebhookSubscriptionRequest {
    /// Creates a webhook subscription request.
    pub fn new(
        tenant_id: TenantId,
        workbook_id: WorkbookId,
        callback_url: impl Into<String>,
        event_kinds: Vec<SheetsWebhookEventKind>,
        webhook_secret_ref: impl Into<String>,
    ) -> Result<Self, ApiContractError> {
        let callback_url = validate_required_text("sheets webhook callback url", callback_url)?;
        if !callback_url.starts_with("https://") {
            return Err(ApiContractError::new(
                "sheets webhook callback url must use https",
            ));
        }
        if event_kinds.is_empty() {
            return Err(ApiContractError::new(
                "sheets webhook requires at least one event kind",
            ));
        }
        Ok(Self {
            tenant_id,
            workbook_id,
            callback_url,
            event_kinds,
            webhook_secret_ref: validate_required_text(
                "sheets webhook secret reference",
                webhook_secret_ref,
            )?,
        })
    }

    /// Returns route.
    #[must_use]
    pub const fn route(&self) -> SheetsApiRoute {
        SheetsApiRoute::RegisterWebhook
    }

    /// Returns tenant id.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns workbook id.
    #[must_use]
    pub const fn workbook_id(&self) -> &WorkbookId {
        &self.workbook_id
    }

    /// Returns event kinds.
    #[must_use]
    pub fn event_kinds(&self) -> &[SheetsWebhookEventKind] {
        self.event_kinds.as_slice()
    }

    /// Returns callback URL.
    #[must_use]
    pub fn callback_url(&self) -> &str {
        self.callback_url.as_str()
    }

    /// Returns webhook secret reference.
    #[must_use]
    pub fn webhook_secret_ref(&self) -> &str {
        self.webhook_secret_ref.as_str()
    }
}

/// Sheets SDK/platform capability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SheetsSdkCapability {
    /// Bounded batch update.
    BatchUpdate,
    /// Range read/write.
    RangeReadWrite,
    /// Formula automation.
    FormulaAutomation,
    /// Streaming workbook/range changes.
    Streaming,
    /// Webhook subscriptions.
    Webhooks,
    /// Integration connector ports.
    ConnectorPorts,
    /// Secure embedding/session-token seam.
    Embedding,
}

/// Required integration connector ports for Sheets platform parity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SheetsConnectorPort {
    /// Drive object binding, ACL, lifecycle, version, and export.
    Drive,
    /// Tenant isolation, authz, policy, and data class controls.
    TenantSecurity,
    /// Audit event emission.
    Audit,
    /// Quota/rate-limit accounting.
    Quota,
    /// Search/index updates.
    Search,
    /// Workflow/automation integration.
    Workflow,
    /// Realtime collaboration gateway.
    Collaboration,
    /// XLSX import/export/roundtrip workers.
    Format,
    /// Secure embed/session-token issuer and Leptos island bootstrap seam.
    Embedding,
}

/// Returns required connector ports for production Sheets platform parity.
#[must_use]
pub const fn sheets_required_connector_ports() -> [SheetsConnectorPort; 9] {
    [
        SheetsConnectorPort::Drive,
        SheetsConnectorPort::TenantSecurity,
        SheetsConnectorPort::Audit,
        SheetsConnectorPort::Quota,
        SheetsConnectorPort::Search,
        SheetsConnectorPort::Workflow,
        SheetsConnectorPort::Collaboration,
        SheetsConnectorPort::Format,
        SheetsConnectorPort::Embedding,
    ]
}

/// Secure embed mode for Sheets surfaces.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SheetsEmbedMode {
    /// View-only embedded workbook.
    View,
    /// Comment-capable embedded workbook.
    Comment,
    /// Edit-capable embedded workbook after authz and protected-range checks.
    Edit,
}

/// Bounded request to mint an embedded Sheets session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetsEmbedSessionRequest {
    tenant_id: TenantId,
    workbook_id: WorkbookId,
    actor_id: PrincipalId,
    allowed_origin: String,
    mode: SheetsEmbedMode,
    ttl_seconds: u32,
}

impl SheetsEmbedSessionRequest {
    /// Maximum embed session lifetime before re-authz.
    pub const MAX_TTL_SECONDS: u32 = 3_600;

    /// Creates an embed session request for a validated tenant actor and origin.
    pub fn new(
        tenant_id: TenantId,
        workbook_id: WorkbookId,
        actor_id: PrincipalId,
        allowed_origin: impl Into<String>,
        mode: SheetsEmbedMode,
        ttl_seconds: u32,
    ) -> Result<Self, ApiContractError> {
        let allowed_origin = validate_required_text("sheets embed allowed origin", allowed_origin)?;
        if !allowed_origin.starts_with("https://") {
            return Err(ApiContractError::new(
                "sheets embed allowed origin must use https",
            ));
        }
        if ttl_seconds == 0 || ttl_seconds > Self::MAX_TTL_SECONDS {
            return Err(ApiContractError::new(
                "sheets embed ttl must be between 1 and 3600 seconds",
            ));
        }
        Ok(Self {
            tenant_id,
            workbook_id,
            actor_id,
            allowed_origin,
            mode,
            ttl_seconds,
        })
    }

    /// Returns route.
    #[must_use]
    pub const fn route(&self) -> SheetsApiRoute {
        SheetsApiRoute::CreateEmbedSession
    }

    /// Returns tenant id.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns workbook id.
    #[must_use]
    pub const fn workbook_id(&self) -> &WorkbookId {
        &self.workbook_id
    }

    /// Returns actor id.
    #[must_use]
    pub const fn actor_id(&self) -> &PrincipalId {
        &self.actor_id
    }

    /// Returns allowed browser origin.
    #[must_use]
    pub fn allowed_origin(&self) -> &str {
        self.allowed_origin.as_str()
    }

    /// Returns embed mode.
    #[must_use]
    pub const fn mode(&self) -> SheetsEmbedMode {
        self.mode
    }

    /// Returns session TTL in seconds.
    #[must_use]
    pub const fn ttl_seconds(&self) -> u32 {
        self.ttl_seconds
    }
}

/// Rust SDK surface declaration for Sheets platform parity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetsSdkSurface {
    capabilities: Vec<SheetsSdkCapability>,
    routes: Vec<SheetsApiRoute>,
}

impl SheetsSdkSurface {
    /// Returns the default complete Sheets platform capability surface.
    #[must_use]
    pub fn platform_default() -> Self {
        Self {
            capabilities: vec![
                SheetsSdkCapability::BatchUpdate,
                SheetsSdkCapability::RangeReadWrite,
                SheetsSdkCapability::FormulaAutomation,
                SheetsSdkCapability::Streaming,
                SheetsSdkCapability::Webhooks,
                SheetsSdkCapability::ConnectorPorts,
                SheetsSdkCapability::Embedding,
            ],
            routes: sheets_routes().to_vec(),
        }
    }

    /// Returns true when the SDK declares a capability.
    #[must_use]
    pub fn supports(&self, capability: SheetsSdkCapability) -> bool {
        self.capabilities.contains(&capability)
    }

    /// Returns routes.
    #[must_use]
    pub fn routes(&self) -> &[SheetsApiRoute] {
        self.routes.as_slice()
    }
}

fn validate_required_text(
    field: &'static str,
    value: impl Into<String>,
) -> Result<String, ApiContractError> {
    let value = value.into();
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ApiContractError::new(format!("{field} must not be empty")));
    }
    Ok(trimmed.to_owned())
}

#[cfg(test)]
mod tests {
    use oya_office_kernel::{PrincipalId, TenantId};
    use oya_office_sheet_domain::{CellAddress, CellValue, FormulaExpression, WorkbookId};

    use super::{
        ARCHITECTURE_LAYER, CRATE_NAME, SheetsApiRoute, SheetsBatchOperation,
        SheetsBatchUpdateRequest, SheetsConnectorPort, SheetsEmbedMode, SheetsEmbedSessionRequest,
        SheetsFormulaAutomationRequest, SheetsRangeReadRequest, SheetsRangeRef,
        SheetsSdkCapability, SheetsSdkSurface, SheetsWebhookEventKind,
        SheetsWebhookSubscriptionRequest, VERTICAL_SLICE, sheets_required_connector_ports,
        sheets_routes,
    };

    #[test]
    fn scaffold_identity_is_declared() {
        assert!(!ARCHITECTURE_LAYER.is_empty());
        assert!(!CRATE_NAME.is_empty());
        assert!(!VERTICAL_SLICE.is_empty());
    }

    fn tenant() -> TenantId {
        TenantId::new("tenant-alpha").expect("valid tenant")
    }

    fn actor() -> PrincipalId {
        PrincipalId::new("user-alpha").expect("valid principal")
    }

    fn workbook_id() -> WorkbookId {
        WorkbookId::from_drive_object_id(
            oya_office_kernel::ObjectId::new("sheet-1").expect("valid object id"),
        )
    }

    fn address(row: u32, column: u32) -> CellAddress {
        CellAddress::new("sheet-main".to_owned(), row, column).expect("cell address")
    }

    #[test]
    fn sheets_routes_cover_rest_grpc_streaming_and_webhook_surface() {
        let routes = sheets_routes();

        assert!(routes.contains(&SheetsApiRoute::GetWorkbook));
        assert!(routes.contains(&SheetsApiRoute::BatchUpdate));
        assert!(routes.contains(&SheetsApiRoute::GrpcBatchUpdate));
        assert!(routes.contains(&SheetsApiRoute::ReadRange));
        assert!(routes.contains(&SheetsApiRoute::WriteRange));
        assert!(routes.contains(&SheetsApiRoute::CalculateFormula));
        assert!(routes.contains(&SheetsApiRoute::OpenChangeStream));
        assert!(routes.contains(&SheetsApiRoute::RegisterWebhook));
        assert!(routes.contains(&SheetsApiRoute::CreateEmbedSession));
        assert_eq!(
            SheetsApiRoute::BatchUpdate.method_and_path(),
            ("POST", "/api/sheets/v1/workbooks/{workbookId}:batchUpdate")
        );
        assert!(SheetsApiRoute::OpenChangeStream.is_streaming());
    }

    #[test]
    fn batch_update_request_is_tenant_bound_versioned_and_bounded() {
        let op = SheetsBatchOperation::set_value(address(1, 1), CellValue::Text("Q1".to_owned()))
            .expect("operation");
        let request =
            SheetsBatchUpdateRequest::new(tenant(), workbook_id(), actor(), vec![op.clone()], 7)
                .expect("request");

        assert_eq!(request.tenant_id().as_str(), "tenant-alpha");
        assert_eq!(request.operation_count(), 1);
        assert_eq!(request.expected_version_sequence(), 7);
        assert!(
            SheetsBatchUpdateRequest::new(tenant(), workbook_id(), actor(), Vec::new(), 7).is_err()
        );
        assert!(
            SheetsBatchUpdateRequest::new(
                tenant(),
                workbook_id(),
                actor(),
                vec![op; SheetsBatchUpdateRequest::MAX_OPERATIONS + 1],
                7,
            )
            .is_err()
        );
    }

    #[test]
    fn range_read_request_bounds_cells_and_rejects_unbounded_ranges() {
        let bounded = SheetsRangeRef::new(workbook_id(), "sheet-main".to_owned(), 1, 1, 100, 5)
            .expect("bounded range");
        let request = SheetsRangeReadRequest::new(tenant(), bounded.clone()).expect("range read");

        assert_eq!(request.range().cell_count(), 500);
        assert!(request.range().is_bounded());
        assert!(
            SheetsRangeRef::new(workbook_id(), "sheet-main".to_owned(), 1, 1, 20_000, 20).is_err()
        );
    }

    #[test]
    fn formula_automation_and_webhooks_are_explicit_contracts() {
        let formula = FormulaExpression::new("=SUM(A1:A10)".to_owned()).expect("formula");
        let automation = SheetsFormulaAutomationRequest::new(
            tenant(),
            workbook_id(),
            actor(),
            address(11, 1),
            formula,
            9,
        )
        .expect("automation");

        assert_eq!(automation.route(), SheetsApiRoute::CalculateFormula);
        assert_eq!(automation.tenant_id().as_str(), "tenant-alpha");
        assert_eq!(automation.expected_version_sequence(), 9);

        let webhook = SheetsWebhookSubscriptionRequest::new(
            tenant(),
            workbook_id(),
            "https://hooks.example.invalid/sheets".to_owned(),
            vec![
                SheetsWebhookEventKind::RangeUpdated,
                SheetsWebhookEventKind::FormulaRecalculated,
            ],
            "secret-ref-sheets-webhook".to_owned(),
        )
        .expect("webhook");

        assert_eq!(webhook.route(), SheetsApiRoute::RegisterWebhook);
        assert_eq!(webhook.tenant_id().as_str(), "tenant-alpha");
        assert_eq!(webhook.event_kinds().len(), 2);
        assert!(
            SheetsWebhookSubscriptionRequest::new(
                tenant(),
                workbook_id(),
                "http://insecure.example.invalid".to_owned(),
                vec![SheetsWebhookEventKind::RangeUpdated],
                "secret-ref".to_owned(),
            )
            .is_err()
        );
    }

    #[test]
    fn embed_session_request_is_https_tenant_bound_and_ttl_bounded() {
        let embed = SheetsEmbedSessionRequest::new(
            tenant(),
            workbook_id(),
            actor(),
            "https://app.example.invalid".to_owned(),
            SheetsEmbedMode::Edit,
            600,
        )
        .expect("embed session");

        assert_eq!(embed.route(), SheetsApiRoute::CreateEmbedSession);
        assert_eq!(embed.tenant_id().as_str(), "tenant-alpha");
        assert_eq!(embed.allowed_origin(), "https://app.example.invalid");
        assert_eq!(embed.mode(), SheetsEmbedMode::Edit);
        assert_eq!(embed.ttl_seconds(), 600);
        assert!(
            SheetsEmbedSessionRequest::new(
                tenant(),
                workbook_id(),
                actor(),
                "http://app.example.invalid".to_owned(),
                SheetsEmbedMode::View,
                600,
            )
            .is_err()
        );
        assert!(
            SheetsEmbedSessionRequest::new(
                tenant(),
                workbook_id(),
                actor(),
                "https://app.example.invalid".to_owned(),
                SheetsEmbedMode::Comment,
                SheetsEmbedSessionRequest::MAX_TTL_SECONDS + 1,
            )
            .is_err()
        );
    }

    #[test]
    fn sheets_sdk_surface_covers_platform_capabilities_and_connector_ports() {
        let surface = SheetsSdkSurface::platform_default();

        assert!(surface.supports(SheetsSdkCapability::BatchUpdate));
        assert!(surface.supports(SheetsSdkCapability::RangeReadWrite));
        assert!(surface.supports(SheetsSdkCapability::FormulaAutomation));
        assert!(surface.supports(SheetsSdkCapability::Streaming));
        assert!(surface.supports(SheetsSdkCapability::Webhooks));
        assert!(surface.supports(SheetsSdkCapability::ConnectorPorts));
        assert!(surface.supports(SheetsSdkCapability::Embedding));
        assert!(surface.routes().contains(&SheetsApiRoute::GrpcBatchUpdate));
        assert!(
            surface
                .routes()
                .contains(&SheetsApiRoute::CreateEmbedSession)
        );
        assert!(sheets_required_connector_ports().contains(&SheetsConnectorPort::Drive));
        assert!(sheets_required_connector_ports().contains(&SheetsConnectorPort::TenantSecurity));
        assert!(sheets_required_connector_ports().contains(&SheetsConnectorPort::Audit));
        assert!(sheets_required_connector_ports().contains(&SheetsConnectorPort::Quota));
        assert!(sheets_required_connector_ports().contains(&SheetsConnectorPort::Search));
        assert!(sheets_required_connector_ports().contains(&SheetsConnectorPort::Workflow));
        assert!(sheets_required_connector_ports().contains(&SheetsConnectorPort::Collaboration));
        assert!(sheets_required_connector_ports().contains(&SheetsConnectorPort::Format));
        assert!(sheets_required_connector_ports().contains(&SheetsConnectorPort::Embedding));
    }

    #[test]
    fn g067_sheets_sdk_api_integration_parity_is_contractual_not_deferred() {
        let surface = SheetsSdkSurface::platform_default();
        let routes = sheets_routes();
        let required_ports = sheets_required_connector_ports();

        for capability in [
            SheetsSdkCapability::BatchUpdate,
            SheetsSdkCapability::RangeReadWrite,
            SheetsSdkCapability::FormulaAutomation,
            SheetsSdkCapability::Streaming,
            SheetsSdkCapability::Webhooks,
            SheetsSdkCapability::ConnectorPorts,
            SheetsSdkCapability::Embedding,
        ] {
            assert!(surface.supports(capability));
        }

        for route in [
            SheetsApiRoute::GetWorkbook,
            SheetsApiRoute::BatchUpdate,
            SheetsApiRoute::GrpcBatchUpdate,
            SheetsApiRoute::ReadRange,
            SheetsApiRoute::WriteRange,
            SheetsApiRoute::CalculateFormula,
            SheetsApiRoute::OpenChangeStream,
            SheetsApiRoute::RegisterWebhook,
            SheetsApiRoute::CreateEmbedSession,
        ] {
            assert!(routes.contains(&route));
            assert!(surface.routes().contains(&route));
        }

        assert!(SheetsApiRoute::OpenChangeStream.is_streaming());
        assert_eq!(routes.len(), 9);
        assert_eq!(required_ports.len(), 9);
        for port in [
            SheetsConnectorPort::Drive,
            SheetsConnectorPort::TenantSecurity,
            SheetsConnectorPort::Audit,
            SheetsConnectorPort::Quota,
            SheetsConnectorPort::Search,
            SheetsConnectorPort::Workflow,
            SheetsConnectorPort::Collaboration,
            SheetsConnectorPort::Format,
            SheetsConnectorPort::Embedding,
        ] {
            assert!(required_ports.contains(&port));
        }

        let range = SheetsRangeRef::new(workbook_id(), "sheet-main".to_owned(), 1, 1, 10, 3)
            .expect("bounded range");
        let operation = SheetsBatchOperation::clear_range(range);
        let batch =
            SheetsBatchUpdateRequest::new(tenant(), workbook_id(), actor(), vec![operation], 11)
                .expect("batch request");
        assert_eq!(batch.operation_count(), 1);
        assert_eq!(batch.expected_version_sequence(), 11);

        let formula = FormulaExpression::new("=SUM(A1:C10)".to_owned()).expect("formula");
        let automation = SheetsFormulaAutomationRequest::new(
            tenant(),
            workbook_id(),
            actor(),
            address(11, 1),
            formula,
            12,
        )
        .expect("formula automation");
        assert_eq!(automation.route(), SheetsApiRoute::CalculateFormula);

        let webhook = SheetsWebhookSubscriptionRequest::new(
            tenant(),
            workbook_id(),
            "https://hooks.example.invalid/sheets".to_owned(),
            vec![
                SheetsWebhookEventKind::RangeUpdated,
                SheetsWebhookEventKind::WorkbookVersionCommitted,
            ],
            "secret-ref-g067-sheets-webhook".to_owned(),
        )
        .expect("webhook");
        assert_eq!(webhook.route(), SheetsApiRoute::RegisterWebhook);

        let embed = SheetsEmbedSessionRequest::new(
            tenant(),
            workbook_id(),
            actor(),
            "https://app.example.invalid".to_owned(),
            SheetsEmbedMode::Edit,
            900,
        )
        .expect("embed");
        assert_eq!(embed.route(), SheetsApiRoute::CreateEmbedSession);
    }
}
