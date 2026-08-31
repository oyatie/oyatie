use std::time::Instant;

use crate::{DigestBuilder, EvidenceDigest, ProfileId, SchemaId, SnapshotFailure};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotLimits {
    max_entries: u64,
    max_path_bytes: u64,
    max_manifest_bytes: u64,
    max_selected_contents: u64,
    max_content_bytes: u64,
    max_total_content_bytes: u64,
    max_stdout_bytes: u64,
    max_stderr_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotLimitSpec {
    pub max_entries: u64,
    pub max_path_bytes: u64,
    pub max_manifest_bytes: u64,
    pub max_selected_contents: u64,
    pub max_content_bytes: u64,
    pub max_total_content_bytes: u64,
    pub max_stdout_bytes: u64,
    pub max_stderr_bytes: u64,
}

impl SnapshotLimits {
    pub fn new(spec: SnapshotLimitSpec) -> Result<Self, SnapshotFailure> {
        let SnapshotLimitSpec {
            max_entries,
            max_path_bytes,
            max_manifest_bytes,
            max_selected_contents,
            max_content_bytes,
            max_total_content_bytes,
            max_stdout_bytes,
            max_stderr_bytes,
        } = spec;
        let limits = Self {
            max_entries,
            max_path_bytes,
            max_manifest_bytes,
            max_selected_contents,
            max_content_bytes,
            max_total_content_bytes,
            max_stdout_bytes,
            max_stderr_bytes,
        };
        for (name, value) in limits.fields() {
            if value == 0 {
                return Err(SnapshotFailure::InvalidProfile(format!(
                    "{name} must be greater than zero"
                )));
            }
        }
        if max_content_bytes > max_total_content_bytes {
            return Err(SnapshotFailure::InvalidProfile(
                "max_content_bytes must not exceed max_total_content_bytes".to_owned(),
            ));
        }
        Ok(limits)
    }

    pub const fn max_entries(self) -> u64 {
        self.max_entries
    }

    pub const fn max_path_bytes(self) -> u64 {
        self.max_path_bytes
    }

    pub const fn max_manifest_bytes(self) -> u64 {
        self.max_manifest_bytes
    }

    pub const fn max_selected_contents(self) -> u64 {
        self.max_selected_contents
    }

    pub const fn max_content_bytes(self) -> u64 {
        self.max_content_bytes
    }

    pub const fn max_total_content_bytes(self) -> u64 {
        self.max_total_content_bytes
    }

    pub const fn max_stdout_bytes(self) -> u64 {
        self.max_stdout_bytes
    }

    pub const fn max_stderr_bytes(self) -> u64 {
        self.max_stderr_bytes
    }

    pub(crate) fn digest_into(self, digest: &mut DigestBuilder) {
        for (_, value) in self.fields() {
            digest.push_u64(value);
        }
    }

    fn fields(self) -> [(&'static str, u64); 8] {
        [
            ("max_entries", self.max_entries),
            ("max_path_bytes", self.max_path_bytes),
            ("max_manifest_bytes", self.max_manifest_bytes),
            ("max_selected_contents", self.max_selected_contents),
            ("max_content_bytes", self.max_content_bytes),
            ("max_total_content_bytes", self.max_total_content_bytes),
            ("max_stdout_bytes", self.max_stdout_bytes),
            ("max_stderr_bytes", self.max_stderr_bytes),
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotProfile {
    id: ProfileId,
    schema: SchemaId,
    limits: SnapshotLimits,
    digest: EvidenceDigest,
}

impl SnapshotProfile {
    pub fn new(id: ProfileId, schema: SchemaId, limits: SnapshotLimits) -> Self {
        let mut digest = DigestBuilder::new(b"pipeline-repository-profile-v1");
        digest.push_bytes(id.as_str().as_bytes());
        digest.push_bytes(schema.as_str().as_bytes());
        limits.digest_into(&mut digest);
        Self {
            id,
            schema,
            limits,
            digest: digest.finish(),
        }
    }

    pub fn id(&self) -> &ProfileId {
        &self.id
    }

    pub fn schema(&self) -> &SchemaId {
        &self.schema
    }

    pub const fn limits(&self) -> SnapshotLimits {
        self.limits
    }

    pub const fn digest(&self) -> EvidenceDigest {
        self.digest
    }
}

pub trait WorkControl: Send + Sync {
    fn is_cancelled(&self) -> bool;

    fn deadline(&self) -> Option<Instant>;

    fn checkpoint(&self) -> Result<(), SnapshotFailure> {
        if self.is_cancelled() {
            return Err(SnapshotFailure::Cancelled);
        }
        if self
            .deadline()
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            return Err(SnapshotFailure::DeadlineExceeded);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NoCancellation {
    deadline: Option<Instant>,
}

impl NoCancellation {
    pub const fn without_deadline() -> Self {
        Self { deadline: None }
    }

    pub const fn until(deadline: Instant) -> Self {
        Self {
            deadline: Some(deadline),
        }
    }
}

impl WorkControl for NoCancellation {
    fn is_cancelled(&self) -> bool {
        false
    }

    fn deadline(&self) -> Option<Instant> {
        self.deadline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> SnapshotLimits {
        SnapshotLimits::new(limit_spec()).unwrap()
    }

    #[test]
    fn profile_digest_binds_every_limit() {
        let base = SnapshotProfile::new(
            ProfileId::new("path-layout-v1").unwrap(),
            SchemaId::new("repository-snapshot-v1").unwrap(),
            limits(),
        );
        let changed = SnapshotProfile::new(
            ProfileId::new("path-layout-v1").unwrap(),
            SchemaId::new("repository-snapshot-v1").unwrap(),
            SnapshotLimits::new(SnapshotLimitSpec {
                max_entries: 11,
                ..limit_spec()
            })
            .unwrap(),
        );

        assert_ne!(base.digest(), changed.digest());
    }

    #[test]
    fn invalid_limit_specs_refuse_before_work() {
        assert!(matches!(
            SnapshotLimits::new(SnapshotLimitSpec {
                max_entries: 0,
                ..limit_spec()
            }),
            Err(SnapshotFailure::InvalidProfile(_))
        ));
        assert!(matches!(
            SnapshotLimits::new(SnapshotLimitSpec {
                max_content_bytes: 61,
                max_total_content_bytes: 60,
                ..limit_spec()
            }),
            Err(SnapshotFailure::InvalidProfile(_))
        ));
    }

    fn limit_spec() -> SnapshotLimitSpec {
        SnapshotLimitSpec {
            max_entries: 10,
            max_path_bytes: 20,
            max_manifest_bytes: 30,
            max_selected_contents: 4,
            max_content_bytes: 50,
            max_total_content_bytes: 60,
            max_stdout_bytes: 70,
            max_stderr_bytes: 8,
        }
    }

    #[test]
    fn expired_deadline_refuses_at_checkpoint() {
        let control = NoCancellation::until(Instant::now());
        assert!(matches!(
            control.checkpoint(),
            Err(SnapshotFailure::DeadlineExceeded)
        ));
    }
}
