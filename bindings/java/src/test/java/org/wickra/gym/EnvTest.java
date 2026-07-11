package org.wickra.gym;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

class EnvTest {
    private static final String SPEC =
            "{\"dataset_ref\":\"smoke\",\"symbol\":\"TEST\","
                    + "\"observation\":{\"features\":[{\"kind\":\"price\",\"field\":\"close\"}]},"
                    + "\"action_space\":{\"type\":\"discrete\",\"n\":3},"
                    + "\"reward\":\"pnl\",\"episode\":{\"max_steps\":100,\"warmup\":0}}";

    private static String loadCommand() {
        StringBuilder sb = new StringBuilder("{\"cmd\":\"load\",\"candles\":[");
        for (int i = 0; i < 5; i++) {
            double p = 100.0 + i;
            if (i > 0) {
                sb.append(',');
            }
            sb.append("{\"ts\":").append(i)
                    .append(",\"open\":").append(p)
                    .append(",\"high\":").append(p)
                    .append(",\"low\":").append(p)
                    .append(",\"close\":").append(p).append('}');
        }
        return sb.append("]}").toString();
    }

    @Test
    void versionIsNonEmpty() {
        assertFalse(Env.version().isEmpty());
    }

    @Test
    void loadResetStep() {
        try (Env env = new Env(SPEC)) {
            assertEquals("{\"ok\":true}", env.command(loadCommand()));
            String reset = env.command("{\"cmd\":\"reset\"}");
            assertTrue(reset.contains("\"observation\""), reset);
            String step = env.command("{\"cmd\":\"step\",\"action\":2}");
            assertTrue(step.contains("\"reward\":1.0"), step);
            assertTrue(step.contains("\"terminated\":false"), step);
        }
    }

    @Test
    void badSpecThrows() {
        assertThrows(IllegalArgumentException.class, () -> new Env("{\"not\":\"a spec\"}"));
    }
}
