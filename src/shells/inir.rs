//! iNiR — a Quickshell desktop shell for Niri, packaged for RakuOS as
//! `inir-shell`.
//!
//! What is the same wherever it runs (today that is Niri only):
//!
//! - **package**: `inir-shell`, from the RakuOS repository. It ships the QML
//!   payload to `/usr/share/quickshell/inir`, the `inir` launcher, and the
//!   systemd user unit the launcher's `run --session` mode is written for.
//! - **binary**: `inir` is the package's own command, so a `which` hit means
//!   this shell — and not the shared `qs` every Quickshell shell drags in — is
//!   on the machine.
//! - **probe**: the launcher `exec`s the shared runtime, so no live process is
//!   ever named `inir`. The process is `qs -n -p <config dir>`, and the config
//!   directory differs between the system payload and a user copy under
//!   `~/.config/quickshell/inir`. `process_match` matches whole static
//!   arguments only, so the pattern lands together with the backend that pins
//!   which of the two paths a switch uses; until then an empty pattern — and so
//!   matching by name alone — is the only truthful answer.
//! - **logo**: `inir.svg`, the project's mascot.
//!
//! The backend is deliberately absent for now — the card is what exists. No
//! `install_plan`, no `config_dir` and no skel yet (see the Niri port), so a
//! switch refuses in `Refusal::NoConfig` before the backup is taken and nothing
//! on the machine is touched.

use crate::shell::Identity;

pub const IDENTITY: Identity = Identity {
    id: "inir",
    display_name: "iNiR",
    upstream: "snowarch/iNiR",
    package: Some("inir-shell"),
    binary: "inir",
    start_argv: &["inir", "run", "--session"],
    process_match: &[],
    install_plan: None,
    uninstall_plan: None,
    companion_systemd_user_units: &[],
    autostart_systemd_user_units: &[],
    companion_paths: &[],
    generated_paths: &[],
    discard_paths: &[],
    runtime_packages: &[],
    polkit_agent: &[],
    logo: Some("inir.svg"),
};
