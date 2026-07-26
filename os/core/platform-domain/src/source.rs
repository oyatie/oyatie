//! [`ConfigSource`]: the faithful, data-only description of *where* a platform's
//! machine config lives — an HTTP endpoint (URL + headers), a removable disk
//! (ISO/CD-ROM label + in-volume path), or a kernel command-line value.
//!
//! These mirror the concrete fetch shapes in each
//! `platform/<provider>/{,metadata}.go` file upstream. Keeping them as data
//! lets the unit tests assert the exact endpoint, header set, label and path
//! without performing any I/O.

use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;

/// A single HTTP request header (name + value), e.g. `Metadata-Flavor: Google`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Header {
    /// Header name as sent on the wire (case preserved from the Go source).
    pub name: String,
    /// Header value.
    pub value: String,
}

impl Header {
    /// Construct a header from name/value pair.
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Header {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// Where a platform's machine-config bytes come from.
///
/// Each variant captures the faithful shape from the corresponding Talos Go
/// source so it can be matched/asserted exactly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigSource {
    /// An HTTP(S) metadata/user-data endpoint with required request headers.
    ///
    /// Examples: AWS IMDS user-data, GCP metadata server attribute, Azure IMDS,
    /// NoCloud network seed (`meta-base/user-data`).
    Http {
        /// Fully-formed request URL.
        url: String,
        /// Request headers that must be present (order-insensitive on the wire,
        /// but preserved here for stable assertions).
        headers: Vec<Header>,
    },
    /// A removable disk / CD-ROM identified by its filesystem volume label,
    /// from which a file at `path` is read.
    ///
    /// Examples: NoCloud `cidata` ISO with `user-data`; Azure provisioning
    /// CD-ROM with `ovf-env.xml`; Metal `metal-iso` with `config.yaml`.
    Disk {
        /// Candidate filesystem volume labels (Talos probes several casings).
        labels: Vec<String>,
        /// File path read from the mounted volume root.
        path: String,
    },
    /// A value taken from the kernel command line (`talos.config=<value>`).
    ///
    /// The `value` is the raw cmdline argument; for metal it is typically a URL
    /// (or the sentinel `metal-iso`, or `none`).
    KernelCmdline {
        /// Kernel parameter name, e.g. `talos.config`.
        param: String,
        /// The parameter's value.
        value: String,
    },
}

impl ConfigSource {
    /// Build an [`ConfigSource::Http`] source.
    pub fn http(url: impl Into<String>, headers: Vec<Header>) -> Self {
        ConfigSource::Http {
            url: url.into(),
            headers,
        }
    }

    /// Build an [`ConfigSource::Http`] source with no headers.
    pub fn http_no_headers(url: impl Into<String>) -> Self {
        ConfigSource::Http {
            url: url.into(),
            headers: Vec::new(),
        }
    }

    /// Build a [`ConfigSource::Disk`] source.
    pub fn disk(labels: &[&str], path: impl Into<String>) -> Self {
        ConfigSource::Disk {
            labels: labels.iter().map(|s| (*s).to_owned()).collect(),
            path: path.into(),
        }
    }

    /// Build a [`ConfigSource::KernelCmdline`] source.
    pub fn kernel_cmdline(param: impl Into<String>, value: impl Into<String>) -> Self {
        ConfigSource::KernelCmdline {
            param: param.into(),
            value: value.into(),
        }
    }

    /// The endpoint URL, for HTTP sources.
    pub fn url(&self) -> Option<&str> {
        match self {
            ConfigSource::Http { url, .. } => Some(url),
            _ => None,
        }
    }

    /// The request headers, for HTTP sources (empty slice otherwise).
    pub fn headers(&self) -> &[Header] {
        match self {
            ConfigSource::Http { headers, .. } => headers,
            _ => &[],
        }
    }

    /// Look up a header value by (case-insensitive) name.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers()
            .iter()
            .find(|h| h.name.eq_ignore_ascii_case(name))
            .map(|h| h.value.as_str())
    }

    /// The in-volume file path, for disk sources.
    pub fn path(&self) -> Option<&str> {
        match self {
            ConfigSource::Disk { path, .. } => Some(path),
            ConfigSource::KernelCmdline { .. } | ConfigSource::Http { .. } => None,
        }
    }

    /// The candidate volume labels, for disk sources.
    pub fn labels(&self) -> &[String] {
        match self {
            ConfigSource::Disk { labels, .. } => labels,
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn http_accessors() {
        let s = ConfigSource::http(
            "http://example/user-data",
            vec![Header::new("Metadata", "true")],
        );
        assert_eq!(s.url(), Some("http://example/user-data"));
        assert_eq!(s.header("metadata"), Some("true"));
        assert_eq!(s.header("Missing"), None);
        assert!(s.path().is_none());
    }

    #[test]
    fn disk_accessors() {
        let s = ConfigSource::disk(&["cidata", "CIDATA"], "user-data");
        assert_eq!(s.path(), Some("user-data"));
        assert_eq!(s.labels(), &["cidata", "CIDATA"]);
        assert!(s.url().is_none());
        assert!(s.headers().is_empty());
    }

    #[test]
    fn cmdline_accessor() {
        let s = ConfigSource::kernel_cmdline("talos.config", "https://x/config.yaml");
        assert!(matches!(s, ConfigSource::KernelCmdline { .. }));
        assert!(s.url().is_none());
    }
}
