# Wickra Gym — Node.js

A deterministic, Gymnasium-compatible backtest environment, powered by Rust. The
whole candle dataset is precomputed once into a fixed feature tensor, so each
`step` is a constant-time array index — and the same spec, data, seed and actions
produce a byte-identical trajectory across every language binding.

## Install

```sh
npm install wickra-gym
```

The correct native binary is pulled in automatically as an optional dependency
for your platform (Linux / macOS / Windows, x64 / arm64).

## Usage

```js
const { Env } = require('wickra-gym')

const spec = JSON.stringify({
  dataset_ref: 'demo',
  symbol: 'BTCUSDT',
  observation: { features: [
    { kind: 'price', field: 'close' },
    { kind: 'indicator', name: 'Rsi', params: [14] },
  ] },
  action_space: { type: 'discrete', n: 3 },
  reward: 'pnl',
  episode: { max_steps: 256, warmup: 14 },
})

const candles = Array.from({ length: 300 }, (_, i) => ({
  ts: i, open: 100 + i, high: 100 + i, low: 100 + i, close: 100 + i,
}))

const env = new Env(spec)
env.command(JSON.stringify({ cmd: 'load', candles }))
const reset = JSON.parse(env.command(JSON.stringify({ cmd: 'reset', seed: 0 })))
const step = JSON.parse(env.command(JSON.stringify({ cmd: 'step', action: 2 })))
console.log(step.reward, step.terminated)
```

The surface is a single JSON command boundary — the same one every language
binding forwards verbatim. Commands: `load`, `reset`, `step`, `spec`, `version`.
Domain errors come back as `{"ok": false, "error": ...}`; a bad spec throws at
construction.

## Determinism

`spec`, `data`, `seed` and the action sequence fully determine a byte-identical
`{reset, trajectory}`. See the [main repository](https://github.com/wickra-lib/wickra-gym)
for the observation and reward semantics.

## License

Dual-licensed under [MIT](https://github.com/wickra-lib/wickra-gym/blob/main/LICENSE-MIT)
or [Apache-2.0](https://github.com/wickra-lib/wickra-gym/blob/main/LICENSE-APACHE),
at your option.
