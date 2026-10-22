query meta(key) -> MetaRow;
mutation set_meta(key, value) -> ok {
  create Meta { key = $key, value = $value } on conflict (key) update { value = $value };
}
mutation forget_meta(key) -> ok { delete Meta where (key = $key); }

query sync_snapshot(source, external_id) -> SnapshotRow;
mutation set_sync_snapshot(source, external_id, snapshot) -> ok {
  create SyncState { source = $source, external_id = $external_id, snapshot = $snapshot }
    on conflict (source, external_id) update { snapshot = $snapshot };
}
mutation forget_sync_snapshot(source, external_id) -> ok {
  delete SyncState where (source = $source and external_id = $external_id);
}
mutation expire_sync_snapshots() -> ok { delete SyncState where (orphaned_calendar); }
