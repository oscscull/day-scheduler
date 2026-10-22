# Something that needs the user now. Dismissing deletes it.
@sort(raised_at desc, id desc)
Urgent {
  id:          serial
  title:       text
  detail:      text
  area:        text
  raised_at:   timestamp
  schedule_id: text?
}
shape UrgentRow from Urgent { id, title, detail, area, raised_at, schedule_id }
