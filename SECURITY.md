# Security Policy

`wickra-gym` is a reproducibility tool: it recomputes a deterministic
backtest report and a canonical hash from a curated `(strategy, dataset)` case
and compares them against a frozen expectation. It places no orders, holds no
secret material, and opens no authenticated connections. The attack surface is
therefore narrow — principally the parsing of untrusted `BenchmarkCase`
(strategy + expected report) and candle data JSON as it crosses the C ABI and
WASM boundary. See [THREAT_MODEL.md](THREAT_MODEL.md) for the asset and
trust-boundary breakdown.

## Supported versions

Until the first stable release, only the latest `0.x` minor line receives
security fixes.

| Version | Supported |
|---------|-----------|
| latest `0.x` | :white_check_mark: |
| older        | :x:       |

## Reporting a vulnerability

Please report suspected vulnerabilities privately via GitHub's
[security advisories](https://github.com/wickra-lib/wickra-gym/security/advisories/new)
form rather than a public issue. We aim to acknowledge within 72 hours.

Because the entire product value is **determinism**, a report that a given
`(spec, data)` yields a **different `report_hash` across two languages or two
runs** is treated as a security-class defect: it breaks the verifiability
guarantee. Please include the spec, the data, and both hashes.

## Scope

In scope: memory-safety or panic issues reachable by parsing untrusted input at
the FFI/WASM boundary; any cross-language or cross-run divergence of
`report_hash` for identical input; canonicalization ambiguities.

Out of scope: the correctness of a strategy's economics; third-party registries;
denial of service from pathologically large inputs supplied by the caller.
