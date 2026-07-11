#' The wickra-gym library version.
#' @return A version string.
#' @export
wkgym_version <- function() {
  .Call(C_wkgym_version)
}

#' Create a gym environment from a spec JSON.
#' @param spec_json An `EnvSpec` JSON string.
#' @return A `wickra_gym` environment handle (an external pointer).
#' @export
wkgym_new <- function(spec_json) {
  .Call(C_wkgym_new, spec_json)
}

#' Apply a command JSON and return the resulting response JSON.
#' @param env An environment handle from [wkgym_new()].
#' @param cmd_json A command JSON string (`load`, `reset`, `step`, `spec`,
#'   `version`).
#' @return The response as a JSON string.
#' @export
wkgym_command <- function(env, cmd_json) {
  .Call(C_wkgym_command, env, cmd_json)
}
