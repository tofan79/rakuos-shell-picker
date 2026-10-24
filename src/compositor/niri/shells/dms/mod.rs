//! DMS on Niri.
//!
//! Only what is specific to this compositor. The package list, the two install
//! stages, the `dcal` companion and the systemd unit are in
//! [`crate::shells::dms`], and are not repeated here.
//!
//! - **start**: `systemctl --user start dms`, the same unit Hyprland uses. The
//!   `rakuos-niri` image enables it — and `dsearch` — globally with
//!   `systemctl --global enable`, and `dms.service` is
//!   `WantedBy=graphical-session.target`, so a login starts DMS on its own
//!   whether or not it is the shell the user picked. That is why leaving DMS
//!   has to stop, disable and mask both units: without the mask the next login
//!   brings them back beside the shell that was chosen. `start` unwinds exactly
//!   that when the shell comes home, and does the same on Hyprland, where
//!   nothing enables these units but the picker itself.
//! - **clean**: `~/.config/dms` and `~/.local/state/dms`, the same two the
//!   backup takes. DMS 2.0 renamed both from `DankMaterialShell`; the old
//!   spellings survive only as compatibility symlinks.
//! - **verify**: a process named `dms` is alive.
//!
//! The Niri skel is the image's own `etc/skel` plus theme and launcher files to
//! keep theming consistent across sessions and to hide Noctalia's launcher while
//! DMS is active (switching back restores the visible launcher).

use crate::shell::{Port, Shell, StartMechanism};
use crate::shells::dms::IDENTITY;

const PORT: Port = Port {
    config_dir: Some(".config/dms"),
    state_dir: Some(".local/state/dms"),
    systemd_user_unit: Some("dms"),
    systemd_user_unit_wanted_by: None,
    mechanism: StartMechanism::SystemdUserUnit,
    skel_extra_paths: &[
        ".config/gtk-3.0/dank-colors.css",
        ".config/gtk-3.0/gtk.css",
        ".config/gtk-3.0/settings.ini",
        ".config/gtk-4.0/dank-colors.css",
        ".config/gtk-4.0/gtk.css",
        ".config/gtk-4.0/settings.ini",
        ".config/qt6ct/qt6ct.conf",
        ".local/share/applications/dev.noctalia.Noctalia.desktop",
        ".config/ghostty/config",
        ".config/ghostty/themes/dankcolors",
    ],
    backup_extra_paths: &[
        ".local/share/applications/dev.noctalia.Noctalia.desktop",
        ".config/ghostty/config",
        ".config/ghostty/themes/dankcolors",
    ],
};

pub static SHELL: Shell = Shell::join(IDENTITY, PORT);
