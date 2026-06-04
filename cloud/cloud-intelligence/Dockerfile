# oya-cloud-intelligence — Dockerfile compatibility wrapper, not build authority.
#
# 2026-06-02 Buck2 authority amendment: CI/CD/scripts/build lanes build this
# service image through Buck2, not Cargo or cargo-chef. Canonical image target:
#
#   buck2 build root//cloud/cloud-intelligence/iac/oci:cloud-intelligence-oci
#
# This Dockerfile remains only for external runtime-image smoke tooling that can
# provide a Buck2-produced binary in the Docker build context. It does not compile
# Rust and it does not satisfy branch-protection or Phase-0 exit authority.

FROM gcr.io/distroless/static-debian12:nonroot
ARG OYA_BUCK2_BINARY=dist/cloud-intelligence
COPY ${OYA_BUCK2_BINARY} /usr/local/bin/cloud-intelligence
USER 65532:65532
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/cloud-intelligence"]
