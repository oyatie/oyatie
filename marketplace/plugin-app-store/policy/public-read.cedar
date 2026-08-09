// plugin-app-store µservice — public-read policy
// Cedar v4 schema
// Purpose: unauthenticated users can browse the public catalog (read-only).

permit (
  principal in AnonymousUser::?u,
  action in [
    Action::"list_plugins",
    Action::"get_plugin",
    Action::"search_plugins",
    Action::"list_categories"
  ],
  resource in PublicCatalog::?r
);
