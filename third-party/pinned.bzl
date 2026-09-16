load("@prelude//rust:cargo_buildscript.bzl", "buildscript_run")
load("@prelude//rust:cargo_package.bzl", "cargo")

def pinned_crate(name, version, sha256, deps = [], features = [], edition = "2021", proc_macro = False, build_script = False, build_deps = [], visibility = [], crate_root = "src/lib.rs", package = None, crate_name = None, named_deps = {}, build_root = "build.rs", build_env = {}, rustc_link_lib = False, rustc_link_search = False):
    package = package or name
    archive = name + "-source"
    native.http_archive(
        name = archive,
        urls = ["https://static.crates.io/crates/{}/{}/download".format(package, version)],
        sha256 = sha256,
        strip_prefix = package + "-" + version,
    )
    parts = version.split(".")
    env = {
        "CARGO_MANIFEST_DIR": archive,
        "CARGO_PKG_NAME": package,
        "CARGO_PKG_VERSION": version,
        "CARGO_PKG_VERSION_MAJOR": parts[0],
        "CARGO_PKG_VERSION_MINOR": parts[1],
        "CARGO_PKG_VERSION_PATCH": parts[2],
        "CARGO_PKG_VERSION_PRE": "",
    }
    flags = []
    if build_script:
        cargo.rust_binary(
            name = name + "-build",
            srcs = [":" + archive],
            crate = "build_script_build",
            crate_root = archive + "/" + build_root,
            edition = edition,
            env = env,
            features = features,
            deps = build_deps,
        )
        buildscript_run(
            name = name + "-build-run",
            package_name = package,
            buildscript_rule = ":" + name + "-build",
            manifest_dir = ":" + archive,
            env = dict(env, **build_env),
            features = features,
            version = version,
            rustc_link_lib = rustc_link_lib,
            rustc_link_search = rustc_link_search,
        )
        env = dict(env)
        env["OUT_DIR"] = "$(location :" + name + "-build-run[out_dir])"
        flags = ["@$(location :" + name + "-build-run[rustc_flags])"]
    cargo.rust_library(
        name = name,
        srcs = [":" + archive],
        crate = crate_name or package.replace("-", "_"),
        crate_root = archive + "/" + crate_root,
        edition = edition,
        env = env,
        features = features,
        rustc_flags = flags,
        proc_macro = proc_macro,
        deps = deps,
        named_deps = named_deps,
        visibility = visibility,
    )
