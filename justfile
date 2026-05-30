release:
    cargo check
    cargo clippy
    cargo package --list --allow-dirty
    cargo publish --dry-run --allow-dirty
    git cliff --bump -o CHANGELOG.md
    echo "If everything looks good, run 'just publish' to push the release."

publish:
    git add CHANGELOG.md
    git commit -m "chore: release v$(git cliff --bumped-version)"
    git tag "v$(git cliff --bumped-version)" -m "Release v$(git cliff --bumped-version)"
    git push && git push --tags 