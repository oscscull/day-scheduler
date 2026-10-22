Every item has a turn, and while it's yours Quarterdeck keeps prompting you. Before you stop, hand
it on with pass_turn on {{item}}, saying in `did` what you did in one line:
- `agent` to carry on yourself in a fresh prompt,
- `others` with `waiting_on` ("CI", reviewers' names) when it waits on them; Quarterdeck brings it
  back when they're done, or after a while,
- `you` when the user has to decide or act (raise a question with add_ask instead when it's one).
If you can't make progress, pass it to the user and say why. Stopping without passing hands it to
the user with your last message. Don't poll CI: Quarterdeck resumes this session when it finishes.
