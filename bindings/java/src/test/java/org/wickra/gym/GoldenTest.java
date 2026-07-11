package org.wickra.gym;

import static org.junit.jupiter.api.Assertions.assertEquals;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.stream.Stream;
import org.junit.jupiter.api.Assumptions;
import org.junit.jupiter.api.Test;

/**
 * Cross-language golden parity: replay each committed rollout and assert the
 * response equals the expected JSON. The binding returns the core's canonical
 * command_json string verbatim, so byte equality is the exact parity check.
 * Skips cleanly until the golden fixtures land (§4).
 */
class GoldenTest {
    // bindings/java -> repo root -> golden/
    private static final Path GOLDEN = Path.of(System.getProperty("user.dir"), "..", "..", "golden");

    @Test
    void goldenRollouts() throws IOException {
        Assumptions.assumeTrue(Files.isDirectory(GOLDEN), "golden fixtures not present yet");

        try (Stream<Path> dirs = Files.list(GOLDEN)) {
            for (Path dir : dirs.filter(Files::isDirectory).toList()) {
                Path specPath = dir.resolve("spec.json");
                if (!Files.exists(specPath)) {
                    continue;
                }
                String spec = Files.readString(specPath);
                String candles = Files.readString(dir.resolve("candles.json"));
                String expected = Files.readString(dir.resolve("expected.json"));

                List<String> actions = jsonArrayOf(expected, "actions");
                List<String> trajectory = jsonObjectsOf(expected, "trajectory");
                String resetExpected = jsonObject(expected, "reset");
                String seed = scalar(expected, "seed");

                try (Env env = new Env(spec)) {
                    env.command("{\"cmd\":\"load\",\"candles\":" + candles + "}");
                    String resetCmd = seed != null
                            ? "{\"cmd\":\"reset\",\"seed\":" + seed + "}"
                            : "{\"cmd\":\"reset\"}";
                    assertEquals(canon(resetExpected), canon(env.command(resetCmd)), dir + " reset");
                    for (int i = 0; i < actions.size(); i++) {
                        String step = env.command("{\"cmd\":\"step\",\"action\":" + actions.get(i) + "}");
                        assertEquals(canon(trajectory.get(i)), canon(step), dir + " step " + i);
                    }
                }
            }
        }
    }

    // Minimal, whitespace-insensitive canonicalization: the core already emits
    // compact JSON, so stripping insignificant whitespace suffices for parity.
    private static String canon(String json) {
        StringBuilder out = new StringBuilder(json.length());
        boolean inString = false;
        boolean escaped = false;
        for (int i = 0; i < json.length(); i++) {
            char c = json.charAt(i);
            if (inString) {
                out.append(c);
                if (escaped) {
                    escaped = false;
                } else if (c == '\\') {
                    escaped = true;
                } else if (c == '"') {
                    inString = false;
                }
            } else if (c == '"') {
                inString = true;
                out.append(c);
            } else if (!Character.isWhitespace(c)) {
                out.append(c);
            }
        }
        return out.toString();
    }

    private static String scalar(String json, String key) {
        int at = valueStart(json, key);
        if (at < 0) {
            return null;
        }
        int end = at;
        while (end < json.length() && ",}] \n\r\t".indexOf(json.charAt(end)) < 0) {
            end++;
        }
        String v = json.substring(at, end).trim();
        return v.equals("null") ? null : v;
    }

    private static String jsonObject(String json, String key) {
        int at = valueStart(json, key);
        return sliceBalanced(json, at, '{', '}');
    }

    private static List<String> jsonArrayOf(String json, String key) {
        int at = valueStart(json, key);
        String arr = sliceBalanced(json, at, '[', ']');
        return splitTopLevel(arr.substring(1, arr.length() - 1));
    }

    private static List<String> jsonObjectsOf(String json, String key) {
        int at = valueStart(json, key);
        String arr = sliceBalanced(json, at, '[', ']');
        return splitTopLevel(arr.substring(1, arr.length() - 1));
    }

    private static int valueStart(String json, String key) {
        int k = json.indexOf("\"" + key + "\"");
        if (k < 0) {
            return -1;
        }
        int colon = json.indexOf(':', k);
        int at = colon + 1;
        while (at < json.length() && Character.isWhitespace(json.charAt(at))) {
            at++;
        }
        return at;
    }

    private static String sliceBalanced(String json, int at, char open, char close) {
        int depth = 0;
        boolean inString = false;
        boolean escaped = false;
        for (int i = at; i < json.length(); i++) {
            char c = json.charAt(i);
            if (inString) {
                if (escaped) {
                    escaped = false;
                } else if (c == '\\') {
                    escaped = true;
                } else if (c == '"') {
                    inString = false;
                }
            } else if (c == '"') {
                inString = true;
            } else if (c == open) {
                depth++;
            } else if (c == close) {
                depth--;
                if (depth == 0) {
                    return json.substring(at, i + 1);
                }
            }
        }
        throw new IllegalStateException("unbalanced JSON");
    }

    private static List<String> splitTopLevel(String body) {
        List<String> parts = new ArrayList<>();
        int depth = 0;
        boolean inString = false;
        boolean escaped = false;
        int start = 0;
        for (int i = 0; i < body.length(); i++) {
            char c = body.charAt(i);
            if (inString) {
                if (escaped) {
                    escaped = false;
                } else if (c == '\\') {
                    escaped = true;
                } else if (c == '"') {
                    inString = false;
                }
            } else if (c == '"') {
                inString = true;
            } else if (c == '{' || c == '[') {
                depth++;
            } else if (c == '}' || c == ']') {
                depth--;
            } else if (c == ',' && depth == 0) {
                parts.add(body.substring(start, i).trim());
                start = i + 1;
            }
        }
        String last = body.substring(start).trim();
        if (!last.isEmpty()) {
            parts.add(last);
        }
        return parts;
    }
}
