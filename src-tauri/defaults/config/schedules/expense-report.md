+++
name = "Expense report"
area = "work"
scope = "nothing"
autonomy = "act"
enabled = true
request = "do my expenses every Wednesday afternoon"

[trigger]
type = "cron"
expr = "0 14 * * 3"
+++

Collect this week's receipts from my email and submit the expense report.
