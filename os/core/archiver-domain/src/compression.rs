//! Decompression abstractions for the archiver.
//!
//! Talos consumes compressed streams in several places: installer image layers
//! (gzip / zstd), the kernel/initramfs (`xz`, `zstd`), and `talosctl` artifacts.
//! Talos detects the codec from a stream's magic number, then hands the stream
//! to the appropriate decompressor.
//!
//! Implementing production xz/zstd/gzip decoders is out of scope (and would need
//! external crates, which this workspace forbids), so the OS boundary — the
//! actual decompression — is modeled as the [`Decompressor`] trait. The *codec
//! detection*, *framing*, and *dispatch* logic is the real, faithful part and is
//! fully tested. An [`IdentityCodec`] provides a reversible in-memory codec so
//! the end-to-end pipeline (detect -> decompress) is exercised offline, and the
//! real gzip/xz/zstd codecs validate magic and surface an [`Unsupported`] error
//! for actual payloads, exactly where a real decoder would be wired in.
//!
//! [`Unsupported`]: os_kernel::Error::Unsupported

use os_kernel::Error;

/// The compression codecs Talos recognizes by magic number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    /// Uncompressed pass-through framing used by this crate's tests.
    Identity,
    /// gzip (RFC 1952), magic `1f 8b`.
    Gzip,
    /// xz (`.xz`), magic `fd 37 7a 58 5a 00`.
    Xz,
    /// zstd, magic `28 b5 2f fd`.
    Zstd,
}

impl Codec {
    /// The magic-number prefix identifying this codec on the wire.
    pub fn magic(self) -> &'static [u8] {
        match self {
            // Identity framing uses a private 4-byte sentinel.
            Codec::Identity => b"ID0\x01",
            Codec::Gzip => &[0x1f, 0x8b],
            Codec::Xz => &[0xfd, b'7', b'z', b'X', b'Z', 0x00],
            Codec::Zstd => &[0x28, 0xb5, 0x2f, 0xfd],
        }
    }

    /// A short, stable name.
    pub fn name(self) -> &'static str {
        match self {
            Codec::Identity => "identity",
            Codec::Gzip => "gzip",
            Codec::Xz => "xz",
            Codec::Zstd => "zstd",
        }
    }

    /// Detect the codec of `data` from its leading magic bytes.
    pub fn detect(data: &[u8]) -> Option<Codec> {
        // Check longer magics first to avoid ambiguity.
        for codec in [Codec::Xz, Codec::Zstd, Codec::Identity, Codec::Gzip] {
            let m = codec.magic();
            if data.len() >= m.len() && &data[..m.len()] == m {
                return Some(codec);
            }
        }
        None
    }
}

/// The OS-boundary trait: turn a compressed stream into plaintext bytes.
///
/// Real Talos backs this with a streaming decoder (`compress/gzip`,
/// `github.com/ulikunitz/xz`, `klauspost/compress/zstd`). The in-memory impls
/// here either round-trip ([`IdentityCodec`]) or validate framing and report
/// the decode boundary as unsupported.
pub trait Decompressor {
    /// The codec this decompressor handles.
    fn codec(&self) -> Codec;

    /// Decompress a full in-memory buffer.
    fn decompress(&self, input: &[u8]) -> crate::Result<Vec<u8>>;

    /// Validate that `input` begins with this codec's magic.
    fn check_magic(&self, input: &[u8]) -> crate::Result<()> {
        let m = self.codec().magic();
        if input.len() < m.len() || &input[..m.len()] != m {
            return Err(Error::parse(format!(
                "stream is not {} (bad magic)",
                self.codec().name()
            )));
        }
        Ok(())
    }
}

/// A reversible pass-through codec. Framing: 4-byte magic, 4-byte big-endian
/// length, then the raw payload. Used to exercise the full pipeline in tests.
#[derive(Debug, Clone, Copy, Default)]
pub struct IdentityCodec;

impl IdentityCodec {
    /// Frame `payload` into an identity-coded blob.
    pub fn compress(&self, payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(8 + payload.len());
        out.extend_from_slice(Codec::Identity.magic());
        out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        out.extend_from_slice(payload);
        out
    }
}

impl Decompressor for IdentityCodec {
    fn codec(&self) -> Codec {
        Codec::Identity
    }

    fn decompress(&self, input: &[u8]) -> crate::Result<Vec<u8>> {
        self.check_magic(input)?;
        if input.len() < 8 {
            return Err(Error::parse("identity stream too short".to_string()));
        }
        let len = u32::from_be_bytes([input[4], input[5], input[6], input[7]]) as usize;
        let body = &input[8..];
        if body.len() < len {
            return Err(Error::parse(format!(
                "identity stream truncated: declared {len}, have {}",
                body.len()
            )));
        }
        Ok(body[..len].to_vec())
    }
}

/// Macro-free concrete codecs whose decode path is the modeled OS boundary.
macro_rules! boundary_codec {
    ($name:ident, $codec:expr, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, Default)]
        pub struct $name;

        impl Decompressor for $name {
            fn codec(&self) -> Codec {
                $codec
            }

            fn decompress(&self, input: &[u8]) -> crate::Result<Vec<u8>> {
                // Validate framing first so callers get precise errors; the
                // actual entropy decode is the OS boundary we don't implement.
                self.check_magic(input)?;
                Err(Error::unsupported(format!(
                    "{} decode requires a native decoder (OS boundary)",
                    self.codec().name()
                )))
            }
        }
    };
}

boundary_codec!(GzipCodec, Codec::Gzip, "gzip decompressor (boundary).");
boundary_codec!(XzCodec, Codec::Xz, "xz decompressor (boundary).");
boundary_codec!(ZstdCodec, Codec::Zstd, "zstd decompressor (boundary).");

/// Return a boxed [`Decompressor`] for a detected codec.
pub fn decompressor_for(codec: Codec) -> Box<dyn Decompressor> {
    match codec {
        Codec::Identity => Box::new(IdentityCodec),
        Codec::Gzip => Box::new(GzipCodec),
        Codec::Xz => Box::new(XzCodec),
        Codec::Zstd => Box::new(ZstdCodec),
    }
}

/// Detect the codec of `data` and decompress it, dispatching to the right
/// [`Decompressor`]. Returns [`Error::Unsupported`] when the detected codec's
/// decode path is an unimplemented OS boundary, and [`Error::Parse`] when no
/// codec magic matches.
pub fn decompress_auto(data: &[u8]) -> crate::Result<Vec<u8>> {
    match Codec::detect(data) {
        Some(codec) => decompressor_for(codec).decompress(data),
        None => Err(Error::parse("unrecognized compression magic".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_known_magics() {
        assert_eq!(Codec::detect(&[0x1f, 0x8b, 0x08]), Some(Codec::Gzip));
        assert_eq!(
            Codec::detect(&[0xfd, b'7', b'z', b'X', b'Z', 0x00, 0x00]),
            Some(Codec::Xz)
        );
        assert_eq!(
            Codec::detect(&[0x28, 0xb5, 0x2f, 0xfd, 0x00]),
            Some(Codec::Zstd)
        );
    }

    #[test]
    fn detect_unknown_is_none() {
        assert_eq!(Codec::detect(b"not compressed"), None);
        assert_eq!(Codec::detect(&[]), None);
    }

    #[test]
    fn identity_roundtrip() {
        let codec = IdentityCodec;
        let payload = b"some bytes to round trip".to_vec();
        let blob = codec.compress(&payload);
        assert_eq!(Codec::detect(&blob), Some(Codec::Identity));
        assert_eq!(codec.decompress(&blob).unwrap(), payload);
    }

    #[test]
    fn identity_truncated_detected() {
        let codec = IdentityCodec;
        let mut blob = codec.compress(b"hello");
        blob.truncate(blob.len() - 2);
        let err = codec.decompress(&blob).unwrap_err();
        assert_eq!(err.kind(), "parse");
    }

    #[test]
    fn boundary_codecs_validate_then_report_unsupported() {
        let gz = GzipCodec;
        // Wrong magic -> parse error.
        assert_eq!(gz.decompress(b"xx").unwrap_err().kind(), "parse");
        // Correct magic -> unsupported (boundary).
        let framed = [0x1f, 0x8b, 0x08, 0x00, 0x00];
        assert_eq!(gz.decompress(&framed).unwrap_err().kind(), "unsupported");
    }

    #[test]
    fn decompress_auto_dispatches() {
        let blob = IdentityCodec.compress(b"payload");
        assert_eq!(decompress_auto(&blob).unwrap(), b"payload");

        let xz = [0xfd, b'7', b'z', b'X', b'Z', 0x00, 0x01];
        assert_eq!(decompress_auto(&xz).unwrap_err().kind(), "unsupported");

        assert_eq!(decompress_auto(b"plain").unwrap_err().kind(), "parse");
    }

    #[test]
    fn codec_names_stable() {
        assert_eq!(Codec::Gzip.name(), "gzip");
        assert_eq!(Codec::Xz.name(), "xz");
        assert_eq!(Codec::Zstd.name(), "zstd");
        assert_eq!(Codec::Identity.name(), "identity");
    }

    #[test]
    fn decompressor_for_returns_matching_codec() {
        assert_eq!(decompressor_for(Codec::Zstd).codec(), Codec::Zstd);
        assert_eq!(decompressor_for(Codec::Gzip).codec(), Codec::Gzip);
    }
}
