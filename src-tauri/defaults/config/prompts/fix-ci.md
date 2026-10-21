CI is failing on {{item}}: "{{title}}" ({{link}}). Nobody was on it, so Bridge has handed it to you.

1. Use `gh pr checks` and `gh run view --log-failed` to find what failed and why.
2. If it's flaky (a timeout, a network blip, an unrelated test), re-run it with `gh run rerun --failed`.
3. Otherwise record the cause with add_note on {{item}}, and keep its state label current with
   update_item, e.g. "CI: type error in auth.ts".
4. Raise an ask with add_ask only if a human decision is needed.

Don't poll CI: Bridge resumes this session when it finishes. Reply with one short sentence.
