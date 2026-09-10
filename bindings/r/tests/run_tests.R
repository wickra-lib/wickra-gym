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


## The streamed rollout equals the batch tensor, through the same boundary.
##
## The dataset is precomputed once into a fixed feature tensor -- the batch half
## -- and step() then streams through it as a pure array index. gym-core proves
## the two agree in Rust; this checks the boundary the R binding crosses.

stream_spec <- paste0(
  '{"dataset_ref":"streaming","symbol":"TEST",',
  '"observation":{"features":[',
  '{"kind":"indicator","name":"Sma","params":[3]},',
  '{"kind":"price","field":"close"}]},',
  '"action_space":{"type":"discrete","n":3},',
  '"reward":"pnl","episode":{"max_steps":32,"warmup":3}}'
)

stream_candles <- paste0(
  "[",
  paste(vapply(0:19, function(i) {
    paste0(
      '{"ts":', i, ',"open":', 100 + i, ',"high":', 101 + i,
      ',"low":', 99 + i, ',"close":', 100 + i, ',"volume":1}'
    )
  }, ""), collapse = ","),
  "]"
)

stream_rollout <- function(steps) {
  e <- wkgym_new(stream_spec)
  wkgym_command(e, paste0('{"cmd":"load","candles":', stream_candles, "}"))
  trace <- wkgym_command(e, '{"cmd":"reset","seed":7}')
  for (i in seq_len(steps)) {
    trace <- c(trace, wkgym_command(e, '{"cmd":"step","action":2.0}'))
  }
  trace
}

## Replaying the rollout reproduces it byte for byte.
stopifnot(identical(stream_rollout(6), stream_rollout(6)))

## A longer rollout agrees with a shorter one on the bars they share.
long_run <- stream_rollout(6)
short_run <- stream_rollout(4)
stopifnot(identical(long_run[seq_along(short_run)], short_run))

## Bar 3 closes at 103; Sma(3) over the three bars ending there is 102.
stopifnot(grepl('"observation":[102,103]', stream_rollout(0)[1], fixed = TRUE))

## A warmup below the indicator lookback is refused: below it an observation
## column is 0.0 because nothing has been produced yet.
bad_spec <- sub('"warmup":3', '"warmup":1', stream_spec, fixed = TRUE)
bad_env <- wkgym_new(bad_spec)
bad_load <- wkgym_command(bad_env, paste0('{"cmd":"load","candles":', stream_candles, "}"))
stopifnot(grepl("warmup", bad_load, fixed = TRUE))

cat("wickra-gym R tests passed\n")
