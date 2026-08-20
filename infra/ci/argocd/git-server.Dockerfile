# Prebaked, hardened git daemon for the local CI-farm GitOps source.
#
# Why this exists: the previous manifest ran `apk add git git-daemon` at
# container *startup*, which is an anti-pattern — non-reproducible,
# network-dependent, root-requiring, and impossible to run under a
# read-only root filesystem (Trivy KSV-0014/KSV-0118). Installing at BUILD
# time instead lets the container run non-root with a read-only root fs.
#
# License posture (OSI-strict): alpine base (MIT/BSD userland) + git +
# git-daemon (GPL-2.0). No BSL/SSPL/source-available components.
#
# Build + load onto the local k3s farm:
#   docker build -t oya-git-server:local -f infra/ci/argocd/git-server.Dockerfile infra/ci/argocd
#   k3s ctr images import <(docker save oya-git-server:local)   # or push to the in-cluster registry
FROM alpine:3.20

RUN apk add --no-cache git git-daemon \
 && adduser -D -u 1000 -g git git \
 && mkdir -p /srv/git \
 && chown -R git:git /srv

USER 1000:1000
EXPOSE 9418

# Serve every repo under /srv/git read-only over git://. The repo is seeded
# into the shared volume by the seed-repo initContainer at pod start.
ENTRYPOINT ["git", "daemon", "--reuseaddr", "--export-all", "--base-path=/srv/git", "--verbose"]
