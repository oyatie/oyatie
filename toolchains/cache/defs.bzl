# Cache-only execution platform for NativeLink CAS-backed warm-substrate.
#
# This platform preserves the host/os constraints from prelude default and only
# alters executor knobs required for cache-first behavior:
# - local_enabled=true (keep action execution local by default)
# - remote_enabled=false (Remote Execution stays staged behind cache; cache-only first)
# - remote_cache_enabled / allow_cache_uploads controlled via [oya_cache] root overlay
#
# Why it is isolated: this keeps root `.buckconfig` unchanged for local builds.
# CI can opt in safely by writing `.buckconfig.local` with:
#   [build]
#   execution_platforms = toolchains//cache:cache-platform
#   [build]
#   + this platform rule's flags via [oya_cache]
load("@prelude//cfg/exec_platform:marker.bzl", "get_exec_platform_marker")


def _cache_execution_platform_impl(ctx: AnalysisContext) -> list[Provider]:
    constraints = dict()
    constraints.update(ctx.attrs.cpu_configuration[ConfigurationInfo].constraints)
    constraints.update(ctx.attrs.os_configuration[ConfigurationInfo].constraints)
    cfg = ConfigurationInfo(constraints = constraints, values = {})

    name = ctx.label.raw_target()
    platform = ExecutionPlatformInfo(
        label = name,
        configuration = cfg,
        executor_config = CommandExecutorConfig(
            local_enabled = True,
            # Cache-only stage: local execution remains authoritative while remote cache
            # absorbs repeated work across runners. Remote execution is a separate, later
            # control-plane stage.
            remote_enabled = False,
            remote_cache_enabled = ctx.attrs.remote_cache_enabled,
            allow_cache_uploads = ctx.attrs.allow_cache_uploads,
            use_windows_path_separators = ctx.attrs.use_windows_path_separators,
        ),
    )

    return [
        DefaultInfo(),
        platform,
        PlatformInfo(label = str(name), configuration = cfg),
        ExecutionPlatformRegistrationInfo(
            platforms = [platform],
            exec_marker_constraint = get_exec_platform_marker(),
        ),
    ]


cache_execution_platform = rule(
    impl = _cache_execution_platform_impl,
    attrs = {
        "allow_cache_uploads": attrs.bool(default = False),
        "cpu_configuration": attrs.dep(providers = [ConfigurationInfo]),
        "os_configuration": attrs.dep(providers = [ConfigurationInfo]),
        "remote_cache_enabled": attrs.bool(default = False),
        "use_windows_path_separators": attrs.bool(default = False),
    },
)
