#![forbid(unsafe_code)]
//! Leptos SSR suite shell scaffold with selective hydration planned.
//!
//! Until the Leptos runtime dependency lands behind the dependency policy, this
//! crate stores pure route/render contracts for the Drive shell and editor islands.

use oya_office_drive_api::DriveLaunchTarget;
use oya_office_kernel::{ObjectId, TenantId};

/// Stable application identifier used by workspace and Buck2 scaffold verification.
pub const APP_NAME: &str = "oya-office-web-app";

/// Product vertical slice owned by this deployable.
pub const VERTICAL_SLICE: &str = "web";

/// Source-shaped deployable layer represented by this scaffold.
pub const DEPLOYABLE_LAYER: &str = "leptos-ssr-shell";

/// G075 designer UX/editor collaboration contract version.
pub const G075_DESIGNER_UX_CONTRACT_VERSION: &str = "g075-designer-ux-v1";

/// G080 Drive shell route contract version.
pub const G080_DRIVE_SHELL_ROUTE_CONTRACT_VERSION: &str = "g080-drive-shell-route-v1";

/// G084 Leptos SSR Drive/workspace shell contract version.
pub const G084_LEPTOS_SHELL_CONTRACT_VERSION: &str = "g084-leptos-shell-v1";

/// Official Leptos SSR lifecycle source used by the G084 shell contract.
pub const G084_LEPTOS_SSR_LIFECYCLE_SOURCE: &str = "https://book.leptos.dev/ssr/22_life_cycle.html";

/// Official Leptos islands source used by the G084 selective-hydration contract.
pub const G084_LEPTOS_ISLANDS_SOURCE: &str = "https://book.leptos.dev/islands.html";

/// Web shell validation error.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WebShellError {
    /// Client-only rendering is rejected for the suite shell.
    CsrOnlyHydration,
    /// Whole-page hydration is rejected; editor islands hydrate selectively.
    WholePageHydration,
    /// SSR marker is missing from rendered shell markup.
    MissingSsrMarker,
    /// Selective hydration marker is missing from rendered shell markup.
    MissingSelectiveHydration,
    /// Performance budget has invalid or unsupported values.
    InvalidPerformanceBudget,
    /// Performance observation has invalid values.
    InvalidPerformanceObservation,
    /// Editor interaction panel contract has invalid values.
    InvalidInteractionPanel,
}

impl core::fmt::Display for WebShellError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for WebShellError {}

/// Hydration mode allowed by the SSR shell contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HydrationMode {
    /// SSR shell with selective island hydration.
    SelectiveIslands,
    /// Whole-page hydration, rejected by the Oya Office shell.
    WholePage,
    /// Client-only rendering, rejected by the Oya Office shell.
    ClientOnly,
}

/// Performance-budget evaluation result for the web shell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WebShellPerformanceDecision {
    /// Observation is within the configured budget.
    Pass,
    /// Observation exceeds at least one budget dimension.
    Fail,
}

/// SSR shell performance budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SsrShellPerformanceBudget {
    hydration_mode: HydrationMode,
    max_ssr_render_p50_millis: u64,
    max_ssr_render_p95_millis: u64,
    max_shell_html_bytes: usize,
    max_selective_islands: usize,
}

impl SsrShellPerformanceBudget {
    /// Creates a fail-closed SSR shell performance budget.
    pub const fn new(
        hydration_mode: HydrationMode,
        max_ssr_render_p50_millis: u64,
        max_ssr_render_p95_millis: u64,
        max_shell_html_bytes: usize,
        max_selective_islands: usize,
    ) -> Result<Self, WebShellError> {
        match hydration_mode {
            HydrationMode::ClientOnly => return Err(WebShellError::CsrOnlyHydration),
            HydrationMode::WholePage => return Err(WebShellError::WholePageHydration),
            HydrationMode::SelectiveIslands => {}
        }
        if max_ssr_render_p50_millis == 0
            || max_ssr_render_p95_millis == 0
            || max_ssr_render_p50_millis > max_ssr_render_p95_millis
            || max_shell_html_bytes == 0
            || max_selective_islands == 0
        {
            return Err(WebShellError::InvalidPerformanceBudget);
        }
        Ok(Self {
            hydration_mode,
            max_ssr_render_p50_millis,
            max_ssr_render_p95_millis,
            max_shell_html_bytes,
            max_selective_islands,
        })
    }

    /// Returns the allowed hydration mode.
    #[must_use]
    pub const fn hydration_mode(self) -> HydrationMode {
        self.hydration_mode
    }

    /// Returns the p50 SSR render budget in milliseconds.
    #[must_use]
    pub const fn max_ssr_render_p50_millis(self) -> u64 {
        self.max_ssr_render_p50_millis
    }

    /// Returns the p95 SSR render budget in milliseconds.
    #[must_use]
    pub const fn max_ssr_render_p95_millis(self) -> u64 {
        self.max_ssr_render_p95_millis
    }

    /// Returns the maximum shell HTML payload size in bytes.
    #[must_use]
    pub const fn max_shell_html_bytes(self) -> usize {
        self.max_shell_html_bytes
    }

    /// Returns the maximum number of selective hydration islands.
    #[must_use]
    pub const fn max_selective_islands(self) -> usize {
        self.max_selective_islands
    }

    /// Evaluates an observed shell sample against this budget.
    #[must_use]
    pub fn evaluate(
        self,
        observation: &SsrShellPerformanceObservation,
    ) -> WebShellPerformanceDecision {
        if self.hydration_mode != HydrationMode::SelectiveIslands
            || observation.ssr_render_p50_millis() > self.max_ssr_render_p50_millis
            || observation.ssr_render_p95_millis() > self.max_ssr_render_p95_millis
            || observation.shell_html_bytes() > self.max_shell_html_bytes
            || observation.selective_islands() > self.max_selective_islands
        {
            WebShellPerformanceDecision::Fail
        } else {
            WebShellPerformanceDecision::Pass
        }
    }
}

/// SSR shell performance observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SsrShellPerformanceObservation {
    ssr_render_p50_millis: u64,
    ssr_render_p95_millis: u64,
    shell_html_bytes: usize,
    selective_islands: usize,
}

impl SsrShellPerformanceObservation {
    /// Creates an observed SSR shell performance sample.
    pub const fn new(
        ssr_render_p50_millis: u64,
        ssr_render_p95_millis: u64,
        shell_html_bytes: usize,
        selective_islands: usize,
    ) -> Result<Self, WebShellError> {
        if ssr_render_p50_millis == 0
            || ssr_render_p95_millis == 0
            || ssr_render_p50_millis > ssr_render_p95_millis
            || shell_html_bytes == 0
        {
            return Err(WebShellError::InvalidPerformanceObservation);
        }
        Ok(Self {
            ssr_render_p50_millis,
            ssr_render_p95_millis,
            shell_html_bytes,
            selective_islands,
        })
    }

    /// Creates an observation from rendered shell markup and measured SSR timings.
    pub fn from_shell_html(
        shell_html: &str,
        ssr_render_p50_millis: u64,
        ssr_render_p95_millis: u64,
    ) -> Result<Self, WebShellError> {
        validate_ssr_shell_markup(shell_html)?;
        let selective_islands = shell_html
            .matches("data-hydration-boundary=\"editor\"")
            .count()
            .saturating_add(
                shell_html
                    .matches("data-hydration-boundary=\"drive\"")
                    .count(),
            );
        Self::new(
            ssr_render_p50_millis,
            ssr_render_p95_millis,
            shell_html.as_bytes().len(),
            selective_islands,
        )
    }

    /// Returns observed p50 SSR render time in milliseconds.
    #[must_use]
    pub const fn ssr_render_p50_millis(self) -> u64 {
        self.ssr_render_p50_millis
    }

    /// Returns observed p95 SSR render time in milliseconds.
    #[must_use]
    pub const fn ssr_render_p95_millis(self) -> u64 {
        self.ssr_render_p95_millis
    }

    /// Returns observed shell HTML payload size in bytes.
    #[must_use]
    pub const fn shell_html_bytes(self) -> usize {
        self.shell_html_bytes
    }

    /// Returns observed selective hydration island count.
    #[must_use]
    pub const fn selective_islands(self) -> usize {
        self.selective_islands
    }
}

/// Returns the default SSR shell performance budget for scaffold tests.
#[must_use]
pub fn ssr_shell_performance_budget() -> SsrShellPerformanceBudget {
    match SsrShellPerformanceBudget::new(HydrationMode::SelectiveIslands, 80, 250, 16_384, 4) {
        Ok(budget) => budget,
        Err(_) => unreachable!("static SSR shell performance budget is valid"),
    }
}

/// Web shell route kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WebShellRouteKind {
    /// Drive home.
    DriveHome,
    /// Drive object preview.
    DriveObject,
    /// Docs editor island launched from Drive.
    DocsEditor,
    /// Sheets editor island launched from Drive.
    SheetsEditor,
    /// Slides editor island launched from Drive.
    SlidesEditor,
}

/// Web route contract for SSR shell routing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WebShellRoute {
    kind: WebShellRouteKind,
    path_template: &'static str,
    ssr_required: bool,
}

impl WebShellRoute {
    /// Creates a route descriptor.
    #[must_use]
    pub const fn new(
        kind: WebShellRouteKind,
        path_template: &'static str,
        ssr_required: bool,
    ) -> Self {
        Self {
            kind,
            path_template,
            ssr_required,
        }
    }

    /// Returns route kind.
    #[must_use]
    pub const fn kind(&self) -> WebShellRouteKind {
        self.kind
    }

    /// Returns path template.
    #[must_use]
    pub const fn path_template(&self) -> &'static str {
        self.path_template
    }

    /// Returns true when the route must render through SSR shell.
    #[must_use]
    pub const fn ssr_required(&self) -> bool {
        self.ssr_required
    }
}

/// Tenant/object-aware Drive shell route contract for G080.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DriveShellRouteContract {
    kind: WebShellRouteKind,
    path_template: &'static str,
    launch_target: Option<DriveLaunchTarget>,
    tenant_scoped: bool,
    object_scoped: bool,
    ssr_required: bool,
}

impl DriveShellRouteContract {
    /// Creates a static Drive shell route contract row.
    #[must_use]
    pub const fn new(
        kind: WebShellRouteKind,
        path_template: &'static str,
        launch_target: Option<DriveLaunchTarget>,
        tenant_scoped: bool,
        object_scoped: bool,
        ssr_required: bool,
    ) -> Self {
        Self {
            kind,
            path_template,
            launch_target,
            tenant_scoped,
            object_scoped,
            ssr_required,
        }
    }

    /// Returns route kind.
    #[must_use]
    pub const fn kind(self) -> WebShellRouteKind {
        self.kind
    }

    /// Returns route template.
    #[must_use]
    pub const fn path_template(self) -> &'static str {
        self.path_template
    }

    /// Returns the launch target represented by this route when it opens an editor.
    #[must_use]
    pub const fn launch_target(self) -> Option<DriveLaunchTarget> {
        self.launch_target
    }

    /// Returns true when route includes a tenant axis.
    #[must_use]
    pub const fn is_tenant_scoped(self) -> bool {
        self.tenant_scoped
    }

    /// Returns true when route includes a Drive object axis.
    #[must_use]
    pub const fn is_object_scoped(self) -> bool {
        self.object_scoped
    }

    /// Returns true when route must render through SSR.
    #[must_use]
    pub const fn ssr_required(self) -> bool {
        self.ssr_required
    }
}

/// Returns Drive shell routes.
#[must_use]
pub const fn drive_shell_routes() -> [WebShellRoute; 5] {
    [
        WebShellRoute::new(WebShellRouteKind::DriveHome, "/drive", true),
        WebShellRoute::new(WebShellRouteKind::DriveObject, "/drive/:object_id", true),
        WebShellRoute::new(
            WebShellRouteKind::DocsEditor,
            "/drive/:object_id/docs",
            true,
        ),
        WebShellRoute::new(
            WebShellRouteKind::SheetsEditor,
            "/drive/:object_id/sheets",
            true,
        ),
        WebShellRoute::new(
            WebShellRouteKind::SlidesEditor,
            "/drive/:object_id/slides",
            true,
        ),
    ]
}

/// Returns the G080 Drive shell route contract rows.
#[must_use]
pub const fn drive_shell_route_contracts() -> [DriveShellRouteContract; 5] {
    [
        DriveShellRouteContract::new(
            WebShellRouteKind::DriveHome,
            "/t/:tenant_id/drive",
            None,
            true,
            false,
            true,
        ),
        DriveShellRouteContract::new(
            WebShellRouteKind::DriveObject,
            "/t/:tenant_id/drive/:object_id",
            Some(DriveLaunchTarget::Preview),
            true,
            true,
            true,
        ),
        DriveShellRouteContract::new(
            WebShellRouteKind::DocsEditor,
            "/t/:tenant_id/drive/:object_id/docs",
            Some(DriveLaunchTarget::Docs),
            true,
            true,
            true,
        ),
        DriveShellRouteContract::new(
            WebShellRouteKind::SheetsEditor,
            "/t/:tenant_id/drive/:object_id/sheets",
            Some(DriveLaunchTarget::Sheets),
            true,
            true,
            true,
        ),
        DriveShellRouteContract::new(
            WebShellRouteKind::SlidesEditor,
            "/t/:tenant_id/drive/:object_id/slides",
            Some(DriveLaunchTarget::Slides),
            true,
            true,
            true,
        ),
    ]
}

/// Returns tenant-aware Drive shell routes for the public-SaaS SSR shell.
#[must_use]
pub const fn tenant_aware_drive_shell_routes() -> [WebShellRoute; 5] {
    [
        WebShellRoute::new(WebShellRouteKind::DriveHome, "/t/:tenant_id/drive", true),
        WebShellRoute::new(
            WebShellRouteKind::DriveObject,
            "/t/:tenant_id/drive/:object_id",
            true,
        ),
        WebShellRoute::new(
            WebShellRouteKind::DocsEditor,
            "/t/:tenant_id/drive/:object_id/docs",
            true,
        ),
        WebShellRoute::new(
            WebShellRouteKind::SheetsEditor,
            "/t/:tenant_id/drive/:object_id/sheets",
            true,
        ),
        WebShellRoute::new(
            WebShellRouteKind::SlidesEditor,
            "/t/:tenant_id/drive/:object_id/slides",
            true,
        ),
    ]
}

/// Maps Drive launch target to SSR route kind.
#[must_use]
pub const fn route_kind_for_launch(target: DriveLaunchTarget) -> WebShellRouteKind {
    match target {
        DriveLaunchTarget::Docs => WebShellRouteKind::DocsEditor,
        DriveLaunchTarget::Sheets => WebShellRouteKind::SheetsEditor,
        DriveLaunchTarget::Slides => WebShellRouteKind::SlidesEditor,
        DriveLaunchTarget::Preview => WebShellRouteKind::DriveObject,
    }
}

/// Selective-hydration editor island kind inside the Drive workspace shell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EditorIslandKind {
    /// Drive file list/navigation island.
    DriveList,
    /// Drive preview island for folders and binary files.
    DrivePreview,
    /// Docs editor island.
    DocsEditor,
    /// Sheets editor island.
    SheetsEditor,
    /// Slides editor island.
    SlidesEditor,
}

impl EditorIslandKind {
    /// Returns the stable island label emitted into SSR data attributes.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DriveList => "drive-list",
            Self::DrivePreview => "preview",
            Self::DocsEditor => "docs",
            Self::SheetsEditor => "sheets",
            Self::SlidesEditor => "slides",
        }
    }
}

/// SSR-visible Drive workspace shell region for G084.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DriveWorkspaceShellRegionKind {
    /// Product chrome and suite identity.
    SuiteBanner,
    /// Drive navigation and object switching.
    DriveNavigation,
    /// Selected Drive object context.
    ObjectContext,
    /// Docs editor island placeholder.
    DocsEditorIsland,
    /// Sheets editor island placeholder.
    SheetsEditorIsland,
    /// Slides editor island placeholder.
    SlidesEditorIsland,
    /// Collaboration controls shared by editor islands.
    CollaborationControls,
    /// Hydration and test manifest emitted with SSR markup.
    HydrationManifest,
}

impl DriveWorkspaceShellRegionKind {
    /// Returns the stable region label emitted in SSR markup.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SuiteBanner => "suite-banner",
            Self::DriveNavigation => "drive-navigation",
            Self::ObjectContext => "object-context",
            Self::DocsEditorIsland => "docs-editor-island",
            Self::SheetsEditorIsland => "sheets-editor-island",
            Self::SlidesEditorIsland => "slides-editor-island",
            Self::CollaborationControls => "collaboration-controls",
            Self::HydrationManifest => "hydration-manifest",
        }
    }

    /// Returns the accessible label for the SSR-visible region.
    #[must_use]
    pub const fn aria_label(self) -> &'static str {
        match self {
            Self::SuiteBanner => "Oya Office workspace",
            Self::DriveNavigation => "Drive workspace",
            Self::ObjectContext => "Drive object context",
            Self::DocsEditorIsland => "Docs editor island",
            Self::SheetsEditorIsland => "Sheets editor island",
            Self::SlidesEditorIsland => "Slides editor island",
            Self::CollaborationControls => "Collaboration interactions",
            Self::HydrationManifest => "Hydration test manifest",
        }
    }

    /// Returns the selective-hydration boundary represented by the region.
    #[must_use]
    pub const fn hydration_boundary(self) -> &'static str {
        match self {
            Self::SuiteBanner => "static",
            Self::DriveNavigation => "navigation",
            Self::ObjectContext => "drive",
            Self::DocsEditorIsland | Self::SheetsEditorIsland | Self::SlidesEditorIsland => {
                "editor"
            }
            Self::CollaborationControls => "interaction",
            Self::HydrationManifest => "manifest",
        }
    }

    /// Returns the editor island represented by this shell region, when any.
    #[must_use]
    pub const fn editor_island(self) -> Option<EditorIslandKind> {
        match self {
            Self::DocsEditorIsland => Some(EditorIslandKind::DocsEditor),
            Self::SheetsEditorIsland => Some(EditorIslandKind::SheetsEditor),
            Self::SlidesEditorIsland => Some(EditorIslandKind::SlidesEditor),
            _ => None,
        }
    }
}

/// SSR-visible Drive workspace region contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DriveWorkspaceShellRegion {
    kind: DriveWorkspaceShellRegionKind,
    ssr_visible: bool,
    drive_bound: bool,
    keyboard_reachable: bool,
}

impl DriveWorkspaceShellRegion {
    /// Creates a static Drive workspace shell region.
    #[must_use]
    pub const fn new(
        kind: DriveWorkspaceShellRegionKind,
        ssr_visible: bool,
        drive_bound: bool,
        keyboard_reachable: bool,
    ) -> Self {
        Self {
            kind,
            ssr_visible,
            drive_bound,
            keyboard_reachable,
        }
    }

    /// Returns the region kind.
    #[must_use]
    pub const fn kind(self) -> DriveWorkspaceShellRegionKind {
        self.kind
    }

    /// Returns true when the region is emitted in server-rendered HTML.
    #[must_use]
    pub const fn is_ssr_visible(self) -> bool {
        self.ssr_visible
    }

    /// Returns true when the region remains scoped to the selected Drive object.
    #[must_use]
    pub const fn is_drive_bound(self) -> bool {
        self.drive_bound
    }

    /// Returns true when keyboard users can reach or skip to the region.
    #[must_use]
    pub const fn is_keyboard_reachable(self) -> bool {
        self.keyboard_reachable
    }

    /// Returns the selective hydration boundary for the region.
    #[must_use]
    pub const fn hydration_boundary(self) -> &'static str {
        self.kind.hydration_boundary()
    }

    /// Returns the stable SSR markup label for the region.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.kind.as_str()
    }

    /// Returns the accessible label for the region.
    #[must_use]
    pub const fn aria_label(self) -> &'static str {
        self.kind.aria_label()
    }
}

/// G084 hydration gate kind for SSR shell tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HydrationTestGateKind {
    /// Server build must compile the shared app with the `ssr` feature.
    SsrServerFeature,
    /// Browser build must hydrate the server-rendered shell with the `hydrate` feature.
    HydrateBrowserFeature,
    /// Server and browser route trees must match for tenant/object routes.
    SharedRouteTree,
    /// Hydration must pick up existing DOM instead of replacing it.
    ExistingDomPickup,
    /// Interactive editor work is isolated to explicit islands.
    IslandBoundary,
    /// Client-only shell fallback is forbidden.
    NoCsrOnlyFallback,
    /// Whole-page hydration is forbidden for the Drive workspace shell.
    NoWholePageHydration,
    /// Keyboard and text affordances must exist before hydration.
    AccessibilityBeforeHydration,
    /// SSR timing, payload, and island count stay within scaffold budget.
    PerformanceBudget,
}

impl HydrationTestGateKind {
    /// Returns the stable gate label emitted in manifests and tests.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SsrServerFeature => "ssr-server-feature",
            Self::HydrateBrowserFeature => "hydrate-browser-feature",
            Self::SharedRouteTree => "shared-route-tree",
            Self::ExistingDomPickup => "existing-dom-pickup",
            Self::IslandBoundary => "island-boundary",
            Self::NoCsrOnlyFallback => "no-csr-only-fallback",
            Self::NoWholePageHydration => "no-whole-page-hydration",
            Self::AccessibilityBeforeHydration => "accessibility-before-hydration",
            Self::PerformanceBudget => "performance-budget",
        }
    }
}

/// Launch-blocking hydration test gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HydrationTestGate {
    kind: HydrationTestGateKind,
    launch_blocking: bool,
    evidence: &'static str,
}

impl HydrationTestGate {
    /// Creates a static hydration gate row.
    #[must_use]
    pub const fn new(
        kind: HydrationTestGateKind,
        launch_blocking: bool,
        evidence: &'static str,
    ) -> Self {
        Self {
            kind,
            launch_blocking,
            evidence,
        }
    }

    /// Returns the gate kind.
    #[must_use]
    pub const fn kind(self) -> HydrationTestGateKind {
        self.kind
    }

    /// Returns true when the gate blocks launch until green.
    #[must_use]
    pub const fn is_launch_blocking(self) -> bool {
        self.launch_blocking
    }

    /// Returns the required evidence for the gate.
    #[must_use]
    pub const fn evidence(self) -> &'static str {
        self.evidence
    }
}

/// Returns the Leptos feature flags represented by the G084 shell contract.
#[must_use]
pub const fn g084_required_leptos_feature_flags() -> [&'static str; 2] {
    ["ssr", "hydrate"]
}

/// Returns G084 SSR-visible Drive workspace shell regions.
#[must_use]
pub const fn g084_drive_workspace_shell_regions() -> [DriveWorkspaceShellRegion; 8] {
    [
        DriveWorkspaceShellRegion::new(
            DriveWorkspaceShellRegionKind::SuiteBanner,
            true,
            false,
            true,
        ),
        DriveWorkspaceShellRegion::new(
            DriveWorkspaceShellRegionKind::DriveNavigation,
            true,
            true,
            true,
        ),
        DriveWorkspaceShellRegion::new(
            DriveWorkspaceShellRegionKind::ObjectContext,
            true,
            true,
            true,
        ),
        DriveWorkspaceShellRegion::new(
            DriveWorkspaceShellRegionKind::DocsEditorIsland,
            true,
            true,
            true,
        ),
        DriveWorkspaceShellRegion::new(
            DriveWorkspaceShellRegionKind::SheetsEditorIsland,
            true,
            true,
            true,
        ),
        DriveWorkspaceShellRegion::new(
            DriveWorkspaceShellRegionKind::SlidesEditorIsland,
            true,
            true,
            true,
        ),
        DriveWorkspaceShellRegion::new(
            DriveWorkspaceShellRegionKind::CollaborationControls,
            true,
            true,
            true,
        ),
        DriveWorkspaceShellRegion::new(
            DriveWorkspaceShellRegionKind::HydrationManifest,
            true,
            false,
            true,
        ),
    ]
}

/// Returns launch-blocking G084 hydration test gates.
#[must_use]
pub const fn g084_hydration_test_gates() -> [HydrationTestGate; 9] {
    [
        HydrationTestGate::new(
            HydrationTestGateKind::SsrServerFeature,
            true,
            "server build target uses Leptos ssr feature",
        ),
        HydrationTestGate::new(
            HydrationTestGateKind::HydrateBrowserFeature,
            true,
            "browser build target uses Leptos hydrate feature",
        ),
        HydrationTestGate::new(
            HydrationTestGateKind::SharedRouteTree,
            true,
            "tenant/object routes are identical on server and browser",
        ),
        HydrationTestGate::new(
            HydrationTestGateKind::ExistingDomPickup,
            true,
            "hydration picks up existing server-rendered DOM",
        ),
        HydrationTestGate::new(
            HydrationTestGateKind::IslandBoundary,
            true,
            "Docs/Sheets/Slides hydrate only in explicit editor islands",
        ),
        HydrationTestGate::new(
            HydrationTestGateKind::NoCsrOnlyFallback,
            true,
            "client-only shell fallback is rejected",
        ),
        HydrationTestGate::new(
            HydrationTestGateKind::NoWholePageHydration,
            true,
            "whole-page hydration is rejected",
        ),
        HydrationTestGate::new(
            HydrationTestGateKind::AccessibilityBeforeHydration,
            true,
            "landmarks, labels, skip link, and text status exist before hydration",
        ),
        HydrationTestGate::new(
            HydrationTestGateKind::PerformanceBudget,
            true,
            "SSR p95, shell bytes, and island count satisfy scaffold budget",
        ),
    ]
}

/// Maps a Drive launch target to an SSR editor island placeholder.
#[must_use]
pub const fn editor_island_for_launch_target(target: DriveLaunchTarget) -> EditorIslandKind {
    match target {
        DriveLaunchTarget::Docs => EditorIslandKind::DocsEditor,
        DriveLaunchTarget::Sheets => EditorIslandKind::SheetsEditor,
        DriveLaunchTarget::Slides => EditorIslandKind::SlidesEditor,
        DriveLaunchTarget::Preview => EditorIslandKind::DrivePreview,
    }
}

/// Collaboration status surfaced by editor interaction panels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CollaborationStatusKind {
    /// Collaboration is offline and cannot accept realtime edits.
    Offline,
    /// Collaboration is connecting before a live session is established.
    Connecting,
    /// Collaboration is live for presence/editing.
    Live,
    /// Local changes are being saved.
    Saving,
    /// Local changes are durably saved.
    Saved,
    /// A version or edit conflict needs visible recovery.
    Conflict,
    /// Collaboration is reconnecting after an interruption.
    Reconnecting,
}

impl CollaborationStatusKind {
    /// Returns the stable status label emitted into SSR data attributes.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Offline => "offline",
            Self::Connecting => "connecting",
            Self::Live => "live",
            Self::Saving => "saving",
            Self::Saved => "saved",
            Self::Conflict => "conflict",
            Self::Reconnecting => "reconnecting",
        }
    }
}

/// UX panel kind for Drive-bound editor collaboration interactions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EditorInteractionPanelKind {
    /// Presence and active collaborator avatars/list.
    Presence,
    /// Comment thread entry point.
    Comments,
    /// Suggested edits / review changes entry point.
    Suggestions,
    /// Version history entry point.
    VersionHistory,
    /// Share and permissions entry point.
    Share,
    /// Save-state indicator and last-save affordance.
    SaveState,
    /// Conflict/recovery/reconnect affordance.
    Recovery,
}

impl EditorInteractionPanelKind {
    /// Returns the stable panel label emitted into SSR data attributes.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Presence => "presence",
            Self::Comments => "comments",
            Self::Suggestions => "suggestions",
            Self::VersionHistory => "version-history",
            Self::Share => "share",
            Self::SaveState => "save-state",
            Self::Recovery => "recovery",
        }
    }

    /// Returns the accessible label for the panel group.
    #[must_use]
    pub const fn aria_label(self) -> &'static str {
        match self {
            Self::Presence => "Collaborator presence",
            Self::Comments => "Document comments",
            Self::Suggestions => "Suggested edits",
            Self::VersionHistory => "Version history",
            Self::Share => "Share and permissions",
            Self::SaveState => "Save status",
            Self::Recovery => "Conflict recovery",
        }
    }

    /// Returns the default status visible in the SSR shell before hydration.
    #[must_use]
    pub const fn default_status(self) -> CollaborationStatusKind {
        match self {
            Self::Presence => CollaborationStatusKind::Live,
            Self::Comments | Self::Suggestions | Self::VersionHistory | Self::Share => {
                CollaborationStatusKind::Saved
            }
            Self::SaveState => CollaborationStatusKind::Saving,
            Self::Recovery => CollaborationStatusKind::Reconnecting,
        }
    }

    /// Returns the keyboard focus order within the collaboration interaction region.
    #[must_use]
    pub const fn focus_order(self) -> u8 {
        match self {
            Self::Presence => 1,
            Self::Comments => 2,
            Self::Suggestions => 3,
            Self::VersionHistory => 4,
            Self::Share => 5,
            Self::SaveState => 6,
            Self::Recovery => 7,
        }
    }

    /// Returns the hydration-boundary label for collaboration interaction islands.
    #[must_use]
    pub const fn hydration_boundary(self) -> &'static str {
        "interaction"
    }
}

/// Drive-bound collaboration interaction panel rendered by the SSR shell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EditorInteractionPanel {
    kind: EditorInteractionPanelKind,
    editor_island: EditorIslandKind,
    collaboration_status: CollaborationStatusKind,
    drive_bound: bool,
    focus_order: u8,
}

impl EditorInteractionPanel {
    /// Creates a validated editor interaction panel.
    pub const fn new(
        kind: EditorInteractionPanelKind,
        editor_island: EditorIslandKind,
        collaboration_status: CollaborationStatusKind,
        drive_bound: bool,
        focus_order: u8,
    ) -> Result<Self, WebShellError> {
        if focus_order == 0 || !drive_bound {
            return Err(WebShellError::InvalidInteractionPanel);
        }
        Ok(Self {
            kind,
            editor_island,
            collaboration_status,
            drive_bound,
            focus_order,
        })
    }

    /// Returns the panel kind.
    #[must_use]
    pub const fn kind(self) -> EditorInteractionPanelKind {
        self.kind
    }

    /// Returns the editor island that owns this interaction panel.
    #[must_use]
    pub const fn editor_island(self) -> EditorIslandKind {
        self.editor_island
    }

    /// Returns the collaboration status emitted by the SSR shell.
    #[must_use]
    pub const fn collaboration_status(self) -> CollaborationStatusKind {
        self.collaboration_status
    }

    /// Returns true when the panel remains bound to a Drive object.
    #[must_use]
    pub const fn is_drive_bound(self) -> bool {
        self.drive_bound
    }

    /// Returns keyboard focus order within the interaction region.
    #[must_use]
    pub const fn focus_order(self) -> u8 {
        self.focus_order
    }

    /// Returns the accessible label for the panel group.
    #[must_use]
    pub const fn aria_label(self) -> &'static str {
        self.kind.aria_label()
    }

    /// Returns the selective hydration boundary label for this interaction panel.
    #[must_use]
    pub const fn hydration_boundary(self) -> &'static str {
        self.kind.hydration_boundary()
    }

    /// Returns true because status changes must be exposed as text, not color alone.
    #[must_use]
    pub const fn requires_text_status(self) -> bool {
        true
    }
}

fn checked_editor_interaction_panel(
    kind: EditorInteractionPanelKind,
    editor_island: EditorIslandKind,
) -> EditorInteractionPanel {
    EditorInteractionPanel::new(
        kind,
        editor_island,
        kind.default_status(),
        true,
        kind.focus_order(),
    )
    .expect("static editor interaction panel is valid")
}

/// Returns Drive-bound collaboration panels for one editor launch target.
#[must_use]
pub fn editor_interaction_panels(target: DriveLaunchTarget) -> [EditorInteractionPanel; 7] {
    let editor_island = editor_island_for_launch_target(target);
    [
        checked_editor_interaction_panel(EditorInteractionPanelKind::Presence, editor_island),
        checked_editor_interaction_panel(EditorInteractionPanelKind::Comments, editor_island),
        checked_editor_interaction_panel(EditorInteractionPanelKind::Suggestions, editor_island),
        checked_editor_interaction_panel(EditorInteractionPanelKind::VersionHistory, editor_island),
        checked_editor_interaction_panel(EditorInteractionPanelKind::Share, editor_island),
        checked_editor_interaction_panel(EditorInteractionPanelKind::SaveState, editor_island),
        checked_editor_interaction_panel(EditorInteractionPanelKind::Recovery, editor_island),
    ]
}

/// Renders the SSR-visible collaboration interaction contract for one editor target.
#[must_use]
pub fn render_editor_interaction_contract(target: DriveLaunchTarget) -> String {
    editor_interaction_panels(target)
        .iter()
        .map(|panel| {
            format!(
                concat!(
                    "<section role=\"group\" tabindex=\"-1\" ",
                    "data-ux-contract=\"{}\" data-editor-island=\"{}\" ",
                    "data-collab-panel=\"{}\" data-collab-status=\"{}\" ",
                    "data-drive-bound=\"{}\" data-focus-order=\"{}\" ",
                    "data-hydration-boundary=\"{}\" data-text-status-required=\"{}\" ",
                    "aria-label=\"{}\">",
                    "<span data-status-text=\"{}\">{}</span>",
                    "</section>"
                ),
                G075_DESIGNER_UX_CONTRACT_VERSION,
                panel.editor_island().as_str(),
                panel.kind().as_str(),
                panel.collaboration_status().as_str(),
                panel.is_drive_bound(),
                panel.focus_order(),
                panel.hydration_boundary(),
                panel.requires_text_status(),
                panel.aria_label(),
                panel.collaboration_status().as_str(),
                panel.collaboration_status().as_str()
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

fn render_g084_region_probe(region: DriveWorkspaceShellRegion) -> String {
    format!(
        concat!(
            "<span data-shell-region-probe=\"{}\" data-region-boundary=\"{}\" ",
            "data-ssr-visible=\"{}\" data-drive-bound=\"{}\" ",
            "data-keyboard-reachable=\"{}\" aria-label=\"{}\"></span>"
        ),
        region.as_str(),
        region.hydration_boundary(),
        region.is_ssr_visible(),
        region.is_drive_bound(),
        region.is_keyboard_reachable(),
        region.aria_label()
    )
}

fn render_g084_hydration_gate(gate: HydrationTestGate) -> String {
    format!(
        "<span data-hydration-gate=\"{}\" data-launch-blocking=\"{}\">{}</span>",
        gate.kind().as_str(),
        gate.is_launch_blocking(),
        gate.evidence()
    )
}

/// Drive workspace navigation item rendered by the SSR shell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveWorkspaceNavigationItem {
    label: &'static str,
    target: DriveLaunchTarget,
    editor_island: EditorIslandKind,
    path: String,
    drive_bound: bool,
}

impl DriveWorkspaceNavigationItem {
    /// Creates a Drive-bound workspace navigation item.
    #[must_use]
    pub fn new(
        label: &'static str,
        target: DriveLaunchTarget,
        editor_island: EditorIslandKind,
        path: String,
    ) -> Self {
        Self {
            label,
            target,
            editor_island,
            path,
            drive_bound: true,
        }
    }

    /// Returns the visible navigation label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        self.label
    }

    /// Returns the Drive launch target represented by the item.
    #[must_use]
    pub const fn target(&self) -> DriveLaunchTarget {
        self.target
    }

    /// Returns the editor island hydrated by the item.
    #[must_use]
    pub const fn editor_island(&self) -> EditorIslandKind {
        self.editor_island
    }

    /// Returns the tenant/object-aware route path.
    #[must_use]
    pub fn path(&self) -> &str {
        self.path.as_str()
    }

    /// Returns true when the navigation item stays bound to a Drive object.
    #[must_use]
    pub const fn is_drive_bound(&self) -> bool {
        self.drive_bound
    }
}

/// Returns Drive-bound Docs/Sheets/Slides/Preview navigation for one object.
#[must_use]
pub fn drive_workspace_navigation(
    tenant_id: &TenantId,
    object_id: &ObjectId,
) -> [DriveWorkspaceNavigationItem; 4] {
    let base_path = format!("/t/{}/drive/{}", tenant_id.as_str(), object_id.as_str());
    [
        DriveWorkspaceNavigationItem::new(
            "Docs",
            DriveLaunchTarget::Docs,
            EditorIslandKind::DocsEditor,
            format!("{base_path}/docs"),
        ),
        DriveWorkspaceNavigationItem::new(
            "Sheets",
            DriveLaunchTarget::Sheets,
            EditorIslandKind::SheetsEditor,
            format!("{base_path}/sheets"),
        ),
        DriveWorkspaceNavigationItem::new(
            "Slides",
            DriveLaunchTarget::Slides,
            EditorIslandKind::SlidesEditor,
            format!("{base_path}/slides"),
        ),
        DriveWorkspaceNavigationItem::new(
            "Preview",
            DriveLaunchTarget::Preview,
            EditorIslandKind::DrivePreview,
            base_path,
        ),
    ]
}

/// Renders a minimal SSR shell placeholder with selective editor islands.
#[must_use]
pub fn render_drive_shell_placeholder(object_id: Option<&ObjectId>) -> String {
    let object = object_id.map_or("none", ObjectId::as_str);
    format!(
        "<main data-ssr=\"leptos\" data-shell=\"drive\" data-object-id=\"{object}\"><section data-island=\"drive-list\"></section><section data-island=\"editor-launch\"></section></main>"
    )
}

/// Renders a tenant-aware Drive workspace SSR shell with selective islands.
#[must_use]
pub fn render_drive_workspace_shell(tenant_id: &TenantId, object_id: Option<&ObjectId>) -> String {
    let object = object_id.map_or("none", ObjectId::as_str);
    let navigation = object_id.map_or_else(String::new, |object_id| {
        drive_workspace_navigation(tenant_id, object_id)
            .iter()
            .map(|item| {
                format!(
                    "<a data-drive-bound=\"{}\" data-editor-island=\"{}\" href=\"{}\">{}</a>",
                    item.is_drive_bound(),
                    item.editor_island().as_str(),
                    item.path(),
                    item.label()
                )
            })
            .collect::<Vec<_>>()
            .join("")
    });
    let collaboration_interactions = object_id.map_or_else(String::new, |_| {
        [
            DriveLaunchTarget::Docs,
            DriveLaunchTarget::Sheets,
            DriveLaunchTarget::Slides,
        ]
        .iter()
        .map(|target| render_editor_interaction_contract(*target))
        .collect::<Vec<_>>()
        .join("")
    });
    format!(
        concat!(
            "<main data-ssr=\"leptos\" data-hydration=\"selective\" data-shell=\"drive-workspace\" ",
            "data-drive-contract=\"{}\" data-tenant-id=\"{}\" data-object-id=\"{}\">",
            "<nav aria-label=\"Drive workspace\">{}</nav>",
            "<section data-island=\"drive-list\" data-hydration-boundary=\"drive\"></section>",
            "<section data-editor-island=\"docs\" data-hydration-boundary=\"editor\"></section>",
            "<section data-editor-island=\"sheets\" data-hydration-boundary=\"editor\"></section>",
            "<section data-editor-island=\"slides\" data-hydration-boundary=\"editor\"></section>",
            "<aside aria-label=\"Collaboration interactions\" data-ux-contract=\"{}\">{}</aside>",
            "</main>"
        ),
        G080_DRIVE_SHELL_ROUTE_CONTRACT_VERSION,
        tenant_id.as_str(),
        object,
        navigation,
        G075_DESIGNER_UX_CONTRACT_VERSION,
        collaboration_interactions
    )
}

/// Renders the G084 SSR Drive/workspace shell scaffold and hydration manifest.
///
/// Source-driven contract:
/// - Leptos documents separate server `ssr` and browser `hydrate` build targets:
///   <https://book.leptos.dev/ssr/22_life_cycle.html>
/// - Leptos islands keep only island code interactive in the browser:
///   <https://book.leptos.dev/islands.html>
#[must_use]
pub fn render_g084_drive_workspace_shell(
    tenant_id: &TenantId,
    object_id: Option<&ObjectId>,
) -> String {
    let object = object_id.map_or("none", ObjectId::as_str);
    let navigation = object_id.map_or_else(String::new, |object_id| {
        drive_workspace_navigation(tenant_id, object_id)
            .iter()
            .map(|item| {
                format!(
                    "<a data-drive-bound=\"{}\" data-editor-island=\"{}\" href=\"{}\">{}</a>",
                    item.is_drive_bound(),
                    item.editor_island().as_str(),
                    item.path(),
                    item.label()
                )
            })
            .collect::<Vec<_>>()
            .join("")
    });
    let region_probes = g084_drive_workspace_shell_regions()
        .iter()
        .map(|region| render_g084_region_probe(*region))
        .collect::<Vec<_>>()
        .join("");
    let hydration_gates = g084_hydration_test_gates()
        .iter()
        .map(|gate| render_g084_hydration_gate(*gate))
        .collect::<Vec<_>>()
        .join("");
    let collaboration_interactions = object_id.map_or_else(String::new, |_| {
        [
            DriveLaunchTarget::Docs,
            DriveLaunchTarget::Sheets,
            DriveLaunchTarget::Slides,
        ]
        .iter()
        .map(|target| render_editor_interaction_contract(*target))
        .collect::<Vec<_>>()
        .join("")
    });

    format!(
        concat!(
            "<main data-ssr=\"leptos\" data-hydration=\"selective\" data-shell=\"drive-workspace\" ",
            "data-shell-contract=\"{}\" data-leptos-ssr-feature=\"{}\" ",
            "data-leptos-hydrate-feature=\"{}\" data-tenant-id=\"{}\" data-object-id=\"{}\">",
            "<a href=\"#drive-workspace-content\" data-skip-link=\"true\">Skip to Drive workspace</a>",
            "<header role=\"banner\" aria-label=\"Oya Office workspace\" ",
            "data-shell-region=\"suite-banner\" data-hydration-boundary=\"static\">Oya Office</header>",
            "<nav aria-label=\"Drive workspace\" data-shell-region=\"drive-navigation\" ",
            "data-hydration-boundary=\"navigation\">{}</nav>",
            "<section id=\"drive-workspace-content\" aria-label=\"Drive object context\" ",
            "data-shell-region=\"object-context\" data-hydration-boundary=\"drive\" tabindex=\"-1\">",
            "<section aria-label=\"Docs editor island\" data-shell-region=\"docs-editor-island\" ",
            "data-editor-island=\"docs\" data-hydration-boundary=\"editor\" data-dom-pickup=\"preserve\"></section>",
            "<section aria-label=\"Sheets editor island\" data-shell-region=\"sheets-editor-island\" ",
            "data-editor-island=\"sheets\" data-hydration-boundary=\"editor\" data-dom-pickup=\"preserve\"></section>",
            "<section aria-label=\"Slides editor island\" data-shell-region=\"slides-editor-island\" ",
            "data-editor-island=\"slides\" data-hydration-boundary=\"editor\" data-dom-pickup=\"preserve\"></section>",
            "</section>",
            "<aside aria-label=\"Collaboration interactions\" data-shell-region=\"collaboration-controls\" ",
            "data-hydration-boundary=\"interaction\" data-ux-contract=\"{}\">{}</aside>",
            "<section aria-label=\"Hydration test manifest\" data-shell-region=\"hydration-manifest\" ",
            "data-hydration-boundary=\"manifest\" data-ssr-source=\"{}\" data-islands-source=\"{}\">{}{}</section>",
            "</main>"
        ),
        G084_LEPTOS_SHELL_CONTRACT_VERSION,
        g084_required_leptos_feature_flags()[0],
        g084_required_leptos_feature_flags()[1],
        tenant_id.as_str(),
        object,
        navigation,
        G075_DESIGNER_UX_CONTRACT_VERSION,
        collaboration_interactions,
        G084_LEPTOS_SSR_LIFECYCLE_SOURCE,
        G084_LEPTOS_ISLANDS_SOURCE,
        region_probes,
        hydration_gates
    )
}

/// Validates that shell markup keeps SSR plus selective hydration markers.
pub fn validate_ssr_shell_markup(shell_html: &str) -> Result<(), WebShellError> {
    if shell_html.contains("data-hydration=\"csr-only\"")
        || shell_html.contains("data-hydration=\"client-only\"")
        || shell_html.contains("data-rendering=\"csr-only\"")
        || shell_html.contains("data-rendering=\"client-only\"")
        || shell_html.contains("data-shell=\"client-only\"")
    {
        return Err(WebShellError::CsrOnlyHydration);
    }
    if shell_html.contains("data-hydration=\"whole-page\"")
        || shell_html.contains("data-rendering=\"whole-page\"")
    {
        return Err(WebShellError::WholePageHydration);
    }
    if !shell_html.contains("data-ssr=\"leptos\"") {
        return Err(WebShellError::MissingSsrMarker);
    }
    if !shell_html.contains("data-hydration=\"selective\"") {
        return Err(WebShellError::MissingSelectiveHydration);
    }
    Ok(())
}

/// Starts the scaffolded application entrypoint.
pub fn run() {}

#[cfg(test)]
mod tests {
    use oya_office_drive_api::DriveLaunchTarget;
    use oya_office_kernel::{ObjectId, TenantId};

    use super::{
        APP_NAME, CollaborationStatusKind, DEPLOYABLE_LAYER, DriveWorkspaceNavigationItem,
        DriveWorkspaceShellRegionKind, EditorInteractionPanel, EditorInteractionPanelKind,
        EditorIslandKind, G075_DESIGNER_UX_CONTRACT_VERSION,
        G080_DRIVE_SHELL_ROUTE_CONTRACT_VERSION, G084_LEPTOS_ISLANDS_SOURCE,
        G084_LEPTOS_SHELL_CONTRACT_VERSION, G084_LEPTOS_SSR_LIFECYCLE_SOURCE, HydrationMode,
        HydrationTestGateKind, SsrShellPerformanceObservation, VERTICAL_SLICE, WebShellError,
        WebShellPerformanceDecision, WebShellRouteKind, drive_shell_route_contracts,
        drive_shell_routes, drive_workspace_navigation, editor_interaction_panels,
        editor_island_for_launch_target, g084_drive_workspace_shell_regions,
        g084_hydration_test_gates, g084_required_leptos_feature_flags,
        render_drive_shell_placeholder, render_drive_workspace_shell,
        render_editor_interaction_contract, render_g084_drive_workspace_shell,
        route_kind_for_launch, ssr_shell_performance_budget, tenant_aware_drive_shell_routes,
        validate_ssr_shell_markup,
    };

    #[test]
    fn scaffold_identity_is_declared() {
        assert!(!APP_NAME.is_empty());
        assert!(!DEPLOYABLE_LAYER.is_empty());
        assert!(!VERTICAL_SLICE.is_empty());
    }

    #[test]
    fn drive_shell_routes_are_ssr_required() {
        let routes = drive_shell_routes();
        assert_eq!(routes[0].path_template(), "/drive");
        assert!(routes.iter().all(super::WebShellRoute::ssr_required));
    }

    #[test]
    fn drive_launch_targets_map_to_editor_routes() {
        assert_eq!(
            route_kind_for_launch(DriveLaunchTarget::Sheets),
            WebShellRouteKind::SheetsEditor
        );
    }

    #[test]
    fn drive_shell_placeholder_marks_leptos_ssr_and_islands() {
        let object_id = ObjectId::new("drive-object-1").expect("valid object id");
        let shell = render_drive_shell_placeholder(Some(&object_id));
        assert!(shell.contains("data-ssr=\"leptos\""));
        assert!(shell.contains("data-island=\"editor-launch\""));
        assert!(shell.contains("drive-object-1"));
    }

    #[test]
    fn tenant_aware_routes_are_ssr_required_and_include_tenant_axis() {
        let routes = tenant_aware_drive_shell_routes();

        assert_eq!(routes.len(), 5);
        assert!(routes.iter().all(super::WebShellRoute::ssr_required));
        assert!(
            routes
                .iter()
                .all(|route| route.path_template().contains(":tenant_id"))
        );
        assert_eq!(routes[0].path_template(), "/t/:tenant_id/drive");
    }

    #[test]
    fn drive_workspace_navigation_is_drive_bound_and_covers_editor_surfaces() {
        let tenant_id = TenantId::new("tenant-ui").expect("valid tenant id");
        let object_id = ObjectId::new("object-ui").expect("valid object id");

        let nav = drive_workspace_navigation(&tenant_id, &object_id);

        assert_eq!(nav.len(), 4);
        assert!(nav.iter().all(DriveWorkspaceNavigationItem::is_drive_bound));
        assert!(nav.iter().any(|item| item.label() == "Docs"
            && item.path() == "/t/tenant-ui/drive/object-ui/docs"
            && item.editor_island() == EditorIslandKind::DocsEditor));
        assert!(nav.iter().any(|item| item.label() == "Sheets"
            && item.path() == "/t/tenant-ui/drive/object-ui/sheets"
            && item.editor_island() == EditorIslandKind::SheetsEditor));
        assert!(nav.iter().any(|item| item.label() == "Slides"
            && item.path() == "/t/tenant-ui/drive/object-ui/slides"
            && item.editor_island() == EditorIslandKind::SlidesEditor));
    }

    #[test]
    fn launch_targets_select_editor_island_placeholders() {
        assert_eq!(
            editor_island_for_launch_target(DriveLaunchTarget::Docs),
            EditorIslandKind::DocsEditor
        );
        assert_eq!(
            editor_island_for_launch_target(DriveLaunchTarget::Sheets),
            EditorIslandKind::SheetsEditor
        );
        assert_eq!(
            editor_island_for_launch_target(DriveLaunchTarget::Slides),
            EditorIslandKind::SlidesEditor
        );
        assert_eq!(
            editor_island_for_launch_target(DriveLaunchTarget::Preview),
            EditorIslandKind::DrivePreview
        );
    }

    #[test]
    fn rendered_workspace_shell_is_tenant_aware_selective_hydration() {
        let tenant_id = TenantId::new("tenant-ui").expect("valid tenant id");
        let object_id = ObjectId::new("object-ui").expect("valid object id");

        let shell = render_drive_workspace_shell(&tenant_id, Some(&object_id));

        assert!(shell.contains("data-ssr=\"leptos\""));
        assert!(shell.contains("data-hydration=\"selective\""));
        assert!(shell.contains(G080_DRIVE_SHELL_ROUTE_CONTRACT_VERSION));
        assert!(shell.contains("data-tenant-id=\"tenant-ui\""));
        assert!(shell.contains("data-object-id=\"object-ui\""));
        assert!(shell.contains("data-editor-island=\"docs\""));
        assert!(shell.contains("data-editor-island=\"sheets\""));
        assert!(shell.contains("data-editor-island=\"slides\""));
        assert!(!shell.contains("csr-only"));
    }

    #[test]
    fn g084_leptos_shell_contract_scaffolds_accessible_regions_and_hydration_gates() {
        let regions = g084_drive_workspace_shell_regions();
        let gates = g084_hydration_test_gates();
        let feature_flags = g084_required_leptos_feature_flags();

        assert_eq!(G084_LEPTOS_SHELL_CONTRACT_VERSION, "g084-leptos-shell-v1");
        assert_eq!(feature_flags, ["ssr", "hydrate"]);
        assert!(G084_LEPTOS_SSR_LIFECYCLE_SOURCE.contains("book.leptos.dev/ssr"));
        assert!(G084_LEPTOS_ISLANDS_SOURCE.contains("book.leptos.dev/islands"));

        assert_eq!(regions.len(), 8);
        assert!(regions.iter().all(|region| region.is_ssr_visible()));
        assert!(regions.iter().all(|region| region.is_keyboard_reachable()));
        assert!(regions.iter().any(|region| {
            region.kind() == DriveWorkspaceShellRegionKind::DriveNavigation
                && region.is_drive_bound()
                && region.hydration_boundary() == "navigation"
        }));
        assert!(regions.iter().any(|region| {
            region.kind() == DriveWorkspaceShellRegionKind::DocsEditorIsland
                && region.is_drive_bound()
                && region.kind().editor_island() == Some(EditorIslandKind::DocsEditor)
                && region.hydration_boundary() == "editor"
        }));
        assert!(regions.iter().any(|region| {
            region.kind() == DriveWorkspaceShellRegionKind::SheetsEditorIsland
                && region.kind().editor_island() == Some(EditorIslandKind::SheetsEditor)
        }));
        assert!(regions.iter().any(|region| {
            region.kind() == DriveWorkspaceShellRegionKind::SlidesEditorIsland
                && region.kind().editor_island() == Some(EditorIslandKind::SlidesEditor)
        }));

        assert_eq!(gates.len(), 9);
        assert!(gates.iter().all(|gate| gate.is_launch_blocking()));
        assert!(gates.iter().any(|gate| {
            gate.kind() == HydrationTestGateKind::SsrServerFeature
                && gate.evidence().contains("ssr")
        }));
        assert!(gates.iter().any(|gate| {
            gate.kind() == HydrationTestGateKind::HydrateBrowserFeature
                && gate.evidence().contains("hydrate")
        }));
        assert!(
            gates
                .iter()
                .any(|gate| gate.kind() == HydrationTestGateKind::ExistingDomPickup)
        );
        assert!(
            gates
                .iter()
                .any(|gate| gate.kind() == HydrationTestGateKind::NoCsrOnlyFallback)
        );
        assert!(
            gates
                .iter()
                .any(|gate| gate.kind() == HydrationTestGateKind::NoWholePageHydration)
        );
    }

    #[test]
    fn g084_rendered_drive_workspace_shell_is_ssr_visible_and_hydration_testable() {
        let tenant_id = TenantId::new("tenant-g084").expect("valid tenant id");
        let object_id = ObjectId::new("object-g084").expect("valid object id");

        let shell = render_g084_drive_workspace_shell(&tenant_id, Some(&object_id));
        let observation =
            SsrShellPerformanceObservation::from_shell_html(&shell, 40, 150).expect("valid");

        assert_eq!(validate_ssr_shell_markup(&shell), Ok(()));
        assert!(shell.contains(G084_LEPTOS_SHELL_CONTRACT_VERSION));
        assert!(shell.contains("data-leptos-ssr-feature=\"ssr\""));
        assert!(shell.contains("data-leptos-hydrate-feature=\"hydrate\""));
        assert!(shell.contains("href=\"#drive-workspace-content\""));
        assert!(shell.contains("role=\"banner\""));
        assert!(shell.contains("aria-label=\"Drive workspace\""));
        assert!(shell.contains("data-shell-region=\"object-context\""));
        assert!(shell.contains("data-shell-region=\"docs-editor-island\""));
        assert!(shell.contains("data-shell-region=\"sheets-editor-island\""));
        assert!(shell.contains("data-shell-region=\"slides-editor-island\""));
        assert!(shell.contains("data-dom-pickup=\"preserve\""));
        assert!(shell.contains("data-hydration-gate=\"existing-dom-pickup\""));
        assert!(shell.contains("data-hydration-gate=\"no-csr-only-fallback\""));
        assert!(shell.contains("data-hydration-gate=\"no-whole-page-hydration\""));
        assert!(shell.contains(G084_LEPTOS_SSR_LIFECYCLE_SOURCE));
        assert!(shell.contains(G084_LEPTOS_ISLANDS_SOURCE));
        assert!(!shell.contains("data-hydration=\"csr-only\""));
        assert!(!shell.contains("data-hydration=\"whole-page\""));
        assert_eq!(observation.selective_islands(), 4);
        assert_eq!(
            ssr_shell_performance_budget().evaluate(&observation),
            WebShellPerformanceDecision::Pass
        );
    }

    #[test]
    fn g075_designer_ux_contract_exposes_accessible_collaboration_panels() {
        let panels = editor_interaction_panels(DriveLaunchTarget::Docs);

        assert_eq!(G075_DESIGNER_UX_CONTRACT_VERSION, "g075-designer-ux-v1");
        assert_eq!(panels.len(), 7);
        assert!(panels.iter().all(|panel| panel.is_drive_bound()));
        assert!(
            panels
                .iter()
                .all(|panel| panel.editor_island() == EditorIslandKind::DocsEditor)
        );
        assert!(
            panels
                .iter()
                .all(|panel| panel.hydration_boundary() == "interaction")
        );
        assert!(
            panels
                .iter()
                .all(|panel| panel.requires_text_status() && !panel.aria_label().is_empty())
        );
        assert!(
            panels
                .iter()
                .enumerate()
                .all(|(index, panel)| panel.focus_order() == (index + 1) as u8)
        );

        assert_eq!(panels[0].kind(), EditorInteractionPanelKind::Presence);
        assert_eq!(panels[1].kind(), EditorInteractionPanelKind::Comments);
        assert_eq!(panels[2].kind(), EditorInteractionPanelKind::Suggestions);
        assert_eq!(panels[3].kind(), EditorInteractionPanelKind::VersionHistory);
        assert_eq!(panels[4].kind(), EditorInteractionPanelKind::Share);
        assert_eq!(panels[5].kind(), EditorInteractionPanelKind::SaveState);
        assert_eq!(panels[6].kind(), EditorInteractionPanelKind::Recovery);
        assert_eq!(
            panels[0].collaboration_status(),
            CollaborationStatusKind::Live
        );
        assert_eq!(CollaborationStatusKind::Offline.as_str(), "offline");
        assert_eq!(CollaborationStatusKind::Connecting.as_str(), "connecting");
        assert_eq!(CollaborationStatusKind::Conflict.as_str(), "conflict");
        assert_eq!(EditorInteractionPanelKind::SaveState.as_str(), "save-state");
    }

    #[test]
    fn g075_interaction_panels_reject_non_drive_bound_or_zero_focus_order() {
        assert_eq!(
            EditorInteractionPanel::new(
                EditorInteractionPanelKind::Presence,
                EditorIslandKind::DocsEditor,
                CollaborationStatusKind::Live,
                true,
                0,
            ),
            Err(WebShellError::InvalidInteractionPanel)
        );
        assert_eq!(
            EditorInteractionPanel::new(
                EditorInteractionPanelKind::Presence,
                EditorIslandKind::DocsEditor,
                CollaborationStatusKind::Live,
                false,
                1,
            ),
            Err(WebShellError::InvalidInteractionPanel)
        );
    }

    #[test]
    fn rendered_workspace_shell_includes_g075_collaboration_interactions_without_whole_page_hydration()
     {
        let tenant_id = TenantId::new("tenant-ui").expect("valid tenant id");
        let object_id = ObjectId::new("object-ui").expect("valid object id");
        let shell = render_drive_workspace_shell(&tenant_id, Some(&object_id));
        let sheets_contract = render_editor_interaction_contract(DriveLaunchTarget::Sheets);

        assert!(shell.contains("aria-label=\"Collaboration interactions\""));
        assert!(shell.contains(G075_DESIGNER_UX_CONTRACT_VERSION));
        assert!(shell.contains("data-collab-panel=\"presence\""));
        assert!(shell.contains("data-collab-panel=\"comments\""));
        assert!(shell.contains("data-collab-panel=\"suggestions\""));
        assert!(shell.contains("data-collab-panel=\"save-state\""));
        assert!(shell.contains("data-collab-panel=\"recovery\""));
        assert!(shell.contains("data-hydration-boundary=\"interaction\""));
        assert!(shell.contains("data-drive-bound=\"true\""));
        assert!(shell.contains("data-text-status-required=\"true\""));
        assert!(sheets_contract.contains("data-editor-island=\"sheets\""));
        assert!(sheets_contract.contains("aria-label=\"Version history\""));
        assert!(!shell.contains("whole-page"));
        assert!(!shell.contains("csr-only"));
        assert_eq!(validate_ssr_shell_markup(&shell), Ok(()));
    }

    #[test]
    fn g080_drive_shell_routes_are_tenant_object_bound_and_ssr_only() {
        let contracts = drive_shell_route_contracts();
        let routes = tenant_aware_drive_shell_routes();

        assert_eq!(
            G080_DRIVE_SHELL_ROUTE_CONTRACT_VERSION,
            "g080-drive-shell-route-v1"
        );
        assert_eq!(contracts.len(), routes.len());
        assert!(contracts.iter().all(|contract| contract.is_tenant_scoped()));
        assert!(contracts.iter().all(|contract| contract.ssr_required()));
        assert!(
            contracts
                .iter()
                .filter(|contract| contract.is_object_scoped())
                .all(|contract| contract.path_template().contains(":object_id"))
        );
        assert!(contracts.iter().any(|contract| {
            contract.kind() == WebShellRouteKind::DocsEditor
                && contract.launch_target() == Some(DriveLaunchTarget::Docs)
                && contract.path_template() == "/t/:tenant_id/drive/:object_id/docs"
        }));
        assert!(contracts.iter().any(|contract| {
            contract.kind() == WebShellRouteKind::SheetsEditor
                && contract.launch_target() == Some(DriveLaunchTarget::Sheets)
                && route_kind_for_launch(DriveLaunchTarget::Sheets)
                    == WebShellRouteKind::SheetsEditor
        }));
        assert!(contracts.iter().any(|contract| {
            contract.kind() == WebShellRouteKind::SlidesEditor
                && contract.launch_target() == Some(DriveLaunchTarget::Slides)
        }));
        assert!(
            contracts
                .iter()
                .any(|contract| contract.kind() == WebShellRouteKind::DriveObject
                    && contract.launch_target() == Some(DriveLaunchTarget::Preview))
        );
    }

    #[test]
    fn ssr_performance_budget_requires_selective_hydration() {
        let budget = ssr_shell_performance_budget();

        assert_eq!(budget.hydration_mode(), HydrationMode::SelectiveIslands);
        assert_eq!(
            super::SsrShellPerformanceBudget::new(HydrationMode::ClientOnly, 100, 250, 16_384, 4),
            Err(WebShellError::CsrOnlyHydration)
        );
        assert_eq!(
            super::SsrShellPerformanceBudget::new(HydrationMode::WholePage, 100, 250, 16_384, 4),
            Err(WebShellError::WholePageHydration)
        );
    }

    #[test]
    fn ssr_shell_markup_policy_rejects_csr_only_and_whole_page_hydration() {
        let tenant_id = TenantId::new("tenant-ui").expect("valid tenant id");
        let object_id = ObjectId::new("object-ui").expect("valid object id");
        let shell = render_drive_workspace_shell(&tenant_id, Some(&object_id));

        assert_eq!(validate_ssr_shell_markup(&shell), Ok(()));
        assert_eq!(
            validate_ssr_shell_markup("<main data-hydration=\"csr-only\"></main>"),
            Err(WebShellError::CsrOnlyHydration)
        );
        assert_eq!(
            validate_ssr_shell_markup(
                "<main data-ssr=\"leptos\" data-hydration=\"whole-page\"></main>"
            ),
            Err(WebShellError::WholePageHydration)
        );
    }

    #[test]
    fn g066_leptos_ssr_shell_contract_rejects_csr_only_and_whole_page_hydration() {
        let tenant_id = TenantId::new("tenant-g066").expect("valid tenant id");
        let object_id = ObjectId::new("object-g066").expect("valid object id");
        let shell = render_drive_workspace_shell(&tenant_id, Some(&object_id));
        let budget = ssr_shell_performance_budget();

        assert_eq!(budget.hydration_mode(), HydrationMode::SelectiveIslands);
        assert!(shell.contains("data-ssr=\"leptos\""));
        assert!(shell.contains("data-hydration=\"selective\""));
        assert_eq!(validate_ssr_shell_markup(&shell), Ok(()));
        assert_eq!(
            validate_ssr_shell_markup("<main data-hydration=\"csr-only\"></main>"),
            Err(WebShellError::CsrOnlyHydration)
        );
        assert_eq!(
            validate_ssr_shell_markup(
                "<main data-ssr=\"leptos\" data-hydration=\"whole-page\"></main>"
            ),
            Err(WebShellError::WholePageHydration)
        );
    }

    #[test]
    fn ssr_performance_budget_fails_p95_payload_or_island_regressions() {
        let budget = ssr_shell_performance_budget();
        let passing = SsrShellPerformanceObservation::new(45, 140, 8_192, 4).expect("valid");
        let slow_ssr = SsrShellPerformanceObservation::new(
            45,
            budget.max_ssr_render_p95_millis() + 1,
            8_192,
            4,
        )
        .expect("valid");
        let large_payload =
            SsrShellPerformanceObservation::new(45, 140, budget.max_shell_html_bytes() + 1, 4)
                .expect("valid");
        let too_many_islands =
            SsrShellPerformanceObservation::new(45, 140, 8_192, budget.max_selective_islands() + 1)
                .expect("valid");

        assert_eq!(budget.evaluate(&passing), WebShellPerformanceDecision::Pass);
        assert_eq!(
            budget.evaluate(&slow_ssr),
            WebShellPerformanceDecision::Fail
        );
        assert_eq!(
            budget.evaluate(&large_payload),
            WebShellPerformanceDecision::Fail
        );
        assert_eq!(
            budget.evaluate(&too_many_islands),
            WebShellPerformanceDecision::Fail
        );
    }

    #[test]
    fn rendered_shell_stays_within_scaffold_html_budget() {
        let tenant_id = TenantId::new("tenant-ui").expect("valid tenant id");
        let object_id = ObjectId::new("object-ui").expect("valid object id");
        let shell = render_drive_workspace_shell(&tenant_id, Some(&object_id));
        let observation =
            SsrShellPerformanceObservation::from_shell_html(&shell, 35, 120).expect("valid");

        assert_eq!(
            ssr_shell_performance_budget().evaluate(&observation),
            WebShellPerformanceDecision::Pass
        );
    }
}
