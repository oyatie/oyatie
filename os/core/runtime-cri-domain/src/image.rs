//! OCI image references and the in-memory model of image pull/unpack.
//!
//! Mirrors `pkg/images` and the containerd image client: a reference is parsed
//! into registry/repository/tag-or-digest, and an [`Image`] tracks the lifecycle
//! of being pulled and unpacked into a snapshot.

use os_kernel::error::{Error, Result};

/// A parsed OCI image reference such as
/// `registry.k8s.io/pause:3.9` or `docker.io/library/etcd@sha256:abc...`.
///
/// Modeled on `containerd/reference` / `distribution/reference` parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageRef {
    /// Registry host (e.g. `registry.k8s.io`). Defaults to `docker.io`.
    pub registry: String,
    /// Repository path (e.g. `library/etcd`).
    pub repository: String,
    /// Tag (e.g. `3.5.9`) if the reference is tag-based.
    pub tag: Option<String>,
    /// Digest (e.g. `sha256:...`) if the reference is digest-pinned.
    pub digest: Option<String>,
}

impl ImageRef {
    /// Default registry when none is specified.
    pub const DEFAULT_REGISTRY: &'static str = "docker.io";
    /// Default tag when neither tag nor digest is specified.
    pub const DEFAULT_TAG: &'static str = "latest";

    /// Parse an image reference string.
    ///
    /// Rules (a pragmatic subset of distribution/reference):
    /// * The first path component is treated as the registry iff it contains a
    ///   `.` or `:` or equals `localhost`.
    /// * `@sha256:...` denotes a digest, `:tag` a tag.
    pub fn parse(s: &str) -> Result<Self> {
        if s.is_empty() {
            return Err(Error::parse("empty image reference"));
        }

        // Split off digest first (it always comes last, after '@').
        let (name_and_tag, digest) = match s.split_once('@') {
            Some((lhs, dig)) => {
                if !dig.starts_with("sha256:") || dig.len() != "sha256:".len() + 64 {
                    return Err(Error::parse("invalid digest"));
                }
                (lhs, Some(dig.to_string()))
            }
            None => (s, None),
        };

        // Determine the registry by inspecting the first '/'-separated element.
        let (registry, remainder) = match name_and_tag.split_once('/') {
            Some((first, rest))
                if first.contains('.') || first.contains(':') || first == "localhost" =>
            {
                (first.to_string(), rest)
            }
            _ => (Self::DEFAULT_REGISTRY.to_string(), name_and_tag),
        };

        // Split tag off the remainder (a ':' after the last '/').
        let last_slash = remainder.rfind('/').map_or(0, |i| i + 1);
        let last_component = &remainder[last_slash..];
        let (repository, tag) = match last_component.split_once(':') {
            Some((before_colon, t)) => {
                if t.is_empty() {
                    return Err(Error::parse("empty tag"));
                }
                let cut = last_slash + before_colon.len();
                (remainder[..cut].to_string(), Some(t.to_string()))
            }
            None => (remainder.to_string(), None),
        };

        if repository.is_empty() {
            return Err(Error::parse("empty repository"));
        }

        Ok(ImageRef {
            registry,
            repository,
            tag,
            digest,
        })
    }

    /// Whether the reference is pinned to a content digest.
    pub fn is_pinned(&self) -> bool {
        self.digest.is_some()
    }

    /// The effective tag, falling back to `latest` when neither tag nor digest
    /// is set.
    pub fn effective_tag(&self) -> &str {
        match (&self.tag, &self.digest) {
            (Some(t), _) => t,
            (None, Some(_)) => "",
            (None, None) => Self::DEFAULT_TAG,
        }
    }

    /// Canonical, fully-qualified reference string.
    pub fn canonical(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.registry);
        out.push('/');
        out.push_str(&self.repository);
        if let Some(d) = &self.digest {
            out.push('@');
            out.push_str(d);
        } else {
            out.push(':');
            out.push_str(self.effective_tag());
        }
        out
    }
}

/// An OCI content descriptor, mirroring `ocispec.Descriptor`.
///
/// Identifies a blob (manifest, config, or layer) by media type, digest, and
/// size — the content-addressable unit the content store is keyed on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Descriptor {
    /// OCI media type, e.g. `application/vnd.oci.image.layer.v1.tar+gzip`.
    pub media_type: String,
    /// Content digest, e.g. `sha256:...`.
    pub digest: String,
    /// Size of the referenced blob in bytes.
    pub size: u64,
}

impl Descriptor {
    /// The OCI media type for an image manifest.
    pub const MANIFEST: &'static str = "application/vnd.oci.image.manifest.v1+json";
    /// The OCI media type for an image config.
    pub const CONFIG: &'static str = "application/vnd.oci.image.config.v1+json";
    /// The OCI media type for a gzipped layer.
    pub const LAYER_GZIP: &'static str = "application/vnd.oci.image.layer.v1.tar+gzip";

    /// Validate the descriptor (digest shape and non-empty media type).
    pub fn validate(&self) -> Result<()> {
        if self.media_type.is_empty() {
            return Err(Error::invalid("descriptor media type required"));
        }
        validate_digest(&self.digest)
    }
}

/// Validate a `sha256:<64-hex>` digest string.
pub fn validate_digest(digest: &str) -> Result<()> {
    let hex = digest
        .strip_prefix("sha256:")
        .ok_or_else(|| Error::parse("digest must start with sha256:"))?;
    if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(Error::parse("digest must be 64 hex chars"));
    }
    Ok(())
}

/// An OCI image manifest: a config descriptor plus ordered layer descriptors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// The image config blob descriptor.
    pub config: Descriptor,
    /// Layer descriptors, base-most first.
    pub layers: Vec<Descriptor>,
}

impl Manifest {
    /// Build and validate a manifest.
    pub fn new(config: Descriptor, layers: Vec<Descriptor>) -> Result<Self> {
        config.validate()?;
        if layers.is_empty() {
            return Err(Error::invalid("manifest must have at least one layer"));
        }
        for l in &layers {
            l.validate()?;
        }
        Ok(Manifest { config, layers })
    }

    /// Total size of all layers plus the config blob.
    pub fn total_size(&self) -> u64 {
        self.config.size + self.layers.iter().map(|l| l.size).sum::<u64>()
    }

    /// The layer digests, base-most first.
    pub fn layer_digests(&self) -> Vec<String> {
        self.layers.iter().map(|l| l.digest.clone()).collect()
    }
}

/// Lifecycle state of an image inside the content store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageState {
    /// Reference is known but no content has been fetched.
    Missing,
    /// Manifest + layers are present in the content store.
    Pulled,
    /// Layers have been unpacked into a snapshot, ready to run.
    Unpacked,
}

/// An image tracked by the runtime, including a parsed reference, its content
/// state, and the digests of its layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    /// The parsed reference.
    pub reference: ImageRef,
    /// Current content/snapshot state.
    pub state: ImageState,
    /// Layer digests (top-most last), populated on pull.
    pub layers: Vec<String>,
    /// Total size in bytes of the pulled content.
    pub size: u64,
}

impl Image {
    /// Create a freshly-referenced, not-yet-pulled image.
    pub fn new(reference: ImageRef) -> Self {
        Image {
            reference,
            state: ImageState::Missing,
            layers: Vec::new(),
            size: 0,
        }
    }

    /// Record a successful pull. Fails if there are no layers.
    pub fn mark_pulled(&mut self, layers: Vec<String>, size: u64) -> Result<()> {
        if layers.is_empty() {
            return Err(Error::invalid("image must have at least one layer"));
        }
        self.layers = layers;
        self.size = size;
        self.state = ImageState::Pulled;
        Ok(())
    }

    /// Record a successful pull from a parsed manifest, deriving the layer
    /// digests and total size from the descriptors.
    pub fn mark_pulled_from_manifest(&mut self, manifest: &Manifest) -> Result<()> {
        self.mark_pulled(manifest.layer_digests(), manifest.total_size())
    }

    /// Unpack the image into a snapshot. Only valid once pulled.
    pub fn mark_unpacked(&mut self) -> Result<()> {
        if self.state != ImageState::Pulled {
            return Err(Error::invalid_state("image must be pulled before unpack"));
        }
        self.state = ImageState::Unpacked;
        Ok(())
    }

    /// Whether the image is ready to be used to create a container.
    pub fn is_runnable(&self) -> bool {
        self.state == ImageState::Unpacked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_reference_with_tag() {
        let r = ImageRef::parse("registry.k8s.io/pause:3.9").unwrap();
        assert_eq!(r.registry, "registry.k8s.io");
        assert_eq!(r.repository, "pause");
        assert_eq!(r.tag.as_deref(), Some("3.9"));
        assert!(!r.is_pinned());
        assert_eq!(r.canonical(), "registry.k8s.io/pause:3.9");
    }

    #[test]
    fn parse_default_registry_and_tag() {
        let r = ImageRef::parse("library/etcd").unwrap();
        assert_eq!(r.registry, "docker.io");
        assert_eq!(r.repository, "library/etcd");
        assert_eq!(r.effective_tag(), "latest");
        assert_eq!(r.canonical(), "docker.io/library/etcd:latest");
    }

    #[test]
    fn parse_digest_pinned() {
        let dig = "sha256:".to_string() + &"a".repeat(64);
        let s = "registry.k8s.io/etcd@".to_string() + &dig;
        let r = ImageRef::parse(&s).unwrap();
        assert!(r.is_pinned());
        assert_eq!(r.digest.as_deref(), Some(dig.as_str()));
        assert!(r.canonical().contains('@'));
    }

    #[test]
    fn parse_rejects_bad_input() {
        assert!(ImageRef::parse("").is_err());
        assert!(ImageRef::parse("repo@sha256:short").is_err());
        assert!(ImageRef::parse("repo:").is_err());
    }

    #[test]
    fn image_lifecycle_transitions() {
        let mut img = Image::new(ImageRef::parse("registry.k8s.io/pause:3.9").unwrap());
        assert_eq!(img.state, ImageState::Missing);
        assert!(img.mark_unpacked().is_err());
        img.mark_pulled(vec!["sha256:l1".to_string()], 1024)
            .unwrap();
        assert_eq!(img.state, ImageState::Pulled);
        assert!(!img.is_runnable());
        img.mark_unpacked().unwrap();
        assert!(img.is_runnable());
    }

    #[test]
    fn pull_requires_layers() {
        let mut img = Image::new(ImageRef::parse("a/b:1").unwrap());
        assert!(img.mark_pulled(Vec::new(), 0).is_err());
    }

    fn d(digest_byte: char, size: u64, mt: &str) -> Descriptor {
        Descriptor {
            media_type: mt.to_string(),
            digest: format!("sha256:{}", digest_byte.to_string().repeat(64)),
            size,
        }
    }

    #[test]
    fn validate_digest_shape() {
        assert!(validate_digest(&format!("sha256:{}", "a".repeat(64))).is_ok());
        assert!(validate_digest("md5:abc").is_err());
        assert!(validate_digest("sha256:short").is_err());
        assert!(validate_digest(&format!("sha256:{}", "g".repeat(64))).is_err());
    }

    #[test]
    fn descriptor_validation() {
        let good = d('a', 10, Descriptor::LAYER_GZIP);
        assert!(good.validate().is_ok());
        let bad = Descriptor {
            media_type: String::new(),
            ..good.clone()
        };
        assert!(bad.validate().is_err());
    }

    #[test]
    fn manifest_aggregates_sizes_and_layers() {
        let config = d('0', 5, Descriptor::CONFIG);
        let l1 = d('1', 100, Descriptor::LAYER_GZIP);
        let l2 = d('2', 200, Descriptor::LAYER_GZIP);
        let m = Manifest::new(config, vec![l1.clone(), l2.clone()]).unwrap();
        assert_eq!(m.total_size(), 5 + 100 + 200);
        assert_eq!(m.layer_digests(), vec![l1.digest, l2.digest]);
    }

    #[test]
    fn manifest_requires_layers() {
        let config = d('0', 5, Descriptor::CONFIG);
        assert!(Manifest::new(config, Vec::new()).is_err());
    }

    #[test]
    fn pull_from_manifest_derives_layers_and_size() {
        let config = d('0', 5, Descriptor::CONFIG);
        let l1 = d('1', 100, Descriptor::LAYER_GZIP);
        let m = Manifest::new(config, vec![l1.clone()]).unwrap();
        let mut img = Image::new(ImageRef::parse("registry.k8s.io/etcd:3.5").unwrap());
        img.mark_pulled_from_manifest(&m).unwrap();
        assert_eq!(img.state, ImageState::Pulled);
        assert_eq!(img.layers, vec![l1.digest]);
        assert_eq!(img.size, 105);
    }
}
