# wickra-gym WASM examples

Browser demos for the `wickra-gym-wasm` binding.

The WASM build carries the whole environment core with `--no-default-features`:
the tensor is precomputed sequentially rather than in parallel, and byte-for-byte
identically, which is what the golden rollouts pin down. A spec is data, not
code, so the bytes on this page are the same ones `examples/node/rollout.js`
sends, and the trajectory is the same trajectory.

## Build

The module ships as a `wasm-pack` `--target web` bundle. Build it once from the
repository root:

```bash
wasm-pack build bindings/wasm --target web --release
```

That writes `bindings/wasm/pkg/` with the `.wasm` binary, the JS loader and the
type declarations the page imports.

## Run

The page loads its module over `http://`, not `file://`, because ES module
imports and `WebAssembly.instantiateStreaming` both need a real origin. Serve the
repository root:

```bash
python -m http.server 8000
```

Then open `http://localhost:8000/examples/wasm/rollout.html`.

## Pages

| Page | What it does |
|------|--------------|
| `rollout.html` | Loads a twenty-bar dataset, resets with seed 7 and takes six steps of the same action, showing the observation and reward per step plus the raw JSON. The page counterpart of `examples/node/rollout.js`. |

## See also

- [examples/README.md](../README.md) — the same rollout in every other language.
- [bindings/wasm/README.md](../../bindings/wasm/README.md) — the binding itself.
