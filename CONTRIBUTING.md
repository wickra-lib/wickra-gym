# Contributing

Thanks for your interest in `wickra-gym`. This project values one thing above
all: **determinism**. Every change must preserve the guarantee that a fixed
`(spec, data)` yields a bit-identical `report_hash` in every language.

## Workflow

1. Branch from `main` (`feat/...`, `fix/...`, `docs/...`).
2. Make one logical change per commit; keep PRs focused (~5 units max).
3. Sign your commits and add a DCO `Signed-off-by` trailer (see [DCO](DCO)).
4. Open a PR against `main`, let CI go green, then squash-merge.

No AI/co-authored attribution, no emoji, and English for all public artifacts.

## Verify before pushing

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy --workspace --all-targets --no-default-features -- -D warnings
cargo test --workspace --all-features 2>&1 | grep "test result:"
cargo test -p gym-core --no-default-features 2>&1 | grep "test result:"
cargo deny check
```

For binding changes, run that binding's test suite (see per-binding READMEs
under `bindings/`). The Node binding regenerates `index.js` / `index.d.ts` —
commit them.

## Determinism rules (non-negotiable)

- Use `BTreeMap`, never `HashMap`, on any path that reaches a hash.
- No RNG, no time-of-day, no filesystem order in hashed output.
- Quantize floats via `round_to(_, 1e-8)` before serialization.
- Never hand-edit golden expected files; regenerate them with the bless flow.
