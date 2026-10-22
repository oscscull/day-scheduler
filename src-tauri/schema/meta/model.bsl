# Small facts about the app itself, e.g. when the scheduler last ran.
@key(key)
Meta {
  key:   text
  value: text
}
shape MetaRow from Meta { value }

# What a connector last saw for each thing it syncs, so it can tell what changed upstream
# and leave alone anything the user or an agent changed since.
@key(source, external_id)
SyncState {
  source:      text
  external_id: text
  snapshot:    text
}
shape SnapshotRow from SyncState { snapshot }

# Calendar snapshots whose meeting has gone.
filter orphaned_calendar = raw`source LIKE 'calendar:%' AND NOT EXISTS (
  SELECT 1 FROM commitment c
  WHERE c.source = sync_state.source AND c.external_id = sync_state.external_id)`;
