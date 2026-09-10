package org.wickra.gym;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

/**
 * The streamed rollout equals the batch tensor, through the command boundary.
 *
 * <p>The dataset is precomputed once into a fixed feature tensor — that is the
 * batch half — and {@code step()} then streams through it as a pure array index.
 * {@code gym-core} proves the two agree in Rust, but that says nothing about the
 * boundary this binding crosses: a binding that mis-serialised an observation
 * row, or truncated it, would hand back numbers that look plausible and are not
 * the tensor's.
 *
 * <p>So the rollout is driven twice through this binding and the two must be
 * byte-identical, a longer rollout must agree with a shorter one on the bars they
 * share, and the first observation must be the warmup bar the core computed.
 */
class StreamingTest {
    private static final String SPEC =
            "{\"dataset_ref\":\"streaming\",\"symbol\":\"TEST\","
                    + "\"observation\":{\"features\":["
                    + "{\"kind\":\"indicator\",\"name\":\"Sma\",\"params\":[3]},"
                    + "{\"kind\":\"price\",\"field\":\"close\"}]},"
                    + "\"action_space\":{\"type\":\"discrete\",\"n\":3},"
                    + "\"reward\":\"pnl\",\"episode\":{\"max_steps\":32,\"warmup\":3}}";

    private static final int STEPS = 6;

    private static String candles() {
        StringBuilder out = new StringBuilder("[");
        for (int i = 0; i < 20; i++) {
            if (i > 0) {
                out.append(',');
            }
            double f = i;
            out.append("{\"ts\":").append(i)
                    .append(",\"open\":").append(100.0 + f)
                    .append(",\"high\":").append(101.0 + f)
                    .append(",\"low\":").append(99.0 + f)
                    .append(",\"close\":").append(100.0 + f)
                    .append(",\"volume\":1}");
        }
        return out.append(']').toString();
    }

    private static Env loaded(String spec) {
        Env env = new Env(spec);
        env.command("{\"cmd\":\"load\",\"candles\":" + candles() + "}");
        return env;
    }

    private static String[] rollout(int steps) {
        Env env = loaded(SPEC);
        String[] trace = new String[steps + 1];
        trace[0] = env.command("{\"cmd\":\"reset\",\"seed\":7}");
        for (int i = 0; i < steps; i++) {
            trace[i + 1] = env.command("{\"cmd\":\"step\",\"action\":2.0}");
        }
        return trace;
    }

    @Test
    void theStreamedRolloutIsByteIdenticalWhenReplayed() {
        String[] first = rollout(STEPS);
        String[] second = rollout(STEPS);
        assertEquals(first.length, second.length);
        for (int i = 0; i < first.length; i++) {
            assertEquals(first[i], second[i], "step " + i + " differs between runs");
        }
    }

    @Test
    void aPrefixOfTheRolloutMatchesAShorterOne() {
        String[] longRun = rollout(STEPS);
        String[] shortRun = rollout(STEPS - 2);
        for (int i = 0; i < shortRun.length; i++) {
            assertEquals(shortRun[i], longRun[i],
                    "step " + i + " differs between runs of different length");
        }
    }

    @Test
    void theFirstObservationIsTheWarmupBar() {
        Env env = loaded(SPEC);
        String reset = env.command("{\"cmd\":\"reset\",\"seed\":7}");
        // Bar 3 closes at 103; Sma(3) over the three bars ending there is 102.
        assertTrue(reset.contains("\"observation\":[102.0,103.0]"), reset);
    }

    @Test
    void aWarmupBelowTheIndicatorLookbackIsRefused() {
        Env env = new Env(SPEC.replace("\"warmup\":3", "\"warmup\":1"));
        String response = env.command("{\"cmd\":\"load\",\"candles\":" + candles() + "}");
        assertTrue(response.contains("warmup"), response);
        assertFalse(response.contains("\"ok\":true"), response);
    }
}
