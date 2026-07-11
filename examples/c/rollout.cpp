// A minimal C++ example over the wickra-gym C ABI: load a spec and its candle
// dataset, then drive a fixed long policy and print each step. DATA_DIR is
// injected by CMake.
#include <cstdio>
#include <fstream>
#include <sstream>
#include <string>

#include "wickra_gym.h"

static std::string slurp(const std::string &path) {
    std::ifstream f(path, std::ios::binary);
    std::ostringstream ss;
    ss << f.rdbuf();
    return ss.str();
}

// Run a command via the length-out protocol; the core caches the not-yet-
// delivered response so the delivering call reuses it (steps run once).
static std::string run(WickraGymEnv *env, const std::string &cmd) {
    int len = wickra_gym_command(env, cmd.c_str(), nullptr, 0);
    if (len < 0) {
        return {};
    }
    std::string buf(static_cast<size_t>(len) + 1, '\0');
    // std::string::data() is const before C++17; &buf[0] gives a writable ptr.
    wickra_gym_command(env, cmd.c_str(), &buf[0], buf.size());
    buf.resize(static_cast<size_t>(len));
    return buf;
}

int main() {
    const std::string dir = DATA_DIR;
    const std::string spec = slurp(dir + "/specs/momentum_discrete.json");
    const std::string candles = slurp(dir + "/candles.json");
    if (spec.empty() || candles.empty()) {
        std::fprintf(stderr, "cannot read example data\n");
        return 1;
    }

    WickraGymEnv *env = wickra_gym_new(spec.c_str());
    if (!env) {
        std::fprintf(stderr, "invalid spec\n");
        return 1;
    }

    run(env, "{\"cmd\":\"load\",\"candles\":" + candles + "}");
    std::printf("wickra-gym %s\n", wickra_gym_version());
    std::printf("reset: %s\n", run(env, "{\"cmd\":\"reset\",\"seed\":42}").c_str());

    int steps = 0;
    bool ok = true;
    for (;;) {
        const std::string step = run(env, "{\"cmd\":\"step\",\"action\":2}");
        if (step.empty() || step.find("\"ok\":false") != std::string::npos) {
            ok = false;
            break;
        }
        std::printf("step %d: %s\n", steps, step.c_str());
        const bool done = step.find("\"terminated\":true") != std::string::npos ||
                          step.find("\"truncated\":true") != std::string::npos;
        ++steps;
        if (done) {
            break;
        }
    }

    wickra_gym_free(env);
    if (!ok || steps == 0) {
        std::fprintf(stderr, "rollout failed\n");
        return 1;
    }
    return 0;
}
