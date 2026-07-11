// Package wickra provides Go bindings for wickra-gym — a deterministic,
// Gymnasium-compatible backtest environment — over the C ABI. Env forwards
// command JSONs to the core verbatim, so a rollout is byte-identical to every
// other language binding.
package wickra

/*
#cgo CFLAGS: -I${SRCDIR}/include
#cgo linux LDFLAGS: -L${SRCDIR}/lib/linux_amd64 -lwickra_gym
#cgo darwin LDFLAGS: -L${SRCDIR}/lib/darwin_arm64 -lwickra_gym
#cgo windows LDFLAGS: -L${SRCDIR}/lib/windows_amd64 -lwickra_gym
#include <stdlib.h>
#include "wickra_gym.h"
*/
import "C"

import (
	"errors"
	"unsafe"
)

// Env is a handle to a gym environment. Construct it with New, drive it with
// Command, and release it with Free.
type Env struct {
	p *C.WickraGymEnv
}

// New constructs an environment from an EnvSpec JSON string. It returns an error
// if the spec fails to parse or validate.
func New(specJSON string) (*Env, error) {
	cs := C.CString(specJSON)
	defer C.free(unsafe.Pointer(cs))
	p := C.wickra_gym_new(cs)
	if p == nil {
		return nil, errors.New("wickra-gym: invalid spec")
	}
	return &Env{p: p}, nil
}

// Command applies a command JSON and returns the response JSON string. Domain
// errors are returned in-band as {"ok":false,"error":...}; only unusable
// arguments or a caught panic produce a Go error.
func (e *Env) Command(cmdJSON string) (string, error) {
	cc := C.CString(cmdJSON)
	defer C.free(unsafe.Pointer(cc))

	// Start with a large buffer so a single call suffices; the core caches the
	// response of a not-yet-delivered command, so an identical retry never
	// re-executes (never double-steps).
	buf := make([]byte, 65536)
	n := C.wickra_gym_command(e.p, cc, (*C.char)(unsafe.Pointer(&buf[0])), C.size_t(len(buf)))
	if n < 0 {
		return "", errors.New("wickra-gym: command error")
	}
	if int(n) >= len(buf) {
		buf = make([]byte, int(n)+1)
		n = C.wickra_gym_command(e.p, cc, (*C.char)(unsafe.Pointer(&buf[0])), C.size_t(len(buf)))
		if n < 0 {
			return "", errors.New("wickra-gym: command error")
		}
	}
	return string(buf[:int(n)]), nil
}

// Version returns the wickra-gym version string.
func Version() string {
	return C.GoString(C.wickra_gym_version())
}

// Free releases the environment handle. It is safe to call once.
func (e *Env) Free() {
	C.wickra_gym_free(e.p)
	e.p = nil
}
