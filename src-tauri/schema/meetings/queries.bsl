query commitments() -> CommitmentRow[] { list Commitment where (hidden = false); }
# Everything from one source, hidden ones included, for syncing.
query commitments_from(source) -> CommitmentRow[];
query commitment(id) -> CommitmentRow;
query commitment_by_external(source, external_id: text) -> CommitmentRow;

mutation add_commitment(title, area, starts_at: timestamp, ends_at: timestamp,
                        location: text?, join_url: text?, source, external_id: text?,
                        hidden: bool) -> ok {
  create Commitment {
    title = $title, area = $area, starts_at = $starts_at, ends_at = $ends_at,
    location = $location, join_url = $join_url, source = $source,
    external_id = $external_id, hidden = $hidden
  };
}
mutation update_commitment(id, title, area, starts_at: timestamp, ends_at: timestamp,
                           location: text?, join_url: text?, hidden: bool) -> ok {
  update Commitment where (id = $id) {
    title = $title, area = $area, starts_at = $starts_at, ends_at = $ends_at,
    location = $location, join_url = $join_url, hidden = $hidden
  };
}
mutation delete_commitment(id) -> ok { delete Commitment where (id = $id); }
# Alert state goes with the meeting.
mutation expire_commitments(before: timestamp) -> ok {
  delete Commitment where (ends_at < $before);
}

query alert_states() -> AlertRow[] { list AlertState; }
mutation alert_done(commitment: Commitment) -> ok {
  create AlertState { commitment = $commitment, done = true }
    on conflict (commitment) update { done = true };
}
mutation alert_snooze(commitment: Commitment, until: timestamp) -> ok {
  create AlertState { commitment = $commitment, snoozed_until = $until }
    on conflict (commitment) update { snoozed_until = $until };
}
