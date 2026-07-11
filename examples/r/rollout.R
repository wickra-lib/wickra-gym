#!/usr/bin/env Rscript
# A runnable R example: load the momentum_discrete spec and its candle dataset
# over the wickra-gym C ABI binding, then drive a fixed long policy and print
# each step. The rollout is byte-identical to the other language examples on the
# same seed.
#
#   R CMD INSTALL bindings/r
#   Rscript examples/r/rollout.R

library(wickragym)

find_data_dir <- function() {
  dir <- normalizePath(getwd(), mustWork = FALSE)
  for (i in seq_len(10)) {
    candidate <- file.path(dir, "examples", "data")
    if (dir.exists(candidate)) {
      return(candidate)
    }
    parent <- dirname(dir)
    if (identical(parent, dir)) break
    dir <- parent
  }
  stop("examples/data not found")
}

data_dir <- find_data_dir()
spec <- paste(readLines(file.path(data_dir, "specs", "momentum_discrete.json")),
              collapse = "\n")
candles <- paste(readLines(file.path(data_dir, "candles.json")), collapse = "\n")

env <- wkgym_new(spec)
wkgym_command(env, sprintf('{"cmd":"load","candles":%s}', candles))
cat("reset:", wkgym_command(env, '{"cmd":"reset","seed":42}'), "\n")

equity <- 0
step <- 0
repeat {
  result <- jsonlite::fromJSON(wkgym_command(env, '{"cmd":"step","action":2}'))
  equity <- equity + result$reward
  cat(sprintf("step %d: reward %+.6f  equity %+.6f  terminated=%s truncated=%s\n",
              step, result$reward, equity,
              tolower(as.character(result$terminated)),
              tolower(as.character(result$truncated))))
  if (isTRUE(result$terminated) || isTRUE(result$truncated)) break
  step <- step + 1
}
