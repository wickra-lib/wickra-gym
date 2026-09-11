package wickra

// The streamed rollout equals the batch tensor, through the command boundary.
//
// The dataset is precomputed once into a fixed feature tensor -- that is the batch
// half -- and `step()` then streams through it as a pure array index. `wickra-gym-core`
// proves the two agree in Rust, but that says nothing about the boundary this
// binding crosses: a binding that mis-serialised an observation row, or truncated
// it, would hand back numbers that look plausible and are not the tensor's.
//
// So the rollout is driven twice through this binding and the two must be
// byte-identical, a longer rollout must agree with a shorter one on the bars they
// share, and the first observation must be the warmup bar the core computed.

import (
	"encoding/json"
	"fmt"
	"strings"
	"testing"
)

const streamSpec = `{"dataset_ref":"streaming","symbol":"TEST","observation":{"features":[{"kind":"indicator","name":"Sma","params":[3]},{"kind":"price","field":"close"}]},"action_space":{"type":"discrete","n":3},"reward":"pnl","episode":{"max_steps":32,"warmup":3}}`

const streamSteps = 6

func streamCandles() string {
	parts := make([]string, 0, 20)
	for i := 0; i < 20; i++ {
		f := float64(i)
		parts = append(parts, fmt.Sprintf(
			`{"ts":%d,"open":%g,"high":%g,"low":%g,"close":%g,"volume":1}`,
			i, 100.0+f, 101.0+f, 99.0+f, 100.0+f))
	}
	return "[" + strings.Join(parts, ",") + "]"
}

func loadedEnv(t *testing.T, spec string) *Env {
	t.Helper()
	e, err := New(spec)
	if err != nil {
		t.Fatalf("new env: %v", err)
	}
	if _, err := e.Command(`{"cmd":"load","candles":` + streamCandles() + `}`); err != nil {
		t.Fatalf("load: %v", err)
	}
	return e
}

func streamRollout(t *testing.T, steps int) []string {
	t.Helper()
	e := loadedEnv(t, streamSpec)
	defer e.Free()
	out, err := e.Command(`{"cmd":"reset","seed":7}`)
	if err != nil {
		t.Fatalf("reset: %v", err)
	}
	trace := []string{out}
	for i := 0; i < steps; i++ {
		out, err := e.Command(`{"cmd":"step","action":2.0}`)
		if err != nil {
			t.Fatalf("step %d: %v", i, err)
		}
		trace = append(trace, out)
	}
	return trace
}

func TestStreamedRolloutIsByteIdenticalWhenReplayed(t *testing.T) {
	a := streamRollout(t, streamSteps)
	b := streamRollout(t, streamSteps)
	if len(a) != len(b) {
		t.Fatalf("trace lengths differ: %d vs %d", len(a), len(b))
	}
	for i := range a {
		if a[i] != b[i] {
			t.Errorf("step %d differs\n first: %s\nsecond: %s", i, a[i], b[i])
		}
	}
}

func TestAPrefixOfTheRolloutMatchesAShorterOne(t *testing.T) {
	long := streamRollout(t, streamSteps)
	short := streamRollout(t, streamSteps-2)
	for i := range short {
		if long[i] != short[i] {
			t.Errorf("step %d differs between runs of different length", i)
		}
	}
}

func TestTheFirstObservationIsTheWarmupBar(t *testing.T) {
	e := loadedEnv(t, streamSpec)
	defer e.Free()
	out, err := e.Command(`{"cmd":"reset","seed":7}`)
	if err != nil {
		t.Fatalf("reset: %v", err)
	}
	var reset struct {
		Observation []float64 `json:"observation"`
	}
	if err := json.Unmarshal([]byte(out), &reset); err != nil {
		t.Fatalf("parse reset: %v", err)
	}
	// Bar 3 closes at 103; Sma(3) over the three bars ending there is 102.
	if len(reset.Observation) != 2 || reset.Observation[0] != 102 || reset.Observation[1] != 103 {
		t.Errorf("unexpected first observation: %v", reset.Observation)
	}
}

func TestAWarmupBelowTheIndicatorLookbackIsRefused(t *testing.T) {
	bad := strings.Replace(streamSpec, `"warmup":3`, `"warmup":1`, 1)
	e, err := New(bad)
	if err != nil {
		t.Fatalf("new env: %v", err)
	}
	defer e.Free()
	out, err := e.Command(`{"cmd":"load","candles":` + streamCandles() + `}`)
	if err == nil && !strings.Contains(out, "warmup") {
		t.Errorf("a warmup below the lookback must be refused, got %s", out)
	}
}
