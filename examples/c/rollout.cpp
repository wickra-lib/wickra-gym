// A minimal C++ example: load a spec and its candle dataset, then drive a fixed
// long policy and print each step. DATA_DIR is injected by CMake.
//
// This goes through `wickra_gym.hpp`, the C++ hull shipped beside the C header,
// because that hull is what a C++ caller is meant to use: it owns and frees the
// handle, runs the two-call length protocol behind `wickra_gym_command` for you
// -- the core carries the produced-but-undelivered response between the two
// calls, so a mutating `step` runs once, not twice -- and turns a refusal into
// an exception rather than a negative integer that is easy to ignore. Calling
// the C functions directly from C++ works too, but then the hull would be
// shipped without anything building it, which is how it came to reference a
// type that does not exist.
#include <cstdio>
#include <fstream>
#include <sstream>
#include <string>

#include "wickra_gym.hpp"

static std::string slurp(const std::string &path) {
    std::ifstream f(path, std::ios::binary);
    std::ostringstream ss;
    ss << f.rdbuf();
    return ss.str();
}

int main() {
    const std::string dir = DATA_DIR;
    const std::string spec = slurp(dir + "/specs/momentum_discrete.json");
    const std::string candles = slurp(dir + "/candles.json");
    if (spec.empty() || candles.empty()) {
        std::fprintf(stderr, "cannot read example data\n");
        return 1;
    }

    int steps = 0;
    try {
        wickra::Env env(spec);
        env.command("{\"cmd\":\"load\",\"candles\":" + candles + "}");
        std::printf("wickra-gym %s\n", wickra::Env::version().c_str());
        std::printf("reset: %s\n", env.command("{\"cmd\":\"reset\",\"seed\":42}").c_str());

        for (;;) {
            const std::string step = env.command("{\"cmd\":\"step\",\"action\":2}");
            // An in-band refusal: the ABI answered, the core declined. That is a
            // response rather than an error, so the hull does not throw on it.
            if (step.find("\"ok\":false") != std::string::npos) {
                std::fprintf(stderr, "the environment refused a step: %s\n", step.c_str());
                return 1;
            }
            std::printf("step %d: %s\n", steps, step.c_str());
            const bool done = step.find("\"terminated\":true") != std::string::npos ||
                              step.find("\"truncated\":true") != std::string::npos;
            ++steps;
            if (done) {
                break;
            }
        }
    } catch (const wickra::GymError &err) {
        // Every failure arrives here: a spec the core rejects, a command it does
        // not know, a response that changed length between the two ABI calls.
        std::fprintf(stderr, "%s\n", err.what());
        return 1;
    }

    if (steps == 0) {
        std::fprintf(stderr, "the episode ended before its first step\n");
        return 1;
    }
    return 0;
}
