//! SHA-256 (FIPS 180-4) over plain `std`, pinned to the published NIST
//! test vectors.
//!
//! Why hand-rolled: [`crate::ErasureReceipt::merkle_leaf`] is fixed at a
//! 32-byte digest and this capability's lockfile is frozen, so no vetted
//! hash crate can be added. See the "Gaps" paragraph in `lib.rs` — this
//! module is a deliberate stopgap, not a claim of cryptographic pedigree.
//!
//! The implementation is the textbook one: 64-byte blocks, big-endian
//! length padding, the FIPS 180-4 constant tables. It carries no `unsafe`,
//! no `as` casts and no panicking helpers; every byte-to-word and
//! word-to-byte move goes through `from_be_bytes` / `to_be_bytes`.

/// SHA-256 block size in bytes.
const BLOCK_BYTES: usize = 64;

/// Byte offset inside a block at which the 64-bit length field starts.
const LENGTH_OFFSET: usize = 56;

/// FIPS 180-4 §5.3.3 initial hash value H(0).
const INITIAL_STATE: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];

/// FIPS 180-4 §4.2.2 round constants K(0..63).
const ROUND_CONSTANTS: [u32; 64] = [
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

/// A streaming SHA-256 hasher.
///
/// Feed bytes with [`Sha256::update`] and read the digest with
/// [`Sha256::finalize`]. The hasher is byte-oriented so message lengths
/// need no width conversion.
#[derive(Clone, Debug)]
pub struct Sha256 {
    state: [u32; 8],
    block: [u8; BLOCK_BYTES],
    filled: usize,
    /// Message length in bytes, counted one byte at a time (mod 2^64, as
    /// FIPS 180-4 specifies for the padded length field).
    message_bytes: u64,
}

impl Default for Sha256 {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha256 {
    /// A hasher primed with the FIPS 180-4 initial hash value.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: INITIAL_STATE,
            block: [0_u8; BLOCK_BYTES],
            filled: 0,
            message_bytes: 0,
        }
    }

    /// Absorb `data` into the running digest.
    pub fn update(&mut self, data: &[u8]) {
        for byte in data {
            self.message_bytes = self.message_bytes.wrapping_add(1);
            self.push(*byte);
        }
    }

    /// Finish the message and return the 32-byte digest.
    #[must_use]
    pub fn finalize(mut self) -> [u8; 32] {
        // FIPS 180-4 §5.1.1: append 0x80, pad with zeros to the length
        // field, then the message length in BITS, big-endian.
        let bit_length = self.message_bytes.wrapping_mul(8);
        self.push(0x80);
        while self.filled != LENGTH_OFFSET {
            self.push(0x00);
        }
        for byte in bit_length.to_be_bytes() {
            self.push(byte);
        }

        let mut digest = [0_u8; 32];
        for (word, chunk) in self.state.iter().zip(digest.chunks_exact_mut(4)) {
            chunk.copy_from_slice(&word.to_be_bytes());
        }
        digest
    }

    /// Append one byte, compressing whenever a full block is buffered.
    fn push(&mut self, byte: u8) {
        if let Some(slot) = self.block.get_mut(self.filled) {
            *slot = byte;
        }
        self.filled += 1;
        if self.filled == BLOCK_BYTES {
            self.compress();
            self.filled = 0;
        }
    }

    /// FIPS 180-4 §6.2.2 compression over the buffered block.
    fn compress(&mut self) {
        let mut schedule = [0_u32; 64];
        for (word, chunk) in schedule.iter_mut().take(16).zip(self.block.chunks_exact(4)) {
            let mut bytes = [0_u8; 4];
            bytes.copy_from_slice(chunk);
            *word = u32::from_be_bytes(bytes);
        }
        for index in 16..64 {
            let s0 = {
                let w = schedule[index - 15];
                w.rotate_right(7) ^ w.rotate_right(18) ^ (w >> 3)
            };
            let s1 = {
                let w = schedule[index - 2];
                w.rotate_right(17) ^ w.rotate_right(19) ^ (w >> 10)
            };
            schedule[index] = schedule[index - 16]
                .wrapping_add(s0)
                .wrapping_add(schedule[index - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;
        for (word, constant) in schedule.iter().zip(ROUND_CONSTANTS.iter()) {
            let big_s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choose = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(big_s1)
                .wrapping_add(choose)
                .wrapping_add(*constant)
                .wrapping_add(*word);
            let big_s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = big_s0.wrapping_add(majority);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        for (slot, value) in self.state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = slot.wrapping_add(value);
        }
    }
}

/// One-shot SHA-256 over `data`.
#[must_use]
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize()
}

/// Lowercase hex rendering of a digest, for logs and certificates.
#[must_use]
pub fn to_hex(digest: &[u8; 32]) -> String {
    let mut out = String::with_capacity(64);
    for byte in digest {
        // `{:02x}` on a u8 cannot fail and cannot truncate.
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{Sha256, sha256, to_hex};

    /// NIST FIPS 180-4 / CAVP known-answer vectors. A hash that is not
    /// pinned to published vectors is a guess, not a hash.
    #[test]
    fn nist_vector_empty_message() {
        assert_eq!(
            to_hex(&sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn nist_vector_abc() {
        assert_eq!(
            to_hex(&sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn nist_vector_448_bit_multi_block() {
        let message = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        assert_eq!(
            to_hex(&sha256(message)),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn nist_vector_896_bit_multi_block() {
        let message = b"abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu";
        assert_eq!(
            to_hex(&sha256(message)),
            "cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1"
        );
    }

    #[test]
    fn nist_vector_one_million_a() {
        let mut hasher = Sha256::new();
        for _ in 0..1_000_000 {
            hasher.update(b"a");
        }
        assert_eq!(
            to_hex(&hasher.finalize()),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }

    #[test]
    fn streaming_matches_one_shot_across_block_boundaries() {
        // 55/56/57/63/64/65 bytes straddle every padding branch.
        for length in [0_usize, 1, 55, 56, 57, 63, 64, 65, 119, 128] {
            let message: Vec<u8> = (0..length)
                .map(|index| u8::try_from(index % 251).unwrap())
                .collect();
            let mut hasher = Sha256::new();
            for byte in &message {
                hasher.update(&[*byte]);
            }
            assert_eq!(
                hasher.finalize(),
                sha256(&message),
                "byte-at-a-time streaming diverged at length {length}"
            );
        }
    }

    #[test]
    fn single_bit_flip_changes_the_digest() {
        let left = sha256(b"tenant-erasure-receipt");
        let right = sha256(b"tenant-erasure-receipu");
        assert_ne!(left, right);
    }
}
