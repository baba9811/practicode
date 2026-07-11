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
make release VERSION=0.2.0
```

The release script checks Cargo/npm/lock versions, runs the complete lesson, launcher, package, clippy, test, and isolated smoke gates, creates the version commit and tag, then atomically pushes `main` plus the tag.

The tag workflow first verifies that the exact tag commit belongs to `origin/main`, validates again, and builds five native assets: macOS Intel/Apple Silicon, Linux x64/arm64 musl, and Windows x64. Each binary is smoked on its matching hosted runner. A draft GitHub Release receives all binaries plus `SHA256SUMS`, then publication proceeds in dependency order: crates.io, the public GitHub Release, and finally npm. This guarantees that a newly published npm launcher can download public assets immediately.

Both registry jobs use the GitHub `release` environment. Keep its deployment policy restricted to tag pattern `v*.*.*` with no required human reviewer; the workflow itself accepts only exact `vMAJOR.MINOR.PATCH` tags reachable from `main`. Keep workflow Actions pinned to full commit SHAs and update their adjacent version comments and pins together.

Verify publication:

```bash
gh run list --workflow release.yml --limit 5
gh run watch <run-id> --exit-status
gh release view v0.2.0 --json assets --jq '.assets[].name'
npm view practicode@0.2.0 version
npm view practicode dist.signatures dist.attestations --json
cargo search practicode --limit 1
```

Download the published assets to a temporary directory and verify `SHA256SUMS` before announcing the release.

Do not print or commit tokens. Local `.env` and `.npmrc` are ignored; GitHub Actions uses `NPM_TOKEN` and `CRATES_TOKEN` repository secrets.

For npm supply-chain posture, keep `publishConfig.provenance` enabled and keep the release job's `id-token: write` permission. When the npm package's Trusted Publisher setting is configured for this repository and `.github/workflows/release.yml`, remove the long-lived `NPM_TOKEN` dependency from the npm publish steps and disallow token publishing in the npm package settings.

Socket.dev indexes the npm package page at <https://socket.dev/npm/package/practicode>. It may lag behind npm immediately after a release; verify npm first, then re-check Socket after indexing catches up. The npm package must not include install lifecycle scripts. Its Node launcher downloads the immutable versioned release asset, verifies SHA-256, installs it atomically in a per-user cache, and never invokes Cargo on the native path.

## Documentation Ownership

| File | Audience |
| --- | --- |
| [../README.md](../README.md) | Users installing and running practicode |
| [COMMANDS.md](COMMANDS.md) | Users looking up slash commands and aliases |
| [CONTRIBUTING.md](CONTRIBUTING.md) | External contributors opening issues or pull requests |
| [MAINTAINING.md](MAINTAINING.md) | Maintainers reviewing, triaging, and releasing |
| [problem-authoring-notes.md](problem-authoring-notes.md) | AI/local problem generation rules |
| [../assets/lessons/README.md](../assets/lessons/README.md) | Syntax lesson catalog layout and required fields |

## Maintainer References

- GitHub reviewing pull requests: https://docs.github.com/articles/reviewing-proposed-changes-in-a-pull-request
- GitHub helping reviewers: https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/getting-started/helping-others-review-your-changes
- GitHub healthy contributions: https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions
- Git contributing through forks: https://git-scm.com/book/en/v2/GitHub-Contributing-to-a-Project
