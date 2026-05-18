// developer-sdk µservice — public-read policy
// Cedar v4 schema
// Purpose: SDK family list + downloadable specs are public; no auth needed.

permit (
  principal in AnonymousUser::?u,
  action in [
    Action::"list_sdk_families",
    Action::"download_sdk",
    Action::"download_openapi_spec",
    Action::"download_asyncapi_spec",
    Action::"download_proto_spec"
  ],
  resource in PublicSdkArtifact::?r
);
