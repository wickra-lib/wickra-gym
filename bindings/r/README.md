# Wickra Gym — R

R bindings for [wickra-gym](https://github.com/wickra-lib/wickra-gym) — a
deterministic, Gymnasium-compatible backtest environment — over the C ABI, via
R's `.Call` interface. A rollout is byte-identical to every other language
binding.

## Install

The package links the native `wickra_gym` C ABI library. Point the build at its
header and library directories:

```sh
export WKGYM_INC=/path/to/wickra-gym/bindings/c/include
export WKGYM_LIB=/path/to/wickra-gym/target/release   # holds libwickra_gym.*
R CMD INSTALL bindings/r
```

At run time the loader must find the shared library (via `PATH` on Windows,
`LD_LIBRARY_PATH` on Linux, `DYLD_LIBRARY_PATH` on macOS).

## Usage

```r
library(wickragym)

spec <- paste0(
  '{"dataset_ref":"demo","symbol":"BTCUSDT",',
  '"observation":{"features":[{"kind":"price","field":"close"}]},',
  '"action_space":{"type":"discrete","n":3},',
  '"reward":"pnl","episode":{"max_steps":256,"warmup":0}}'
)

env <- wkgym_new(spec)

candles <- paste0(
  '[', paste(vapply(0:299, function(i) {
    p <- 100 + i
    sprintf('{"ts":%d,"open":%f,"high":%f,"low":%f,"close":%f}', i, p, p, p, p)
  }, character(1)), collapse = ","), ']'
)
wkgym_command(env, paste0('{"cmd":"load","candles":', candles, '}'))

reset <- wkgym_command(env, '{"cmd":"reset","seed":0}')
step  <- wkgym_command(env, '{"cmd":"step","action":2}')
cat(step, "\n")
```

Commands: `load`, `reset`, `step`, `spec`, `version`. Domain errors come back as
`{"ok":false,"error":...}`; a bad spec raises an R error.

## License

Dual-licensed under [MIT](https://github.com/wickra-lib/wickra-gym/blob/main/LICENSE-MIT)
or [Apache-2.0](https://github.com/wickra-lib/wickra-gym/blob/main/LICENSE-APACHE),
at your option.
