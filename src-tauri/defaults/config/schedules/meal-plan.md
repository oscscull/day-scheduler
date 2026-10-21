+++
name = "Meal plan"
area = "life"
scope = "nothing"
autonomy = "ask"
enabled = true
request = "plan dinners for the week every Sunday evening"

[trigger]
type = "cron"
expr = "0 18 * * 0"
+++

Suggest dinners for the coming week that fit what is already in my calendar, and a shopping list. Ask before adding anything to my to-dos.
