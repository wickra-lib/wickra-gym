package wickra

import (
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
)

// Cross-language golden parity: for each committed golden/<case>, drive the
// rollout and assert the response equals the expected JSON. The binding returns
// the core's canonical command_json string verbatim, so byte equality is the
// exact cross-language parity check. Skips cleanly until the golden fixtures
// land (§4).
func TestGolden(t *testing.T) {
	// bindings/go -> repo root -> golden/
	root := filepath.Join("..", "..", "golden")
	entries, err := os.ReadDir(root)
	if err != nil {
		t.Skip("golden fixtures not present yet")
	}

	for _, entry := range entries {
		if !entry.IsDir() {
			continue
		}
		dir := filepath.Join(root, entry.Name())
		specBytes, err := os.ReadFile(filepath.Join(dir, "spec.json"))
		if err != nil {
			continue
		}
		t.Run(entry.Name(), func(t *testing.T) {
			candlesBytes, err := os.ReadFile(filepath.Join(dir, "candles.json"))
			if err != nil {
				t.Fatalf("read candles: %v", err)
			}
			expectedBytes, err := os.ReadFile(filepath.Join(dir, "expected.json"))
			if err != nil {
				t.Fatalf("read expected: %v", err)
			}
			var expected struct {
				Seed       *uint64           `json:"seed"`
				Actions    []float64         `json:"actions"`
				Reset      json.RawMessage   `json:"reset"`
				Trajectory []json.RawMessage `json:"trajectory"`
			}
			if err := json.Unmarshal(expectedBytes, &expected); err != nil {
				t.Fatalf("unmarshal expected: %v", err)
			}

			e, err := New(string(specBytes))
			if err != nil {
				t.Fatalf("new: %v", err)
			}
			defer e.Free()

			var candles json.RawMessage = candlesBytes
			loadCmd, _ := json.Marshal(map[string]any{"cmd": "load", "candles": candles})
			if _, err := e.Command(string(loadCmd)); err != nil {
				t.Fatalf("load: %v", err)
			}

			resetReq := map[string]any{"cmd": "reset"}
			if expected.Seed != nil {
				resetReq["seed"] = *expected.Seed
			}
			resetCmd, _ := json.Marshal(resetReq)
			reset, err := e.Command(string(resetCmd))
			if err != nil {
				t.Fatalf("reset: %v", err)
			}
			assertJSONEqual(t, "reset", reset, string(expected.Reset))

			for i, action := range expected.Actions {
				stepCmd, _ := json.Marshal(map[string]any{"cmd": "step", "action": action})
				step, err := e.Command(string(stepCmd))
				if err != nil {
					t.Fatalf("step %d: %v", i, err)
				}
				assertJSONEqual(t, "step", step, string(expected.Trajectory[i]))
			}
		})
	}
}

func assertJSONEqual(t *testing.T, label, got, want string) {
	t.Helper()
	var g, w any
	if err := json.Unmarshal([]byte(got), &g); err != nil {
		t.Fatalf("%s: unmarshal got: %v", label, err)
	}
	if err := json.Unmarshal([]byte(want), &w); err != nil {
		t.Fatalf("%s: unmarshal want: %v", label, err)
	}
	gb, _ := json.Marshal(g)
	wb, _ := json.Marshal(w)
	if string(gb) != string(wb) {
		t.Fatalf("%s mismatch:\n got: %s\nwant: %s", label, gb, wb)
	}
}
