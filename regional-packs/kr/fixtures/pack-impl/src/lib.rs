// Regional-pack fixture logic: explicit KR-only controls stay in the pack layer.

pub const PACK_ID: &str = "oya-pack-kr";
pub const REGULATORY_CONTROLS: &[&str] = &["PIPA", "KISA", "CSAP"];

pub fn pack_scope() -> &'static str {
    "regional-packs/kr"
}
