// Stand-ins for the upstream-internal types the pinned body_structure suite
// constructs. Every rendering decision is delegated to the server's FETCH
// writers through `mail_protocol_imap::fetch_render`; this file only holds
// the upstream wire framing (`imap_proto`) the suite compares byte-for-byte.
use mail_protocol_imap::fetch_render;

/// Error stand-in for the upstream `Result<_, ()>` surfaces.
#[derive(Debug)]
pub struct Refused;

pub fn build_metadata_contents(_: mail_parser::Message<'_>) {}

pub struct MessageMetadata {
    pub preview: String,
    pub raw_headers: Vec<u8>,
    pub blob_hash: (),
    pub blob_body_offset: u32,
    pub contents: (),
    pub rcvd_attach: u32,
}
#[derive(Clone, Copy)]
pub struct Native(u32);
impl Native {
    pub fn to_native(self) -> u32 {
        self.0
    }
}
pub struct Archived {
    pub raw_headers: Vec<u8>,
    pub blob_body_offset: Native,
}
pub trait Serialize {
    fn serialize(self) -> Result<MessageMetadata, Refused>;
}
pub trait Deserialize: Sized {
    fn deserialize_owned(value: MessageMetadata) -> Result<Self, Refused>;
}
pub struct Archiver(MessageMetadata);
impl Archiver {
    pub fn new(metadata: MessageMetadata) -> Self {
        Self(metadata)
    }
}
impl Serialize for Archiver {
    fn serialize(self) -> Result<MessageMetadata, Refused> {
        Ok(self.0)
    }
}
pub struct Archive(Archived);
impl Deserialize for Archive {
    fn deserialize_owned(value: MessageMetadata) -> Result<Self, Refused> {
        Ok(Self(Archived {
            raw_headers: value.raw_headers,
            blob_body_offset: Native(value.blob_body_offset),
        }))
    }
}
impl Archive {
    pub fn unarchive<T>(&self) -> Result<&Archived, Refused> {
        Ok(&self.0)
    }
}
pub struct ChainedBytes(Vec<u8>);
impl ChainedBytes {
    pub fn new(first: &[u8]) -> Self {
        Self(first.to_vec())
    }
    pub fn with_last(mut self, last: &[u8]) -> Self {
        self.0.extend_from_slice(last);
        self
    }
}
/// The reassembled raw message the server renders from.
pub struct Decoded(Vec<u8>);
impl Archived {
    pub fn decode_contents(&self, raw: ChainedBytes) -> Decoded {
        Decoded(raw.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Section {
    Part { num: u32 },
    Header,
    HeaderFields { not: bool, fields: Vec<String> },
    Text,
    Mime,
}
impl Section {
    fn serialize(&self, buf: &mut Vec<u8>) {
        match self {
            Section::Part { num } => buf.extend_from_slice(num.to_string().as_bytes()),
            Section::Header => buf.extend_from_slice(b"HEADER"),
            Section::Text => buf.extend_from_slice(b"TEXT"),
            Section::Mime => buf.extend_from_slice(b"MIME"),
            Section::HeaderFields { not, fields } => {
                buf.extend_from_slice(if *not {
                    b"HEADER.FIELDS.NOT (".as_slice()
                } else {
                    b"HEADER.FIELDS (".as_slice()
                });
                for (pos, field) in fields.iter().enumerate() {
                    if pos > 0 {
                        buf.push(b' ');
                    }
                    buf.extend_from_slice(field.to_ascii_uppercase().as_bytes());
                }
                buf.push(b')');
            }
        }
    }
}
impl std::fmt::Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = Vec::new();
        self.serialize(&mut buf);
        f.write_str(&String::from_utf8_lossy(&buf))
    }
}
fn spec(sections: &[Section]) -> String {
    let mut buf = Vec::new();
    for (pos, section) in sections.iter().enumerate() {
        if pos > 0 {
            buf.push(b'.');
        }
        section.serialize(&mut buf);
    }
    String::from_utf8(buf).unwrap()
}
fn nums(sections: &[u32]) -> String {
    sections
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(".")
}
fn partial(range: Option<(u32, u32)>) -> String {
    range.map_or(String::new(), |(start, count)| format!("<{start}.{count}>"))
}

pub enum BodyContents {
    Text(std::borrow::Cow<'static, str>),
    Bytes(Vec<u8>),
}
pub struct Structure<'a> {
    raw: &'a [u8],
}
impl Structure<'_> {
    pub fn serialize(&self, buf: &mut Vec<u8>, is_extended: bool, is_utf8: bool) {
        let rendered = fetch_render::structure(self.raw, is_extended, is_utf8)
            .unwrap_or_else(|error| panic!("BODYSTRUCTURE refused: {error}"));
        buf.extend_from_slice(&rendered);
    }
}
pub trait AsImapDataItem {
    fn body_structure<'a>(&self, decoded: &'a Decoded, is_extended: bool) -> Structure<'a>;
    fn body_section(
        &self,
        decoded: &Decoded,
        sections: &[Section],
        partial: Option<(u32, u32)>,
    ) -> Option<Vec<u8>>;
    fn binary(
        &self,
        decoded: &Decoded,
        sections: &[u32],
        partial: Option<(u32, u32)>,
    ) -> Result<Option<BodyContents>, Refused>;
    fn binary_size(&self, decoded: &Decoded, sections: &[u32]) -> Option<usize>;
}
impl AsImapDataItem for Archived {
    fn body_structure<'a>(&self, decoded: &'a Decoded, _: bool) -> Structure<'a> {
        Structure { raw: &decoded.0 }
    }
    fn body_section(
        &self,
        decoded: &Decoded,
        sections: &[Section],
        range: Option<(u32, u32)>,
    ) -> Option<Vec<u8>> {
        let item = format!("BODY[{}]{}", spec(sections), partial(range));
        fetch_render::section(&decoded.0, &item).ok().flatten()
    }
    fn binary(
        &self,
        decoded: &Decoded,
        sections: &[u32],
        range: Option<(u32, u32)>,
    ) -> Result<Option<BodyContents>, Refused> {
        let item = format!("BINARY[{}]{}", nums(sections), partial(range));
        match fetch_render::section(&decoded.0, &item) {
            Ok(value) => Ok(value.map(BodyContents::Bytes)),
            Err(error) if error.contains("UNKNOWN-CTE") => Err(Refused),
            Err(_) => Ok(None),
        }
    }
    fn binary_size(&self, decoded: &Decoded, sections: &[u32]) -> Option<usize> {
        let item = format!("BINARY.SIZE[{}]", nums(sections));
        fetch_render::section(&decoded.0, &item)
            .ok()
            .flatten()
            .map(|value| value.len())
    }
}

pub enum DataItem {
    Binary {
        sections: Vec<u32>,
        offset: Option<u32>,
        contents: BodyContents,
    },
    BinarySize {
        sections: Vec<u32>,
        size: usize,
    },
    BodySection {
        sections: Vec<Section>,
        origin_octet: Option<u32>,
        contents: Vec<u8>,
    },
}
fn literal(buf: &mut Vec<u8>, bytes: &[u8]) {
    buf.extend_from_slice(format!("{{{}}}\r\n", bytes.len()).as_bytes());
    buf.extend_from_slice(bytes);
}
impl DataItem {
    pub fn serialize(&self, buf: &mut Vec<u8>, _: bool) {
        match self {
            DataItem::Binary {
                sections,
                offset,
                contents,
            } => {
                buf.extend_from_slice(format!("BINARY[{}]", nums(sections)).as_bytes());
                buf.extend_from_slice(offset.map_or(" ".into(), |o| format!("<{o}> ")).as_bytes());
                match contents {
                    BodyContents::Text(text) => literal(buf, text.as_bytes()),
                    BodyContents::Bytes(bytes) => {
                        buf.extend_from_slice(format!("~{{{}}}\r\n", bytes.len()).as_bytes());
                        buf.extend_from_slice(bytes);
                    }
                }
            }
            DataItem::BinarySize { sections, size } => {
                buf.extend_from_slice(format!("BINARY.SIZE[{}] {size}", nums(sections)).as_bytes())
            }
            DataItem::BodySection {
                sections,
                origin_octet,
                contents,
            } => {
                buf.extend_from_slice(format!("BODY[{}]", spec(sections)).as_bytes());
                buf.extend_from_slice(
                    origin_octet
                        .map_or(" ".into(), |o| format!("<{o}> "))
                        .as_bytes(),
                );
                literal(buf, contents);
            }
        }
    }
}

pub enum ResponseCode {
    UnknownCte,
}
pub struct StatusResponse {
    message: String,
    code: Option<ResponseCode>,
}
impl StatusResponse {
    pub fn no(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: None,
        }
    }
    pub fn with_code(mut self, code: ResponseCode) -> Self {
        self.code = Some(code);
        self
    }
    pub fn serialize(self, mut buf: Vec<u8>) -> Vec<u8> {
        buf.extend_from_slice(b"* NO ");
        if let Some(ResponseCode::UnknownCte) = self.code {
            buf.extend_from_slice(b"[UNKNOWN-CTE] ");
        }
        buf.extend_from_slice(self.message.as_bytes());
        buf.extend_from_slice(b"\r\n");
        buf
    }
}
