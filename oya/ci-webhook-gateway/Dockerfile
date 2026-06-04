# CI Webhook Gateway — Dockerfile compatibility wrapper, not build authority.
#
# 2026-06-02 Buck2 authority amendment: CI/CD/scripts/build lanes build this
# service image through Buck2, not Cargo. Canonical image target:
#
#   buck2 build root//oya/ci-webhook-gateway/iac/oci:gateway-oci \
#     --target-platforms //platforms:linux-aarch64-musl
#
# This Dockerfile remains only for external runtime-image smoke tooling that can
# provide a Buck2-produced musl/static binary in the Docker build context. It does
# not compile Rust and it does not satisfy branch-protection or Phase-0 exit
# authority.

FROM gcr.io/distroless/static-debian12:nonroot
ARG OYA_BUCK2_BINARY=dist/oya-ci-webhook-gateway
COPY ${OYA_BUCK2_BINARY} /usr/local/bin/oya-ci-webhook-gateway
EXPOSE 8099
USER 65532:65532
ENTRYPOINT ["/usr/local/bin/oya-ci-webhook-gateway"]
