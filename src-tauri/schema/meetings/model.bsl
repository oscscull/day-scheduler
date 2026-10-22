# A meeting or other fixed-time commitment, by hand or from a calendar feed.
@sort(starts_at, id)
Commitment {
  id:          serial
  title:       text
  area:        text
  starts_at:   timestamp
  ends_at:     timestamp
  location:    text?
  join_url:    text?
  source:      text
  external_id: text?
  hidden:      bool (default false)
  @index(source, external_id) unique
  @index(hidden, starts_at)
  @index ends_at
}
shape CommitmentRow from Commitment {
  id, title, area, starts_at, ends_at, location, join_url, source, external_id, hidden
}

# Whether a meeting's alert was dealt with or snoozed.
@sort(commitment)
@key(commitment)
AlertState {
  commitment:    Commitment @fk("alert state goes with its meeting", on_delete: cascade)
  done:          bool (default false)
  snoozed_until: timestamp?
}
shape AlertRow from AlertState { commitment_id = commitment.id, done, snoozed_until }
