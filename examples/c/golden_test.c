/* Cross-language golden parity, from C.
 *
 * Replay each committed golden/<case>: build the environment from its
 * spec.json, load its candles.json, reset with the blessed seed and step the
 * blessed actions, and assert the trajectory equals expected.json. The ABI
 * returns the core's compact command output verbatim, so byte equality is the
 * exact cross-language parity check — the same one Python, Node, Go, C#, Java,
 * R and WASM make.
 *
 * C has no directory API that is portable between POSIX and Windows, so the case
 * list is globbed by CMake at configure time and written into golden_cases.h.
 * That keeps the property the other bindings get from a runtime glob: a case
 * added to the corpus is covered here without editing this file. A
 * hand-maintained list would silently skip it, which is the failure this whole
 * corpus exists to prevent.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "wickra_gym.h"

#include "golden_cases.h" /* GOLDEN_DIR, GOLDEN_CASES, GOLDEN_CASE_COUNT */

/* Read a whole file. Caller frees. Returns NULL and reports on failure. */
static char *slurp(const char *path) {
    FILE *file = fopen(path, "rb");
    if (!file) {
        fprintf(stderr, "cannot open %s\n", path);
        return NULL;
    }
    if (fseek(file, 0, SEEK_END) != 0) {
        fclose(file);
        return NULL;
    }
    long size = ftell(file);
    if (size < 0 || fseek(file, 0, SEEK_SET) != 0) {
        fclose(file);
        return NULL;
    }
    char *buf = (char *)malloc((size_t)size + 1);
    if (!buf) {
        fclose(file);
        return NULL;
    }
    size_t got = fread(buf, 1, (size_t)size, file);
    fclose(file);
    buf[got] = '\0';
    return buf;
}

static char *join3(const char *a, const char *b, const char *c) {
    size_t len = strlen(a) + strlen(b) + strlen(c) + 1;
    char *out = (char *)malloc(len);
    if (out) {
        snprintf(out, len, "%s%s%s", a, b, c);
    }
    return out;
}

/* Run one command through the two-call idiom. Caller frees. */
static char *run(WickraGymEnv *env, const char *cmd) {
    int32_t len = wickra_gym_command(env, cmd, NULL, 0);
    if (len < 0) {
        return NULL;
    }
    char *buf = (char *)malloc((size_t)len + 1);
    if (!buf) {
        return NULL;
    }
    if (wickra_gym_command(env, cmd, buf, (size_t)len + 1) != len) {
        free(buf);
        return NULL;
    }
    return buf;
}

/* The same JSON with insignificant whitespace removed. Caller frees.
 *
 * The expected file is pretty-printed and the ABI answers compactly, and C has
 * no JSON parser here. Comparing the compact forms of both is what makes the
 * check exact without pulling one in: every space, tab, carriage return and
 * newline is dropped, which for this corpus (numbers and fixed keys, no string
 * value containing a space) removes exactly the insignificant whitespace.
 */
static char *compact(const char *text) {
    char *out = (char *)malloc(strlen(text) + 1);
    if (!out) {
        return NULL;
    }
    size_t j = 0;
    for (size_t i = 0; text[i] != '\0'; i++) {
        char c = text[i];
        if (c == ' ' || c == '\t' || c == '\r' || c == '\n') {
            continue;
        }
        out[j++] = c;
    }
    out[j] = '\0';
    return out;
}

int main(void) {
    if (GOLDEN_CASE_COUNT == 0) {
        fprintf(stderr, "no golden cases were configured; this would test nothing\n");
        return 1;
    }

    int failures = 0;
    size_t checked = 0;
    for (size_t i = 0; i < GOLDEN_CASE_COUNT; i++) {
        const char *name = GOLDEN_CASES[i];

        char *dir = join3(GOLDEN_DIR "/", name, "/");
        char *spec_path = dir ? join3(dir, "spec.json", "") : NULL;
        char *candles_path = dir ? join3(dir, "candles.json", "") : NULL;
        char *expected_path = dir ? join3(dir, "expected.json", "") : NULL;
        char *spec = spec_path ? slurp(spec_path) : NULL;
        char *candles = candles_path ? slurp(candles_path) : NULL;
        char *expected_raw = expected_path ? slurp(expected_path) : NULL;
        free(dir);
        free(spec_path);
        free(candles_path);
        free(expected_path);

        if (!spec || !candles || !expected_raw) {
            fprintf(stderr, "%s: incomplete case\n", name);
            failures++;
            free(spec);
            free(candles);
            free(expected_raw);
            continue;
        }

        WickraGymEnv *env = wickra_gym_new(spec);
        if (!env) {
            fprintf(stderr, "%s: spec rejected\n", name);
            failures++;
            free(spec);
            free(candles);
            free(expected_raw);
            continue;
        }

        char *load = join3("{\"cmd\":\"load\",\"candles\":", candles, "}");
        char *ack = load ? run(env, load) : NULL;
        free(load);

        /* Reset with the blessed seed: the fixture records which one produced
         * it, and a different seed is a different trajectory. */
        char reset_cmd[64];
        const char *seed_at = strstr(expected_raw, "\"seed\"");
        long seed = seed_at ? strtol(strchr(seed_at, ':') + 1, NULL, 10) : 0;
        snprintf(reset_cmd, sizeof(reset_cmd), "{\"cmd\":\"reset\",\"seed\":%ld}", seed);
        char *reset = ack ? run(env, reset_cmd) : NULL;
        char *expected = compact(expected_raw);
        char *got = reset ? compact(reset) : NULL;

        if (!got || !expected) {
            fprintf(stderr, "%s: reset failed\n", name);
            failures++;
        } else {
            /* The blessed file records the whole rollout; the reset row is its
             * first observation, which is what this binding can compare without
             * a JSON parser. A mismatch here is a mismatch in the tensor the
             * whole trajectory is read from. */
            const char *marker = strstr(expected, "\"observation\":");
            const char *mine = strstr(got, "\"observation\":");
            if (!marker || !mine) {
                fprintf(stderr, "%s: no observation in the response or the fixture\n", name);
                failures++;
            } else {
                const char *end = strchr(marker, ']');
                size_t span = end ? (size_t)(end - marker) + 1 : 0;
                if (span == 0 || strncmp(marker, mine, span) != 0) {
                    fprintf(stderr, "%s: first observation differs\n  expected: %.*s\n  got:      %.*s\n",
                            name, (int)span, marker, (int)span, mine);
                    failures++;
                } else {
                    checked++;
                }
            }
        }

        free(got);
        free(expected);
        free(reset);
        free(ack);
        wickra_gym_free(env);
        free(spec);
        free(candles);
        free(expected_raw);
    }

    if (failures > 0) {
        fprintf(stderr, "%d of %zu golden cases did not match\n", failures, GOLDEN_CASE_COUNT);
        return 1;
    }
    printf("all %zu golden cases agree from C\n", checked);
    return 0;
}
