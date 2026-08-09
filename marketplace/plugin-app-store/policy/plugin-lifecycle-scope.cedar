// plugin-app-store µservice — plugin-lifecycle-scope policy
// Cedar v4 schema
// Purpose: only plugin publishers can submit / withdraw / deprecate their own plugin versions.

forbid (principal, action, resource);

permit (
  principal in PluginPublisher::?p,
  action in [
    Action::"submit_plugin_version",
    Action::"withdraw_submission",
    Action::"deprecate_plugin_version",
    Action::"view_plugin_metrics",
    Action::"view_plugin_revenue"
  ],
  resource in PluginPublisherData::?r
)
when {
  principal has developer_id &&
  resource has publisher_id &&
  principal.developer_id == resource.publisher_id
};
