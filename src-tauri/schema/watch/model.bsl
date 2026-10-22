# The agent session holding an item, so Quarterdeck can resume it with answers and CI results.
@sort(item)
@key(item)
Watch {
  item:    Item @fk("the session goes with its item", on_delete: cascade)
  session: text?
  cwd:     text?
}
shape WatchRow from Watch { item_id = item.id, session, cwd }
