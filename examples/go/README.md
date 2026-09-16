# Wickra Gym examples — Go

Runnable Go examples for the [Wickra Gym Go binding](../../bindings/go). The binding links against the
prebuilt C ABI library, so build and stage it once before running anything:

```bash
cargo build -p wickra-gym-c --release
mkdir -p bindings/go/lib/linux_amd64
cp target/release/libwickra_gym.so bindings/go/lib/linux_amd64/
```

## Run

As the CI examples job runs it, from the repository root:

```bash
cd examples/go && go run .
```

## The examples

| Example | What it does |
|---------|--------------|
| `rollout.go` | A runnable Go example: load the momentum_discrete spec and its candle dataset over the wickra-gym C ABI binding, then drive a fixed long policy and print each step. |
