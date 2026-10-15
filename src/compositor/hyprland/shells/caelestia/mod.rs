//! Caelestia Shell on Hyprland.
//!
//! Only what is specific to this compositor. What Caelestia is — its package,
//! its install plan, its shared runtime — is in [`crate::shells::caelestia`],
//! and is not repeated here.
//!
//! - **backup**: `~/.config/caelestia` (its `shell.json` and monitor overrides)
//!   and `~/.local/state/caelestia` (its colour scheme, wallpaper, notifications
//!   and catalog), plus the compositor config every switch saves.
//! - **clean**: those same two paths and the theme files `caelestia.rs` lists,
//!   each only once this switch's backup reproduces it byte for byte.
//! - **start**: the compositor's start event. The skel's own
//!   `config/hyprland/execs.lua` runs `caelestia shell -d`, and the live switch
//!   runs the same command directly, so both the current and the next session
//!   end with `qs -c caelestia`.
//!
//! The skel is Caelestia's own Hyprland configuration rather than the shared
//! RakuOS one: the shell is written against its variables, binds and resizer
//! rules, so its `~/.config/hypr` comes from upstream's dots. The `config_dir`
//! is `~/.config/caelestia` — where the shell keeps `shell.json`, monitor
//! overrides and the user's `hypr-vars.lua`/`hypr-user.lua` the Hyprland config
//! loads — and not a path under `~/.config/quickshell`, which the package does
//! not write.

use crate::shell::{Port, Shell, StartMechanism};
use crate::shells::caelestia::IDENTITY;

const PORT: Port = Port {
    config_dir: Some(".config/caelestia"),
    state_dir: Some(".local/state/caelestia"),
    systemd_user_unit: None,
    mechanism: StartMechanism::StartEvent,
    unit_enabled_by_image: false,
    // The launcher entry, and only it: Caelestia's own GTK and Qt entry points
    // are written by the shell from its templates, so no theme file is the
    // switch's to install. The one file is the counterpart to the same path in
    // the DMS and Noctalia skels — a switch must not leave the shell it is
    // leaving advertised in the app menu. Noctalia's launcher ships visible and
    // DMS's ships with `NoDisplay=true`; this skel takes the hidden spelling so
    // that coming *from* Noctalia hides it, and the shared `pull` logic brings
    // the visible one back on the way *to* Noctalia, from that shell's own
    // backup or skel. A single file rather than the directory, so the rest of
    // `~/.local/share/applications` is left alone.
    skel_extra_paths: &[".local/share/applications/dev.noctalia.Noctalia.desktop"],
    // Same file, listed for the second question a switch asks: nothing else
    // would back it up. "Being installed" and "being kept" are not the same
    // fact, and the plan deduplicates the path so it is not staged twice.
    backup_extra_paths: &[".local/share/applications/dev.noctalia.Noctalia.desktop"],
};

pub static SHELL: Shell = Shell::join(IDENTITY, PORT);
