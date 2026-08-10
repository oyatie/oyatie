//! Workspace sheets kernel.
//!
//! Typed kernel records for the W-Workspace-Stable Sheets surface named by
//! `docs/products/workspace/PRD.md` and ADR-0029. The kernel owns sheet
//! metadata, CRDT binding to the shared collab runtime, formula cells, cell
//! dependency graph validation, and the Foundry what-if seam without owning
//! formula execution or storage adapters.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use oya_collab_runtime_domain::{CollabRuntime, CollabSurface};
use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const SHEET_SCHEMA_VERSION: u32 = 1;
const CELL_GRAPH_SCHEMA_VERSION: u32 = 1;
const MAX_SHEET_ROWS: u32 = 1_048_576;
const MAX_SHEET_COLUMNS: u16 = 16_384;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SheetError {
    InvalidSheetId,
    InvalidTenantId,
    InvalidRegion,
    InvalidCellId,
    InvalidDrivePath,
    InvalidTitle,
    InvalidCollabRuntime,
    InvalidCollabSurface,
    InvalidTabId,
    InvalidCellAddress,
    DuplicateCellAddress,
    InvalidFormula,
    UnexpectedFormula,
    MissingFormula,
    MissingDependencyEndpoint,
    DependencyTargetNotFormula,
    SelfDependency,
    CyclicDependency,
    InvalidScenarioId,
    InvalidActorRef,
    EmptyWhatIfInputs,
    EmptyWhatIfOutputs,
    InvalidTimeOrder,
    InvalidDataClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CellValueKind {
    Empty,
    Text,
    Number,
    Boolean,
    Formula,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SheetCreate {
    pub id: String,                            // data_class: INTERNAL_ONLY
    pub title: String,                         // data_class: PII_QUASI_IDENTIFIER
    pub drive_path: String,                    // data_class: PII_QUASI_IDENTIFIER
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub region: String,                        // data_class: INTERNAL_ONLY
    pub cell_id: String,                       // data_class: INTERNAL_ONLY
    pub data_class: Option<PrivacyDataClass>,  // data_class: INTERNAL_ONLY
    pub collab_runtime: CollabRuntime,         // data_class: PII_IDENTIFYING
    pub cell_graph: CellGraph,                 // data_class: PII_IDENTIFYING
    pub indexed_at_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,         // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: u64,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sheet {
    pub id: Classified<String>,                    // data_class: INTERNAL_ONLY
    pub title: Classified<String>,                 // data_class: PII_QUASI_IDENTIFIER
    pub drive_path: Classified<String>,            // data_class: PII_QUASI_IDENTIFIER
    pub tenant_id: Classified<String>,             // data_class: INTERNAL_ONLY
    pub region: Classified<String>,                // data_class: INTERNAL_ONLY
    pub cell_id: Classified<String>,               // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>,  // data_class: INTERNAL_ONLY
    pub collab_runtime: Classified<CollabRuntime>, // data_class: PII_IDENTIFYING
    pub cell_graph: Classified<CellGraph>,         // data_class: PII_IDENTIFYING
    pub indexed_at_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CellAddress {
    pub tab_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub column: Classified<u16>,    // data_class: INTERNAL_ONLY
    pub row: Classified<u32>,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SheetCellCreate {
    pub address: CellAddress,                 // data_class: INTERNAL_ONLY
    pub value_kind: CellValueKind,            // data_class: INTERNAL_ONLY
    pub formula: Option<String>,              // data_class: PII_IDENTIFYING
    pub value_preview: Option<String>,        // data_class: PII_IDENTIFYING
    pub data_class: Option<PrivacyDataClass>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SheetCell {
    pub address: Classified<CellAddress>, // data_class: INTERNAL_ONLY
    pub value_kind: Classified<CellValueKind>, // data_class: INTERNAL_ONLY
    pub formula: Classified<Option<String>>, // data_class: PII_IDENTIFYING
    pub value_preview: Classified<Option<String>>, // data_class: PII_IDENTIFYING
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CellDependency {
    pub source: CellAddress, // data_class: INTERNAL_ONLY
    pub target: CellAddress, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellGraph {
    pub cells: Vec<SheetCell>,             // data_class: PII_IDENTIFYING
    pub dependencies: Vec<CellDependency>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatIfScenarioCreate {
    pub scenario_id: String,            // data_class: INTERNAL_ONLY
    pub sheet_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub actor_ref: String,              // data_class: PII_IDENTIFYING
    pub input_cells: Vec<CellAddress>,  // data_class: INTERNAL_ONLY
    pub output_cells: Vec<CellAddress>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatIfScenario {
    pub scenario_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub sheet_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub actor_ref: Classified<String>,   // data_class: PII_IDENTIFYING
    pub input_cells: Classified<Vec<CellAddress>>, // data_class: INTERNAL_ONLY
    pub output_cells: Classified<Vec<CellAddress>>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
}

pub trait SheetReader {
    fn read_cell(
        &self,
        tenant_id: &str,
        sheet_id: &str,
        address: &CellAddress,
    ) -> Result<Option<SheetCell>, SheetError>;
}

impl Sheet {
    pub fn new(input: SheetCreate) -> Result<Self, SheetError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_sheet_data_class());
        validate_non_empty(&input.id, SheetError::InvalidSheetId)?;
        validate_non_empty(&input.tenant_id, SheetError::InvalidTenantId)?;
        validate_non_empty(&input.region, SheetError::InvalidRegion)?;
        validate_non_empty(&input.cell_id, SheetError::InvalidCellId)?;
        validate_non_empty(&input.title, SheetError::InvalidTitle)?;
        validate_drive_path(&input.drive_path)?;
        validate_time_order(
            input.created_at_epoch_seconds,
            input.updated_at_epoch_seconds,
        )?;
        validate_collab_runtime_binding(
            &input.collab_runtime,
            &input.id,
            &input.tenant_id,
            &input.region,
            &input.cell_id,
        )?;
        input.cell_graph.validate()?;

        Ok(Self {
            id: internal(input.id),
            title: Classified::new(input.title, title_data_class()),
            drive_path: Classified::new(input.drive_path, title_data_class()),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            cell_id: internal(input.cell_id),
            data_class: internal(data_class),
            collab_runtime: Classified::new(input.collab_runtime, sheet_content_data_class()),
            cell_graph: Classified::new(input.cell_graph, sheet_content_data_class()),
            indexed_at_epoch_seconds: internal(input.indexed_at_epoch_seconds),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.updated_at_epoch_seconds),
            schema_version: internal(SHEET_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

impl CellAddress {
    pub fn new(tab_id: String, column: u16, row: u32) -> Result<Self, SheetError> {
        validate_non_empty(&tab_id, SheetError::InvalidTabId)?;
        if column == 0 || column > MAX_SHEET_COLUMNS || row == 0 || row > MAX_SHEET_ROWS {
            return Err(SheetError::InvalidCellAddress);
        }
        Ok(Self {
            tab_id: internal(tab_id),
            column: internal(column),
            row: internal(row),
        })
    }
}

impl SheetCell {
    pub fn new(input: SheetCellCreate) -> Result<Self, SheetError> {
        input.address.validate()?;
        validate_formula_shape(input.value_kind, input.formula.as_deref())?;
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_sheet_data_class());

        Ok(Self {
            address: internal(input.address),
            value_kind: internal(input.value_kind),
            formula: Classified::new(input.formula, sheet_content_data_class()),
            value_preview: Classified::new(input.value_preview, sheet_content_data_class()),
            data_class: internal(data_class),
        })
    }

    pub fn is_formula(&self) -> bool {
        self.value_kind.value == CellValueKind::Formula
    }
}

impl CellAddress {
    fn validate(&self) -> Result<(), SheetError> {
        validate_non_empty(&self.tab_id.value, SheetError::InvalidTabId)?;
        if self.column.value == 0
            || self.column.value > MAX_SHEET_COLUMNS
            || self.row.value == 0
            || self.row.value > MAX_SHEET_ROWS
        {
            return Err(SheetError::InvalidCellAddress);
        }
        Ok(())
    }
}

impl CellDependency {
    pub fn new(source: CellAddress, target: CellAddress) -> Result<Self, SheetError> {
        source.validate()?;
        target.validate()?;
        if source == target {
            return Err(SheetError::SelfDependency);
        }
        Ok(Self { source, target })
    }
}

impl CellGraph {
    pub fn new(
        cells: Vec<SheetCell>,
        dependencies: Vec<CellDependency>,
    ) -> Result<Self, SheetError> {
        let graph = Self {
            cells,
            dependencies,
            schema_version: internal(CELL_GRAPH_SCHEMA_VERSION),
        };
        graph.validate()?;
        Ok(graph)
    }

    pub fn validate(&self) -> Result<(), SheetError> {
        let mut cells_by_address = BTreeMap::new();
        for cell in &self.cells {
            cell.address.value.validate()?;
            if cells_by_address
                .insert(cell.address.value.clone(), cell)
                .is_some()
            {
                return Err(SheetError::DuplicateCellAddress);
            }
        }
        validate_dependencies(&cells_by_address, &self.dependencies)?;
        Ok(())
    }
}

impl WhatIfScenario {
    pub fn new(input: WhatIfScenarioCreate) -> Result<Self, SheetError> {
        validate_non_empty(&input.scenario_id, SheetError::InvalidScenarioId)?;
        validate_non_empty(&input.sheet_id, SheetError::InvalidSheetId)?;
        validate_non_empty(&input.tenant_id, SheetError::InvalidTenantId)?;
        validate_non_empty(&input.actor_ref, SheetError::InvalidActorRef)?;
        if input.input_cells.is_empty() {
            return Err(SheetError::EmptyWhatIfInputs);
        }
        if input.output_cells.is_empty() {
            return Err(SheetError::EmptyWhatIfOutputs);
        }
        for address in input.input_cells.iter().chain(input.output_cells.iter()) {
            address.validate()?;
        }

        Ok(Self {
            scenario_id: internal(input.scenario_id),
            sheet_id: internal(input.sheet_id),
            tenant_id: internal(input.tenant_id),
            actor_ref: Classified::new(input.actor_ref, actor_data_class()),
            input_cells: internal(input.input_cells),
            output_cells: internal(input.output_cells),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
        })
    }
}

pub fn default_workspace_sheet_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn sheet_content_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn title_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_quasi_identifier()
}

pub fn actor_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn workspace_sheet_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, SheetError> {
    PrivacyDataClass::new(data_class).map_err(|_| SheetError::InvalidDataClass)
}

fn validate_collab_runtime_binding(
    runtime: &CollabRuntime,
    sheet_id: &str,
    tenant_id: &str,
    region: &str,
    cell_id: &str,
) -> Result<(), SheetError> {
    if runtime.surface.value != CollabSurface::Sheets {
        return Err(SheetError::InvalidCollabSurface);
    }
    if runtime.document_id.value != sheet_id
        || runtime.tenant_id.value != tenant_id
        || runtime.region.value != region
        || runtime.cell_id.value != cell_id
    {
        return Err(SheetError::InvalidCollabRuntime);
    }
    Ok(())
}

fn validate_dependencies(
    cells_by_address: &BTreeMap<CellAddress, &SheetCell>,
    dependencies: &[CellDependency],
) -> Result<(), SheetError> {
    let mut edges: BTreeMap<CellAddress, Vec<CellAddress>> = BTreeMap::new();
    for dependency in dependencies {
        dependency.source.validate()?;
        dependency.target.validate()?;
        if dependency.source == dependency.target {
            return Err(SheetError::SelfDependency);
        }
        let Some(target_cell) = cells_by_address.get(&dependency.target) else {
            return Err(SheetError::MissingDependencyEndpoint);
        };
        if !cells_by_address.contains_key(&dependency.source) {
            return Err(SheetError::MissingDependencyEndpoint);
        }
        if !target_cell.is_formula() {
            return Err(SheetError::DependencyTargetNotFormula);
        }
        edges
            .entry(dependency.source.clone())
            .or_default()
            .push(dependency.target.clone());
    }
    reject_cycles(&edges)
}

fn reject_cycles(edges: &BTreeMap<CellAddress, Vec<CellAddress>>) -> Result<(), SheetError> {
    let mut visited = BTreeSet::new();
    let mut visiting = BTreeSet::new();
    for node in edges.keys() {
        visit_node(node, edges, &mut visiting, &mut visited)?;
    }
    Ok(())
}

fn visit_node(
    node: &CellAddress,
    edges: &BTreeMap<CellAddress, Vec<CellAddress>>,
    visiting: &mut BTreeSet<CellAddress>,
    visited: &mut BTreeSet<CellAddress>,
) -> Result<(), SheetError> {
    if visited.contains(node) {
        return Ok(());
    }
    if !visiting.insert(node.clone()) {
        return Err(SheetError::CyclicDependency);
    }
    if let Some(targets) = edges.get(node) {
        for target in targets {
            visit_node(target, edges, visiting, visited)?;
        }
    }
    visiting.remove(node);
    visited.insert(node.clone());
    Ok(())
}

fn validate_formula_shape(
    value_kind: CellValueKind,
    formula: Option<&str>,
) -> Result<(), SheetError> {
    match (value_kind, formula) {
        (CellValueKind::Formula, Some(formula)) => {
            if formula.trim() != formula
                || !formula.starts_with('=')
                || formula.len() == 1
                || formula.chars().any(char::is_control)
            {
                Err(SheetError::InvalidFormula)
            } else {
                Ok(())
            }
        }
        (CellValueKind::Formula, None) => Err(SheetError::MissingFormula),
        (_, Some(_)) => Err(SheetError::UnexpectedFormula),
        (_, None) => Ok(()),
    }
}

fn validate_drive_path(path: &str) -> Result<(), SheetError> {
    if path.trim() != path
        || !path.starts_with('/')
        || path == "/"
        || path.ends_with('/')
        || path.contains("//")
        || path.chars().any(char::is_control)
    {
        return Err(SheetError::InvalidDrivePath);
    }
    if path
        .split('/')
        .skip(1)
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(SheetError::InvalidDrivePath);
    }
    Ok(())
}

fn validate_time_order(created_at: u64, updated_at: u64) -> Result<(), SheetError> {
    if updated_at < created_at {
        Err(SheetError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_non_empty(value: &str, error: SheetError) -> Result<(), SheetError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_collab_runtime_domain::{CollabRuntimeCreate, CollabSnapshotRef, CollabStateVectorRef};
    use oya_data_boundary_kernel::{DataClassification, OperationalDataClass};

    fn runtime() -> CollabRuntime {
        CollabRuntime::new(CollabRuntimeCreate {
            document_id: "sheet-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            surface: CollabSurface::Sheets,
            data_class: None,
            snapshot: CollabSnapshotRef::new(
                "snap-1".into(),
                "tenant-1/sheets/sheet-1/snap-1".into(),
                "sha256:snapshot".into(),
                "sv:1".into(),
                1,
                1,
                128,
            )
            .unwrap(),
            state_vector: CollabStateVectorRef::new("sv:1".into(), 1, 1, 32).unwrap(),
            active_awareness: Vec::new(),
            created_at_epoch_millis: 1_700_000_000_000,
            updated_at_epoch_millis: 1_700_000_010_000,
        })
        .unwrap()
    }

    fn a1() -> CellAddress {
        CellAddress::new("tab-1".into(), 1, 1).unwrap()
    }

    fn b1() -> CellAddress {
        CellAddress::new("tab-1".into(), 2, 1).unwrap()
    }

    fn c1() -> CellAddress {
        CellAddress::new("tab-1".into(), 3, 1).unwrap()
    }

    fn value_cell(address: CellAddress) -> SheetCell {
        SheetCell::new(SheetCellCreate {
            address,
            value_kind: CellValueKind::Number,
            formula: None,
            value_preview: Some("7".into()),
            data_class: None,
        })
        .unwrap()
    }

    fn formula_cell(address: CellAddress, formula: &str) -> SheetCell {
        SheetCell::new(SheetCellCreate {
            address,
            value_kind: CellValueKind::Formula,
            formula: Some(formula.into()),
            value_preview: Some("14".into()),
            data_class: None,
        })
        .unwrap()
    }

    fn cell_graph() -> CellGraph {
        CellGraph::new(
            vec![value_cell(a1()), formula_cell(b1(), "=A1*2")],
            vec![CellDependency::new(a1(), b1()).unwrap()],
        )
        .unwrap()
    }

    fn sheet_input() -> SheetCreate {
        SheetCreate {
            id: "sheet-1".into(),
            title: "Forecast".into(),
            drive_path: "/finance/forecast.oyasheet".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            data_class: None,
            collab_runtime: runtime(),
            cell_graph: cell_graph(),
            indexed_at_epoch_seconds: None,
            created_at_epoch_seconds: 1_700_000_000,
            updated_at_epoch_seconds: 1_700_000_010,
        }
    }

    #[test]
    fn sheet_defaults_to_identifying_and_binds_to_sheet_collab_runtime() {
        let sheet = Sheet::new(sheet_input()).unwrap();

        assert_eq!(
            sheet.privacy_data_class().data_class(),
            DataClass::PiiIdentifying
        );
        assert_eq!(
            sheet.title.data_class,
            DataClassification::Privacy(title_data_class())
        );
        assert_eq!(
            sheet.cell_graph.data_class,
            DataClassification::Privacy(sheet_content_data_class())
        );
        assert_eq!(sheet.schema_version.value, 1);
    }

    #[test]
    fn formula_cells_require_explicit_formula_shape() {
        assert_eq!(
            SheetCell::new(SheetCellCreate {
                address: a1(),
                value_kind: CellValueKind::Formula,
                formula: None,
                value_preview: None,
                data_class: None,
            }),
            Err(SheetError::MissingFormula)
        );
        assert_eq!(
            SheetCell::new(SheetCellCreate {
                address: a1(),
                value_kind: CellValueKind::Number,
                formula: Some("=A1".into()),
                value_preview: None,
                data_class: None,
            }),
            Err(SheetError::UnexpectedFormula)
        );
    }

    #[test]
    fn cell_graph_rejects_missing_endpoints_and_cycles() {
        let missing_endpoint = CellGraph::new(
            vec![formula_cell(b1(), "=A1*2")],
            vec![CellDependency::new(a1(), b1()).unwrap()],
        );
        assert_eq!(missing_endpoint, Err(SheetError::MissingDependencyEndpoint));

        let cyclic = CellGraph::new(
            vec![formula_cell(a1(), "=B1"), formula_cell(b1(), "=A1")],
            vec![
                CellDependency::new(b1(), a1()).unwrap(),
                CellDependency::new(a1(), b1()).unwrap(),
            ],
        );
        assert_eq!(cyclic, Err(SheetError::CyclicDependency));

        let non_formula_target = CellGraph::new(
            vec![value_cell(a1()), value_cell(c1())],
            vec![CellDependency::new(a1(), c1()).unwrap()],
        );
        assert_eq!(
            non_formula_target,
            Err(SheetError::DependencyTargetNotFormula)
        );
    }

    #[test]
    fn what_if_scenario_requires_actor_inputs_and_outputs() {
        let scenario = WhatIfScenario::new(WhatIfScenarioCreate {
            scenario_id: "scenario-1".into(),
            sheet_id: "sheet-1".into(),
            tenant_id: "tenant-1".into(),
            actor_ref: "user:analyst@example.com".into(),
            input_cells: vec![a1()],
            output_cells: vec![b1()],
            created_at_epoch_seconds: 1_700_000_020,
        })
        .unwrap();
        assert_eq!(
            scenario.actor_ref.data_class,
            DataClassification::Privacy(actor_data_class())
        );

        assert_eq!(
            WhatIfScenario::new(WhatIfScenarioCreate {
                scenario_id: "scenario-2".into(),
                sheet_id: "sheet-1".into(),
                tenant_id: "tenant-1".into(),
                actor_ref: "user:analyst@example.com".into(),
                input_cells: Vec::new(),
                output_cells: vec![b1()],
                created_at_epoch_seconds: 1_700_000_020,
            }),
            Err(SheetError::EmptyWhatIfInputs)
        );
    }

    #[test]
    fn legacy_data_class_conversion_rejects_operational_markers() {
        assert_eq!(
            workspace_sheet_data_class_from_legacy(DataClass::Audit),
            Err(SheetError::InvalidDataClass)
        );
        assert_eq!(
            DataClassification::from(OperationalDataClass::Audit).privacy_data_class(),
            None
        );
    }
}

// ---------------------------------------------------------------------------
// M03-P06-IP — workspace.sheets STAGING surface markers (SPEC §4 rows).
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SheetsSurfaceStaging {
    pub sheet_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub yrs_state_vector: Classified<Vec<u8>>, // data_class: INTERNAL_ONLY
}

impl SheetsSurfaceStaging {
    pub fn new(sheet_id: String, tenant_id: String, yrs_state_vector: Vec<u8>) -> Self {
        Self {
            sheet_id: Classified::new(sheet_id, DataClass::InternalOnly),
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            yrs_state_vector: Classified::new(yrs_state_vector, DataClass::InternalOnly),
        }
    }
}

#[cfg(test)]
mod m03_p06_tests {
    use super::*;

    fn sample() -> SheetsSurfaceStaging {
        SheetsSurfaceStaging::new("sheets-1".into(), "sheets-1".into(), vec![])
    }

    #[test]
    fn surface_staging_constructor_sets_internal_only() {
        let s = sample();
        assert_eq!(s.sheet_id.data_class, DataClass::InternalOnly.into());
    }

    #[test]
    fn surface_staging_round_trip_equality() {
        assert_eq!(sample(), sample());
    }
}
