package wickra

import (
	"encoding/json"
	"strings"
	"testing"
)

const spec = `{"dataset_ref":"smoke","symbol":"TEST",` +
	`"observation":{"features":[{"kind":"price","field":"close"}]},` +
	`"action_space":{"type":"discrete","n":3},` +
	`"reward":"pnl","episode":{"max_steps":100,"warmup":0}}`

func candles() []map[string]float64 {
	out := make([]map[string]float64, 0, 5)
	for i := 0; i < 5; i++ {
		p := 100.0 + float64(i)
		out = append(out, map[string]float64{
			"ts": float64(i), "open": p, "high": p, "low": p, "close": p,
		})
	}
	return out
}

func mustLoad(t *testing.T, e *Env) {
	t.Helper()
	payload, err := json.Marshal(map[string]any{"cmd": "load", "candles": candles()})
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}
	if _, err := e.Command(string(payload)); err != nil {
		t.Fatalf("load: %v", err)
	}
}

func TestVersion(t *testing.T) {
	if Version() == "" {
		t.Fatal("empty version")
	}
}

func TestLoadResetStep(t *testing.T) {
	e, err := New(spec)
	if err != nil {
		t.Fatalf("new: %v", err)
	}
	defer e.Free()
	mustLoad(t, e)

	reset, err := e.Command(`{"cmd":"reset"}`)
	if err != nil {
		t.Fatalf("reset: %v", err)
	}
	if !strings.Contains(reset, `"observation"`) {
		t.Fatalf("reset missing observation: %s", reset)
	}

	step, err := e.Command(`{"cmd":"step","action":2}`)
	if err != nil {
		t.Fatalf("step: %v", err)
	}
	var parsed struct {
		Reward     float64 `json:"reward"`
		Terminated bool    `json:"terminated"`
	}
	if err := json.Unmarshal([]byte(step), &parsed); err != nil {
		t.Fatalf("unmarshal step: %v", err)
	}
	if parsed.Reward != 1.0 {
		t.Fatalf("reward = %v, want 1.0", parsed.Reward)
	}
	if parsed.Terminated {
		t.Fatal("terminated too early")
	}
}

func TestBadSpec(t *testing.T) {
	if _, err := New(`{"not":"a spec"}`); err == nil {
		t.Fatal("expected error for bad spec")
	}
}
