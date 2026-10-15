//! DMS on Niri.
//!
//! Only what is specific to this compositor. The package list, the two install
//! stages, the `dcal` companion and the systemd unit are in
//! [`crate::shells::dms`], and are not repeated here.
//!
//! - **start**: `systemctl --user start dms`, the same unit Hyprland uses. The
//!   `rakuos-niri` image enables it globally (`systemctl --global enable dms`),
//!   and `dms.service` is `WantedBy=graphical-session.target`, so a login starts
//!   DMS on its own whether or not it is the shell the user picked. That is the
//!   one fact `unit_enabled_by_image` records: leaving DMS disables and masks the
//!   unit so the next login does not bring it back beside the shell that was
//!   chosen, and the Hyprland port leaves the flag `false` because its image
//!   enables nothing globally.
//! - **clean**: `~/.config/DankMaterialShell` and
//!   `~/.local/state/DankMaterialShell`, the same two the backup takes.
//! - **verify**: a process named `dms` is alive.
//!
//! The Niri skel is the image's own `etc/skel` plus theme and launcher files to
//! keep theming consistent across sessions and to hide Noctalia's launcher while
//! DMS is active (switching back restores the visible launcher).

use crate::shell::{Port, Shell, StartMechanism};
use crate::shells::dms::IDENTITY;

const PORT: Port = Port {
    config_dir: Some(".config/DankMaterialShell"),
    state_dir: Some(".local/state/DankMaterialShell"),
    systemd_user_unit: Some("dms"),
    mechanism: StartMechanism::SystemdUserUnit,
    unit_enabled_by_image: true,
    skel_extra_paths: &[
        ".config/gtk-3.0/dank-colors.css",
        ".config/gtk-3.0/gtk.css",
        ".config/gtk-3.0/settings.ini",
        ".config/gtk-4.0/dank-colors.css",
        ".config/gtk-4.0/gtk.css",
        ".config/gtk-4.0/settings.ini",
        ".config/qt6ct/qt6ct.conf",
        ".local/share/applications/dev.noctalia.Noctalia.desktop",
    ],
    backup_extra_paths: &[".local/share/applications/dev.noctalia.Noctalia.desktop"],
};

pub static SHELL: Shell = Shell::join(IDENTITY, PORT);
