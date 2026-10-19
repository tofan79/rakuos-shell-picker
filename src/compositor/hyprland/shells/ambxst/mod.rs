//! Ambxst on Hyprland.
//!
//! Only what is specific to this compositor. Everything about Ambxst itself —
//! that it is packaged, that it is a direct process, and the runtime it shares
//! with the other quickshell shells — is in [`crate::shells::ambxst`], and is
//! not repeated here.
//!
//! - **config**: `~/.config/ambxst`, which holds `binds.json`, the `config/`
//!   tree its reactive settings live in, and `presets/`. Ambxst writes these
//!   itself on first run; the switch seeds nothing into that directory and takes
//!   it from the shell's own backup when there is one, exactly as it does for
//!   every other shell.
//! - **state**: `~/.local/state/ambxst`, its `states.json` (the running UI state)
//!   and `clipboard.key`. Backed up alongside the config, because losing the key
//!   while `~/.local/share/ambxst/clipboard-*.db` stays behind would strand the
//!   clipboard history.
//! - **start**: a direct process, `ambxst`. Ambxst arms its own
//!   `hyprland.start` hook from the `~/.local/share/ambxst/hyprland.lua` it
//!   regenerates, which this shell's Hyprland skel loads — so the switch starts
//!   it here, and the next login starts it from that hook.
//! - **backup**: those two directories, plus the launcher entry and the shared
//!   Qt `[Appearance]` file a switch overwrites, plus the data Ambxst keeps
//!   outside them (`~/.local/share/ambxst`, `~/.local/share/ambxst-notes` and
//!   `~/.config/axctl`) through [`crate::shell::Identity::companion_paths`].
//! - **clean**: the same two directories, that data, the `~/.cache` trees with
//!   nothing in them, and the two Qt colour files Ambxst renders for itself —
//!   each only once this switch's backup reproduces it byte for byte. The data
//!   under `~/.local/share` goes as well, because the backup now holds it and a
//!   switch away that left the chat history behind would be a switch that did
//!   not finish.
//! - **verify**: a process named `ambxst` is alive.
//!
//! `~/.config/hypr` comes from this shell's skel whole, as for every shell,
//! because it is the compositor's `config_dir` and where the shell is started
//! from. The Qt `color_scheme_path` is in the skel because Ambxst renders only
//! the palette (`qt5ct/colors/ambxst.colors` and its qt6ct twin) and never the
//! pointer to it.

use crate::shell::{Port, Shell, StartMechanism};
use crate::shells::ambxst::IDENTITY;

const PORT: Port = Port {
    config_dir: Some(".config/ambxst"),
    state_dir: Some(".local/state/ambxst"),
    systemd_user_unit: None,
    systemd_user_unit_wanted_by: None,
    mechanism: StartMechanism::DirectProcess,
    // The launcher entry is the same counterpart the other shells carry: a
    // switch must not leave Noctalia advertised in the app menu while Ambxst is
    // the shell. This copy ships hidden (`NoDisplay=true`), like DMS's.
    //
    // `qt6ct/qt6ct.conf` holds `color_scheme_path`; Ambxst writes the palette it
    // points at and never the pointer, so the switch has to install it. It is in
    // `~/.config/qt6ct`, which `switch::SHARED_PATHS` already backs up, so it
    // needs no `backup_extra_paths` entry.
    skel_extra_paths: &[
        ".config/qt6ct/qt6ct.conf",
        ".local/share/applications/dev.noctalia.Noctalia.desktop",
    ],
    // The launcher entry is in `skel_extra_paths` because a switch installs it
    // and in `backup_extra_paths` because nothing else would save it. The plan
    // deduplicates the path, so it is not staged twice.
    backup_extra_paths: &[".local/share/applications/dev.noctalia.Noctalia.desktop"],
};

pub static SHELL: Shell = Shell::join(IDENTITY, PORT);
