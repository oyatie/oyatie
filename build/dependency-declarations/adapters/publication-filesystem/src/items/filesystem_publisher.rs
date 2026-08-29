use std::fs::File;
use std::io::{Read, Write};
use std::sync::Mutex;

use dependency_declarations_publication::{PublicationCapabilityPort, PublicationPort};
use dependency_declarations_reconcile::{
    DigestV1, DurabilityStateV1, FailureClassV1, FailureV1, PublicationObservationV1,
    PublicationOutcomeV1, PublicationPortErrorV1, PublicationRequestV1, PublisherProfileV1,
    ReplacementStateV1, ValidationBoundsV1,
};
use rustix::fs::{
    AtFlags, FileType, FlockOperation, Mode, OFlags, fchmod, flock, fstat, fstatfs, fsync,
    openat, renameat, unlinkat,
};

/// Configuration failures detected before a filesystem publisher is admitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FilesystemPublisherConfigurationFailureV1 {
    NotDirectory,
    FilesystemUnavailable,
    ProfileMismatch,
}

impl std::fmt::Display for FilesystemPublisherConfigurationFailureV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reason = match self {
            Self::NotDirectory => "the supplied capability is not a directory",
            Self::FilesystemUnavailable => "the filesystem profile cannot be observed",
            Self::ProfileMismatch => "the observed filesystem does not match the profile",
        };
        write!(formatter, "filesystem publisher configuration refused: {reason}")
    }
}

impl std::error::Error for FilesystemPublisherConfigurationFailureV1 {}

/// Atomic publisher scoped to an already-open `third-party` directory.
pub struct FilesystemPublisherV1 {
    directory: File,
    profile: PublisherProfileV1,
    process_lease: Mutex<()>,
}

impl FilesystemPublisherV1 {
    /// Admits a directory only when its observed filesystem matches the profile.
    pub fn try_new(
        directory: File,
        profile: PublisherProfileV1,
    ) -> Result<Self, FilesystemPublisherConfigurationFailureV1> {
        let metadata = directory
            .metadata()
            .map_err(|_| FilesystemPublisherConfigurationFailureV1::FilesystemUnavailable)?;
        if !metadata.is_dir() {
            return Err(FilesystemPublisherConfigurationFailureV1::NotDirectory);
        }
        let observed = detected_publisher_profile(&directory)
            .map_err(|_| FilesystemPublisherConfigurationFailureV1::FilesystemUnavailable)?;
        if observed != Some(profile) {
            return Err(FilesystemPublisherConfigurationFailureV1::ProfileMismatch);
        }
        Ok(Self::from_directory(directory, profile))
    }

    fn from_directory(directory: File, profile: PublisherProfileV1) -> Self {
        Self {
            directory,
            profile,
            process_lease: Mutex::new(()),
        }
    }

    fn publish_content(
        &self,
        expected_preimage: Option<DigestV1>,
        bytes: &[u8],
    ) -> PublicationOutcomeV1 {
        if bytes.len() > ValidationBoundsV1::MAX_OUTPUT_BYTES {
            return failed_publication(FailureClassV1::InternalInvariant);
        }
        let Ok(_process_lease) = self.process_lease.try_lock() else {
            return failed_publication(FailureClassV1::DestinationLeaseUnavailable);
        };
        let Ok(_directory_lease) = DirectoryLeaseV1::try_acquire(&self.directory) else {
            return failed_publication(FailureClassV1::DestinationLeaseUnavailable);
        };
        let mut transaction = RustixPublicationTransactionV1::new(&self.directory);
        run_publication_transaction(&mut transaction, expected_preimage, bytes)
    }
}

impl PublicationCapabilityPort<PublisherProfileV1> for FilesystemPublisherV1 {
    fn supports(&self, profile: &PublisherProfileV1) -> bool {
        *profile == self.profile
    }
}

impl PublicationPort<PublicationRequestV1, PublicationObservationV1, PublicationPortErrorV1>
    for FilesystemPublisherV1
{
    fn publish(
        &self,
        request: &PublicationRequestV1,
    ) -> Result<PublicationObservationV1, PublicationPortErrorV1> {
        let outcome = if self.supports(&request.intent().publisher()) {
            self.publish_content(
                request.intent().expected_preimage(),
                request.generation().bytes(),
            )
        } else {
            indeterminate_publication(FailureClassV1::InternalInvariant)
        };
        Ok(PublicationObservationV1::new(outcome))
    }
}
