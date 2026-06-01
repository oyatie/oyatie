# Bespoke OCI Pipeline: Sequencing (loop-live-on-interim → bespoke-OCI-product)

> Idea-refine output, 2026-05-31. Founder steer: "loop live fast on interim" + "bespoke OCI
> tooling IS a product surface (ECR/Artifact-Registry equiv)" + "loop first (delay not
> acceptable)". Reverses an earlier "full bespoke-OCI build image now" pick after stress-test.

## Problem Statement
How might we bring oyatie's bespoke CI gate live NOW — true to the dogfood/bespoke
doctrine — without the toolchain-image path or a not-yet-built bespoke OCI stack
delaying the live gate that drains the ~36 banked, unverifiable PRs?

## Recommended Direction
Close the loop on the bespoke ASSEMBLER we already have (`oya-oci-assemble`, proven
2026-05-31) + INTERIM registry I/O and signing (curl on the auth-free in-cluster registry;
crane/cosign as sanctioned interim adapters — same status as Forgejo/Jenkins/SeaweedFS).
Put crane/cosign/BuildKit behind `OciRegistry` + `ImageSigner` trait seams so only the
adapters are transitory. Commit the bespoke Rust OCI registry + artifact builder + signer
as a ROADMAPPED PRODUCT (oya's ECR / Artifact-Registry equivalent) via its own ADR —
built when it earns it, NOT as a gate prerequisite. Runtime images stay buck2-OCI
(controller proven); the build/toolchain image and registry are the bespoke-product track.

Full-bespoke-now was the over-reach: it's not gate-plumbing to rush, it's a product to
sequence. buck2-OCI fits declarative runtime images (base + static binary); it fits
imperative toolchain images poorly, and the bespoke path still needs registry I/O either
way — so the registry client is a real product, not a quick hack.

## Key Assumptions to Validate
- [x] RESOLVED 2026-05-31: NO admission policy enforces signatures right now. Kubewarden
      (canonical per ADR-0379, supersedes ADR-0183; Rust→WASM policies; Cedar=app-authz,
      Kyverno=adapter-only) is deployed but enforces 0 ClusterAdmissionPolicies. The
      `verify-oya-registry-images-signed` policy is KYVERNO-authored + NOT deployed
      (`NotFound`). ⇒ cosign is DEFERRABLE for the interim gate. TWO drift items to track:
      (a) Kyverno still runs in the base overlay — ADR-0379 §2 says adapter-only, remove it;
      (b) the signing fast-follow MUST be a Kubewarden Rust/WASM policy (ADR-0379 §Verification:
      "image-signing admission is enforced by a Kubewarden policy (follow-on)"), NOT the Kyverno one.
- [ ] curl can push an OCI image to the insecure in-cluster registry (distribution API:
      POST/PUT blobs + manifest). Test: push a tiny test image via curl, re-pull it.
- [ ] distroless-static arm64 is curl-pullable from GCR (anon token dance) OR mirror-once.
      Test: curl the GCR token + manifest + 2 blobs; assemble a valid OCI layout.
- [ ] `:distroless-base` http_archive defect (registry won't serve files) is fixed by a
      local pinned tarball / genrule. Test: `buck2 build controller-oci` resolves the base.

## MVP Scope (the live gate)
IN:  curl-OCI distroless base (+ fix the base http_archive defect) → buck2 build
     controller-oci (oya-oci-assemble) → curl-OCI push to in-cluster registry →
     deploy controller Deployment → gate verifies AFFECTED targets on a PR →
     founder relax-merge green → drain the 36.
OUT: bespoke registry client, bespoke signer, full-toolchain-as-buck2-layers, ripping
     out BuildKit/crane/cosign, retiring the Jenkins gate (interim until controller proven).
SIGNING: required iff Kubewarden enforces it (assumption #1); else fast-follow.

## Not Doing (and Why)
- Bespoke Rust OCI registry/builder/signer NOW — it's a product, not gate-glue; mis-sequenced.
- Full toolchain-as-buck2-layers for the build image — buck2-OCI fits runtime, not imperative
  toolchain images; high friction for ~zero gate value.
- Ripping out crane/cosign/BuildKit — they're the interim adapters; swap when bespoke earns it.
- Re-deriving the drifted rust-ci Dockerfile — base off the deployed image, don't reconstruct.

## Open Questions
- Where does the bespoke-OCI-registry product sit on the roadmap vs the bespoke SCM/object-store?
  (It shares the content-addressed-blob substrate with both — likely the same Colossus-shaped core.)
- Does ADR-0514 (retire Dockerfile/BuildKit) need a carve-out: "runtime images now; build/registry
  images follow the bespoke-OCI-product track"?
- Is Kubewarden the canonical admission/policy substrate, or interim toward a bespoke
  Cedar-based admission controller (oya already uses Cedar for authz)? Hyperscaler-pattern ADR?
