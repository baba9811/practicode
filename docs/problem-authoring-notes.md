# Problem Authoring Notes

Use this when AI creates a new `/next` problem.

## Shape

- Keep one clear task per problem.
- Prefer stdin/stdout fundamentals first: arithmetic, strings, arrays, maps, sorting, two pointers, then graphs/DP later.
- State input, output, examples, and constraints explicitly.
- Make examples small enough to verify by hand.
- Public examples should illustrate the format and one or two edge cases, not exhaust the solution.
- Include enough hidden cases to catch empty/min/max, duplicates, ties, ordering, and whitespace mistakes.
- Keep answers for `python`, `ts`, `java`, and `rust` in `.practicode/problem_bank.json`; never put answers in `README.md`.
- Do not create `solution.*`, `test_solution.*`, or answer-revealing files inside `problems/NNN-slug/`.
  The learner's editable code belongs under `submissions/`, and answer keys belong only in the local bank or built-in data.

## Difficulty

- `easy`: one idea, direct parsing, O(n) or O(n log n), no tricky proof.
- `medium`: combine two ideas or need a careful invariant.
- Move up only after recent submitted solutions pass and look clean.

## Local Preferences

If `.practicode/problem_notes.md` exists, read it too. That file is for personal themes like:

```text
Prefer Korean statements.
I want more string and hashmap practice.
Avoid DP until I ask for it.
```

## References

- Kattis problem package format: https://www.kattis.com/problem-package-format/
- ICPC judging guidelines: https://icpc.global/regionals/regional-contest-cookbook-judging-guidelines
- MIT Teaching + Learning Lab, worked examples: https://tll.mit.edu/teaching-resources/how-people-learn/worked-examples/
- Roediger, Agarwal, McDaniel, and McDermott, test-enhanced learning: https://pdf.retrievalpractice.org/guide/Roediger_Agarwal_etal_2011_JEPA.pdf
- Parsons problems literature review: https://juholeinonen.com/assets/pdf/ericson2022parsons.pdf
