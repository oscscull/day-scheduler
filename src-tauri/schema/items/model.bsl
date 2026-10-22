enum Stage  { up_next, in_progress, in_review, done }
enum Status { decision, review, ci, ci_failed, dependency, approved, proceeding, queued }

# A piece of work. `id` is a ticket key (JD-104), a PR (app#12) or a made-up one (B-3).
@sort(rank)
@key(id)
Item {
  id:          text
  title:       text
  state_label: text
  stage:       Stage
  status:      Status?
  holder:      text
  area:        text
  link:        text?
  # "manual" for items made by hand or by an agent, else the connector that made it.
  source:      text (default "manual")
  external_id: text?
  # Synced items the user deleted stay hidden, so the next sync doesn't bring them back.
  hidden:      bool (default false)
  # Set aside by the user: off every list until unparked. Syncs never change it.
  parked:      bool (default false)
  opened_at:   timestamp
  state_since: timestamp
  rank:        int
  @index(source, external_id) unique
  @index(hidden, parked, rank)
  @index(stage, state_since)
}

shape ItemRow from Item {
  id, title, state_label, stage, status, holder, area, link, source, external_id,
  opened_at, state_since, rank
}
shape ItemKey from Item { id }
shape ItemParked from Item { parked }
shape TopRank from Item { top = max(rank) }
shape ItemCount from Item { n = count() }

# One line of an item's history. `change` is the app's `Change` enum as JSON.
@sort(at desc, id desc)
ItemEvent {
  id:     serial
  item:   Item @fk("history goes with its item", on_delete: cascade)
  at:     timestamp
  change: json
  @index(item, at)
}
shape EventRow from ItemEvent { id, item_id = item.id, at, change }
