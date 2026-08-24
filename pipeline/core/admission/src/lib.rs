//! Admission primitives for the protected `presubmit` graph.

pub mod cadence;
pub mod fanin;
pub mod git_change;
pub mod layout;
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
    ALLOWED_DOT_ROOT_DIRS, ALLOWED_ROOT_DIRS, ALLOWED_ROOT_FILES, BUILD_ROOT_DIRS, CAP_EXTRAS,
    FACES, FORBIDDEN_NAMES, META_ROOTS, cap_root_file_ok, changed_layout_violations, face_dir_ok,
    is_capability_root, layout_violations,
};
pub use occupancy::{
    OYATIE_HUB_PREFIXES, OYATIE_HUBS, OccupancyRefused, OccupiedSet, admit, hits_hub,
};
pub use owners::{ROOT_OCCUPANT, owners_occupant};
