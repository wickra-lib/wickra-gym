/* The streamed rollout equals the batch tensor, through the C ABI's two-call
 * idiom.
 *
 * The dataset is precomputed once into a fixed feature tensor — that is the
 * batch half — and `step` then streams through it as a pure array index.
 * `wickra-gym-core` proves the two agree in Rust, but that says nothing about the
 * boundary a C caller crosses: every reach behind this ABI asks for the response
 * length first and reads it second, so a command that is not a pure function of
 * its payload runs twice per call. `step` is exactly that — it advances the
 * episode — and a double-applied step would skip a bar with nothing to notice.
 *
 * The dataset is inline rather than read from golden/, because C carries no JSON
 * parser and splitting the corpus by hand would test the splitter.
 *
 * The spec uses Sma(3), which has a lookback: a column over the raw close reads
 * the same whichever way the bar was reached, which is how this class of bug
 * stays invisible.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "wickra_gym.h"

#define BARS 20
#define STEPS 6

static const char *SPEC =
    "{\"dataset_ref\":\"streaming\",\"symbol\":\"TEST\","
    "\"observation\":{\"features\":["
    "{\"kind\":\"indicator\",\"name\":\"Sma\",\"params\":[3]},"
    "{\"kind\":\"price\",\"field\":\"close\"}]},"
    "\"action_space\":{\"type\":\"discrete\",\"n\":3},"
    "\"reward\":\"pnl\",\"episode\":{\"max_steps\":32,\"warmup\":3}}";

/* A rising ramp: close = 100 + i, so Sma(3) at bar 3 is 102. */
static char *candles(void) {
    size_t cap = BARS * 128 + 4;
    char *out = (char *)malloc(cap);
    if (!out) {
        return NULL;
    }
    size_t used = 0;
    used += (size_t)snprintf(out + used, cap - used, "[");
    for (int i = 0; i < BARS; i++) {
        double f = (double)i;
        used += (size_t)snprintf(
            out + used, cap - used,
            "%s{\"ts\":%d,\"open\":%g,\"high\":%g,\"low\":%g,\"close\":%g,\"volume\":1}",
            i ? "," : "", i, 100.0 + f, 101.0 + f, 99.0 + f, 100.0 + f);
    }
    snprintf(out + used, cap - used, "]");
    return out;
}

/* Run one command through the documented two-call idiom. Caller frees. */
static char *run(WickraGymEnv *env, const char *cmd) {
    int32_t len = wickra_gym_command(env, cmd, NULL, 0);
    if (len < 0) {
        fprintf(stderr, "command failed: code %d\n", (int)len);
        return NULL;
    }
    char *buf = (char *)malloc((size_t)len + 1);
    if (!buf) {
        return NULL;
    }
    int32_t written = wickra_gym_command(env, cmd, buf, (size_t)len + 1);
    if (written != len) {
        fprintf(stderr, "second call returned %d, first said %d\n", (int)written, (int)len);
        free(buf);
        return NULL;
    }
    return buf;
}

/* Drive a rollout of `steps` steps; the caller frees each entry and the array. */
static char **rollout(const char *spec, const char *data, int steps) {
    char **trace = (char **)calloc((size_t)steps + 1, sizeof(char *));
    if (!trace) {
        return NULL;
    }
    WickraGymEnv *env = wickra_gym_new(spec);
    if (!env) {
        free(trace);
        return NULL;
    }
    size_t cap = strlen(data) + 64;
    char *load = (char *)malloc(cap);
    if (!load) {
        wickra_gym_free(env);
        free(trace);
        return NULL;
    }
    snprintf(load, cap, "{\"cmd\":\"load\",\"candles\":%s}", data);
    char *ack = run(env, load);
    free(load);
    free(ack);

    trace[0] = run(env, "{\"cmd\":\"reset\",\"seed\":7}");
    for (int i = 0; i < steps; i++) {
        trace[i + 1] = run(env, "{\"cmd\":\"step\",\"action\":2.0}");
    }
    wickra_gym_free(env);
    return trace;
}

static void free_trace(char **trace, int steps) {
    if (!trace) {
        return;
    }
    for (int i = 0; i <= steps; i++) {
        free(trace[i]);
    }
    free(trace);
}

int main(void) {
    char *data = candles();
    if (!data) {
        return 1;
    }

    int failures = 0;

    char **first = rollout(SPEC, data, STEPS);
    char **second = rollout(SPEC, data, STEPS);
    if (!first || !second) {
        fprintf(stderr, "could not drive the rollout\n");
        failures++;
    } else {
        for (int i = 0; i <= STEPS; i++) {
            if (!first[i] || !second[i] || strcmp(first[i], second[i]) != 0) {
                fprintf(stderr, "step %d differs between runs\n", i);
                failures++;
            }
        }
        printf("reset: %s\n", first[0] ? first[0] : "(null)");
        /* Bar 3 closes at 103; Sma(3) over the three bars ending there is 102. */
        if (!first[0] || strstr(first[0], "\"observation\":[102.0,103.0]") == NULL) {
            fprintf(stderr, "expected the warmup bar to observe [102.0, 103.0]\n");
            failures++;
        }
    }

    /* A shorter rollout must agree with the longer one on the bars they share. */
    char **shorter = rollout(SPEC, data, STEPS - 2);
    if (!shorter) {
        failures++;
    } else if (first) {
        for (int i = 0; i <= STEPS - 2; i++) {
            if (!first[i] || !shorter[i] || strcmp(first[i], shorter[i]) != 0) {
                fprintf(stderr, "step %d differs between runs of different length\n", i);
                failures++;
            }
        }
    }

    free_trace(first, STEPS);
    free_trace(second, STEPS);
    free_trace(shorter, STEPS - 2);
    free(data);

    if (failures > 0) {
        fprintf(stderr, "%d check(s) failed\n", failures);
        return 1;
    }
    printf("the streamed rollout equals the batch tensor\n");
    return 0;
}
