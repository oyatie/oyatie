//! Container-base-image validator (ADR-0146).
//!
//! Naming justification:
//! - Crate id `oya-check-container-base-image` — `oya-` brand prefix
//!   (ADR-0017 / MFL-0011), `check` lane class (Layer 1 kernel-tier validator
//!   per ADR-0083), `container-base-image` three-word subject.
//! - Library identifier `oya_check_container_base_image` — snake_case
//!   mirror (ADR-0105 v4 BNF §2.2).
//!
//! Enforces ADR-0146: every `microservices/*/iac/build/Dockerfile*` and
//! Helm `values.yaml` baseImage entry MUST point at
//! `gcr.io/distroless/static-debian12:nonroot` (or `:debug` for explicit
//! dev builds), and the Dockerfile MUST declare `USER 65532:65532`
//! (`65532` is the distroless `nonroot` UID; the surrounding
//! `oya.securityContext.podStandard65534` standard accepts 65532 as well
//! since that is the only UID baked into the canonical base).
//!
//! Tier 1 (kernel-tier) per ADR-0083: pure logic over already-loaded
//! file content. No filesystem IO inside the kernel.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;

/// The single canonical base image per ADR-0146. The `:debug` variant is
/// accepted for explicit dev builds (it ships busybox so a developer can
/// `kubectl exec` for ephemeral debugging without rebuilding the chart).
pub const CANONICAL_BASE_IMAGES: &[&str] = &[
    "gcr.io/distroless/static-debian12:nonroot",
    "gcr.io/distroless/static-debian12:debug-nonroot",
];

/// The single canonical UID/GID pair baked into distroless `:nonroot`.
pub const CANONICAL_USER: &str = "65532:65532";

/// A Dockerfile under inspection. `path` is informational; `content` is
/// the raw text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DockerfileInput {
    pub path: String,
    pub content: String,
}

/// One specific violation found by [`validate`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Violation {
    /// `FROM` directive uses a non-canonical base image.
    NonCanonicalBase { path: String, base: String },
    /// `USER` directive is missing from the file entirely (or only the
    /// builder stage carried one and the runtime stage forgot).
    MissingUser { path: String },
    /// `USER` directive present but specifies a UID/GID other than 65532.
    NonCanonicalUser { path: String, user: String },
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonCanonicalBase { path, base } => {
                write!(
                    f,
                    "{path}: non-canonical FROM '{base}'; ADR-0146 requires {}",
                    CANONICAL_BASE_IMAGES[0]
                )
            }
            Self::MissingUser { path } => {
                write!(
                    f,
                    "{path}: missing USER directive on runtime stage; \
                     ADR-0146 requires USER {CANONICAL_USER}"
                )
            }
            Self::NonCanonicalUser { path, user } => {
                write!(
                    f,
                    "{path}: non-canonical USER '{user}'; ADR-0146 requires USER {CANONICAL_USER}"
                )
            }
        }
    }
}

/// Pass report emitted on clean inputs.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Report {
    pub files_checked: usize,
    pub canonical_base_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    ViolationsFound(Vec<Violation>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ViolationsFound(v) => {
                writeln!(f, "container-base-image violations: {}", v.len())?;
                for violation in v {
                    writeln!(f, "  - {violation}")?;
                }
                Ok(())
            }
        }
    }
}

/// Validate a slice of Dockerfile inputs. Multi-stage builds are tolerated
/// — only the FINAL stage's `FROM` + `USER` are enforced (since prior
/// stages are throwaway builder images).
pub fn validate(files: &[DockerfileInput]) -> Result<Report, Error> {
    let mut violations = Vec::new();
    let mut canonical = 0usize;
    for file in files {
        let stages = collect_stages(&file.content);
        let Some(final_stage) = stages.last() else {
            violations.push(Violation::MissingUser {
                path: file.path.clone(),
            });
            continue;
        };
        if is_canonical_base(&final_stage.from) {
            canonical += 1;
        } else {
            violations.push(Violation::NonCanonicalBase {
                path: file.path.clone(),
                base: final_stage.from.clone(),
            });
        }
        match final_stage.user.as_deref() {
            None => violations.push(Violation::MissingUser {
                path: file.path.clone(),
            }),
            Some(user) if user.trim() == CANONICAL_USER || user.trim() == "65532" => {}
            Some(user) => violations.push(Violation::NonCanonicalUser {
                path: file.path.clone(),
                user: user.trim().to_string(),
            }),
        }
    }
    if violations.is_empty() {
        Ok(Report {
            files_checked: files.len(),
            canonical_base_count: canonical,
        })
    } else {
        Err(Error::ViolationsFound(violations))
    }
}

#[derive(Clone, Debug, Default)]
struct Stage {
    from: String,
    user: Option<String>,
}

fn collect_stages(content: &str) -> Vec<Stage> {
    let mut stages: Vec<Stage> = Vec::new();
    for raw_line in content.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Multi-stage builds tag stages with `FROM <image> AS <alias>`; only
        // the FROM token + image identifier matter for the kernel.
        if let Some(rest) = strip_directive(trimmed, "FROM") {
            let from = rest.split_whitespace().next().unwrap_or("").to_string();
            stages.push(Stage { from, user: None });
        } else if let Some(rest) = strip_directive(trimmed, "USER")
            && let Some(stage) = stages.last_mut()
        {
            stage.user = Some(rest.trim().to_string());
        }
    }
    stages
}

fn strip_directive<'a>(line: &'a str, directive: &str) -> Option<&'a str> {
    let upper = line
        .chars()
        .take(directive.len())
        .collect::<String>()
        .to_uppercase();
    if upper == directive {
        let after = &line[directive.len()..];
        if after.starts_with(char::is_whitespace) {
            return Some(after.trim_start());
        }
    }
    None
}

fn is_canonical_base(base: &str) -> bool {
    CANONICAL_BASE_IMAGES.contains(&base)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn df(path: &str, content: &str) -> DockerfileInput {
        DockerfileInput {
            path: path.to_string(),
            content: content.to_string(),
        }
    }

    const CANONICAL_DOCKERFILE: &str = "\
FROM rust:1.97.1-slim AS builder
WORKDIR /build
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM gcr.io/distroless/static-debian12:nonroot
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/app /app
USER 65532:65532
ENTRYPOINT [\"/app\"]
";

    #[test]
    fn canonical_dockerfile_passes() {
        let files = vec![df(
            "microservices/x/iac/build/Dockerfile",
            CANONICAL_DOCKERFILE,
        )];
        let report = validate(&files).expect("canonical dockerfile must pass");
        assert_eq!(report.files_checked, 1);
        assert_eq!(report.canonical_base_count, 1);
    }

    #[test]
    fn alpine_base_rejected() {
        let content = "FROM alpine:3.20\nUSER 65532:65532\nENTRYPOINT [\"/app\"]\n";
        let files = vec![df("Dockerfile", content)];
        let err = validate(&files).expect_err("alpine must fail");
        let Error::ViolationsFound(v) = err;
        assert!(matches!(v[0], Violation::NonCanonicalBase { .. }));
    }

    #[test]
    fn scratch_base_rejected() {
        // ADR-0146 carves out scratch only with explicit per-µservice ADR;
        // a bare `FROM scratch` is treated as non-canonical.
        let content = "FROM scratch\nUSER 65532:65532\nENTRYPOINT [\"/app\"]\n";
        let files = vec![df("Dockerfile", content)];
        assert!(validate(&files).is_err());
    }

    #[test]
    fn missing_user_directive_rejected() {
        let content = "FROM gcr.io/distroless/static-debian12:nonroot\nENTRYPOINT [\"/app\"]\n";
        let files = vec![df("Dockerfile", content)];
        let err = validate(&files).expect_err("missing USER must fail");
        let Error::ViolationsFound(v) = err;
        assert!(matches!(v[0], Violation::MissingUser { .. }));
    }

    #[test]
    fn non_canonical_user_rejected() {
        let content = "FROM gcr.io/distroless/static-debian12:nonroot\nUSER 1000:1000\n";
        let files = vec![df("Dockerfile", content)];
        let err = validate(&files).expect_err("uid 1000 must fail");
        let Error::ViolationsFound(v) = err;
        assert!(matches!(v[0], Violation::NonCanonicalUser { .. }));
    }

    #[test]
    fn debug_variant_accepted_for_dev_builds() {
        let content = "FROM gcr.io/distroless/static-debian12:debug-nonroot\nUSER 65532:65532\n";
        let files = vec![df("Dockerfile", content)];
        let report = validate(&files).expect(":debug-nonroot must pass per ADR-0146");
        assert_eq!(report.canonical_base_count, 1);
    }

    #[test]
    fn user_directive_accepts_bare_uid() {
        let content = "FROM gcr.io/distroless/static-debian12:nonroot\nUSER 65532\n";
        let files = vec![df("Dockerfile", content)];
        assert!(validate(&files).is_ok());
    }
}
