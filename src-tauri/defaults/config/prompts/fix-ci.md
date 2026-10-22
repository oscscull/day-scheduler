CI is failing on {{item}}: "{{title}}" ({{link}}). Nobody was on it, so Quarterdeck has handed it to you.

{{workspace}}

1. Use `gh pr checks` and `gh run view --log-failed` to find what failed and why.
2. If it's flaky (a timeout, a network blip, an unrelated test), re-run it with `gh run rerun --failed`.
3. If the fix is clear and small (a stale branch, a type error, a lint), make it and push.
4. Otherwise record the cause with add_note on {{item}}, keep its state label current with
   update_item, and raise an ask with add_ask (offering options where it's a choice) only if a
   human decision is needed.

Reply with one short sentence.
