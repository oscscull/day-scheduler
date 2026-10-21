+++
name = "Morning brief"
area = "work"
scope = "nothing"
autonomy = "act"
enabled = true
request = "give me a morning summary every weekday"

[trigger]
type = "cron"
expr = "0 9 * * 1-5"
+++

Summarise today: meetings, what needs me on my top priorities, and what Claude finished overnight. Keep it to five lines.
