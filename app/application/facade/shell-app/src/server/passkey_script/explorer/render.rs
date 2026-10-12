pub(super) const SCRIPT: &str = r#"
function ontologyTable(caption, columns, rows) {
  const table = document.createElement('table');
  const title = document.createElement('caption'); title.textContent = caption; table.append(title);
  const head = document.createElement('thead'), headers = document.createElement('tr');
  for (const name of columns) { const th = document.createElement('th'); th.scope = 'col'; th.textContent = name; headers.append(th); }
  head.append(headers); table.append(head);
  const body = document.createElement('tbody');
  for (const row of rows) {
    const tr = document.createElement('tr');
    for (const value of row) { const td = document.createElement('td'); if (value instanceof Element) td.append(value); else td.textContent = value === null ? '—' : String(value); tr.append(td); }
    body.append(tr);
  }
  if (!rows.length) { const tr = document.createElement('tr'), td = document.createElement('td'); td.colSpan = columns.length; td.textContent = 'No native rows.'; tr.append(td); body.append(tr); }
  table.append(body); return table;
}
const ontologyDecode = (() => {
  const object = v => v !== null && typeof v === 'object' && !Array.isArray(v);
  const integer = v => Number.isSafeInteger(v) && v >= 0;
  const string = v => typeof v === 'string';
  const nullable = v => v === null || string(v);
  const rows = (v, accepts) => { if (!Array.isArray(v) || !v.every(accepts)) throw new Error('source_unavailable'); };
  const pinned = v => object(v) && string(v.object_ref) && integer(v.written_revision)
    && ['current', 'upcast_pending'].includes(v.upcast_state) && object(v.properties)
    && Object.values(v.properties).every(p => object(p) && string(p.value_type) && string(p.data_class) && Object.hasOwn(p, 'value'));
  return {
    types: v => rows(v, t => object(t) && string(t.entity_type) && integer(t.revision) && Array.isArray(t.properties) && t.properties.every(string)),
    page: (v, pin, limit) => { if (!object(v) || v.requested_revision !== pin || !nullable(v.next)) throw new Error('source_unavailable'); rows(v.objects, pinned); if (v.objects.length > limit) throw new Error('source_unavailable'); },
    detail: (v, ref, pin) => { if (!object(v) || v.requested_revision !== pin || !pinned(v.object) || v.object.object_ref !== ref) throw new Error('source_unavailable'); },
    history: v => rows(v, r => object(r) && ['ordinal', 'object_sequence', 'schema_revision'].every(k => integer(r[k])) && ['principal_id', 'decision_id', 'action_type', 'audit_event_type'].every(k => string(r[k]))),
    audit: v => rows(v, r => object(r) && integer(r.ordinal) && string(r.object_ref) && string(r.action_type) && ['applied', 'poisoned'].includes(r.disposition) && ['poison_reason', 'principal_id', 'decision_id'].every(k => nullable(r[k]))),
  };
})();
"#;
