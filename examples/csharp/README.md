# Wickra Gym examples — C#

Runnable C# examples for the [Wickra Gym C# binding](../../bindings/csharp). The binding consumes the C ABI
library through P/Invoke, so build it once before running anything:

```bash
cargo build -p wickra-gym-c --release
```

## Run

As the CI examples job runs it, from the repository root:

```bash
dotnet run --project examples/csharp/Rollout
```

## The examples

| Example | What it does |
|---------|--------------|
| `Rollout/Program.cs` | A runnable C# example: load the momentum_discrete spec and its candle dataset over the wickra-gym C ABI binding, then drive a fixed long policy and print each step. |
