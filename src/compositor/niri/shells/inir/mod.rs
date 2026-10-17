//! iNiR on Niri.
//!
//! Only what is specific to this compositor. What iNiR is — its package, its
//! launcher, its logo — is in [`crate::shells::inir`], and is not repeated
//! here.
//!
//! This port is a card first: the fields a switch would execute on are still
//! empty on purpose, and each one lands with the backend that needs it.
//!
//! - **start**: the systemd user unit `inir` the package ships. It is tied to
//!   `niri.service` (`PartOf`/`Requisite`) because Niri owns the session
//!   environment the shell renders into.
//! - **config**: `None` for now. The shell reads its QML from
//!   `/usr/share/quickshell/inir`, with a user copy under
//!   `~/.config/quickshell/inir` taking precedence, and which of the two a
//!   switch seeds is a backend decision. Until `config_dir` is declared, a
//!   switch refuses in `Refusal::NoConfig` before any backup.
//! - **probe**: see [`crate::shells::inir`] — the argv pattern needs that same
//!   decision about the config path, so it stays empty with `config_dir`.
//! - **skel**: none yet. Nothing is installed to the machine, so a switch has
//!   nothing to complete with.

use crate::shell::{Port, Shell, StartMechanism};
use crate::shells::inir::IDENTITY;

const PORT: Port = Port {
    config_dir: None,
    state_dir: None,
    systemd_user_unit: Some("inir"),
    mechanism: StartMechanism::SystemdUserUnit,
    skel_extra_paths: &[],
    backup_extra_paths: &[],
};

pub static SHELL: Shell = Shell::join(IDENTITY, PORT);
