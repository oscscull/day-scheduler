# The "From Claude" feed: one-line updates, pruned after their TTL.
@sort(at desc, id desc)
Activity {
  id:     serial
  text:   text
  area:   text
  at:     timestamp
  unread: bool
  @index at
}
shape ActivityRow from Activity { id, text, area, at, unread }
