+++
name = "Review replies"
area = "work"
scope = "each_pr"
autonomy = "act"
enabled = true
request = "respond to code comments like we did yesterday"

[trigger]
type = "event"
kind = "pr_new_comments"
+++

Address the new review comments on this PR. Fix what is asked, push, and reply on each thread the way we did yesterday. If a comment needs a product decision, ask me instead.
