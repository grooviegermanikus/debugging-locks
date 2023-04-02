#!/usr/bin/env bash
set -euo pipefail

test $# -eq 1 || {
 echo "Usage: $0 <version>"
 exit 1
}

git diff --exit-code >/dev/null || {
 echo "Aborted: Git working directory must be clean"
 exit 1
}

BRANCH="$(git rev-parse --abbrev-ref HEAD)"
if [[ $BRANCH != "main" ]]; then
 echo "Error: Must be working on master branch"
 exit 8
fi

# e.g. 1.1.5
NEW_VERSION="$1"

[[ $NEW_VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
 echo "Error: version format must be x.y.z"
 exit 2
}

# requires: cargo install cargo-bump
cargo set-version $NEW_VERSION || exit 3

if git diff --exit-code >/dev/null; then
 echo "Usage: no changes to commit"
 exit 11
fi

VERSION_TAG="v$NEW_VERSION"

git add Cargo.toml && git commit -m "---- Version $NEW_VERSION" || exit 4

git tag "$VERSION_TAG" || exit 9

echo
echo "Updated to version $NEW_VERSION; commited but did not push; use ..."
echo "- git push origin main --tags"
