use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;

pub mod runtime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessError {
    InvalidProfile,
    TargetMismatch,
    ArchiveMismatch,
    InvalidCredentials,
    CertificateExpired,
    DependencyFailed,
    OutputLimit,
    Timeout,
    Cancelled,
    CleanupFailed,
    OciAuthenticationRequired,
}

impl std::fmt::Display for AccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "operator_access_{}",
            match self {
                Self::InvalidProfile => "invalid_profile",
                Self::TargetMismatch => "target_mismatch",
                Self::ArchiveMismatch => "archive_mismatch",
                Self::InvalidCredentials => "invalid_credentials",
                Self::CertificateExpired => "certificate_expired",
                Self::DependencyFailed => "dependency_failed",
                Self::OutputLimit => "output_limit",
                Self::Timeout => "timeout",
                Self::Cancelled => "cancelled",
                Self::CleanupFailed => "cleanup_failed",
                Self::OciAuthenticationRequired => "oci_authentication_required",
            }
        )
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub schema: u32,
    pub region: String,
    pub compartment: String,
    pub instance: String,
    pub boot_volume: String,
    pub shape: String,
    pub ocpus: u32,
    pub memory_gib: u32,
    pub private_ip: String,
    pub node_name: String,
    pub bastion: String,
    pub namespace: String,
    pub bucket: String,
    pub object: String,
    pub object_version: String,
    pub archive_sha256: String,
    pub talos_member: String,
    pub kube_member: String,
    pub talos_context: String,
    pub talos_ca_sha256: String,
    pub kube_ca_sha256: String,
    pub identity_file: String,
    pub ssh_identity_file: String,
    pub ssh_known_hosts_file: String,
    pub oci_config_file: String,
    pub oci_profile: String,
}

impl Profile {
    pub fn validate(&self) -> Result<(), AccessError> {
        let identifiers = [
            &self.region,
            &self.compartment,
            &self.instance,
            &self.boot_volume,
            &self.shape,
            &self.node_name,
            &self.bastion,
            &self.namespace,
            &self.bucket,
            &self.object_version,
            &self.oci_profile,
        ];
        if self.schema != 1
            || self.ocpus == 0
            || self.memory_gib == 0
            || identifiers.iter().any(|s| !safe_identifier(s))
            || !self.compartment.starts_with("ocid1.compartment.")
            || !self.instance.starts_with("ocid1.instance.")
            || !self.boot_volume.starts_with("ocid1.bootvolume.")
            || !self.bastion.starts_with("ocid1.bastion.")
            || self.private_ip.parse::<std::net::Ipv4Addr>().is_err()
            || !valid_archive_member(&self.object)
            || !valid_archive_member(&self.talos_member)
            || !valid_archive_member(&self.kube_member)
            || self.talos_member == self.kube_member
            || !safe_identifier(&self.talos_context)
            || [
                &self.archive_sha256,
                &self.talos_ca_sha256,
                &self.kube_ca_sha256,
            ]
            .iter()
            .any(|s| !valid_digest(s))
            || [
                &self.identity_file,
                &self.ssh_identity_file,
                &self.ssh_known_hosts_file,
                &self.oci_config_file,
            ]
            .iter()
            .any(|s| !Path::new(s).is_absolute() || s.chars().any(char::is_control))
        {
            return Err(AccessError::InvalidProfile);
        }
        Ok(())
    }

    pub fn verify_instance(&self, value: &Value) -> Result<(), AccessError> {
        let d = &value["data"];
        if d["id"] != self.instance
            || d["compartment-id"] != self.compartment
            || d["shape"] != self.shape
            || d["lifecycle-state"] != "RUNNING"
            || d["shape-config"]["ocpus"].as_f64() != Some(f64::from(self.ocpus))
            || d["shape-config"]["memory-in-gbs"].as_f64() != Some(f64::from(self.memory_gib))
            || d["source-details"]["boot-volume-id"] != self.boot_volume
        {
            return Err(AccessError::TargetMismatch);
        }
        Ok(())
    }

    pub fn verify_bastion(&self, value: &Value) -> Result<(), AccessError> {
        let d = &value["data"];
        if d["id"] != self.bastion
            || d["compartment-id"] != self.compartment
            || d["lifecycle-state"] != "ACTIVE"
        {
            return Err(AccessError::TargetMismatch);
        }
        Ok(())
    }
}

fn safe_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        && !s.starts_with('-')
}

pub fn valid_archive_member(value: &str) -> bool {
    !value.is_empty()
        && value
            .split('/')
            .all(|part| safe_identifier(part) && part != "." && part != "..")
}

pub fn valid_digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

pub fn verified_digest(bytes: &[u8], expected: &str) -> bool {
    valid_digest(expected) && format!("{:x}", Sha256::digest(bytes)) == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admits_only_exact_relative_archive_members() {
        assert!(valid_archive_member("seed/generated/talosconfig"));
        for path in [
            "",
            "/etc/passwd",
            "../key",
            "a/../key",
            "a//key",
            "-C",
            "a\\key",
        ] {
            assert!(!valid_archive_member(path), "{path}");
        }
    }

    #[test]
    fn verifies_archive_bytes_before_decryption() {
        let digest = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert!(verified_digest(b"abc", digest));
        assert!(!verified_digest(b"abd", digest));
        assert!(!verified_digest(b"abc", ""));
    }

    pub(crate) fn profile() -> Profile {
        serde_json::from_value(serde_json::json!({
            "schema": 1, "region": "ap-chuncheon-1", "compartment": "ocid1.compartment.prod",
            "instance": "ocid1.instance.a1", "boot_volume": "ocid1.bootvolume.a1",
            "shape": "VM.Standard.A1.Flex", "ocpus": 4, "memory_gib": 24,
            "private_ip": "10.0.0.227", "node_name": "seed", "bastion": "ocid1.bastion.seed",
            "namespace": "namespace", "bucket": "recovery", "object": "seed/archive.age",
            "object_version": "immutable-version", "archive_sha256": "a".repeat(64),
            "talos_member": "seed/talosconfig", "kube_member": "seed/kubeconfig",
            "talos_context": "seed", "talos_ca_sha256": "b".repeat(64), "kube_ca_sha256": "c".repeat(64),
            "identity_file": "/operator/identity", "ssh_identity_file": "/operator/ssh",
            "ssh_known_hosts_file": "/operator/known_hosts", "oci_config_file": "/operator/oci",
            "oci_profile": "default"
        })).unwrap()
    }

    #[test]
    fn profile_requires_pinned_references_and_absolute_local_paths() {
        let mut p = profile();
        assert_eq!(p.validate(), Ok(()));
        p.object_version.clear();
        assert_eq!(p.validate(), Err(AccessError::InvalidProfile));
        p = profile();
        p.identity_file = "~/key".into();
        assert_eq!(p.validate(), Err(AccessError::InvalidProfile));
        p = profile();
        p.region = "region/../../host".into();
        assert_eq!(p.validate(), Err(AccessError::InvalidProfile));
        p = profile();
        p.kube_member = p.talos_member.clone();
        assert_eq!(p.validate(), Err(AccessError::InvalidProfile));
    }

    #[test]
    fn refuses_wrong_instance_compartment_capacity_boot_volume_and_state() {
        let p = profile();
        let correct = serde_json::json!({"data": {
            "id": p.instance, "compartment-id": p.compartment, "shape": p.shape,
            "lifecycle-state": "RUNNING", "shape-config": {"ocpus": 4, "memory-in-gbs": 24},
            "source-details": {"boot-volume-id": p.boot_volume}
        }});
        assert_eq!(p.verify_instance(&correct), Ok(()));
        for field in [
            "id",
            "compartment-id",
            "shape",
            "lifecycle-state",
            "shape-config",
            "source-details",
        ] {
            let mut wrong = correct.clone();
            wrong["data"][field] = Value::Null;
            assert_eq!(p.verify_instance(&wrong), Err(AccessError::TargetMismatch));
        }
        let mut resized = correct;
        resized["data"]["shape-config"]["ocpus"] = serde_json::json!(2);
        assert_eq!(
            p.verify_instance(&resized),
            Err(AccessError::TargetMismatch)
        );
    }

    #[test]
    fn refuses_wrong_bastion_scope() {
        let p = profile();
        let mut value = serde_json::json!({"data": {"id": p.bastion,
            "compartment-id": p.compartment, "lifecycle-state": "ACTIVE"}});
        assert_eq!(p.verify_bastion(&value), Ok(()));
        value["data"]["compartment-id"] = serde_json::json!("other");
        assert_eq!(p.verify_bastion(&value), Err(AccessError::TargetMismatch));
    }
}
