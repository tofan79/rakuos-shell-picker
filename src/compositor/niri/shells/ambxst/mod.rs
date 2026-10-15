//! Ambxst on Niri.
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
//! - **start**: a direct process, `ambxst`. Ambxst regenerates its Niri config
//!   into `~/.local/share/ambxst/niri.kdl`, which this shell's Niri skel pulls in
//!   with an include, and that config starts it from `spawn-at-startup "ambxst"`.
//!   So the switch starts it here, and the next login starts it from Niri's own
//!   `spawn-at-startup`. It is a direct process rather than
//!   [`StartMechanism::SpawnAtStartup`](crate::shell::StartMechanism::SpawnAtStartup)
//!   because `ambxst` is its own supervisor and takes no `--daemon` flag: the
//!   command the switch runs is exactly the one Niri runs at login.
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
//! `~/.config/niri` comes from this shell's skel whole, as for every shell,
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
    mechanism: StartMechanism::DirectProcess,
    unit_enabled_by_image: false,
    // The launcher entry hides Noctalia while Ambxst is the shell: Noctalia's
    // package installs a visible `dev.noctalia.Noctalia.desktop`, and leaving it
    // visible would advertise a shell that is not the one running. This copy
    // ships hidden (`NoDisplay=true`).
    //
    // `qt6ct/qt6ct.conf` holds `color_scheme_path`; Ambxst writes the palette it
    // points at and never the pointer, so the switch has to install it.
    skel_extra_paths: &[
        ".config/qt6ct/qt6ct.conf",
        ".local/share/applications/dev.noctalia.Noctalia.desktop",
    ],
    backup_extra_paths: &[".local/share/applications/dev.noctalia.Noctalia.desktop"],
};

pub static SHELL: Shell = Shell::join(IDENTITY, PORT);
