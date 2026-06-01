#!/usr/bin/env python3
# tools/oci/pull-oci-base.py
#
# Pull a distroless base image from an OCI distribution registry and write an
# OCI Image Layout directory that `oya-oci-assemble` (tools/oci/crates/
# oya-oci-assemble) consumes via find_oci_root().
#
# STDLIB-ONLY (no requests / no third-party deps) so it runs unmodified inside
# a buck2 genrule on the build host (darwin aarch64 or linux aarch64) with only
# the hermetic python3 toolchain available.
#
# Usage:
#   python3 pull-oci-base.py REGISTRY REPO DIGEST OUTDIR
#
#   REGISTRY   registry host, e.g. gcr.io
#   REPO       repository path, e.g. distroless/static-debian12
#   DIGEST     image manifest digest, e.g. sha256:82043e1c...  (immutable pin)
#   OUTDIR     output directory for the OCI Image Layout
#
# Output layout (OCI spec 1.0):
#   <OUTDIR>/
#     oci-layout                 {"imageLayoutVersion":"1.0.0"}
#     index.json                 one manifest entry → DIGEST, platform arm64/linux
#     blobs/
#       sha256/
#         <manifest-hex>         the image manifest bytes (sha256 == DIGEST)
#         <config-hex>           the image config blob
#         <layer-hex> ...        every layer blob
#
# The pulled DIGEST MUST reference a single-arch (linux/arm64) image manifest,
# not a multi-arch image index.  The distroless arm64 digest is pinned in the
# call-site BUCK genrule; bump it when Google publishes a CVE-patched rebuild.
#
# Auth: anonymous bearer token from the registry's token service.  For gcr.io
# this is GET https://<reg>/v2/token?service=<reg>&scope=repository:<repo>:pull.
# Other registries advertise the realm/service via a WWW-Authenticate challenge
# on an unauthenticated /v2/ probe; we fall back to parsing that challenge.

import hashlib
import json
import os
import sys
import urllib.error
import urllib.parse
import urllib.request

# Media types we are willing to accept for the manifest GET.  We request the
# OCI image manifest type first, then the docker v2 schema-2 manifest, then the
# index/list types (so a clear error can be raised if the pin points at an
# index rather than a single-arch manifest).
MANIFEST_ACCEPT = ", ".join(
    [
        "application/vnd.oci.image.manifest.v1+json",
        "application/vnd.docker.distribution.manifest.v2+json",
        "application/vnd.oci.image.index.v1+json",
        "application/vnd.docker.distribution.manifest.list.v2+json",
    ]
)

INDEX_MEDIA_TYPES = frozenset(
    [
        "application/vnd.oci.image.index.v1+json",
        "application/vnd.docker.distribution.manifest.list.v2+json",
    ]
)

USER_AGENT = "oya-pull-oci-base/1.0 (+tools/oci/pull-oci-base.py)"


def _http_get(url, headers=None):
    """GET a URL; return (status, headers, body_bytes).  Raises on network error."""
    req = urllib.request.Request(url, method="GET")
    req.add_header("User-Agent", USER_AGENT)
    for k, v in (headers or {}).items():
        req.add_header(k, v)
    try:
        with urllib.request.urlopen(req) as resp:
            return resp.status, dict(resp.headers), resp.read()
    except urllib.error.HTTPError as exc:
        # Return the error response so callers can inspect WWW-Authenticate.
        return exc.code, dict(exc.headers or {}), exc.read()


def _parse_www_authenticate(value):
    """Parse a Bearer WWW-Authenticate header into a dict of its params.

    Example input:
      Bearer realm="https://gcr.io/v2/token",service="gcr.io",scope="repository:distroless/static:pull"
    """
    value = value.strip()
    if value.lower().startswith("bearer "):
        value = value[len("bearer "):]
    params = {}
    # Split on commas that separate key="value" pairs.  Values never contain a
    # bare comma in registry challenges, so a simple split is sufficient.
    for part in value.split(","):
        part = part.strip()
        if "=" not in part:
            continue
        key, _, raw = part.partition("=")
        params[key.strip()] = raw.strip().strip('"')
    return params


def get_bearer_token(registry, repo):
    """Obtain an anonymous bearer token for repository:<repo>:pull.

    Primary path (gcr.io and most token-auth registries):
      GET https://<registry>/v2/token?service=<registry>&scope=repository:<repo>:pull

    Fallback: probe /v2/ unauthenticated, parse the WWW-Authenticate challenge,
    and request a token from the advertised realm/service.
    """
    scope = "repository:{}:pull".format(repo)

    # Primary: the conventional gcr.io-style token endpoint.
    primary_qs = urllib.parse.urlencode({"service": registry, "scope": scope})
    primary_url = "https://{}/v2/token?{}".format(registry, primary_qs)
    status, _hdrs, body = _http_get(primary_url)
    if status == 200:
        token = _extract_token(body)
        if token:
            return token

    # Fallback: probe /v2/ to read the auth challenge.
    probe_url = "https://{}/v2/".format(registry)
    status, hdrs, _body = _http_get(probe_url)
    if status == 200:
        # Registry allows anonymous access without a token.
        return None
    if status != 401:
        raise SystemExit(
            "registry {} /v2/ probe returned {} (expected 401 challenge or 200)".format(
                registry, status
            )
        )
    challenge = hdrs.get("WWW-Authenticate") or hdrs.get("Www-Authenticate")
    if not challenge:
        raise SystemExit(
            "registry {} returned 401 with no WWW-Authenticate header".format(registry)
        )
    params = _parse_www_authenticate(challenge)
    realm = params.get("realm")
    if not realm:
        raise SystemExit(
            "WWW-Authenticate from {} has no realm: {!r}".format(registry, challenge)
        )
    service = params.get("service", registry)
    qs = urllib.parse.urlencode({"service": service, "scope": scope})
    token_url = "{}?{}".format(realm, qs)
    status, _hdrs, body = _http_get(token_url)
    if status != 200:
        raise SystemExit(
            "token request to {} returned {}".format(token_url, status)
        )
    token = _extract_token(body)
    if not token:
        raise SystemExit("token response from {} had no token".format(token_url))
    return token


def _extract_token(body):
    """Extract a bearer token from a token-service JSON response."""
    try:
        doc = json.loads(body.decode("utf-8"))
    except (ValueError, UnicodeDecodeError):
        return None
    # Registries return the token under "token" or "access_token".
    return doc.get("token") or doc.get("access_token")


def _auth_headers(token, accept=None):
    headers = {}
    if token:
        headers["Authorization"] = "Bearer {}".format(token)
    if accept:
        headers["Accept"] = accept
    return headers


def fetch_manifest(registry, repo, digest, token):
    """GET the manifest by digest; return its raw bytes and media type.

    Verifies sha256(bytes) == digest and rejects multi-arch index manifests
    (the pin must reference a single-arch linux/arm64 image manifest).
    """
    url = "https://{}/v2/{}/manifests/{}".format(registry, repo, digest)
    status, hdrs, body = _http_get(url, _auth_headers(token, MANIFEST_ACCEPT))
    if status != 200:
        raise SystemExit(
            "manifest GET {} returned {} (body: {!r})".format(url, status, body[:256])
        )

    # Verify the content digest matches the requested pin.
    actual = "sha256:" + hashlib.sha256(body).hexdigest()
    if actual != digest:
        raise SystemExit(
            "manifest digest mismatch: requested {} but downloaded bytes hash to {}".format(
                digest, actual
            )
        )

    media_type = (hdrs.get("Content-Type") or "").split(";")[0].strip()
    try:
        doc = json.loads(body.decode("utf-8"))
    except (ValueError, UnicodeDecodeError) as exc:
        raise SystemExit("manifest {} is not valid JSON: {}".format(digest, exc))

    # The mediaType inside the document is authoritative if present.
    doc_media = doc.get("mediaType", "") or media_type
    if doc_media in INDEX_MEDIA_TYPES:
        raise SystemExit(
            "digest {} references a multi-arch image index ({}), not a single-arch "
            "manifest. Re-pin to the linux/arm64 child digest "
            "(e.g. `crane digest --platform linux/arm64 {}/{}`).".format(
                digest, doc_media, registry, repo
            )
        )
    return body, doc


def fetch_blob(registry, repo, digest, token):
    """GET a blob by digest; verify its sha256 and return the raw bytes."""
    url = "https://{}/v2/{}/blobs/{}".format(registry, repo, digest)
    status, _hdrs, body = _http_get(url, _auth_headers(token))
    if status != 200:
        raise SystemExit("blob GET {} returned {}".format(url, status))
    actual = "sha256:" + hashlib.sha256(body).hexdigest()
    if actual != digest:
        raise SystemExit(
            "blob digest mismatch for {}: downloaded bytes hash to {}".format(
                digest, actual
            )
        )
    return body


def _blob_hex(digest):
    """Strip the `sha256:` prefix, returning the bare hex string."""
    if digest.startswith("sha256:"):
        return digest[len("sha256:"):]
    raise SystemExit("expected sha256: digest, got {!r}".format(digest))


def _write_blob(blobs_dir, digest, data):
    """Write `data` to <blobs_dir>/<hex(digest)>."""
    path = os.path.join(blobs_dir, _blob_hex(digest))
    with open(path, "wb") as fh:
        fh.write(data)


def main(argv):
    if len(argv) != 5:
        sys.stderr.write(
            "usage: pull-oci-base.py REGISTRY REPO DIGEST OUTDIR\n"
        )
        return 2
    registry, repo, digest, outdir = argv[1], argv[2], argv[3], argv[4]

    if not digest.startswith("sha256:"):
        sys.stderr.write("DIGEST must be a sha256: digest, got {!r}\n".format(digest))
        return 2

    # 1. Prepare the OCI layout directory tree.
    blobs_dir = os.path.join(outdir, "blobs", "sha256")
    os.makedirs(blobs_dir, exist_ok=True)

    # 2. Anonymous bearer token.
    token = get_bearer_token(registry, repo)

    # 3. Fetch + verify the image manifest.
    manifest_bytes, manifest = fetch_manifest(registry, repo, digest, token)

    # 4. Persist the manifest bytes verbatim as a blob keyed by DIGEST.
    #    (sha256(manifest_bytes) == DIGEST was verified in fetch_manifest.)
    _write_blob(blobs_dir, digest, manifest_bytes)

    # 5. Download the config blob.
    config_desc = manifest.get("config")
    if not config_desc or "digest" not in config_desc:
        raise SystemExit("manifest {} has no config descriptor".format(digest))
    config_digest = config_desc["digest"]
    config_bytes = fetch_blob(registry, repo, config_digest, token)
    _write_blob(blobs_dir, config_digest, config_bytes)

    # 6. Download every layer blob.
    layers = manifest.get("layers") or []
    if not layers:
        raise SystemExit("manifest {} declares no layers".format(digest))
    for layer in layers:
        layer_digest = layer["digest"]
        layer_bytes = fetch_blob(registry, repo, layer_digest, token)
        _write_blob(blobs_dir, layer_digest, layer_bytes)

    # 7. Write index.json: one manifest entry pointing at DIGEST, platform
    #    arm64/linux (matches the oya-oci-assemble single-arch selector).
    index = {
        "schemaVersion": 2,
        "mediaType": "application/vnd.oci.image.index.v1+json",
        "manifests": [
            {
                "mediaType": manifest.get(
                    "mediaType", "application/vnd.oci.image.manifest.v1+json"
                ),
                "digest": digest,
                "size": len(manifest_bytes),
                "platform": {"architecture": "arm64", "os": "linux"},
            }
        ],
    }
    with open(os.path.join(outdir, "index.json"), "w") as fh:
        json.dump(index, fh, indent=2)
        fh.write("\n")

    # 8. Write the oci-layout marker.
    with open(os.path.join(outdir, "oci-layout"), "w") as fh:
        fh.write('{"imageLayoutVersion":"1.0.0"}')

    sys.stderr.write(
        "pull-oci-base: wrote OCI layout to {} "
        "(manifest {}, config {}, {} layer(s))\n".format(
            outdir, digest, config_digest, len(layers)
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
