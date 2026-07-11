#!/usr/bin/env bash
set -euo pipefail

current=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
npm_current=$(node -p "require('./package.json').version")

if [[ "$current" != "$npm_current" ]]; then
  echo "Cargo.toml version ($current) does not match package.json ($npm_current)" >&2
  exit 1
fi

if [[ -n "$(git status --porcelain)" ]]; then
  echo "working tree is dirty; commit or stash changes first" >&2
  exit 1
fi

branch=$(git branch --show-current)
if [[ "$branch" != "main" ]]; then
  echo "release from main, not $branch" >&2
  exit 1
fi

git fetch origin main --tags
if [[ "$(git rev-parse HEAD)" != "$(git rev-parse origin/main)" ]]; then
  echo "local main is not synced with origin/main" >&2
  exit 1
fi

echo "Current version: $current"
version="${1:-}"
if [[ -z "$version" ]]; then
  read -r -p "Next version: " version
fi

if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "version must look like 0.1.1" >&2
  exit 2
fi

node -e '
const [current, next] = process.argv.slice(1).map(v => v.split(".").map(Number));
const ok = next[0] > current[0] ||
  (next[0] === current[0] && (next[1] > current[1] ||
  (next[1] === current[1] && next[2] > current[2])));
process.exit(ok ? 0 : 1);
' "$current" "$version" || {
  echo "next version must be greater than $current" >&2
  exit 2
}

if git rev-parse -q --verify "refs/tags/v$version" >/dev/null; then
  echo "tag v$version already exists locally" >&2
  exit 1
fi
if git ls-remote --exit-code --tags origin "v$version" >/dev/null 2>&1; then
  echo "tag v$version already exists on origin" >&2
  exit 1
fi

echo "Releasing v$version"
version_commit_created=false
restore_version_files() {
  if [[ "$version_commit_created" == false ]]; then
    git restore -- Cargo.toml Cargo.lock package.json
  fi
}
trap restore_version_files EXIT

VERSION="$version" perl -0pi -e 's/^version = ".*"/version = "$ENV{VERSION}"/m' Cargo.toml
VERSION="$version" node -e "const fs=require('fs'); const p=require('./package.json'); p.version=process.env.VERSION; fs.writeFileSync('package.json', JSON.stringify(p, null, 2) + '\n')"
cargo check
lock_version=$(sed -n '/name = "practicode"/{n;s/version = "\(.*\)"/\1/p;q;}' Cargo.lock)
if [[ "$lock_version" != "$version" ]]; then
  echo "Cargo.lock version ($lock_version) does not match $version" >&2
  exit 1
fi

git diff --check
make test
cargo package --allow-dirty --list >/dev/null
if command -v actionlint >/dev/null 2>&1; then
  actionlint .github/workflows/ci.yml .github/workflows/release.yml
fi

git add Cargo.toml Cargo.lock package.json
git commit -m "Release v$version"
version_commit_created=true
trap - EXIT
git tag "v$version"
git push --atomic origin main "v$version"
