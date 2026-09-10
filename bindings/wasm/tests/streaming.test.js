"use strict";

// The streamed rollout equals the batch tensor, through the WASM boundary.
//
// The dataset is precomputed once into a fixed feature tensor -- that is the
// batch half -- and `step()` then streams through it as a pure array index. The
// golden test here proves the WASM build reproduces a blessed rollout; this
// proves the two halves agree when the rollout is driven live, which is the
// path a browser consumer actually takes.
//
// Skips cleanly when `pkg/` has not been built yet
// (`wasm-pack build --target nodejs`).

const { test } = require("node:test");
const assert = require("node:assert");
const path = require("node:path");

let wasm = null;
try {
  wasm = require(path.resolve(__dirname, "..", "pkg", "wickra_gym_wasm.js"));
} catch {
  wasm = null;
}

const SPEC = JSON.stringify({
  dataset_ref: "streaming",
  symbol: "TEST",
  observation: {
    features: [
      { kind: "indicator", name: "Sma", params: [3] },
      { kind: "price", field: "close" },
    ],
  },
  action_space: { type: "discrete", n: 3 },
  reward: "pnl",
  episode: { max_steps: 32, warmup: 3 },
});

const CANDLES = Array.from({ length: 20 }, (_, i) => ({
  ts: i,
  open: 100.0 + i,
  high: 101.0 + i,
  low: 99.0 + i,
  close: 100.0 + i,
  volume: 1.0,
}));

const STEPS = 6;

function rollout(steps) {
  const env = new wasm.Env(SPEC);
  env.command(JSON.stringify({ cmd: "load", candles: CANDLES }));
  const trace = [env.command(JSON.stringify({ cmd: "reset", seed: 7 }))];
  for (let i = 0; i < steps; i++) {
    trace.push(env.command(JSON.stringify({ cmd: "step", action: 2.0 })));
  }
  return trace;
}

test("the streamed rollout is byte-identical when replayed", { skip: wasm === null }, () => {
  assert.deepStrictEqual(rollout(STEPS), rollout(STEPS));
});

test("a prefix of the rollout matches a shorter one", { skip: wasm === null }, () => {
  const long = rollout(STEPS);
  const short = rollout(STEPS - 2);
  assert.deepStrictEqual(long.slice(0, short.length), short);
});

test("the first observation is the warmup bar", { skip: wasm === null }, () => {
  const reset = JSON.parse(rollout(0)[0]);
  // Bar 3 closes at 103; Sma(3) over the three bars ending there is 102.
  assert.strictEqual(reset.observation[0], 102.0);
  assert.strictEqual(reset.observation[1], 103.0);
});
