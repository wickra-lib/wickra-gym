// A runnable Go example: load the momentum_discrete spec and its candle dataset
// over the wickra-gym C ABI binding, then drive a fixed long policy and print
// each step. The rollout is byte-identical to the other language examples on the
// same seed.
//
//	cargo build --release -p wickra-gym-c
//	# stage the library under bindings/go/lib/<goos>_<goarch>/ (CI does this)
//	cd examples/go && go run .
package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	wickra "github.com/wickra-lib/wickra-gym/bindings/go"
)

// dataDir walks up from the working directory to find examples/data.
func dataDir() string {
	dir, _ := os.Getwd()
	for i := 0; i < 8; i++ {
		candidate := filepath.Join(dir, "examples", "data")
		if _, err := os.Stat(candidate); err == nil {
			return candidate
		}
		dir = filepath.Dir(dir)
	}
	panic("examples/data not found")
}

// stepResult is the subset of the step response this example reads.
type stepResult struct {
	Reward     float64 `json:"reward"`
	Terminated bool    `json:"terminated"`
	Truncated  bool    `json:"truncated"`
}

func mustCommand(env *wickra.Env, cmd string) string {
	resp, err := env.Command(cmd)
	if err != nil {
		panic(err)
	}
	return resp
}

func main() {
	data := dataDir()
	spec, err := os.ReadFile(filepath.Join(data, "specs", "momentum_discrete.json"))
	if err != nil {
		panic(err)
	}
	candles, err := os.ReadFile(filepath.Join(data, "candles.json"))
	if err != nil {
		panic(err)
	}

	env, err := wickra.New(string(spec))
	if err != nil {
		panic(err)
	}
	defer env.Free()

	mustCommand(env, fmt.Sprintf(`{"cmd":"load","candles":%s}`, candles))
	fmt.Println("reset:", mustCommand(env, `{"cmd":"reset","seed":42}`))

	equity := 0.0
	for step := 0; ; step++ {
		var result stepResult
		if err := json.Unmarshal([]byte(mustCommand(env, `{"cmd":"step","action":2}`)), &result); err != nil {
			panic(err)
		}
		equity += result.Reward
		fmt.Printf("step %d: reward %+.6f  equity %+.6f  terminated=%t truncated=%t\n",
			step, result.Reward, equity, result.Terminated, result.Truncated)
		if result.Terminated || result.Truncated {
			break
		}
	}
}
