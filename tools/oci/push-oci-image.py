#!/usr/bin/env python3
# tools/oci/push-oci-image.py
#
# oya-oci-push — stdlib-only OCI Image push client (no crane / skopeo / docker).
#
# Sibling of tools/oci/pull-oci-base.py.  Where the puller speaks the OCI
# distribution API to FETCH a base image, this pusher speaks it to UPLOAD a
# fully-assembled OCI Image Layout (the output of `buck2 build
# //...:<svc>-oci`, i.e. the oya-oci-assemble tree) to a registry.
#
# This retires the `crane push` dependency in push-and-sign.sh: the CI build
# pod ships python3 but not crane/skopeo/docker (ADR-0514/0515 bespoke OCI
# pipeline + bespoke-over-OSS + single-bootstrap doctrine).
#
# Scope: PUSH ONLY.  cosign signing (ADR-0181) stays a separate step keyed off
# the ESO-projected cosign key; the dev lane runs cosign.required=false.
#
# Push flow (OCI distribution-spec v2):
#   For the single image manifest in index.json (linux/arm64):
#     1. For config + each layer blob, in dependency order:
#          HEAD /v2/<repo>/blobs/<digest>           -> 200 = already present, skip
#          POST /v2/<repo>/blobs/uploads/           -> 202 + Location (upload URL)
#          PUT  <upload-url>?digest=<digest>        -> 201 (monolithic-after-POST)
#     2. PUT /v2/<repo>/manifests/<tag>             -> 201
#          Content-Type = the manifest's own mediaType
#   The manifest references config+layers by digest, so all blobs MUST be
#   uploaded before the manifest PUT (else the registry 400s on unknown blob).
#
# The registry blob/manifest digests are content-addressed, so re-pushing is
# idempotent (HEAD short-circuits existing blobs).  The pushed manifest digest
# (sha256 of the manifest bytes) is printed to stdout on success — feed it to
# the Helm values.yaml image.digest field / deployment patch.
#
# Usage:
#   push-oci-image.py <oci-layout-dir> <registry> <repository> <tag> [--insecure]
#
# Example (in-cluster registry, plain HTTP):
#   push-oci-image.py \
#     buck-out/.../__controller-oci__/oci-layout \
#     registry.oya-registry.svc.cluster.local:5000 \
#     oya-ci-controller dev --insecure
#
# --insecure  : use http:// instead of https:// (in-cluster registry on :5000).

import json
import os
import sys
import urllib.error
import urllib.request

UPLOAD_CHUNK = 1 << 20  # not used for streaming; PUT sends the whole blob body.


def _scheme(insecure: bool) -> str:
    return "http" if insecure else "https"


def _req(method: str, url: str, *, data=None, headers=None):
    """Issue an HTTP request; return (status, resp_headers, body_bytes).

    Raises on transport error; does NOT raise on HTTP status (caller branches
    on the returned status code).
    """
    req = urllib.request.Request(url, data=data, method=method)
    for k, v in (headers or {}).items():
        req.add_header(k, v)
    try:
        with urllib.request.urlopen(req) as resp:
            return resp.status, dict(resp.headers), resp.read()
    except urllib.error.HTTPError as e:
        return e.code, dict(e.headers), e.read()


def blob_present(base: str, repo: str, digest: str) -> bool:
    status, _, _ = _req("HEAD", f"{base}/v2/{repo}/blobs/{digest}")
    return status == 200


def push_blob(base: str, repo: str, digest: str, path: str) -> None:
    """Upload one blob (config or layer) by digest if not already present."""
    if blob_present(base, repo, digest):
        print(f"    blob {digest[:19]}... already present, skip", file=sys.stderr)
        return

    # 1) Open an upload session.
    status, hdrs, body = _req("POST", f"{base}/v2/{repo}/blobs/uploads/")
    if status not in (202, 201):
        raise RuntimeError(
            f"open upload for {digest} failed: HTTP {status}: {body[:200]!r}"
        )
    location = hdrs.get("Location") or hdrs.get("location")
    if not location:
        raise RuntimeError(f"upload POST for {digest} returned no Location header")
    # Location may be absolute or registry-relative.
    if location.startswith("/"):
        location = base + location

    # 2) Monolithic PUT with ?digest= to finalize.
    sep = "&" if "?" in location else "?"
    put_url = f"{location}{sep}digest={digest}"
    with open(path, "rb") as f:
        data = f.read()
    status, _, body = _req(
        "PUT",
        put_url,
        data=data,
        headers={
            "Content-Type": "application/octet-stream",
            "Content-Length": str(len(data)),
        },
    )
    if status not in (201, 202):
        raise RuntimeError(
            f"PUT blob {digest} failed: HTTP {status}: {body[:200]!r}"
        )
    print(f"    pushed blob {digest[:19]}... ({len(data)} bytes)", file=sys.stderr)


def main() -> int:
    args = [a for a in sys.argv[1:] if a != "--insecure"]
    insecure = "--insecure" in sys.argv[1:]
    if len(args) != 4:
        print(
            "usage: push-oci-image.py <oci-layout-dir> <registry> <repository> "
            "<tag> [--insecure]",
            file=sys.stderr,
        )
        return 2
    layout, registry, repo, tag = args
    base = f"{_scheme(insecure)}://{registry}"

    if not os.path.isfile(os.path.join(layout, "oci-layout")):
        print(
            f"ERROR: {layout} is not an OCI Image Layout (no oci-layout marker)",
            file=sys.stderr,
        )
        return 1

    # Resolve the single image manifest from index.json.
    index = json.load(open(os.path.join(layout, "index.json")))
    manifests = index.get("manifests", [])
    if not manifests:
        print("ERROR: index.json has no manifests", file=sys.stderr)
        return 1
    # The assembler emits exactly one (single-arch linux/arm64) manifest.
    mdesc = manifests[0]
    mhex = mdesc["digest"].split(":")[1]
    blobs_dir = os.path.join(layout, "blobs", "sha256")
    manifest_path = os.path.join(blobs_dir, mhex)
    manifest = json.load(open(manifest_path))

    print(f"==> Pushing {registry}/{repo}:{tag}", file=sys.stderr)

    # 1) Push config + every layer blob FIRST (manifest references them).
    blob_descs = [manifest["config"]] + list(manifest["layers"])
    for d in blob_descs:
        dg = d["digest"]
        push_blob(base, repo, dg, os.path.join(blobs_dir, dg.split(":")[1]))

    # 2) PUT the manifest by tag with its own mediaType.
    manifest_bytes = open(manifest_path, "rb").read()
    media_type = manifest.get(
        "mediaType", "application/vnd.oci.image.manifest.v1+json"
    )
    status, hdrs, body = _req(
        "PUT",
        f"{base}/v2/{repo}/manifests/{tag}",
        data=manifest_bytes,
        headers={"Content-Type": media_type},
    )
    if status not in (201, 202):
        print(
            f"ERROR: PUT manifest failed: HTTP {status}: {body[:300]!r}",
            file=sys.stderr,
        )
        return 1

    # The registry returns the canonical digest in Docker-Content-Digest;
    # fall back to the layout's own manifest digest (identical content).
    pushed_digest = (
        hdrs.get("Docker-Content-Digest")
        or hdrs.get("docker-content-digest")
        or mdesc["digest"]
    )
    print(f"==> Pushed manifest digest: {pushed_digest}", file=sys.stderr)
    # Machine-readable digest on stdout (for the Helm values / deploy patch).
    print(pushed_digest)
    return 0


if __name__ == "__main__":
    sys.exit(main())
