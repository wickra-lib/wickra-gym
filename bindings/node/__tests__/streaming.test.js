"use strict";

// The streamed rollout equals the batch tensor, through the command boundary.
//
// The dataset is precomputed once into a fixed feature tensor -- that is the batch
// half -- and `step()` then streams through it as a pure array index. `gym-core`
// proves the two agree in Rust, but that says nothing about the boundary this
// binding crosses: a binding that mis-serialised an observation row, or truncated
// it, would hand back numbers that look plausible and are not the tensor's.
//
// So the rollout is driven twice through this binding and the two must be
// byte-identical, a longer rollout must agree with a shorter one on the bars they
// share, and the first observation must be the warmup bar the core computed.

const { test } = require("node:test");
const assert = require("node:assert");
const { Env } = require("../index.js");

const SPEC = JSON.stringify({"dataset_ref":"streaming","symbol":"TEST","observation":{"features":[{"kind":"indicator","name":"Sma","params":[3]},{"kind":"price","field":"close"}]},"action_space":{"type":"discrete","n":3},"reward":"pnl","episode":{"max_steps":32,"warmup":3}});

const CANDLES = Array.from({ length: 20 }, (_, i) => ({
  ts: i,
  open: 100.0 + i,
  high: 101.0 + i,
  low: 99.0 + i,
  close: 100.0 + i,
  volume: 1.0,
}));

const STEPS = 6;

function loaded() {
  const env = new Env(SPEC);
  env.command(JSON.stringify({ cmd: "load", candles: CANDLES }));
  return env;
}

function rollout(steps) {
  const env = loaded();
  const trace = [env.command(JSON.stringify({ cmd: "reset", seed: 7 }))];
  for (let i = 0; i < steps; i++) {
    trace.push(env.command(JSON.stringify({ cmd: "step", action: 2.0 })));
  }
  return trace;
}

test("the streamed rollout is byte-identical when replayed", () => {
  assert.deepStrictEqual(rollout(STEPS), rollout(STEPS));
});

test("a prefix of the rollout matches a shorter one", () => {
  const long = rollout(STEPS);
  const short = rollout(STEPS - 2);
  assert.deepStrictEqual(long.slice(0, short.length), short);
});

test("the first observation is the warmup bar of the tensor", () => {
  const env = loaded();
  const reset = JSON.parse(env.command(JSON.stringify({ cmd: "reset", seed: 7 })));
  // Bar 3 closes at 103; Sma(3) over the three bars ending there is 102.
  assert.strictEqual(reset.observation[0], 102.0);
  assert.strictEqual(reset.observation[1], 103.0);
});

test("a warmup below the indicator lookback is refused", () => {
  const spec = JSON.parse(SPEC);
  spec.episode.warmup = 1;
  const env = new Env(JSON.stringify(spec));
  const response = env.command(JSON.stringify({ cmd: "load", candles: CANDLES }));
  assert.match(response, /warmup/, response);
  assert.doesNotMatch(response, /"ok":true/, response);
});
