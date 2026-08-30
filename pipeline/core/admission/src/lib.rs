//! Admission primitives for the protected `presubmit` graph.

pub mod cadence;
pub mod fanin;
pub mod git_change;
pub mod layout;
pub mod line_budget;
pub mod occupancy;
pub mod owners;

pub use cadence::{
    CadenceEvent, LIVE_POSTGRES_CRATES, LIVE_POSTGRES_PATH_PREFIXES, POSTSUBMIT_JOBS,
    PRESUBMIT_JOBS, WORKFLOW_FILES, live_postgres_required,
};
pub use fanin::{FanIn, fan_in_ok, occupancy_ok, postgres_ok, postsubmit_ok, required_success};
pub use git_change::{
    GitChangePaths, PathSetParseError, git_change_paths_from_name_status_z,
    paths_from_name_status_z,
};
pub(crate) use layout::path_parts;
pub use layout::{
    ALLOWED_DOT_ROOT_DIRS, ALLOWED_ROOT_DIRS, ALLOWED_ROOT_FILES, APP_PRODUCT_DIRS,
    BUILD_ROOT_DIRS, CAP_EXTRAS, DATA_ROOTS, FACES, FORBIDDEN_NAMES, META_ROOTS,
    WORKSPACE_EXCLUDES, WORKSPACE_MEMBER_GLOBS, base_admission_violations, cap_root_file_ok,
    cargo_config_violations, cargo_entrypoint, cargo_manifest_for_crate_path,
    cargo_manifest_for_entrypoint, cargo_manifest_violations, changed_layout_violations,
    draft_dependency_violations, face_dir_ok, is_capability_root, layout_violations,
    owner_core_regression_violations, owner_law_regression_violations, proto_package_violations,
    workspace_draft_dependency_violations, workspace_membership_violations,
};
pub use line_budget::file_budget_violations;
pub use occupancy::{
    OccupancyRefused, OccupiedSet, admit, admit_authored, authored_paths, declared_mergeable,
};
pub use owners::{ROOT_OCCUPANT, owners_occupant};
