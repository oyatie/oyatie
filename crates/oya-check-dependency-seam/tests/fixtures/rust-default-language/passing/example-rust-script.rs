// Passing fixture for check_rust_default_language.
// Represents a scripts/ tree with ONLY .rs files — total_non_rust counter stays at 0.
// When the sub-check arms (F-LANE-RUST-DEFAULT-ENFORCE), running it against a
// workspace whose scripts/ equals this dir should produce status=Pass.
fn main() {
    println!("Rust-default-language compliant.");
}
