//! end4-pC — the `pctrade` fork of end-4's illogical-impulse Quickshell
//! configuration, packaged for RakuOS as `end4-pc`.
//!
//! What is the same wherever it runs (today that is Hyprland only):
//!
//! - **package**: `end4-pc`, from the RakuOS repository. It ships the QML only,
//!   to `/usr/share/quickshell/end4-pC`; the compositor configuration that
//!   starts it comes from the package's own skel when the backend lands.
//! - **binary**: `end4-pC` is the quickshell config identity, not a real
//!   program — no launcher ships with the package, and nothing named this is
//!   meant to appear in `PATH`. That is deliberate: the live process is the
//!   shared `qs`, and if `binary` were `qs` then `installed()` would pass any
//!   machine with Quickshell on it off as this shell. With a name that resolves
//!   nowhere, installedness stays driven by the package origin.
//! - **probe**: `qs -c end4-pC`, the command upstream's README starts it with.
//!   Those two argv words tell it apart from the sibling Quickshell shells on
//!   the same runtime; the base `ii` runs as `qs -c ii`.
//! - **logo**: `end4pc.svg`, the illogical-impulse mark.
//!
//! The backend is deliberately absent for now — the card is what exists. No
//! `install_plan`, no `config_dir` and no skel yet (see the Hyprland port), so
//! a switch refuses in `Refusal::NoConfig` before the backup is taken and
//! nothing on the machine is touched.

use crate::shell::Identity;

pub const IDENTITY: Identity = Identity {
    id: "end4pc",
    display_name: "end4-pC",
    upstream: "pctrade/end4-pC",
    package: Some("end4-pc"),
    binary: "end4-pC",
    start_argv: &["qs", "-c", "end4-pC"],
    process_match: &["-c", "end4-pC"],
    install_plan: None,
    uninstall_plan: None,
    companion_systemd_user_units: &[],
    autostart_systemd_user_units: &[],
    companion_paths: &[],
    generated_paths: &[],
    discard_paths: &[],
    runtime_packages: &[],
    polkit_agent: &[],
    logo: Some("end4pc.svg"),
};
