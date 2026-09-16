# Wickra Gym examples — R

Runnable R examples for the [Wickra Gym R binding](../../bindings/r). The package compiles a thin
`.Call` glue layer against the C ABI library, so build the library and install
the package first (the CI examples job does exactly this):

```bash
cargo build -p wickra-gym-c --release
R CMD INSTALL bindings/r
```

## Run

```bash
Rscript examples/r/rollout.R
```

## The examples

| Example | What it does |
|---------|--------------|
| `rollout.R` | A runnable R example: load the momentum_discrete spec and its candle dataset over the wickra-gym C ABI binding, then drive a fixed long policy and print each step. |
