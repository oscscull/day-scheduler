+++
name = "CI watch"
area = "work"
scope = "each_pr"
autonomy = "act"
enabled = true
request = "keep an eye on CI for me"

[trigger]
type = "event"
kind = "ci_failed"
+++

Check whether the failing job is a known flaky one. If so, re-run it once. If it fails again, or it is a real failure, tell me what broke.
