//! Source-guided image-cache runtime/mount orchestration.
//!
//! This is a host-safe Rust model of Talos
//! `internal/app/machined/pkg/controllers/cri/ImageCacheConfigController`.
//! The real controller writes COSI resources and copies files between mounted
//! volumes. This module keeps those effects as explicit plans so controller
//! logic, mount request read-only rules, finalizer intent, copy status, and root
//! ordering remain testable without privileged mounts or containerd.

use std::{
    borrow::Cow,
    collections::BTreeSet,
    fmt, fs,
    io::{self, Read},
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use os_block_domain::{
    MountRequestSpec, VolumeMountRequestResource, VolumeMountRequestSpec,
    VolumeMountStatusResource, VolumePhase, VolumeStatus, VolumeStatusResource,
    mount::{volume_mount_request_key, volume_mount_status_id, volume_mount_status_key},
};
use os_kernel::ResourceId;
use os_cosi_domain::{
    Controller as CosiController, ControllerError, Event, EventKind, Input, Metadata, Output,
    ReconcileContext, ReconcileResult, Resource, ResourceKind, Spec, State, StoreError,
    StoreResult,
};

use crate::cri::{MACHINE_CONFIG_ACTIVE_ID, MACHINE_CONFIG_NAMESPACE, MACHINE_CONFIG_TYPE};

/// Talos CRI `ImageCacheConfig` resource type.
///
/// Source: `pkg/machinery/resources/cri/image_cache_config.go`.
pub const IMAGE_CACHE_CONFIG_TYPE: &str = "ImageCacheConfigs.cri.talos.dev";

/// Talos CRI resource namespace.
///
/// Source: `pkg/machinery/resources/cri/cri.go`.
pub const IMAGE_CACHE_NAMESPACE: &str = "cri";

/// Talos CRI `ImageCacheConfig` singleton id.
pub const IMAGE_CACHE_CONFIG_ID: &str = "image-cache";

/// Boot-owned Rust bridge resource type for source controller copy memory.
pub const IMAGE_CACHE_COPY_STATE_TYPE: &str = "ImageCacheCopyStates.cri.talos.dev";

/// Singleton id for the boot-owned image-cache copy completion marker.
pub const IMAGE_CACHE_COPY_STATE_ID: &str = "image-cache-copy";

/// Source controller name.
pub const IMAGE_CACHE_CONTROLLER_NAME: &str = "cri.ImageCacheConfigController";

/// Talos v1alpha1 resource namespace used by source Service resources.
///
/// Source: Talos v1.13.0 `pkg/machinery/resources/v1alpha1/v1alpha1.go`.
pub const V1ALPHA1_NAMESPACE: &str = "runtime";

/// Talos v1alpha1 `Service` resource type.
///
/// Source: Talos v1.13.0 `pkg/machinery/resources/v1alpha1/service.go`.
pub const V1ALPHA1_SERVICE_TYPE: &str = "Services.v1alpha1.talos.dev";

/// Talos block `VolumeConfig` resource type.
///
/// Source: Talos v1.13.0 `pkg/machinery/resources/block/volume_config.go`.
pub const VOLUME_CONFIG_TYPE: &str = os_block_domain::VOLUME_CONFIG_TYPE;

/// Disk-backed image-cache volume id (`constants.ImageCachePartitionLabel`).
pub const IMAGE_CACHE_DISK_VOLUME_ID: &str = "IMAGECACHE";

/// ISO-backed sidecar image-cache volume id.
pub const IMAGE_CACHE_ISO_VOLUME_ID: &str = "IMAGECACHE-ISO";

/// Disk image-cache mount target.
pub const IMAGE_CACHE_DISK_MOUNT_POINT: &str = "/system/imagecache/disk";

/// ISO image-cache mount target.
pub const IMAGE_CACHE_ISO_MOUNT_POINT: &str = "/system/imagecache/iso";

/// Source minimum image-cache partition size: 500 MiB.
pub const MIN_IMAGE_CACHE_SIZE_BYTES: u64 = 500 * 1024 * 1024;

/// Source maximum image-cache partition size: 1 GiB.
pub const MAX_IMAGE_CACHE_SIZE_BYTES: u64 = 1024 * 1024 * 1024;

/// Talos registryd system service id (`services.RegistryID`).
pub const REGISTRYD_SERVICE_ID: &str = "registryd";

/// Talos registryd loopback listen address (`constants.RegistrydListenAddress`).
pub const REGISTRYD_LISTEN_ADDRESS: &str = "127.0.0.1:3172";

/// Talos registryd health-check path used by `services.NewRegistryD()`.
pub const REGISTRYD_HEALTH_PATH: &str = "/healthz";

/// Source health URL used by machined's registryd service health check.
pub const REGISTRYD_HEALTH_URL: &str = "http://127.0.0.1:3172/healthz";

/// Return the source-shaped registryd health URL.
pub fn registryd_health_url() -> &'static str {
    REGISTRYD_HEALTH_URL
}

/// Source COSI kind for the active MachineConfig input.
pub fn machine_config_kind() -> ResourceKind {
    ResourceKind::new(MACHINE_CONFIG_NAMESPACE, MACHINE_CONFIG_TYPE)
}

/// Source COSI kind for the registryd v1alpha1 Service input.
pub fn registryd_service_kind() -> ResourceKind {
    ResourceKind::new(V1ALPHA1_NAMESPACE, V1ALPHA1_SERVICE_TYPE)
}

/// Source COSI kind for block VolumeConfig shared output.
pub fn volume_config_kind() -> ResourceKind {
    ResourceKind::new(os_block_domain::mount::BLOCK_NAMESPACE, VOLUME_CONFIG_TYPE)
}

/// Minimal HTTP response model for the registryd health endpoint.
///
/// Talos `registryd` is started as an in-process service and machined checks
/// `http://127.0.0.1:3172/healthz` before treating it as healthy. This model
/// intentionally covers only that health response instead of inventing a full
/// registry API surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrydHttpResponse {
    /// Numeric HTTP status code.
    pub status_code: u16,
    /// Stable reason phrase for the status line.
    pub reason: &'static str,
}

impl RegistrydHttpResponse {
    /// Return the source-shaped HTTP/1.1 status line.
    pub fn status_line(&self) -> String {
        format!("HTTP/1.1 {} {}", self.status_code, self.reason)
    }

    /// True when machined's simple health check would accept the response.
    pub fn is_success(&self) -> bool {
        RegistrydHealthProbe::source().accepts_status(self.status_code)
    }
}

/// Host-safe registryd content response model.
///
/// This captures the headers and body decisions Talos registryd makes after a
/// cache hit without opening a socket or depending on containerd's content
/// store. Error responses intentionally carry no content metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrydContentResponse {
    /// Numeric HTTP status code.
    pub status_code: u16,
    /// Stable reason phrase for the status line.
    pub reason: &'static str,
    /// Source `Content-Length` value for successful content responses.
    pub content_length: Option<usize>,
    /// Source `Content-Range` value for partial content responses.
    pub content_range: Option<String>,
    /// Source `Accept-Ranges` value emitted by `http.ServeContent`.
    pub accept_ranges: Option<String>,
    /// Source `Docker-Content-Digest` header for successful content responses.
    pub docker_content_digest: Option<String>,
    /// Source-shaped `Content-Type` when Talos or `net/http` supplies one.
    pub content_type: Option<String>,
    /// Source `Last-Modified` value emitted by `http.ServeContent`.
    pub last_modified: Option<String>,
    /// Source error-response content type guard emitted by `http.Error`.
    pub x_content_type_options: Option<String>,
    /// Response bytes. `HEAD` responses keep headers but omit the body.
    pub body: Vec<u8>,
    /// Source bytes retained for range math when headers omit the success body.
    pub range_source: Option<Vec<u8>>,
    /// Absolute-but-not-canonicalized cache path that supplied the body bytes.
    pub content_path: Option<PathBuf>,
}

/// Request headers that influence source-shaped registryd content serving.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RegistrydSourceRequestHeaders<'a> {
    /// Source `Range` request header.
    pub range: Option<&'a str>,
    /// Source `If-Match` request precondition.
    pub if_match: Option<&'a str>,
    /// Source `If-Unmodified-Since` request precondition.
    pub if_unmodified_since: Option<&'a str>,
    /// Source `If-None-Match` request precondition.
    pub if_none_match: Option<&'a str>,
    /// Source `If-Modified-Since` request precondition.
    pub if_modified_since: Option<&'a str>,
    /// Source `If-Range` request validator.
    pub if_range: Option<&'a str>,
}

impl RegistrydContentResponse {
    fn status(status_code: u16) -> Self {
        Self {
            status_code,
            reason: registryd_status_reason(status_code),
            content_length: None,
            content_range: None,
            accept_ranges: None,
            docker_content_digest: None,
            content_type: None,
            last_modified: None,
            x_content_type_options: None,
            body: Vec::new(),
            range_source: None,
            content_path: None,
        }
    }

    fn error(status_code: u16) -> Self {
        Self::status(status_code)
    }

    fn ok_manifest(
        method: &str,
        digest: String,
        media_type: String,
        bytes: Vec<u8>,
        last_modified: Option<String>,
        content_path: PathBuf,
    ) -> Self {
        let body = if method == "HEAD" {
            Vec::new()
        } else {
            bytes.clone()
        };

        Self {
            status_code: 200,
            reason: registryd_status_reason(200),
            content_length: Some(bytes.len()),
            content_range: None,
            accept_ranges: (method != "HEAD").then(|| "bytes".to_string()),
            docker_content_digest: Some(digest),
            content_type: Some(media_type),
            last_modified: (method != "HEAD").then_some(last_modified).flatten(),
            x_content_type_options: None,
            body,
            range_source: (method != "HEAD").then(|| bytes.clone()),
            content_path: Some(content_path),
        }
    }

    fn ok_blob(
        method: &str,
        digest: String,
        bytes: Vec<u8>,
        last_modified: Option<String>,
        content_path: PathBuf,
    ) -> Self {
        let body = if method == "HEAD" {
            Vec::new()
        } else {
            bytes.clone()
        };

        Self {
            status_code: 200,
            reason: registryd_status_reason(200),
            content_length: Some(bytes.len()),
            content_range: None,
            accept_ranges: Some("bytes".to_string()),
            docker_content_digest: Some(digest),
            content_type: Some(registryd_blob_content_type(&bytes).to_string()),
            last_modified,
            x_content_type_options: None,
            body,
            range_source: Some(bytes.clone()),
            content_path: Some(content_path),
        }
    }

    /// Project source-shaped HTTP headers for a content response.
    ///
    /// Talos registryd writes `Content-Length` and `Docker-Content-Digest` for
    /// successful cached content responses, plus modeled `Last-Modified` and
    /// `Content-Type` when the source writer supplies them.
    /// Header order follows the source handler writes and remains stable for
    /// the future PID1 socket-backed response writer.
    pub fn source_http_headers(&self) -> Vec<(&'static str, String)> {
        let mut headers = Vec::new();

        if let Some(content_length) = self.content_length {
            headers.push(("Content-Length", content_length.to_string()));
        }
        if let Some(digest) = &self.docker_content_digest {
            headers.push(("Docker-Content-Digest", digest.clone()));
        }
        if let Some(last_modified) = &self.last_modified {
            headers.push(("Last-Modified", last_modified.clone()));
        }
        if let Some(content_type) = &self.content_type {
            headers.push(("Content-Type", content_type.clone()));
        }
        if let Some(x_content_type_options) = &self.x_content_type_options {
            headers.push(("X-Content-Type-Options", x_content_type_options.clone()));
        }
        if let Some(content_range) = &self.content_range {
            headers.push(("Content-Range", content_range.clone()));
        }
        if let Some(accept_ranges) = &self.accept_ranges {
            headers.push(("Accept-Ranges", accept_ranges.clone()));
        }

        headers
    }

    /// Render response bytes after applying a source-shaped byte range.
    ///
    /// Talos registryd delegates cached blob/manifests GET serving to
    /// `http.ServeContent`, which emits `206 Partial Content` for valid byte
    /// ranges and `416 Requested Range Not Satisfiable` for invalid or
    /// no-overlap byte ranges.
    pub fn source_http_response_bytes_for_range(&self, range_header: Option<&str>) -> Vec<u8> {
        self.source_http_response_bytes_for_request_headers(RegistrydSourceRequestHeaders {
            range: range_header,
            ..RegistrydSourceRequestHeaders::default()
        })
    }

    /// Render response bytes after applying source-shaped request headers.
    pub fn source_http_response_bytes_for_request_headers(
        &self,
        headers: RegistrydSourceRequestHeaders<'_>,
    ) -> Vec<u8> {
        match headers.if_match {
            Some(value) if !self.source_if_match_satisfied(value) => {
                return self.source_precondition_failed_response_bytes();
            }
            None => {
                if headers
                    .if_unmodified_since
                    .is_some_and(|value| self.source_if_unmodified_since_rejects(value))
                {
                    return self.source_precondition_failed_response_bytes();
                }
            }
            Some(_) => {}
        }

        match headers.if_none_match {
            Some(value) if self.source_if_none_match_not_modified(value) => {
                return self.source_not_modified_response_bytes();
            }
            None => {
                if let Some(response) = headers
                    .if_modified_since
                    .and_then(|value| self.source_not_modified_response(value))
                {
                    return response.source_http_response_bytes();
                }
            }
            Some(_) => {}
        }

        let range_header = self.source_range_header_after_if_range(headers.range, headers.if_range);
        match range_header.and_then(|range_header| self.source_byte_range_response(range_header)) {
            Some(response) => response.source_http_response_bytes(),
            None => self.source_http_response_bytes(),
        }
    }

    fn source_range_header_after_if_range<'a>(
        &self,
        range_header: Option<&'a str>,
        if_range: Option<&str>,
    ) -> Option<&'a str> {
        let range_header = range_header?;
        match if_range {
            Some(value) if !self.source_if_range_matches(value) => None,
            _ => Some(range_header),
        }
    }

    fn source_if_range_matches(&self, if_range: &str) -> bool {
        let Some(last_modified) = self.last_modified.as_deref() else {
            return false;
        };
        let Some(content_seconds) = registryd_http_time_unix_seconds(last_modified) else {
            return false;
        };

        registryd_http_time_unix_seconds(if_range) == Some(content_seconds)
    }

    fn source_if_match_satisfied(&self, if_match: &str) -> bool {
        if self.status_code != 200 {
            return false;
        }

        let mut remaining = if_match.trim();
        while !remaining.is_empty() {
            remaining = remaining.trim_start();
            if let Some(rest) = remaining.strip_prefix(',') {
                remaining = rest;
                continue;
            }
            if remaining.starts_with('*') {
                return true;
            }
            let Some((_, rest)) = registryd_http_scan_etag(remaining) else {
                return false;
            };
            remaining = rest;
        }

        false
    }

    fn source_if_none_match_not_modified(&self, if_none_match: &str) -> bool {
        if self.status_code != 200 {
            return false;
        }

        let mut remaining = if_none_match.trim();
        while !remaining.is_empty() {
            remaining = remaining.trim_start();
            if let Some(rest) = remaining.strip_prefix(',') {
                remaining = rest;
                continue;
            }
            if remaining.starts_with('*') {
                return true;
            }
            let Some((_, rest)) = registryd_http_scan_etag(remaining) else {
                return false;
            };
            remaining = rest;
        }

        false
    }

    fn source_not_modified_response_shape(&self) -> Self {
        let mut response = self.clone();
        response.status_code = 304;
        response.reason = registryd_status_reason(304);
        response.content_length = None;
        response.content_range = None;
        response.accept_ranges = None;
        response.content_type = None;
        response.x_content_type_options = None;
        response.body.clear();
        response.range_source = None;
        response
    }

    fn source_not_modified_response_bytes(&self) -> Vec<u8> {
        self.source_not_modified_response_shape()
            .source_http_response_bytes()
    }

    fn source_precondition_failed_response_shape(&self) -> Self {
        let mut response = self.clone();
        response.status_code = 412;
        response.reason = registryd_status_reason(412);
        response.content_length = None;
        response.content_range = None;
        response.accept_ranges = None;
        response.content_type = None;
        response.x_content_type_options = None;
        response.body.clear();
        response.range_source = None;
        response
    }

    fn source_precondition_failed_response_bytes(&self) -> Vec<u8> {
        self.source_precondition_failed_response_shape()
            .source_http_response_bytes()
    }

    fn source_if_unmodified_since_rejects(&self, if_unmodified_since: &str) -> bool {
        if self.status_code != 200 {
            return false;
        }

        let Some(last_modified) = self.last_modified.as_deref() else {
            return false;
        };
        let Some(content_seconds) = registryd_http_time_unix_seconds(last_modified) else {
            return false;
        };
        let Some(request_seconds) = registryd_http_time_unix_seconds(if_unmodified_since) else {
            return false;
        };

        content_seconds > request_seconds
    }

    fn source_not_modified_response(&self, if_modified_since: &str) -> Option<Self> {
        if self.status_code != 200 {
            return None;
        }

        let last_modified = self.last_modified.as_deref()?;
        let content_seconds = registryd_http_time_unix_seconds(last_modified)?;
        let request_seconds = registryd_http_time_unix_seconds(if_modified_since)?;
        if content_seconds > request_seconds {
            return None;
        }

        Some(self.source_not_modified_response_shape())
    }

    fn source_byte_range_response(&self, range_header: &str) -> Option<Self> {
        if self.status_code != 200 {
            return None;
        }

        let source = self.range_source.as_deref().unwrap_or(&self.body);
        if source.is_empty() {
            return None;
        }

        let len = source.len();
        match registryd_byte_range_response(range_header, len)? {
            RegistrydByteRangeResponse::Body { start, length } => {
                let body = source[start..start + length].to_vec();
                let mut response = self.clone();
                response.status_code = 206;
                response.reason = registryd_status_reason(206);
                response.content_length = Some(body.len());
                response.content_range =
                    Some(format!("bytes {start}-{}/{}", start + length - 1, len));
                response.body = if self.body.is_empty() {
                    Vec::new()
                } else {
                    body
                };
                Some(response)
            }
            RegistrydByteRangeResponse::Multipart(ranges) => {
                let boundary = registryd_multipart_boundary(self.docker_content_digest.as_deref());
                let multipart_body = registryd_multipart_body(
                    &ranges,
                    source,
                    len,
                    self.content_type.as_deref(),
                    &boundary,
                );
                let mut response = self.clone();
                response.status_code = 206;
                response.reason = registryd_status_reason(206);
                response.content_length = Some(multipart_body.len());
                response.content_range = None;
                response.content_type = Some(format!("multipart/byteranges; boundary={boundary}"));
                response.body = if self.body.is_empty() {
                    Vec::new()
                } else {
                    multipart_body
                };
                Some(response)
            }
            RegistrydByteRangeResponse::NoOverlap => Some(self.source_range_error_response(
                b"invalid range: failed to overlap\n",
                Some(format!("bytes */{len}")),
            )),
            RegistrydByteRangeResponse::Invalid => {
                Some(self.source_range_error_response(b"invalid range\n", None))
            }
        }
    }

    fn source_range_error_response(
        &self,
        message: &'static [u8],
        content_range: Option<String>,
    ) -> Self {
        let mut response = self.clone();
        response.status_code = 416;
        response.reason = registryd_status_reason(416);
        response.content_length = Some(message.len());
        response.content_range = content_range;
        response.accept_ranges = None;
        response.last_modified = None;
        response.content_type = Some("text/plain; charset=utf-8".to_string());
        response.x_content_type_options = Some("nosniff".to_string());
        response.body = if self.body.is_empty() {
            Vec::new()
        } else {
            message.to_vec()
        };
        response.range_source = None;
        response
    }

    /// Render deterministic HTTP/1.1 response bytes for host-safe registryd tests.
    ///
    /// This does not open a socket or add implicit `net/http` behavior; it only
    /// serializes the status, modeled source headers, blank line, and modeled
    /// body bytes already present on this response.
    pub fn source_http_response_bytes(&self) -> Vec<u8> {
        let mut bytes = format!("HTTP/1.1 {} {}\r\n", self.status_code, self.reason).into_bytes();

        for (name, value) in self.source_http_headers() {
            bytes.extend_from_slice(format!("{name}: {value}\r\n").as_bytes());
        }
        bytes.extend_from_slice(b"\r\n");
        bytes.extend_from_slice(&self.body);

        bytes
    }
}

/// Format a source `Last-Modified` header value for `http.ServeContent` parity.
pub fn registryd_http_last_modified_value(modified: SystemTime) -> Option<String> {
    let duration = modified.duration_since(UNIX_EPOCH).ok()?;
    let seconds = duration.as_secs();
    if seconds == 0 {
        return None;
    }

    Some(registryd_imf_fixdate_from_unix_seconds(seconds))
}

fn registryd_source_last_modified_value(metadata: &fs::Metadata) -> Option<String> {
    let updated_at = registryd_source_updated_at(metadata).or_else(|| metadata.modified().ok())?;
    registryd_http_last_modified_value(updated_at)
}

#[cfg(unix)]
fn registryd_source_updated_at(metadata: &fs::Metadata) -> Option<SystemTime> {
    use std::os::unix::fs::MetadataExt;

    let seconds = metadata.atime();
    let nanos = metadata.atime_nsec();
    if seconds < 0 || nanos < 0 {
        return None;
    }

    UNIX_EPOCH.checked_add(Duration::new(seconds as u64, nanos as u32))
}

#[cfg(not(unix))]
fn registryd_source_updated_at(_metadata: &fs::Metadata) -> Option<SystemTime> {
    None
}

fn registryd_imf_fixdate_from_unix_seconds(seconds: u64) -> String {
    const WEEKDAYS: [&str; 7] = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"];
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let days = (seconds / 86_400) as i64;
    let seconds_of_day = seconds % 86_400;
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    let weekday = WEEKDAYS[days.rem_euclid(7) as usize];
    let (year, month, day) = registryd_civil_date_from_unix_days(days);

    format!(
        "{weekday}, {day:02} {} {year:04} {hour:02}:{minute:02}:{second:02} GMT",
        MONTHS[(month - 1) as usize]
    )
}

fn registryd_civil_date_from_unix_days(days: i64) -> (i64, u32, u32) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }

    (year, month as u32, day as u32)
}

fn registryd_http_time_unix_seconds(value: &str) -> Option<u64> {
    registryd_parse_imf_fixdate(value)
        .or_else(|| registryd_parse_rfc850_time(value))
        .or_else(|| registryd_parse_ansic_time(value))
}

fn registryd_http_scan_etag(value: &str) -> Option<(&str, &str)> {
    let value = value.trim_start();
    let start = if value.starts_with("W/") { 2 } else { 0 };
    let bytes = value.as_bytes();
    if bytes.len() < start + 2 || bytes[start] != b'"' {
        return None;
    }

    for index in start + 1..bytes.len() {
        let byte = bytes[index];
        if byte == b'"' {
            return Some((&value[..index + 1], &value[index + 1..]));
        }
        let allowed = byte == 0x21 || (0x23..=0x7e).contains(&byte) || byte >= 0x80;
        if !allowed {
            return None;
        }
    }

    None
}

fn registryd_parse_imf_fixdate(value: &str) -> Option<u64> {
    let value = value.trim();
    let (_, rest) = value.split_once(", ")?;
    let mut parts = rest.split_whitespace();
    let day = parts.next()?.parse::<u32>().ok()?;
    let month = registryd_month_number(parts.next()?)?;
    let year = parts.next()?.parse::<i64>().ok()?;
    let (hour, minute, second) = registryd_parse_http_time_of_day(parts.next()?)?;
    if parts.next()? != "GMT" || parts.next().is_some() {
        return None;
    }

    registryd_unix_seconds_from_parts(year, month, day, hour, minute, second)
}

fn registryd_parse_rfc850_time(value: &str) -> Option<u64> {
    let value = value.trim();
    let (_, rest) = value.split_once(", ")?;
    let mut parts = rest.split_whitespace();
    let mut date = parts.next()?.split('-');
    let day = date.next()?.parse::<u32>().ok()?;
    let month = registryd_month_number(date.next()?)?;
    let year = registryd_two_digit_http_year(date.next()?.parse::<u32>().ok()?);
    if date.next().is_some() {
        return None;
    }
    let (hour, minute, second) = registryd_parse_http_time_of_day(parts.next()?)?;
    if parts.next()? != "GMT" || parts.next().is_some() {
        return None;
    }

    registryd_unix_seconds_from_parts(year, month, day, hour, minute, second)
}

fn registryd_parse_ansic_time(value: &str) -> Option<u64> {
    let mut parts = value.split_whitespace();
    let _weekday = parts.next()?;
    let month = registryd_month_number(parts.next()?)?;
    let day = parts.next()?.parse::<u32>().ok()?;
    let (hour, minute, second) = registryd_parse_http_time_of_day(parts.next()?)?;
    let year = parts.next()?.parse::<i64>().ok()?;
    if parts.next().is_some() {
        return None;
    }

    registryd_unix_seconds_from_parts(year, month, day, hour, minute, second)
}

fn registryd_parse_http_time_of_day(value: &str) -> Option<(u32, u32, u32)> {
    let mut fields = value.split(':');
    let hour = fields.next()?.parse::<u32>().ok()?;
    let minute = fields.next()?.parse::<u32>().ok()?;
    let second = fields.next()?.parse::<u32>().ok()?;
    if fields.next().is_some() {
        return None;
    }
    Some((hour, minute, second))
}

fn registryd_unix_seconds_from_parts(
    year: i64,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> Option<u64> {
    if !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return None;
    }

    let days = registryd_unix_days_from_civil_date(year, month, day);
    if registryd_civil_date_from_unix_days(days) != (year, month, day) || days < 0 {
        return None;
    }

    let seconds = (days as u64)
        .checked_mul(86_400)?
        .checked_add((hour as u64).checked_mul(3_600)?)?
        .checked_add((minute as u64).checked_mul(60)?)?
        .checked_add(second as u64)?;
    Some(seconds)
}

fn registryd_unix_days_from_civil_date(year: i64, month: u32, day: u32) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_prime = month as i64 + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day as i64 - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

    era * 146_097 + day_of_era - 719_468
}

fn registryd_month_number(month: &str) -> Option<u32> {
    match month {
        "Jan" => Some(1),
        "Feb" => Some(2),
        "Mar" => Some(3),
        "Apr" => Some(4),
        "May" => Some(5),
        "Jun" => Some(6),
        "Jul" => Some(7),
        "Aug" => Some(8),
        "Sep" => Some(9),
        "Oct" => Some(10),
        "Nov" => Some(11),
        "Dec" => Some(12),
        _ => None,
    }
}

fn registryd_two_digit_http_year(year: u32) -> i64 {
    if year >= 69 {
        1900 + year as i64
    } else {
        2000 + year as i64
    }
}

fn registryd_blob_content_type(bytes: &[u8]) -> &'static str {
    let data = &bytes[..bytes.len().min(512)];
    let first_non_ws = data
        .iter()
        .position(|byte| !matches!(byte, b'\t' | b'\n' | 0x0c | b'\r' | b' '))
        .unwrap_or(data.len());

    if registryd_html_signature(data, first_non_ws) {
        return "text/html; charset=utf-8";
    }
    if registryd_masked_signature(&data[first_non_ws..], b"\xff\xff\xff\xff\xff", b"<?xml") {
        return "text/xml; charset=utf-8";
    }

    for (signature, content_type) in [
        (b"%PDF-" as &[u8], "application/pdf"),
        (b"%!PS-Adobe-", "application/postscript"),
        (b"\x89PNG\r\n\x1a\n", "image/png"),
        (b"\xff\xd8\xff", "image/jpeg"),
        (b"GIF87a", "image/gif"),
        (b"GIF89a", "image/gif"),
        (b"BM", "image/bmp"),
        (b"\x00\x00\x01\x00", "image/x-icon"),
        (b"\x00\x00\x02\x00", "image/x-icon"),
        (b"ID3", "audio/mpeg"),
        (b"OggS\x00", "application/ogg"),
        (b"\x1a\x45\xdf\xa3", "video/webm"),
    ] {
        if data.starts_with(signature) {
            return content_type;
        }
    }

    if data.len() >= 4 && registryd_masked_signature(data, b"\xff\xff\x00\x00", b"\xfe\xff\x00\x00")
    {
        return "text/plain; charset=utf-16be";
    }
    if data.len() >= 4 && registryd_masked_signature(data, b"\xff\xff\x00\x00", b"\xff\xfe\x00\x00")
    {
        return "text/plain; charset=utf-16le";
    }
    if data.len() >= 4 && registryd_masked_signature(data, b"\xff\xff\xff\x00", b"\xef\xbb\xbf\x00")
    {
        return "text/plain; charset=utf-8";
    }
    if data.len() >= 12
        && registryd_masked_signature(
            data,
            b"\xff\xff\xff\xff\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff",
            b"RIFF\x00\x00\x00\x00WEBPVP",
        )
    {
        return "image/webp";
    }
    if data.len() >= 12
        && registryd_masked_signature(
            data,
            b"\xff\xff\xff\xff\x00\x00\x00\x00\xff\xff\xff\xff",
            b"FORM\x00\x00\x00\x00AIFF",
        )
    {
        return "audio/aiff";
    }
    if data.len() >= 12
        && registryd_masked_signature(
            data,
            b"\xff\xff\xff\xff\x00\x00\x00\x00\xff\xff\xff\xff",
            b"RIFF\x00\x00\x00\x00AVI ",
        )
    {
        return "video/avi";
    }
    if data.len() >= 12
        && registryd_masked_signature(
            data,
            b"\xff\xff\xff\xff\x00\x00\x00\x00\xff\xff\xff\xff",
            b"RIFF\x00\x00\x00\x00WAVE",
        )
    {
        return "audio/wave";
    }
    if data.len() >= 8 && data.starts_with(b"MThd\x00\x00\x00\x06") {
        return "audio/midi";
    }
    if registryd_mp4_signature(data) {
        return "video/mp4";
    }
    if registryd_eot_font_signature(data) {
        return "application/vnd.ms-fontobject";
    }

    for (signature, content_type) in [
        (b"\x00\x01\x00\x00" as &[u8], "font/ttf"),
        (b"OTTO", "font/otf"),
        (b"ttcf", "font/collection"),
        (b"wOFF", "font/woff"),
        (b"wOF2", "font/woff2"),
        (b"\x1f\x8b\x08", "application/x-gzip"),
        (b"PK\x03\x04", "application/zip"),
        (b"Rar!\x1a\x07\x00", "application/x-rar-compressed"),
        (b"Rar!\x1a\x07\x01\x00", "application/x-rar-compressed"),
        (b"\x00asm", "application/wasm"),
    ] {
        if data.starts_with(signature) {
            return content_type;
        }
    }

    if data[first_non_ws..].iter().all(|byte| {
        !matches!(
            *byte,
            0x00..=0x08 | 0x0b | 0x0e..=0x1a | 0x1c..=0x1f
        )
    }) {
        "text/plain; charset=utf-8"
    } else {
        "application/octet-stream"
    }
}

fn registryd_eot_font_signature(data: &[u8]) -> bool {
    data.len() >= 36 && data[34] == b'L' && data[35] == b'P'
}

fn registryd_html_signature(data: &[u8], first_non_ws: usize) -> bool {
    const HTML_SIGNATURES: [&[u8]; 17] = [
        b"<!DOCTYPE HTML",
        b"<HTML",
        b"<HEAD",
        b"<SCRIPT",
        b"<IFRAME",
        b"<H1",
        b"<DIV",
        b"<FONT",
        b"<TABLE",
        b"<A",
        b"<STYLE",
        b"<TITLE",
        b"<B",
        b"<BODY",
        b"<BR",
        b"<P",
        b"<!--",
    ];
    let data = &data[first_non_ws..];
    HTML_SIGNATURES.iter().any(|signature| {
        data.len() > signature.len()
            && data[..signature.len()]
                .iter()
                .zip(signature.iter())
                .all(|(actual, expected)| (*actual).to_ascii_uppercase() == *expected)
            && matches!(data[signature.len()], b' ' | b'>')
    })
}

fn registryd_masked_signature(data: &[u8], mask: &[u8], pattern: &[u8]) -> bool {
    pattern.len() == mask.len()
        && data.len() >= pattern.len()
        && pattern
            .iter()
            .zip(mask.iter())
            .zip(data.iter())
            .all(|((pattern, mask), actual)| (*actual & *mask) == *pattern)
}

fn registryd_mp4_signature(data: &[u8]) -> bool {
    if data.len() < 12 {
        return false;
    }
    let box_size = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if data.len() < box_size || !box_size.is_multiple_of(4) || &data[4..8] != b"ftyp" {
        return false;
    }
    (8..box_size).step_by(4).any(|idx| {
        idx != 12 && idx + 3 <= data.len() && idx + 3 <= box_size && &data[idx..idx + 3] == b"mp4"
    })
}

/// Host-safe model of Talos registryd's `/healthz` endpoint.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RegistrydHealthService;

impl RegistrydHealthService {
    /// Return the singleton source-shaped health service model.
    pub fn source() -> Self {
        Self
    }

    /// Return Talos' registryd loopback listen address.
    pub fn listen_address(self) -> &'static str {
        REGISTRYD_LISTEN_ADDRESS
    }

    /// Serve only the source health route.
    ///
    /// Non-health requests return `None` so this type cannot be mistaken for a
    /// complete registryd HTTP implementation.
    pub fn handle_request(self, method: &str, path: &str) -> Option<RegistrydHttpResponse> {
        if method == "GET" && path == REGISTRYD_HEALTH_PATH {
            Some(RegistrydHttpResponse {
                status_code: 200,
                reason: "OK",
            })
        } else {
            None
        }
    }
}

/// Host-safe model of machined's registryd health probe.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RegistrydHealthProbe;

impl RegistrydHealthProbe {
    /// Return the singleton source-shaped health probe model.
    pub fn source() -> Self {
        Self
    }

    /// Return the exact URL machined probes for registryd health.
    pub fn url(self) -> &'static str {
        REGISTRYD_HEALTH_URL
    }

    /// Return the request line used by the health probe.
    pub fn request_line(self) -> String {
        format!("GET {REGISTRYD_HEALTH_PATH} HTTP/1.1")
    }

    /// Match the 2xx success range accepted by a simple HTTP health check.
    pub fn accepts_status(self, status_code: u16) -> bool {
        (200..300).contains(&status_code)
    }
}

/// Source registry API root.
///
/// Talos registryd registers this root with `GET /v2` and `GET /v2/{$}`;
/// Go's HTTP mux also routes `HEAD` to `GET` handlers.
pub const REGISTRYD_API_ROOT: &str = "/v2";

/// Source registry content path prefix.
pub const REGISTRYD_API_PREFIX: &str = "/v2/";

/// Source-supported registryd content families.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistrydApiContentKind {
    /// `GET|HEAD /v2/{name}/manifests/{tag-or-digest}`.
    Manifest,
    /// `GET|HEAD /v2/{name}/blobs/{digest}`.
    Blob,
}

/// Parsed registryd `/v2/{args...}` request parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrydApiRequest {
    /// Source `ns` query parameter. Missing namespace is later rejected unless
    /// a filesystem lookup can infer it; this host-safe contract model has no
    /// content root, so missing namespaces remain `400 Bad Request`.
    pub registry: Option<String>,
    /// Repository name from the path before `manifests`/`blobs`.
    pub name: String,
    /// Manifest tag/digest or blob digest from the final path segment.
    pub reference: String,
    /// Requested content family.
    pub kind: RegistrydApiContentKind,
}

impl RegistrydApiRequest {
    /// Return the source `params.String()` image reference spelling.
    pub fn source_reference(&self) -> String {
        let mut reference = String::new();
        if let Some(registry) = self
            .registry
            .as_deref()
            .filter(|registry| !registry.is_empty())
        {
            reference.push_str(registry);
            reference.push('/');
        }
        reference.push_str(&self.name);
        if self.reference.starts_with("sha256:") {
            reference.push('@');
        } else {
            reference.push(':');
        }
        reference.push_str(&self.reference);
        reference
    }

    /// Plan the source registryd cache path(s) touched by this request.
    ///
    /// Source `singleFileStore.blobPath` stores digests by replacing the
    /// `sha256:` prefix with `sha256-`. Digest-pinned blob requests read from
    /// `blob/`; tag-shaped blob requests first follow source
    /// `resolveCanonicalRef` through `manifests/<name>/reference/<tag>` and the
    /// matching manifest digest file before reading `blob/<canonical-digest>`.
    /// Manifest digest requests read from `manifests/<name>/digest/`; tagged
    /// manifest requests follow the same reference/digest verification path.
    pub fn cache_path_plan(&self) -> Result<RegistrydCachePathPlan, String> {
        let store_name = self
            .source_store_name()
            .ok_or_else(|| "registry namespace required".to_string())?;

        match self.kind {
            RegistrydApiContentKind::Blob if self.reference.starts_with("sha256:") => {
                Ok(RegistrydCachePathPlan::Blob {
                    content_path: PathBuf::from(REGISTRYD_BLOB_STORE_DIR)
                        .join(registryd_source_digest_file_name(&self.reference)?),
                })
            }
            RegistrydApiContentKind::Blob => {
                if !registryd_source_manifest_tag_is_valid(&self.reference) {
                    return Err("blob tag reference is invalid".to_string());
                }
                let manifest_dir = PathBuf::from(REGISTRYD_MANIFEST_STORE_DIR).join(store_name);
                Ok(RegistrydCachePathPlan::BlobTag {
                    reference_path: manifest_dir
                        .join(REGISTRYD_REFERENCE_STORE_DIR)
                        .join(&self.reference),
                    digest_dir: manifest_dir.join(REGISTRYD_DIGEST_STORE_DIR),
                })
            }
            RegistrydApiContentKind::Manifest if self.reference.starts_with("sha256:") => {
                Ok(RegistrydCachePathPlan::ManifestDigest {
                    content_path: PathBuf::from(REGISTRYD_MANIFEST_STORE_DIR)
                        .join(store_name)
                        .join(REGISTRYD_DIGEST_STORE_DIR)
                        .join(registryd_source_digest_file_name(&self.reference)?),
                })
            }
            RegistrydApiContentKind::Manifest => {
                if !registryd_source_manifest_tag_is_valid(&self.reference) {
                    return Err("manifest tag reference is invalid".to_string());
                }
                let manifest_dir = PathBuf::from(REGISTRYD_MANIFEST_STORE_DIR).join(store_name);
                Ok(RegistrydCachePathPlan::ManifestTag {
                    reference_path: manifest_dir
                        .join(REGISTRYD_REFERENCE_STORE_DIR)
                        .join(&self.reference),
                    digest_dir: manifest_dir.join(REGISTRYD_DIGEST_STORE_DIR),
                })
            }
        }
    }

    fn source_store_name(&self) -> Option<String> {
        let registry = self
            .registry
            .as_deref()
            .filter(|registry| !registry.is_empty())?;
        registryd_source_store_name_from_query(registry, &self.name)
    }
}

/// Source single-file content store directory for blobs.
pub const REGISTRYD_BLOB_STORE_DIR: &str = "blob";

/// Source manifest store directory.
pub const REGISTRYD_MANIFEST_STORE_DIR: &str = "manifests";

/// Source tagged-manifest reference directory.
pub const REGISTRYD_REFERENCE_STORE_DIR: &str = "reference";

/// Source digest-addressed manifest directory.
pub const REGISTRYD_DIGEST_STORE_DIR: &str = "digest";

/// OCI image index media type used by source `getManifestData` inference.
pub const REGISTRYD_OCI_IMAGE_INDEX_MEDIA_TYPE: &str = "application/vnd.oci.image.index.v1+json";

/// OCI image manifest media type used by source `getManifestData` inference.
pub const REGISTRYD_OCI_IMAGE_MANIFEST_MEDIA_TYPE: &str =
    "application/vnd.oci.image.manifest.v1+json";

/// Source registryd cache path plan for a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistrydCachePathPlan {
    /// Blob content is read directly by digest from `blob/sha256-*`.
    Blob {
        /// Relative path below each configured image-cache root.
        content_path: PathBuf,
    },
    /// Tag-shaped blob requests first resolve the manifest tag to a canonical
    /// digest, then read that digest from `blob/sha256-*`.
    BlobTag {
        /// `manifests/<name>/reference/<tag>`.
        reference_path: PathBuf,
        /// `manifests/<name>/digest`.
        digest_dir: PathBuf,
    },
    /// Digest-pinned manifests are read directly from `manifests/.../digest`.
    ManifestDigest {
        /// Relative path below each configured image-cache root.
        content_path: PathBuf,
    },
    /// Tagged manifests are resolved from a reference file, then verified
    /// against the digest-addressed manifest directory.
    ManifestTag {
        /// `manifests/<name>/reference/<tag>`.
        reference_path: PathBuf,
        /// `manifests/<name>/digest`.
        digest_dir: PathBuf,
    },
}

impl RegistrydCachePathPlan {
    /// Return the first source filesystem lookup path for this request.
    ///
    /// Digest-pinned blob and digest-addressed manifest requests read content
    /// directly. Tag-shaped blob and manifest requests first read the tag
    /// reference file before later code can hash it and verify the matching
    /// digest-addressed manifest.
    pub fn initial_lookup_path(&self) -> &Path {
        match self {
            RegistrydCachePathPlan::Blob { content_path }
            | RegistrydCachePathPlan::ManifestDigest { content_path } => content_path.as_path(),
            RegistrydCachePathPlan::BlobTag { reference_path, .. }
            | RegistrydCachePathPlan::ManifestTag { reference_path, .. } => {
                reference_path.as_path()
            }
        }
    }

    /// Resolve a tagged manifest to the source canonical digest path.
    ///
    /// Source `resolveCanonicalRef` hashes the tag reference file, derives a
    /// `sha256-...` digest filename, hashes that digest file, and requires the
    /// two hashes to match before returning a canonical `sha256:...` reference.
    pub fn resolve_tagged_manifest_digest(
        &self,
        reference_bytes: &[u8],
        digest_bytes: &[u8],
    ) -> Result<RegistrydManifestTagDigestResolution, String> {
        let resolution = self.tagged_manifest_digest_resolution_from_reference(reference_bytes)?;
        let digest_file_digest_bytes = registryd_sha256(digest_bytes);
        let reference_digest_bytes = registryd_sha256(reference_bytes);

        if reference_digest_bytes != digest_file_digest_bytes {
            return Err("tagged manifest hash does not match digest file hash".to_string());
        }

        Ok(resolution)
    }

    fn tagged_manifest_digest_resolution_from_reference(
        &self,
        reference_bytes: &[u8],
    ) -> Result<RegistrydManifestTagDigestResolution, String> {
        let digest_dir = match self {
            RegistrydCachePathPlan::BlobTag { digest_dir, .. }
            | RegistrydCachePathPlan::ManifestTag { digest_dir, .. } => digest_dir,
            _ => return Err("tagged manifest plan required".to_string()),
        };

        let canonical_digest = registryd_sha256_digest_string(registryd_sha256(reference_bytes));
        let digest_path = digest_dir.join(registryd_source_digest_file_name(&canonical_digest)?);

        Ok(RegistrydManifestTagDigestResolution {
            canonical_digest,
            digest_path,
        })
    }
}

/// Result of source tagged-manifest canonical digest verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrydManifestTagDigestResolution {
    /// Canonical source digest spelling, `sha256:<64 hex>`.
    pub canonical_digest: String,
    /// Digest-addressed manifest path derived from the reference bytes.
    pub digest_path: PathBuf,
}

/// Source-shaped model of registryd's `MultiPathFS`.
///
/// The Go source iterates cache roots in order, converts each root with
/// `filepath.Abs`, joins the relative cache path, and stops at the first
/// successful `Open`/`Stat`. This host-safe model exposes that lookup decision
/// without opening files, so later serving code can consume a verified path
/// plan without inventing filesystem side effects in tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrydMultiPathFs {
    roots: Vec<PathBuf>,
}

impl RegistrydMultiPathFs {
    /// Build a source-ordered multi-root filesystem model.
    pub fn new<I, P>(roots: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        Self {
            roots: roots.into_iter().map(Into::into).collect(),
        }
    }

    /// Borrow the configured roots in source iteration order.
    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    /// Resolve a relative cache path against each root until `exists` accepts one.
    ///
    /// The returned attempts are absolute-but-not-canonicalized paths, matching
    /// `filepath.Abs(root)` followed by `filepath.Join(abs, name)`.
    pub fn resolve_with<F>(
        &self,
        name: impl AsRef<Path>,
        mut exists: F,
    ) -> io::Result<RegistrydMultiPathFsResolution>
    where
        F: FnMut(&Path) -> bool,
    {
        let name = name.as_ref();
        let mut attempts = Vec::new();

        for root in &self.roots {
            let candidate = registryd_absolutize_root(root)?.join(name);
            attempts.push(candidate.clone());
            if exists(&candidate) {
                return Ok(RegistrydMultiPathFsResolution::Found {
                    path: candidate,
                    attempts,
                });
            }
        }

        Ok(RegistrydMultiPathFsResolution::Missing { attempts })
    }

    /// Read a cache file using source `MultiPathFS.Open` root order.
    ///
    /// Each root is attempted in order. The first successful read returns its
    /// bytes and the paths attempted up to that hit; source-open misses are
    /// tracked for all-roots-missed behavior, while existing-path read errors
    /// are returned as the source serving path would observe them.
    pub fn read_file(&self, name: impl AsRef<Path>) -> io::Result<RegistrydMultiPathFsRead> {
        let name = name.as_ref();
        let mut missing_attempts = Vec::new();
        let mut first_source_error = None;

        for root in &self.roots {
            let candidate = registryd_absolutize_root(root)?.join(name);
            let metadata = match fs::metadata(&candidate) {
                Ok(metadata) => metadata,
                Err(err) => {
                    let error_kind = err.kind();
                    if error_kind != io::ErrorKind::NotFound && first_source_error.is_none() {
                        first_source_error = Some(io::Error::new(error_kind, err.to_string()));
                    }
                    missing_attempts.push(RegistrydMultiPathFsReadAttempt {
                        path: candidate,
                        error_kind,
                    });
                    continue;
                }
            };
            match fs::read(&candidate) {
                Ok(bytes) => {
                    let modified = registryd_source_last_modified_value(&metadata);
                    let mut attempts = missing_attempts
                        .iter()
                        .map(|attempt: &RegistrydMultiPathFsReadAttempt| attempt.path.clone())
                        .collect::<Vec<_>>();
                    attempts.push(candidate.clone());
                    return Ok(RegistrydMultiPathFsRead::Found {
                        path: candidate,
                        bytes,
                        modified,
                        attempts,
                    });
                }
                Err(err) if err.kind() == io::ErrorKind::NotFound => {
                    missing_attempts.push(RegistrydMultiPathFsReadAttempt {
                        path: candidate,
                        error_kind: err.kind(),
                    });
                }
                Err(err) => return Err(err),
            }
        }

        if let Some(err) = first_source_error {
            Err(err)
        } else {
            Ok(RegistrydMultiPathFsRead::Missing {
                attempts: missing_attempts,
            })
        }
    }

    /// Infer a missing source `ns` query value from cached manifest roots.
    ///
    /// Source `tryFindRegistry` reads the first available `manifests` directory,
    /// in `fs.ReadDir` filename order, converts port-encoded entries back to
    /// registry names, parses `p.String()` through `ParseDockerRef`, and
    /// accepts the first normalized `<name>/reference` directory that exists.
    pub fn infer_registry_for_request(
        &self,
        request: &RegistrydApiRequest,
    ) -> io::Result<Option<String>> {
        for entry in self.manifest_entry_names()? {
            let registry = registryd_create_registry_with_port(&entry);
            let store_name = registryd_source_store_name_from_query(&registry, &request.name)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "source registry candidate failed to parse as docker ref",
                    )
                })?;
            let reference_dir = PathBuf::from(REGISTRYD_MANIFEST_STORE_DIR)
                .join(store_name)
                .join(REGISTRYD_REFERENCE_STORE_DIR);

            if matches!(
                self.resolve_with(reference_dir, |path| path.exists())?,
                RegistrydMultiPathFsResolution::Found { .. }
            ) {
                return Ok(Some(registry));
            }
        }

        Ok(None)
    }

    fn manifest_entry_names(&self) -> io::Result<Vec<String>> {
        let mut last_error = None;

        for root in &self.roots {
            let candidate = registryd_absolutize_root(root)?.join(REGISTRYD_MANIFEST_STORE_DIR);
            match fs::read_dir(candidate) {
                Ok(entries) => {
                    let mut names = Vec::new();
                    for entry in entries {
                        names.push(entry?.file_name().to_string_lossy().into_owned());
                    }
                    names.sort();
                    return Ok(names);
                }
                Err(err) => last_error = Some(err),
            }
        }

        Err(last_error.unwrap_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "registryd manifests directory missing",
            )
        }))
    }
}

/// Result of a source-ordered registryd cache-root lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistrydMultiPathFsResolution {
    /// The first existing path and the attempted paths up to that hit.
    Found {
        /// Absolute-but-not-canonicalized file path that should be used.
        path: PathBuf,
        /// Paths tried in source order, including the found path.
        attempts: Vec<PathBuf>,
    },
    /// No root contained the path; empty roots produce empty attempts.
    Missing {
        /// Paths tried in source order.
        attempts: Vec<PathBuf>,
    },
}

/// Result of source-ordered registryd cache byte lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistrydMultiPathFsRead {
    /// The first readable file below the configured cache roots.
    Found {
        /// Absolute-but-not-canonicalized file path that was read.
        path: PathBuf,
        /// File bytes read from the first successful root.
        bytes: Vec<u8>,
        /// Source-shaped updated time for `http.ServeContent`.
        modified: Option<String>,
        /// Paths tried in source order, including the found path.
        attempts: Vec<PathBuf>,
    },
    /// No configured root produced readable bytes.
    Missing {
        /// Per-root miss diagnostics in source order.
        attempts: Vec<RegistrydMultiPathFsReadAttempt>,
    },
}

/// One failed `MultiPathFS.Open`-style read attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrydMultiPathFsReadAttempt {
    /// Absolute-but-not-canonicalized path that was attempted.
    pub path: PathBuf,
    /// Stable `std::io` error kind from the read attempt.
    pub error_kind: io::ErrorKind,
}

/// Decoded source fields used by registryd manifest media-type inference.
///
/// Source `getManifestData` unmarshals a manifest blob and reads only the
/// top-level `mediaType`, `manifests`, `layers`, and `config` fields for media
/// type selection. This shape keeps that rule dependency-free while
/// `from_json_bytes` mirrors the small source field surface instead of pulling
/// in a general JSON dependency for host-safe tests.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistrydManifestShape {
    media_type: Option<String>,
    has_manifests: bool,
    has_layers: bool,
    has_config: bool,
}

impl RegistrydManifestShape {
    /// Build an empty decoded manifest shape.
    pub fn new() -> Self {
        Self::default()
    }

    /// Decode the source-observed top-level manifest fields from JSON bytes.
    ///
    /// The Go source uses `json.Unmarshal` into a struct whose RawMessage fields
    /// are considered present when the top-level field exists, even when that
    /// value is `null`. This parser intentionally recognizes only the fields
    /// registryd needs for media-type inference and skips all nested values so
    /// nested `mediaType`/`layers` names cannot accidentally affect the result.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, String> {
        let mut cursor = RegistrydJsonCursor::new(bytes);
        let mut shape = Self::new();

        cursor.skip_ws();
        cursor.expect_byte(b'{')?;
        cursor.skip_ws();

        if cursor.consume_byte(b'}') {
            cursor.finish()?;
            return Ok(shape);
        }

        loop {
            cursor.skip_ws();
            let key = cursor.parse_string()?;
            cursor.skip_ws();
            cursor.expect_byte(b':')?;
            cursor.skip_ws();

            match key.as_str() {
                "mediaType" => {
                    if cursor.peek_byte() == Some(b'"') {
                        shape.media_type = Some(cursor.parse_string()?);
                    } else if cursor.consume_literal("null") {
                        shape.media_type = None;
                    } else {
                        return Err("mediaType must be a JSON string or null".to_string());
                    }
                }
                "manifests" => {
                    shape.has_manifests = true;
                    cursor.skip_value()?;
                }
                "layers" => {
                    shape.has_layers = true;
                    cursor.skip_value()?;
                }
                "config" => {
                    shape.has_config = true;
                    cursor.skip_value()?;
                }
                _ => cursor.skip_value()?,
            }

            cursor.skip_ws();
            if cursor.consume_byte(b',') {
                cursor.skip_ws();
                if cursor.peek_byte() == Some(b'}') {
                    return Err("trailing comma in JSON object".to_string());
                }
                continue;
            }

            cursor.expect_byte(b'}')?;
            cursor.finish()?;
            return Ok(shape);
        }
    }

    /// Set an explicit top-level `mediaType` field.
    pub fn with_media_type(mut self, media_type: impl Into<String>) -> Self {
        self.media_type = Some(media_type.into());
        self
    }

    /// Mark a top-level `manifests` field as present.
    pub fn with_manifests(mut self) -> Self {
        self.has_manifests = true;
        self
    }

    /// Mark a top-level `layers` field as present.
    pub fn with_layers(mut self) -> Self {
        self.has_layers = true;
        self
    }

    /// Mark a top-level `config` field as present.
    pub fn with_config(mut self) -> Self {
        self.has_config = true;
        self
    }
}

/// Infer registryd manifest media type using source `getManifestData` rules.
pub fn registryd_manifest_media_type(shape: &RegistrydManifestShape) -> Result<String, String> {
    if let Some(media_type) = shape
        .media_type
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        return Ok(media_type.to_string());
    }

    if shape.has_manifests {
        return Ok(REGISTRYD_OCI_IMAGE_INDEX_MEDIA_TYPE.to_string());
    }

    if shape.has_layers || shape.has_config {
        return Ok(REGISTRYD_OCI_IMAGE_MANIFEST_MEDIA_TYPE.to_string());
    }

    Err("media type is empty and cannot be inferred".to_string())
}

#[derive(Debug, Clone, Copy)]
struct RegistrydJsonCursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> RegistrydJsonCursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn finish(&mut self) -> Result<(), String> {
        self.skip_ws();
        if self.pos == self.bytes.len() {
            Ok(())
        } else {
            Err("trailing data after JSON value".to_string())
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn consume_byte(&mut self, byte: u8) -> bool {
        if self.peek_byte() == Some(byte) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect_byte(&mut self, byte: u8) -> Result<(), String> {
        if self.consume_byte(byte) {
            Ok(())
        } else {
            Err(format!(
                "expected JSON byte '{}' at offset {}",
                char::from(byte),
                self.pos
            ))
        }
    }

    fn consume_literal(&mut self, literal: &str) -> bool {
        let literal = literal.as_bytes();
        if self.bytes[self.pos..].starts_with(literal) {
            self.pos += literal.len();
            true
        } else {
            false
        }
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek_byte(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.pos += 1;
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect_byte(b'"')?;
        let mut out = String::new();

        loop {
            let byte = self
                .peek_byte()
                .ok_or_else(|| "unterminated JSON string".to_string())?;
            self.pos += 1;

            match byte {
                b'"' => return Ok(out),
                b'\\' => out.push(self.parse_escape()?),
                0x00..=0x1f => {
                    return Err("unescaped control character in JSON string".to_string());
                }
                _ => {
                    let start = self.pos - 1;
                    while let Some(next) = self.peek_byte() {
                        if next == b'"' || next == b'\\' || next <= 0x1f {
                            break;
                        }
                        self.pos += 1;
                    }
                    let segment = std::str::from_utf8(&self.bytes[start..self.pos])
                        .map_err(|_| "invalid UTF-8 in JSON string".to_string())?;
                    out.push_str(segment);
                }
            }
        }
    }

    fn parse_escape(&mut self) -> Result<char, String> {
        let byte = self
            .peek_byte()
            .ok_or_else(|| "unterminated JSON escape".to_string())?;
        self.pos += 1;

        match byte {
            b'"' => Ok('"'),
            b'\\' => Ok('\\'),
            b'/' => Ok('/'),
            b'b' => Ok('\u{0008}'),
            b'f' => Ok('\u{000c}'),
            b'n' => Ok('\n'),
            b'r' => Ok('\r'),
            b't' => Ok('\t'),
            b'u' => self.parse_unicode_escape(),
            _ => Err("invalid JSON string escape".to_string()),
        }
    }

    fn parse_unicode_escape(&mut self) -> Result<char, String> {
        let first = self.parse_hex4()?;
        let codepoint = if (0xd800..=0xdbff).contains(&first) {
            if !self.consume_byte(b'\\') || !self.consume_byte(b'u') {
                return Err("missing low surrogate in JSON unicode escape".to_string());
            }

            let second = self.parse_hex4()?;
            if !(0xdc00..=0xdfff).contains(&second) {
                return Err("invalid low surrogate in JSON unicode escape".to_string());
            }

            0x10000 + (((first - 0xd800) << 10) | (second - 0xdc00))
        } else if (0xdc00..=0xdfff).contains(&first) {
            return Err("unexpected low surrogate in JSON unicode escape".to_string());
        } else {
            first
        };

        char::from_u32(codepoint).ok_or_else(|| "invalid JSON unicode scalar".to_string())
    }

    fn parse_hex4(&mut self) -> Result<u32, String> {
        let mut value = 0u32;
        for _ in 0..4 {
            let byte = self
                .peek_byte()
                .ok_or_else(|| "short JSON unicode escape".to_string())?;
            self.pos += 1;
            let digit = match byte {
                b'0'..=b'9' => u32::from(byte - b'0'),
                b'a'..=b'f' => u32::from(byte - b'a' + 10),
                b'A'..=b'F' => u32::from(byte - b'A' + 10),
                _ => return Err("invalid hex digit in JSON unicode escape".to_string()),
            };
            value = (value << 4) | digit;
        }
        Ok(value)
    }

    fn skip_value(&mut self) -> Result<(), String> {
        self.skip_ws();
        match self.peek_byte() {
            Some(b'"') => self.parse_string().map(|_| ()),
            Some(b'{') => self.skip_object(),
            Some(b'[') => self.skip_array(),
            Some(b't') => self.expect_literal("true"),
            Some(b'f') => self.expect_literal("false"),
            Some(b'n') => self.expect_literal("null"),
            Some(b'-' | b'0'..=b'9') => self.skip_number(),
            Some(_) => Err(format!("unexpected JSON value at offset {}", self.pos)),
            None => Err("expected JSON value, found end of input".to_string()),
        }
    }

    fn expect_literal(&mut self, literal: &str) -> Result<(), String> {
        if self.consume_literal(literal) {
            Ok(())
        } else {
            Err(format!(
                "expected JSON literal {literal} at offset {}",
                self.pos
            ))
        }
    }

    fn skip_object(&mut self) -> Result<(), String> {
        self.expect_byte(b'{')?;
        self.skip_ws();
        if self.consume_byte(b'}') {
            return Ok(());
        }

        loop {
            self.skip_ws();
            self.parse_string()?;
            self.skip_ws();
            self.expect_byte(b':')?;
            self.skip_value()?;
            self.skip_ws();

            if self.consume_byte(b',') {
                self.skip_ws();
                if self.peek_byte() == Some(b'}') {
                    return Err("trailing comma in JSON object".to_string());
                }
                continue;
            }

            self.expect_byte(b'}')?;
            return Ok(());
        }
    }

    fn skip_array(&mut self) -> Result<(), String> {
        self.expect_byte(b'[')?;
        self.skip_ws();
        if self.consume_byte(b']') {
            return Ok(());
        }

        loop {
            self.skip_value()?;
            self.skip_ws();

            if self.consume_byte(b',') {
                self.skip_ws();
                if self.peek_byte() == Some(b']') {
                    return Err("trailing comma in JSON array".to_string());
                }
                continue;
            }

            self.expect_byte(b']')?;
            return Ok(());
        }
    }

    fn skip_number(&mut self) -> Result<(), String> {
        self.consume_byte(b'-');

        match self.peek_byte() {
            Some(b'0') => {
                self.pos += 1;
            }
            Some(b'1'..=b'9') => {
                self.pos += 1;
                while matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                    self.pos += 1;
                }
            }
            _ => return Err("invalid JSON number".to_string()),
        }

        if self.consume_byte(b'.') {
            if !matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                return Err("invalid JSON number fraction".to_string());
            }
            while matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }

        if matches!(self.peek_byte(), Some(b'e' | b'E')) {
            self.pos += 1;
            if !self.consume_byte(b'+') {
                self.consume_byte(b'-');
            }
            if !matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                return Err("invalid JSON number exponent".to_string());
            }
            while matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }

        Ok(())
    }
}

/// Host-safe registryd HTTP contract model.
///
/// This mirrors the source mux and request-parameter/status-code contract from
/// `internal/app/machined/pkg/system/services/registry`. The status-only
/// [`RegistrydHttpService::handle_request`] path keeps the original mux probe
/// model, while [`RegistrydHttpService::handle_cached_content_request`] composes
/// the ported `MultiPathFS` manifest/blob cache serving slices.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RegistrydHttpService;

impl RegistrydHttpService {
    /// Return the singleton source-shaped registryd HTTP contract.
    pub fn source() -> Self {
        Self
    }

    /// Return Talos' registryd loopback listen address.
    pub fn listen_address(self) -> &'static str {
        REGISTRYD_LISTEN_ADDRESS
    }

    /// Classify a source registryd HTTP request target.
    ///
    /// Unknown methods/routes return `None`, matching this host-safe model's
    /// explicit surface boundary. Recognized source routes return the same
    /// success/error status class the Go service exposes before content bytes
    /// are read.
    pub fn handle_request(self, method: &str, target: &str) -> Option<RegistrydHttpResponse> {
        if !registryd_method_can_use_get_handler(method) {
            return None;
        }

        let path = registryd_target_path(target);
        if path == REGISTRYD_API_ROOT
            || path == REGISTRYD_API_PREFIX
            || path == REGISTRYD_HEALTH_PATH
            || path == format!("{REGISTRYD_HEALTH_PATH}/")
        {
            return Some(RegistrydHttpResponse {
                status_code: 200,
                reason: "OK",
            });
        }

        if !path.starts_with(REGISTRYD_API_PREFIX) {
            return None;
        }

        match extract_registryd_api_request(target) {
            Ok(request) if request.registry.as_deref().is_some_and(|ns| !ns.is_empty()) => {
                if request.cache_path_plan().is_err() {
                    return Some(RegistrydHttpResponse {
                        status_code: 400,
                        reason: "Bad Request",
                    });
                }
                Some(RegistrydHttpResponse {
                    status_code: 404,
                    reason: "Not Found",
                })
            }
            Ok(_) | Err(RegistrydRouteError::BadRequest) => Some(RegistrydHttpResponse {
                status_code: 400,
                reason: "Bad Request",
            }),
            Err(RegistrydRouteError::NotFound) => Some(RegistrydHttpResponse {
                status_code: 404,
                reason: "Not Found",
            }),
        }
    }

    /// Serve any source-shaped registryd route from configured image-cache roots.
    ///
    /// This composes the manifest and blob content slices with the same simple
    /// `/v2` and `/healthz` status routes registered by Talos' source mux. It
    /// remains host-safe: responses are modeled as data and no socket is opened.
    pub fn handle_cached_content_request(
        self,
        method: &str,
        target: &str,
        roots: &RegistrydMultiPathFs,
    ) -> Option<RegistrydContentResponse> {
        if !registryd_method_can_use_get_handler(method) {
            return None;
        }

        let path = registryd_target_path(target);
        if path == REGISTRYD_API_ROOT
            || path == REGISTRYD_API_PREFIX
            || path == REGISTRYD_HEALTH_PATH
            || path == format!("{REGISTRYD_HEALTH_PATH}/")
        {
            return Some(RegistrydContentResponse::status(200));
        }

        if !path.starts_with(REGISTRYD_API_PREFIX) {
            return None;
        }

        let mut request = match extract_registryd_api_request(target) {
            Ok(request) => request,
            Err(RegistrydRouteError::BadRequest) => {
                return Some(RegistrydContentResponse::error(400));
            }
            Err(RegistrydRouteError::NotFound) => {
                return Some(RegistrydContentResponse::error(404));
            }
        };

        if request.registry.as_deref().is_none_or(str::is_empty) {
            match roots.infer_registry_for_request(&request) {
                Ok(Some(registry)) => request.registry = Some(registry),
                Ok(None) => return Some(RegistrydContentResponse::error(400)),
                Err(err) if err.kind() == io::ErrorKind::InvalidInput => {
                    return Some(RegistrydContentResponse::error(400));
                }
                Err(_) => return Some(RegistrydContentResponse::error(500)),
            }
        }

        let plan = match request.cache_path_plan() {
            Ok(plan) => plan,
            Err(_) => return Some(RegistrydContentResponse::error(400)),
        };

        let result = match request.kind {
            RegistrydApiContentKind::Manifest => {
                registryd_manifest_content_response(method, &request, &plan, roots)
            }
            RegistrydApiContentKind::Blob => {
                registryd_blob_content_response(method, &request, &plan, roots)
            }
        };

        match result {
            Ok(response) => Some(response),
            Err(RegistrydContentError::NotFound) => Some(RegistrydContentResponse::error(404)),
            Err(RegistrydContentError::Internal) => Some(RegistrydContentResponse::error(500)),
        }
    }

    /// Serve a source-shaped manifest request from configured image-cache roots.
    ///
    /// This is a manifest-only content slice of source `handler`: it parses the
    /// same route params, requires an `ns` registry, resolves digest or tagged
    /// manifest cache paths through `MultiPathFS`, sets
    /// `Docker-Content-Digest`/`Content-Length`/manifest `Content-Type`, and
    /// suppresses the body for `HEAD`.
    pub fn handle_manifest_request(
        self,
        method: &str,
        target: &str,
        roots: &RegistrydMultiPathFs,
    ) -> Option<RegistrydContentResponse> {
        if !registryd_method_can_use_get_handler(method) {
            return None;
        }

        if !registryd_target_path(target).starts_with(REGISTRYD_API_PREFIX) {
            return None;
        }

        let request = match extract_registryd_api_request(target) {
            Ok(request) if request.kind == RegistrydApiContentKind::Manifest => request,
            Ok(_) | Err(RegistrydRouteError::NotFound) => return None,
            Err(RegistrydRouteError::BadRequest) => {
                return Some(RegistrydContentResponse::error(400));
            }
        };

        if request.registry.as_deref().is_none_or(str::is_empty) {
            return Some(RegistrydContentResponse::error(400));
        }

        let plan = match request.cache_path_plan() {
            Ok(plan) => plan,
            Err(_) => return Some(RegistrydContentResponse::error(400)),
        };

        match registryd_manifest_content_response(method, &request, &plan, roots) {
            Ok(response) => Some(response),
            Err(RegistrydContentError::NotFound) => Some(RegistrydContentResponse::error(404)),
            Err(RegistrydContentError::Internal) => Some(RegistrydContentResponse::error(500)),
        }
    }

    /// Serve a source-shaped blob request from configured image-cache roots.
    ///
    /// This mirrors the blob branch of source `handler`: digest-pinned blob
    /// routes require `ns`, read `blob/sha256-...` through `MultiPathFS`, set
    /// `Content-Length` plus `Docker-Content-Digest`, and omit the body for
    /// `HEAD`.
    pub fn handle_blob_request(
        self,
        method: &str,
        target: &str,
        roots: &RegistrydMultiPathFs,
    ) -> Option<RegistrydContentResponse> {
        if !registryd_method_can_use_get_handler(method) {
            return None;
        }

        if !registryd_target_path(target).starts_with(REGISTRYD_API_PREFIX) {
            return None;
        }

        let request = match extract_registryd_api_request(target) {
            Ok(request) if request.kind == RegistrydApiContentKind::Blob => request,
            Ok(_) | Err(RegistrydRouteError::NotFound) => return None,
            Err(RegistrydRouteError::BadRequest) => {
                return Some(RegistrydContentResponse::error(400));
            }
        };

        if request.registry.as_deref().is_none_or(str::is_empty) {
            return Some(RegistrydContentResponse::error(400));
        }

        let plan = match request.cache_path_plan() {
            Ok(plan) => plan,
            Err(_) => return Some(RegistrydContentResponse::error(400)),
        };

        match registryd_blob_content_response(method, &request, &plan, roots) {
            Ok(response) => Some(response),
            Err(RegistrydContentError::NotFound) => Some(RegistrydContentResponse::error(404)),
            Err(RegistrydContentError::Internal) => Some(RegistrydContentResponse::error(500)),
        }
    }

    /// Extract the source `params` fields from `/v2/{args...}`.
    pub fn extract_request(self, target: &str) -> Result<RegistrydApiRequest, String> {
        extract_registryd_api_request(target).map_err(|err| match err {
            RegistrydRouteError::BadRequest => "bad request".to_string(),
            RegistrydRouteError::NotFound => "not found".to_string(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegistrydRouteError {
    BadRequest,
    NotFound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegistrydContentError {
    NotFound,
    Internal,
}

fn registryd_status_reason(status_code: u16) -> &'static str {
    match status_code {
        206 => "Partial Content",
        304 => "Not Modified",
        412 => "Precondition Failed",
        416 => "Requested Range Not Satisfiable",
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RegistrydByteRange {
    start: usize,
    length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RegistrydByteRangeResponse {
    Body { start: usize, length: usize },
    Multipart(Vec<RegistrydByteRange>),
    NoOverlap,
    Invalid,
}

fn registryd_byte_range_response(
    range_header: &str,
    len: usize,
) -> Option<RegistrydByteRangeResponse> {
    if len == 0 {
        return None;
    }

    let Some(spec) = range_header.trim().strip_prefix("bytes=") else {
        return Some(RegistrydByteRangeResponse::Invalid);
    };

    let mut ranges = Vec::new();
    let mut no_overlap = false;
    for range in spec
        .split(',')
        .map(str::trim)
        .filter(|range| !range.is_empty())
    {
        let Some((start, end)) = range.split_once('-') else {
            return Some(RegistrydByteRangeResponse::Invalid);
        };
        let start = start.trim();
        let end = end.trim();

        if start.is_empty() {
            if end.is_empty() || end.starts_with('-') {
                return Some(RegistrydByteRangeResponse::Invalid);
            }
            let Ok(suffix_len) = end.parse::<usize>() else {
                return Some(RegistrydByteRangeResponse::Invalid);
            };
            let length = suffix_len.min(len);
            ranges.push(RegistrydByteRange {
                start: len - length,
                length,
            });
            continue;
        }

        let Ok(start) = start.parse::<usize>() else {
            return Some(RegistrydByteRangeResponse::Invalid);
        };
        if start >= len {
            no_overlap = true;
            continue;
        }

        let end = if end.is_empty() {
            len - 1
        } else {
            let Ok(end) = end.parse::<usize>() else {
                return Some(RegistrydByteRangeResponse::Invalid);
            };
            if start > end {
                return Some(RegistrydByteRangeResponse::Invalid);
            }
            end.min(len - 1)
        };

        ranges.push(RegistrydByteRange {
            start,
            length: end - start + 1,
        });
    }

    if no_overlap && ranges.is_empty() {
        return Some(RegistrydByteRangeResponse::NoOverlap);
    }
    if ranges.is_empty() || ranges.iter().map(|range| range.length).sum::<usize>() > len {
        return None;
    }

    if ranges.len() == 1 {
        let range = ranges[0];
        return Some(RegistrydByteRangeResponse::Body {
            start: range.start,
            length: range.length,
        });
    }

    Some(RegistrydByteRangeResponse::Multipart(ranges))
}

fn registryd_multipart_boundary(digest: Option<&str>) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in digest.unwrap_or("registryd").bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    format!("operating-system-registryd-boundary-{hash:016x}")
}

fn registryd_multipart_body(
    ranges: &[RegistrydByteRange],
    source: &[u8],
    len: usize,
    content_type: Option<&str>,
    boundary: &str,
) -> Vec<u8> {
    let content_type = content_type.unwrap_or("application/octet-stream");
    let mut body = Vec::new();
    for range in ranges {
        let end = range.start + range.length;
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!(
                "Content-Range: bytes {}-{}/{}\r\n",
                range.start,
                end.saturating_sub(1),
                len
            )
            .as_bytes(),
        );
        body.extend_from_slice(format!("Content-Type: {content_type}\r\n\r\n").as_bytes());
        body.extend_from_slice(&source[range.start..end]);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

    body
}

fn registryd_manifest_content_response(
    method: &str,
    request: &RegistrydApiRequest,
    plan: &RegistrydCachePathPlan,
    roots: &RegistrydMultiPathFs,
) -> Result<RegistrydContentResponse, RegistrydContentError> {
    match plan {
        RegistrydCachePathPlan::ManifestDigest { content_path } => {
            let RegistrydMultiPathFsRead::Found {
                path,
                bytes,
                modified,
                attempts: _,
            } = roots
                .read_file(content_path)
                .map_err(|_| RegistrydContentError::Internal)?
            else {
                return Err(RegistrydContentError::NotFound);
            };

            registryd_ok_manifest_response(method, request.reference.clone(), bytes, modified, path)
        }
        RegistrydCachePathPlan::ManifestTag { reference_path, .. } => {
            let RegistrydMultiPathFsRead::Found {
                bytes: reference_bytes,
                ..
            } = roots
                .read_file(reference_path)
                .map_err(|_| RegistrydContentError::Internal)?
            else {
                return Err(RegistrydContentError::NotFound);
            };

            let digest_resolution = plan
                .tagged_manifest_digest_resolution_from_reference(&reference_bytes)
                .map_err(|_| RegistrydContentError::Internal)?;

            let RegistrydMultiPathFsRead::Found {
                path,
                bytes: digest_bytes,
                modified,
                attempts: _,
            } = roots
                .read_file(&digest_resolution.digest_path)
                .map_err(|_| RegistrydContentError::Internal)?
            else {
                return Err(RegistrydContentError::Internal);
            };

            let digest_resolution = plan
                .resolve_tagged_manifest_digest(&reference_bytes, &digest_bytes)
                .map_err(|_| RegistrydContentError::Internal)?;

            registryd_ok_manifest_response(
                method,
                digest_resolution.canonical_digest,
                digest_bytes,
                modified,
                path,
            )
        }
        RegistrydCachePathPlan::Blob { .. } | RegistrydCachePathPlan::BlobTag { .. } => {
            Err(RegistrydContentError::NotFound)
        }
    }
}

fn registryd_ok_manifest_response(
    method: &str,
    digest: String,
    bytes: Vec<u8>,
    last_modified: Option<String>,
    path: PathBuf,
) -> Result<RegistrydContentResponse, RegistrydContentError> {
    let shape = RegistrydManifestShape::from_json_bytes(&bytes)
        .map_err(|_| RegistrydContentError::Internal)?;
    let media_type =
        registryd_manifest_media_type(&shape).map_err(|_| RegistrydContentError::Internal)?;

    Ok(RegistrydContentResponse::ok_manifest(
        method,
        digest,
        media_type,
        bytes,
        last_modified,
        path,
    ))
}

fn registryd_blob_content_response(
    method: &str,
    request: &RegistrydApiRequest,
    plan: &RegistrydCachePathPlan,
    roots: &RegistrydMultiPathFs,
) -> Result<RegistrydContentResponse, RegistrydContentError> {
    let (content_path, digest) = match plan {
        RegistrydCachePathPlan::Blob { content_path } => {
            (content_path.clone(), request.reference.clone())
        }
        RegistrydCachePathPlan::BlobTag { reference_path, .. } => {
            let RegistrydMultiPathFsRead::Found {
                bytes: reference_bytes,
                ..
            } = roots
                .read_file(reference_path)
                .map_err(|_| RegistrydContentError::Internal)?
            else {
                return Err(RegistrydContentError::NotFound);
            };

            let digest_resolution = plan
                .tagged_manifest_digest_resolution_from_reference(&reference_bytes)
                .map_err(|_| RegistrydContentError::Internal)?;

            let RegistrydMultiPathFsRead::Found {
                bytes: digest_bytes,
                ..
            } = roots
                .read_file(&digest_resolution.digest_path)
                .map_err(|_| RegistrydContentError::Internal)?
            else {
                return Err(RegistrydContentError::Internal);
            };

            let digest_resolution = plan
                .resolve_tagged_manifest_digest(&reference_bytes, &digest_bytes)
                .map_err(|_| RegistrydContentError::Internal)?;
            let content_path = PathBuf::from(REGISTRYD_BLOB_STORE_DIR).join(
                registryd_source_digest_file_name(&digest_resolution.canonical_digest)
                    .map_err(|_| RegistrydContentError::Internal)?,
            );

            (content_path, digest_resolution.canonical_digest)
        }
        _ => return Err(RegistrydContentError::NotFound),
    };

    let RegistrydMultiPathFsRead::Found {
        path,
        bytes,
        modified,
        attempts: _,
    } = roots
        .read_file(content_path)
        .map_err(|_| RegistrydContentError::Internal)?
    else {
        return Err(RegistrydContentError::NotFound);
    };

    Ok(RegistrydContentResponse::ok_blob(
        method, digest, bytes, modified, path,
    ))
}

fn extract_registryd_api_request(
    target: &str,
) -> std::result::Result<RegistrydApiRequest, RegistrydRouteError> {
    let (path, query) = registryd_target_path_and_query(target);
    let Some(args) = path.strip_prefix(REGISTRYD_API_PREFIX) else {
        return Err(RegistrydRouteError::NotFound);
    };

    let args = registryd_path_unescape(args).map_err(|()| RegistrydRouteError::BadRequest)?;
    let args = registryd_clean_route_args(&args);
    let args = args.trim_matches('/');
    if args.is_empty() {
        return Err(RegistrydRouteError::NotFound);
    }

    let parts: Vec<&str> = args.split('/').collect();
    if parts.len() < 3 {
        return Err(RegistrydRouteError::NotFound);
    }

    let kind = match parts[parts.len() - 2] {
        "manifests" => RegistrydApiContentKind::Manifest,
        "blobs" => RegistrydApiContentKind::Blob,
        _ => return Err(RegistrydRouteError::NotFound),
    };

    let name_parts = &parts[..parts.len() - 2];
    if !registryd_reference_name_is_source_shaped(name_parts) {
        return Err(RegistrydRouteError::BadRequest);
    }

    let reference = parts[parts.len() - 1];
    if reference.is_empty() {
        return Err(RegistrydRouteError::NotFound);
    }

    Ok(RegistrydApiRequest {
        registry: registryd_query_value(query, "ns"),
        name: name_parts.join("/"),
        reference: reference.to_string(),
        kind,
    })
}

fn registryd_method_can_use_get_handler(method: &str) -> bool {
    method == "GET" || method == "HEAD"
}

fn registryd_target_path(target: &str) -> &str {
    registryd_target_path_and_query(target).0
}

fn registryd_target_path_and_query(target: &str) -> (&str, Option<&str>) {
    target
        .split_once('?')
        .map_or((target, None), |(path, query)| (path, Some(query)))
}

fn registryd_clean_route_args(args: &str) -> String {
    let mut clean_parts = Vec::new();
    for part in args.split('/') {
        match part {
            "" | "." => {}
            ".." if clean_parts.last().is_some_and(|last| *last != "..") => {
                clean_parts.pop();
            }
            _ => clean_parts.push(part),
        }
    }

    if clean_parts.is_empty() {
        ".".to_string()
    } else {
        clean_parts.join("/")
    }
}

fn registryd_query_value(query: Option<&str>, key: &str) -> Option<String> {
    query?.split('&').find_map(|pair| {
        if pair.as_bytes().contains(&b';') {
            return None;
        }

        let (raw_key, raw_value) = pair.split_once('=').unwrap_or((pair, ""));
        let Ok(pair_key) = registryd_query_unescape(raw_key) else {
            return None;
        };
        if pair_key.as_ref() != key {
            return None;
        }

        registryd_query_unescape(raw_value)
            .ok()
            .map(Cow::into_owned)
    })
}

fn registryd_query_unescape(input: &str) -> Result<Cow<'_, str>, ()> {
    if !input
        .as_bytes()
        .iter()
        .any(|byte| matches!(byte, b'%' | b'+'))
    {
        return Ok(Cow::Borrowed(input));
    }

    let bytes = input.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut idx = 0;
    while idx < bytes.len() {
        match bytes[idx] {
            b'+' => {
                decoded.push(b' ');
                idx += 1;
            }
            b'%' => {
                let Some(high) = bytes
                    .get(idx + 1)
                    .and_then(|byte| registryd_hex_value(*byte))
                else {
                    return Err(());
                };
                let Some(low) = bytes
                    .get(idx + 2)
                    .and_then(|byte| registryd_hex_value(*byte))
                else {
                    return Err(());
                };
                decoded.push((high << 4) | low);
                idx += 3;
            }
            byte => {
                decoded.push(byte);
                idx += 1;
            }
        }
    }

    String::from_utf8(decoded).map(Cow::Owned).map_err(|_| ())
}

fn registryd_path_unescape(input: &str) -> Result<Cow<'_, str>, ()> {
    if !input.as_bytes().contains(&b'%') {
        return Ok(Cow::Borrowed(input));
    }

    let bytes = input.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut idx = 0;
    while idx < bytes.len() {
        match bytes[idx] {
            b'%' => {
                let Some(high) = bytes
                    .get(idx + 1)
                    .and_then(|byte| registryd_hex_value(*byte))
                else {
                    return Err(());
                };
                let Some(low) = bytes
                    .get(idx + 2)
                    .and_then(|byte| registryd_hex_value(*byte))
                else {
                    return Err(());
                };
                decoded.push((high << 4) | low);
                idx += 3;
            }
            byte => {
                decoded.push(byte);
                idx += 1;
            }
        }
    }

    String::from_utf8(decoded).map(Cow::Owned).map_err(|_| ())
}

fn registryd_hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn registryd_source_digest_file_name(digest: &str) -> Result<String, String> {
    let hex = digest
        .strip_prefix("sha256:")
        .ok_or_else(|| "digest must start with sha256:".to_string())?;
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    {
        return Err("digest must be 64 lowercase hex chars".to_string());
    }
    Ok(format!("sha256-{hex}"))
}

fn registryd_source_manifest_tag_is_valid(tag: &str) -> bool {
    let bytes = tag.as_bytes();
    let Some((&first, rest)) = bytes.split_first() else {
        return false;
    };
    bytes.len() <= 128
        && registryd_source_tag_word_byte(first)
        && rest
            .iter()
            .all(|byte| registryd_source_tag_word_byte(*byte) || matches!(*byte, b'.' | b'-'))
}

fn registryd_source_tag_word_byte(byte: u8) -> bool {
    matches!(byte, b'0'..=b'9' | b'A'..=b'Z' | b'_' | b'a'..=b'z')
}

fn registryd_sha256_digest_string(digest: [u8; 32]) -> String {
    let mut out = String::from("sha256:");
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn registryd_sha256(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a_2f98,
        0x7137_4491,
        0xb5c0_fbcf,
        0xe9b5_dba5,
        0x3956_c25b,
        0x59f1_11f1,
        0x923f_82a4,
        0xab1c_5ed5,
        0xd807_aa98,
        0x1283_5b01,
        0x2431_85be,
        0x550c_7dc3,
        0x72be_5d74,
        0x80de_b1fe,
        0x9bdc_06a7,
        0xc19b_f174,
        0xe49b_69c1,
        0xefbe_4786,
        0x0fc1_9dc6,
        0x240c_a1cc,
        0x2de9_2c6f,
        0x4a74_84aa,
        0x5cb0_a9dc,
        0x76f9_88da,
        0x983e_5152,
        0xa831_c66d,
        0xb003_27c8,
        0xbf59_7fc7,
        0xc6e0_0bf3,
        0xd5a7_9147,
        0x06ca_6351,
        0x1429_2967,
        0x27b7_0a85,
        0x2e1b_2138,
        0x4d2c_6dfc,
        0x5338_0d13,
        0x650a_7354,
        0x766a_0abb,
        0x81c2_c92e,
        0x9272_2c85,
        0xa2bf_e8a1,
        0xa81a_664b,
        0xc24b_8b70,
        0xc76c_51a3,
        0xd192_e819,
        0xd699_0624,
        0xf40e_3585,
        0x106a_a070,
        0x19a4_c116,
        0x1e37_6c08,
        0x2748_774c,
        0x34b0_bcb5,
        0x391c_0cb3,
        0x4ed8_aa4a,
        0x5b9c_ca4f,
        0x682e_6ff3,
        0x748f_82ee,
        0x78a5_636f,
        0x84c8_7814,
        0x8cc7_0208,
        0x90be_fffa,
        0xa450_6ceb,
        0xbef9_a3f7,
        0xc671_78f2,
    ];

    let mut h = [
        0x6a09_e667u32,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = Vec::with_capacity(((data.len() + 9).div_ceil(64)) * 64);
    msg.extend_from_slice(data);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (idx, word) in w.iter_mut().take(16).enumerate() {
            let offset = idx * 4;
            *word = u32::from_be_bytes([
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ]);
        }
        for idx in 16..64 {
            w[idx] = registryd_small_sigma1(w[idx - 2])
                .wrapping_add(w[idx - 7])
                .wrapping_add(registryd_small_sigma0(w[idx - 15]))
                .wrapping_add(w[idx - 16]);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];

        for idx in 0..64 {
            let t1 = hh
                .wrapping_add(registryd_big_sigma1(e))
                .wrapping_add((e & f) ^ ((!e) & g))
                .wrapping_add(K[idx])
                .wrapping_add(w[idx]);
            let t2 = registryd_big_sigma0(a).wrapping_add((a & b) ^ (a & c) ^ (b & c));
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for (idx, word) in h.iter().enumerate() {
        out[(idx * 4)..(idx * 4) + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

fn registryd_big_sigma0(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

fn registryd_big_sigma1(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}

fn registryd_small_sigma0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}

fn registryd_small_sigma1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}

fn registryd_handle_registry_with_port(named: &str, registry: &str) -> String {
    let Some(idx) = registry.rfind(':').filter(|idx| *idx > 0) else {
        return named.to_string();
    };
    let path = named.strip_prefix(registry).unwrap_or(named);
    format!("{}_{}_{}", &registry[..idx], &registry[idx + 1..], path)
}

fn registryd_source_store_name_from_query(registry: &str, name: &str) -> Option<String> {
    let named = registryd_source_normalized_named_from_query(registry, name)?;
    Some(registryd_handle_registry_with_port(&named, registry))
}

fn registryd_source_normalized_named_from_query(registry: &str, name: &str) -> Option<String> {
    let parts: Vec<&str> = registry.split('/').chain(name.split('/')).collect();
    let first = parts.first()?;

    let (domain, remote_parts) = if registryd_source_reference_first_component_is_domain(first) {
        if !registryd_source_registry_namespace_is_source_shaped(first) {
            return None;
        }

        let domain = if *first == "index.docker.io" {
            "docker.io"
        } else {
            first
        };
        (domain, &parts[1..])
    } else {
        ("docker.io", parts.as_slice())
    };

    if remote_parts.is_empty()
        || !remote_parts
            .iter()
            .all(|part| registryd_reference_path_component_is_source_shaped(part))
    {
        return None;
    }

    let mut remote = remote_parts.join("/");
    if domain == "docker.io" && !remote.contains('/') {
        remote = format!("library/{remote}");
    }

    Some(format!("{domain}/{remote}"))
}

fn registryd_source_reference_first_component_is_domain(component: &str) -> bool {
    component == "localhost"
        || component.starts_with('[')
        || component.contains('.')
        || component.contains(':')
}

fn registryd_source_registry_namespace_is_source_shaped(registry: &str) -> bool {
    if registry.is_empty()
        || registry.bytes().any(|byte| {
            byte.is_ascii_whitespace() || matches!(byte, b'/' | b'\\' | b'%' | b'?' | b'#')
        })
    {
        return false;
    }

    let Some((host, port)) = registryd_source_registry_host_port(registry) else {
        return false;
    };

    if port.is_some_and(|port| port.is_empty() || !port.bytes().all(|byte| byte.is_ascii_digit())) {
        return false;
    }

    if host == "localhost" {
        return true;
    }

    if host.starts_with('[') {
        return registryd_source_ipv6_host_is_source_shaped(host);
    }

    registryd_source_domain_host_is_source_shaped(host)
}

fn registryd_source_registry_host_port(registry: &str) -> Option<(&str, Option<&str>)> {
    if registry.starts_with('[') {
        let end = registry.find(']')?;
        let host_end = end + 1;
        let host = &registry[..host_end];
        let rest = &registry[host_end..];

        if rest.is_empty() {
            return Some((host, None));
        }

        return rest.strip_prefix(':').map(|port| (host, Some(port)));
    }

    match registry.rsplit_once(':') {
        Some((host, _)) if host.contains(':') => None,
        Some((host, port)) => Some((host, Some(port))),
        None => Some((registry, None)),
    }
}

fn registryd_source_ipv6_host_is_source_shaped(host: &str) -> bool {
    let Some(inner) = host
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return false;
    };

    !inner.is_empty()
        && inner.contains(':')
        && inner
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() || byte == b':')
}

fn registryd_source_domain_host_is_source_shaped(host: &str) -> bool {
    !host.is_empty()
        && host
            .split('.')
            .all(registryd_source_domain_component_is_source_shaped)
}

fn registryd_source_domain_component_is_source_shaped(component: &str) -> bool {
    let bytes = component.as_bytes();
    bytes
        .first()
        .is_some_and(|byte| registryd_source_domain_alphanumeric_byte(*byte))
        && bytes
            .last()
            .is_some_and(|byte| registryd_source_domain_alphanumeric_byte(*byte))
        && bytes
            .iter()
            .all(|byte| registryd_source_domain_alphanumeric_byte(*byte) || matches!(*byte, b'-'))
}

fn registryd_source_domain_alphanumeric_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
}

fn registryd_create_registry_with_port(store_entry: &str) -> String {
    let parts: Vec<&str> = store_entry.splitn(3, '_').collect();
    if parts.len() < 3 {
        return store_entry.to_string();
    }

    format!("{}:{}{}", parts[0], parts[1], parts[2])
}

fn registryd_absolutize_root(root: &Path) -> io::Result<PathBuf> {
    if root.is_absolute() {
        Ok(root.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(root))
    }
}

fn registryd_reference_name_is_source_shaped(parts: &[&str]) -> bool {
    !parts.is_empty()
        && parts
            .iter()
            .all(|part| registryd_reference_path_component_is_source_shaped(part))
}

fn registryd_reference_path_component_is_source_shaped(component: &str) -> bool {
    let bytes = component.as_bytes();
    if bytes
        .first()
        .is_none_or(|byte| !registryd_reference_alphanumeric_byte(*byte))
    {
        return false;
    }

    let mut idx = 1;
    while idx < bytes.len() {
        if registryd_reference_alphanumeric_byte(bytes[idx]) {
            idx += 1;
            continue;
        }

        match bytes[idx] {
            b'.' => idx += 1,
            b'_' => {
                idx += 1;
                if bytes.get(idx).is_some_and(|byte| *byte == b'_') {
                    idx += 1;
                }
            }
            b'-' => {
                while bytes.get(idx).is_some_and(|byte| *byte == b'-') {
                    idx += 1;
                }
            }
            _ => return false,
        }

        if bytes
            .get(idx)
            .is_none_or(|byte| !registryd_reference_alphanumeric_byte(*byte))
        {
            return false;
        }
    }

    true
}

fn registryd_reference_alphanumeric_byte(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit()
}

const IMAGE_CACHE_ISO_ROOT_DIR: &str = "imagecache";

/// Status of the image-cache config resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageCacheStatus {
    /// Source enum's zero value.
    Unknown,
    /// Feature disabled or all known volumes resolved without usable roots.
    Disabled,
    /// Roots are being discovered, mounted, copied, or registryd is not yet healthy.
    Preparing,
    /// At least one root exists and registryd is running+healthy.
    Ready,
}

impl ImageCacheStatus {
    /// Stable source spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            ImageCacheStatus::Unknown => "unknown",
            ImageCacheStatus::Disabled => "disabled",
            ImageCacheStatus::Preparing => "preparing",
            ImageCacheStatus::Ready => "ready",
        }
    }

    /// Parse the source enum spelling carried by type-erased COSI fingerprints.
    pub fn from_source_str(value: &str) -> Option<Self> {
        match value {
            "unknown" => Some(ImageCacheStatus::Unknown),
            "disabled" => Some(ImageCacheStatus::Disabled),
            "preparing" => Some(ImageCacheStatus::Preparing),
            "ready" => Some(ImageCacheStatus::Ready),
            _ => None,
        }
    }
}

/// Status of the ISO-to-disk image-cache copy operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageCacheCopyStatus {
    /// Source enum's zero value.
    Unknown,
    /// No ISO is present, or no disk target is configured.
    Skipped,
    /// ISO/disk copy prerequisites are not all ready.
    Pending,
    /// The cache is already copied or the current plan completes it.
    Ready,
}

impl ImageCacheCopyStatus {
    /// Stable source spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            ImageCacheCopyStatus::Unknown => "unknown",
            ImageCacheCopyStatus::Skipped => "skipped",
            ImageCacheCopyStatus::Pending => "copying",
            ImageCacheCopyStatus::Ready => "ready",
        }
    }

    /// Parse the source enum spelling carried by type-erased COSI fingerprints.
    pub fn from_source_str(value: &str) -> Option<Self> {
        match value {
            "unknown" => Some(ImageCacheCopyStatus::Unknown),
            "skipped" => Some(ImageCacheCopyStatus::Skipped),
            "copying" => Some(ImageCacheCopyStatus::Pending),
            "ready" => Some(ImageCacheCopyStatus::Ready),
            _ => None,
        }
    }
}

/// The projected `ImageCacheConfigSpec`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageCacheConfig {
    /// Overall status.
    pub status: ImageCacheStatus,
    /// ISO-to-disk copy status.
    pub copy_status: ImageCacheCopyStatus,
    /// Runtime image-cache roots, ordered disk first then ISO.
    pub roots: Vec<String>,
}

impl Default for ImageCacheConfig {
    fn default() -> Self {
        ImageCacheConfig {
            status: ImageCacheStatus::Unknown,
            copy_status: ImageCacheCopyStatus::Unknown,
            roots: Vec::new(),
        }
    }
}

impl ImageCacheConfig {
    /// Source `WaitForImageCache` condition: image cache is either disabled or ready.
    pub fn wait_for_image_cache_satisfied(&self) -> bool {
        matches!(
            self.status,
            ImageCacheStatus::Disabled | ImageCacheStatus::Ready
        )
    }

    /// Source `WaitForImageCacheCopy` condition: copy is either skipped or done/ready.
    pub fn wait_for_image_cache_copy_satisfied(&self) -> bool {
        matches!(
            self.copy_status,
            ImageCacheCopyStatus::Skipped | ImageCacheCopyStatus::Ready
        )
    }
}

/// Errors returned while evaluating source-shaped image-cache wait watches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageCacheWaitError {
    /// The requested COSI watch channel no longer exists.
    MissingWatch {
        /// Watched resource kind.
        kind: ResourceKind,
        /// Watch index returned by [`watch_image_cache_config`].
        index: usize,
    },
    /// The watch channel overran its bounded buffer.
    WatchOverrun {
        /// Watched resource kind.
        kind: ResourceKind,
        /// Watch index returned by [`watch_image_cache_config`].
        index: usize,
    },
    /// A same-kind singleton config resource could not be decoded.
    MalformedImageCacheConfig {
        /// Resource key.
        key: String,
        /// Resource fingerprint that failed to decode.
        fingerprint: String,
    },
}

impl fmt::Display for ImageCacheWaitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImageCacheWaitError::MissingWatch { kind, index } => {
                write!(f, "image-cache watch {kind}#{index} is not registered")
            }
            ImageCacheWaitError::WatchOverrun { kind, index } => {
                write!(f, "image-cache watch {kind}#{index} overran its buffer")
            }
            ImageCacheWaitError::MalformedImageCacheConfig { key, fingerprint } => write!(
                f,
                "image-cache config {key} has malformed fingerprint {fingerprint:?}"
            ),
        }
    }
}

impl std::error::Error for ImageCacheWaitError {}

/// Register a COSI watch for source `ImageCacheConfig` wait helpers.
pub fn watch_image_cache_config(state: &mut State, capacity: usize) -> usize {
    state.watch_kind(ImageCacheConfigResource::kind(), capacity)
}

/// Poll a registered watch for source `WaitForImageCache` completion.
pub fn poll_wait_for_image_cache(
    state: &mut State,
    watch_index: usize,
) -> Result<bool, ImageCacheWaitError> {
    poll_image_cache_watch(state, watch_index, |config| {
        config.wait_for_image_cache_satisfied()
    })
}

/// Poll a registered watch for source `WaitForImageCacheCopy` completion.
pub fn poll_wait_for_image_cache_copy(
    state: &mut State,
    watch_index: usize,
) -> Result<bool, ImageCacheWaitError> {
    poll_image_cache_watch(state, watch_index, |config| {
        config.wait_for_image_cache_copy_satisfied()
    })
}

/// Register and immediately poll the current state for `WaitForImageCache`.
pub fn wait_for_image_cache_in_state(
    state: &mut State,
    watch_capacity: usize,
) -> Result<bool, ImageCacheWaitError> {
    let watch_index = watch_image_cache_config(state, watch_capacity);
    poll_wait_for_image_cache(state, watch_index)
}

/// Register and immediately poll the current state for `WaitForImageCacheCopy`.
pub fn wait_for_image_cache_copy_in_state(
    state: &mut State,
    watch_capacity: usize,
) -> Result<bool, ImageCacheWaitError> {
    let watch_index = watch_image_cache_config(state, watch_capacity);
    poll_wait_for_image_cache_copy(state, watch_index)
}

fn poll_image_cache_watch(
    state: &mut State,
    watch_index: usize,
    predicate: impl Fn(&ImageCacheConfig) -> bool,
) -> Result<bool, ImageCacheWaitError> {
    let kind = ImageCacheConfigResource::kind();
    let channel =
        state
            .watch_mut(&kind, watch_index)
            .ok_or_else(|| ImageCacheWaitError::MissingWatch {
                kind: kind.clone(),
                index: watch_index,
            })?;

    if channel.is_overran() {
        return Err(ImageCacheWaitError::WatchOverrun {
            kind,
            index: watch_index,
        });
    }

    let events = channel.drain();
    if channel.is_overran() {
        return Err(ImageCacheWaitError::WatchOverrun {
            kind,
            index: watch_index,
        });
    }

    image_cache_wait_events_satisfied(&events, predicate)
}

fn image_cache_wait_events_satisfied(
    events: &[Event],
    predicate: impl Fn(&ImageCacheConfig) -> bool,
) -> Result<bool, ImageCacheWaitError> {
    let config_key =
        image_cache_config_key().map_err(|err| ImageCacheWaitError::MalformedImageCacheConfig {
            key: IMAGE_CACHE_CONFIG_ID.to_string(),
            fingerprint: err.to_string(),
        })?;

    for event in events {
        if !matches!(event.kind(), EventKind::Created | EventKind::Updated) {
            continue;
        }

        let Some(resource) = event.resource() else {
            continue;
        };
        if resource.metadata().key() != config_key {
            continue;
        }

        let config = ImageCacheConfigResource::from_resource(resource.as_ref())?.spec;
        if predicate(&config) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// COSI resource form of Talos's CRI `ImageCacheConfig`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageCacheConfigResource {
    meta: Metadata,
    /// Projected image-cache config spec.
    pub spec: ImageCacheConfig,
}

impl ImageCacheConfigResource {
    /// Build the singleton image-cache config resource.
    pub fn new(spec: ImageCacheConfig) -> Self {
        ImageCacheConfigResource {
            meta: image_cache_config_metadata(),
            spec,
        }
    }

    /// Kind descriptor for `ImageCacheConfig`.
    pub fn kind() -> ResourceKind {
        ResourceKind::new(IMAGE_CACHE_NAMESPACE, IMAGE_CACHE_CONFIG_TYPE)
    }

    /// Borrow the COSI metadata.
    pub fn metadata(&self) -> &Metadata {
        &self.meta
    }

    /// Mutably borrow the COSI metadata.
    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    /// Convert a type-erased COSI resource of the same singleton kind back into config.
    pub fn from_resource(resource: &dyn Resource) -> Result<Self, ImageCacheWaitError> {
        if resource.resource_kind() != Self::kind() {
            return Err(ImageCacheWaitError::MalformedImageCacheConfig {
                key: resource.metadata().key(),
                fingerprint: resource.spec_fingerprint(),
            });
        }

        let fingerprint = resource.spec_fingerprint();
        let spec = image_cache_config_from_fingerprint(&fingerprint).ok_or_else(|| {
            ImageCacheWaitError::MalformedImageCacheConfig {
                key: resource.metadata().key(),
                fingerprint: fingerprint.clone(),
            }
        })?;

        Ok(ImageCacheConfigResource {
            meta: resource.metadata().clone(),
            spec,
        })
    }
}

fn image_cache_config_from_fingerprint(fingerprint: &str) -> Option<ImageCacheConfig> {
    let rest = fingerprint.strip_prefix("status=")?;
    let (status, rest) = rest.split_once(";copy_status=")?;
    let (copy_status, roots) = rest.split_once(";roots=[")?;
    let roots = roots.strip_suffix(']')?;

    Some(ImageCacheConfig {
        status: ImageCacheStatus::from_source_str(status)?,
        copy_status: ImageCacheCopyStatus::from_source_str(copy_status)?,
        roots: if roots.is_empty() {
            Vec::new()
        } else {
            roots.split(',').map(ToOwned::to_owned).collect()
        },
    })
}

impl Resource for ImageCacheConfigResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        format!(
            "status={};copy_status={};roots=[{}]",
            self.spec.status.as_str(),
            self.spec.copy_status.as_str(),
            self.spec.roots.join(",")
        )
    }

    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

/// Boot-owned copy completion state that stands in for source controller memory.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ImageCacheCopyState {
    /// Whether the privileged ISO-to-disk copy has completed successfully.
    pub done: bool,
}

impl ImageCacheCopyState {
    /// Build a successful-copy marker.
    pub fn done() -> Self {
        ImageCacheCopyState { done: true }
    }
}

/// COSI resource form of the Rust boot bridge's image-cache copy completion marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageCacheCopyStateResource {
    meta: Metadata,
    /// Projected copy completion state.
    pub spec: ImageCacheCopyState,
}

impl ImageCacheCopyStateResource {
    /// Build the singleton copy state resource.
    pub fn new(spec: ImageCacheCopyState) -> Self {
        ImageCacheCopyStateResource {
            meta: image_cache_copy_state_metadata(),
            spec,
        }
    }

    /// Kind descriptor for `ImageCacheCopyState`.
    pub fn kind() -> ResourceKind {
        ResourceKind::new(IMAGE_CACHE_NAMESPACE, IMAGE_CACHE_COPY_STATE_TYPE)
    }

    /// Borrow the COSI metadata.
    pub fn metadata(&self) -> &Metadata {
        &self.meta
    }

    /// Mutably borrow the COSI metadata.
    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    /// Convert a type-erased COSI resource of the same kind back into copy state.
    pub fn from_resource(resource: &dyn Resource) -> Option<Self> {
        if resource.resource_kind() != Self::kind() {
            return None;
        }

        let done = resource
            .spec_fingerprint()
            .strip_prefix("done=")?
            .parse()
            .ok()?;
        Some(ImageCacheCopyStateResource {
            meta: resource.metadata().clone(),
            spec: ImageCacheCopyState { done },
        })
    }
}

impl Resource for ImageCacheCopyStateResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        format!("done={}", self.spec.done)
    }

    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

/// Source-shaped Talos v1alpha1 Service spec.
///
/// This intentionally models only the status booleans consumed by
/// `ImageCacheConfigController` for the registryd input.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct V1Alpha1ServiceSpec {
    /// Whether the service is running.
    pub running: bool,
    /// Whether the service is healthy.
    pub healthy: bool,
    /// Whether service health is unknown.
    pub unknown: bool,
}

/// COSI resource form of a Talos v1alpha1 Service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V1Alpha1ServiceResource {
    meta: Metadata,
    /// Projected source service state.
    pub spec: V1Alpha1ServiceSpec,
}

impl V1Alpha1ServiceResource {
    /// Build a v1alpha1 Service resource in the source namespace/type.
    pub fn new(id: impl Into<String>, spec: V1Alpha1ServiceSpec) -> os_kernel::Result<Self> {
        Ok(V1Alpha1ServiceResource {
            meta: Metadata::new(
                V1ALPHA1_NAMESPACE,
                V1ALPHA1_SERVICE_TYPE,
                ResourceId::new(id.into())?,
            ),
            spec,
        })
    }

    /// Build the registryd v1alpha1 Service singleton.
    pub fn registryd(spec: V1Alpha1ServiceSpec) -> os_kernel::Result<Self> {
        Self::new(REGISTRYD_SERVICE_ID, spec)
    }

    /// Kind descriptor for Talos v1alpha1 Services.
    pub fn kind() -> ResourceKind {
        registryd_service_kind()
    }

    /// Borrow the COSI metadata.
    pub fn metadata(&self) -> &Metadata {
        &self.meta
    }

    /// Mutably borrow the COSI metadata.
    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    /// Convert a type-erased COSI resource of the same kind back into Service state.
    pub fn from_resource(resource: &dyn Resource) -> Option<Self> {
        if resource.resource_kind() != Self::kind() {
            return None;
        }

        Some(V1Alpha1ServiceResource {
            meta: resource.metadata().clone(),
            spec: parse_v1alpha1_service_spec_fingerprint(&resource.spec_fingerprint())?,
        })
    }
}

impl Resource for V1Alpha1ServiceResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        format!(
            "running={};healthy={};unknown={}",
            self.spec.running, self.spec.healthy, self.spec.unknown
        )
    }

    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

fn parse_v1alpha1_service_spec_fingerprint(fingerprint: &str) -> Option<V1Alpha1ServiceSpec> {
    let mut running = None;
    let mut healthy = None;
    let mut unknown = None;

    for field in fingerprint.split(';') {
        let (key, value) = field.split_once('=')?;
        match key {
            "running" => running = Some(value.parse().ok()?),
            "healthy" => healthy = Some(value.parse().ok()?),
            "unknown" => unknown = Some(value.parse().ok()?),
            _ => return None,
        }
    }

    Some(V1Alpha1ServiceSpec {
        running: running?,
        healthy: healthy?,
        unknown: unknown?,
    })
}

/// Observed registryd service state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RegistrydState {
    /// Whether registryd is running.
    pub running: bool,
    /// Whether registryd reports healthy.
    pub healthy: bool,
}

impl From<V1Alpha1ServiceSpec> for RegistrydState {
    fn from(spec: V1Alpha1ServiceSpec) -> Self {
        RegistrydState {
            running: spec.running,
            healthy: spec.running && spec.healthy,
        }
    }
}

/// Planned registryd action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistrydAction {
    /// No service operation is needed.
    None,
    /// Start registryd because at least one image-cache root is available.
    Start,
}

/// Error emitted by a runtime registryd service adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistrydServiceError {
    /// Service manager failed to report whether registryd is running.
    IsRunning { service_id: String, message: String },
    /// Service manager failed to start registryd.
    Start { service_id: String, message: String },
    /// Service manager failed to apply a registryd health observation.
    Health { service_id: String, message: String },
}

impl fmt::Display for RegistrydServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistrydServiceError::IsRunning {
                service_id,
                message,
            } => write!(
                f,
                "error checking whether registryd service {service_id} is running: {message}"
            ),
            RegistrydServiceError::Start {
                service_id,
                message,
            } => write!(
                f,
                "error starting registryd service {service_id}: {message}"
            ),
            RegistrydServiceError::Health {
                service_id,
                message,
            } => write!(
                f,
                "error observing registryd service {service_id} health: {message}"
            ),
        }
    }
}

impl std::error::Error for RegistrydServiceError {}

/// Minimal service-manager boundary needed for Talos registryd load/start parity.
///
/// Source Talos uses `V1Alpha1ServiceManager.IsRunning`, loads
/// `services.NewRegistryD()` when that lookup fails, then starts
/// `services.RegistryID` if it is not running. This trait keeps that mutable
/// service effect outside the host-safe COSI resource reconciliation path.
pub trait RegistrydServiceManager {
    /// Return whether `service_id` is running.
    fn is_running(&mut self, service_id: &str) -> std::result::Result<bool, RegistrydServiceError>;

    /// Load the registryd service definition with the runtime root bridge.
    fn load_registryd(&mut self, service: RegistrydRuntimeService);

    /// Start `service_id`.
    fn start(&mut self, service_id: &str) -> std::result::Result<(), RegistrydServiceError>;
}

/// Runtime outcome for a registryd service adapter attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistrydServiceExecutionStatus {
    /// The image-cache plan had no registryd work to perform.
    NoAction,
    /// Registryd was already running according to the service manager.
    AlreadyRunning,
    /// Registryd was not running and was started.
    Started,
    /// Registryd lookup failed, so the service was loaded and then started.
    LoadedAndStarted,
}

/// Source-shaped report for registryd runtime service effects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrydServiceReport {
    /// Execution outcome.
    pub status: RegistrydServiceExecutionStatus,
    /// Service id used for manager calls.
    pub service_id: String,
    /// Whether the adapter loaded the registryd service definition.
    pub loaded: bool,
    /// Whether the adapter started registryd.
    pub started: bool,
}

impl RegistrydServiceReport {
    fn new(status: RegistrydServiceExecutionStatus, loaded: bool, started: bool) -> Self {
        RegistrydServiceReport {
            status,
            service_id: REGISTRYD_SERVICE_ID.to_string(),
            loaded,
            started,
        }
    }

    fn no_action() -> Self {
        Self::new(RegistrydServiceExecutionStatus::NoAction, false, false)
    }

    fn already_running() -> Self {
        Self::new(
            RegistrydServiceExecutionStatus::AlreadyRunning,
            false,
            false,
        )
    }

    fn started(loaded: bool) -> Self {
        let status = if loaded {
            RegistrydServiceExecutionStatus::LoadedAndStarted
        } else {
            RegistrydServiceExecutionStatus::Started
        };
        Self::new(status, loaded, true)
    }
}

/// Explicit effect adapter for registryd service load/start plans.
///
/// The pure image-cache controller only reports `RegistrydAction::Start`.
/// Boot/runtime code that owns a real service manager can cross this adapter
/// boundary to perform Talos-compatible load/start behavior.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RegistrydRuntimeAdapter;

impl RegistrydRuntimeAdapter {
    /// Execute registryd service effects requested by a host-safe image-cache plan.
    pub fn execute(
        self,
        plan: &ImageCacheRuntimePlan,
        manager: &mut dyn RegistrydServiceManager,
    ) -> std::result::Result<RegistrydServiceReport, RegistrydServiceError> {
        if plan.registryd_action == RegistrydAction::None || plan.config.roots.is_empty() {
            return Ok(RegistrydServiceReport::no_action());
        }

        let mut loaded = false;
        let running = match manager.is_running(REGISTRYD_SERVICE_ID) {
            Ok(running) => running,
            Err(_err) => {
                manager.load_registryd(RegistrydRuntimeService::from_runtime_plan(plan));
                loaded = true;
                false
            }
        };

        if running {
            return Ok(RegistrydServiceReport::already_running());
        }

        manager.start(REGISTRYD_SERVICE_ID)?;
        Ok(RegistrydServiceReport::started(loaded))
    }
}

/// One image-cache root skipped by the source-shaped registryd runner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrydRuntimeRootSkip {
    /// Root from `ImageCacheConfig.roots`.
    pub root: PathBuf,
    /// Stable `std::io` error kind observed while checking the root.
    pub error_kind: io::ErrorKind,
}

/// Host-safe registryd service runtime bound to the current image-cache roots.
///
/// Source `services.NewRegistryD().Runner` reads the CRI `ImageCacheConfig`,
/// `os.Stat`s each configured root, yields only existing roots into
/// `registry.NewMultiPathFS`, and then runs the registry HTTP service. This
/// model performs the same read-only root selection and delegates recognized
/// requests to [`RegistrydHttpService::handle_cached_content_request`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrydRuntimeService {
    roots: RegistrydMultiPathFs,
    skipped_roots: Vec<RegistrydRuntimeRootSkip>,
}

impl RegistrydRuntimeService {
    /// Build a source-shaped registryd runtime service from an image-cache config.
    pub fn from_image_cache_config(config: &ImageCacheConfig) -> Self {
        let mut roots = Vec::new();
        let mut skipped_roots = Vec::new();

        for root in &config.roots {
            let path = PathBuf::from(root);
            match fs::metadata(&path) {
                Ok(_) => roots.push(path),
                Err(err) => skipped_roots.push(RegistrydRuntimeRootSkip {
                    root: path,
                    error_kind: err.kind(),
                }),
            }
        }

        RegistrydRuntimeService {
            roots: RegistrydMultiPathFs::new(roots),
            skipped_roots,
        }
    }

    /// Build a runtime service from a full image-cache runtime plan.
    pub fn from_runtime_plan(plan: &ImageCacheRuntimePlan) -> Self {
        Self::from_image_cache_config(&plan.config)
    }

    /// Existing roots yielded to the source-shaped `MultiPathFS`.
    pub fn roots(&self) -> &RegistrydMultiPathFs {
        &self.roots
    }

    /// Configured roots skipped because their source `Stat` check failed.
    pub fn skipped_roots(&self) -> &[RegistrydRuntimeRootSkip] {
        &self.skipped_roots
    }

    /// Serve a registryd request from the runtime-selected image-cache roots.
    pub fn handle_request(&self, method: &str, target: &str) -> Option<RegistrydContentResponse> {
        RegistrydHttpService::source().handle_cached_content_request(method, target, &self.roots)
    }
}

/// Host-safe plan for one source-shaped volume mount request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageCacheMountRequestPlan {
    /// COSI id: `cri.ImageCacheConfigController-<volume id>`.
    pub id: String,
    /// Requester/volume/read-only fields carried by Talos `VolumeMountRequest`.
    pub spec: MountRequestSpec,
}

/// Source `block.VolumeConfig.Type` values needed by Image Cache.
pub type SourceBlockVolumeType = os_block_domain::VolumeType;

/// Source-shaped partition provisioning subset used by the image-cache volumes.
pub type ImageCacheVolumeProvisioningSpec = os_block_domain::VolumeConfigProvisioningSpec;

/// Source `block.VolumeConfig.Mount` subset written by Image Cache.
pub type ImageCacheVolumeMountSpec = os_block_domain::VolumeConfigMountSpec;

/// Source-shaped `block.VolumeConfig` spec for Image Cache COSI output.
pub type ImageCacheVolumeConfigSpec = os_block_domain::VolumeConfigSpec;

/// Generic COSI resource wrapper for source block `VolumeConfig` output.
pub type ImageCacheVolumeConfigResource = os_block_domain::VolumeConfigResource;

fn image_cache_volume_mount_spec(target_path: &str) -> ImageCacheVolumeMountSpec {
    ImageCacheVolumeMountSpec::new(target_path, 0o700, 0, 0)
}

fn image_cache_disk_volume_mount_spec() -> ImageCacheVolumeMountSpec {
    image_cache_volume_mount_spec(IMAGE_CACHE_DISK_MOUNT_POINT)
}

fn image_cache_iso_volume_mount_spec() -> ImageCacheVolumeMountSpec {
    image_cache_volume_mount_spec(IMAGE_CACHE_ISO_MOUNT_POINT)
}

/// Finalizer action needed for a `VolumeMountStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageCacheFinalizerOperation {
    /// Add the controller finalizer while using the mounted root.
    Add,
    /// Remove the controller finalizer while the mount status tears down.
    Remove,
}

/// A finalizer mutation plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageCacheFinalizerAction {
    /// VolumeMountStatus id.
    pub status_id: String,
    /// Mutation to apply.
    pub operation: ImageCacheFinalizerOperation,
}

/// Host-safe representation of a copy that Talos would perform on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageCacheCopyPlan {
    /// ISO image-cache root (`/system/imagecache/iso/imagecache`).
    pub source: String,
    /// Disk image-cache target (`/system/imagecache/disk`).
    pub target: String,
}

/// Explicit execution gate for image-cache copy plans.
///
/// The default is host-safe and mirrors Talos's `DisableCacheCopy` test knob:
/// copy intent remains visible, but no filesystem mutation happens unless a
/// caller opts into execution for a controlled test or VM/runtime adapter.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ImageCacheCopyGate {
    /// Do not execute the copy plan.
    #[default]
    Disabled,
    /// Execute the copy plan against the provided source/target paths.
    Enabled,
}

/// Result status for an explicit image-cache copy attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageCacheCopyExecutionStatus {
    /// No copy plan was provided.
    NoPlan,
    /// A plan existed, but the runtime environment is not allowed to mutate
    /// image-cache storage.
    DisabledByEnvironment,
    /// A plan existed but the explicit execution gate was disabled.
    DisabledByGate,
    /// The enabled copy walk completed successfully.
    Copied,
}

/// Source-shaped copy accounting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageCacheCopyReport {
    /// Execution outcome.
    pub status: ImageCacheCopyExecutionStatus,
    /// Source root from the copy plan.
    pub source: String,
    /// Target root from the copy plan.
    pub target: String,
    /// Number of regular files physically copied through a temporary file.
    pub files_copied: usize,
    /// Number of regular files skipped because the destination was byte-identical.
    pub files_skipped: usize,
    /// Number of target directories newly created by the copy walk.
    pub directories_created: usize,
    /// Sum of source regular-file sizes observed by the walk.
    pub bytes_copied: u64,
}

impl ImageCacheCopyReport {
    fn no_plan() -> Self {
        ImageCacheCopyReport {
            status: ImageCacheCopyExecutionStatus::NoPlan,
            source: String::new(),
            target: String::new(),
            files_copied: 0,
            files_skipped: 0,
            directories_created: 0,
            bytes_copied: 0,
        }
    }

    fn disabled_by_gate(plan: &ImageCacheCopyPlan) -> Self {
        ImageCacheCopyReport {
            status: ImageCacheCopyExecutionStatus::DisabledByGate,
            source: plan.source.clone(),
            target: plan.target.clone(),
            files_copied: 0,
            files_skipped: 0,
            directories_created: 0,
            bytes_copied: 0,
        }
    }

    fn disabled_by_environment(plan: &ImageCacheCopyPlan) -> Self {
        ImageCacheCopyReport {
            status: ImageCacheCopyExecutionStatus::DisabledByEnvironment,
            source: plan.source.clone(),
            target: plan.target.clone(),
            files_copied: 0,
            files_skipped: 0,
            directories_created: 0,
            bytes_copied: 0,
        }
    }

    fn copied(plan: &ImageCacheCopyPlan) -> Self {
        ImageCacheCopyReport {
            status: ImageCacheCopyExecutionStatus::Copied,
            source: plan.source.clone(),
            target: plan.target.clone(),
            files_copied: 0,
            files_skipped: 0,
            directories_created: 0,
            bytes_copied: 0,
        }
    }
}

/// Error emitted by the explicit image-cache copy executor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageCacheCopyError {
    /// Source tree walk or metadata lookup failed.
    WalkSource { path: String, message: String },
    /// Relative path calculation failed.
    RelativePath { path: String, message: String },
    /// Target directory creation failed.
    CreateDirectory { path: String, message: String },
    /// Regular-file copy failed.
    CopyFile {
        source: String,
        target: String,
        message: String,
    },
    /// The source walk found something other than a directory or regular file.
    UnsupportedFileType { path: String, file_type: String },
}

impl fmt::Display for ImageCacheCopyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImageCacheCopyError::WalkSource { path, message } => {
                write!(f, "error walking source directory {path}: {message}")
            }
            ImageCacheCopyError::RelativePath { path, message } => {
                write!(f, "error getting relative path for {path}: {message}")
            }
            ImageCacheCopyError::CreateDirectory { path, message } => {
                write!(f, "error creating directory {path}: {message}")
            }
            ImageCacheCopyError::CopyFile {
                source,
                target,
                message,
            } => {
                write!(f, "error copying file {source} to {target}: {message}")
            }
            ImageCacheCopyError::UnsupportedFileType { path, file_type } => {
                write!(f, "unsupported file type {file_type}: {path}")
            }
        }
    }
}

impl std::error::Error for ImageCacheCopyError {}

/// Execute a copy plan only when explicitly enabled.
///
/// This mirrors the source controller's directory/regular-file-only semantics
/// while keeping host/runtime side effects out of the pure reconciliation path.
pub fn execute_image_cache_copy_plan(
    plan: Option<&ImageCacheCopyPlan>,
    gate: ImageCacheCopyGate,
) -> Result<ImageCacheCopyReport, ImageCacheCopyError> {
    let Some(plan) = plan else {
        return Ok(ImageCacheCopyReport::no_plan());
    };

    if gate == ImageCacheCopyGate::Disabled {
        return Ok(ImageCacheCopyReport::disabled_by_gate(plan));
    }

    copy_image_cache_tree(plan)
}

/// Runtime environment where an image-cache copy adapter is running.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ImageCacheCopyRuntimeEnvironment {
    /// Ordinary host/unit-test/controller mode: copy intent may be observed but
    /// storage mutation is not allowed.
    #[default]
    HostSafe,
    /// Explicit VM/privileged mode where the caller owns the storage effect
    /// boundary and may opt into copy execution with [`ImageCacheCopyGate`].
    VmPrivileged,
}

/// Explicit effect adapter for runtime image-cache copy plans.
///
/// The COSI controller remains host-safe and only emits copy intent. Future
/// boot/VM runtime code should cross this adapter boundary after proving it is
/// operating in a privileged image-cache storage context. Both environment and
/// copy gates must be open before filesystem mutation can happen.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ImageCacheCopyRuntimeAdapter {
    environment: ImageCacheCopyRuntimeEnvironment,
    gate: ImageCacheCopyGate,
}

impl ImageCacheCopyRuntimeAdapter {
    /// Build a copy adapter with an explicit environment and copy gate.
    pub fn new(environment: ImageCacheCopyRuntimeEnvironment, gate: ImageCacheCopyGate) -> Self {
        ImageCacheCopyRuntimeAdapter { environment, gate }
    }

    /// Runtime environment configured on this adapter.
    pub fn environment(self) -> ImageCacheCopyRuntimeEnvironment {
        self.environment
    }

    /// Copy execution gate configured on this adapter.
    pub fn gate(self) -> ImageCacheCopyGate {
        self.gate
    }

    /// Execute the copy plan from a runtime plan only when both gates allow it.
    pub fn execute(
        self,
        plan: &ImageCacheRuntimePlan,
    ) -> Result<ImageCacheCopyReport, ImageCacheCopyError> {
        let Some(copy_plan) = plan.copy_plan.as_ref() else {
            return Ok(ImageCacheCopyReport::no_plan());
        };

        if self.environment != ImageCacheCopyRuntimeEnvironment::VmPrivileged {
            return Ok(ImageCacheCopyReport::disabled_by_environment(copy_plan));
        }

        execute_image_cache_copy_plan(Some(copy_plan), self.gate)
    }
}

fn copy_image_cache_tree(
    plan: &ImageCacheCopyPlan,
) -> Result<ImageCacheCopyReport, ImageCacheCopyError> {
    let source = Path::new(&plan.source);
    let target = Path::new(&plan.target);
    let source_type = fs::symlink_metadata(source).map_err(|err| walk_source_error(source, err))?;
    if !source_type.is_dir() {
        return Err(ImageCacheCopyError::UnsupportedFileType {
            path: source.display().to_string(),
            file_type: format!("{:?}", source_type.file_type()),
        });
    }

    let mut report = ImageCacheCopyReport::copied(plan);
    create_copy_directory(target, &mut report)?;
    copy_image_cache_dir(source, source, target, &mut report)?;
    Ok(report)
}

fn copy_image_cache_dir(
    source_root: &Path,
    current_source: &Path,
    target_root: &Path,
    report: &mut ImageCacheCopyReport,
) -> Result<(), ImageCacheCopyError> {
    for entry in read_source_dir_lexical(current_source)? {
        let source_path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|err| walk_source_error(&source_path, err))?;
        let relative_path = source_path.strip_prefix(source_root).map_err(|err| {
            ImageCacheCopyError::RelativePath {
                path: source_path.display().to_string(),
                message: err.to_string(),
            }
        })?;
        let target_path = target_root.join(relative_path);

        if file_type.is_dir() {
            create_copy_directory(&target_path, report)?;
            copy_image_cache_dir(source_root, &source_path, target_root, report)?;
        } else if file_type.is_file() {
            let source_metadata =
                entry
                    .metadata()
                    .map_err(|err| ImageCacheCopyError::CopyFile {
                        source: source_path.display().to_string(),
                        target: target_path.display().to_string(),
                        message: format!("error getting source file info: {err}"),
                    })?;
            report.bytes_copied += source_metadata.len();
            if copy_file_safe(&source_path, &target_path)? {
                report.files_copied += 1;
            } else {
                report.files_skipped += 1;
            }
        } else {
            return Err(ImageCacheCopyError::UnsupportedFileType {
                path: source_path.display().to_string(),
                file_type: format!("{file_type:?}"),
            });
        }
    }

    Ok(())
}

fn create_copy_directory(
    target_path: &Path,
    report: &mut ImageCacheCopyReport,
) -> Result<(), ImageCacheCopyError> {
    let existed = target_path.is_dir();
    create_dir_all_source_mode(target_path).map_err(|err| {
        ImageCacheCopyError::CreateDirectory {
            path: target_path.display().to_string(),
            message: err.to_string(),
        }
    })?;
    if !existed {
        report.directories_created += 1;
    }
    Ok(())
}

fn copy_file_safe(source: &Path, target: &Path) -> Result<bool, ImageCacheCopyError> {
    let source_metadata = fs::metadata(source).map_err(|err| ImageCacheCopyError::CopyFile {
        source: source.display().to_string(),
        target: target.display().to_string(),
        message: format!("error getting source file info: {err}"),
    })?;

    if let Ok(target_metadata) = fs::metadata(target)
        && source_metadata.len() == target_metadata.len()
        && copy_file_contents_match(source, target)?
    {
        return Ok(false);
    }

    if let Some(parent) = target.parent() {
        create_dir_all_source_mode(parent).map_err(|err| ImageCacheCopyError::CreateDirectory {
            path: parent.display().to_string(),
            message: err.to_string(),
        })?;
    }

    let mut source_file = fs::File::open(source).map_err(|err| ImageCacheCopyError::CopyFile {
        source: source.display().to_string(),
        target: target.display().to_string(),
        message: format!("error opening source file: {err}"),
    })?;
    let temp_path = temp_copy_path(target);
    let mut target_file =
        fs::File::create(&temp_path).map_err(|err| ImageCacheCopyError::CopyFile {
            source: source.display().to_string(),
            target: target.display().to_string(),
            message: format!("error creating destination file: {err}"),
        })?;

    if let Err(err) = io::copy(&mut source_file, &mut target_file) {
        drop(target_file);
        let cleanup = temp_copy_cleanup_message(&temp_path);
        return Err(ImageCacheCopyError::CopyFile {
            source: source.display().to_string(),
            target: target.display().to_string(),
            message: copy_error_with_temp_cleanup(
                format!(
                    "error copying file, source size is {}: {err}",
                    source_metadata.len()
                ),
                cleanup,
            ),
        });
    }
    drop(target_file);

    if let Err(err) = fs::rename(&temp_path, target) {
        let cleanup = temp_copy_cleanup_message(&temp_path);
        return Err(ImageCacheCopyError::CopyFile {
            source: source.display().to_string(),
            target: target.display().to_string(),
            message: copy_error_with_temp_cleanup(format!("error renaming file: {err}"), cleanup),
        });
    }

    Ok(true)
}

#[cfg(unix)]
fn create_dir_all_source_mode(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;

    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o755)
        .create(path)
}

#[cfg(not(unix))]
fn create_dir_all_source_mode(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)
}

fn temp_copy_cleanup_message(temp_path: &Path) -> Option<String> {
    match fs::remove_file(temp_path) {
        Ok(()) => None,
        Err(err) if err.kind() == io::ErrorKind::NotFound => None,
        Err(err) => Some(format!(
            "failed to remove temporary copy {}: {err}",
            temp_path.display()
        )),
    }
}

fn copy_error_with_temp_cleanup(error: String, cleanup: Option<String>) -> String {
    match cleanup {
        Some(cleanup) => format!("{error}; {cleanup}"),
        None => error,
    }
}

fn copy_file_contents_match(source: &Path, target: &Path) -> Result<bool, ImageCacheCopyError> {
    let mut source_file = fs::File::open(source).map_err(|err| ImageCacheCopyError::CopyFile {
        source: source.display().to_string(),
        target: target.display().to_string(),
        message: format!("error opening source file for comparison: {err}"),
    })?;
    let mut target_file = fs::File::open(target).map_err(|err| ImageCacheCopyError::CopyFile {
        source: source.display().to_string(),
        target: target.display().to_string(),
        message: format!("error opening destination file for comparison: {err}"),
    })?;
    let mut source_buf = [0_u8; 8192];
    let mut target_buf = [0_u8; 8192];

    loop {
        let source_len =
            source_file
                .read(&mut source_buf)
                .map_err(|err| ImageCacheCopyError::CopyFile {
                    source: source.display().to_string(),
                    target: target.display().to_string(),
                    message: format!("error reading source file for comparison: {err}"),
                })?;
        let target_len =
            target_file
                .read(&mut target_buf)
                .map_err(|err| ImageCacheCopyError::CopyFile {
                    source: source.display().to_string(),
                    target: target.display().to_string(),
                    message: format!("error reading destination file for comparison: {err}"),
                })?;

        if source_len != target_len {
            return Ok(false);
        }
        if source_len == 0 {
            return Ok(true);
        }
        if source_buf[..source_len] != target_buf[..target_len] {
            return Ok(false);
        }
    }
}

fn temp_copy_path(target: &Path) -> PathBuf {
    let mut temp_path = target.as_os_str().to_os_string();
    temp_path.push(".tmp");
    PathBuf::from(temp_path)
}

fn read_source_dir_lexical(
    current_source: &Path,
) -> Result<Vec<fs::DirEntry>, ImageCacheCopyError> {
    let mut entries = fs::read_dir(current_source)
        .map_err(|err| walk_source_error(current_source, err))?
        .map(|entry| entry.map_err(|err| walk_source_error(current_source, err)))
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    Ok(entries)
}

#[cfg(test)]
fn source_walk_order_for_test<I, S>(names: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut names = names.into_iter().map(Into::into).collect::<Vec<_>>();
    names.sort();
    names
}

fn walk_source_error(path: &Path, err: io::Error) -> ImageCacheCopyError {
    ImageCacheCopyError::WalkSource {
        path: path.display().to_string(),
        message: err.to_string(),
    }
}

/// Inputs needed to reconcile image-cache runtime state.
#[derive(Debug, Clone, Copy)]
pub struct ImageCacheReconcileInput<'a> {
    /// Machine feature flag: `machine.features.imageCache.localEnabled`.
    pub local_enabled: bool,
    /// Observed registryd state.
    pub registryd: RegistrydState,
    /// Known block volume statuses.
    pub volume_statuses: &'a [VolumeStatus],
    /// Known block volume mount statuses.
    pub volume_mount_statuses: &'a [VolumeMountStatusResource],
}

/// Full host-safe reconciliation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageCacheRuntimePlan {
    /// Projected `ImageCacheConfig`.
    pub config: ImageCacheConfig,
    /// Desired mount requests to write.
    pub mount_requests: Vec<ImageCacheMountRequestPlan>,
    /// Finalizer mutations needed for currently observed mount statuses.
    pub finalizer_actions: Vec<ImageCacheFinalizerAction>,
    /// Copy intent when ISO and disk roots are ready and copy was not already done.
    pub copy_plan: Option<ImageCacheCopyPlan>,
    /// Service action needed for registryd.
    pub registryd_action: RegistrydAction,
}

impl Default for ImageCacheRuntimePlan {
    fn default() -> Self {
        ImageCacheRuntimePlan {
            config: ImageCacheConfig::default(),
            mount_requests: Vec::new(),
            finalizer_actions: Vec::new(),
            copy_plan: None,
            registryd_action: RegistrydAction::None,
        }
    }
}

impl ImageCacheRuntimePlan {
    /// Clear executable copy intent only after the privileged runtime adapter
    /// reports that the copy walk completed successfully.
    ///
    /// Source Talos flips `cacheCopyDone` after `copyImageCache` succeeds, not
    /// when the controller first discovers ready ISO/disk roots. Rust keeps the
    /// pure controller host-safe by emitting copy intent; this helper lets the
    /// runtime effect boundary remove that intent after a real success while
    /// preserving it for host-safe suppression, disabled gates, and no-plan
    /// reports.
    pub fn reproject_after_copy_report(&self, copy_report: &ImageCacheCopyReport) -> Self {
        let mut plan = self.clone();
        if copy_report.status == ImageCacheCopyExecutionStatus::Copied {
            plan.config.copy_status = ImageCacheCopyStatus::Ready;
            plan.copy_plan = None;
        }
        plan
    }

    /// Re-apply the registryd-dependent readiness/action projection after a
    /// runtime adapter observes service state.
    ///
    /// The source controller derives mount roots from block observations first,
    /// then uses registryd `Running && Healthy` to promote a rooted
    /// `ImageCacheConfig` from `Preparing` to `Ready`. Runtime service effects
    /// happen after the boot projection, so this helper intentionally preserves
    /// the existing block/copy/finalizer plans and updates only the fields that
    /// depend on the newly observed registryd state.
    pub fn reproject_after_registryd_observation(&self, registryd: RegistrydState) -> Self {
        let mut plan = self.clone();
        if plan.config.roots.is_empty() || plan.config.status == ImageCacheStatus::Disabled {
            return plan;
        }

        plan.config.status = ImageCacheStatus::Preparing;
        plan.registryd_action = if registryd.running {
            RegistrydAction::None
        } else {
            RegistrydAction::Start
        };
        if registryd.running && registryd.healthy {
            plan.config.status = ImageCacheStatus::Ready;
        }

        plan
    }
}

/// Return `machine.features.imageCache.localEnabled` from a Talos MachineConfig.
///
/// Source `ImageCacheConfigController` treats absent MachineConfig, absent
/// `machine`, absent feature path, or an explicit `false` as disabled. A present
/// non-boolean `localEnabled` is malformed and must not be silently coerced.
pub fn image_cache_local_enabled_from_machine_config_contents(
    contents: &str,
) -> std::result::Result<bool, String> {
    if contents.trim().is_empty() {
        return Ok(false);
    }

    let body = image_cache_v1alpha1_body(contents);
    let doc = os_machine_config_domain::yaml::parse(&body)
        .map_err(|err| format!("machine.features.imageCache.localEnabled: {err}"))?;

    let Some(machine_value) = doc.get("machine") else {
        return Ok(false);
    };
    let machine = machine_value
        .as_mapping()
        .ok_or_else(|| "machine must be a mapping".to_string())?;

    let Some(features_value) = machine.get("features") else {
        return Ok(false);
    };
    let features = features_value
        .as_mapping()
        .ok_or_else(|| "machine.features must be a mapping".to_string())?;

    let Some(image_cache_value) = features.get("imageCache") else {
        return Ok(false);
    };
    let image_cache = image_cache_value
        .as_mapping()
        .ok_or_else(|| "machine.features.imageCache must be a mapping".to_string())?;

    let Some(local_enabled_value) = image_cache.get("localEnabled") else {
        return Ok(false);
    };

    local_enabled_value
        .as_bool()
        .ok_or_else(|| "machine.features.imageCache.localEnabled must be boolean".to_string())
}

fn image_cache_v1alpha1_body(contents: &str) -> Cow<'_, str> {
    match os_machine_config_domain::decode_documents(contents) {
        Ok(documents) => documents
            .iter()
            .find(|document| document.meta.kind == "v1alpha1")
            .map(|document| Cow::Owned(document.body.clone()))
            .unwrap_or(Cow::Borrowed(contents)),
        Err(_) => Cow::Borrowed(contents),
    }
}

/// Source-shaped controller state.
#[derive(Debug, Clone, Default)]
pub struct ImageCacheConfigController {
    cache_copy_done: bool,
}

impl ImageCacheConfigController {
    /// Construct a fresh controller with no remembered copy completion.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the controller has already completed the ISO copy.
    pub fn cache_copy_done(&self) -> bool {
        self.cache_copy_done
    }

    /// Mark the ISO copy complete after a privileged runtime adapter reports
    /// successful copy execution.
    pub fn mark_cache_copy_done(&mut self) {
        self.cache_copy_done = true;
    }

    /// Source-guided, host-safe reconcile.
    pub fn reconcile(&mut self, input: ImageCacheReconcileInput<'_>) -> ImageCacheRuntimePlan {
        if !input.local_enabled {
            return ImageCacheRuntimePlan {
                config: ImageCacheConfig {
                    status: ImageCacheStatus::Disabled,
                    copy_status: ImageCacheCopyStatus::Skipped,
                    roots: Vec::new(),
                },
                ..ImageCacheRuntimePlan::default()
            };
        }

        let mut plan = ImageCacheRuntimePlan::default();
        plan.config.status = ImageCacheStatus::Preparing;

        let cache_volume_status = self.analyze_image_cache_volumes(
            input.volume_statuses,
            input.volume_mount_statuses,
            &mut plan,
        );
        plan.config.roots = cache_volume_status.roots;
        plan.config.copy_status = cache_volume_status.copy_status;
        plan.copy_plan = cache_volume_status.copy_plan;

        if cache_volume_status.all_ready && plan.config.roots.is_empty() {
            plan.config.status = ImageCacheStatus::Disabled;
        }

        if plan.config.status == ImageCacheStatus::Preparing && !plan.config.roots.is_empty() {
            if !input.registryd.running {
                plan.registryd_action = RegistrydAction::Start;
            }
            if input.registryd.running && input.registryd.healthy {
                plan.config.status = ImageCacheStatus::Ready;
                plan.registryd_action = RegistrydAction::None;
            }
        }

        plan
    }

    fn analyze_image_cache_volumes(
        &mut self,
        volume_statuses: &[VolumeStatus],
        volume_mount_statuses: &[VolumeMountStatusResource],
        plan: &mut ImageCacheRuntimePlan,
    ) -> ImageCacheVolumeStatus {
        let iso_present = volume_status(volume_statuses, IMAGE_CACHE_ISO_VOLUME_ID)
            .is_some_and(|status| status.phase == VolumePhase::Ready);
        let disk_missing = volume_status(volume_statuses, IMAGE_CACHE_DISK_VOLUME_ID).is_none();

        let ordered = ordered_image_cache_statuses(volume_statuses);
        for status in &ordered {
            plan.mount_requests
                .push(mount_request_for(status, iso_present));
        }

        let mut roots = Vec::with_capacity(ordered.len());
        let mut all_ready = true;
        let mut iso_ready = false;
        let mut disk_ready = false;
        let mut copy_source = String::new();
        let mut copy_target = String::new();

        for status in ordered {
            let root_result = image_cache_root(status, volume_mount_statuses);
            plan.finalizer_actions.extend(root_result.finalizer_actions);

            if root_result.ready {
                match status.config.id.as_str() {
                    IMAGE_CACHE_ISO_VOLUME_ID => {
                        iso_ready = true;
                        copy_source = root_result.root.clone().unwrap_or_default();
                    }
                    IMAGE_CACHE_DISK_VOLUME_ID => {
                        disk_ready = true;
                        copy_target = root_result.root.clone().unwrap_or_default();
                    }
                    _ => {}
                }
            }

            all_ready = all_ready && root_result.ready;
            if let Some(root) = root_result.root {
                roots.push(root);
            }
        }

        let (copy_status, copy_plan) = match () {
            _ if !iso_present => (ImageCacheCopyStatus::Skipped, None),
            _ if disk_missing => (ImageCacheCopyStatus::Skipped, None),
            _ if self.cache_copy_done => (ImageCacheCopyStatus::Ready, None),
            _ if iso_ready && disk_ready && !copy_source.is_empty() && !copy_target.is_empty() => (
                ImageCacheCopyStatus::Ready,
                Some(ImageCacheCopyPlan {
                    source: copy_source,
                    target: copy_target,
                }),
            ),
            _ => (ImageCacheCopyStatus::Pending, None),
        };

        ImageCacheVolumeStatus {
            roots,
            all_ready,
            copy_status,
            copy_plan,
        }
    }
}

/// COSI runtime adapter for [`ImageCacheConfigController`].
///
/// The pure controller above intentionally emits side-effect plans. This adapter
/// wires those plans into the COSI reconcile loop used by boot/runtime code:
/// it reads block volume status resources, writes the CRI image-cache config and
/// volume mount requests, and uses COSI finalizers on strong mount-status inputs.
/// Copy and service-start effects remain explicit in the plan and are not
/// executed by this host-safe controller.
#[derive(Debug, Clone)]
pub struct ImageCacheCosiController {
    inner: ImageCacheConfigController,
    local_enabled: bool,
    registryd: RegistrydState,
}

impl ImageCacheCosiController {
    /// Build a COSI-runtime adapter with explicit host-observed inputs.
    pub fn new(local_enabled: bool, registryd: RegistrydState) -> Self {
        ImageCacheCosiController {
            inner: ImageCacheConfigController::new(),
            local_enabled,
            registryd,
        }
    }

    /// Borrow the pure source-shaped controller.
    pub fn inner(&self) -> &ImageCacheConfigController {
        &self.inner
    }

    /// Update the host-observed feature flag for the next reconcile.
    pub fn set_local_enabled(&mut self, local_enabled: bool) {
        self.local_enabled = local_enabled;
    }

    /// Return the constructor/setter compatibility feature flag.
    ///
    /// Source-shaped COSI reconciliation derives image-cache enablement from the
    /// active MachineConfig input. This accessor keeps the explicit host flag
    /// observable for callers that still construct the adapter with it.
    pub fn local_enabled_compat(&self) -> bool {
        self.local_enabled
    }

    /// Update the host-observed registryd state for the next reconcile.
    pub fn set_registryd(&mut self, registryd: RegistrydState) {
        self.registryd = registryd;
    }
}

impl CosiController for ImageCacheCosiController {
    fn name(&self) -> &str {
        IMAGE_CACHE_CONTROLLER_NAME
    }

    fn spec(&self) -> Spec {
        Spec::new()
            .with_input(Input::weak(machine_config_kind()).with_id(MACHINE_CONFIG_ACTIVE_ID))
            .with_input(Input::weak(VolumeStatusResource::kind()))
            .with_input(Input::weak(registryd_service_kind()).with_id(REGISTRYD_SERVICE_ID))
            .with_input(Input::strong(VolumeMountStatusResource::kind()))
            .with_input(Input::destroy_ready(VolumeMountRequestResource::kind()))
            .with_output(Output::exclusive(ImageCacheConfigResource::kind()))
            .with_output(Output::shared(volume_config_kind()))
            .with_output(Output::shared(VolumeMountRequestResource::kind()))
    }

    fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult {
        let volume_statuses = collect_volume_statuses(ctx)?;
        let volume_mount_statuses = collect_volume_mount_statuses(ctx)?;
        if image_cache_copy_done_from_context(ctx) {
            self.inner.mark_cache_copy_done();
        }
        let machine_config_contents = crate::cri::machine_config_contents_from_context(ctx)
            .map_err(|err| ControllerError::Failed(format!("error getting config: {err}")))?;
        let local_enabled = machine_config_contents
            .as_deref()
            .map(image_cache_local_enabled_from_machine_config_contents)
            .transpose()
            .map_err(|message| ControllerError::Failed(format!("error getting config: {message}")))?
            .unwrap_or(false);
        let desired_volume_configs = if local_enabled {
            image_cache_volume_configs_from_machine_config_contents(
                machine_config_contents.as_deref().unwrap_or(""),
            )
            .map_err(|message| {
                ControllerError::Failed(format!("error creating volume config: {message}"))
            })?
        } else {
            Vec::new()
        };
        let registryd = registryd_state_from_context(ctx).unwrap_or(self.registryd);
        let plan = self.inner.reconcile(ImageCacheReconcileInput {
            local_enabled,
            registryd,
            volume_statuses: &volume_statuses,
            volume_mount_statuses: &volume_mount_statuses,
        });

        apply_image_cache_plan_to_context(ctx, &plan)?;
        for volume_config in desired_volume_configs {
            upsert_image_cache_volume_config_in_context(ctx, volume_config)?;
        }

        Ok(())
    }
}

/// Source-compatible id for image-cache volume mount requests/statuses.
pub fn image_cache_mount_status_id(volume_id: &str) -> String {
    volume_mount_status_id(IMAGE_CACHE_CONTROLLER_NAME, volume_id)
}

/// Canonical COSI key for the CRI `ImageCacheConfig` singleton.
pub fn image_cache_config_key() -> os_kernel::Result<String> {
    Ok(os_cosi_domain::Metadata::new(
        IMAGE_CACHE_NAMESPACE,
        IMAGE_CACHE_CONFIG_TYPE,
        ResourceId::new(IMAGE_CACHE_CONFIG_ID)?,
    )
    .key())
}

/// Canonical COSI key for the Rust boot bridge's copy completion marker.
pub fn image_cache_copy_state_key() -> os_kernel::Result<String> {
    Ok(os_cosi_domain::Metadata::new(
        IMAGE_CACHE_NAMESPACE,
        IMAGE_CACHE_COPY_STATE_TYPE,
        ResourceId::new(IMAGE_CACHE_COPY_STATE_ID)?,
    )
    .key())
}

/// Canonical COSI key for the registryd v1alpha1 Service input.
pub fn registryd_service_key() -> os_kernel::Result<String> {
    Ok(os_cosi_domain::Metadata::new(
        V1ALPHA1_NAMESPACE,
        V1ALPHA1_SERVICE_TYPE,
        ResourceId::new(REGISTRYD_SERVICE_ID)?,
    )
    .key())
}

/// Canonical COSI key for a block `VolumeConfig` id.
pub fn image_cache_volume_config_key(id: &str) -> os_kernel::Result<String> {
    Ok(os_block_domain::volume_config_key(id)?)
}

/// Source-shaped Image Cache `VolumeConfig` outputs for an active MachineConfig.
///
/// Talos creates both `IMAGECACHE-ISO` and `IMAGECACHE` block `VolumeConfig`
/// resources only after `machine.features.imageCache.localEnabled` enables the
/// controller. The disk volume consumes the optional `VolumeConfig` document
/// named `IMAGECACHE`; when that document is present, missing fields fall back
/// to the source controller's 500 MiB/1 GiB/system_disk/ext4 defaults.
pub fn image_cache_volume_configs_from_machine_config_contents(
    contents: &str,
) -> std::result::Result<Vec<ImageCacheVolumeConfigResource>, String> {
    Ok(vec![
        image_cache_iso_volume_config_resource()?,
        image_cache_disk_volume_config_resource(contents)?,
    ])
}

fn image_cache_iso_volume_config_resource()
-> std::result::Result<ImageCacheVolumeConfigResource, String> {
    let spec = ImageCacheVolumeConfigSpec::new(
        IMAGE_CACHE_ISO_VOLUME_ID,
        SourceBlockVolumeType::Disk,
        "volume.name in [\"iso9660\", \"vfat\"] && volume.label.startsWith(\"TALOS_\")",
    )
    .with_provisioning(ImageCacheVolumeProvisioningSpec::default())
    .with_mount(image_cache_iso_volume_mount_spec());
    ImageCacheVolumeConfigResource::new(spec).map_err(|err| err.to_string())
}

fn image_cache_disk_volume_config_resource(
    contents: &str,
) -> std::result::Result<ImageCacheVolumeConfigResource, String> {
    let mut provisioning = ImageCacheVolumeProvisioningSpec::default();

    if !contents.trim().is_empty() {
        let container =
            os_machine_config_domain::load_from_bytes(contents).map_err(|err| err.to_string())?;
        let volume_configs =
            os_machine_config_domain::volume_configs(&container).map_err(|err| err.to_string())?;
        if let Some(doc) = volume_configs
            .into_iter()
            .find(|doc| doc.name == IMAGE_CACHE_DISK_VOLUME_ID)
        {
            apply_image_cache_disk_volume_config_override(&mut provisioning, &doc);
        }
    }

    let spec = ImageCacheVolumeConfigSpec::new(
        IMAGE_CACHE_DISK_VOLUME_ID,
        SourceBlockVolumeType::Partition,
        "volume.partition_label == \"IMAGECACHE\"",
    )
    .with_provisioning(provisioning)
    .with_mount(image_cache_disk_volume_mount_spec());
    ImageCacheVolumeConfigResource::new(spec).map_err(|err| err.to_string())
}

fn apply_image_cache_disk_volume_config_override(
    provisioning: &mut ImageCacheVolumeProvisioningSpec,
    doc: &os_machine_config_domain::VolumeConfigDoc,
) {
    provisioning.wave = os_block_domain::WAVE_SYSTEM_DISK;
    provisioning.disk_selector = Some("system_disk".to_string());
    provisioning.label = Some(IMAGE_CACHE_DISK_VOLUME_ID.to_string());
    provisioning.min_size = MIN_IMAGE_CACHE_SIZE_BYTES;
    provisioning.max_size = Some(MAX_IMAGE_CACHE_SIZE_BYTES);
    provisioning.relative_max_size = None;
    provisioning.negative_max_size = false;
    provisioning.grow = false;
    provisioning.type_uuid = Some(os_block_domain::layout::type_guid::LINUX_FILESYSTEM.to_string());
    provisioning.filesystem = Some(os_block_domain::FilesystemType::Ext4);

    if let Some(disk_selector) = &doc.provisioning.disk_selector {
        provisioning.disk_selector = Some(disk_selector.clone());
    }
    if let Some(min_size) = doc.provisioning.min_size {
        provisioning.min_size = min_size;
    }
    if let Some(max_size) = doc.provisioning.max_size {
        match max_size {
            os_machine_config_domain::SizeLimit::Absolute(bytes) => {
                provisioning.max_size = Some(bytes);
                provisioning.relative_max_size = None;
                provisioning.negative_max_size = false;
            }
            os_machine_config_domain::SizeLimit::RelativePercent(percent) => {
                provisioning.max_size = None;
                provisioning.relative_max_size = Some(percent);
                provisioning.negative_max_size = false;
            }
            os_machine_config_domain::SizeLimit::NegativeBytes(bytes) => {
                provisioning.max_size = Some(bytes);
                provisioning.relative_max_size = None;
                provisioning.negative_max_size = true;
            }
            os_machine_config_domain::SizeLimit::NegativeRelativePercent(percent) => {
                provisioning.max_size = None;
                provisioning.relative_max_size = Some(percent);
                provisioning.negative_max_size = true;
            }
        }
    }
    if let Some(grow) = doc.provisioning.grow {
        provisioning.grow = grow;
    }
    provisioning.encryption_configured = doc.encryption_configured;
}

/// Whether boot-owned COSI records a successful image-cache copy.
pub fn image_cache_copy_done_from_state(state: &State) -> bool {
    let Ok(key) = image_cache_copy_state_key() else {
        return false;
    };

    state
        .get(&key)
        .and_then(|resource| ImageCacheCopyStateResource::from_resource(resource.as_ref()))
        .is_some_and(|resource| resource.spec.done)
}

/// Record successful image-cache copy completion in boot-owned COSI state.
pub fn record_image_cache_copy_done_in_state(state: &mut State) -> StoreResult<()> {
    let mut desired = ImageCacheCopyStateResource::new(ImageCacheCopyState::done());
    desired
        .metadata_mut()
        .set_owner("imageCacheCopyRuntimeAdapter");
    let key = desired.metadata().key();

    if let Some(existing) = state.get(&key) {
        let expected_version = existing.metadata().version();
        state.update(Box::new(desired), expected_version)?;
    } else {
        state.create(Box::new(desired))?;
    }

    Ok(())
}

/// Persist a successful runtime copy report; preserve state for non-success reports.
pub fn apply_image_cache_copy_report_to_state(
    state: &mut State,
    report: &ImageCacheCopyReport,
) -> StoreResult<()> {
    if report.status == ImageCacheCopyExecutionStatus::Copied {
        record_image_cache_copy_done_in_state(state)?;
    }

    Ok(())
}

/// Apply a host-safe [`ImageCacheRuntimePlan`] to an in-memory COSI state.
///
/// This is the live-resource bridge for the side effects that the pure
/// controller intentionally emits as plans: the CRI config resource,
/// `VolumeMountRequest` resources, and controller finalizers on observed
/// `VolumeMountStatus` resources. It remains host-safe: copy and registryd
/// service actions stay in the returned plan for higher layers to execute.
pub fn apply_image_cache_plan_to_state(
    state: &mut State,
    plan: &ImageCacheRuntimePlan,
) -> StoreResult<()> {
    upsert_image_cache_config(state, plan.config.clone())?;

    let mut desired_request_ids = BTreeSet::new();
    for request in &plan.mount_requests {
        desired_request_ids.insert(request.id.clone());
        upsert_volume_mount_request(state, request)?;
    }
    cleanup_stale_image_cache_mount_requests(state, &desired_request_ids)?;

    for action in &plan.finalizer_actions {
        let key = block_key_to_store(volume_mount_status_key(&action.status_id))?;
        match action.operation {
            ImageCacheFinalizerOperation::Add => {
                state.add_finalizer(&key, IMAGE_CACHE_CONTROLLER_NAME)?;
            }
            ImageCacheFinalizerOperation::Remove => {
                state.remove_finalizer(&key, IMAGE_CACHE_CONTROLLER_NAME)?;
            }
        }
    }

    Ok(())
}

fn collect_volume_statuses(
    ctx: &ReconcileContext<'_>,
) -> Result<Vec<VolumeStatus>, ControllerError> {
    ctx.list(&VolumeStatusResource::kind(), None)
        .into_iter()
        .map(|resource| {
            let key = resource.metadata().key();
            VolumeStatusResource::from_resource(resource.as_ref())
                .map(|resource| resource.status)
                .ok_or_else(|| ControllerError::Failed(format!("failed to decode {key}")))
        })
        .collect()
}

fn collect_volume_mount_statuses(
    ctx: &ReconcileContext<'_>,
) -> Result<Vec<VolumeMountStatusResource>, ControllerError> {
    ctx.list(&VolumeMountStatusResource::kind(), None)
        .into_iter()
        .map(|resource| {
            let key = resource.metadata().key();
            VolumeMountStatusResource::from_resource(resource.as_ref())
                .ok_or_else(|| ControllerError::Failed(format!("failed to decode {key}")))
        })
        .collect()
}

fn image_cache_copy_done_from_context(ctx: &ReconcileContext<'_>) -> bool {
    let Ok(key) = image_cache_copy_state_key() else {
        return false;
    };

    ctx.get(&key)
        .and_then(|resource| ImageCacheCopyStateResource::from_resource(resource.as_ref()))
        .is_some_and(|resource| resource.spec.done)
}

fn registryd_state_from_context(ctx: &ReconcileContext<'_>) -> Option<RegistrydState> {
    let key = registryd_service_key().ok()?;

    ctx.get(&key)
        .and_then(|resource| V1Alpha1ServiceResource::from_resource(resource.as_ref()))
        .map(|resource| resource.spec.into())
}

fn apply_image_cache_plan_to_context(
    ctx: &mut ReconcileContext<'_>,
    plan: &ImageCacheRuntimePlan,
) -> ReconcileResult {
    upsert_image_cache_config_in_context(ctx, plan.config.clone())?;

    let mut desired_request_ids = BTreeSet::new();
    for request in &plan.mount_requests {
        desired_request_ids.insert(request.id.clone());
        upsert_volume_mount_request_in_context(ctx, request)?;
    }
    cleanup_stale_image_cache_mount_requests_in_context(ctx, &desired_request_ids)?;

    for action in &plan.finalizer_actions {
        let key = block_key_to_store(volume_mount_status_key(&action.status_id))?;
        match action.operation {
            ImageCacheFinalizerOperation::Add => ctx.add_finalizer(&key)?,
            ImageCacheFinalizerOperation::Remove => ctx.remove_finalizer(&key)?,
        }
    }

    Ok(())
}

fn image_cache_config_metadata() -> Metadata {
    Metadata::new(
        IMAGE_CACHE_NAMESPACE,
        IMAGE_CACHE_CONFIG_TYPE,
        ResourceId::new(IMAGE_CACHE_CONFIG_ID)
            .expect("Talos image-cache config id is a valid COSI resource id"),
    )
}

fn image_cache_copy_state_metadata() -> Metadata {
    Metadata::new(
        IMAGE_CACHE_NAMESPACE,
        IMAGE_CACHE_COPY_STATE_TYPE,
        ResourceId::new(IMAGE_CACHE_COPY_STATE_ID)
            .expect("Talos image-cache copy state id is a valid COSI resource id"),
    )
}

fn block_key_to_store<T>(result: os_block_domain::Result<T>) -> StoreResult<T> {
    result.map_err(|err| StoreError::NotFound(err.to_string()))
}

fn upsert_image_cache_config_in_context(
    ctx: &mut ReconcileContext<'_>,
    spec: ImageCacheConfig,
) -> StoreResult<()> {
    let mut desired = ImageCacheConfigResource::new(spec);
    desired
        .metadata_mut()
        .set_owner(IMAGE_CACHE_CONTROLLER_NAME);
    let key = desired.metadata().key();

    if let Some(existing) = ctx.get(&key) {
        let mut meta = existing.metadata().clone();
        if meta.owner().is_empty() {
            meta.set_owner(IMAGE_CACHE_CONTROLLER_NAME);
        }
        *desired.metadata_mut() = meta;
        ctx.update(Box::new(desired), existing.metadata().version())?;
    } else {
        ctx.create(Box::new(desired))?;
    }

    Ok(())
}

fn upsert_image_cache_config(state: &mut State, spec: ImageCacheConfig) -> StoreResult<()> {
    let mut desired = ImageCacheConfigResource::new(spec);
    desired
        .metadata_mut()
        .set_owner(IMAGE_CACHE_CONTROLLER_NAME);
    let key = desired.metadata().key();

    if let Some(existing) = state.get(&key) {
        let mut meta = existing.metadata().clone();
        if meta.owner().is_empty() {
            meta.set_owner(IMAGE_CACHE_CONTROLLER_NAME);
        }
        *desired.metadata_mut() = meta;
        let expected_version = existing.metadata().version();
        state.update(Box::new(desired), expected_version)?;
    } else {
        state.create(Box::new(desired))?;
    }

    Ok(())
}

fn upsert_image_cache_volume_config_in_context(
    ctx: &mut ReconcileContext<'_>,
    mut desired: ImageCacheVolumeConfigResource,
) -> StoreResult<()> {
    desired
        .metadata_mut()
        .set_owner(IMAGE_CACHE_CONTROLLER_NAME);
    let key = desired.metadata().key();

    if let Some(existing) = ctx.get(&key) {
        let mut meta = existing.metadata().clone();
        if meta.owner().is_empty() {
            meta.set_owner(IMAGE_CACHE_CONTROLLER_NAME);
        }
        *desired.metadata_mut() = meta;
        ctx.update(Box::new(desired), existing.metadata().version())?;
    } else {
        ctx.create(Box::new(desired))?;
    }

    Ok(())
}

fn upsert_volume_mount_request_in_context(
    ctx: &mut ReconcileContext<'_>,
    plan: &ImageCacheMountRequestPlan,
) -> StoreResult<()> {
    let mut desired = block_key_to_store(VolumeMountRequestResource::new(
        plan.id.clone(),
        VolumeMountRequestSpec::new(&plan.spec.volume_id, IMAGE_CACHE_CONTROLLER_NAME)
            .with_read_only(plan.spec.read_only)
            .with_detached(plan.spec.detached)
            .with_disable_access_time(plan.spec.disable_access_time)
            .with_secure(plan.spec.secure),
    ))?;
    desired
        .metadata_mut()
        .set_owner(IMAGE_CACHE_CONTROLLER_NAME);
    let key = desired.metadata().key();

    if let Some(existing) = ctx.get(&key) {
        let mut meta = existing.metadata().clone();
        if meta.owner().is_empty() {
            meta.set_owner(IMAGE_CACHE_CONTROLLER_NAME);
        }
        *desired.metadata_mut() = meta;
        ctx.update(Box::new(desired), existing.metadata().version())?;
    } else {
        ctx.create(Box::new(desired))?;
    }

    Ok(())
}

fn upsert_volume_mount_request(
    state: &mut State,
    plan: &ImageCacheMountRequestPlan,
) -> StoreResult<()> {
    let mut desired = block_key_to_store(VolumeMountRequestResource::new(
        plan.id.clone(),
        VolumeMountRequestSpec::new(&plan.spec.volume_id, IMAGE_CACHE_CONTROLLER_NAME)
            .with_read_only(plan.spec.read_only)
            .with_detached(plan.spec.detached)
            .with_disable_access_time(plan.spec.disable_access_time)
            .with_secure(plan.spec.secure),
    ))?;
    desired
        .metadata_mut()
        .set_owner(IMAGE_CACHE_CONTROLLER_NAME);
    let key = desired.metadata().key();

    if let Some(existing) = state.get(&key) {
        let mut meta = existing.metadata().clone();
        if meta.owner().is_empty() {
            meta.set_owner(IMAGE_CACHE_CONTROLLER_NAME);
        }
        *desired.metadata_mut() = meta;
        let expected_version = existing.metadata().version();
        state.update(Box::new(desired), expected_version)?;
    } else {
        state.create(Box::new(desired))?;
    }

    Ok(())
}

fn cleanup_stale_image_cache_mount_requests_in_context(
    ctx: &mut ReconcileContext<'_>,
    desired_request_ids: &BTreeSet<String>,
) -> StoreResult<()> {
    for id in [
        image_cache_mount_status_id(IMAGE_CACHE_DISK_VOLUME_ID),
        image_cache_mount_status_id(IMAGE_CACHE_ISO_VOLUME_ID),
    ] {
        if desired_request_ids.contains(&id) {
            continue;
        }

        let key = block_key_to_store(volume_mount_request_key(&id))?;
        let Some(existing) = ctx.get(&key) else {
            continue;
        };
        if existing.metadata().owner() != IMAGE_CACHE_CONTROLLER_NAME {
            continue;
        }

        ctx.teardown(&key)?;
        let Some(tearing_down) = ctx.get(&key) else {
            continue;
        };
        if tearing_down.metadata().can_destroy() {
            ctx.destroy(&key)?;
        }
    }

    Ok(())
}

fn cleanup_stale_image_cache_mount_requests(
    state: &mut State,
    desired_request_ids: &BTreeSet<String>,
) -> StoreResult<()> {
    for id in [
        image_cache_mount_status_id(IMAGE_CACHE_DISK_VOLUME_ID),
        image_cache_mount_status_id(IMAGE_CACHE_ISO_VOLUME_ID),
    ] {
        if desired_request_ids.contains(&id) {
            continue;
        }

        let key = block_key_to_store(volume_mount_request_key(&id))?;
        let Some(existing) = state.get(&key) else {
            continue;
        };
        if existing.metadata().owner() != IMAGE_CACHE_CONTROLLER_NAME {
            continue;
        }

        let version = state.teardown(&key, existing.metadata().version())?;
        let Some(tearing_down) = state.get(&key) else {
            continue;
        };
        if tearing_down.metadata().can_destroy() {
            state.destroy(&key, version)?;
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ImageCacheVolumeStatus {
    roots: Vec<String>,
    all_ready: bool,
    copy_status: ImageCacheCopyStatus,
    copy_plan: Option<ImageCacheCopyPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ImageCacheRootResult {
    root: Option<String>,
    ready: bool,
    finalizer_actions: Vec<ImageCacheFinalizerAction>,
}

fn volume_status<'a>(statuses: &'a [VolumeStatus], id: &str) -> Option<&'a VolumeStatus> {
    statuses.iter().find(|status| status.config.id == id)
}

fn ordered_image_cache_statuses(statuses: &[VolumeStatus]) -> Vec<&VolumeStatus> {
    [IMAGE_CACHE_DISK_VOLUME_ID, IMAGE_CACHE_ISO_VOLUME_ID]
        .into_iter()
        .filter_map(|id| volume_status(statuses, id))
        .collect()
}

fn mount_status<'a>(
    statuses: &'a [VolumeMountStatusResource],
    id: &str,
) -> Option<&'a VolumeMountStatusResource> {
    statuses
        .iter()
        .find(|status| status.metadata().id().as_str() == id)
}

fn mount_request_for(status: &VolumeStatus, iso_present: bool) -> ImageCacheMountRequestPlan {
    let volume_id = status.config.id.as_str();
    let id = image_cache_mount_status_id(volume_id);
    let read_only = !(volume_id == IMAGE_CACHE_DISK_VOLUME_ID && iso_present);
    ImageCacheMountRequestPlan {
        id: id.clone(),
        spec: MountRequestSpec::new(volume_id)
            .with_read_only(read_only)
            .with_requester_id(IMAGE_CACHE_CONTROLLER_NAME, id),
    }
}

fn image_cache_root(
    status: &VolumeStatus,
    mount_statuses: &[VolumeMountStatusResource],
) -> ImageCacheRootResult {
    match status.phase {
        VolumePhase::Waiting | VolumePhase::Failed | VolumePhase::Closed => {
            return ImageCacheRootResult {
                root: None,
                ready: true,
                finalizer_actions: Vec::new(),
            };
        }
        VolumePhase::Ready => {}
        VolumePhase::Located | VolumePhase::Provisioning | VolumePhase::Opening => {
            return ImageCacheRootResult {
                root: None,
                ready: false,
                finalizer_actions: Vec::new(),
            };
        }
    }

    let volume_id = status.config.id.as_str();
    let mount_id = image_cache_mount_status_id(volume_id);
    let Some(mount_status) = mount_status(mount_statuses, &mount_id) else {
        return ImageCacheRootResult {
            root: None,
            ready: false,
            finalizer_actions: Vec::new(),
        };
    };

    if mount_status.metadata().phase().as_str() == "tearingdown" {
        return ImageCacheRootResult {
            root: None,
            ready: true,
            finalizer_actions: vec![ImageCacheFinalizerAction {
                status_id: mount_id,
                operation: ImageCacheFinalizerOperation::Remove,
            }],
        };
    }

    let mut finalizer_actions = Vec::new();
    if !mount_status
        .metadata()
        .finalizers()
        .contains(IMAGE_CACHE_CONTROLLER_NAME)
    {
        finalizer_actions.push(ImageCacheFinalizerAction {
            status_id: mount_id,
            operation: ImageCacheFinalizerOperation::Add,
        });
    }

    let root = if volume_id == IMAGE_CACHE_ISO_VOLUME_ID {
        join_unix_path(&mount_status.spec.target, IMAGE_CACHE_ISO_ROOT_DIR)
    } else {
        mount_status.spec.target.clone()
    };

    ImageCacheRootResult {
        root: Some(root),
        ready: true,
        finalizer_actions,
    }
}

fn join_unix_path(parent: &str, child: &str) -> String {
    if parent.ends_with('/') {
        format!("{parent}{child}")
    } else {
        format!("{parent}/{child}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env, fs,
        path::{Path, PathBuf},
        process::Command,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };
    use os_block_domain::{
        VolumeConfig, VolumeMountStatusResource, VolumeMountStatusSpec, VolumePhase, VolumeStatus,
        mount::volume_mount_request_key,
    };
    use os_cosi_domain::Phase as CosiPhase;

    fn volume(id: &str, phase: VolumePhase) -> VolumeStatus {
        let mut status = VolumeStatus::new(VolumeConfig::partition(id, id, 1));
        status.phase = phase;
        status
    }

    fn mount(volume_id: &str, target: &str, read_only: bool) -> VolumeMountStatusResource {
        VolumeMountStatusResource::new(
            image_cache_mount_status_id(volume_id),
            VolumeMountStatusSpec::new(volume_id, IMAGE_CACHE_CONTROLLER_NAME, target)
                .with_read_only(read_only),
        )
        .unwrap()
    }

    fn ready_registryd() -> RegistrydState {
        RegistrydState {
            running: true,
            healthy: true,
        }
    }

    #[derive(Debug, Clone)]
    struct TestMachineConfigDocument {
        meta: Metadata,
        contents: String,
    }

    impl TestMachineConfigDocument {
        fn new(contents: impl Into<String>) -> Self {
            Self {
                meta: Metadata::new(
                    MACHINE_CONFIG_NAMESPACE,
                    MACHINE_CONFIG_TYPE,
                    ResourceId::new(MACHINE_CONFIG_ACTIVE_ID).unwrap(),
                ),
                contents: contents.into(),
            }
        }
    }

    impl Resource for TestMachineConfigDocument {
        fn metadata(&self) -> &Metadata {
            &self.meta
        }

        fn metadata_mut(&mut self) -> &mut Metadata {
            &mut self.meta
        }

        fn spec_fingerprint(&self) -> String {
            format!("contents={}", test_hex_bytes(self.contents.as_bytes()))
        }

        fn clone_box(&self) -> Box<dyn Resource> {
            Box::new(self.clone())
        }
    }

    fn test_hex_bytes(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    fn enabled_image_cache_machine_config() -> TestMachineConfigDocument {
        TestMachineConfigDocument::new(
            r#"
version: v1alpha1
machine:
  type: worker
  features:
    imageCache:
      localEnabled: true
"#,
        )
    }

    fn copy_plan(source: &Path, target: &Path) -> ImageCacheCopyPlan {
        ImageCacheCopyPlan {
            source: source.display().to_string(),
            target: target.display().to_string(),
        }
    }

    struct TestDir {
        root: PathBuf,
    }

    impl TestDir {
        fn new(label: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = env::temp_dir().join(format!(
                "operating-system-wave84-{label}-{}-{nanos}",
                std::process::id()
            ));
            fs::create_dir_all(&root).unwrap();
            TestDir { root }
        }

        fn path(&self, child: &str) -> PathBuf {
            self.root.join(child)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn registryd_test_last_modified(path: &Path) -> String {
        registryd_source_last_modified_value(&fs::metadata(path).unwrap())
            .expect("source-shaped last modified")
    }

    #[cfg(unix)]
    fn registryd_test_set_file_times(path: &Path, accessed: SystemTime, modified: SystemTime) {
        let accessed = accessed
            .duration_since(UNIX_EPOCH)
            .expect("test access time after epoch")
            .as_secs()
            .to_string();
        let modified = modified
            .duration_since(UNIX_EPOCH)
            .expect("test modified time after epoch")
            .as_secs()
            .to_string();
        let status = Command::new("python3")
            .arg("-c")
            .arg("import os,sys; os.utime(sys.argv[1], (int(sys.argv[2]), int(sys.argv[3])))")
            .arg(path)
            .arg(accessed)
            .arg(modified)
            .status()
            .expect("set file times with python3");

        assert!(status.success());
    }

    #[test]
    fn registryd_http_last_modified_value_matches_http_time_format() {
        assert_eq!(
            registryd_http_last_modified_value(UNIX_EPOCH + Duration::from_secs(784_111_777))
                .as_deref(),
            Some("Sun, 06 Nov 1994 08:49:37 GMT")
        );
        for value in [
            "Sun, 06 Nov 1994 08:49:37 GMT",
            "Sunday, 06-Nov-94 08:49:37 GMT",
            "Sun Nov  6 08:49:37 1994",
        ] {
            assert_eq!(registryd_http_time_unix_seconds(value), Some(784_111_777));
        }
        assert_eq!(registryd_http_time_unix_seconds("not an HTTP time"), None);
        assert_eq!(
            registryd_http_time_unix_seconds("Sun, 31 Nov 1994 08:49:37 GMT"),
            None
        );
        assert_eq!(registryd_http_last_modified_value(UNIX_EPOCH), None);
    }

    fn reconcile(
        controller: &mut ImageCacheConfigController,
        local_enabled: bool,
        registryd: RegistrydState,
        volumes: &[VolumeStatus],
        mounts: &[VolumeMountStatusResource],
    ) -> ImageCacheRuntimePlan {
        controller.reconcile(ImageCacheReconcileInput {
            local_enabled,
            registryd,
            volume_statuses: volumes,
            volume_mount_statuses: mounts,
        })
    }

    #[test]
    fn image_cache_wait_condition_matches_source_disabled_or_ready() {
        for status in [ImageCacheStatus::Unknown, ImageCacheStatus::Preparing] {
            let config = ImageCacheConfig {
                status,
                copy_status: ImageCacheCopyStatus::Skipped,
                roots: Vec::new(),
            };
            assert!(!config.wait_for_image_cache_satisfied());
        }

        for status in [ImageCacheStatus::Disabled, ImageCacheStatus::Ready] {
            let config = ImageCacheConfig {
                status,
                copy_status: ImageCacheCopyStatus::Pending,
                roots: Vec::new(),
            };
            assert!(config.wait_for_image_cache_satisfied());
        }
    }

    #[test]
    fn image_cache_copy_wait_condition_matches_source_skipped_or_ready() {
        for copy_status in [ImageCacheCopyStatus::Unknown, ImageCacheCopyStatus::Pending] {
            let config = ImageCacheConfig {
                status: ImageCacheStatus::Ready,
                copy_status,
                roots: Vec::new(),
            };
            assert!(!config.wait_for_image_cache_copy_satisfied());
        }

        for copy_status in [ImageCacheCopyStatus::Skipped, ImageCacheCopyStatus::Ready] {
            let config = ImageCacheConfig {
                status: ImageCacheStatus::Preparing,
                copy_status,
                roots: Vec::new(),
            };
            assert!(config.wait_for_image_cache_copy_satisfied());
        }
    }

    #[test]
    fn image_cache_wait_for_state_uses_created_and_updated_events() {
        let mut state = State::new();
        assert!(!wait_for_image_cache_in_state(&mut state, 8).unwrap());

        let watch_index = watch_image_cache_config(&mut state, 8);
        state
            .create(Box::new(ImageCacheConfigResource::new(ImageCacheConfig {
                status: ImageCacheStatus::Preparing,
                copy_status: ImageCacheCopyStatus::Skipped,
                roots: Vec::new(),
            })))
            .unwrap();
        assert!(!poll_wait_for_image_cache(&mut state, watch_index).unwrap());

        let existing = state.get(&image_cache_config_key().unwrap()).unwrap();
        let mut ready = ImageCacheConfigResource::new(ImageCacheConfig {
            status: ImageCacheStatus::Ready,
            copy_status: ImageCacheCopyStatus::Skipped,
            roots: vec![IMAGE_CACHE_DISK_MOUNT_POINT.to_string()],
        });
        *ready.metadata_mut() = existing.metadata().clone();
        state
            .update(Box::new(ready), existing.metadata().version())
            .unwrap();

        assert!(poll_wait_for_image_cache(&mut state, watch_index).unwrap());
    }

    #[test]
    fn image_cache_copy_wait_for_state_uses_created_and_updated_events() {
        let mut state = State::new();
        assert!(!wait_for_image_cache_copy_in_state(&mut state, 8).unwrap());

        let watch_index = watch_image_cache_config(&mut state, 8);
        state
            .create(Box::new(ImageCacheConfigResource::new(ImageCacheConfig {
                status: ImageCacheStatus::Ready,
                copy_status: ImageCacheCopyStatus::Pending,
                roots: Vec::new(),
            })))
            .unwrap();
        assert!(!poll_wait_for_image_cache_copy(&mut state, watch_index).unwrap());

        let existing = state.get(&image_cache_config_key().unwrap()).unwrap();
        let mut copied = ImageCacheConfigResource::new(ImageCacheConfig {
            status: ImageCacheStatus::Preparing,
            copy_status: ImageCacheCopyStatus::Ready,
            roots: Vec::new(),
        });
        *copied.metadata_mut() = existing.metadata().clone();
        state
            .update(Box::new(copied), existing.metadata().version())
            .unwrap();

        assert!(poll_wait_for_image_cache_copy(&mut state, watch_index).unwrap());
    }

    #[test]
    fn image_cache_wait_for_state_ignores_bootstrap_and_destroyed_events() {
        let mut state = State::new();
        state
            .create(Box::new(ImageCacheConfigResource::new(ImageCacheConfig {
                status: ImageCacheStatus::Preparing,
                copy_status: ImageCacheCopyStatus::Ready,
                roots: Vec::new(),
            })))
            .unwrap();
        let watch_index = watch_image_cache_config(&mut state, 8);
        assert!(!poll_wait_for_image_cache(&mut state, watch_index).unwrap());

        let existing = state.get(&image_cache_config_key().unwrap()).unwrap();
        state
            .teardown(
                &image_cache_config_key().unwrap(),
                existing.metadata().version(),
            )
            .unwrap();
        let tearing_down = state.get(&image_cache_config_key().unwrap()).unwrap();
        state
            .destroy(
                &image_cache_config_key().unwrap(),
                tearing_down.metadata().version(),
            )
            .unwrap();

        assert!(!poll_wait_for_image_cache(&mut state, watch_index).unwrap());
    }

    #[test]
    fn image_cache_copy_executor_is_disabled_until_explicitly_enabled() {
        let temp = TestDir::new("copy-disabled");
        let source = temp.path("source");
        let target = temp.path("target");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("layer"), b"cached layer").unwrap();
        let plan = copy_plan(&source, &target);

        let report =
            execute_image_cache_copy_plan(Some(&plan), ImageCacheCopyGate::Disabled).unwrap();

        assert_eq!(report.status, ImageCacheCopyExecutionStatus::DisabledByGate);
        assert_eq!(report.files_copied, 0);
        assert_eq!(report.files_skipped, 0);
        assert!(!target.join("layer").exists());
        assert_eq!(
            execute_image_cache_copy_plan(None, ImageCacheCopyGate::Enabled)
                .unwrap()
                .status,
            ImageCacheCopyExecutionStatus::NoPlan
        );
    }

    #[test]
    fn image_cache_copy_executor_copies_directories_and_regular_files() {
        let temp = TestDir::new("copy-tree");
        let source = temp.path("source");
        let target = temp.path("target");
        fs::create_dir_all(source.join("nested")).unwrap();
        fs::write(source.join("manifest.json"), b"{}").unwrap();
        fs::write(source.join("nested").join("layer"), b"cached layer").unwrap();
        let plan = copy_plan(&source, &target);

        let report =
            execute_image_cache_copy_plan(Some(&plan), ImageCacheCopyGate::Enabled).unwrap();

        assert_eq!(report.status, ImageCacheCopyExecutionStatus::Copied);
        assert_eq!(report.files_copied, 2);
        assert_eq!(report.files_skipped, 0);
        assert!(report.directories_created >= 2);
        assert_eq!(report.bytes_copied, 14);
        assert_eq!(fs::read(target.join("manifest.json")).unwrap(), b"{}");
        assert_eq!(
            fs::read(target.join("nested").join("layer")).unwrap(),
            b"cached layer"
        );

        let idempotent =
            execute_image_cache_copy_plan(Some(&plan), ImageCacheCopyGate::Enabled).unwrap();
        assert_eq!(idempotent.files_copied, 0);
        assert_eq!(idempotent.files_skipped, 2);
        assert_eq!(idempotent.directories_created, 0);
        assert_eq!(idempotent.bytes_copied, 14);
    }

    #[test]
    fn image_cache_copy_source_walk_order_matches_talos_walkdir_lexical_order() {
        assert_eq!(
            source_walk_order_for_test(["z-layer", "a-manifest", "m-config"]),
            vec!["a-manifest", "m-config", "z-layer"]
        );
    }

    #[cfg(unix)]
    #[test]
    fn image_cache_copy_executor_creates_directories_with_source_mode() {
        use std::os::unix::fs::PermissionsExt;

        unsafe extern "C" {
            fn umask(mask: u32) -> u32;
        }

        struct UmaskGuard(u32);

        impl UmaskGuard {
            fn set(mask: u32) -> Self {
                let previous = unsafe { umask(mask) };
                UmaskGuard(previous)
            }
        }

        impl Drop for UmaskGuard {
            fn drop(&mut self) {
                unsafe {
                    umask(self.0);
                }
            }
        }

        let temp = TestDir::new("copy-dir-mode");
        let source = temp.path("source");
        let target = temp.path("target");
        fs::create_dir_all(source.join("nested")).unwrap();
        fs::write(source.join("nested").join("layer"), b"cached layer").unwrap();
        let plan = copy_plan(&source, &target);

        let _umask = UmaskGuard::set(0);
        let report =
            execute_image_cache_copy_plan(Some(&plan), ImageCacheCopyGate::Enabled).unwrap();

        assert_eq!(report.status, ImageCacheCopyExecutionStatus::Copied);
        assert_eq!(report.directories_created, 2);
        for directory in [&target, &target.join("nested")] {
            assert_eq!(
                fs::metadata(directory).unwrap().permissions().mode() & 0o777,
                0o755,
                "image-cache copy directories should use Talos MkdirAll(..., 0o755) mode"
            );
        }
    }

    #[test]
    fn image_cache_copy_executor_overwrites_same_size_stale_file() {
        let temp = TestDir::new("copy-stale-same-size");
        let source = temp.path("source");
        let target = temp.path("target");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&target).unwrap();
        fs::write(source.join("layer"), b"freshdata").unwrap();
        fs::write(target.join("layer"), b"staledata").unwrap();
        let plan = copy_plan(&source, &target);

        let report =
            execute_image_cache_copy_plan(Some(&plan), ImageCacheCopyGate::Enabled).unwrap();

        assert_eq!(report.status, ImageCacheCopyExecutionStatus::Copied);
        assert_eq!(report.files_copied, 1);
        assert_eq!(report.files_skipped, 0);
        assert_eq!(report.bytes_copied, 9);
        assert_eq!(fs::read(target.join("layer")).unwrap(), b"freshdata");
    }

    #[cfg(unix)]
    #[test]
    fn image_cache_copy_executor_removes_temp_file_after_rename_error() {
        let temp = TestDir::new("copy-rename-error-cleanup");
        let source = temp.path("source");
        let target = temp.path("target");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(target.join("layer")).unwrap();
        fs::write(source.join("layer"), b"cached layer").unwrap();
        let plan = copy_plan(&source, &target);

        let err = execute_image_cache_copy_plan(Some(&plan), ImageCacheCopyGate::Enabled)
            .expect_err("renaming a temp file over an existing directory must fail");

        assert!(matches!(err, ImageCacheCopyError::CopyFile { .. }));
        assert!(
            !target.join("layer.tmp").exists(),
            "failed atomic rename must not leave a stale image-cache temp file"
        );
    }

    #[cfg(unix)]
    #[test]
    fn image_cache_copy_executor_rejects_unsupported_file_type() {
        let temp = TestDir::new("copy-unsupported");
        let source = temp.path("source");
        let target = temp.path("target");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("layer"), b"cached layer").unwrap();
        std::os::unix::fs::symlink(source.join("layer"), source.join("layer-link")).unwrap();
        let plan = copy_plan(&source, &target);

        let err = execute_image_cache_copy_plan(Some(&plan), ImageCacheCopyGate::Enabled)
            .expect_err("symlinks are not part of the Talos image-cache copy contract");

        assert!(matches!(
            err,
            ImageCacheCopyError::UnsupportedFileType { .. }
        ));
        assert!(!target.join("layer-link").exists());
    }

    fn copy_runtime_plan(source: &Path, target: &Path) -> ImageCacheRuntimePlan {
        ImageCacheRuntimePlan {
            copy_plan: Some(copy_plan(source, target)),
            ..ImageCacheRuntimePlan::default()
        }
    }

    #[test]
    fn image_cache_copy_runtime_adapter_requires_vm_mode_and_enabled_gate() {
        let temp = TestDir::new("copy-runtime-gates");
        let source = temp.path("source");
        let target = temp.path("target");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("layer"), b"cached layer").unwrap();
        let plan = copy_runtime_plan(&source, &target);

        let host_safe = ImageCacheCopyRuntimeAdapter::new(
            ImageCacheCopyRuntimeEnvironment::HostSafe,
            ImageCacheCopyGate::Enabled,
        );
        let host_report = host_safe.execute(&plan).unwrap();
        assert_eq!(
            host_report.status,
            ImageCacheCopyExecutionStatus::DisabledByEnvironment
        );
        assert!(!target.join("layer").exists());

        let vm_disabled = ImageCacheCopyRuntimeAdapter::new(
            ImageCacheCopyRuntimeEnvironment::VmPrivileged,
            ImageCacheCopyGate::Disabled,
        );
        let disabled_report = vm_disabled.execute(&plan).unwrap();
        assert_eq!(
            disabled_report.status,
            ImageCacheCopyExecutionStatus::DisabledByGate
        );
        assert!(!target.join("layer").exists());
    }

    #[test]
    fn image_cache_copy_runtime_adapter_executes_vm_enabled_plan() {
        let temp = TestDir::new("copy-runtime-vm-enabled");
        let source = temp.path("source");
        let target = temp.path("target");
        fs::create_dir_all(source.join("nested")).unwrap();
        fs::write(source.join("manifest.json"), b"{}").unwrap();
        fs::write(source.join("nested").join("layer"), b"cached layer").unwrap();
        let plan = copy_runtime_plan(&source, &target);

        let adapter = ImageCacheCopyRuntimeAdapter::new(
            ImageCacheCopyRuntimeEnvironment::VmPrivileged,
            ImageCacheCopyGate::Enabled,
        );

        let report = adapter.execute(&plan).unwrap();
        assert_eq!(report.status, ImageCacheCopyExecutionStatus::Copied);
        assert_eq!(report.files_copied, 2);
        assert_eq!(report.files_skipped, 0);
        assert_eq!(fs::read(target.join("manifest.json")).unwrap(), b"{}");
        assert_eq!(
            fs::read(target.join("nested").join("layer")).unwrap(),
            b"cached layer"
        );

        let idempotent = adapter.execute(&plan).unwrap();
        assert_eq!(idempotent.status, ImageCacheCopyExecutionStatus::Copied);
        assert_eq!(idempotent.files_copied, 0);
        assert_eq!(idempotent.files_skipped, 2);
    }

    fn registryd_runtime_plan() -> ImageCacheRuntimePlan {
        ImageCacheRuntimePlan {
            config: ImageCacheConfig {
                status: ImageCacheStatus::Preparing,
                copy_status: ImageCacheCopyStatus::Skipped,
                roots: vec![IMAGE_CACHE_DISK_MOUNT_POINT.to_string()],
            },
            registryd_action: RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        }
    }

    #[test]
    fn image_cache_runtime_plan_reprojects_registryd_observation_after_runtime_effects() {
        let plan = registryd_runtime_plan();

        let stopped = plan.reproject_after_registryd_observation(RegistrydState::default());
        assert_eq!(stopped.config.status, ImageCacheStatus::Preparing);
        assert_eq!(stopped.registryd_action, RegistrydAction::Start);
        assert_eq!(stopped.config.roots, plan.config.roots);

        let running_unhealthy = plan.reproject_after_registryd_observation(RegistrydState {
            running: true,
            healthy: false,
        });
        assert_eq!(running_unhealthy.config.status, ImageCacheStatus::Preparing);
        assert_eq!(running_unhealthy.registryd_action, RegistrydAction::None);
        assert_eq!(
            running_unhealthy.config.copy_status,
            plan.config.copy_status
        );

        let ready = plan.reproject_after_registryd_observation(ready_registryd());
        assert_eq!(ready.config.status, ImageCacheStatus::Ready);
        assert_eq!(ready.registryd_action, RegistrydAction::None);
        assert_eq!(ready.config.roots, vec![IMAGE_CACHE_DISK_MOUNT_POINT]);
    }

    #[test]
    fn image_cache_runtime_plan_clears_copy_intent_only_after_successful_copy_report() {
        let copy_plan = ImageCacheCopyPlan {
            source: format!("{IMAGE_CACHE_ISO_MOUNT_POINT}/imagecache"),
            target: IMAGE_CACHE_DISK_MOUNT_POINT.to_string(),
        };
        let plan = ImageCacheRuntimePlan {
            config: ImageCacheConfig {
                status: ImageCacheStatus::Preparing,
                copy_status: ImageCacheCopyStatus::Ready,
                roots: vec![
                    IMAGE_CACHE_DISK_MOUNT_POINT.to_string(),
                    format!("{IMAGE_CACHE_ISO_MOUNT_POINT}/imagecache"),
                ],
            },
            copy_plan: Some(copy_plan.clone()),
            registryd_action: RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };

        let disabled = plan.reproject_after_copy_report(
            &ImageCacheCopyReport::disabled_by_environment(&copy_plan),
        );
        assert_eq!(disabled.copy_plan, plan.copy_plan);
        assert_eq!(disabled.config.copy_status, ImageCacheCopyStatus::Ready);
        assert_eq!(disabled.registryd_action, RegistrydAction::Start);

        let copied = plan.reproject_after_copy_report(&ImageCacheCopyReport::copied(&copy_plan));
        assert_eq!(copied.copy_plan, None);
        assert_eq!(copied.config.copy_status, ImageCacheCopyStatus::Ready);
        assert_eq!(copied.registryd_action, RegistrydAction::Start);
        assert_eq!(copied.config.roots, plan.config.roots);
    }

    #[derive(Debug)]
    struct TestRegistrydServiceManager {
        running_result: std::result::Result<bool, &'static str>,
        queried: Vec<String>,
        loaded: usize,
        loaded_services: Vec<RegistrydRuntimeService>,
        started: Vec<String>,
    }

    impl TestRegistrydServiceManager {
        fn running(running: bool) -> Self {
            TestRegistrydServiceManager {
                running_result: Ok(running),
                queried: Vec::new(),
                loaded: 0,
                loaded_services: Vec::new(),
                started: Vec::new(),
            }
        }

        fn missing() -> Self {
            TestRegistrydServiceManager {
                running_result: Err("service not loaded"),
                queried: Vec::new(),
                loaded: 0,
                loaded_services: Vec::new(),
                started: Vec::new(),
            }
        }
    }

    impl RegistrydServiceManager for TestRegistrydServiceManager {
        fn is_running(
            &mut self,
            service_id: &str,
        ) -> std::result::Result<bool, RegistrydServiceError> {
            self.queried.push(service_id.to_string());
            self.running_result
                .map_err(|message| RegistrydServiceError::IsRunning {
                    service_id: service_id.to_string(),
                    message: message.to_string(),
                })
        }

        fn load_registryd(&mut self, service: RegistrydRuntimeService) {
            self.loaded += 1;
            self.loaded_services.push(service);
        }

        fn start(&mut self, service_id: &str) -> std::result::Result<(), RegistrydServiceError> {
            self.started.push(service_id.to_string());
            Ok(())
        }
    }

    #[test]
    fn registryd_health_endpoint_matches_source_service_contract() {
        assert_eq!(REGISTRYD_SERVICE_ID, "registryd");
        assert_eq!(REGISTRYD_LISTEN_ADDRESS, "127.0.0.1:3172");
        assert_eq!(REGISTRYD_HEALTH_PATH, "/healthz");
        assert_eq!(registryd_health_url(), "http://127.0.0.1:3172/healthz");
        assert_eq!(
            REGISTRYD_HEALTH_URL,
            format!("http://{REGISTRYD_LISTEN_ADDRESS}{REGISTRYD_HEALTH_PATH}")
        );
    }

    #[test]
    fn registryd_health_service_serves_only_get_healthz() {
        let service = RegistrydHealthService::source();
        assert_eq!(service.listen_address(), REGISTRYD_LISTEN_ADDRESS);

        let response = service
            .handle_request("GET", REGISTRYD_HEALTH_PATH)
            .expect("health response");
        assert_eq!(response.status_code, 200);
        assert_eq!(response.reason, "OK");
        assert_eq!(response.status_line(), "HTTP/1.1 200 OK");
        assert!(response.is_success());

        assert!(service.handle_request("GET", "/").is_none());
        assert!(
            service
                .handle_request("POST", REGISTRYD_HEALTH_PATH)
                .is_none()
        );
    }

    #[test]
    fn registryd_health_probe_uses_source_url_and_success_status() {
        let probe = RegistrydHealthProbe::source();

        assert_eq!(probe.url(), REGISTRYD_HEALTH_URL);
        assert_eq!(probe.request_line(), "GET /healthz HTTP/1.1");
        assert!(probe.accepts_status(200));
        assert!(probe.accepts_status(204));
        assert!(!probe.accepts_status(199));
        assert!(!probe.accepts_status(300));
        assert!(!probe.accepts_status(404));
    }

    #[test]
    fn registryd_http_service_matches_source_mux_and_missing_content_statuses() {
        let service = RegistrydHttpService::source();

        for method in ["GET", "HEAD"] {
            assert_eq!(
                service.handle_request(method, "/v2/").unwrap().status_code,
                200
            );
            assert_eq!(
                service
                    .handle_request(method, "/healthz")
                    .unwrap()
                    .status_code,
                200
            );
        }

        assert!(service.handle_request("POST", "/healthz").is_none());

        let missing_namespace = service
            .handle_request("GET", "/v2/alpine/manifests/3.20.3")
            .expect("registry route");
        assert_eq!(missing_namespace.status_code, 400);
        assert_eq!(missing_namespace.reason, "Bad Request");

        let missing_content = service
            .handle_request("GET", "/v2/alpine/manifests/3.20.3?ns=docker.io")
            .expect("registry route");
        assert_eq!(missing_content.status_code, 404);
        assert_eq!(missing_content.reason, "Not Found");

        let unsupported_ref = service
            .handle_request("GET", "/v2/alpine/tags/list?ns=docker.io")
            .expect("registry route");
        assert_eq!(unsupported_ref.status_code, 404);
    }

    #[test]
    fn registryd_http_service_rejects_source_invalid_digest_references_before_lookup() {
        let service = RegistrydHttpService::source();
        let uppercase_sha256 = format!("sha256:{}", "A".repeat(64));
        let unsupported_digest = format!("sha512:{}", "c".repeat(128));

        for target in [
            format!("/v2/library/alpine/blobs/{uppercase_sha256}?ns=docker.io"),
            format!("/v2/library/alpine/manifests/{unsupported_digest}?ns=docker.io"),
        ] {
            let response = service
                .handle_request("GET", &target)
                .expect("registry route");
            assert_eq!(response.status_code, 400);
            assert_eq!(response.reason, "Bad Request");
        }
    }

    #[test]
    fn registryd_http_service_rejects_source_invalid_name_components_before_lookup() {
        let service = RegistrydHttpService::source();

        for target in [
            "/v2/_library/alpine/manifests/latest?ns=docker.io",
            "/v2/library_/alpine/manifests/latest?ns=docker.io",
            "/v2/library/alpha___beta/manifests/latest?ns=docker.io",
            "/v2/library/alpha.-beta/manifests/latest?ns=docker.io",
        ] {
            let response = service
                .handle_request("GET", target)
                .expect("registry route");
            assert_eq!(response.status_code, 400);
            assert_eq!(response.reason, "Bad Request");
        }

        for target in [
            "/v2/library/alpha__beta/manifests/latest?ns=docker.io",
            "/v2/library/alpha---beta/manifests/latest?ns=docker.io",
        ] {
            let response = service
                .handle_request("GET", target)
                .expect("registry route");
            assert_eq!(response.status_code, 404);
            assert_eq!(response.reason, "Not Found");
        }
    }

    #[test]
    fn registryd_http_service_rejects_source_invalid_registry_namespaces_before_lookup() {
        let service = RegistrydHttpService::source();

        for target in [
            "/v2/library/alpine/manifests/latest?ns=../bad",
            "/v2/library/alpine/manifests/latest?ns=bad%20registry",
            "/v2/library/alpine/manifests/latest?ns=bad:port",
            "/v2/library/alpine/manifests/latest?ns=:5000",
            "/v2/library/alpine/manifests/latest?ns=bad_",
            "/v2/pause/manifests/3.9?ns=registry.k8s.io:",
            "/v2/pause/manifests/3.9?ns=registry.k8s.io:443.",
        ] {
            let response = service
                .handle_request("GET", target)
                .expect("registry route");
            assert_eq!(response.status_code, 400);
            assert_eq!(response.reason, "Bad Request");
        }

        let ported_registry = service
            .extract_request("/v2/pause/manifests/3.9?ns=registry.k8s.io:443")
            .expect("source-shaped registry namespace");

        assert_eq!(
            ported_registry.cache_path_plan().expect("tag path"),
            RegistrydCachePathPlan::ManifestTag {
                reference_path: PathBuf::from("manifests/registry.k8s.io_443_/pause/reference/3.9"),
                digest_dir: PathBuf::from("manifests/registry.k8s.io_443_/pause/digest"),
            }
        );
    }

    #[test]
    fn registryd_cache_path_plan_uses_source_parse_docker_ref_namespace_normalization() {
        let service = RegistrydHttpService::source();

        let single_component = service
            .extract_request("/v2/library/alpine/manifests/latest?ns=bad")
            .expect("single component namespace");
        assert_eq!(
            single_component.cache_path_plan().expect("tag path"),
            RegistrydCachePathPlan::ManifestTag {
                reference_path: PathBuf::from(
                    "manifests/docker.io/bad/library/alpine/reference/latest"
                ),
                digest_dir: PathBuf::from("manifests/docker.io/bad/library/alpine/digest"),
            }
        );

        let nested_namespace = service
            .extract_request("/v2/library/alpine/manifests/latest?ns=bad%2Fregistry")
            .expect("query-decoded namespace");
        assert_eq!(
            nested_namespace.cache_path_plan().expect("tag path"),
            RegistrydCachePathPlan::ManifestTag {
                reference_path: PathBuf::from(
                    "manifests/docker.io/bad/registry/library/alpine/reference/latest"
                ),
                digest_dir: PathBuf::from("manifests/docker.io/bad/registry/library/alpine/digest"),
            }
        );

        let response = service
            .handle_request(
                "GET",
                "/v2/library/alpine/manifests/latest?ns=bad%2Fregistry",
            )
            .expect("registry route");
        assert_eq!(response.status_code, 404);
        assert_eq!(response.reason, "Not Found");
    }

    #[test]
    fn registryd_http_service_limits_source_mux_to_get_and_head_routes() {
        let service = RegistrydHttpService::source();

        assert_eq!(
            service.handle_request("GET", "/v2").unwrap().status_code,
            200
        );
        assert_eq!(
            service.handle_request("HEAD", "/v2/").unwrap().status_code,
            200
        );
        assert_eq!(
            service
                .handle_request("HEAD", "/v2/alpine/blobs/sha256:012345?ns=docker.io")
                .unwrap()
                .status_code,
            400
        );

        assert!(service.handle_request("POST", "/v2").is_none());
        assert!(
            service
                .handle_request("POST", "/v2/alpine/manifests/3.20.3?ns=docker.io")
                .is_none()
        );
    }

    #[test]
    fn registryd_http_service_extracts_source_image_reference_params() {
        let request = RegistrydHttpService::source()
            .extract_request("/v2/library/alpine/manifests/3.20.3?ns=docker.io")
            .expect("manifest request");

        assert_eq!(request.registry.as_deref(), Some("docker.io"));
        assert_eq!(request.name, "library/alpine");
        assert_eq!(request.reference, "3.20.3");
        assert_eq!(request.kind, RegistrydApiContentKind::Manifest);
        assert_eq!(
            request.source_reference(),
            "docker.io/library/alpine:3.20.3"
        );

        let blob = RegistrydHttpService::source()
            .extract_request("/v2/library/alpine/blobs/sha256:0123456789abcdef?ns=docker.io")
            .expect("blob request");

        assert_eq!(blob.kind, RegistrydApiContentKind::Blob);
        assert_eq!(
            blob.source_reference(),
            "docker.io/library/alpine@sha256:0123456789abcdef"
        );
    }

    #[test]
    fn registryd_http_service_decodes_source_namespace_query_values() {
        let encoded_port = RegistrydHttpService::source()
            .extract_request("/v2/library/alpine/manifests/latest?ns=registry.k8s.io%3A443")
            .expect("encoded-port namespace request");

        assert_eq!(
            encoded_port.registry.as_deref(),
            Some("registry.k8s.io:443")
        );
        assert_eq!(
            encoded_port
                .cache_path_plan()
                .expect("encoded-port tag path"),
            RegistrydCachePathPlan::ManifestTag {
                reference_path: PathBuf::from(
                    "manifests/registry.k8s.io_443_/library/alpine/reference/latest"
                ),
                digest_dir: PathBuf::from("manifests/registry.k8s.io_443_/library/alpine/digest"),
            }
        );

        let encoded_key = RegistrydHttpService::source()
            .extract_request("/v2/library/alpine/manifests/latest?n%73=docker.io")
            .expect("encoded-key namespace request");

        assert_eq!(encoded_key.registry.as_deref(), Some("docker.io"));

        let malformed_first = RegistrydHttpService::source()
            .extract_request("/v2/library/alpine/manifests/latest?ns=broken%zz&ns=docker.io")
            .expect("malformed-first namespace request");

        assert_eq!(malformed_first.registry.as_deref(), Some("docker.io"));

        let empty_first = RegistrydHttpService::source()
            .extract_request("/v2/library/alpine/manifests/latest?ns&ns=docker.io")
            .expect("empty-first namespace request");

        assert_eq!(empty_first.registry.as_deref(), Some(""));
        assert_eq!(
            RegistrydHttpService::source()
                .handle_request("GET", "/v2/library/alpine/manifests/latest?ns&ns=docker.io")
                .expect("empty-first namespace status")
                .status_code,
            400
        );
    }

    #[test]
    fn registryd_http_service_cleans_source_route_args_before_extracting_params() {
        let repeated_separator = RegistrydHttpService::source()
            .extract_request("/v2/library//alpine/manifests/latest?ns=docker.io")
            .expect("source-cleaned manifest request");

        assert_eq!(repeated_separator.name, "library/alpine");
        assert_eq!(repeated_separator.reference, "latest");
        assert_eq!(
            repeated_separator.cache_path_plan().expect("tag path"),
            RegistrydCachePathPlan::ManifestTag {
                reference_path: PathBuf::from(
                    "manifests/docker.io/library/alpine/reference/latest"
                ),
                digest_dir: PathBuf::from("manifests/docker.io/library/alpine/digest"),
            }
        );

        let dot_segment = RegistrydHttpService::source()
            .extract_request("/v2/library/./alpine/manifests/latest?ns=docker.io")
            .expect("source-cleaned dot-segment manifest request");

        assert_eq!(dot_segment.name, "library/alpine");
    }

    #[test]
    fn registryd_http_service_unescapes_source_route_args_before_cleaning() {
        let encoded_slash = RegistrydHttpService::source()
            .extract_request("/v2/library%2Falpine/manifests/latest?ns=docker.io")
            .expect("encoded slash manifest request");

        assert_eq!(encoded_slash.name, "library/alpine");
        assert_eq!(encoded_slash.reference, "latest");
        assert_eq!(
            encoded_slash.cache_path_plan().expect("tag path"),
            RegistrydCachePathPlan::ManifestTag {
                reference_path: PathBuf::from(
                    "manifests/docker.io/library/alpine/reference/latest"
                ),
                digest_dir: PathBuf::from("manifests/docker.io/library/alpine/digest"),
            }
        );

        let encoded_dot = RegistrydHttpService::source()
            .extract_request("/v2/library/%2e/alpine/manifests/latest?ns=docker.io")
            .expect("encoded dot manifest request");

        assert_eq!(encoded_dot.name, "library/alpine");
    }

    #[test]
    fn registryd_cache_path_plan_matches_source_blob_and_digest_manifest_layout() {
        let digest = format!("sha256:{}", "a".repeat(64));
        let blob = RegistrydHttpService::source()
            .extract_request(&format!("/v2/library/alpine/blobs/{digest}?ns=docker.io"))
            .expect("blob request")
            .cache_path_plan()
            .expect("blob path");
        assert_eq!(
            blob,
            RegistrydCachePathPlan::Blob {
                content_path: PathBuf::from(format!("blob/sha256-{}", "a".repeat(64)))
            }
        );

        let manifest = RegistrydHttpService::source()
            .extract_request(&format!(
                "/v2/library/alpine/manifests/{digest}?ns=registry.k8s.io:443"
            ))
            .expect("manifest request")
            .cache_path_plan()
            .expect("manifest path");
        assert_eq!(
            manifest,
            RegistrydCachePathPlan::ManifestDigest {
                content_path: PathBuf::from(format!(
                    "manifests/registry.k8s.io_443_/library/alpine/digest/sha256-{}",
                    "a".repeat(64)
                ))
            }
        );
    }

    #[test]
    fn registryd_cache_path_plan_matches_source_tag_manifest_reference_layout() {
        let request = RegistrydHttpService::source()
            .extract_request("/v2/library/alpine/manifests/3.20.3?ns=docker.io")
            .expect("tagged manifest request");

        assert_eq!(
            request.cache_path_plan().expect("tag path"),
            RegistrydCachePathPlan::ManifestTag {
                reference_path: PathBuf::from(
                    "manifests/docker.io/library/alpine/reference/3.20.3"
                ),
                digest_dir: PathBuf::from("manifests/docker.io/library/alpine/digest"),
            }
        );
    }

    #[test]
    fn registryd_cache_path_plan_applies_source_docker_official_image_normalization() {
        let docker_hub = RegistrydHttpService::source()
            .extract_request("/v2/alpine/manifests/3.20.3?ns=docker.io")
            .expect("docker official-image request");

        assert_eq!(
            docker_hub.cache_path_plan().expect("docker hub tag path"),
            RegistrydCachePathPlan::ManifestTag {
                reference_path: PathBuf::from(
                    "manifests/docker.io/library/alpine/reference/3.20.3"
                ),
                digest_dir: PathBuf::from("manifests/docker.io/library/alpine/digest"),
            }
        );

        let legacy_hub = RegistrydHttpService::source()
            .extract_request("/v2/alpine/manifests/3.20.3?ns=index.docker.io")
            .expect("legacy docker official-image request");

        assert_eq!(
            legacy_hub
                .cache_path_plan()
                .expect("legacy docker tag path"),
            RegistrydCachePathPlan::ManifestTag {
                reference_path: PathBuf::from(
                    "manifests/docker.io/library/alpine/reference/3.20.3"
                ),
                digest_dir: PathBuf::from("manifests/docker.io/library/alpine/digest"),
            }
        );

        let non_docker = RegistrydHttpService::source()
            .extract_request("/v2/pause/manifests/3.9?ns=registry.k8s.io")
            .expect("non-docker official-image request");

        assert_eq!(
            non_docker.cache_path_plan().expect("non-docker tag path"),
            RegistrydCachePathPlan::ManifestTag {
                reference_path: PathBuf::from("manifests/registry.k8s.io/pause/reference/3.9"),
                digest_dir: PathBuf::from("manifests/registry.k8s.io/pause/digest"),
            }
        );
    }

    #[test]
    fn registryd_cache_path_plan_requires_namespace_and_valid_blob_digest() {
        let missing_namespace = RegistrydHttpService::source()
            .extract_request("/v2/library/alpine/blobs/sha256:0123456789abcdef")
            .expect("blob request");
        assert!(missing_namespace.cache_path_plan().is_err());

        let invalid_digest = RegistrydHttpService::source()
            .extract_request("/v2/library/alpine/blobs/sha256:short?ns=docker.io")
            .expect("blob request");
        assert!(invalid_digest.cache_path_plan().is_err());
    }

    #[test]
    fn registryd_cache_path_plan_rejects_uppercase_sha256_digest_hex() {
        let digest = format!("sha256:{}", "A".repeat(64));
        let uppercase_digest = RegistrydHttpService::source()
            .extract_request(&format!("/v2/library/alpine/blobs/{digest}?ns=docker.io"))
            .expect("blob request");

        assert!(uppercase_digest.cache_path_plan().is_err());
    }

    #[test]
    fn registryd_multipath_fs_resolves_first_existing_cache_root_in_source_order() {
        let digest = format!("sha256:{}", "b".repeat(64));
        let plan = RegistrydHttpService::source()
            .extract_request(&format!("/v2/library/alpine/blobs/{digest}?ns=docker.io"))
            .expect("blob request")
            .cache_path_plan()
            .expect("cache path plan");

        let fs = RegistrydMultiPathFs::new([
            "/system/imagecache/iso",
            "/system/imagecache/disk",
            "/system/imagecache/late",
        ]);
        let wanted = PathBuf::from("/system/imagecache/disk").join(plan.initial_lookup_path());

        assert_eq!(
            fs.resolve_with(plan.initial_lookup_path(), |path| path == wanted)
                .expect("source-shaped lookup"),
            RegistrydMultiPathFsResolution::Found {
                path: wanted,
                attempts: vec![
                    PathBuf::from(
                        "/system/imagecache/iso/blob/sha256-".to_string() + &"b".repeat(64)
                    ),
                    PathBuf::from(
                        "/system/imagecache/disk/blob/sha256-".to_string() + &"b".repeat(64)
                    ),
                ],
            }
        );
    }

    #[test]
    fn registryd_multipath_fs_reports_source_missing_for_no_roots_and_all_misses() {
        let empty = RegistrydMultiPathFs::new(Vec::<PathBuf>::new());
        assert_eq!(
            empty
                .resolve_with("blob/sha256-missing", |_| true)
                .expect("empty root lookup"),
            RegistrydMultiPathFsResolution::Missing { attempts: vec![] }
        );

        let roots = RegistrydMultiPathFs::new(["/cache/iso", "/cache/disk"]);
        assert_eq!(
            roots
                .resolve_with("blob/sha256-missing", |_| false)
                .expect("missing lookup"),
            RegistrydMultiPathFsResolution::Missing {
                attempts: vec![
                    PathBuf::from("/cache/iso/blob/sha256-missing"),
                    PathBuf::from("/cache/disk/blob/sha256-missing"),
                ],
            }
        );
    }

    #[test]
    fn registryd_multipath_fs_absolutizes_relative_roots_without_canonicalizing() {
        let cwd = std::env::current_dir().expect("current dir");
        let fs = RegistrydMultiPathFs::new(["relative-cache"]);
        let expected = cwd.join("relative-cache/blob/sha256-present");

        assert_eq!(
            fs.resolve_with("blob/sha256-present", |path| path == expected)
                .expect("relative root lookup"),
            RegistrydMultiPathFsResolution::Found {
                path: expected.clone(),
                attempts: vec![expected],
            }
        );
    }

    #[test]
    fn registryd_multipath_fs_reads_first_existing_file_bytes_in_source_order() {
        let temp = TestDir::new("registryd-multipath-read");
        let first = temp.path("first");
        let second = temp.path("second");
        fs::create_dir_all(second.join("blob")).unwrap();
        let second_file = second.join("blob/sha256-present");
        fs::write(&second_file, b"from-second").unwrap();
        let second_last_modified = registryd_test_last_modified(&second_file);

        let roots = RegistrydMultiPathFs::new([first.clone(), second.clone()]);

        assert_eq!(
            roots.read_file("blob/sha256-present").expect("read lookup"),
            RegistrydMultiPathFsRead::Found {
                path: second_file,
                bytes: b"from-second".to_vec(),
                modified: Some(second_last_modified),
                attempts: vec![
                    first.join("blob/sha256-present"),
                    second.join("blob/sha256-present"),
                ],
            }
        );

        fs::create_dir_all(first.join("blob")).unwrap();
        let first_file = first.join("blob/sha256-present");
        fs::write(&first_file, b"from-first").unwrap();
        let first_last_modified = registryd_test_last_modified(&first_file);

        assert_eq!(
            roots.read_file("blob/sha256-present").expect("read lookup"),
            RegistrydMultiPathFsRead::Found {
                path: first_file,
                bytes: b"from-first".to_vec(),
                modified: Some(first_last_modified),
                attempts: vec![first.join("blob/sha256-present")],
            }
        );
    }

    #[test]
    #[cfg(unix)]
    fn registryd_multipath_fs_projects_source_access_time_as_last_modified() {
        let temp = TestDir::new("registryd-multipath-atime");
        let root = temp.path("root");
        let content_path = root.join("blob/sha256-present");
        fs::create_dir_all(content_path.parent().unwrap()).unwrap();
        fs::write(&content_path, b"from-root").unwrap();

        let source_accessed = UNIX_EPOCH + Duration::from_secs(784_111_777);
        let non_source_modified = UNIX_EPOCH + Duration::from_secs(946_684_800);
        registryd_test_set_file_times(&content_path, source_accessed, non_source_modified);

        let source_last_modified =
            registryd_http_last_modified_value(source_accessed).expect("source access time");
        let non_source_last_modified =
            registryd_http_last_modified_value(non_source_modified).expect("modified time");
        assert_ne!(source_last_modified, non_source_last_modified);

        let RegistrydMultiPathFsRead::Found { modified, .. } = RegistrydMultiPathFs::new([root])
            .read_file("blob/sha256-present")
            .expect("read lookup")
        else {
            panic!("expected source file");
        };

        assert_eq!(modified.as_deref(), Some(source_last_modified.as_str()));
    }

    #[test]
    fn registryd_multipath_fs_read_reports_no_roots_and_all_misses() {
        let empty = RegistrydMultiPathFs::new(Vec::<PathBuf>::new());
        assert_eq!(
            empty
                .read_file("blob/sha256-missing")
                .expect("empty read lookup"),
            RegistrydMultiPathFsRead::Missing { attempts: vec![] }
        );

        let temp = TestDir::new("registryd-multipath-missing-read");
        let first = temp.path("first");
        let second = temp.path("second");
        let roots = RegistrydMultiPathFs::new([first.clone(), second.clone()]);

        assert_eq!(
            roots
                .read_file("blob/sha256-missing")
                .expect("missing read lookup"),
            RegistrydMultiPathFsRead::Missing {
                attempts: vec![
                    RegistrydMultiPathFsReadAttempt {
                        path: first.join("blob/sha256-missing"),
                        error_kind: io::ErrorKind::NotFound,
                    },
                    RegistrydMultiPathFsReadAttempt {
                        path: second.join("blob/sha256-missing"),
                        error_kind: io::ErrorKind::NotFound,
                    },
                ],
            }
        );
    }

    #[test]
    fn registryd_multipath_fs_read_preserves_existing_source_read_errors() {
        let temp = TestDir::new("registryd-multipath-existing-read-error");
        let root = temp.path("root");
        let content_path = root.join("blob/sha256-directory");
        fs::create_dir_all(&content_path).unwrap();

        let err = RegistrydMultiPathFs::new([root])
            .read_file("blob/sha256-directory")
            .expect_err("existing source read error");

        assert_ne!(err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn registryd_manifest_media_type_uses_explicit_source_field() {
        let shape = RegistrydManifestShape::new()
            .with_media_type("application/vnd.docker.distribution.manifest.v2+json")
            .with_layers();

        assert_eq!(
            registryd_manifest_media_type(&shape).expect("explicit media type"),
            "application/vnd.docker.distribution.manifest.v2+json"
        );
    }

    #[test]
    fn registryd_manifest_media_type_infers_oci_index_from_manifests_field() {
        let shape = RegistrydManifestShape::new().with_manifests();

        assert_eq!(
            registryd_manifest_media_type(&shape).expect("index media type"),
            REGISTRYD_OCI_IMAGE_INDEX_MEDIA_TYPE
        );
    }

    #[test]
    fn registryd_manifest_media_type_infers_oci_manifest_from_layers_or_config() {
        assert_eq!(
            registryd_manifest_media_type(&RegistrydManifestShape::new().with_layers())
                .expect("layers manifest media type"),
            REGISTRYD_OCI_IMAGE_MANIFEST_MEDIA_TYPE
        );
        assert_eq!(
            registryd_manifest_media_type(&RegistrydManifestShape::new().with_config())
                .expect("config manifest media type"),
            REGISTRYD_OCI_IMAGE_MANIFEST_MEDIA_TYPE
        );
    }

    #[test]
    fn registryd_manifest_media_type_rejects_empty_uninferrable_shape() {
        let err = registryd_manifest_media_type(&RegistrydManifestShape::new()).unwrap_err();

        assert!(err.contains("media type is empty and cannot be inferred"));
    }

    #[test]
    fn registryd_manifest_shape_from_json_reads_only_source_top_level_fields() {
        let explicit = RegistrydManifestShape::from_json_bytes(
            br#"{
                "schemaVersion": 2,
                "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
                "layers": [],
                "config": {}
            }"#,
        )
        .expect("explicit manifest shape");

        assert_eq!(
            registryd_manifest_media_type(&explicit).expect("explicit media type"),
            "application/vnd.docker.distribution.manifest.v2+json"
        );

        let inferred = RegistrydManifestShape::from_json_bytes(
            br#"{
                "annotations": {
                    "mediaType": "nested values do not drive source inference",
                    "layers": []
                },
                "manifests": [{"digest": "sha256:abc"}]
            }"#,
        )
        .expect("index manifest shape");

        assert_eq!(
            registryd_manifest_media_type(&inferred).expect("index media type"),
            REGISTRYD_OCI_IMAGE_INDEX_MEDIA_TYPE
        );
    }

    #[test]
    fn registryd_manifest_shape_from_json_tracks_rawmessage_field_presence() {
        let shape =
            RegistrydManifestShape::from_json_bytes(br#"{"mediaType": null, "manifests": null}"#)
                .expect("null raw message field is present");

        assert_eq!(
            registryd_manifest_media_type(&shape).expect("null manifests still present"),
            REGISTRYD_OCI_IMAGE_INDEX_MEDIA_TYPE
        );
    }

    #[test]
    fn registryd_manifest_shape_from_json_rejects_invalid_json_and_media_type_type() {
        assert!(RegistrydManifestShape::from_json_bytes(b"not-json").is_err());
        assert!(RegistrydManifestShape::from_json_bytes(br#"{"mediaType": 5}"#).is_err());
        assert!(RegistrydManifestShape::from_json_bytes(br#"{"mediaType": "ok",}"#).is_err());
    }

    #[test]
    fn registryd_manifest_tag_digest_resolution_matches_source_hash_and_path() {
        let request = RegistrydApiRequest {
            registry: Some("registry.k8s.io".to_string()),
            name: "pause".to_string(),
            reference: "3.9".to_string(),
            kind: RegistrydApiContentKind::Manifest,
        };
        let plan = request.cache_path_plan().expect("tagged manifest plan");

        let resolved = plan
            .resolve_tagged_manifest_digest(b"abc", b"abc")
            .expect("matching tag and digest content");

        assert_eq!(
            resolved.canonical_digest,
            "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            resolved.digest_path,
            PathBuf::from(
                "manifests/registry.k8s.io/pause/digest/sha256-ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
            )
        );
    }

    #[test]
    fn registryd_manifest_tag_digest_resolution_rejects_mismatched_digest_content() {
        let plan = RegistrydCachePathPlan::ManifestTag {
            reference_path: PathBuf::from("manifests/docker.io/library/alpine/reference/latest"),
            digest_dir: PathBuf::from("manifests/docker.io/library/alpine/digest"),
        };

        let err = plan
            .resolve_tagged_manifest_digest(b"abc", b"different content")
            .unwrap_err();

        assert!(err.contains("tagged manifest hash does not match digest file hash"));
    }

    #[test]
    fn registryd_manifest_tag_digest_resolution_requires_tag_plan() {
        let plan = RegistrydCachePathPlan::Blob {
            content_path: PathBuf::from("blob/sha256-present"),
        };

        let err = plan
            .resolve_tagged_manifest_digest(b"abc", b"abc")
            .unwrap_err();

        assert!(err.contains("tagged manifest plan required"));
    }

    #[test]
    fn registryd_manifest_content_response_serves_digest_manifest_get_and_head() {
        let temp = TestDir::new("registryd-manifest-content-digest");
        let root = temp.path("root");
        let digest = format!("sha256:{}", "c".repeat(64));
        let manifest_path = root.join(format!(
            "manifests/docker.io/library/alpine/digest/sha256-{}",
            "c".repeat(64)
        ));
        let manifest =
            br#"{"mediaType":"application/vnd.docker.distribution.manifest.v2+json","layers":[]}"#;
        fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
        fs::write(&manifest_path, manifest).unwrap();
        let manifest_last_modified = registryd_test_last_modified(&manifest_path);

        let roots = RegistrydMultiPathFs::new([root.clone()]);
        let target = format!("/v2/library/alpine/manifests/{digest}?ns=docker.io");

        let get = RegistrydHttpService::source()
            .handle_manifest_request("GET", &target, &roots)
            .expect("manifest route");

        assert_eq!(get.status_code, 200);
        assert_eq!(get.reason, "OK");
        assert_eq!(get.content_length, Some(manifest.len()));
        assert_eq!(get.docker_content_digest.as_deref(), Some(digest.as_str()));
        assert_eq!(
            get.content_type.as_deref(),
            Some("application/vnd.docker.distribution.manifest.v2+json")
        );
        assert_eq!(get.accept_ranges.as_deref(), Some("bytes"));
        assert_eq!(
            get.last_modified.as_deref(),
            Some(manifest_last_modified.as_str())
        );
        assert_eq!(get.body, manifest);
        assert_eq!(get.content_path, Some(manifest_path.clone()));

        let head = RegistrydHttpService::source()
            .handle_manifest_request("HEAD", &target, &roots)
            .expect("manifest route");

        assert_eq!(head.status_code, 200);
        assert_eq!(head.content_length, Some(manifest.len()));
        assert_eq!(head.docker_content_digest.as_deref(), Some(digest.as_str()));
        assert_eq!(head.content_type, get.content_type);
        assert_eq!(head.accept_ranges, None);
        assert_eq!(head.last_modified, None);
        assert!(head.body.is_empty());
        assert_eq!(head.content_path, Some(manifest_path));
    }

    #[test]
    fn registryd_manifest_content_response_serves_tag_manifest_with_canonical_digest() {
        let temp = TestDir::new("registryd-manifest-content-tag");
        let first = temp.path("first");
        let second = temp.path("second");
        let manifest = br#"{"config":{},"layers":[]}"#;
        let canonical_digest = registryd_sha256_digest_string(registryd_sha256(manifest));
        let digest_file = registryd_source_digest_file_name(&canonical_digest).unwrap();

        let reference_path = first.join("manifests/registry.k8s.io/pause/reference/3.9");
        let digest_path = second
            .join("manifests/registry.k8s.io/pause/digest")
            .join(digest_file);
        fs::create_dir_all(reference_path.parent().unwrap()).unwrap();
        fs::create_dir_all(digest_path.parent().unwrap()).unwrap();
        fs::write(&reference_path, manifest).unwrap();
        fs::write(&digest_path, manifest).unwrap();

        let roots = RegistrydMultiPathFs::new([first.clone(), second.clone()]);
        let response = RegistrydHttpService::source()
            .handle_manifest_request("GET", "/v2/pause/manifests/3.9?ns=registry.k8s.io", &roots)
            .expect("manifest route");

        assert_eq!(response.status_code, 200);
        assert_eq!(
            response.docker_content_digest.as_deref(),
            Some(canonical_digest.as_str())
        );
        assert_eq!(
            response.content_type.as_deref(),
            Some(REGISTRYD_OCI_IMAGE_MANIFEST_MEDIA_TYPE)
        );
        assert_eq!(response.content_length, Some(manifest.len()));
        assert_eq!(response.body, manifest);
        assert_eq!(response.content_path, Some(digest_path));
    }

    #[test]
    fn registryd_manifest_content_response_maps_source_error_classes() {
        let temp = TestDir::new("registryd-manifest-content-errors");
        let root = temp.path("root");
        let roots = RegistrydMultiPathFs::new([root.clone()]);

        let missing_namespace = RegistrydHttpService::source()
            .handle_manifest_request("GET", "/v2/library/alpine/manifests/latest", &roots)
            .expect("manifest route");
        assert_eq!(missing_namespace.status_code, 400);
        assert_eq!(missing_namespace.reason, "Bad Request");

        let missing_tag = RegistrydHttpService::source()
            .handle_manifest_request(
                "GET",
                "/v2/library/alpine/manifests/latest?ns=docker.io",
                &roots,
            )
            .expect("manifest route");
        assert_eq!(missing_tag.status_code, 404);
        assert_eq!(missing_tag.reason, "Not Found");

        let reference_path = root.join("manifests/docker.io/library/alpine/reference/latest");
        fs::create_dir_all(reference_path.parent().unwrap()).unwrap();
        fs::write(&reference_path, b"manifest bytes").unwrap();

        let missing_digest = RegistrydHttpService::source()
            .handle_manifest_request(
                "GET",
                "/v2/library/alpine/manifests/latest?ns=docker.io",
                &roots,
            )
            .expect("manifest route");
        assert_eq!(missing_digest.status_code, 500);
        assert_eq!(missing_digest.reason, "Internal Server Error");

        assert!(
            RegistrydHttpService::source()
                .handle_manifest_request(
                    "POST",
                    "/v2/library/alpine/manifests/latest?ns=docker.io",
                    &roots,
                )
                .is_none()
        );
        assert!(
            RegistrydHttpService::source()
                .handle_manifest_request(
                    "GET",
                    "/v2/library/alpine/blobs/sha256:0123456789abcdef?ns=docker.io",
                    &roots,
                )
                .is_none()
        );
    }

    #[test]
    fn registryd_blob_content_response_serves_blob_get_and_head() {
        let temp = TestDir::new("registryd-blob-content");
        let root = temp.path("root");
        let digest = format!("sha256:{}", "d".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "d".repeat(64)));
        let blob = b"compressed layer bytes";
        fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let roots = RegistrydMultiPathFs::new([root.clone()]);
        let target = format!("/v2/library/alpine/blobs/{digest}?ns=docker.io");

        let get = RegistrydHttpService::source()
            .handle_blob_request("GET", &target, &roots)
            .expect("blob route");

        assert_eq!(get.status_code, 200);
        assert_eq!(get.reason, "OK");
        assert_eq!(get.content_length, Some(blob.len()));
        assert_eq!(get.docker_content_digest.as_deref(), Some(digest.as_str()));
        assert_eq!(
            get.content_type.as_deref(),
            Some("text/plain; charset=utf-8")
        );
        assert_eq!(get.accept_ranges.as_deref(), Some("bytes"));
        assert_eq!(
            get.last_modified.as_deref(),
            Some(blob_last_modified.as_str())
        );
        assert_eq!(get.body, blob);
        assert_eq!(get.range_source.as_deref(), Some(blob.as_slice()));
        assert_eq!(get.content_path, Some(blob_path.clone()));

        let head_last_modified = registryd_test_last_modified(&blob_path);
        let head = RegistrydHttpService::source()
            .handle_blob_request("HEAD", &target, &roots)
            .expect("blob route");

        assert_eq!(head.status_code, 200);
        assert_eq!(head.content_length, Some(blob.len()));
        assert_eq!(head.docker_content_digest.as_deref(), Some(digest.as_str()));
        assert_eq!(
            head.content_type.as_deref(),
            Some("text/plain; charset=utf-8")
        );
        assert_eq!(head.accept_ranges.as_deref(), Some("bytes"));
        assert_eq!(
            head.last_modified.as_deref(),
            Some(head_last_modified.as_str())
        );
        assert!(head.body.is_empty());
        assert_eq!(head.range_source.as_deref(), Some(blob.as_slice()));
        assert_eq!(head.content_path, Some(blob_path));
    }

    #[test]
    fn registryd_blob_content_response_resolves_source_tagged_blob_reference() {
        let temp = TestDir::new("registryd-blob-tag-content");
        let root = temp.path("root");
        let manifest = br#"{"config":{},"layers":[]}"#;
        let canonical_digest = registryd_sha256_digest_string(registryd_sha256(manifest));
        let digest_file = registryd_source_digest_file_name(&canonical_digest).unwrap();
        let reference_path = root.join("manifests/docker.io/library/alpine/reference/latest");
        let manifest_digest_path = root
            .join("manifests/docker.io/library/alpine/digest")
            .join(&digest_file);
        let blob_path = root.join(REGISTRYD_BLOB_STORE_DIR).join(&digest_file);
        let blob = b"blob bytes resolved through manifest tag";

        fs::create_dir_all(reference_path.parent().unwrap()).unwrap();
        fs::create_dir_all(manifest_digest_path.parent().unwrap()).unwrap();
        fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        fs::write(&reference_path, manifest).unwrap();
        fs::write(&manifest_digest_path, manifest).unwrap();
        fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let roots = RegistrydMultiPathFs::new([root]);
        let response = RegistrydHttpService::source()
            .handle_cached_content_request(
                "GET",
                "/v2/library/alpine/blobs/latest?ns=docker.io",
                &roots,
            )
            .expect("blob route");

        assert_eq!(response.status_code, 200);
        assert_eq!(response.reason, "OK");
        assert_eq!(response.content_length, Some(blob.len()));
        assert_eq!(
            response.docker_content_digest.as_deref(),
            Some(canonical_digest.as_str())
        );
        assert_eq!(
            response.content_type.as_deref(),
            Some("text/plain; charset=utf-8")
        );
        assert_eq!(response.accept_ranges.as_deref(), Some("bytes"));
        assert_eq!(
            response.last_modified.as_deref(),
            Some(blob_last_modified.as_str())
        );
        assert_eq!(response.body, blob);
        assert_eq!(response.range_source.as_deref(), Some(blob.as_slice()));
        assert_eq!(response.content_path, Some(blob_path));
    }

    #[test]
    fn registryd_blob_content_response_maps_source_tagged_reference_errors() {
        let temp = TestDir::new("registryd-blob-tag-errors");
        let root = temp.path("root");
        let roots = RegistrydMultiPathFs::new([root.clone()]);
        let target = "/v2/library/alpine/blobs/latest?ns=docker.io";

        let missing_reference = RegistrydHttpService::source()
            .handle_cached_content_request("GET", target, &roots)
            .expect("blob tag route");
        assert_eq!(missing_reference.status_code, 404);
        assert_eq!(missing_reference.reason, "Not Found");

        let manifest = br#"{"config":{},"layers":[]}"#;
        let canonical_digest = registryd_sha256_digest_string(registryd_sha256(manifest));
        let digest_file = registryd_source_digest_file_name(&canonical_digest).unwrap();
        let reference_path = root.join("manifests/docker.io/library/alpine/reference/latest");
        let digest_path = root
            .join("manifests/docker.io/library/alpine/digest")
            .join(&digest_file);

        fs::create_dir_all(reference_path.parent().unwrap()).unwrap();
        fs::write(&reference_path, manifest).unwrap();

        let missing_digest = RegistrydHttpService::source()
            .handle_cached_content_request("GET", target, &roots)
            .expect("blob tag route");
        assert_eq!(missing_digest.status_code, 500);
        assert_eq!(missing_digest.reason, "Internal Server Error");

        fs::create_dir_all(digest_path.parent().unwrap()).unwrap();
        fs::write(&digest_path, b"different manifest bytes").unwrap();

        let mismatched_digest = RegistrydHttpService::source()
            .handle_cached_content_request("GET", target, &roots)
            .expect("blob tag route");
        assert_eq!(mismatched_digest.status_code, 500);
        assert_eq!(mismatched_digest.reason, "Internal Server Error");

        fs::write(&digest_path, manifest).unwrap();

        let missing_blob = RegistrydHttpService::source()
            .handle_cached_content_request("GET", target, &roots)
            .expect("blob tag route");
        assert_eq!(missing_blob.status_code, 404);
        assert_eq!(missing_blob.reason, "Not Found");
    }

    #[test]
    fn registryd_blob_content_response_sniffs_source_content_type() {
        assert_eq!(
            registryd_blob_content_type(b"wire response byte range model"),
            "text/plain; charset=utf-8"
        );
        assert_eq!(
            registryd_blob_content_type(&[0x1f, 0x8b, 0x08, 0, 0, 0, 0, 0, 0, 0xff]),
            "application/x-gzip"
        );
        let mut eot_font = [0_u8; 36];
        eot_font[34] = b'L';
        eot_font[35] = b'P';
        assert_eq!(
            registryd_blob_content_type(&eot_font),
            "application/vnd.ms-fontobject"
        );
        assert_eq!(
            registryd_blob_content_type(&[0, 1, 2, 3]),
            "application/octet-stream"
        );
        assert_eq!(
            registryd_blob_content_type(b""),
            "text/plain; charset=utf-8"
        );
        assert_eq!(
            registryd_blob_content_type(br#"{"schemaVersion":2}"#),
            "text/plain; charset=utf-8"
        );

        let temp = TestDir::new("registryd-blob-content-type-sniff");
        let root = temp.path("root");
        let digest = format!("sha256:{}", "c".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "c".repeat(64)));
        let mut blob = [0_u8; 36];
        blob[34] = b'L';
        blob[35] = b'P';
        fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let roots = RegistrydMultiPathFs::new([root]);
        let response = RegistrydHttpService::source()
            .handle_blob_request(
                "GET",
                &format!("/v2/library/alpine/blobs/{digest}?ns=docker.io"),
                &roots,
            )
            .expect("blob route");

        assert_eq!(response.status_code, 200);
        assert_eq!(
            response.content_type.as_deref(),
            Some("application/vnd.ms-fontobject")
        );
        assert_eq!(
            response.source_http_headers(),
            vec![
                ("Content-Length", blob.len().to_string()),
                ("Docker-Content-Digest", digest),
                ("Last-Modified", blob_last_modified),
                ("Content-Type", "application/vnd.ms-fontobject".to_string()),
                ("Accept-Ranges", "bytes".to_string()),
            ]
        );
    }

    #[test]
    fn registryd_content_response_projects_source_http_headers() {
        let temp = TestDir::new("registryd-content-response-headers");
        let root = temp.path("root");
        let roots = RegistrydMultiPathFs::new([root.clone()]);

        let blob_digest = format!("sha256:{}", "7".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "7".repeat(64)));
        let blob = b"header projection blob bytes";
        fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let blob_head = RegistrydHttpService::source()
            .handle_blob_request(
                "HEAD",
                &format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io"),
                &roots,
            )
            .expect("blob route");
        assert!(blob_head.body.is_empty());
        assert_eq!(
            blob_head.source_http_headers(),
            vec![
                ("Content-Length", blob.len().to_string()),
                ("Docker-Content-Digest", blob_digest.clone()),
                ("Last-Modified", blob_last_modified),
                ("Content-Type", "text/plain; charset=utf-8".to_string()),
                ("Accept-Ranges", "bytes".to_string()),
            ]
        );

        let manifest_digest = format!("sha256:{}", "8".repeat(64));
        let manifest_path = root.join(format!(
            "manifests/docker.io/library/alpine/digest/sha256-{}",
            "8".repeat(64)
        ));
        let manifest =
            br#"{"mediaType":"application/vnd.docker.distribution.manifest.v2+json","layers":[]}"#;
        fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
        fs::write(&manifest_path, manifest).unwrap();
        let manifest_last_modified = registryd_test_last_modified(&manifest_path);

        let manifest_get = RegistrydHttpService::source()
            .handle_manifest_request(
                "GET",
                &format!("/v2/library/alpine/manifests/{manifest_digest}?ns=docker.io"),
                &roots,
            )
            .expect("manifest route");
        assert_eq!(
            manifest_get.source_http_headers(),
            vec![
                ("Content-Length", manifest.len().to_string()),
                ("Docker-Content-Digest", manifest_digest),
                ("Last-Modified", manifest_last_modified),
                (
                    "Content-Type",
                    "application/vnd.docker.distribution.manifest.v2+json".to_string(),
                ),
                ("Accept-Ranges", "bytes".to_string()),
            ]
        );

        let health = RegistrydHttpService::source()
            .handle_cached_content_request("GET", "/healthz", &roots)
            .expect("health route");
        assert!(health.source_http_headers().is_empty());
    }

    #[test]
    fn registryd_content_response_projects_source_http_response_bytes() {
        let temp = TestDir::new("registryd-content-response-bytes");
        let root = temp.path("root");
        let digest = format!("sha256:{}", "9".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "9".repeat(64)));
        let blob = b"wire response blob bytes";
        fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let roots = RegistrydMultiPathFs::new([root]);
        let target = format!("/v2/library/alpine/blobs/{digest}?ns=docker.io");
        let get = RegistrydHttpService::source()
            .handle_blob_request("GET", &target, &roots)
            .expect("blob route");

        let bytes = get.source_http_response_bytes();
        let prefix = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nDocker-Content-Digest: {digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert!(bytes.starts_with(prefix.as_bytes()));
        assert_eq!(&bytes[prefix.len()..], blob);

        let head = RegistrydHttpService::source()
            .handle_blob_request("HEAD", &target, &roots)
            .expect("blob route");
        assert_eq!(head.source_http_response_bytes(), prefix.into_bytes());
    }

    #[test]
    fn registryd_content_response_projects_source_if_modified_since_response() {
        let temp = TestDir::new("registryd-content-response-if-modified-since");
        let root = temp.path("root");
        let digest = format!("sha256:{}", "6".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "6".repeat(64)));
        let blob = b"conditional response blob bytes";
        fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let roots = RegistrydMultiPathFs::new([root]);
        let target = format!("/v2/library/alpine/blobs/{digest}?ns=docker.io");
        let get = RegistrydHttpService::source()
            .handle_blob_request("GET", &target, &roots)
            .expect("blob route");

        let not_modified =
            get.source_http_response_bytes_for_request_headers(RegistrydSourceRequestHeaders {
                if_modified_since: Some(&blob_last_modified),
                ..RegistrydSourceRequestHeaders::default()
            });
        let expected = format!(
            "HTTP/1.1 304 Not Modified\r\nDocker-Content-Digest: {digest}\r\nLast-Modified: {blob_last_modified}\r\n\r\n"
        );
        assert_eq!(not_modified.as_slice(), expected.as_bytes());

        let range_with_fresh_validator =
            get.source_http_response_bytes_for_request_headers(RegistrydSourceRequestHeaders {
                range: Some("bytes=0-5"),
                if_modified_since: Some(&blob_last_modified),
                ..RegistrydSourceRequestHeaders::default()
            });
        assert_eq!(range_with_fresh_validator.as_slice(), expected.as_bytes());

        let older_validator = "Sun, 06 Nov 1994 08:49:37 GMT";
        let bytes =
            get.source_http_response_bytes_for_request_headers(RegistrydSourceRequestHeaders {
                if_modified_since: Some(older_validator),
                ..RegistrydSourceRequestHeaders::default()
            });
        let prefix = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nDocker-Content-Digest: {digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert!(bytes.starts_with(prefix.as_bytes()));
        assert_eq!(&bytes[prefix.len()..], blob);

        assert_eq!(
            get.source_http_response_bytes_for_request_headers(RegistrydSourceRequestHeaders {
                if_modified_since: Some("not an HTTP time"),
                ..RegistrydSourceRequestHeaders::default()
            }),
            get.source_http_response_bytes()
        );
    }

    #[test]
    fn registryd_content_response_projects_source_if_none_match_response() {
        let temp = TestDir::new("registryd-content-response-if-none-match");
        let root = temp.path("root");
        let digest = format!("sha256:{}", "3".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "3".repeat(64)));
        let blob = b"etag precondition response blob";
        fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let roots = RegistrydMultiPathFs::new([root]);
        let target = format!("/v2/library/alpine/blobs/{digest}?ns=docker.io");
        let get = RegistrydHttpService::source()
            .handle_blob_request("GET", &target, &roots)
            .expect("blob route");

        let not_modified =
            get.source_http_response_bytes_for_request_headers(RegistrydSourceRequestHeaders {
                if_none_match: Some("*"),
                ..RegistrydSourceRequestHeaders::default()
            });
        let expected = format!(
            "HTTP/1.1 304 Not Modified\r\nDocker-Content-Digest: {digest}\r\nLast-Modified: {blob_last_modified}\r\n\r\n"
        );
        assert_eq!(not_modified.as_slice(), expected.as_bytes());

        let range_with_wildcard =
            get.source_http_response_bytes_for_request_headers(RegistrydSourceRequestHeaders {
                range: Some("bytes=0-5"),
                if_none_match: Some("*"),
                if_modified_since: Some("Sun, 06 Nov 1994 08:49:37 GMT"),
                ..RegistrydSourceRequestHeaders::default()
            });
        assert_eq!(range_with_wildcard.as_slice(), expected.as_bytes());

        let non_matching_etag =
            get.source_http_response_bytes_for_request_headers(RegistrydSourceRequestHeaders {
                if_none_match: Some("\"sha256:test\""),
                if_modified_since: Some(&blob_last_modified),
                ..RegistrydSourceRequestHeaders::default()
            });
        let full_prefix = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nDocker-Content-Digest: {digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert!(non_matching_etag.starts_with(full_prefix.as_bytes()));
        assert_eq!(&non_matching_etag[full_prefix.len()..], blob);
    }

    #[test]
    fn registryd_content_response_projects_source_if_match_response() {
        let temp = TestDir::new("registryd-content-response-if-match");
        let root = temp.path("root");
        let digest = format!("sha256:{}", "2".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "2".repeat(64)));
        let blob = b"if match precondition response blob";
        fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let roots = RegistrydMultiPathFs::new([root]);
        let target = format!("/v2/library/alpine/blobs/{digest}?ns=docker.io");
        let get = RegistrydHttpService::source()
            .handle_blob_request("GET", &target, &roots)
            .expect("blob route");

        let full_prefix = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nDocker-Content-Digest: {digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        let wildcard_with_stale_date =
            get.source_http_response_bytes_for_request_headers(RegistrydSourceRequestHeaders {
                if_match: Some("*"),
                if_unmodified_since: Some("Sun, 06 Nov 1994 08:49:37 GMT"),
                ..RegistrydSourceRequestHeaders::default()
            });
        assert!(wildcard_with_stale_date.starts_with(full_prefix.as_bytes()));
        assert_eq!(&wildcard_with_stale_date[full_prefix.len()..], blob);

        let expected_precondition = format!(
            "HTTP/1.1 412 Precondition Failed\r\nDocker-Content-Digest: {digest}\r\nLast-Modified: {blob_last_modified}\r\n\r\n"
        );
        let quoted_without_http_etag =
            get.source_http_response_bytes_for_request_headers(RegistrydSourceRequestHeaders {
                if_match: Some("\"sha256:test\""),
                if_unmodified_since: Some(&blob_last_modified),
                ..RegistrydSourceRequestHeaders::default()
            });
        assert_eq!(
            quoted_without_http_etag.as_slice(),
            expected_precondition.as_bytes()
        );

        let quoted_with_none_match =
            get.source_http_response_bytes_for_request_headers(RegistrydSourceRequestHeaders {
                if_match: Some("\"sha256:test\""),
                if_none_match: Some("*"),
                ..RegistrydSourceRequestHeaders::default()
            });
        assert_eq!(
            quoted_with_none_match.as_slice(),
            expected_precondition.as_bytes()
        );

        let invalid_token =
            get.source_http_response_bytes_for_request_headers(RegistrydSourceRequestHeaders {
                if_match: Some("not-an-etag"),
                ..RegistrydSourceRequestHeaders::default()
            });
        assert_eq!(invalid_token.as_slice(), expected_precondition.as_bytes());
    }

    #[test]
    fn registryd_content_response_projects_source_if_unmodified_since_response() {
        let temp = TestDir::new("registryd-content-response-if-unmodified-since");
        let root = temp.path("root");
        let digest = format!("sha256:{}", "4".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "4".repeat(64)));
        let blob = b"precondition response blob";
        fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let roots = RegistrydMultiPathFs::new([root]);
        let target = format!("/v2/library/alpine/blobs/{digest}?ns=docker.io");
        let get = RegistrydHttpService::source()
            .handle_blob_request("GET", &target, &roots)
            .expect("blob route");

        let stale_precondition =
            get.source_http_response_bytes_for_request_headers(RegistrydSourceRequestHeaders {
                if_unmodified_since: Some("Sun, 06 Nov 1994 08:49:37 GMT"),
                ..RegistrydSourceRequestHeaders::default()
            });
        let expected_precondition = format!(
            "HTTP/1.1 412 Precondition Failed\r\nDocker-Content-Digest: {digest}\r\nLast-Modified: {blob_last_modified}\r\n\r\n"
        );
        assert_eq!(
            stale_precondition.as_slice(),
            expected_precondition.as_bytes()
        );

        let range_with_stale_precondition =
            get.source_http_response_bytes_for_request_headers(RegistrydSourceRequestHeaders {
                range: Some("bytes=0-5"),
                if_unmodified_since: Some("Sun, 06 Nov 1994 08:49:37 GMT"),
                ..RegistrydSourceRequestHeaders::default()
            });
        assert_eq!(
            range_with_stale_precondition.as_slice(),
            expected_precondition.as_bytes()
        );

        let fresh_precondition =
            get.source_http_response_bytes_for_request_headers(RegistrydSourceRequestHeaders {
                if_unmodified_since: Some(&blob_last_modified),
                ..RegistrydSourceRequestHeaders::default()
            });
        let full_prefix = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nDocker-Content-Digest: {digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert!(fresh_precondition.starts_with(full_prefix.as_bytes()));
        assert_eq!(&fresh_precondition[full_prefix.len()..], blob);

        assert_eq!(
            get.source_http_response_bytes_for_request_headers(RegistrydSourceRequestHeaders {
                if_unmodified_since: Some("not an HTTP time"),
                ..RegistrydSourceRequestHeaders::default()
            }),
            get.source_http_response_bytes()
        );
    }

    #[test]
    fn registryd_content_response_projects_source_if_range_response() {
        let temp = TestDir::new("registryd-content-response-if-range");
        let root = temp.path("root");
        let digest = format!("sha256:{}", "5".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "5".repeat(64)));
        let blob = b"conditional range response blob";
        fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let roots = RegistrydMultiPathFs::new([root]);
        let target = format!("/v2/library/alpine/blobs/{digest}?ns=docker.io");
        let get = RegistrydHttpService::source()
            .handle_blob_request("GET", &target, &roots)
            .expect("blob route");

        let equal_if_range =
            get.source_http_response_bytes_for_request_headers(RegistrydSourceRequestHeaders {
                range: Some("bytes=5-12"),
                if_range: Some(&blob_last_modified),
                ..RegistrydSourceRequestHeaders::default()
            });
        let partial_prefix = format!(
            "HTTP/1.1 206 Partial Content\r\nContent-Length: 8\r\nDocker-Content-Digest: {digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Range: bytes 5-12/{}\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert!(equal_if_range.starts_with(partial_prefix.as_bytes()));
        assert_eq!(&equal_if_range[partial_prefix.len()..], &blob[5..=12]);

        let older_if_range =
            get.source_http_response_bytes_for_request_headers(RegistrydSourceRequestHeaders {
                range: Some("bytes=5-12"),
                if_range: Some("Sun, 06 Nov 1994 08:49:37 GMT"),
                ..RegistrydSourceRequestHeaders::default()
            });
        let full_prefix = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nDocker-Content-Digest: {digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert!(older_if_range.starts_with(full_prefix.as_bytes()));
        assert_eq!(&older_if_range[full_prefix.len()..], blob);

        assert_eq!(
            get.source_http_response_bytes_for_request_headers(RegistrydSourceRequestHeaders {
                range: Some("bytes=5-12"),
                if_range: Some("not an HTTP time"),
                ..RegistrydSourceRequestHeaders::default()
            }),
            get.source_http_response_bytes()
        );
    }

    #[test]
    fn registryd_content_response_projects_source_single_byte_range_response() {
        let temp = TestDir::new("registryd-content-response-range");
        let root = temp.path("root");
        let digest = format!("sha256:{}", "8".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "8".repeat(64)));
        let blob = b"wire response byte range model";
        fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let roots = RegistrydMultiPathFs::new([root]);
        let target = format!("/v2/library/alpine/blobs/{digest}?ns=docker.io");
        let get = RegistrydHttpService::source()
            .handle_blob_request("GET", &target, &roots)
            .expect("blob route");

        let bytes = get.source_http_response_bytes_for_range(Some("bytes=5-12"));
        let prefix = format!(
            "HTTP/1.1 206 Partial Content\r\nContent-Length: 8\r\nDocker-Content-Digest: {digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Range: bytes 5-12/{}\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert!(bytes.starts_with(prefix.as_bytes()));
        assert_eq!(&bytes[prefix.len()..], &blob[5..=12]);

        let bytes = get.source_http_response_bytes_for_range(Some("bytes=5-"));
        let prefix = format!(
            "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nDocker-Content-Digest: {digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Range: bytes 5-{}/{}\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len() - 5,
            blob.len() - 1,
            blob.len()
        );
        assert!(bytes.starts_with(prefix.as_bytes()));
        assert_eq!(&bytes[prefix.len()..], &blob[5..]);

        let head = RegistrydHttpService::source()
            .handle_blob_request("HEAD", &target, &roots)
            .expect("blob route");
        let head_last_modified = head
            .last_modified
            .as_deref()
            .expect("HEAD response carries source last-modified");
        let bytes = head.source_http_response_bytes_for_range(Some("bytes=5-12"));
        let prefix = format!(
            "HTTP/1.1 206 Partial Content\r\nContent-Length: 8\r\nDocker-Content-Digest: {digest}\r\nLast-Modified: {head_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Range: bytes 5-12/{}\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert_eq!(bytes, prefix.into_bytes());

        let bytes = get.source_http_response_bytes_for_range(Some("bytes=-6"));
        let suffix_start = blob.len() - 6;
        let prefix = format!(
            "HTTP/1.1 206 Partial Content\r\nContent-Length: 6\r\nDocker-Content-Digest: {digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Range: bytes {suffix_start}-{}/{}\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len() - 1,
            blob.len()
        );
        assert!(bytes.starts_with(prefix.as_bytes()));
        assert_eq!(&bytes[prefix.len()..], &blob[suffix_start..]);

        let bytes = head.source_http_response_bytes_for_range(Some("bytes=-6"));
        let prefix = format!(
            "HTTP/1.1 206 Partial Content\r\nContent-Length: 6\r\nDocker-Content-Digest: {digest}\r\nLast-Modified: {head_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Range: bytes {suffix_start}-{}/{}\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len() - 1,
            blob.len()
        );
        assert_eq!(bytes, prefix.into_bytes());

        let bytes = get.source_http_response_bytes_for_range(Some("bytes=-0"));
        let prefix = format!(
            "HTTP/1.1 206 Partial Content\r\nContent-Length: 0\r\nDocker-Content-Digest: {digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Range: bytes {}-{}/{}\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len(),
            blob.len() - 1,
            blob.len()
        );
        assert_eq!(bytes, prefix.into_bytes());

        let bytes = get.source_http_response_bytes_for_range(Some("bytes=999-1000"));
        let prefix = format!(
            "HTTP/1.1 416 Requested Range Not Satisfiable\r\nContent-Length: 33\r\nDocker-Content-Digest: {digest}\r\nContent-Type: text/plain; charset=utf-8\r\nX-Content-Type-Options: nosniff\r\nContent-Range: bytes */{}\r\n\r\n",
            blob.len()
        );
        assert!(bytes.starts_with(prefix.as_bytes()));
        assert_eq!(
            &bytes[prefix.len()..],
            b"invalid range: failed to overlap\n"
        );

        let bytes = head.source_http_response_bytes_for_range(Some("bytes=999-1000"));
        let prefix = format!(
            "HTTP/1.1 416 Requested Range Not Satisfiable\r\nContent-Length: 33\r\nDocker-Content-Digest: {digest}\r\nContent-Type: text/plain; charset=utf-8\r\nX-Content-Type-Options: nosniff\r\nContent-Range: bytes */{}\r\n\r\n",
            blob.len()
        );
        assert_eq!(bytes, prefix.into_bytes());

        let bytes = get.source_http_response_bytes_for_range(Some("bytes=5-4"));
        let prefix = format!(
            "HTTP/1.1 416 Requested Range Not Satisfiable\r\nContent-Length: 14\r\nDocker-Content-Digest: {digest}\r\nContent-Type: text/plain; charset=utf-8\r\nX-Content-Type-Options: nosniff\r\n\r\n"
        );
        assert!(bytes.starts_with(prefix.as_bytes()));
        assert_eq!(&bytes[prefix.len()..], b"invalid range\n");

        let bytes = head.source_http_response_bytes_for_range(Some("bytes=5-4"));
        assert_eq!(bytes, prefix.clone().into_bytes());

        let bytes = get.source_http_response_bytes_for_range(Some("items=0-1"));
        assert!(bytes.starts_with(prefix.as_bytes()));
        assert_eq!(&bytes[prefix.len()..], b"invalid range\n");

        let bytes = head.source_http_response_bytes_for_range(Some("items=0-1"));
        assert_eq!(bytes, prefix.into_bytes());
    }

    #[test]
    fn registryd_content_response_projects_source_multipart_range_response() {
        let temp = TestDir::new("registryd-content-response-multipart-range");
        let root = temp.path("root");
        let digest = format!("sha256:{}", "7".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "7".repeat(64)));
        let blob = b"multipart range response bytes";
        fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let roots = RegistrydMultiPathFs::new([root]);
        let target = format!("/v2/library/alpine/blobs/{digest}?ns=docker.io");
        let get = RegistrydHttpService::source()
            .handle_blob_request("GET", &target, &roots)
            .expect("blob route");

        let bytes = get.source_http_response_bytes_for_range(Some("bytes=0-3,10-13"));
        let response = String::from_utf8(bytes).expect("registryd response is utf-8");
        let (headers, body) = response
            .split_once("\r\n\r\n")
            .expect("response has header delimiter");
        assert!(headers.starts_with("HTTP/1.1 206 Partial Content\r\n"));
        assert!(headers.contains(&format!("Docker-Content-Digest: {digest}\r\n")));
        assert!(headers.contains(&format!("Last-Modified: {blob_last_modified}\r\n")));
        assert!(headers.lines().any(|line| line == "Accept-Ranges: bytes"));
        assert!(!headers.contains("\r\nContent-Range: "));
        let boundary = headers
            .lines()
            .find_map(|line| line.strip_prefix("Content-Type: multipart/byteranges; boundary="))
            .expect("multipart boundary header");
        let content_length = headers
            .lines()
            .find_map(|line| line.strip_prefix("Content-Length: "))
            .expect("multipart content length")
            .parse::<usize>()
            .expect("content length is numeric");
        assert_eq!(content_length, body.len());
        assert!(body.contains(&format!(
            "--{boundary}\r\nContent-Range: bytes 0-3/{}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nmult\r\n",
            blob.len()
        )));
        assert!(body.contains(&format!(
            "--{boundary}\r\nContent-Range: bytes 10-13/{}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nrang\r\n",
            blob.len()
        )));
        assert!(body.ends_with(&format!("--{boundary}--\r\n")));

        let bytes = get.source_http_response_bytes_for_range(Some("bytes=999-1000,0-3"));
        let prefix = format!(
            "HTTP/1.1 206 Partial Content\r\nContent-Length: 4\r\nDocker-Content-Digest: {digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Range: bytes 0-3/{}\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert!(bytes.starts_with(prefix.as_bytes()));
        assert_eq!(&bytes[prefix.len()..], &blob[0..4]);

        let bytes = get.source_http_response_bytes_for_range(Some("bytes=0-30,0-30"));
        assert_eq!(bytes, get.source_http_response_bytes());

        let head = RegistrydHttpService::source()
            .handle_blob_request("HEAD", &target, &roots)
            .expect("blob route");
        let bytes = head.source_http_response_bytes_for_range(Some("bytes=0-3,10-13"));
        let response = String::from_utf8(bytes).expect("registryd head response is utf-8");
        let (headers, body) = response
            .split_once("\r\n\r\n")
            .expect("response has header delimiter");
        assert!(headers.starts_with("HTTP/1.1 206 Partial Content\r\n"));
        assert!(headers.contains("Content-Type: multipart/byteranges; boundary="));
        assert!(headers.contains("Content-Length: "));
        assert!(body.is_empty());
    }

    #[test]
    fn registryd_blob_content_response_maps_source_error_classes() {
        let temp = TestDir::new("registryd-blob-content-errors");
        let root = temp.path("root");
        let roots = RegistrydMultiPathFs::new([root]);
        let digest = format!("sha256:{}", "e".repeat(64));

        let missing_namespace = RegistrydHttpService::source()
            .handle_blob_request("GET", &format!("/v2/library/alpine/blobs/{digest}"), &roots)
            .expect("blob route");
        assert_eq!(missing_namespace.status_code, 400);
        assert_eq!(missing_namespace.reason, "Bad Request");

        let invalid_digest = RegistrydHttpService::source()
            .handle_blob_request(
                "GET",
                "/v2/library/alpine/blobs/sha256:short?ns=docker.io",
                &roots,
            )
            .expect("blob route");
        assert_eq!(invalid_digest.status_code, 400);
        assert_eq!(invalid_digest.reason, "Bad Request");

        let missing_blob = RegistrydHttpService::source()
            .handle_blob_request(
                "GET",
                &format!("/v2/library/alpine/blobs/{digest}?ns=docker.io"),
                &roots,
            )
            .expect("blob route");
        assert_eq!(missing_blob.status_code, 404);
        assert_eq!(missing_blob.reason, "Not Found");

        assert!(
            RegistrydHttpService::source()
                .handle_blob_request(
                    "POST",
                    &format!("/v2/library/alpine/blobs/{digest}?ns=docker.io"),
                    &roots,
                )
                .is_none()
        );
        assert!(
            RegistrydHttpService::source()
                .handle_blob_request(
                    "GET",
                    "/v2/library/alpine/manifests/latest?ns=docker.io",
                    &roots,
                )
                .is_none()
        );
    }

    #[test]
    fn registryd_cached_content_dispatcher_serves_manifest_and_blob_routes() {
        let temp = TestDir::new("registryd-cached-content-dispatch");
        let root = temp.path("root");

        let manifest_digest = format!("sha256:{}", "f".repeat(64));
        let manifest_path = root.join(format!(
            "manifests/docker.io/library/alpine/digest/sha256-{}",
            "f".repeat(64)
        ));
        let manifest =
            br#"{"mediaType":"application/vnd.docker.distribution.manifest.v2+json","layers":[]}"#;
        fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
        fs::write(&manifest_path, manifest).unwrap();
        let manifest_last_modified = registryd_test_last_modified(&manifest_path);

        let blob_digest = format!("sha256:{}", "1".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "1".repeat(64)));
        let blob = b"layer bytes from cache";
        fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let roots = RegistrydMultiPathFs::new([root.clone()]);
        let service = RegistrydHttpService::source();

        let manifest = service
            .handle_cached_content_request(
                "GET",
                &format!("/v2/library/alpine/manifests/{manifest_digest}?ns=docker.io"),
                &roots,
            )
            .expect("manifest route");

        assert_eq!(manifest.status_code, 200);
        assert_eq!(
            manifest.docker_content_digest.as_deref(),
            Some(manifest_digest.as_str())
        );
        assert_eq!(
            manifest.content_type.as_deref(),
            Some("application/vnd.docker.distribution.manifest.v2+json")
        );
        assert_eq!(
            manifest.last_modified.as_deref(),
            Some(manifest_last_modified.as_str())
        );
        assert_eq!(manifest.content_path, Some(manifest_path));

        let blob = service
            .handle_cached_content_request(
                "HEAD",
                &format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io"),
                &roots,
            )
            .expect("blob route");

        assert_eq!(blob.status_code, 200);
        assert_eq!(
            blob.docker_content_digest.as_deref(),
            Some(blob_digest.as_str())
        );
        assert_eq!(
            blob.content_type.as_deref(),
            Some("text/plain; charset=utf-8")
        );
        assert_eq!(
            blob.last_modified.as_deref(),
            Some(blob_last_modified.as_str())
        );
        assert_eq!(blob.content_length, Some(b"layer bytes from cache".len()));
        assert!(blob.body.is_empty());
        assert_eq!(blob.content_path, Some(blob_path));
    }

    #[test]
    fn registryd_cached_content_dispatcher_preserves_source_mux_statuses() {
        let service = RegistrydHttpService::source();
        let temp = TestDir::new("registryd-cached-mux");
        let root = temp.path("root");
        fs::create_dir_all(root.join(REGISTRYD_MANIFEST_STORE_DIR)).unwrap();
        let roots = RegistrydMultiPathFs::new([root]);

        for target in ["/v2", "/v2/", "/healthz", "/healthz/"] {
            let response = service
                .handle_cached_content_request("GET", target, &roots)
                .expect("source simple route");
            assert_eq!(response.status_code, 200);
            assert_eq!(response.reason, "OK");
            assert_eq!(response.content_length, None);
            assert_eq!(response.docker_content_digest, None);
            assert!(response.body.is_empty());
        }

        let missing_namespace = service
            .handle_cached_content_request("GET", "/v2/library/alpine/manifests/latest", &roots)
            .expect("registry route");
        assert_eq!(missing_namespace.status_code, 400);
        assert_eq!(missing_namespace.reason, "Bad Request");

        let unsupported = service
            .handle_cached_content_request(
                "GET",
                "/v2/library/alpine/tags/list?ns=docker.io",
                &roots,
            )
            .expect("source mux route");
        assert_eq!(unsupported.status_code, 404);
        assert_eq!(unsupported.reason, "Not Found");

        assert!(
            service
                .handle_cached_content_request(
                    "POST",
                    "/v2/library/alpine/manifests/latest",
                    &roots
                )
                .is_none()
        );
        assert!(
            service
                .handle_cached_content_request("GET", "/", &roots)
                .is_none()
        );
    }

    #[test]
    fn registryd_cached_content_dispatcher_infers_missing_namespace_from_manifest_roots() {
        let temp = TestDir::new("registryd-cached-infer-ns");
        let root = temp.path("root");
        let manifest = br#"{"config":{},"layers":[]}"#;
        let canonical_digest = registryd_sha256_digest_string(registryd_sha256(manifest));
        let digest_file = registryd_source_digest_file_name(&canonical_digest).unwrap();

        let reference_path = root.join("manifests/docker.io/library/alpine/reference/latest");
        let digest_path = root
            .join("manifests/docker.io/library/alpine/digest")
            .join(digest_file);
        fs::create_dir_all(reference_path.parent().unwrap()).unwrap();
        fs::create_dir_all(digest_path.parent().unwrap()).unwrap();
        fs::write(&reference_path, manifest).unwrap();
        fs::write(&digest_path, manifest).unwrap();

        let roots = RegistrydMultiPathFs::new([root]);
        let response = RegistrydHttpService::source()
            .handle_cached_content_request("GET", "/v2/library/alpine/manifests/latest", &roots)
            .expect("manifest route");

        assert_eq!(response.status_code, 200);
        assert_eq!(
            response.docker_content_digest.as_deref(),
            Some(canonical_digest.as_str())
        );
        assert_eq!(
            response.content_type.as_deref(),
            Some(REGISTRYD_OCI_IMAGE_MANIFEST_MEDIA_TYPE)
        );
        assert_eq!(response.body, manifest);

        let official_image = RegistrydHttpService::source()
            .handle_cached_content_request(
                "GET",
                "/v2/alpine/manifests/latest?ns=docker.io",
                &roots,
            )
            .expect("official-image manifest route");

        assert_eq!(official_image.status_code, 200);
        assert_eq!(
            official_image.docker_content_digest.as_deref(),
            Some(canonical_digest.as_str())
        );
        assert_eq!(official_image.body, manifest);

        let cleaned_route = RegistrydHttpService::source()
            .handle_cached_content_request(
                "GET",
                "/v2/library//alpine/manifests/latest?ns=docker.io",
                &roots,
            )
            .expect("source-cleaned manifest route");

        assert_eq!(cleaned_route.status_code, 200);
        assert_eq!(
            cleaned_route.docker_content_digest.as_deref(),
            Some(canonical_digest.as_str())
        );
        assert_eq!(cleaned_route.body, manifest);

        let inferred_official_image = RegistrydHttpService::source()
            .handle_cached_content_request("GET", "/v2/alpine/manifests/latest", &roots)
            .expect("inferred official-image manifest route");

        assert_eq!(inferred_official_image.status_code, 200);
        assert_eq!(
            inferred_official_image.docker_content_digest.as_deref(),
            Some(canonical_digest.as_str())
        );
        assert_eq!(inferred_official_image.body, manifest);
    }

    #[test]
    fn registryd_cached_content_dispatcher_infers_non_domain_namespace_through_source_parse_docker_ref()
     {
        let temp = TestDir::new("registryd-cached-infer-non-domain-ns");
        let root = temp.path("root");
        let manifest = br#"{"config":{},"layers":[]}"#;
        let canonical_digest = registryd_sha256_digest_string(registryd_sha256(manifest));
        let digest_file = registryd_source_digest_file_name(&canonical_digest).unwrap();

        fs::create_dir_all(root.join("manifests/bad")).unwrap();
        let reference_path = root.join("manifests/docker.io/bad/library/alpine/reference/latest");
        let digest_path = root
            .join("manifests/docker.io/bad/library/alpine/digest")
            .join(digest_file);
        fs::create_dir_all(reference_path.parent().unwrap()).unwrap();
        fs::create_dir_all(digest_path.parent().unwrap()).unwrap();
        fs::write(&reference_path, manifest).unwrap();
        fs::write(&digest_path, manifest).unwrap();

        let roots = RegistrydMultiPathFs::new([root]);
        let response = RegistrydHttpService::source()
            .handle_cached_content_request("GET", "/v2/library/alpine/manifests/latest", &roots)
            .expect("manifest route");

        assert_eq!(response.status_code, 200);
        assert_eq!(
            response.docker_content_digest.as_deref(),
            Some(canonical_digest.as_str())
        );
        assert_eq!(
            response.content_type.as_deref(),
            Some(REGISTRYD_OCI_IMAGE_MANIFEST_MEDIA_TYPE)
        );
        assert_eq!(response.body, manifest);
    }

    #[test]
    fn registryd_cached_content_dispatcher_preserves_source_tryfindregistry_parse_errors() {
        let temp = TestDir::new("registryd-cached-infer-parse-error");
        let root = temp.path("root");
        let manifest = br#"{"config":{},"layers":[]}"#;
        let canonical_digest = registryd_sha256_digest_string(registryd_sha256(manifest));
        let digest_file = registryd_source_digest_file_name(&canonical_digest).unwrap();

        fs::create_dir_all(root.join("manifests/%bad")).unwrap();
        let reference_path = root.join("manifests/docker.io/library/alpine/reference/latest");
        let digest_path = root
            .join("manifests/docker.io/library/alpine/digest")
            .join(digest_file);
        fs::create_dir_all(reference_path.parent().unwrap()).unwrap();
        fs::create_dir_all(digest_path.parent().unwrap()).unwrap();
        fs::write(&reference_path, manifest).unwrap();
        fs::write(&digest_path, manifest).unwrap();

        let roots = RegistrydMultiPathFs::new([root]);
        let response = RegistrydHttpService::source()
            .handle_cached_content_request("GET", "/v2/library/alpine/manifests/latest", &roots)
            .expect("manifest route");

        assert_eq!(response.status_code, 400);
        assert_eq!(response.reason, "Bad Request");
        assert!(response.body.is_empty());
    }

    #[test]
    fn registryd_multipath_fs_manifest_entries_follow_source_readdir_sort_order() {
        let temp = TestDir::new("registryd-manifest-entry-order");
        let root = temp.path("root");
        fs::create_dir_all(root.join("manifests/z.example.com")).unwrap();
        fs::create_dir_all(root.join("manifests/a.example.com")).unwrap();

        let roots = RegistrydMultiPathFs::new([root]);

        assert_eq!(
            roots.manifest_entry_names().unwrap(),
            vec!["a.example.com".to_string(), "z.example.com".to_string()]
        );
    }

    #[test]
    fn registryd_cached_content_dispatcher_infers_port_registry_from_source_entry_name() {
        let temp = TestDir::new("registryd-cached-infer-port-ns");
        let root = temp.path("root");
        let manifest = br#"{"manifests":[]}"#;
        let canonical_digest = registryd_sha256_digest_string(registryd_sha256(manifest));
        let digest_file = registryd_source_digest_file_name(&canonical_digest).unwrap();

        let reference_path = root.join("manifests/registry.example.com_5000_/pause/reference/3.9");
        let digest_path = root
            .join("manifests/registry.example.com_5000_/pause/digest")
            .join(digest_file);
        fs::create_dir_all(reference_path.parent().unwrap()).unwrap();
        fs::create_dir_all(digest_path.parent().unwrap()).unwrap();
        fs::write(&reference_path, manifest).unwrap();
        fs::write(&digest_path, manifest).unwrap();

        let roots = RegistrydMultiPathFs::new([root]);
        let response = RegistrydHttpService::source()
            .handle_cached_content_request("GET", "/v2/pause/manifests/3.9", &roots)
            .expect("manifest route");

        assert_eq!(response.status_code, 200);
        assert_eq!(
            response.docker_content_digest.as_deref(),
            Some(canonical_digest.as_str())
        );
        assert_eq!(
            response.content_type.as_deref(),
            Some(REGISTRYD_OCI_IMAGE_INDEX_MEDIA_TYPE)
        );

        let encoded_port = RegistrydHttpService::source()
            .handle_cached_content_request(
                "GET",
                "/v2/pause/manifests/3.9?ns=registry.example.com%3A5000",
                &roots,
            )
            .expect("encoded port namespace manifest route");

        assert_eq!(encoded_port.status_code, 200);
        assert_eq!(
            encoded_port.docker_content_digest.as_deref(),
            Some(canonical_digest.as_str())
        );
    }

    #[test]
    fn registryd_runtime_service_filters_source_config_roots_before_serving_requests() {
        let temp = TestDir::new("registryd-runtime-service-roots");
        let missing_root = temp.path("missing-root");
        let root = temp.path("root");
        let blob_digest = format!("sha256:{}", "2".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "2".repeat(64)));
        let blob = b"runtime-root blob bytes";
        fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        fs::write(&blob_path, blob).unwrap();

        let config = ImageCacheConfig {
            status: ImageCacheStatus::Ready,
            copy_status: ImageCacheCopyStatus::Skipped,
            roots: vec![
                missing_root.display().to_string(),
                root.display().to_string(),
            ],
        };

        let service = RegistrydRuntimeService::from_image_cache_config(&config);

        assert_eq!(service.roots().roots(), std::slice::from_ref(&root));
        assert_eq!(service.skipped_roots().len(), 1);
        assert_eq!(service.skipped_roots()[0].root, missing_root);
        assert_eq!(
            service.skipped_roots()[0].error_kind,
            io::ErrorKind::NotFound
        );

        let response = service
            .handle_request(
                "GET",
                &format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io"),
            )
            .expect("runtime service blob route");

        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, blob);
        assert_eq!(response.content_path, Some(blob_path));
        assert_eq!(
            response.docker_content_digest.as_deref(),
            Some(blob_digest.as_str())
        );
    }

    #[test]
    fn registryd_runtime_service_serves_from_runtime_plan_roots_with_namespace_inference() {
        let temp = TestDir::new("registryd-runtime-service-plan");
        let root = temp.path("root");
        let manifest = br#"{"config":{},"layers":[]}"#;
        let canonical_digest = registryd_sha256_digest_string(registryd_sha256(manifest));
        let digest_file = registryd_source_digest_file_name(&canonical_digest).unwrap();

        let reference_path = root.join("manifests/docker.io/library/alpine/reference/latest");
        let digest_path = root
            .join("manifests/docker.io/library/alpine/digest")
            .join(digest_file);
        fs::create_dir_all(reference_path.parent().unwrap()).unwrap();
        fs::create_dir_all(digest_path.parent().unwrap()).unwrap();
        fs::write(&reference_path, manifest).unwrap();
        fs::write(&digest_path, manifest).unwrap();

        let mut plan = registryd_runtime_plan();
        plan.config.roots = vec![root.display().to_string()];
        let service = RegistrydRuntimeService::from_runtime_plan(&plan);

        assert_eq!(service.roots().roots(), std::slice::from_ref(&root));

        let response = service
            .handle_request("GET", "/v2/library/alpine/manifests/latest")
            .expect("runtime service inferred manifest route");

        assert_eq!(response.status_code, 200);
        assert_eq!(
            response.docker_content_digest.as_deref(),
            Some(canonical_digest.as_str())
        );
        assert_eq!(
            response.content_type.as_deref(),
            Some(REGISTRYD_OCI_IMAGE_MANIFEST_MEDIA_TYPE)
        );
        assert_eq!(response.body, manifest);
    }

    #[test]
    fn image_cache_registryd_runtime_adapter_loads_runtime_service_roots_when_manager_missing() {
        let temp = TestDir::new("registryd-runtime-adapter-service-roots");
        let missing_root = temp.path("missing-root");
        let root = temp.path("root");
        fs::create_dir_all(&root).unwrap();

        let mut plan = registryd_runtime_plan();
        plan.config.roots = vec![
            missing_root.display().to_string(),
            root.display().to_string(),
        ];
        let mut manager = TestRegistrydServiceManager::missing();

        let report = RegistrydRuntimeAdapter
            .execute(&plan, &mut manager)
            .unwrap();

        assert_eq!(
            report.status,
            RegistrydServiceExecutionStatus::LoadedAndStarted
        );
        assert_eq!(manager.loaded_services.len(), 1);
        let loaded = &manager.loaded_services[0];
        assert_eq!(loaded.roots().roots(), std::slice::from_ref(&root));
        assert_eq!(loaded.skipped_roots()[0].root, missing_root);
        assert_eq!(
            loaded.skipped_roots()[0].error_kind,
            io::ErrorKind::NotFound
        );
    }

    #[test]
    fn image_cache_registryd_runtime_adapter_loads_and_starts_when_manager_missing() {
        let plan = registryd_runtime_plan();
        let mut manager = TestRegistrydServiceManager::missing();

        let report = RegistrydRuntimeAdapter
            .execute(&plan, &mut manager)
            .unwrap();

        assert_eq!(REGISTRYD_SERVICE_ID, "registryd");
        assert_eq!(
            report.status,
            RegistrydServiceExecutionStatus::LoadedAndStarted
        );
        assert_eq!(report.service_id, REGISTRYD_SERVICE_ID);
        assert!(report.loaded);
        assert!(report.started);
        assert_eq!(manager.queried, vec![REGISTRYD_SERVICE_ID]);
        assert_eq!(manager.loaded, 1);
        assert_eq!(manager.started, vec![REGISTRYD_SERVICE_ID]);
    }

    #[test]
    fn image_cache_registryd_runtime_adapter_starts_when_manager_reports_stopped() {
        let plan = registryd_runtime_plan();
        let mut manager = TestRegistrydServiceManager::running(false);

        let report = RegistrydRuntimeAdapter
            .execute(&plan, &mut manager)
            .unwrap();

        assert_eq!(report.status, RegistrydServiceExecutionStatus::Started);
        assert_eq!(report.service_id, REGISTRYD_SERVICE_ID);
        assert!(!report.loaded);
        assert!(report.started);
        assert_eq!(manager.queried, vec![REGISTRYD_SERVICE_ID]);
        assert_eq!(manager.loaded, 0);
        assert_eq!(manager.started, vec![REGISTRYD_SERVICE_ID]);
    }

    #[test]
    fn image_cache_registryd_runtime_adapter_skips_when_running_or_no_roots() {
        let plan = registryd_runtime_plan();
        let mut running_manager = TestRegistrydServiceManager::running(true);

        let report = RegistrydRuntimeAdapter
            .execute(&plan, &mut running_manager)
            .unwrap();

        assert_eq!(
            report.status,
            RegistrydServiceExecutionStatus::AlreadyRunning
        );
        assert_eq!(running_manager.loaded, 0);
        assert!(running_manager.started.is_empty());

        let mut no_root_plan = registryd_runtime_plan();
        no_root_plan.config.roots.clear();
        let mut idle_manager = TestRegistrydServiceManager::running(false);
        let idle_report = RegistrydRuntimeAdapter
            .execute(&no_root_plan, &mut idle_manager)
            .unwrap();

        assert_eq!(
            idle_report.status,
            RegistrydServiceExecutionStatus::NoAction
        );
        assert!(idle_manager.queried.is_empty());
        assert_eq!(idle_manager.loaded, 0);
        assert!(idle_manager.started.is_empty());
    }

    #[test]
    fn image_cache_machine_config_local_enabled_parser_matches_source_shape() {
        let enabled = r#"
version: v1alpha1
machine:
  type: worker
  features:
    imageCache:
      localEnabled: true
---
apiVersion: v1alpha1
kind: SideroLinkConfig
apiUrl: grpc://example
"#;
        assert!(image_cache_local_enabled_from_machine_config_contents(enabled).unwrap());

        let explicit_false = r#"
version: v1alpha1
machine:
  features:
    imageCache:
      localEnabled: false
"#;
        assert!(!image_cache_local_enabled_from_machine_config_contents(explicit_false).unwrap());

        let absent = "version: v1alpha1
machine:
  type: worker
";
        assert!(!image_cache_local_enabled_from_machine_config_contents(absent).unwrap());
        assert!(!image_cache_local_enabled_from_machine_config_contents("").unwrap());

        let invalid = r#"
version: v1alpha1
machine:
  features:
    imageCache:
      localEnabled: sometimes
"#;
        let err = image_cache_local_enabled_from_machine_config_contents(invalid).unwrap_err();
        assert!(err.contains("localEnabled must be boolean"));
    }

    #[test]
    fn image_cache_disabled_has_disabled_config_and_no_mounts() {
        let mut controller = ImageCacheConfigController::new();
        let plan = reconcile(&mut controller, false, RegistrydState::default(), &[], &[]);

        assert_eq!(plan.config.status, ImageCacheStatus::Disabled);
        assert_eq!(plan.config.copy_status, ImageCacheCopyStatus::Skipped);
        assert!(plan.config.roots.is_empty());
        assert!(plan.mount_requests.is_empty());
        assert_eq!(plan.registryd_action, RegistrydAction::None);
    }

    #[test]
    fn image_cache_disk_ready_without_iso_mounts_readonly_and_becomes_ready_after_registryd() {
        let mut controller = ImageCacheConfigController::new();
        let volumes = vec![volume(IMAGE_CACHE_DISK_VOLUME_ID, VolumePhase::Ready)];
        let mounts = vec![mount(
            IMAGE_CACHE_DISK_VOLUME_ID,
            IMAGE_CACHE_DISK_MOUNT_POINT,
            true,
        )];

        let plan = reconcile(&mut controller, true, ready_registryd(), &volumes, &mounts);

        assert_eq!(plan.config.status, ImageCacheStatus::Ready);
        assert_eq!(plan.config.copy_status, ImageCacheCopyStatus::Skipped);
        assert_eq!(plan.config.roots, vec![IMAGE_CACHE_DISK_MOUNT_POINT]);
        assert_eq!(plan.mount_requests.len(), 1);
        assert_eq!(
            plan.mount_requests[0].id,
            image_cache_mount_status_id(IMAGE_CACHE_DISK_VOLUME_ID)
        );
        assert!(plan.mount_requests[0].spec.read_only);
        assert_eq!(
            plan.finalizer_actions,
            vec![ImageCacheFinalizerAction {
                status_id: image_cache_mount_status_id(IMAGE_CACHE_DISK_VOLUME_ID),
                operation: ImageCacheFinalizerOperation::Add,
            }]
        );
    }

    #[test]
    fn image_cache_iso_and_disk_ready_emit_rw_disk_ro_iso_copy_ready_roots_order() {
        let mut controller = ImageCacheConfigController::new();
        let volumes = vec![
            volume(IMAGE_CACHE_DISK_VOLUME_ID, VolumePhase::Ready),
            volume(IMAGE_CACHE_ISO_VOLUME_ID, VolumePhase::Ready),
        ];
        let mounts = vec![
            mount(
                IMAGE_CACHE_DISK_VOLUME_ID,
                IMAGE_CACHE_DISK_MOUNT_POINT,
                false,
            ),
            mount(IMAGE_CACHE_ISO_VOLUME_ID, IMAGE_CACHE_ISO_MOUNT_POINT, true),
        ];

        let plan = reconcile(&mut controller, true, ready_registryd(), &volumes, &mounts);

        assert_eq!(plan.config.status, ImageCacheStatus::Ready);
        assert_eq!(plan.config.copy_status, ImageCacheCopyStatus::Ready);
        assert_eq!(
            plan.config.roots,
            vec![
                IMAGE_CACHE_DISK_MOUNT_POINT.to_string(),
                format!("{IMAGE_CACHE_ISO_MOUNT_POINT}/imagecache"),
            ]
        );

        let disk_request = plan
            .mount_requests
            .iter()
            .find(|request| request.spec.volume_id == IMAGE_CACHE_DISK_VOLUME_ID)
            .unwrap();
        let iso_request = plan
            .mount_requests
            .iter()
            .find(|request| request.spec.volume_id == IMAGE_CACHE_ISO_VOLUME_ID)
            .unwrap();
        assert!(!disk_request.spec.read_only);
        assert!(iso_request.spec.read_only);
        assert_eq!(
            plan.copy_plan,
            Some(ImageCacheCopyPlan {
                source: format!("{IMAGE_CACHE_ISO_MOUNT_POINT}/imagecache"),
                target: IMAGE_CACHE_DISK_MOUNT_POINT.to_string(),
            })
        );
        assert!(!controller.cache_copy_done());

        controller.mark_cache_copy_done();
        let post_copy = reconcile(&mut controller, true, ready_registryd(), &volumes, &mounts);

        assert_eq!(post_copy.config.copy_status, ImageCacheCopyStatus::Ready);
        assert_eq!(post_copy.copy_plan, None);
        assert!(controller.cache_copy_done());
    }

    #[test]
    fn image_cache_copy_intent_does_not_mark_done_before_runtime_success() {
        let mut controller = ImageCacheConfigController::new();
        let volumes = vec![
            volume(IMAGE_CACHE_DISK_VOLUME_ID, VolumePhase::Ready),
            volume(IMAGE_CACHE_ISO_VOLUME_ID, VolumePhase::Ready),
        ];
        let mounts = vec![
            mount(
                IMAGE_CACHE_DISK_VOLUME_ID,
                IMAGE_CACHE_DISK_MOUNT_POINT,
                false,
            ),
            mount(IMAGE_CACHE_ISO_VOLUME_ID, IMAGE_CACHE_ISO_MOUNT_POINT, true),
        ];

        let plan = reconcile(&mut controller, true, ready_registryd(), &volumes, &mounts);

        assert_eq!(
            plan.copy_plan,
            Some(ImageCacheCopyPlan {
                source: format!("{IMAGE_CACHE_ISO_MOUNT_POINT}/imagecache"),
                target: IMAGE_CACHE_DISK_MOUNT_POINT.to_string(),
            })
        );
        assert!(!controller.cache_copy_done());

        controller.mark_cache_copy_done();
        let post_copy = reconcile(&mut controller, true, ready_registryd(), &volumes, &mounts);

        assert_eq!(post_copy.config.copy_status, ImageCacheCopyStatus::Ready);
        assert_eq!(post_copy.copy_plan, None);
        assert!(controller.cache_copy_done());
    }

    #[test]
    fn image_cache_ready_volume_waits_for_mount_status_before_using_root() {
        let mut controller = ImageCacheConfigController::new();
        let volumes = vec![volume(IMAGE_CACHE_DISK_VOLUME_ID, VolumePhase::Ready)];

        let plan = reconcile(&mut controller, true, ready_registryd(), &volumes, &[]);

        assert_eq!(plan.config.status, ImageCacheStatus::Preparing);
        assert_eq!(plan.config.copy_status, ImageCacheCopyStatus::Skipped);
        assert!(plan.config.roots.is_empty());
        assert_eq!(plan.mount_requests.len(), 1);
        assert!(plan.finalizer_actions.is_empty());
    }

    #[test]
    fn image_cache_root_starts_registryd_until_service_is_healthy() {
        let mut controller = ImageCacheConfigController::new();
        let volumes = vec![volume(IMAGE_CACHE_DISK_VOLUME_ID, VolumePhase::Ready)];
        let mounts = vec![mount(
            IMAGE_CACHE_DISK_VOLUME_ID,
            IMAGE_CACHE_DISK_MOUNT_POINT,
            true,
        )];

        let plan = reconcile(
            &mut controller,
            true,
            RegistrydState {
                running: false,
                healthy: false,
            },
            &volumes,
            &mounts,
        );

        assert_eq!(plan.config.status, ImageCacheStatus::Preparing);
        assert_eq!(plan.registryd_action, RegistrydAction::Start);
        assert_eq!(plan.config.roots, vec![IMAGE_CACHE_DISK_MOUNT_POINT]);
    }

    #[test]
    fn image_cache_tearing_down_mount_status_removes_finalizer_and_drops_root() {
        let mut controller = ImageCacheConfigController::new();
        let volumes = vec![volume(IMAGE_CACHE_DISK_VOLUME_ID, VolumePhase::Ready)];
        let mut disk_mount = mount(
            IMAGE_CACHE_DISK_VOLUME_ID,
            IMAGE_CACHE_DISK_MOUNT_POINT,
            true,
        );
        disk_mount
            .metadata_mut()
            .finalizers_mut()
            .add(IMAGE_CACHE_CONTROLLER_NAME);
        disk_mount.metadata_mut().set_phase(CosiPhase::TearingDown);
        let mounts = vec![disk_mount];

        let plan = reconcile(&mut controller, true, ready_registryd(), &volumes, &mounts);

        assert_eq!(plan.config.status, ImageCacheStatus::Disabled);
        assert!(plan.config.roots.is_empty());
        assert_eq!(
            plan.finalizer_actions,
            vec![ImageCacheFinalizerAction {
                status_id: image_cache_mount_status_id(IMAGE_CACHE_DISK_VOLUME_ID),
                operation: ImageCacheFinalizerOperation::Remove,
            }]
        );
    }

    #[test]
    fn image_cache_bridge_writes_config_mount_request_and_finalizer_to_cosi_state() {
        let mut controller = ImageCacheConfigController::new();
        let volumes = vec![volume(IMAGE_CACHE_DISK_VOLUME_ID, VolumePhase::Ready)];
        let mounts = vec![mount(
            IMAGE_CACHE_DISK_VOLUME_ID,
            IMAGE_CACHE_DISK_MOUNT_POINT,
            true,
        )];
        let plan = reconcile(&mut controller, true, ready_registryd(), &volumes, &mounts);

        let mut state = os_cosi_domain::State::new();
        state.create(Box::new(mounts[0].clone())).unwrap();

        apply_image_cache_plan_to_state(&mut state, &plan).unwrap();

        let config = state.get(&image_cache_config_key().unwrap()).unwrap();
        assert_eq!(config.metadata().namespace(), IMAGE_CACHE_NAMESPACE);
        assert_eq!(config.metadata().owner(), IMAGE_CACHE_CONTROLLER_NAME);
        assert_eq!(
            config.spec_fingerprint(),
            "status=ready;copy_status=skipped;roots=[/system/imagecache/disk]"
        );

        let request_key =
            volume_mount_request_key(&image_cache_mount_status_id(IMAGE_CACHE_DISK_VOLUME_ID))
                .unwrap();
        let request = state.get(&request_key).unwrap();
        assert_eq!(request.metadata().owner(), IMAGE_CACHE_CONTROLLER_NAME);
        assert_eq!(
            request.spec_fingerprint(),
            "volume_id=IMAGECACHE;requester=cri.ImageCacheConfigController;read_only=true;detached=false;disable_access_time=false;secure=false"
        );

        let mounted = state.get(mounts[0].metadata().key().as_str()).unwrap();
        assert!(
            mounted
                .metadata()
                .finalizers()
                .contains(IMAGE_CACHE_CONTROLLER_NAME)
        );

        let config_version = config.metadata().version();
        let request_version = request.metadata().version();
        let mounted_version = mounted.metadata().version();
        apply_image_cache_plan_to_state(&mut state, &plan).unwrap();

        assert_eq!(
            state
                .get(&image_cache_config_key().unwrap())
                .unwrap()
                .metadata()
                .version(),
            config_version
        );
        assert_eq!(
            state.get(&request_key).unwrap().metadata().version(),
            request_version
        );
        assert_eq!(
            state
                .get(mounts[0].metadata().key().as_str())
                .unwrap()
                .metadata()
                .version(),
            mounted_version
        );
    }

    #[test]
    fn image_cache_bridge_removes_finalizer_from_tearing_down_mount_status() {
        let mut controller = ImageCacheConfigController::new();
        let volumes = vec![volume(IMAGE_CACHE_DISK_VOLUME_ID, VolumePhase::Ready)];
        let mut disk_mount = mount(
            IMAGE_CACHE_DISK_VOLUME_ID,
            IMAGE_CACHE_DISK_MOUNT_POINT,
            true,
        );
        disk_mount
            .metadata_mut()
            .finalizers_mut()
            .add(IMAGE_CACHE_CONTROLLER_NAME);
        disk_mount.metadata_mut().set_phase(CosiPhase::TearingDown);
        let mounts = vec![disk_mount];
        let plan = reconcile(&mut controller, true, ready_registryd(), &volumes, &mounts);

        let mut state = os_cosi_domain::State::new();
        state.create(Box::new(mounts[0].clone())).unwrap();

        apply_image_cache_plan_to_state(&mut state, &plan).unwrap();

        let mounted = state.get(mounts[0].metadata().key().as_str()).unwrap();
        assert!(
            !mounted
                .metadata()
                .finalizers()
                .contains(IMAGE_CACHE_CONTROLLER_NAME)
        );
        assert_eq!(
            state
                .get(&image_cache_config_key().unwrap())
                .unwrap()
                .spec_fingerprint(),
            "status=disabled;copy_status=skipped;roots=[]"
        );
    }

    #[test]
    fn image_cache_cosi_controller_runs_under_runtime_loop() {
        let mut runtime = os_cosi_domain::Runtime::new();
        let disk_volume =
            VolumeStatusResource::new(volume(IMAGE_CACHE_DISK_VOLUME_ID, VolumePhase::Ready))
                .unwrap();
        let disk_mount = mount(
            IMAGE_CACHE_DISK_VOLUME_ID,
            IMAGE_CACHE_DISK_MOUNT_POINT,
            true,
        );
        runtime
            .state_mut()
            .create(Box::new(disk_volume.clone()))
            .unwrap();
        runtime
            .state_mut()
            .create(Box::new(disk_mount.clone()))
            .unwrap();
        runtime
            .state_mut()
            .create(Box::new(enabled_image_cache_machine_config()))
            .unwrap();
        runtime
            .register(Box::new(ImageCacheCosiController::new(
                true,
                ready_registryd(),
            )))
            .unwrap();

        runtime.run().unwrap();

        let config = runtime
            .state()
            .get(&image_cache_config_key().unwrap())
            .unwrap();
        assert_eq!(config.metadata().owner(), IMAGE_CACHE_CONTROLLER_NAME);
        assert_eq!(
            config.spec_fingerprint(),
            "status=ready;copy_status=skipped;roots=[/system/imagecache/disk]"
        );

        let request_key =
            volume_mount_request_key(&image_cache_mount_status_id(IMAGE_CACHE_DISK_VOLUME_ID))
                .unwrap();
        let request = runtime.state().get(&request_key).unwrap();
        assert_eq!(
            request.spec_fingerprint(),
            "volume_id=IMAGECACHE;requester=cri.ImageCacheConfigController;read_only=true;detached=false;disable_access_time=false;secure=false"
        );
        assert!(
            runtime
                .state()
                .get(disk_mount.metadata().key().as_str())
                .unwrap()
                .metadata()
                .finalizers()
                .contains(IMAGE_CACHE_CONTROLLER_NAME)
        );

        runtime.run().unwrap();
        assert!(
            runtime.history().iter().all(|record| record.writes == 0),
            "second runtime pass must be idempotent: {:?}",
            runtime.history()
        );
    }

    #[test]
    fn image_cache_cosi_controller_reads_local_enabled_from_machine_config_input() {
        let mut runtime = os_cosi_domain::Runtime::new();
        let disk_volume =
            VolumeStatusResource::new(volume(IMAGE_CACHE_DISK_VOLUME_ID, VolumePhase::Ready))
                .unwrap();
        let disk_mount = mount(
            IMAGE_CACHE_DISK_VOLUME_ID,
            IMAGE_CACHE_DISK_MOUNT_POINT,
            true,
        );
        let registryd_service = V1Alpha1ServiceResource::registryd(V1Alpha1ServiceSpec {
            running: true,
            healthy: true,
            unknown: false,
        })
        .unwrap();
        runtime
            .state_mut()
            .create(Box::new(disk_volume.clone()))
            .unwrap();
        runtime
            .state_mut()
            .create(Box::new(disk_mount.clone()))
            .unwrap();
        runtime
            .state_mut()
            .create(Box::new(enabled_image_cache_machine_config()))
            .unwrap();
        runtime
            .state_mut()
            .create(Box::new(registryd_service))
            .unwrap();
        runtime
            .register(Box::new(ImageCacheCosiController::new(
                false,
                RegistrydState::default(),
            )))
            .unwrap();

        runtime.run().unwrap();

        let config = runtime
            .state()
            .get(&image_cache_config_key().unwrap())
            .unwrap();
        assert_eq!(
            config.spec_fingerprint(),
            "status=ready;copy_status=skipped;roots=[/system/imagecache/disk]"
        );
    }

    #[test]
    fn image_cache_cosi_controller_reads_registryd_service_input_from_context() {
        let mut runtime = os_cosi_domain::Runtime::new();
        let disk_volume =
            VolumeStatusResource::new(volume(IMAGE_CACHE_DISK_VOLUME_ID, VolumePhase::Ready))
                .unwrap();
        let disk_mount = mount(
            IMAGE_CACHE_DISK_VOLUME_ID,
            IMAGE_CACHE_DISK_MOUNT_POINT,
            true,
        );
        let registryd_service = V1Alpha1ServiceResource::registryd(V1Alpha1ServiceSpec {
            running: true,
            healthy: true,
            unknown: false,
        })
        .unwrap();
        runtime
            .state_mut()
            .create(Box::new(disk_volume.clone()))
            .unwrap();
        runtime
            .state_mut()
            .create(Box::new(disk_mount.clone()))
            .unwrap();
        runtime
            .state_mut()
            .create(Box::new(enabled_image_cache_machine_config()))
            .unwrap();
        runtime
            .state_mut()
            .create(Box::new(registryd_service))
            .unwrap();
        runtime
            .register(Box::new(ImageCacheCosiController::new(
                true,
                RegistrydState::default(),
            )))
            .unwrap();

        runtime.run().unwrap();

        let config = runtime
            .state()
            .get(&image_cache_config_key().unwrap())
            .unwrap();
        assert_eq!(
            config.spec_fingerprint(),
            "status=ready;copy_status=skipped;roots=[/system/imagecache/disk]"
        );
    }

    #[test]
    fn image_cache_cosi_controller_disables_without_machine_config_input() {
        let mut runtime = os_cosi_domain::Runtime::new();
        let disk_volume =
            VolumeStatusResource::new(volume(IMAGE_CACHE_DISK_VOLUME_ID, VolumePhase::Ready))
                .unwrap();
        let disk_mount = mount(
            IMAGE_CACHE_DISK_VOLUME_ID,
            IMAGE_CACHE_DISK_MOUNT_POINT,
            true,
        );
        let registryd_service = V1Alpha1ServiceResource::registryd(V1Alpha1ServiceSpec {
            running: true,
            healthy: true,
            unknown: false,
        })
        .unwrap();
        runtime
            .state_mut()
            .create(Box::new(disk_volume.clone()))
            .unwrap();
        runtime
            .state_mut()
            .create(Box::new(disk_mount.clone()))
            .unwrap();
        runtime
            .state_mut()
            .create(Box::new(registryd_service))
            .unwrap();
        runtime
            .register(Box::new(ImageCacheCosiController::new(
                true,
                ready_registryd(),
            )))
            .unwrap();

        runtime.run().unwrap();

        let config = runtime
            .state()
            .get(&image_cache_config_key().unwrap())
            .unwrap();
        assert_eq!(
            config.spec_fingerprint(),
            "status=disabled;copy_status=skipped;roots=[]"
        );
    }

    #[test]
    fn image_cache_cosi_controller_writes_volume_configs_from_enabled_machine_config() {
        let mut runtime = os_cosi_domain::Runtime::new();
        runtime
            .state_mut()
            .create(Box::new(enabled_image_cache_machine_config()))
            .unwrap();
        runtime
            .register(Box::new(ImageCacheCosiController::new(
                false,
                RegistrydState::default(),
            )))
            .unwrap();

        runtime.run().unwrap();

        let iso = runtime
            .state()
            .get("runtime/VolumeConfigs.block.talos.dev/IMAGECACHE-ISO")
            .expect("source creates ISO VolumeConfig when image cache is enabled");
        assert_eq!(iso.metadata().owner(), IMAGE_CACHE_CONTROLLER_NAME);
        assert_eq!(
            iso.spec_fingerprint(),
            "id=IMAGECACHE-ISO;type=disk;locator_match=volume.name in [\"iso9660\", \"vfat\"] && volume.label.startsWith(\"TALOS_\");locator_disk_match=;provisioning_wave=0;provisioning_disk_selector=;provisioning_external_source=;provisioning_label=;provisioning_min_size=0;provisioning_max_size=0;provisioning_relative_max_size=0;provisioning_negative_max_size=false;provisioning_grow=false;provisioning_type_uuid=;provisioning_filesystem=;provisioning_encryption_configured=false;mount_target=/system/imagecache/iso;mount_selinux_label=;mount_project_quota_support=false;mount_file_mode=448;mount_uid=0;mount_gid=0;mount_secure=false;mount_parent_id=;mount_bind_target="
        );

        let disk = runtime
            .state()
            .get("runtime/VolumeConfigs.block.talos.dev/IMAGECACHE")
            .expect("source creates disk VolumeConfig when image cache is enabled");
        assert_eq!(disk.metadata().owner(), IMAGE_CACHE_CONTROLLER_NAME);
        assert_eq!(
            disk.spec_fingerprint(),
            "id=IMAGECACHE;type=partition;locator_match=volume.partition_label == \"IMAGECACHE\";locator_disk_match=;provisioning_wave=0;provisioning_disk_selector=;provisioning_external_source=;provisioning_label=;provisioning_min_size=0;provisioning_max_size=0;provisioning_relative_max_size=0;provisioning_negative_max_size=false;provisioning_grow=false;provisioning_type_uuid=;provisioning_filesystem=;provisioning_encryption_configured=false;mount_target=/system/imagecache/disk;mount_selinux_label=;mount_project_quota_support=false;mount_file_mode=448;mount_uid=0;mount_gid=0;mount_secure=false;mount_parent_id=;mount_bind_target="
        );
    }

    #[test]
    fn image_cache_volume_config_projection_uses_source_disk_overrides() {
        let configs = image_cache_volume_configs_from_machine_config_contents(
            r#"
version: v1alpha1
machine:
  type: worker
  features:
    imageCache:
      localEnabled: true
---
apiVersion: v1alpha1
kind: VolumeConfig
name: IMAGECACHE
provisioning:
  diskSelector:
    match: disk.transport == "nvme"
  minSize: 629145600
  maxSize: 2147483648
  grow: true
"#,
        )
        .unwrap();

        let disk = configs
            .iter()
            .find(|config| config.metadata().id().as_str() == IMAGE_CACHE_DISK_VOLUME_ID)
            .expect("disk VolumeConfig");
        assert_eq!(
            disk.spec_fingerprint(),
            "id=IMAGECACHE;type=partition;locator_match=volume.partition_label == \"IMAGECACHE\";locator_disk_match=;provisioning_wave=-1;provisioning_disk_selector=disk.transport == \"nvme\";provisioning_external_source=;provisioning_label=IMAGECACHE;provisioning_min_size=629145600;provisioning_max_size=2147483648;provisioning_relative_max_size=0;provisioning_negative_max_size=false;provisioning_grow=true;provisioning_type_uuid=0fc63daf-8483-4772-8e79-3d69d8477de4;provisioning_filesystem=ext4;provisioning_encryption_configured=false;mount_target=/system/imagecache/disk;mount_selinux_label=;mount_project_quota_support=false;mount_file_mode=448;mount_uid=0;mount_gid=0;mount_secure=false;mount_parent_id=;mount_bind_target="
        );
    }

    #[test]
    fn image_cache_cosi_controller_does_not_create_volume_configs_when_disabled() {
        let mut runtime = os_cosi_domain::Runtime::new();
        runtime
            .register(Box::new(ImageCacheCosiController::new(
                true,
                ready_registryd(),
            )))
            .unwrap();

        runtime.run().unwrap();

        assert!(
            runtime
                .state()
                .get("runtime/VolumeConfigs.block.talos.dev/IMAGECACHE-ISO")
                .is_none()
        );
        assert!(
            runtime
                .state()
                .get("runtime/VolumeConfigs.block.talos.dev/IMAGECACHE")
                .is_none()
        );
    }

    #[test]
    fn image_cache_cosi_controller_spec_matches_source_declarations() {
        let spec = ImageCacheCosiController::new(false, RegistrydState::default()).spec();

        let expected_inputs = vec![
            Input::weak(ResourceKind::new(
                crate::cri::MACHINE_CONFIG_NAMESPACE,
                crate::cri::MACHINE_CONFIG_TYPE,
            ))
            .with_id(crate::cri::MACHINE_CONFIG_ACTIVE_ID),
            Input::weak(VolumeStatusResource::kind()),
            Input::weak(ResourceKind::new(V1ALPHA1_NAMESPACE, V1ALPHA1_SERVICE_TYPE))
                .with_id(REGISTRYD_SERVICE_ID),
            Input::strong(VolumeMountStatusResource::kind()),
            Input::destroy_ready(VolumeMountRequestResource::kind()),
        ];
        assert_eq!(spec.inputs(), expected_inputs.as_slice());
        assert_eq!(
            spec.strong_input_kinds(),
            vec![VolumeMountStatusResource::kind()]
        );
        assert!(
            spec.input_kinds()
                .contains(&VolumeMountRequestResource::kind())
        );

        let expected_outputs = vec![
            Output::exclusive(ImageCacheConfigResource::kind()),
            Output::shared(ResourceKind::new(
                os_block_domain::mount::BLOCK_NAMESPACE,
                VOLUME_CONFIG_TYPE,
            )),
            Output::shared(VolumeMountRequestResource::kind()),
        ];
        assert_eq!(spec.outputs(), expected_outputs.as_slice());
    }

    #[test]
    fn image_cache_source_resource_constants_match_talos() {
        assert_eq!(IMAGE_CACHE_CONFIG_TYPE, "ImageCacheConfigs.cri.talos.dev");
        assert_eq!(IMAGE_CACHE_CONFIG_ID, "image-cache");
        assert_eq!(
            IMAGE_CACHE_CONTROLLER_NAME,
            "cri.ImageCacheConfigController"
        );
        assert_eq!(IMAGE_CACHE_DISK_VOLUME_ID, "IMAGECACHE");
        assert_eq!(IMAGE_CACHE_ISO_VOLUME_ID, "IMAGECACHE-ISO");
        assert_eq!(IMAGE_CACHE_DISK_MOUNT_POINT, "/system/imagecache/disk");
        assert_eq!(IMAGE_CACHE_ISO_MOUNT_POINT, "/system/imagecache/iso");
        assert_eq!(V1ALPHA1_NAMESPACE, "runtime");
        assert_eq!(V1ALPHA1_SERVICE_TYPE, "Services.v1alpha1.talos.dev");
        assert_eq!(VOLUME_CONFIG_TYPE, "VolumeConfigs.block.talos.dev");
        assert_eq!(
            machine_config_kind(),
            ResourceKind::new(
                crate::cri::MACHINE_CONFIG_NAMESPACE,
                crate::cri::MACHINE_CONFIG_TYPE,
            )
        );
        assert_eq!(
            registryd_service_kind(),
            ResourceKind::new(V1ALPHA1_NAMESPACE, V1ALPHA1_SERVICE_TYPE)
        );
        assert_eq!(
            registryd_service_key().unwrap(),
            "runtime/Services.v1alpha1.talos.dev/registryd"
        );
        let service_spec = V1Alpha1ServiceSpec {
            running: true,
            healthy: true,
            unknown: false,
        };
        let service = V1Alpha1ServiceResource::registryd(service_spec).unwrap();
        assert_eq!(V1Alpha1ServiceResource::kind(), registryd_service_kind());
        assert_eq!(service.metadata().key(), registryd_service_key().unwrap());
        assert_eq!(
            service.spec_fingerprint(),
            "running=true;healthy=true;unknown=false"
        );
        assert_eq!(
            V1Alpha1ServiceResource::from_resource(&service)
                .unwrap()
                .spec,
            service_spec
        );
        assert_eq!(
            volume_config_kind(),
            ResourceKind::new(os_block_domain::mount::BLOCK_NAMESPACE, VOLUME_CONFIG_TYPE)
        );
    }
}
