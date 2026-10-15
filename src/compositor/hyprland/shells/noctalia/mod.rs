//! Noctalia on Hyprland.
//!
//! Only what is specific to this compositor. Everything about Noctalia itself —
//! its package, its binary, that it is in the base image — is in
//! [`crate::shells::noctalia`], and is not repeated here.
//!
//! - **start**: the compositor runs it from `config/autostart.lua`, so switching to
//!   Noctalia means starting the process and switching away means ending it.
//! - **backup**: `~/.config/noctalia` plus `~/.local/state/noctalia`, because most
//!   of a Noctalia setup lives in the second one.
//! - **clean**: those same two paths, plus the four rendered themes, once the
//!   backup taken for that switch reproduces them byte for byte. Nothing else is
//!   cleaned: `gtk-3.0/gtk.css` and `qt6ct/qt6ct.conf` are not in that list
//!   because the shell being switched to overwrites them from its own skel on
//!   the way in, which is what keeps either shell's `@import` and
//!   `color_scheme_path` pointing at a sheet its own shell is running to write.
//! - **verify**: a process named `noctalia` is alive.
//!
//! The Hyprland page also carries the launcher entry and the theme files, because
//! its skel is the one that installs them. Theme tweaks are listed as individual
//! files, never as `~/.config/gtk-3.0` or `~/.config/gtk-4.0`: those two
//! directories are shared with every other GTK program on the desktop, and the swap
//! replaces a path outright rather than merging into it, so listing the directory
//! would take the user's own stylesheets with it.

use crate::shell::{Port, Shell, StartMechanism};
use crate::shells::noctalia::IDENTITY;

const PORT: Port = Port {
    config_dir: Some(".config/noctalia"),
    state_dir: Some(".local/state/noctalia"),
    systemd_user_unit: None,
    mechanism: StartMechanism::Autostart,
    // `config_dir` is handled separately, since it comes from a backup when one
    // holds it. The launcher entry is the counterpart to the one the DMS skel
    // replaces: without it, switching back would leave Noctalia installed and
    // invisible.
    skel_extra_paths: &[
        ".config/gtk-3.0/gtk.css",
        ".config/gtk-3.0/noctalia.css",
        ".config/gtk-3.0/settings.ini",
        ".config/gtk-4.0/noctalia.css",
        ".config/gtk-4.0/settings.ini",
        ".config/gtk-4.0/gtk.css",
        ".config/qt6ct/qt6ct.conf",
        ".local/share/applications/dev.noctalia.Noctalia.desktop",
    ],
    backup_extra_paths: &[".local/share/applications/dev.noctalia.Noctalia.desktop"],
};

pub static SHELL: Shell = Shell::join(IDENTITY, PORT);
