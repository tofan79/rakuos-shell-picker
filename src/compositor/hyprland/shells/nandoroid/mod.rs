//! NAnDoroid on Hyprland.
//!
//! Only what is specific to this compositor. Everything about NAnDoroid itself —
//! its package, its plans, that it answers polkit from its own process — is in
//! [`crate::shells::nandoroid`], and is not repeated here.
//!
//! - **config**: `~/.config/quickshell/nandoroid`, the symlink the skel creates
//!   into `/usr/share/quickshell/nandoroid`. `qs -c nandoroid` resolves the
//!   config by that name, so the symlink is the one path a switch must write for
//!   the shell to start at all.
//! - **start**: the compositor's start event — but not from this skel. The
//!   package's `hypr/nandoroid/nandoroid.lua` arms `hl.on("hyprland.start")` to
//!   run `quickshell -c nandoroid`, and the skel reaches that file through a
//!   symlink (`~/.config/hypr/nandoroid/nandoroid.lua` →
//!   `/usr/share/nandoroid/hypr/nandoroid/nandoroid.lua`), so a login and a
//!   switch start the shell the same way and the keybinds stay the package's.
//! - **the Hyprland config is upstream's `extras/hypr/`**: `hyprland.lua`,
//!   `theme.lua` (loaded by `configs/env.lua`), `configs/{env,execs,general,
//!   rules,keybinds}.lua`, `hypridle.conf`, `hyprlock.conf` and
//!   `hyprlock/colors.conf`. That tree is *not* in the RPM, so it is copied into
//!   the skel verbatim — a switch replaces `~/.config/hypr` whole, and without it
//!   there would be no config at all. `hyprland.lua` sets a generic monitor
//!   (`output = ""`, preferred, auto), the same no-black-screen default the other
//!   skels use; there is no per-machine `monitors.lua` to ship or to overwrite.
//! - **`user_persistence.lua`** is the one file in `nandoroid/` that is a real
//!   file rather than a link: the shell writes the display profile and game-mode
//!   choices a person makes there, and the upstream `hyprland.lua` loads it last.
//!   It travels with `~/.config/hypr`, so the backup of the compositor directory
//!   is what saves it.
//! - **backup**: `config_dir`, plus the matugen directory the skel installs and
//!   the hidden Noctalia launcher entry. `~/.config/hypr` itself is the
//!   compositor's `config_dir` and is backed up by `switch::SHARED_PATHS` for
//!   every shell.
//! - **clean**: nothing of its own. The rendered GTK/hyprlock sheets are in
//!   `generated_paths`, the cache in `discard_paths`, and the rest is either
//!   backed up and reproduced (the matugen directory) or the compositor's.
//! - **verify**: a process whose command line carries `-c nandoroid` is alive.
//!
//! The matugen directory is a skel path rather than a companion because the
//! switch is the only thing that installs it: no part of the shell writes
//! `config.toml`, and the templates are a symlink into the package, so a backup
//! of the directory holds one file and a link.

use crate::shell::{Port, Shell, StartMechanism};
use crate::shells::nandoroid::IDENTITY;

const PORT: Port = Port {
    config_dir: Some(".config/quickshell/nandoroid"),
    state_dir: None,
    systemd_user_unit: None,
    systemd_user_unit_wanted_by: None,
    mechanism: StartMechanism::StartEvent,
    skel_extra_paths: &[
        ".config/matugen",
        ".local/share/applications/dev.noctalia.Noctalia.desktop",
        ".config/ghostty/config",
    ],
    // The matugen directory is in `skel_extra_paths` because a switch installs
    // it and here because nothing else would save it; `~/.config/matugen` is not
    // in `switch::SHARED_PATHS`. The launcher entry is the same counterpart
    // every other skel carries: a switch must not leave Noctalia advertised in
    // the app menu while NAnDoroid is the shell.
    backup_extra_paths: &[
        ".config/matugen",
        ".local/share/applications/dev.noctalia.Noctalia.desktop",
        ".config/ghostty/config",
    ],
};

pub static SHELL: Shell = Shell::join(IDENTITY, PORT);
