# Shipped with the package and run by `R CMD check`, so it must work inside the
# built tarball -- where no repository sits above it and no fixture file exists.
# It touches nothing but the package: load the library, take a handle, drive one
# command through the boundary, and read the version back.
#
# The cross-language golden parity test lives in run_tests.R, which is excluded
# from the tarball by .Rbuildignore and run from the repository by CI. That one
# needs golden/ above it; this one needs nothing.

library(wickragym)

v <- wkgym_version()
stopifnot(is.character(v), length(v) == 1L, nzchar(v))

h <- wkgym_new('{"dataset_ref":"smoke","symbol":"TEST","observation":{"features":[{"kind":"price","field":"close"}]},"action_space":{"type":"discrete","n":3},"reward":"pnl","episode":{"max_steps":100,"warmup":0}}')
out <- wkgym_command(h, '{"cmd":"version"}')
stopifnot(is.character(out), length(out) == 1L)
stopifnot(grepl("version", out, fixed = TRUE))

cat("wickra-gym R package smoke: ok (version ", v, ")\n", sep = "")
