<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Gym — a Gymnasium-compatible, microstructure-aware backtest environment with O(1) steps for fast, deterministic RL rollouts" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/ci.svg)](https://github.com/wickra-lib/wickra-gym/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-gym)
[![Go module](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/go.svg)](https://pkg.go.dev/github.com/wickra-lib/wickra-gym/bindings/go)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/license.svg)](https://github.com/wickra-lib/wickra-gym#license)

# Wickra Gym — Go

---

**Part of the [Wickra ecosystem](https://github.com/wickra-lib) — for Go. `go get github.com/wickra-lib/wickra-gym/bindings/go` — over the C ABI via cgo, prebuilt library bundled in the module.**

Go bindings for [wickra-gym](https://github.com/wickra-lib/wickra-gym) — a
deterministic, Gymnasium-compatible backtest environment — over the C ABI via
cgo. A rollout is byte-identical to every other language binding.

## Install

Use the published **`wickra-gym-go`** module, which bundles the prebuilt C ABI
library for every platform, so `go get` + `go build` works with no extra steps
(a C compiler is still required, as the binding uses cgo):

```bash
go get github.com/wickra-lib/wickra-gym-go
```

`wickra-gym-go` is generated from this directory by the release pipeline: it mirrors
the Go sources, the vendored C ABI header (`include/wickra_gym.h`) and the prebuilt
libraries under `lib/<goos>_<goarch>/`. On Linux/macOS the library path is baked
in via rpath; on Windows the DLL must be discoverable at run time (next to the
executable or on `PATH`).

The native library (`libwickra_gym`) must be available at build and run time:
place it under `lib/<os>_<arch>/` (git-ignored; staged from a release) and add
that directory to your library search path.

### Building from this repository (contributors)

This `bindings/go` directory is the development source. To build it directly,
compile the C ABI and stage the library into the per-platform directory cgo
links against:

```bash
cargo build -p wickra-gym-c --release
mkdir -p bindings/go/lib/linux_amd64
cp target/release/libwickra_gym.so bindings/go/lib/linux_amd64/
```

Then, with the library on the loader path, run `go test ./...` from this directory.

## Quick start

```go
package main

import (
	"encoding/json"
	"fmt"

	wickra "github.com/wickra-lib/wickra-gym/bindings/go"
)

func main() {
	spec := `{"dataset_ref":"demo","symbol":"BTCUSDT",
	  "observation":{"features":[{"kind":"price","field":"close"}]},
	  "action_space":{"type":"discrete","n":3},
	  "reward":"pnl","episode":{"max_steps":256,"warmup":0}}`

	env, err := wickra.New(spec)
	if err != nil {
		panic(err)
	}
	defer env.Free()

	candles := make([]map[string]float64, 300)
	for i := range candles {
		p := 100.0 + float64(i)
		candles[i] = map[string]float64{"ts": float64(i), "open": p, "high": p, "low": p, "close": p}
	}
	load, _ := json.Marshal(map[string]any{"cmd": "load", "candles": candles})
	env.Command(string(load))

	reset, _ := env.Command(`{"cmd":"reset","seed":0}`)
	fmt.Println(reset)
	step, _ := env.Command(`{"cmd":"step","action":2}`)
	fmt.Println(step)
}
```

Commands: `load`, `reset`, `step`, `spec`, `version`. Domain errors come back as
`{"ok":false,"error":...}`; a bad spec returns an error from `New`.

### Header

`include/wickra_gym.h` is a copy of the canonical C ABI header
(`bindings/c/include/wickra_gym.h`); CI fails if the two drift.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of cgo over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-gym/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-gym>
- **Docs** (guides, spec reference, cookbook): <https://gym.wickra.org>
- **Runnable example:** [`examples/go/`](https://github.com/wickra-lib/wickra-gym/tree/main/examples/go)

Wickra Gym ships native bindings for Python, Node.js, WASM and Rust, plus a C ABI hub that any
C-capable language (C, C++, C#, Go, Java, R) links against — all forwarding to the
same data-driven, `unsafe`-forbidden Rust core.

## Security

Found a security issue? **Please don't open a public issue.** Report it privately
via the repository's *Security* tab (*"Report a vulnerability"*) or email
**support@wickra.org** with a subject line starting `[wickra security]`. Full
policy: <https://github.com/wickra-lib/wickra-gym/blob/main/SECURITY.md>.

## Disclaimer

`wickra-gym` is research and engineering tooling, not financial advice. A trained
agent's backtested performance says nothing about future returns; markets carry
risk and you are responsible for your own decisions. `wickra-gym` is free
software you run yourself: no hosted service, no data collection, no warranty.

## License

Licensed under either of [Apache-2.0](https://github.com/wickra-lib/wickra-gym/blob/main/LICENSE-APACHE)
or [MIT](https://github.com/wickra-lib/wickra-gym/blob/main/LICENSE-MIT) at your option.
