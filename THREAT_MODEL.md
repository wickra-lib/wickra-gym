# Threat model

`wickra-gym` is a local library and CLI: it turns a candle dataset and a JSON
spec into a reinforcement-learning environment. It has no network surface in its
default build, holds no secrets, and stores no user data. The threat model is
therefore narrow.

## Assets

- **Determinism** — the guarantee that a `(seed, policy)` pair yields a
  byte-identical trajectory across all ten languages. This is the core value; a
  silent divergence is the primary "security-relevant" failure.
- **Supply-chain integrity** — the crate graph, the two git dependencies
  (`wickra-backtest`, `wickra-exchange`), and the published packages.

## Actors

- **The user** — runs the CLI or a binding locally over their own data. Trusted.
- **A dataset / spec author** — supplies the JSON spec and candle CSV. Untrusted
  input: malformed specs, out-of-range parameters, and adversarial numbers must
  produce a clean error, never a panic across the FFI boundary or a nondeterministic
  result.
- **A dependency** — a compromised upstream crate. Mitigated by pinned versions,
  `cargo-deny` (advisories, licenses, bans, sources), OSV scanning, and OpenSSF
  Scorecard.

## Threats and mitigations

- **Nondeterminism** (a divergent trajectory) — the determinism invariants
  (`BTreeMap`, fixed observation layout, seeded-only RNG, fixed reduction order)
  plus cross-language golden tests and a parallel-equals-sequential test.
- **Panic across the C ABI** — the C ABI hub catches unwinds and the release
  profile uses `panic = "abort"`; malformed input returns an in-band error.
- **Malicious input** — `deny_unknown_fields` on the spec, bounded parameters,
  and fuzz targets over the parse and step surfaces.
- **Supply-chain compromise** — pinned dependencies, `cargo-deny`, OSV-Scanner,
  CodeQL, zizmor (workflow audit), and SLSA build provenance on release.

## Out of scope

`wickra-gym` makes no claim about the profitability of any agent trained against
it, nor about whether the input data is genuine. It is research tooling, not a
trading system and not financial advice.

## Reporting

See [SECURITY.md](SECURITY.md) for private vulnerability reporting.
