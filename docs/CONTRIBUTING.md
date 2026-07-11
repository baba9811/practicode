# Contributing

Thanks for helping improve practicode. This guide is for contributors opening issues or pull requests.

Maintainer-only review and release steps live in [MAINTAINING.md](MAINTAINING.md).

## Before You Start

- Search existing issues and pull requests first.
- Small bug fixes, docs fixes, tests, and localization updates can go straight to a pull request.
- For larger UI, AI-generation, storage, or packaging changes, open an issue first so the scope is clear.
- Check [ARCHITECTURE.md](ARCHITECTURE.md) before adding commands, settings, provider behavior, or persisted state.
- Do not commit legacy practice data from `.practicode/`, `problems/`, or `submissions/`, or copy global `~/.practicode` data into the repository.
- Do not include secrets, tokens, private prompts, or generated answer keys in docs or examples.

## Fork And Pull Request Flow

1. Fork `baba9811/practicode` on GitHub.
2. Clone your fork and add the original repo as `upstream`.

```bash
git clone https://github.com/<your-user>/practicode.git
cd practicode
git remote add upstream https://github.com/baba9811/practicode.git
```

3. Create a focused branch from the latest `main`.

```bash
git fetch upstream
git checkout -b fix-short-name upstream/main
```

4. Make one focused change.
5. Run the smallest checks that cover your change.
6. Push your branch and open a pull request into `baba9811/practicode:main`.

```bash
git push origin fix-short-name
```

When opening the pull request, include what changed, how you checked it, and screenshots for visible TUI changes.

## Local Setup

Prerequisites:

- Rust stable with Cargo, rustfmt, and clippy.
- Node.js 18+ for the npm launcher and package checks.
- For the complete executable curriculum gate: Python 3.12, Node 22, TypeScript 5.9.3, JDK 21, and stable Rust.

Common commands:

```bash
cargo run --
cargo run -- --smoke
cargo test
npm run test:launcher
```

Full local check:

```bash
make test
```

## Project Map

| Path | Role |
| --- | --- |
| `src/core.rs` | Problem storage, state, rendering, judging |
| `src/core/syntax.rs` | Built-in syntax lessons, exercises, and lesson-copy loading |
| `src/i18n.rs` | Loads UI strings from `assets/i18n/*.json` |
| `src/tui.rs` | Ratatui app, editor, command parser |
| `src/ai.rs` | Codex/Claude command integration and notes |
| `src/text.rs` | UTF-8 cursor math and Hangul composition |
| `src/process.rs` | Process execution helpers |
| `assets/lessons/` | Syntax lesson study copy split by programming language and UI language |
| `tests/` | Integration tests split by module |

## Change Guidelines

- Keep pull requests small and reviewable.
- Reuse existing helpers and patterns before adding new code.
- Prefer the Rust standard library. Add crates only when they remove real complexity.
- Put UI strings in [assets/i18n](../assets/i18n), not inline in Rust code.
- Keep every UI and lesson catalog complete; incomplete locale prose does not fall back silently.
- Put syntax lesson study copy in [assets/lessons](../assets/lessons) and follow its executable-content contract.
- Keep the root [README](../README.md) focused on users.
- Use relative links for repo-local docs and assets.

## Lesson Changes

Lesson corrections and new cases need evidence beyond a prose diff:

1. Make the English behavior, starter, cases, and primary references agree.
2. Update all affected locales without changing identifiers or operators.
3. Run the focused runtime/mutation test and the i18n suite.
4. Ask an independent agent that did not author the change to read the final records against the code and references. A human reviewer is not required.
5. Resolve every Critical/Important finding, then update the relevant review profile verdict if needed.
6. Run `node scripts/check-lessons.js --refresh`, inspect the hash diff, and run the checker again.

CI rejects stale hashes, incomplete 4×5 coverage, self-approval identities, unresolved disagreements, and open high-severity findings.

## Problem Authoring

Use [problem-authoring-notes.md](problem-authoring-notes.md) when changing problem-generation behavior.

Runtime paths are relative to `PRACTICODE_HOME` or `~/.practicode`:

| Path | Purpose |
| --- | --- |
| `problem_bank.json` | Local/custom/generated problems |
| `problem_notes.md` | Personal problem-generation notes |
| `problem-state.json` | Current problem, history, settings |
| `problems/` | Generated problem markdown/index files |
| `submissions/` | Local answer files |

## Pull Request Checklist

- The change is focused and explained.
- Relevant checks were run, or the PR says why they were not.
- Visible TUI changes include a screenshot or short terminal description.
- User-facing text is in `assets/i18n/*.json`.
- Changed lessons have executable evidence and an independent review manifest hash.
- Local generated data, secrets, tokens, and answer keys are not committed.

## References

- GitHub contributing guide: https://docs.github.com/en/get-started/exploring-projects-on-github/contributing-to-open-source
- GitHub contributing guidelines: https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/setting-guidelines-for-repository-contributors
- GitHub pull requests: https://docs.github.com/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/about-pull-requests
- Open Source Guides community notes: https://opensource.guide/building-community/
- WAI-ARIA combobox keyboard interaction: https://www.w3.org/WAI/ARIA/apg/patterns/combobox/
- Command Line Interface Guidelines: https://clig.dev/
- Ratatui terminal UI library: https://ratatui.rs/
- Crossterm terminal backend/events: https://github.com/crossterm-rs/crossterm
- Kattis problem package format: https://www.kattis.com/problem-package-format/
- ICPC judging guidelines: https://icpc.global/regionals/regional-contest-cookbook-judging-guidelines
