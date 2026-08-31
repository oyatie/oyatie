use crate::{CloudComputeError, prefixed_token};

const KEY_PAIR_ID_PREFIX: &str = "key_";
const NODE_POOL_ID_PREFIX: &str = "np_";
const FUNCTION_INVOCATION_ID_PREFIX: &str = "fninv_";
const USER_DATA_URI_PREFIX: &str = "userdata/";
const OCI_IMAGE_PREFIX: &str = "oci://";
const QCOW2_IMAGE_PREFIX: &str = "qcow2://";
const FUNCTION_BUNDLE_PREFIX: &str = "function://";

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ImageRef {
    pub value: String,      // data_class: INTERNAL_ONLY
    pub kind: ImageRefKind, // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ImageRefKind {
    Oci,
    Qcow2,
    FunctionBundle,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct KeyPairId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct UserDataUri {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct NodePoolId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct FunctionName {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct InvocationId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct IdempotencyKey {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ControlPlaneVersion {
    pub value: String, // data_class: PUBLIC
}

impl ImageRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudComputeError> {
        let value = value.into();
        let kind = if value.starts_with(OCI_IMAGE_PREFIX) {
            ImageRefKind::Oci
        } else if value.starts_with(QCOW2_IMAGE_PREFIX) {
            ImageRefKind::Qcow2
        } else if value.starts_with(FUNCTION_BUNDLE_PREFIX) {
            ImageRefKind::FunctionBundle
        } else {
            return Err(CloudComputeError::InvalidImageRef);
        };
        let Some((uri, digest)) = value.rsplit_once("@sha256:") else {
            return Err(CloudComputeError::InvalidImageRef);
        };
        if uri.len() <= kind.prefix().len()
            || digest.len() != 64
            || !digest.bytes().all(|byte| byte.is_ascii_hexdigit())
            || value
                .bytes()
                .any(|byte| byte.is_ascii_control() || byte == b' ')
        {
            return Err(CloudComputeError::InvalidImageRef);
        }
        Ok(Self { value, kind })
    }

    pub const fn is_function_bundle(&self) -> bool {
        matches!(self.kind, ImageRefKind::FunctionBundle)
    }
}

impl ImageRefKind {
    const fn prefix(self) -> &'static str {
        match self {
            Self::Oci => OCI_IMAGE_PREFIX,
            Self::Qcow2 => QCOW2_IMAGE_PREFIX,
            Self::FunctionBundle => FUNCTION_BUNDLE_PREFIX,
        }
    }
}

impl KeyPairId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudComputeError> {
        prefixed_token(
            value.into(),
            KEY_PAIR_ID_PREFIX,
            CloudComputeError::InvalidKeyPairId,
        )
        .map(|value| Self { value })
    }
}

impl UserDataUri {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudComputeError> {
        prefixed_token(
            value.into(),
            USER_DATA_URI_PREFIX,
            CloudComputeError::InvalidUserDataUri,
        )
        .map(|value| Self { value })
    }
}

impl NodePoolId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudComputeError> {
        prefixed_token(
            value.into(),
            NODE_POOL_ID_PREFIX,
            CloudComputeError::InvalidNodePoolId,
        )
        .map(|value| Self { value })
    }
}

impl FunctionName {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudComputeError> {
        let value = value.into();
        if (3..=64).contains(&value.len())
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
            && !value.starts_with('-')
            && !value.ends_with('-')
        {
            Ok(Self { value })
        } else {
            Err(CloudComputeError::InvalidFunctionName)
        }
    }
}

impl InvocationId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudComputeError> {
        prefixed_token(
            value.into(),
            FUNCTION_INVOCATION_ID_PREFIX,
            CloudComputeError::InvalidInvocationId,
        )
        .map(|value| Self { value })
    }
}

impl IdempotencyKey {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudComputeError> {
        let value = value.into();
        if (16..=128).contains(&value.len())
            && !value
                .bytes()
                .any(|byte| byte.is_ascii_control() || byte == b' ')
        {
            Ok(Self { value })
        } else {
            Err(CloudComputeError::InvalidIdempotencyKey)
        }
    }
}
impl ControlPlaneVersion {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudComputeError> {
        let value = value.into();
        let Some(rest) = value.strip_prefix('v') else {
            return Err(CloudComputeError::InvalidControlPlaneVersion);
        };
        if rest.split('.').count() >= 3
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
        {
            Ok(Self { value })
        } else {
            Err(CloudComputeError::InvalidControlPlaneVersion)
        }
    }
}

pub const fn image_ref_kind_label(kind: ImageRefKind) -> &'static str {
    match kind {
        ImageRefKind::Oci => "oci",
        ImageRefKind::Qcow2 => "qcow2",
        ImageRefKind::FunctionBundle => "function_bundle",
    }
}
