# Watchdog state per item an agent claimed: its session (to resume) and check-ins so far.
@sort(item)
@key(item)
Watch {
  item:       Item @fk("the watch goes with its item", on_delete: cascade)
  session:    text?
  cwd:        text?
  nudges:     int (default 0)
  last_nudge: timestamp?
}
shape WatchRow from Watch { item_id = item.id, session, cwd, nudges, last_nudge }
