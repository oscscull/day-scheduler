query open_asks() -> AskRow[] { list Ask where (answer = null); }
query open_asks_for(item: Item) -> AskRow[] { list Ask where (item = $item and answer = null); }
# Answers not yet taken back to whoever asked.
query answered_asks() -> AskRow[] { list Ask where (answer != null); }
query ask(id) -> AskRow;

mutation add_ask(item: Item, question, context, options: json, session: text?,
                 cwd: text?) -> AskRow {
  create Ask { item = $item, question = $question, context = $context, options = $options,
               session = $session, cwd = $cwd };
}
mutation answer_ask(id, answer: text) -> ok { update Ask where (id = $id) { answer = $answer }; }
mutation remove_ask(id) -> ok { delete Ask where (id = $id); }
