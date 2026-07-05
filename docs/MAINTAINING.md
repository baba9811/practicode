# Maintaining

This guide is for maintainers with commit, tag, or publishing responsibility.

Contributor-facing workflow lives in [CONTRIBUTING.md](CONTRIBUTING.md).

## Triage

- Keep issues actionable: expected behavior, actual behavior, reproduction steps, OS, terminal, and install method.
- Use small scopes. Ask broad proposals to become one issue or pull request per behavior change.
- Label approachable work as `good first issue` or `help wanted` when the fix is clear.
- Close duplicates with a link to the canonical issue.

## Pull Request Review

- Review the diff file by file.
- Check correctness, local-first behavior, terminal UX, docs, and tests before style.
- Prefer focused pull requests. Ask contributors to split unrelated changes.
- Require checks or a clear reason checks were not run.
- For visible TUI changes, ask for a screenshot or a short terminal description.
- Merge only after CI passes and the branch is up to date enough to avoid obvious conflicts.

## Release

`main` runs CI only. Releases are tag-based and publish to crates.io and npm through GitHub Actions.

Preflight:

```bash
git checkout main
git pull --ff-only origin main
make test
```

Release:

```bash
make release VERSION=0.1.3
```

The release script checks versions, runs tests, creates the version commit and tag, and pushes `main` plus the tag.

Verify publication:

```bash
gh run list --limit 5
npm view practicode version
npm view practicode dist.signatures dist.attestations --json
cargo search practicode --limit 1
```

Do not print or commit tokens. Local `.env` and `.npmrc` are ignored; GitHub Actions uses `NPM_TOKEN` and `CRATES_TOKEN` repository secrets.

For npm supply-chain posture, keep `publishConfig.provenance` enabled and keep the release job's `id-token: write` permission. When the npm package's Trusted Publisher setting is configured for this repository and `.github/workflows/release.yml`, remove the long-lived `NPM_TOKEN` dependency from the npm publish steps and disallow token publishing in the npm package settings.

Socket.dev indexes the npm package page at <https://socket.dev/npm/package/practicode>. It may lag behind npm immediately after a release; verify npm first with `npm view practicode version`, then re-check Socket after indexing catches up. If Socket flags the npm `postinstall` script, confirm it still only runs the locked Cargo build documented in the README.

## Documentation Ownership

| File | Audience |
| --- | --- |
| [../README.md](../README.md) | Users installing and running practicode |
| [CONTRIBUTING.md](CONTRIBUTING.md) | External contributors opening issues or pull requests |
| [MAINTAINING.md](MAINTAINING.md) | Maintainers reviewing, triaging, and releasing |
| [problem-authoring-notes.md](problem-authoring-notes.md) | AI/local problem generation rules |

## Maintainer References

- GitHub reviewing pull requests: https://docs.github.com/articles/reviewing-proposed-changes-in-a-pull-request
- GitHub helping reviewers: https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/getting-started/helping-others-review-your-changes
- GitHub healthy contributions: https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions
- Git contributing through forks: https://git-scm.com/book/en/v2/GitHub-Contributing-to-a-Project
