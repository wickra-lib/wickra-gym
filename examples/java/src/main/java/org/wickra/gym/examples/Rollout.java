// A runnable Java example: load the momentum_discrete spec and its candle
// dataset over the wickra-gym C ABI binding, then drive a fixed long policy and
// print each step. The rollout is byte-identical to the other language examples
// on the same seed.
//
//   cargo build --release -p wickra-gym-c
//   (cd bindings/java && mvn -q install -DskipTests)
//   mvn -q -f examples/java exec:exec
package org.wickra.gym.examples;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import org.wickra.gym.Env;

public final class Rollout {
    private static final Pattern REWARD = Pattern.compile("\"reward\":(-?[0-9.eE+-]+)");
    private static final Pattern TERMINATED = Pattern.compile("\"terminated\":(true|false)");
    private static final Pattern TRUNCATED = Pattern.compile("\"truncated\":(true|false)");

    private Rollout() {
    }

    private static Path dataDir() {
        Path dir = Path.of(System.getProperty("user.dir")).toAbsolutePath();
        for (int i = 0; i < 10 && dir != null; i++) {
            Path candidate = dir.resolve("examples").resolve("data");
            if (Files.isDirectory(candidate)) {
                return candidate;
            }
            dir = dir.getParent();
        }
        throw new IllegalStateException("examples/data not found");
    }

    private static double group1Double(Pattern p, String s) {
        Matcher m = p.matcher(s);
        if (!m.find()) {
            throw new IllegalStateException("missing field in: " + s);
        }
        return Double.parseDouble(m.group(1));
    }

    private static boolean group1Bool(Pattern p, String s) {
        Matcher m = p.matcher(s);
        if (!m.find()) {
            throw new IllegalStateException("missing field in: " + s);
        }
        return Boolean.parseBoolean(m.group(1));
    }

    public static void main(String[] args) throws IOException {
        Path data = dataDir();
        String spec = Files.readString(data.resolve("specs").resolve("momentum_discrete.json"));
        String candles = Files.readString(data.resolve("candles.json"));

        try (Env env = new Env(spec)) {
            env.command("{\"cmd\":\"load\",\"candles\":" + candles + "}");
            System.out.println("reset: " + env.command("{\"cmd\":\"reset\",\"seed\":42}"));

            double equity = 0.0;
            for (int step = 0; ; step++) {
                String result = env.command("{\"cmd\":\"step\",\"action\":2}");
                double reward = group1Double(REWARD, result);
                boolean terminated = group1Bool(TERMINATED, result);
                boolean truncated = group1Bool(TRUNCATED, result);
                equity += reward;
                System.out.printf(
                        "step %d: reward %+.6f  equity %+.6f  terminated=%b truncated=%b%n",
                        step, reward, equity, terminated, truncated);
                if (terminated || truncated) {
                    break;
                }
            }
        }
    }
}
