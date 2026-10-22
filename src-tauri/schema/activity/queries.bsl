query recent_activity(limit: int) -> ActivityRow[] { list Activity limit ($limit); }
mutation add_activity(text, area, at: timestamp, unread: bool) -> ok {
  create Activity { text = $text, area = $area, at = $at, unread = $unread };
}
mutation expire_activity(before: timestamp) -> ok { delete Activity where (at < $before); }
