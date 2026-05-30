release:
    cargo check
    cargo clippy
    cargo package --list --allow-dirty
    cargo publish --dry-run --allow-dirty
    git cliff --bump -o CHANGELOG.md
    cargo set-version $(git cliff --bumped-version | sed 's/^v//')
    echo "If everything looks good, run 'just publish' to push the release."

publish:
    git add CHANGELOG.md
    git add Cargo.toml
    git add Cargo.lock
    git commit -m "chore: release $(git cliff --bumped-version)"
    git tag "$(git cliff --bumped-version)" -m "Release $(git cliff --bumped-version)"
    git push && git push --tags 