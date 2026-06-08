#![forbid(unsafe_code)]
//! Workbook, cell, formula, protected-range, and Sheets SDK/API domain model.
//!
//! This early slice binds every workbook to Oya Drive so Sheets API/SDK objects
//! share Drive ACL, KMS, lifecycle, and audit semantics.

use oya_office_drive_domain::{DriveObjectBinding, DriveObjectKind};
use oya_office_format_domain::{FormatFixtureBinding, FormatJobDirection, OfficeFormatKind};
use oya_office_kernel::{ObjectId, PrincipalId, TenantId};

/// Stable crate identifier used by workspace and Buck2 scaffold verification.
pub const CRATE_NAME: &str = "oya-office-sheet-domain";

/// Product vertical slice owned by this crate.
pub const VERTICAL_SLICE: &str = "sheets";

/// Source-shaped architectural layer represented by this crate.
pub const ARCHITECTURE_LAYER: &str = "domain";

/// Workbook identifier inside the Sheets slice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkbookId(ObjectId);

impl WorkbookId {
    /// Creates a workbook id from Drive object id.
    #[must_use]
    pub const fn from_drive_object_id(object_id: ObjectId) -> Self {
        Self(object_id)
    }

    /// Returns object id.
    #[must_use]
    pub const fn as_object_id(&self) -> &ObjectId {
        &self.0
    }
}

/// Drive-bound workbook aggregate shell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkbookDriveBinding {
    workbook_id: WorkbookId,
    binding: DriveObjectBinding,
}

impl WorkbookDriveBinding {
    /// Creates a workbook binding and verifies it points to a Drive spreadsheet object.
    pub fn new(binding: DriveObjectBinding) -> Result<Self, WorkbookBindingError> {
        if binding.kind() != DriveObjectKind::Spreadsheet {
            return Err(WorkbookBindingError::new(
                "sheets binding requires a spreadsheet object",
            ));
        }
        Ok(Self {
            workbook_id: WorkbookId::from_drive_object_id(binding.object_id().clone()),
            binding,
        })
    }

    /// Returns workbook id.
    #[must_use]
    pub const fn workbook_id(&self) -> &WorkbookId {
        &self.workbook_id
    }

    /// Returns Drive binding.
    #[must_use]
    pub const fn drive_binding(&self) -> &DriveObjectBinding {
        &self.binding
    }

    /// Returns tenant id.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        self.binding.tenant_id()
    }
}

/// Address of a cell inside one worksheet.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CellAddress {
    sheet_id: String,
    row: u32,
    column: u32,
}

impl CellAddress {
    /// Creates a 1-based worksheet cell address.
    pub fn new(
        sheet_id: impl Into<String>,
        row: u32,
        column: u32,
    ) -> Result<Self, WorkbookBindingError> {
        if row == 0 || column == 0 {
            return Err(WorkbookBindingError::new(
                "cell address row and column must be 1-based",
            ));
        }
        Ok(Self {
            sheet_id: validate_required_text("worksheet id", sheet_id)?,
            row,
            column,
        })
    }

    /// Returns worksheet id.
    #[must_use]
    pub fn sheet_id(&self) -> &str {
        self.sheet_id.as_str()
    }

    /// Returns 1-based row.
    #[must_use]
    pub const fn row(&self) -> u32 {
        self.row
    }

    /// Returns 1-based column.
    #[must_use]
    pub const fn column(&self) -> u32 {
        self.column
    }
}

/// Baseline cell value vocabulary for Sheets contracts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CellValue {
    /// Text cell value.
    Text(String),
    /// Decimal number represented as a stable string for provider-neutral contracts.
    Number(String),
    /// Boolean cell value.
    Boolean(bool),
    /// Blank cell.
    Blank,
}

/// Formula expression with spreadsheet formula syntax.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormulaExpression(String);

impl FormulaExpression {
    /// Creates a formula expression and requires the leading `=`.
    pub fn new(value: impl Into<String>) -> Result<Self, WorkbookBindingError> {
        let value = validate_required_text("formula expression", value)?;
        if !value.starts_with('=') || value.len() == 1 {
            return Err(WorkbookBindingError::new(
                "formula expression must start with '=' and include an expression",
            ));
        }
        Ok(Self(value))
    }

    /// Returns formula text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// One workbook cell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkbookCell {
    address: CellAddress,
    value: CellValue,
    formula: Option<FormulaExpression>,
    version_sequence: u64,
}

impl WorkbookCell {
    /// Creates a cell with optional formula and a version sequence.
    pub fn new(
        address: CellAddress,
        value: CellValue,
        formula: Option<FormulaExpression>,
        version_sequence: u64,
    ) -> Result<Self, WorkbookBindingError> {
        if version_sequence == 0 {
            return Err(WorkbookBindingError::new(
                "workbook cell version sequence must be at least 1",
            ));
        }
        validate_cell_value(&value)?;
        Ok(Self {
            address,
            value,
            formula,
            version_sequence,
        })
    }

    /// Returns the cell address.
    #[must_use]
    pub const fn address(&self) -> &CellAddress {
        &self.address
    }

    /// Returns the cell value.
    #[must_use]
    pub const fn value(&self) -> &CellValue {
        &self.value
    }

    /// Returns formula when present.
    #[must_use]
    pub const fn formula(&self) -> Option<&FormulaExpression> {
        self.formula.as_ref()
    }

    /// Returns cell version sequence.
    #[must_use]
    pub const fn version_sequence(&self) -> u64 {
        self.version_sequence
    }
}

/// One worksheet inside a workbook.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Worksheet {
    sheet_id: String,
    title: String,
    cells: Vec<WorkbookCell>,
}

impl Worksheet {
    /// Creates a worksheet with validated identity and cell addresses.
    pub fn new(
        sheet_id: impl Into<String>,
        title: impl Into<String>,
        cells: Vec<WorkbookCell>,
    ) -> Result<Self, WorkbookBindingError> {
        let sheet_id = validate_required_text("worksheet id", sheet_id)?;
        ensure_cells_belong_to_sheet(&sheet_id, &cells)?;
        ensure_unique_cell_addresses(&cells)?;
        Ok(Self {
            sheet_id,
            title: validate_required_text("worksheet title", title)?,
            cells,
        })
    }

    /// Returns worksheet id.
    #[must_use]
    pub fn sheet_id(&self) -> &str {
        self.sheet_id.as_str()
    }

    /// Returns worksheet title.
    #[must_use]
    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    /// Returns cells.
    #[must_use]
    pub fn cells(&self) -> &[WorkbookCell] {
        self.cells.as_slice()
    }
}

/// Drive-bound workbook aggregate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkbookModel {
    binding: WorkbookDriveBinding,
    title: String,
    sheets: Vec<Worksheet>,
    version_sequence: u64,
}

impl WorkbookModel {
    /// Creates a validated workbook aggregate.
    pub fn new(
        binding: WorkbookDriveBinding,
        title: impl Into<String>,
        sheets: Vec<Worksheet>,
        version_sequence: u64,
    ) -> Result<Self, WorkbookBindingError> {
        if sheets.is_empty() {
            return Err(WorkbookBindingError::new(
                "workbook model requires at least one worksheet",
            ));
        }
        if version_sequence == 0 {
            return Err(WorkbookBindingError::new(
                "workbook model version sequence must be at least 1",
            ));
        }
        ensure_unique_sheet_ids(&sheets)?;
        Ok(Self {
            binding,
            title: validate_required_text("workbook title", title)?,
            sheets,
            version_sequence,
        })
    }

    /// Returns workbook id.
    #[must_use]
    pub const fn workbook_id(&self) -> &WorkbookId {
        self.binding.workbook_id()
    }

    /// Returns Drive-bound workbook binding.
    #[must_use]
    pub const fn drive_binding(&self) -> &WorkbookDriveBinding {
        &self.binding
    }

    /// Returns tenant id.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        self.binding.tenant_id()
    }

    /// Returns workbook title.
    #[must_use]
    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    /// Returns worksheets.
    #[must_use]
    pub fn sheets(&self) -> &[Worksheet] {
        self.sheets.as_slice()
    }

    /// Returns optimistic concurrency workbook version sequence.
    #[must_use]
    pub const fn version_sequence(&self) -> u64 {
        self.version_sequence
    }

    fn has_sheet(&self, sheet_id: &str) -> bool {
        self.sheets.iter().any(|sheet| sheet.sheet_id() == sheet_id)
    }
}

/// Protected spreadsheet range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProtectedRange {
    workbook_id: WorkbookId,
    sheet_id: String,
    start_row: u32,
    start_column: u32,
    end_row: u32,
    end_column: u32,
    editors: Vec<PrincipalId>,
}

impl ProtectedRange {
    /// Creates a protected range with inclusive 1-based bounds.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        workbook_id: WorkbookId,
        sheet_id: impl Into<String>,
        start_row: u32,
        start_column: u32,
        end_row: u32,
        end_column: u32,
        editors: Vec<PrincipalId>,
    ) -> Result<Self, WorkbookBindingError> {
        if start_row == 0 || start_column == 0 || end_row == 0 || end_column == 0 {
            return Err(WorkbookBindingError::new(
                "protected range bounds must be 1-based",
            ));
        }
        if start_row > end_row || start_column > end_column {
            return Err(WorkbookBindingError::new(
                "protected range start must not exceed end",
            ));
        }
        if editors.is_empty() {
            return Err(WorkbookBindingError::new(
                "protected range requires at least one editor",
            ));
        }
        Ok(Self {
            workbook_id,
            sheet_id: validate_required_text("protected range worksheet id", sheet_id)?,
            start_row,
            start_column,
            end_row,
            end_column,
            editors,
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

    /// Returns true when a cell falls inside the inclusive range.
    #[must_use]
    pub fn contains_cell(&self, address: &CellAddress) -> bool {
        self.sheet_id() == address.sheet_id()
            && address.row() >= self.start_row
            && address.row() <= self.end_row
            && address.column() >= self.start_column
            && address.column() <= self.end_column
    }

    /// Returns true when the actor may edit this range.
    #[must_use]
    pub fn allows_actor(&self, actor_id: &PrincipalId) -> bool {
        self.editors.iter().any(|editor| editor == actor_id)
    }

    /// Validates workbook id and worksheet binding.
    pub fn validate_for_workbook(&self, model: &WorkbookModel) -> Result<(), WorkbookBindingError> {
        if self.workbook_id() != model.workbook_id() {
            return Err(WorkbookBindingError::new(
                "protected range workbook id does not match model",
            ));
        }
        if !model.has_sheet(self.sheet_id()) {
            return Err(WorkbookBindingError::new(
                "protected range worksheet does not exist in model",
            ));
        }
        Ok(())
    }
}

/// Collaborative cell edit operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CollaborativeCellEditKind {
    /// Set a literal cell value.
    SetValue,
    /// Set a formula expression.
    SetFormula,
    /// Clear the target cell.
    Clear,
}

/// One optimistic-concurrency collaborative cell edit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollaborativeCellEdit {
    workbook_id: WorkbookId,
    actor_id: PrincipalId,
    address: CellAddress,
    kind: CollaborativeCellEditKind,
    expected_version_sequence: u64,
}

impl CollaborativeCellEdit {
    /// Creates a collaborative cell edit.
    pub fn new(
        workbook_id: WorkbookId,
        actor_id: PrincipalId,
        address: CellAddress,
        kind: CollaborativeCellEditKind,
        expected_version_sequence: u64,
    ) -> Result<Self, WorkbookBindingError> {
        if expected_version_sequence == 0 {
            return Err(WorkbookBindingError::new(
                "collaborative cell edit expected version must be at least 1",
            ));
        }
        Ok(Self {
            workbook_id,
            actor_id,
            address,
            kind,
            expected_version_sequence,
        })
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

    /// Returns target address.
    #[must_use]
    pub const fn address(&self) -> &CellAddress {
        &self.address
    }

    /// Returns edit kind.
    #[must_use]
    pub const fn kind(&self) -> CollaborativeCellEditKind {
        self.kind
    }

    /// Returns expected workbook version sequence.
    #[must_use]
    pub const fn expected_version_sequence(&self) -> u64 {
        self.expected_version_sequence
    }

    /// Validates workbook/sheet/version binding and protected-range authorization.
    pub fn validate_for_workbook(
        &self,
        model: &WorkbookModel,
        protected_ranges: &[ProtectedRange],
    ) -> Result<(), WorkbookBindingError> {
        if self.workbook_id() != model.workbook_id() {
            return Err(WorkbookBindingError::new(
                "collaborative cell edit workbook id does not match model",
            ));
        }
        if !model.has_sheet(self.address().sheet_id()) {
            return Err(WorkbookBindingError::new(
                "collaborative cell edit worksheet does not exist in model",
            ));
        }
        if self.expected_version_sequence() != model.version_sequence() {
            return Err(WorkbookBindingError::new(
                "collaborative cell edit expected version does not match model",
            ));
        }
        for protected_range in protected_ranges {
            protected_range.validate_for_workbook(model)?;
            if protected_range.contains_cell(self.address())
                && !protected_range.allows_actor(self.actor_id())
            {
                return Err(WorkbookBindingError::new(
                    "collaborative cell edit actor cannot edit protected range",
                ));
            }
        }
        Ok(())
    }
}

/// Drive-bound XLSX import/export contract for the Sheets slice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkbookXlsxFormatPlan {
    workbook_binding: WorkbookDriveBinding,
    format_binding: FormatFixtureBinding,
    direction: FormatJobDirection,
    preserve_formulas: bool,
    preserve_protected_ranges: bool,
    requires_drive_binding: bool,
}

impl WorkbookXlsxFormatPlan {
    /// Creates an XLSX format plan from an already validated workbook binding.
    pub fn new(
        workbook_binding: WorkbookDriveBinding,
        direction: FormatJobDirection,
    ) -> Result<Self, WorkbookBindingError> {
        let format_binding = FormatFixtureBinding::new(
            workbook_binding.drive_binding().clone(),
            OfficeFormatKind::Xlsx,
        )
        .map_err(|error| {
            WorkbookBindingError::new(format!(
                "workbook XLSX format binding failed: {}",
                error.message()
            ))
        })?;

        Ok(Self {
            workbook_binding,
            format_binding,
            direction,
            preserve_formulas: true,
            preserve_protected_ranges: true,
            requires_drive_binding: true,
        })
    }

    /// Creates an XLSX format plan from a raw Drive object binding.
    pub fn from_drive_binding(
        binding: DriveObjectBinding,
        direction: FormatJobDirection,
    ) -> Result<Self, WorkbookBindingError> {
        Self::new(WorkbookDriveBinding::new(binding)?, direction)
    }

    /// Returns workbook binding.
    #[must_use]
    pub const fn workbook_binding(&self) -> &WorkbookDriveBinding {
        &self.workbook_binding
    }

    /// Returns format binding.
    #[must_use]
    pub const fn format_binding(&self) -> &FormatFixtureBinding {
        &self.format_binding
    }

    /// Returns Office format kind.
    #[must_use]
    pub const fn format_kind(&self) -> OfficeFormatKind {
        self.format_binding.format_kind()
    }

    /// Returns format job direction.
    #[must_use]
    pub const fn direction(&self) -> FormatJobDirection {
        self.direction
    }

    /// Returns true because XLSX format work must remain Drive-bound.
    #[must_use]
    pub const fn requires_drive_binding(&self) -> bool {
        self.requires_drive_binding
    }

    /// Returns true when formulas are part of the preservation contract.
    #[must_use]
    pub const fn preserve_formulas(&self) -> bool {
        self.preserve_formulas
    }

    /// Returns true when protected ranges are part of the preservation contract.
    #[must_use]
    pub const fn preserve_protected_ranges(&self) -> bool {
        self.preserve_protected_ranges
    }
}

/// Workbook binding validation error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkbookBindingError {
    message: String,
}

impl WorkbookBindingError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns message.
    #[must_use]
    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

impl core::fmt::Display for WorkbookBindingError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.message())
    }
}

impl std::error::Error for WorkbookBindingError {}

fn validate_required_text(
    field: &'static str,
    value: impl Into<String>,
) -> Result<String, WorkbookBindingError> {
    let value = value.into();
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(WorkbookBindingError::new(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

fn validate_cell_value(value: &CellValue) -> Result<(), WorkbookBindingError> {
    match value {
        CellValue::Text(text) => {
            validate_required_text("cell text value", text.as_str())?;
        }
        CellValue::Number(number) => {
            let number = validate_required_text("cell number value", number.as_str())?;
            if !matches!(number.parse::<f64>(), Ok(parsed) if parsed.is_finite()) {
                return Err(WorkbookBindingError::new(
                    "cell number value must parse as a finite decimal",
                ));
            }
        }
        CellValue::Boolean(_) | CellValue::Blank => {}
    }
    Ok(())
}

fn ensure_cells_belong_to_sheet(
    sheet_id: &str,
    cells: &[WorkbookCell],
) -> Result<(), WorkbookBindingError> {
    for cell in cells {
        if cell.address().sheet_id() != sheet_id {
            return Err(WorkbookBindingError::new(
                "worksheet cell address must use the worksheet id",
            ));
        }
    }
    Ok(())
}

fn ensure_unique_cell_addresses(cells: &[WorkbookCell]) -> Result<(), WorkbookBindingError> {
    let mut seen = std::collections::BTreeSet::new();
    for cell in cells {
        if !seen.insert(cell.address()) {
            return Err(WorkbookBindingError::new(
                "worksheet cell addresses must be unique",
            ));
        }
    }
    Ok(())
}

fn ensure_unique_sheet_ids(sheets: &[Worksheet]) -> Result<(), WorkbookBindingError> {
    let mut seen = std::collections::BTreeSet::new();
    for sheet in sheets {
        if !seen.insert(sheet.sheet_id()) {
            return Err(WorkbookBindingError::new(
                "workbook worksheet ids must be unique",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use oya_office_drive_domain::{DriveObjectBinding, DriveObjectKind};
    use oya_office_format_domain::{FormatJobDirection, OfficeFormatKind};
    use oya_office_kernel::{DataClass, ObjectId, PrincipalId, TenantId};

    use super::{
        ARCHITECTURE_LAYER, CRATE_NAME, CellAddress, CellValue, CollaborativeCellEdit,
        CollaborativeCellEditKind, FormulaExpression, ProtectedRange, VERTICAL_SLICE, WorkbookCell,
        WorkbookDriveBinding, WorkbookModel, WorkbookXlsxFormatPlan, Worksheet,
    };

    #[test]
    fn scaffold_identity_is_declared() {
        assert!(!ARCHITECTURE_LAYER.is_empty());
        assert!(!CRATE_NAME.is_empty());
        assert!(!VERTICAL_SLICE.is_empty());
    }

    #[test]
    fn sheets_bind_only_to_drive_spreadsheet_objects() {
        let binding = DriveObjectBinding::new(
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            ObjectId::new("sheet-1").expect("valid object id"),
            DriveObjectKind::Spreadsheet,
            DataClass::Confidential,
        );
        let workbook = WorkbookDriveBinding::new(binding).expect("workbook binding");
        assert_eq!(workbook.workbook_id().as_object_id().as_str(), "sheet-1");

        let wrong_kind = DriveObjectBinding::new(
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            ObjectId::new("doc-1").expect("valid object id"),
            DriveObjectKind::Document,
            DataClass::Internal,
        );
        assert!(WorkbookDriveBinding::new(wrong_kind).is_err());
    }

    fn actor() -> PrincipalId {
        PrincipalId::new("user-alpha").expect("valid principal")
    }

    fn other_actor() -> PrincipalId {
        PrincipalId::new("user-beta").expect("valid principal")
    }

    fn workbook_binding() -> WorkbookDriveBinding {
        WorkbookDriveBinding::new(DriveObjectBinding::new(
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            ObjectId::new("sheet-1").expect("valid object id"),
            DriveObjectKind::Spreadsheet,
            DataClass::Confidential,
        ))
        .expect("workbook binding")
    }

    fn workbook_model() -> WorkbookModel {
        let address = CellAddress::new("sheet-main".to_owned(), 1, 1).expect("cell address");
        let cell = WorkbookCell::new(address, CellValue::Text("Revenue".to_owned()), None, 1)
            .expect("cell");
        let worksheet = Worksheet::new("sheet-main".to_owned(), "Forecast".to_owned(), vec![cell])
            .expect("worksheet");

        WorkbookModel::new(
            workbook_binding(),
            "FY Forecast".to_owned(),
            vec![worksheet],
            1,
        )
        .expect("workbook model")
    }

    #[test]
    fn workbook_model_requires_title_sheets_and_drive_spreadsheet_binding() {
        let model = workbook_model();

        assert_eq!(model.title(), "FY Forecast");
        assert_eq!(model.sheets().len(), 1);
        assert_eq!(model.version_sequence(), 1);
        assert_eq!(
            model.drive_binding().drive_binding().kind(),
            DriveObjectKind::Spreadsheet
        );

        assert!(
            WorkbookModel::new(
                workbook_binding(),
                " ".to_owned(),
                model.sheets().to_vec(),
                1
            )
            .is_err()
        );
        assert!(WorkbookModel::new(workbook_binding(), "Workbook".to_owned(), vec![], 1).is_err());
    }

    #[test]
    fn cell_addresses_formulas_and_values_are_validated() {
        let address = CellAddress::new("sheet-main".to_owned(), 10, 3).expect("address");
        let formula = FormulaExpression::new("=SUM(A1:A9)".to_owned()).expect("formula");
        let cell = WorkbookCell::new(
            address.clone(),
            CellValue::Number("42.00".to_owned()),
            Some(formula),
            2,
        )
        .expect("cell");

        assert_eq!(address.row(), 10);
        assert_eq!(address.column(), 3);
        assert_eq!(cell.version_sequence(), 2);
        assert!(cell.formula().is_some());
        assert!(CellAddress::new("sheet-main".to_owned(), 0, 1).is_err());
        assert!(FormulaExpression::new("SUM(A1:A9)".to_owned()).is_err());
    }

    #[test]
    fn protected_ranges_bind_to_workbook_and_authorized_editors() {
        let model = workbook_model();
        let range = ProtectedRange::new(
            model.workbook_id().clone(),
            "sheet-main".to_owned(),
            1,
            1,
            10,
            4,
            vec![actor()],
        )
        .expect("protected range");

        assert!(range.validate_for_workbook(&model).is_ok());
        assert!(
            range.contains_cell(&CellAddress::new("sheet-main".to_owned(), 5, 2).expect("cell"))
        );
        assert!(range.allows_actor(&actor()));
        assert!(!range.allows_actor(&other_actor()));
    }

    #[test]
    fn collaborative_cell_edit_requires_expected_version_and_respects_protected_ranges() {
        let model = workbook_model();
        let address = CellAddress::new("sheet-main".to_owned(), 1, 1).expect("address");
        let protected = ProtectedRange::new(
            model.workbook_id().clone(),
            "sheet-main".to_owned(),
            1,
            1,
            2,
            2,
            vec![actor()],
        )
        .expect("protected range");
        let edit = CollaborativeCellEdit::new(
            model.workbook_id().clone(),
            actor(),
            address.clone(),
            CollaborativeCellEditKind::SetFormula,
            model.version_sequence(),
        )
        .expect("edit");
        let denied = CollaborativeCellEdit::new(
            model.workbook_id().clone(),
            other_actor(),
            address,
            CollaborativeCellEditKind::SetValue,
            model.version_sequence(),
        )
        .expect("edit");

        assert!(
            edit.validate_for_workbook(&model, &[protected.clone()])
                .is_ok()
        );
        assert!(denied.validate_for_workbook(&model, &[protected]).is_err());
        assert!(
            CollaborativeCellEdit::new(
                model.workbook_id().clone(),
                actor(),
                CellAddress::new("sheet-main".to_owned(), 1, 1).expect("address"),
                CollaborativeCellEditKind::SetValue,
                0,
            )
            .is_err()
        );
    }

    #[test]
    fn xlsx_import_export_plan_is_drive_bound_and_xlsx_only() {
        let plan = WorkbookXlsxFormatPlan::new(workbook_binding(), FormatJobDirection::Import)
            .expect("xlsx plan");

        assert_eq!(plan.format_kind(), OfficeFormatKind::Xlsx);
        assert_eq!(plan.direction(), FormatJobDirection::Import);
        assert!(plan.requires_drive_binding());
        assert!(plan.preserve_formulas());
        assert!(plan.preserve_protected_ranges());

        let wrong_kind = DriveObjectBinding::new(
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            ObjectId::new("doc-1").expect("valid object id"),
            DriveObjectKind::Document,
            DataClass::Internal,
        );
        assert!(
            WorkbookXlsxFormatPlan::from_drive_binding(wrong_kind, FormatJobDirection::Export)
                .is_err()
        );
    }
}
