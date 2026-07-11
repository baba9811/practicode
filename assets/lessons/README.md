# Lesson Catalog Contract

Practicode embeds four executable courses and five complete prose catalogs per course:

```text
assets/lessons/<programming-language>/course.json
assets/lessons/<programming-language>/<ui-language>.json
```

Programming-language directories are `python`, `typescript`, `java`, and `rust`. UI catalogs are `en`, `ko`, `ja`, `zh`, and `es`. The TypeScript runtime key remains `ts`.

`course.json` owns ordered IDs, core/lab classification, examples, starters, deterministic cases, and official references. A locale file owns exactly these ten fields for every course ID:

- `title`
- `concept`
- `worked_example`
- `common_mistakes`
- `self_check`
- `exercise_prompt`
- `objective`
- `language_delta`
- `prediction_prompt`
- `transfer_trap`

There is no generic lesson fallback. Missing, extra, reordered, or templated copy fails the content suite.

## Quality Gate

Every exercise must have three to five bounded cases with distinct inputs and outputs. References must pass; starters, visible-output hardcodes, and declared semantic mutants must fail through the expected compiler/type/runtime/output boundary.

Every final catalog is recorded in `review-manifest.json` with its exact SHA-256 hash, full ID coverage, official source set, distinct author and blind-verifier identities, resolved disagreements, and zero open high-severity findings. The manifest covers 20 catalogs and 550 localized lesson records.

After an independently reviewed lesson change:

```bash
node scripts/check-lessons.js --refresh
node scripts/check-lessons.js
cargo test --test i18n
cargo test --test lesson_quality -- --test-threads=1
```

Inspect the manifest diff. `--refresh` only updates mechanical hashes, IDs, counts, and source coverage; it does not replace an independent review or approve prose.

Prefer official, version-specific primary documentation. Keep code identifiers and operators in backticks, localize explanatory prose, and ensure each mistake/check is specific to the actual starter and cases.
