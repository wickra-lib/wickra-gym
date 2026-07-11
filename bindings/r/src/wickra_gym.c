/* R .Call glue for the wickra-gym C ABI hub. */
#include <R.h>
#include <Rinternals.h>
#include <R_ext/Rdynload.h>
#include <stddef.h>
#include "wickra_gym.h"

/* --- handle lifetime ----------------------------------------------------- */

static void wkgym_finalize(SEXP ext) {
    WickraGymEnv *h = (WickraGymEnv *)R_ExternalPtrAddr(ext);
    if (h) {
        wickra_gym_free(h);
    }
    R_ClearExternalPtr(ext);
}

static WickraGymEnv *handle_of(SEXP ext) {
    WickraGymEnv *h = (WickraGymEnv *)R_ExternalPtrAddr(ext);
    if (!h) {
        Rf_error("wickra-gym: handle is closed");
    }
    return h;
}

/* --- exported .Call entries ---------------------------------------------- */

SEXP wkgym_version(void) {
    return Rf_mkString(wickra_gym_version());
}

SEXP wkgym_new(SEXP spec_json) {
    const char *spec = CHAR(STRING_ELT(spec_json, 0));
    WickraGymEnv *h = wickra_gym_new(spec);
    if (!h) {
        Rf_error("wickra-gym: invalid spec");
    }
    SEXP ext = PROTECT(R_MakeExternalPtr(h, R_NilValue, R_NilValue));
    R_RegisterCFinalizerEx(ext, wkgym_finalize, TRUE);
    UNPROTECT(1);
    return ext;
}

SEXP wkgym_command(SEXP ext, SEXP cmd_json) {
    WickraGymEnv *h = handle_of(ext);
    const char *cmd = CHAR(STRING_ELT(cmd_json, 0));

    /* Length-out protocol: learn the length, then read into a caller buffer.
       The core caches the response of the not-yet-delivered command, so the
       second (delivering) call reuses it without re-executing - a mutating step
       runs exactly once. Domain errors come back in-band as {"ok":false,...}
       JSON, not a negative code; only unusable arguments / a caught panic
       return < 0. */
    int len = wickra_gym_command(h, cmd, NULL, 0);
    if (len < 0) {
        Rf_error("wickra-gym: command failed (code %d)", len);
    }
    char *buf = (char *)R_alloc((size_t)len + 1, 1);
    wickra_gym_command(h, cmd, buf, (size_t)len + 1);
    return Rf_mkString(buf);
}

/* --- registration -------------------------------------------------------- */

static const R_CallMethodDef CallEntries[] = {
    {"wkgym_version", (DL_FUNC)&wkgym_version, 0},
    {"wkgym_new", (DL_FUNC)&wkgym_new, 1},
    {"wkgym_command", (DL_FUNC)&wkgym_command, 2},
    {NULL, NULL, 0}};

void R_init_wickragym(DllInfo *dll) {
    R_registerRoutines(dll, NULL, CallEntries, NULL, NULL);
    R_useDynamicSymbols(dll, FALSE);
}
