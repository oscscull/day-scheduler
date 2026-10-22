query watches() -> WatchRow[] { list Watch; }
# An agent took the item on: a fresh start for the watchdog, keeping any known session.
mutation claim(item: Item) -> ok {
  create Watch { item = $item } on conflict (item) update { nudges = 0, last_nudge = null };
}
mutation claim_with_session(item: Item, session: text, cwd: text?) -> ok {
  create Watch { item = $item, session = $session, cwd = $cwd }
    on conflict (item) update { session = $session, cwd = $cwd, nudges = 0, last_nudge = null };
}
mutation record_nudges(item: Item, nudges: int, last_nudge: timestamp?) -> ok {
  create Watch { item = $item, nudges = $nudges, last_nudge = $last_nudge }
    on conflict (item) update { nudges = $nudges, last_nudge = $last_nudge };
}
mutation release(item: Item) -> ok { delete Watch where (item = $item); }
