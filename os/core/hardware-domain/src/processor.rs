//! Processor (CPU) information.
//!
//! Mirrors the Talos `Processor` COSI resource populated by the SMBIOS
//! controller from each Type 4 *Processor Information* structure (one per
//! socket). Talos surfaces, per socket: socket designation, manufacturer,
//! product/version, max/boot clock speeds, status, core/thread counts.
//!
//! Empty/disabled sockets are skipped, matching Talos which only emits a
//! `Processor` resource for populated, enabled sockets.

use crate::smbios::{Structure, StructureType};

/// Processor *status* (SMBIOS Type 4 offset 0x18, low 3 bits = CPU status,
/// bit 6 = socket populated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProcessorStatus {
    /// Socket is unpopulated.
    #[default]
    Unpopulated,
    /// Populated and enabled.
    Enabled,
    /// Populated but disabled by the user via BIOS setup.
    DisabledByUser,
    /// Populated but disabled by BIOS (POST error).
    DisabledByBios,
    /// Populated but idle.
    Idle,
    /// Populated, status reserved/unknown.
    Other,
}

impl ProcessorStatus {
    /// Decode the Type 4 status byte.
    pub fn from_status_byte(b: u8) -> Self {
        let populated = (b & 0x40) != 0;
        if !populated {
            return ProcessorStatus::Unpopulated;
        }
        match b & 0x07 {
            0 => ProcessorStatus::Other, // unknown
            1 => ProcessorStatus::Enabled,
            2 => ProcessorStatus::DisabledByUser,
            3 => ProcessorStatus::DisabledByBios,
            4 => ProcessorStatus::Idle,
            _ => ProcessorStatus::Other,
        }
    }

    /// Whether this socket is populated (has a physical CPU).
    pub fn is_populated(self) -> bool {
        !matches!(self, ProcessorStatus::Unpopulated)
    }

    /// Whether this socket is populated *and* enabled — the condition under
    /// which Talos emits a `Processor` resource.
    pub fn is_enabled(self) -> bool {
        matches!(self, ProcessorStatus::Enabled)
    }
}

/// A decoded processor, mirroring the Talos `Processor` resource spec.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Processor {
    /// Socket designation (e.g. `CPU0`, `CPU 1`).
    pub socket: String,
    /// Manufacturer string (e.g. `Intel(R) Corporation`).
    pub manufacturer: String,
    /// Product/version string (e.g. `Intel(R) Xeon(R) ...`).
    pub product_name: String,
    /// Maximum clock speed in MHz (offset 0x14).
    pub max_speed_mhz: u16,
    /// Boot/current clock speed in MHz (offset 0x16).
    pub boot_speed_mhz: u16,
    /// Number of physical cores (offset 0x23 or word at 0x2a).
    pub core_count: u16,
    /// Number of enabled cores.
    pub core_enabled: u16,
    /// Number of hardware threads (offset 0x25 or word at 0x2e).
    pub thread_count: u16,
    /// Decoded socket status.
    pub status: ProcessorStatus,
}

impl Processor {
    /// Decode a Type 4 structure into a [`Processor`].
    ///
    /// Returns `None` when the structure is not a Type 4 or the socket is
    /// unpopulated, exactly as Talos skips such sockets.
    pub fn decode(s: &Structure) -> Option<Processor> {
        if s.ty != StructureType::Processor {
            return None;
        }
        let status = ProcessorStatus::from_status_byte(s.byte(0x18).unwrap_or(0));
        if !status.is_populated() {
            return None;
        }

        let max_speed = s.word(0x14).unwrap_or(0);
        let boot_speed = s.word(0x16).unwrap_or(0);

        // Core/thread counts: 1-byte fields at 0x23/0x24/0x25, with 2-byte
        // overflow fields at 0x2a/0x2c/0x2e (used when the byte field == 0xff)
        // in SMBIOS 3.0+.
        let core_count = wide_count(s, 0x23, 0x2a);
        let core_enabled = wide_count(s, 0x24, 0x2c);
        let thread_count = wide_count(s, 0x25, 0x2e);

        Some(Processor {
            socket: s.string(0x04),
            manufacturer: s.string(0x07),
            product_name: s.string(0x10),
            max_speed_mhz: max_speed,
            boot_speed_mhz: boot_speed,
            core_count,
            core_enabled,
            thread_count,
            status,
        })
    }

    /// Whether this CPU exposes simultaneous multithreading (more threads than
    /// cores), e.g. Intel Hyper-Threading.
    pub fn has_smt(&self) -> bool {
        self.core_count > 0 && self.thread_count > self.core_count
    }
}

/// Resolve a possibly-wide SMBIOS count field: read the 1-byte field at
/// `byte_off`; if it is `0xff` (the "use the wide field" sentinel) and the
/// structure is long enough, read the 2-byte field at `word_off`.
fn wide_count(s: &Structure, byte_off: usize, word_off: usize) -> u16 {
    let narrow = s.byte(byte_off).unwrap_or(0);
    if narrow == 0xff
        && let Some(w) = s.word(word_off) {
            return w;
        }
    narrow as u16
}

/// Aggregate CPU topology summed across all populated, enabled sockets — the
/// numbers Talos folds into the node's status.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CpuTopology {
    /// Number of enabled physical sockets.
    pub sockets: usize,
    /// Total physical cores across sockets.
    pub total_cores: u32,
    /// Total hardware threads across sockets.
    pub total_threads: u32,
}

impl CpuTopology {
    /// Compute a topology summary from a list of decoded processors. Only
    /// enabled sockets contribute.
    pub fn from_processors(procs: &[Processor]) -> Self {
        let mut topo = CpuTopology::default();
        for p in procs.iter().filter(|p| p.status.is_enabled()) {
            topo.sockets += 1;
            topo.total_cores += p.core_count as u32;
            topo.total_threads += p.thread_count as u32;
        }
        topo
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smbios::{SmbiosBuilder, SmbiosTable, StructureType};

    /// Build a Type 4 processor structure. `formatted` is the area after the
    /// 4-byte header (so index 0 here == offset 0x04).
    fn cpu_structure(status: u8, max: u16, boot: u16, cores: u8, threads: u8) -> Structure {
        // Build a formatted area large enough to reach offset 0x25.
        let mut f = [0u8; 0x26]; // 0x00..0x25 inclusive
        f[0x04] = 1; // socket idx
        f[0x07] = 2; // manufacturer idx
        f[0x10] = 3; // version idx
        f[0x14] = (max & 0xff) as u8;
        f[0x15] = (max >> 8) as u8;
        f[0x16] = (boot & 0xff) as u8;
        f[0x17] = (boot >> 8) as u8;
        f[0x18] = status;
        f[0x23] = cores;
        f[0x24] = cores;
        f[0x25] = threads;

        let raw = SmbiosBuilder::new()
            .structure(
                StructureType::Processor,
                0x0400,
                &f[4..], // strip the synthetic header bytes; builder re-adds them
                &["CPU0", "Intel(R) Corporation", "Xeon Gold"],
            )
            .finish();
        let table = SmbiosTable::parse(&raw).unwrap();
        table.first(StructureType::Processor).unwrap().clone()
    }

    #[test]
    fn decodes_enabled_processor() {
        let s = cpu_structure(0x41, 3500, 2400, 8, 16);
        let p = Processor::decode(&s).unwrap();
        assert_eq!(p.socket, "CPU0");
        assert_eq!(p.manufacturer, "Intel(R) Corporation");
        assert_eq!(p.product_name, "Xeon Gold");
        assert_eq!(p.max_speed_mhz, 3500);
        assert_eq!(p.boot_speed_mhz, 2400);
        assert_eq!(p.core_count, 8);
        assert_eq!(p.thread_count, 16);
        assert!(p.status.is_enabled());
        assert!(p.has_smt());
    }

    #[test]
    fn unpopulated_socket_skipped() {
        // bit 6 clear => unpopulated.
        let s = cpu_structure(0x01, 0, 0, 0, 0);
        assert!(Processor::decode(&s).is_none());
    }

    #[test]
    fn disabled_socket_decodes_but_not_enabled() {
        // populated (0x40) + disabled-by-user (0x02).
        let s = cpu_structure(0x42, 3000, 3000, 4, 4);
        let p = Processor::decode(&s).unwrap();
        assert_eq!(p.status, ProcessorStatus::DisabledByUser);
        assert!(!p.status.is_enabled());
        assert!(!p.has_smt());
    }

    #[test]
    fn status_byte_decoding() {
        assert_eq!(
            ProcessorStatus::from_status_byte(0x00),
            ProcessorStatus::Unpopulated
        );
        assert_eq!(
            ProcessorStatus::from_status_byte(0x41),
            ProcessorStatus::Enabled
        );
        assert_eq!(
            ProcessorStatus::from_status_byte(0x42),
            ProcessorStatus::DisabledByUser
        );
        assert_eq!(
            ProcessorStatus::from_status_byte(0x43),
            ProcessorStatus::DisabledByBios
        );
        assert_eq!(
            ProcessorStatus::from_status_byte(0x44),
            ProcessorStatus::Idle
        );
    }

    #[test]
    fn topology_sums_only_enabled_sockets() {
        let procs = vec![
            Processor {
                core_count: 8,
                thread_count: 16,
                status: ProcessorStatus::Enabled,
                ..Default::default()
            },
            Processor {
                core_count: 8,
                thread_count: 16,
                status: ProcessorStatus::Enabled,
                ..Default::default()
            },
            Processor {
                core_count: 4,
                thread_count: 4,
                status: ProcessorStatus::DisabledByUser,
                ..Default::default()
            },
        ];
        let topo = CpuTopology::from_processors(&procs);
        assert_eq!(topo.sockets, 2);
        assert_eq!(topo.total_cores, 16);
        assert_eq!(topo.total_threads, 32);
    }
}
