# Install the compiled package shared object plus the bundled C ABI library so
# the package is self-contained: on Windows the wickra_gym_abi.dll (matched by the
# *.dll glob, loaded via PATH in .onLoad); on Linux the libwickra_gym.so (matched by
# the *.so SHLIB_EXT glob); on macOS the libwickra_gym.dylib (added explicitly, since
# R package objects use the .so extension there too). The Unix rpath baked by
# configure ($ORIGIN / @loader_path) resolves it from this libs directory.
files <- unique(c(Sys.glob(paste0("*", SHLIB_EXT)), Sys.glob("libwickra_gym.dylib")))
dest <- file.path(R_PACKAGE_DIR, paste0("libs", R_ARCH))
dir.create(dest, recursive = TRUE, showWarnings = FALSE)
file.copy(files, dest, overwrite = TRUE)
if (file.exists("symbols.rds")) {
  file.copy("symbols.rds", dest, overwrite = TRUE)
}
