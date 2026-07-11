"use strict";

// Golden test over the wasm-pack (nodejs target) output: the WebAssembly build
// runs a rollout byte-identically to the native run. Skips cleanly when `pkg/`
// has not been built yet (`wasm-pack build --target nodejs`).

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");

let wasm = null;
try {
  wasm = require(path.resolve(__dirname, "..", "pkg", "wickra_gym_wasm.js"));
} catch {
  wasm = null;
}

const SPEC = JSON.stringify({
  dataset_ref: "wasm",
  symbol: "TEST",
  observation: {
    features: [
      { kind: "price", field: "close" },
      { kind: "indicator", name: "Sma", params: [3] },
    ],
  },
  action_space: { type: "discrete", n: 3 },
  reward: "pnl",
  episode: { max_steps: 32, warmup: 3 },
});

function candles() {
  const out = [];
  for (let i = 0; i < 40; i++) {
    const base = 100.0 + Math.sin(i * 0.4) * 8.0;
    out.push({ ts: i, open: base, high: base + 1, low: base - 1, close: base + 0.5 });
  }
  return out;
}

function rollout() {
  const env = new wasm.Env(SPEC);
  env.command(JSON.stringify({ cmd: "load", candles: candles() }));
  const reset = env.command(JSON.stringify({ cmd: "reset", seed: 7 }));
  const trajectory = [];
  for (let i = 0; i < 6; i++) {
    trajectory.push(env.command(JSON.stringify({ cmd: "step", action: (i % 3) })));
  }
  return { reset, trajectory };
}

test("wasm rollout is deterministic", { skip: wasm === null ? "pkg/ not built (run wasm-pack build --target nodejs)" : false }, () => {
  const a = rollout();
  const b = rollout();
  assert.deepStrictEqual(a, b);
  assert.match(a.reset, /"observation"/);
});

// bindings/wasm/tests -> repo root -> golden/
const GOLDEN = path.resolve(__dirname, "..", "..", "..", "golden");

function goldenCases() {
  if (!fs.existsSync(GOLDEN)) return [];
  return fs
    .readdirSync(GOLDEN, { withFileTypes: true })
    .filter((d) => d.isDirectory() && fs.existsSync(path.join(GOLDEN, d.name, "spec.json")))
    .map((d) => d.name);
}

const CASES = wasm === null ? [] : goldenCases();

test("cross-language golden rollouts", { skip: CASES.length === 0 ? "golden fixtures not present yet (or pkg/ not built)" : false }, () => {
  for (const name of CASES) {
    const dir = path.join(GOLDEN, name);
    const spec = fs.readFileSync(path.join(dir, "spec.json"), "utf8");
    const cndl = JSON.parse(fs.readFileSync(path.join(dir, "candles.json"), "utf8"));
    const expected = JSON.parse(fs.readFileSync(path.join(dir, "expected.json"), "utf8"));

    const env = new wasm.Env(spec);
    env.command(JSON.stringify({ cmd: "load", candles: cndl }));
    const resetCmd = expected.seed != null ? { cmd: "reset", seed: expected.seed } : { cmd: "reset" };
    const reset = JSON.parse(env.command(JSON.stringify(resetCmd)));
    assert.deepStrictEqual(reset, expected.reset, `${name}: reset`);
    const trajectory = expected.actions.map((action) =>
      JSON.parse(env.command(JSON.stringify({ cmd: "step", action })))
    );
    assert.deepStrictEqual(trajectory, expected.trajectory, `${name}: trajectory`);
  }
});
