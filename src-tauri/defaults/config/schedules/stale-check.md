+++
name = "Stale check"
area = "work"
scope = "each_item"
autonomy = "ask"
enabled = false
request = "tell me when things go stale"

[trigger]
type = "cron"
expr = "0 10 * * 1-5"
+++

Look for items that have not moved in 3 days. Suggest a nudge or a next step for each.
