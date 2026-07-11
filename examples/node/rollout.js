// A minimal Node.js rollout over the wickra-gym command surface.
//
//   npm install && node rollout.js
//
// Loads the momentum_discrete spec and its candle dataset, then drives a fixed
// long policy through the environment via Env.command — the same JSON-in /
// JSON-out boundary the Python, C, Go, C#, Java and R examples use, so this
// trajectory is byte-identical to theirs on the same seed.

const fs = require("fs");
const path = require("path");
const { Env } = require("wickra-gym");

const DATA = path.resolve(__dirname, "..", "data");

function main() {
  const spec = fs.readFileSync(path.join(DATA, "specs", "momentum_discrete.json"), "utf8");
  const candles = JSON.parse(fs.readFileSync(path.join(DATA, "candles.json"), "utf8"));

  const env = new Env(spec);
  env.command(JSON.stringify({ cmd: "load", candles }));

  const reset = JSON.parse(env.command(JSON.stringify({ cmd: "reset", seed: 42 })));
  console.log("reset observation:", reset.observation);

  let equity = 0;
  for (let step = 0; ; step += 1) {
    const result = JSON.parse(env.command(JSON.stringify({ cmd: "step", action: 2 })));
    equity += result.reward;
    console.log(
      `step ${step}: reward ${result.reward.toFixed(6)}  equity ${equity.toFixed(6)}  ` +
        `terminated=${result.terminated} truncated=${result.truncated}`
    );
    if (result.terminated || result.truncated) {
      break;
    }
  }
}

main();
