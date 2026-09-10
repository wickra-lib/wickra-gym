<!-- Keep it short. One logical change per PR. -->

## What

<!-- What does this change and why? -->

## Checklist

- [ ] `cargo fmt --all` and `cargo clippy --workspace --all-targets --all-features -- -D warnings` are clean
- [ ] `cargo test --workspace --all-features` and `--no-default-features` pass (parallel == sequential)
- [ ] `cargo deny check` is clean
- [ ] Tests added/updated (prefer hand-computed expectations for core changes)
- [ ] The spec stays data (a serde `EnvSpec`), never Rust closures
- [ ] Binding surface mirrored across languages; golden reports regenerated if the schema changed
- [ ] `CHANGELOG.md` updated under `[Unreleased]`

<!-- For a larger change, the long-form template asks the questions this one
     does not: append `?template=detailed.md` to the PR URL, or copy
     `.github/PULL_REQUEST_TEMPLATE/detailed.md` in by hand. -->
