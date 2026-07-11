# Wickra Gym — WASM

A deterministic, Gymnasium-compatible backtest environment compiled to
WebAssembly, for running rollouts directly in the browser (or any WASM host).
The feature-tensor precompute runs sequentially here — byte-identical to the
native parallel build — so a WASM rollout matches every other language binding.

## Build

```sh
wasm-pack build --target web       # for bundlers / browsers
wasm-pack build --target nodejs    # for Node.js (used by the tests)
```

## Usage

```js
import init, { Env } from "./pkg/wickra_gym_wasm.js";

await init();

const spec = JSON.stringify({
  dataset_ref: "demo",
  symbol: "BTCUSDT",
  observation: { features: [{ kind: "price", field: "close" }] },
  action_space: { type: "discrete", n: 3 },
  reward: "pnl",
  episode: { max_steps: 256, warmup: 0 },
});

const candles = Array.from({ length: 300 }, (_, i) => ({
  ts: i, open: 100 + i, high: 100 + i, low: 100 + i, close: 100 + i,
}));

const env = new Env(spec);
env.command(JSON.stringify({ cmd: "load", candles }));
const reset = JSON.parse(env.command(JSON.stringify({ cmd: "reset", seed: 0 })));
const step = JSON.parse(env.command(JSON.stringify({ cmd: "step", action: 2 })));
```

The surface is a single JSON command boundary — the same one every language
binding forwards verbatim. Commands: `load`, `reset`, `step`, `spec`, `version`.
Domain errors come back as `{"ok": false, "error": ...}`; a bad spec throws at
construction.

## License

Dual-licensed under [MIT](https://github.com/wickra-lib/wickra-gym/blob/main/LICENSE-MIT)
or [Apache-2.0](https://github.com/wickra-lib/wickra-gym/blob/main/LICENSE-APACHE),
at your option.
