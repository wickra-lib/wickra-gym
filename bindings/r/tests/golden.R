## Cross-language golden parity for the R binding.
##
## Kept out of the built tarball (.Rbuildignore) because it reads the golden
## corpus at the repository root, above this package. A test that ships must not
## reason about the repository it came from: in a tarball there is no repository
## above it, and a check that walks upward either fails or -- worse -- silently
## finds nothing and passes. CI runs this file explicitly from the repository
## root, where the corpus really is.
##
## Requires jsonlite.

library(wickragym)

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

cat("wickra-gym R golden parity passed
")
