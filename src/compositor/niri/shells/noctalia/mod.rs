//! Noctalia on Niri.
//!
//! Only what is specific to this compositor. Everything about Noctalia itself —
//! its package, its binary, its install and removal plans — is in
//! [`crate::shells::noctalia`], and is not repeated here.
//!
//! - **start**: the compositor runs it from `spawn-at-startup "noctalia"` in the
//!   skel's `config.kdl`, so switching to Noctalia means starting the process and
//!   switching away means ending it. The mechanism is
//!   [`StartMechanism::SpawnAtStartup`](crate::shell::StartMechanism::SpawnAtStartup)
//!   and the command is detached so the session does not wait.
//! - **backup**: `~/.config/noctalia` plus `~/.local/state/noctalia`, because most
//!   of a Noctalia setup lives in the second one.
//! - **clean**: those same two paths, plus the four rendered themes once the
//!   backup taken for that switch reproduces them byte for byte.
//! - **verify**: a process named `noctalia` is alive.
//!
//! The Niri skel includes theme and launcher files to keep theming consistent
//! across sessions and to restore the visible Noctalia launcher when active.

use crate::shell::{Port, Shell, StartMechanism};
use crate::shells::noctalia::IDENTITY;

const PORT: Port = Port {
    config_dir: Some(".config/noctalia"),
    state_dir: Some(".local/state/noctalia"),
    systemd_user_unit: None,
    systemd_user_unit_wanted_by: None,
    mechanism: StartMechanism::SpawnAtStartup,
    skel_extra_paths: &[
        ".config/gtk-3.0/gtk.css",
        ".config/gtk-3.0/noctalia.css",
        ".config/gtk-3.0/settings.ini",
        ".config/gtk-4.0/noctalia.css",
        ".config/gtk-4.0/settings.ini",
        ".config/gtk-4.0/gtk.css",
        ".config/qt5ct/colors/noctalia.conf",
        ".config/qt6ct/colors/noctalia.conf",
        ".config/qt6ct/qt6ct.conf",
        ".local/share/applications/dev.noctalia.Noctalia.desktop",
    ],
    backup_extra_paths: &[".local/share/applications/dev.noctalia.Noctalia.desktop"],
};

pub static SHELL: Shell = Shell::join(IDENTITY, PORT);
