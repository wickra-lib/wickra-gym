# Wickra Gym examples — Java

Runnable Java examples for the [Wickra Gym Java binding](../../bindings/java). The binding reaches the C ABI
through the Foreign Function & Memory API (JDK 22+), so build the library once
and point the JVM at it with `-Dnative.lib.dir`:

```bash
cargo build -p wickra-gym-c --release
```

## Run

As the CI examples job runs it, from the repository root:

```bash
mvn -f bindings/java/pom.xml -q install -DskipTests
mvn -f examples/java/pom.xml -q compile exec:exec  -Dnative.lib.dir="$PWD/target/release"
```

## The examples

| Example | What it does |
|---------|--------------|
| `src/main/java/org/wickra/gym/examples/Rollout.java` | A runnable Java example: load the momentum_discrete spec and its candle dataset over the wickra-gym C ABI binding, then drive a fixed long policy and print each step. |
