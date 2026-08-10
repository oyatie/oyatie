use port_engine_api::{LanguagePair, w0_ready};

#[test]
fn slice2_seam_types_are_ready() {
    assert!(w0_ready());
    let pair = LanguagePair {
        source: "go".into(),
        target: "rust".into(),
    };
    assert_eq!(pair.slug().unwrap(), "go-rust");
}
