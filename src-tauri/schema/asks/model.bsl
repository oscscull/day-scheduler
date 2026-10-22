# An open question for the user. Answering keeps the row only until the answer is delivered.
@sort(id)
Ask {
  id:       serial
  item:     Item @fk("asks go with their item", on_delete: cascade)
  question: text
  context:  text
  # The choices offered, a JSON array of strings.
  options:  json
  # The Claude Code session that asked, and the folder it ran in.
  session:  text?
  cwd:      text?
  answer:   text?
  @index(answer)
  @index(item, answer)
}
shape AskRow from Ask { id, item_id = item.id, question, context, options, session, cwd, answer }
