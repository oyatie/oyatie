# tools/oci/defs.bzl
#
# Bespoke buck2-native OCI image rule.
#
# WHY bespoke: the bundled prelude has no OCI/container rule (confirmed absent
# 2026-05-31).  This rule assembles an OCI Image Layout (OCI spec 1.0) from a
# base tarball + one application layer (a tar.gz produced by a genrule over the
# binary artifact), recomputes manifests/configs/index, and exposes the
# resulting directory tree as a DefaultInfo provider.
#
# Intended call-site pattern:
#
#   oci_image(
#       name = "controller-oci",
#       base = ":distroless-base",          # http_archive target → tarball
#       layers = [":controller-layer"],     # genrule targets → layer tar.gz
#       entrypoint = ["/usr/local/bin/oya-ci-controller"],
#       user = "65532:65532",               # ADR-0146 nonroot UID
#       exposed_ports = ["8081/tcp"],
#   )
#
# The rule delegates heavy lifting to the host-tool `oya-oci-assemble`
# (tools/oci/crates/oya-oci-assemble), which is a Rust binary built by buck2.
# On darwin the assembled layout is inspectable / analysable; the binary is
# Mach-O and not runnable on Linux.  Push + cosign-sign is handled by the
# separate push-and-sign.sh script (linux CI only).

def _oci_image_impl(ctx: AnalysisContext) -> list[Provider]:
    # Collect base tarball artifact (from http_archive or genrule).
    base_artifact = ctx.attrs.base[DefaultInfo].default_outputs[0]

    # Collect layer tar.gz artifacts.
    layer_artifacts = []
    for layer in ctx.attrs.layers:
        layer_artifacts += layer[DefaultInfo].default_outputs

    # Output directory for the assembled OCI layout.
    out_dir = ctx.actions.declare_output("oci-layout", dir = True)

    # Build the oya-oci-assemble command.
    assemble_tool = ctx.attrs._assemble_tool[RunInfo]

    cmd = cmd_args(
        assemble_tool,
        "--base", base_artifact,
        "--out", out_dir.as_output(),
    )
    for layer in layer_artifacts:
        cmd.add("--layer", layer)
    for ep in ctx.attrs.entrypoint:
        cmd.add("--entrypoint", ep)
    if ctx.attrs.user:
        cmd.add("--user", ctx.attrs.user)
    for port in ctx.attrs.exposed_ports:
        cmd.add("--port", port)
    if ctx.attrs.image_title:
        cmd.add("--title", ctx.attrs.image_title)

    ctx.actions.run(
        cmd,
        category = "oci_assemble",
        identifier = ctx.label.name,
        # Downloads happen locally; keep prefer_local = True so the daemon
        # does not try to ship the base tarball to a remote executor.
        prefer_local = True,
    )

    return [
        DefaultInfo(default_output = out_dir),
    ]

oci_image = rule(
    impl = _oci_image_impl,
    attrs = {
        # Base image: an http_archive (or genrule) that produces an OCI layout
        # tarball (index.json + blobs/sha256/…).
        "base": attrs.dep(providers = [DefaultInfo]),
        # Application layers: genrule targets that each produce a single
        # tar.gz file to be appended on top of the base.
        "layers": attrs.list(attrs.dep(providers = [DefaultInfo]), default = []),
        # OCI image entrypoint (CMD is not set; callers may add "cmd" later).
        "entrypoint": attrs.list(attrs.string(), default = []),
        # USER in the OCI config (default: distroless nonroot UID).
        "user": attrs.option(attrs.string(), default = None),
        # Exposed ports annotation (informational; not enforced at runtime).
        "exposed_ports": attrs.list(attrs.string(), default = []),
        # Optional human-readable title written into the OCI config labels.
        "image_title": attrs.option(attrs.string(), default = None),
        # Private: the oya-oci-assemble host tool injected by the rule.
        "_assemble_tool": attrs.default_only(
            attrs.exec_dep(
                providers = [RunInfo],
                default = "root//tools/oci:oya-oci-assemble",
            ),
        ),
    },
)
