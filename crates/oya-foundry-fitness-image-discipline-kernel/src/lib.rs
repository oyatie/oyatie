//! Image-discipline fitness kernel — blocks non-distroless bases,
//! shells / package-managers in the final image layer, and oversized
//! images. Per M-CC-P06: distroless + LTS + size budget.
//!
//! I/O-free. Runners parse Dockerfiles / image manifests / OCI inspect
//! output into typed [`ImageDescriptor`] records and feed them to
//! [`check`].
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// A single image build artifact under review.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImageDescriptor {
    pub image_id: String,               // data_class: INTERNAL_ONLY
    pub base_image: String,             // data_class: INTERNAL_ONLY
    pub final_layer_paths: Vec<String>, // data_class: INTERNAL_ONLY
    pub size_bytes: u64,                // data_class: INTERNAL_ONLY
}

/// Per-image-id size budget. Runners populate from
/// `docs/standards/image-size-budgets.md`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImageBudget {
    pub image_id: String, // data_class: INTERNAL_ONLY
    pub max_bytes: u64,   // data_class: INTERNAL_ONLY
}

/// Distroless allowlist. A base passes iff its reference (before
/// any `:tag` / `@digest`) starts with one of these prefixes.
pub const DISTROLESS_PREFIXES: [&str; 4] = [
    "gcr.io/distroless/",
    "cgr.dev/chainguard/",
    "registry.access.redhat.com/ubi9-micro",
    "scratch",
];

/// Path-fragments that must NOT appear in the final image layer.
/// "Final layer" = anything still mounted at container start time.
pub const FORBIDDEN_FINAL_LAYER_FRAGMENTS: [&str; 7] = [
    "/bin/sh",
    "/bin/bash",
    "/usr/bin/apt",
    "/usr/bin/apt-get",
    "/usr/bin/dnf",
    "/usr/bin/yum",
    "/usr/bin/apk",
];

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ImageViolationKind {
    NonDistrolessBase,
    ShellInFinalLayer,
    PackageManagerInFinalLayer,
    OversizedImage,
    MissingBudget,
}

impl ImageViolationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NonDistrolessBase => "non-distroless base image",
            Self::ShellInFinalLayer => "shell in final layer",
            Self::PackageManagerInFinalLayer => "package manager in final layer",
            Self::OversizedImage => "image exceeds size budget",
            Self::MissingBudget => "no size budget declared for image",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImageViolation {
    pub image_id: String,         // data_class: INTERNAL_ONLY
    pub kind: ImageViolationKind, // data_class: INTERNAL_ONLY
    pub detail: String,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImageDisciplineReport {
    pub images_checked: usize,           // data_class: INTERNAL_ONLY
    pub violations: Vec<ImageViolation>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImageDisciplineError {
    EmptyImageId,
    EmptyBaseImage { image_id: String },
    DuplicateBudget { image_id: String },
}

impl ImageDisciplineError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyImageId => "image_id is empty".to_owned(),
            Self::EmptyBaseImage { image_id } => format!("{image_id}: base_image is empty"),
            Self::DuplicateBudget { image_id } => {
                format!("duplicate budget entry for {image_id}")
            }
        }
    }
}

fn base_ref(base_image: &str) -> &str {
    // Strip `:tag` and `@digest` suffixes for prefix matching.
    let s = base_image
        .split_once('@')
        .map(|(l, _)| l)
        .unwrap_or(base_image);
    s.rsplit_once(':').map(|(l, _)| l).unwrap_or(s)
}

fn is_distroless(base_image: &str) -> bool {
    let r = base_ref(base_image);
    DISTROLESS_PREFIXES
        .iter()
        .any(|p| r == *p || r.starts_with(p))
}

fn classify_final_layer_path(path: &str) -> Option<ImageViolationKind> {
    if !FORBIDDEN_FINAL_LAYER_FRAGMENTS.contains(&path) {
        return None;
    }
    Some(match path {
        "/bin/sh" | "/bin/bash" => ImageViolationKind::ShellInFinalLayer,
        _ => ImageViolationKind::PackageManagerInFinalLayer,
    })
}

pub fn check(
    images: &[ImageDescriptor],
    budgets: &[ImageBudget],
) -> Result<ImageDisciplineReport, ImageDisciplineError> {
    let mut budget_by_id: std::collections::BTreeMap<&str, u64> = std::collections::BTreeMap::new();
    for b in budgets {
        if budget_by_id
            .insert(b.image_id.as_str(), b.max_bytes)
            .is_some()
        {
            return Err(ImageDisciplineError::DuplicateBudget {
                image_id: b.image_id.clone(),
            });
        }
    }

    let mut violations = Vec::new();

    for img in images {
        if img.image_id.is_empty() {
            return Err(ImageDisciplineError::EmptyImageId);
        }
        if img.base_image.is_empty() {
            return Err(ImageDisciplineError::EmptyBaseImage {
                image_id: img.image_id.clone(),
            });
        }

        if !is_distroless(&img.base_image) {
            violations.push(ImageViolation {
                image_id: img.image_id.clone(),
                kind: ImageViolationKind::NonDistrolessBase,
                detail: img.base_image.clone(),
            });
        }

        for p in &img.final_layer_paths {
            if let Some(kind) = classify_final_layer_path(p) {
                violations.push(ImageViolation {
                    image_id: img.image_id.clone(),
                    kind,
                    detail: p.clone(),
                });
            }
        }

        match budget_by_id.get(img.image_id.as_str()) {
            Some(max) => {
                if img.size_bytes > *max {
                    violations.push(ImageViolation {
                        image_id: img.image_id.clone(),
                        kind: ImageViolationKind::OversizedImage,
                        detail: format!("{} > {} bytes", img.size_bytes, max),
                    });
                }
            }
            None => violations.push(ImageViolation {
                image_id: img.image_id.clone(),
                kind: ImageViolationKind::MissingBudget,
                detail: String::new(),
            }),
        }
    }

    Ok(ImageDisciplineReport {
        images_checked: images.len(),
        violations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn img(id: &str, base: &str, paths: &[&str], size: u64) -> ImageDescriptor {
        ImageDescriptor {
            image_id: id.into(),
            base_image: base.into(),
            final_layer_paths: paths.iter().map(|p| (*p).into()).collect(),
            size_bytes: size,
        }
    }
    fn budget(id: &str, max: u64) -> ImageBudget {
        ImageBudget {
            image_id: id.into(),
            max_bytes: max,
        }
    }

    #[test]
    fn distroless_clean_image_passes() {
        let r = check(
            &[img(
                "svc",
                "gcr.io/distroless/cc-debian12:latest",
                &[],
                1_000_000,
            )],
            &[budget("svc", 5_000_000)],
        )
        .unwrap();
        assert!(r.violations.is_empty(), "{:?}", r.violations);
    }

    #[test]
    fn scratch_base_passes() {
        let r = check(
            &[img("svc", "scratch", &[], 1_000_000)],
            &[budget("svc", 5_000_000)],
        )
        .unwrap();
        assert!(r.violations.is_empty(), "{:?}", r.violations);
    }

    #[test]
    fn chainguard_base_passes() {
        let r = check(
            &[img("svc", "cgr.dev/chainguard/static:latest", &[], 100)],
            &[budget("svc", 1_000_000)],
        )
        .unwrap();
        assert!(r.violations.is_empty(), "{:?}", r.violations);
    }

    #[test]
    fn ubuntu_base_flagged_as_non_distroless() {
        let r = check(
            &[img("svc", "ubuntu:24.04", &[], 100)],
            &[budget("svc", 1_000_000)],
        )
        .unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == ImageViolationKind::NonDistrolessBase)
        );
    }

    #[test]
    fn alpine_base_flagged_as_non_distroless() {
        let r = check(
            &[img("svc", "alpine:3.20", &[], 100)],
            &[budget("svc", 1_000_000)],
        )
        .unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == ImageViolationKind::NonDistrolessBase)
        );
    }

    #[test]
    fn shell_in_final_layer_flagged() {
        let r = check(
            &[img(
                "svc",
                "gcr.io/distroless/cc-debian12",
                &["/bin/sh"],
                100,
            )],
            &[budget("svc", 1_000_000)],
        )
        .unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == ImageViolationKind::ShellInFinalLayer)
        );
    }

    #[test]
    fn bash_in_final_layer_flagged() {
        let r = check(
            &[img(
                "svc",
                "gcr.io/distroless/cc-debian12",
                &["/bin/bash"],
                100,
            )],
            &[budget("svc", 1_000_000)],
        )
        .unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == ImageViolationKind::ShellInFinalLayer)
        );
    }

    #[test]
    fn package_manager_in_final_layer_flagged() {
        let r = check(
            &[img(
                "svc",
                "gcr.io/distroless/cc-debian12",
                &["/usr/bin/apt-get"],
                100,
            )],
            &[budget("svc", 1_000_000)],
        )
        .unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == ImageViolationKind::PackageManagerInFinalLayer)
        );
    }

    #[test]
    fn oversized_image_flagged() {
        let r = check(
            &[img("svc", "gcr.io/distroless/static", &[], 10_000_000)],
            &[budget("svc", 5_000_000)],
        )
        .unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == ImageViolationKind::OversizedImage)
        );
    }

    #[test]
    fn missing_budget_flagged() {
        let r = check(&[img("svc", "gcr.io/distroless/static", &[], 100)], &[]).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == ImageViolationKind::MissingBudget)
        );
    }

    #[test]
    fn empty_image_id_errors() {
        let err = check(&[img("", "scratch", &[], 100)], &[]).unwrap_err();
        assert!(matches!(err, ImageDisciplineError::EmptyImageId));
    }

    #[test]
    fn empty_base_image_errors() {
        let err = check(&[img("svc", "", &[], 100)], &[]).unwrap_err();
        assert!(matches!(err, ImageDisciplineError::EmptyBaseImage { .. }));
    }

    #[test]
    fn duplicate_budget_errors() {
        let err = check(&[], &[budget("svc", 1), budget("svc", 2)]).unwrap_err();
        assert!(matches!(err, ImageDisciplineError::DuplicateBudget { .. }));
    }

    #[test]
    fn base_image_strips_tag_and_digest() {
        // `gcr.io/distroless/cc-debian12@sha256:abcdef` should pass.
        let r = check(
            &[img(
                "svc",
                "gcr.io/distroless/cc-debian12@sha256:abcdef",
                &[],
                100,
            )],
            &[budget("svc", 1_000_000)],
        )
        .unwrap();
        assert!(
            !r.violations
                .iter()
                .any(|v| v.kind == ImageViolationKind::NonDistrolessBase),
            "{:?}",
            r.violations
        );
    }

    #[test]
    fn multiple_violations_aggregated() {
        let r = check(
            &[img(
                "svc",
                "ubuntu:24.04",
                &["/bin/sh", "/usr/bin/apt"],
                10_000_000,
            )],
            &[budget("svc", 1_000_000)],
        )
        .unwrap();
        // Expect: NonDistrolessBase + ShellInFinalLayer + PackageManagerInFinalLayer
        // + OversizedImage = 4.
        assert_eq!(r.violations.len(), 4, "{:?}", r.violations);
    }

    #[test]
    fn violation_kind_as_str_distinct() {
        let kinds = [
            ImageViolationKind::NonDistrolessBase,
            ImageViolationKind::ShellInFinalLayer,
            ImageViolationKind::PackageManagerInFinalLayer,
            ImageViolationKind::OversizedImage,
            ImageViolationKind::MissingBudget,
        ];
        let names: std::collections::HashSet<_> = kinds.iter().map(|k| k.as_str()).collect();
        assert_eq!(names.len(), kinds.len());
    }
}
