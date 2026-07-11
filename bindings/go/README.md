# Wickra Gym — Go

Go bindings for [wickra-gym](https://github.com/wickra-lib/wickra-gym) — a
deterministic, Gymnasium-compatible backtest environment — over the C ABI via
cgo. A rollout is byte-identical to every other language binding.

## Install

```sh
go get github.com/wickra-lib/wickra-gym/bindings/go
```

The native library (`libwickra_gym`) must be available at build and run time:
place it under `lib/<os>_<arch>/` (git-ignored; staged from a release) and add
that directory to your library search path.

## Usage

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

## Header

`include/wickra_gym.h` is a copy of the canonical C ABI header
(`bindings/c/include/wickra_gym.h`); CI fails if the two drift.

## License

Dual-licensed under [MIT](https://github.com/wickra-lib/wickra-gym/blob/main/LICENSE-MIT)
or [Apache-2.0](https://github.com/wickra-lib/wickra-gym/blob/main/LICENSE-APACHE),
at your option.
