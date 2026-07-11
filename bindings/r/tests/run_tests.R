## Plain-R tests for the wickra-gym R binding (no testthat dependency). Mirrors
## the Rust/Python/Node/WASM/Go/C#/Java tests and doubles as the completeness
## guard: it exercises the full public surface (version + new + command).

library(wickragym)

spec <- paste0(
  '{"dataset_ref":"smoke","symbol":"TEST",',
  '"observation":{"features":[{"kind":"price","field":"close"}]},',
  '"action_space":{"type":"discrete","n":3},',
  '"reward":"pnl","episode":{"max_steps":100,"warmup":0}}'
)

load_command <- function() {
  parts <- vapply(0:4, function(i) {
    p <- 100.0 + i
    paste0(
      '{"ts":', i, ',"open":', p, ',"high":', p,
      ',"low":', p, ',"close":', p, '}'
    )
  }, character(1))
  paste0('{"cmd":"load","candles":[', paste(parts, collapse = ","), ']}')
}

## version
stopifnot(nzchar(wkgym_version()))

## load / reset / step
env <- wkgym_new(spec)
stopifnot(identical(wkgym_command(env, load_command()), '{"ok":true}'))
reset <- wkgym_command(env, '{"cmd":"reset"}')
stopifnot(grepl('"observation"', reset, fixed = TRUE))
step <- wkgym_command(env, '{"cmd":"step","action":2}')
stopifnot(grepl('"reward":1.0', step, fixed = TRUE))
stopifnot(grepl('"terminated":false', step, fixed = TRUE))

## a bad spec is a hard error, not a handle
bad <- tryCatch(
  {
    wkgym_new('{"not":"a spec"}')
    FALSE
  },
  error = function(e) TRUE
)
stopifnot(bad)

## cross-language golden parity: for each committed golden/<case>, replay the
## rollout and assert the response equals the expected JSON. The binding returns
## the core's canonical command output verbatim, so structural equality is the
## exact cross-language parity check. Requires jsonlite; skipped until the
## fixtures land.
golden_dir <- function() {
  d <- normalizePath(getwd(), mustWork = FALSE)
  for (i in seq_len(8)) {
    g <- file.path(d, "golden")
    if (dir.exists(g)) {
      return(g)
    }
    d <- dirname(d)
  }
  NULL
}

g <- golden_dir()
if (!is.null(g) && requireNamespace("jsonlite", quietly = TRUE)) {
  for (case in list.dirs(g, recursive = FALSE)) {
    if (!file.exists(file.path(case, "spec.json"))) {
      next
    }
    spec_j <- paste(readLines(file.path(case, "spec.json"), warn = FALSE), collapse = "")
    candles_j <- paste(readLines(file.path(case, "candles.json"), warn = FALSE), collapse = "")
    expected <- jsonlite::fromJSON(file.path(case, "expected.json"), simplifyVector = FALSE)

    genv <- wkgym_new(spec_j)
    wkgym_command(genv, paste0('{"cmd":"load","candles":', candles_j, '}'))
    reset_cmd <- if (!is.null(expected$seed)) {
      paste0('{"cmd":"reset","seed":', expected$seed, '}')
    } else {
      '{"cmd":"reset"}'
    }
    got_reset <- jsonlite::fromJSON(wkgym_command(genv, reset_cmd), simplifyVector = FALSE)
    stopifnot(identical(got_reset, expected$reset))
    for (i in seq_along(expected$actions)) {
      step_cmd <- paste0('{"cmd":"step","action":', expected$actions[[i]], '}')
      got_step <- jsonlite::fromJSON(wkgym_command(genv, step_cmd), simplifyVector = FALSE)
      stopifnot(identical(got_step, expected$trajectory[[i]]))
    }
  }
}

cat("wickra-gym R tests passed\n")
