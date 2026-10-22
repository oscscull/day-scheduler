query watches() -> WatchRow[] { list Watch; }
mutation claim_with_session(item: Item, session: text, cwd: text?) -> ok {
  create Watch { item = $item, session = $session, cwd = $cwd }
    on conflict (item) update { session = $session, cwd = $cwd };
}
mutation release(item: Item) -> ok { delete Watch where (item = $item); }
