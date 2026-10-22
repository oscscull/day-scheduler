# Something only the user can do.
Todo {
  id:      serial
  title:   text
  area:    text
  due:     date?
  done_at: timestamp?
  @index done_at
}
shape TodoRow from Todo { id, title, area, due, done_at }
