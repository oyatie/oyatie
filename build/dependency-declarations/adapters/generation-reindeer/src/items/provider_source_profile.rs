const REINDEER_PROVIDER_SOURCE_PATHS_V1: [&str; 11] = [
    "src/artifact.rs",
    "src/artifact/serializer.rs",
    "src/artifact/serializer/builders.rs",
    "src/artifact/value.rs",
    "src/buck.rs",
    "src/buckify.rs",
    "src/fixups.rs",
    "src/fixups/buildscript.rs",
    "src/index.rs",
    "src/main.rs",
    "src/version_naming.rs",
];
const REINDEER_PROVIDER_GENERATED_PATHS_V1: [&str; 4] = [
    "src/artifact.rs",
    "src/artifact/serializer.rs",
    "src/artifact/serializer/builders.rs",
    "src/artifact/value.rs",
];
const MAX_PROVIDER_OUTPUT_BYTES_V1: usize = 4 * 1024 * 1024;
const REINDEER_SOURCE_REPOSITORY_V1: &str = "https://github.com/facebookincubator/reindeer";
const REINDEER_SOURCE_TAG_V1: &str = "v2026.08.10.00";
const REINDEER_ADAPTATION_RECIPE_ID_V1: &str = concat!(
    "build.reindeer-provider-source-recipe.v1;",
    "syn=2.0.119@872831b642d1a07999a962a351ed35b955ea2cfc8f3862091e2a240a84f17297;",
    "prettyplease=0.2.37@479ca8adacdd7ce8f1fb39ce9ecccbfe93a3f1344b3d0d97f20bc0196208f62b;",
    "proc-macro2=1.0.107@985e7ec9bb745e6ce6535b544d84d6cd6f7ad8bd711c398938ae983b91a766d9;",
    "quote=1.0.47@1fbf4db142a473a8d80c26bbf18454ed458bf8d26c8219c331daecfdbd079001;",
    "sha2=0.10.9@a7507d819769d01a365ab707794a4084392c824f54a7a6a7862f8c3d0892b283;",
    "public-naming=resolved-compatibility-slot.v1;",
    "reserved-targets=source-distinct.v1;",
    "workspace-roots=non-workspace-public-targets.v1;",
    "workspace-dev-roots=dependency-closed.v1;",
    "container-length-hints=serde-starlark-0.1.19.v1;",
    "cxx-preprocessor-select=fixup-native.v1",
);

struct ReindeerParsedProviderSourceV1<'a> {
    bytes: &'a [u8],
    text: &'a str,
    syntax: syn::File,
}

const REINDEER_BUCK_SHA256_V1: &str =
    "49d79a30a880c042f3c383b6b5d17d3152caacbf82e402ab7d1875087e56237b";
const REINDEER_BUCKIFY_SHA256_V1: &str =
    "6d09d2b7a51b7fca101d2fbd356d96e626467a8b8b02090747eb3979d4f61ecf";
const REINDEER_FIXUPS_SHA256_V1: &str =
    "82063206f972bceb51416c7503fba83b99d0f54d6c543e9c711a18383adf4b3f";
const REINDEER_FIXUP_BUILDSCRIPT_SHA256_V1: &str =
    "62d3710023310b05e5ced3fad154861df74a4828d0fc79183a20d8ecf63ec1ee";
const REINDEER_INDEX_SHA256_V1: &str =
    "23546695e322a9d86298f6aeb38abbeff4e10503674fca251eb153917beb6689";
const REINDEER_MAIN_SHA256_V1: &str =
    "2b53f3680985fec0974441ad37b80397ca3cc85e52c259917af15277ec874a27";
const REINDEER_VERSION_NAMING_SHA256_V1: &str =
    "547603f2df2e163a12d719c290d94e14f56bdaa2451208f093f06d469aab0415";
