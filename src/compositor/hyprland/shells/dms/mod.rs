//! DMS on Hyprland.
//!
//! Only what is specific to this compositor. The package list, the two install
//! stages, the `dcal` companion and the systemd unit are in
//! [`crate::shells::dms`], and are not repeated here.
//!
//! - **start**: `systemctl --user start dms`, for the unit shipped in the
//!   package. Nothing here enables that unit globally, so the enablement a
//!   switch writes when DMS comes in — unmask, enable and start, with `dsearch`
//!   alongside — is what starts DMS at the next login; leaving DMS stops,
//!   disables and masks both again. The skel's autostart no longer starts DMS
//!   itself, only the `dcal` companion its calendar widgets come from.
//! - **clean**: `~/.config/DankMaterialShell` and
//!   `~/.local/state/DankMaterialShell`, the same two the backup takes. The
//!   launcher entry is not among them — the Noctalia skel writes its own copy on
//!   the way in, so it is already replaced rather than left over.
//! - **verify**: a process named `dms` is alive.
//!
//! The Hyprland page also carries the Noctalia launcher entry with `NoDisplay`,
//! the theme files, and DMS's runtime state directory. That last launcher entry is
//! how the Noctalia launcher is hidden while DMS is the shell: a single file is
//! replaced, so the rest of `~/.local/share/applications` is left alone. The
//! Noctalia skel carries the same path without `NoDisplay`, so switching back brings
//! the launcher back instead of leaving it hidden.
//!
//! `gtk-3.0/gtk.css` is in that list for the reason `gtk-4.0/gtk.css` always has
//! been: both are entry points that name the sheet the shell renders beside them
//! (`dank-colors.css` here, `noctalia.css` under Noctalia), and a switch replaces
//! them outright rather than editing them. Leaving the other shell's import in
//! place would keep GTK pointing at a file nobody is running to write. The same
//! goes for `qt6ct/qt6ct.conf`, which holds `color_scheme_path` — matugen writes
//! the palette it points at and never the pointer.
//!
//! The four placeholders under `.config/hypr/dms/` are not here. They belong to
//! the compositor's own `config_dir`, which `plan` takes from the skel whole.
//!
//! The theme files `dms.rs` lists as `generated_paths` are the counterpart:
//! matugen renders those, so nothing here installs them.

use crate::shell::{Port, Shell, StartMechanism};
use crate::shells::dms::IDENTITY;

const PORT: Port = Port {
    config_dir: Some(".config/DankMaterialShell"),
    state_dir: Some(".local/state/DankMaterialShell"),
    systemd_user_unit: Some("dms"),
    systemd_user_unit_wanted_by: None,
    mechanism: StartMechanism::SystemdUserUnit,
    skel_extra_paths: &[
        ".config/gtk-3.0/gtk.css",
        ".config/gtk-3.0/settings.ini",
        ".config/gtk-4.0/settings.ini",
        ".config/gtk-4.0/gtk.css",
        ".config/qt6ct/qt6ct.conf",
        ".local/share/applications/dev.noctalia.Noctalia.desktop",
    ],
    backup_extra_paths: &[".local/share/applications/dev.noctalia.Noctalia.desktop"],
};

pub static SHELL: Shell = Shell::join(IDENTITY, PORT);
