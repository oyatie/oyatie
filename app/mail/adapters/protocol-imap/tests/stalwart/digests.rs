use super::*;
pub fn verify() {
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!(env!("STALWART_THREAD_SOURCE")))
        ),
        "53f9247b7a047814900dc6cc7beb4f31f18eca8e31785ad5c8b9bcb01a905e6c"
    );
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!(env!("STALWART_MAILBOX_SOURCE")))
        ),
        "01e99a32d4d8d5ae21e7c8f3e2af8af44d9f5e1cb7c37bfe9dab794066e61a7b"
    );
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!(env!("STALWART_MUTATE_SOURCE")))
        ),
        "d3fa6608bb7edcde973317c7432c52b3fb6320dce814095beee5797a64e1ae3d"
    );
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!(env!("STALWART_CORE_SOURCE")))
        ),
        "80b64430137cd80efc6c73fda16a308a12f0c287bb3846940358e244e8ce79a3",
        "upstream assertions changed; review before updating baseline"
    );
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!(env!("STALWART_BINARY_SOURCE")))
        ),
        "2d0087d43ee49c338fd71fd34e856245195a2fa1ed2f5d139df8d9ea5d4888d8",
        "upstream binary suite changed"
    );
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!(env!("STALWART_INSPECT_SOURCE")))
        ),
        "8383a5409e2b7fb5bc8781f1b56a6c5de19e53623cad5ecac23b13f899f03025"
    );
}
