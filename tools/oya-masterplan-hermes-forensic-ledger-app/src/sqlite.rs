//! Owned pure-Rust, read-only SQLite file-format reader.
//!
//! Scope is intentionally minimal: enough of the SQLite 3 on-disk format
//! (https://www.sqlite.org/fileformat2.html) to walk a table b-tree and decode
//! records, so the Hermes kanban board can be forensically ingested without any
//! shell/python/sqlite3 dependency. Write paths are deliberately absent.

/// A decoded SQLite record value.
#[derive(Debug, Clone, PartialEq)]
pub enum SqlValue {
    Null,
    Int(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl SqlValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            SqlValue::Text(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            SqlValue::Int(i) => Some(*i),
            _ => None,
        }
    }
}

/// Read-only view over a fully-loaded SQLite database image.
pub struct SqliteDb {
    bytes: Vec<u8>,
    page_size: usize,
    usable: usize,
}

const MAGIC: &[u8; 16] = b"SQLite format 3\0";
const PAGE_TYPE_TABLE_INTERIOR: u8 = 5;
const PAGE_TYPE_TABLE_LEAF: u8 = 13;

fn be16(buf: &[u8], off: usize) -> Result<usize, String> {
    let hi = *buf.get(off).ok_or("be16 out of bounds")? as usize;
    let lo = *buf.get(off + 1).ok_or("be16 out of bounds")? as usize;
    Ok((hi << 8) | lo)
}

fn be32(buf: &[u8], off: usize) -> Result<usize, String> {
    let mut v = 0usize;
    for i in 0..4 {
        v = (v << 8) | *buf.get(off + i).ok_or("be32 out of bounds")? as usize;
    }
    Ok(v)
}

/// SQLite varint: 1-9 bytes, big-endian 7-bit groups; the 9th byte carries 8 bits.
fn varint(buf: &[u8], mut off: usize) -> Result<(i64, usize), String> {
    let mut result: i64 = 0;
    for _ in 0..8 {
        let b = *buf.get(off).ok_or("varint out of bounds")?;
        off += 1;
        result = (result << 7) | i64::from(b & 0x7f);
        if b & 0x80 == 0 {
            return Ok((result, off));
        }
    }
    let b = *buf.get(off).ok_or("varint out of bounds")?;
    off += 1;
    result = (result << 8) | i64::from(b);
    Ok((result, off))
}

impl SqliteDb {
    /// Open a database image. Rejects non-SQLite bytes, non-UTF-8 text
    /// encodings, and unsupported page geometry. A leftover WAL is NOT read:
    /// callers must hand this a checkpointed snapshot (the forensic snapshot
    /// discipline records the snapshot sha256 alongside every extraction).
    pub fn open(bytes: Vec<u8>) -> Result<Self, String> {
        if bytes.len() < 100 {
            return Err("file too small for an SQLite header".into());
        }
        if &bytes[0..16] != MAGIC {
            return Err("not an SQLite 3 database (bad magic)".into());
        }
        let raw = ((bytes[16] as usize) << 8) | bytes[17] as usize;
        let page_size = if raw == 1 { 65536 } else { raw };
        if !page_size.is_power_of_two() || !(512..=65536).contains(&page_size) {
            return Err(format!("unsupported page size {page_size}"));
        }
        let reserved = bytes[20] as usize;
        let usable = page_size
            .checked_sub(reserved)
            .filter(|u| *u >= 480)
            .ok_or("reserved region leaves no usable page space")?;
        let encoding = be32(&bytes, 56)?;
        if encoding != 1 {
            return Err(format!(
                "only UTF-8 text encoding supported, got {encoding}"
            ));
        }
        Ok(Self {
            bytes,
            page_size,
            usable,
        })
    }

    fn page_count(&self) -> usize {
        self.bytes.len() / self.page_size
    }

    fn page(&self, number: usize) -> Result<&[u8], String> {
        if number == 0 || number > self.page_count() {
            return Err(format!("page {number} out of range"));
        }
        let start = (number - 1) * self.page_size;
        Ok(&self.bytes[start..start + self.page_size])
    }

    /// Locate a table's root page and CREATE statement in sqlite_schema.
    pub fn table_schema(&self, table: &str) -> Result<(usize, String), String> {
        let rows = self.walk_table_btree(1)?;
        for row in rows {
            let is_table = row.first().and_then(SqlValue::as_str) == Some("table");
            let name_matches = row.get(1).and_then(SqlValue::as_str) == Some(table);
            if is_table && name_matches {
                let root = row
                    .get(3)
                    .and_then(SqlValue::as_int)
                    .ok_or("schema row missing rootpage")?;
                let sql = row
                    .get(4)
                    .and_then(SqlValue::as_str)
                    .ok_or("schema row missing sql")?
                    .to_owned();
                return Ok((usize::try_from(root).map_err(|_| "bad rootpage")?, sql));
            }
        }
        Err(format!("table {table} not found in sqlite_schema"))
    }

    /// Read a whole table: declared column names plus decoded rows. Rows written
    /// before later ALTER TABLE ADD COLUMN migrations are shorter than the
    /// declared column list; missing trailing columns decode as Null.
    pub fn read_table(&self, table: &str) -> Result<(Vec<String>, Vec<Vec<SqlValue>>), String> {
        let (root, sql) = self.table_schema(table)?;
        let columns = parse_create_table_columns(&sql)?;
        let mut rows = self.walk_table_btree(root)?;
        for row in &mut rows {
            while row.len() < columns.len() {
                row.push(SqlValue::Null);
            }
        }
        Ok((columns, rows))
    }

    fn walk_table_btree(&self, root: usize) -> Result<Vec<Vec<SqlValue>>, String> {
        let mut rows = Vec::new();
        let mut stack = vec![root];
        let mut visited = 0usize;
        while let Some(page_no) = stack.pop() {
            visited += 1;
            if visited > self.page_count() {
                return Err("b-tree walk exceeded page count (cycle?)".into());
            }
            let page = self.page(page_no)?;
            let header_off = if page_no == 1 { 100 } else { 0 };
            let page_type = page[header_off];
            let cell_count = be16(page, header_off + 3)?;
            match page_type {
                PAGE_TYPE_TABLE_INTERIOR => {
                    stack.push(be32(page, header_off + 8)?);
                    for i in 0..cell_count {
                        let cell = be16(page, header_off + 12 + 2 * i)?;
                        stack.push(be32(page, cell)?);
                    }
                }
                PAGE_TYPE_TABLE_LEAF => {
                    for i in 0..cell_count {
                        let cell = be16(page, header_off + 8 + 2 * i)?;
                        let payload = self.leaf_cell_payload(page, cell)?;
                        rows.push(parse_record(&payload)?);
                    }
                }
                other => {
                    return Err(format!(
                        "unexpected b-tree page type {other} on page {page_no}"
                    ));
                }
            }
        }
        Ok(rows)
    }

    /// Decode one table-leaf cell, reassembling overflow chains.
    fn leaf_cell_payload(&self, page: &[u8], cell: usize) -> Result<Vec<u8>, String> {
        let (payload_len, off) = varint(page, cell)?;
        let (_rowid, off) = varint(page, off)?;
        let p = usize::try_from(payload_len).map_err(|_| "negative payload length")?;
        let u = self.usable;
        let x = u - 35;
        if p <= x {
            return page
                .get(off..off + p)
                .map(<[u8]>::to_vec)
                .ok_or_else(|| "inline payload out of bounds".into());
        }
        let m = ((u - 12) * 32 / 255) - 23;
        let k = m + (p - m) % (u - 4);
        let inline = if k <= x { k } else { m };
        let mut buf = page
            .get(off..off + inline)
            .map(<[u8]>::to_vec)
            .ok_or("inline payload out of bounds")?;
        let mut next = be32(page, off + inline)?;
        let mut hops = 0usize;
        while next != 0 && buf.len() < p {
            hops += 1;
            if hops > self.page_count() {
                return Err("overflow chain exceeded page count (cycle?)".into());
            }
            let overflow = self.page(next)?;
            next = be32(overflow, 0)?;
            let take = (p - buf.len()).min(u - 4);
            buf.extend_from_slice(&overflow[4..4 + take]);
        }
        if buf.len() != p {
            return Err(format!(
                "overflow reassembly produced {} of {p} bytes",
                buf.len()
            ));
        }
        Ok(buf)
    }
}

/// Decode the SQLite record format: header of serial types, then values.
fn parse_record(payload: &[u8]) -> Result<Vec<SqlValue>, String> {
    let (header_len, mut off) = varint(payload, 0)?;
    let header_end = usize::try_from(header_len).map_err(|_| "bad record header length")?;
    if header_end > payload.len() {
        return Err("record header exceeds payload".into());
    }
    let mut serials = Vec::new();
    while off < header_end {
        let (serial, next) = varint(payload, off)?;
        serials.push(serial);
        off = next;
    }
    let mut body = header_end;
    let mut values = Vec::with_capacity(serials.len());
    for serial in serials {
        let (value, size) = decode_serial(payload, body, serial)?;
        values.push(value);
        body += size;
    }
    Ok(values)
}

fn read_be_int(payload: &[u8], off: usize, bytes: usize) -> Result<i64, String> {
    let slice = payload
        .get(off..off + bytes)
        .ok_or("integer value out of bounds")?;
    let mut v: i64 = if slice[0] & 0x80 != 0 { -1 } else { 0 };
    for b in slice {
        v = (v << 8) | i64::from(*b);
    }
    Ok(v)
}

fn decode_serial(payload: &[u8], off: usize, serial: i64) -> Result<(SqlValue, usize), String> {
    match serial {
        0 => Ok((SqlValue::Null, 0)),
        1 => Ok((SqlValue::Int(read_be_int(payload, off, 1)?), 1)),
        2 => Ok((SqlValue::Int(read_be_int(payload, off, 2)?), 2)),
        3 => Ok((SqlValue::Int(read_be_int(payload, off, 3)?), 3)),
        4 => Ok((SqlValue::Int(read_be_int(payload, off, 4)?), 4)),
        5 => Ok((SqlValue::Int(read_be_int(payload, off, 6)?), 6)),
        6 => Ok((SqlValue::Int(read_be_int(payload, off, 8)?), 8)),
        7 => {
            let raw = read_be_int(payload, off, 8)?;
            #[allow(clippy::cast_sign_loss)]
            Ok((SqlValue::Real(f64::from_bits(raw as u64)), 8))
        }
        8 => Ok((SqlValue::Int(0), 0)),
        9 => Ok((SqlValue::Int(1), 0)),
        10 | 11 => Err("reserved serial type in record".into()),
        n if n >= 12 && n % 2 == 0 => {
            let len = usize::try_from((n - 12) / 2).map_err(|_| "bad blob length")?;
            let slice = payload.get(off..off + len).ok_or("blob out of bounds")?;
            Ok((SqlValue::Blob(slice.to_vec()), len))
        }
        n if n >= 13 => {
            let len = usize::try_from((n - 13) / 2).map_err(|_| "bad text length")?;
            let slice = payload.get(off..off + len).ok_or("text out of bounds")?;
            Ok((
                SqlValue::Text(String::from_utf8_lossy(slice).into_owned()),
                len,
            ))
        }
        other => Err(format!("unknown serial type {other}")),
    }
}

/// Extract declared column names, in order, from a CREATE TABLE statement.
/// Handles `--` line comments, quoted identifiers, and skips table-level
/// constraint clauses (PRIMARY KEY (...), UNIQUE, CHECK, FOREIGN KEY,
/// CONSTRAINT).
pub fn parse_create_table_columns(sql: &str) -> Result<Vec<String>, String> {
    let mut stripped = String::with_capacity(sql.len());
    let mut chars = sql.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '-' && chars.peek() == Some(&'-') {
            for skipped in chars.by_ref() {
                if skipped == '\n' {
                    stripped.push('\n');
                    break;
                }
            }
        } else {
            stripped.push(c);
        }
    }
    let open = stripped
        .find('(')
        .ok_or("CREATE TABLE without column list")?;
    let body = &stripped[open + 1..];
    let mut depth = 0usize;
    let mut in_quote: Option<char> = None;
    let mut segment = String::new();
    let mut segments = Vec::new();
    let mut closed = false;
    for c in body.chars() {
        if let Some(q) = in_quote {
            segment.push(c);
            if c == q {
                in_quote = None;
            }
            continue;
        }
        match c {
            '\'' | '"' | '`' => {
                in_quote = Some(c);
                segment.push(c);
            }
            '(' => {
                depth += 1;
                segment.push(c);
            }
            ')' => {
                if depth == 0 {
                    closed = true;
                    break;
                }
                depth -= 1;
                segment.push(c);
            }
            ',' if depth == 0 => {
                segments.push(std::mem::take(&mut segment));
            }
            _ => segment.push(c),
        }
    }
    if !closed {
        return Err("unterminated CREATE TABLE column list".into());
    }
    if !segment.trim().is_empty() {
        segments.push(segment);
    }
    let mut columns = Vec::new();
    for segment in segments {
        let trimmed = segment.trim();
        if trimmed.is_empty() {
            continue;
        }
        let first = first_identifier(trimmed);
        let keyword = first.to_ascii_uppercase();
        if matches!(
            keyword.as_str(),
            "PRIMARY" | "UNIQUE" | "CHECK" | "FOREIGN" | "CONSTRAINT"
        ) {
            continue;
        }
        columns.push(first);
    }
    if columns.is_empty() {
        return Err("CREATE TABLE declared no columns".into());
    }
    Ok(columns)
}

fn first_identifier(segment: &str) -> String {
    let mut chars = segment.chars();
    match chars.next() {
        Some(q @ ('"' | '`' | '\'')) => chars.take_while(|c| *c != q).collect(),
        Some('[') => chars.take_while(|c| *c != ']').collect(),
        Some(first) => {
            let mut ident = String::new();
            ident.push(first);
            ident.extend(chars.take_while(|c| c.is_ascii_alphanumeric() || *c == '_'));
            ident
        }
        None => String::new(),
    }
}
