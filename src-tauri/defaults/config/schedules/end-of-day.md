+++
name = "End-of-day wrap"
node = "work-mac"
scope = "nothing"
autonomy = "act"
enabled = true
request = "wrap up my day at half five"

[trigger]
type = "cron"
expr = "30 17 * * 1-5"
+++

Summarise what moved today and propose tomorrow's top three priorities.
