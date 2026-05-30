release:
    cargo check
    cargo clippy
    cargo package --list --allow-dirty
    cargo publish --dry-run --allow-dirty
    git cliff --bump --unreleased
    echo "If everything looks good, run 'just release-final' to finalize the release."

release-final:
    cargo check
    cargo clippy
    git cliff --bump -o CHANGELOG.md
    git add CHANGELOG.md
    git commit -m "chore: release $(git cliff --bumped-version)"
    git tag $(git cliff --bumped-version)
    git push && git push --tags