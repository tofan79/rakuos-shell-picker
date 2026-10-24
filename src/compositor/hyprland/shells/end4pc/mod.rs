//! end4-pC on Hyprland.
//!
//! Only what is specific to this compositor. What end4-pC is — its package,
//! its install plan, its shared runtime — is in [`crate::shells::end4pc`],
//! and is not repeated here.
//!
//! - **backup**: `~/.config/quickshell/end4-pC` (the symlink into the
//!   package), `~/.config/matugen` (the theming tool's configuration and
//!   templates) and the settings directory the shell writes for itself,
//!   `~/.config/illogical-impulse`, which travels as companion data — plus the
//!   compositor config every switch saves. The state directory is deliberately
//!   not declared as one: what lives under `~/.local/state/quickshell` is
//!   split by what it holds — `user/` and `states.json` travel as companion
//!   data (backed up, taken away, read back when end4-pC returns), while the
//!   venv and its bootstrap log are discards, rebuilt by the script on the
//!   next login; [`crate::shells::end4pc`] argues it out. `~/.local/share/uv`
//!   is left alone: a general tool's data directory, whose managed interpreter
//!   the rebuilt venv reuses — only `~/.cache/uv` goes.
//! - **clean**: those paths, the renders `end4pc.rs` lists, and its
//!   `discard_paths` — the paths that carry data only once this switch's
//!   backup reproduces them byte for byte, and the caches on the no-data terms
//!   that field describes.
//! - **start**: the compositor's start event, the same mechanism Caelestia
//!   uses. The skel's own `hyprland/execs.lua` runs `qs -c $qsConfig`, with
//!   `qsConfig` set to `end4-pC` in `hyprland/variables.lua`, and the live
//!   switch runs `qs -c end4-pC` directly — both roads end in the same process.
//!
//! The skel is the base `ii`'s Hyprland configuration, taken from upstream's
//! dots with `qsConfig` pointed at this fork — the fork ships no compositor
//! configuration of its own, and upstream's README names exactly that variable
//! as the way to make it default. `config_dir` is not a directory the switch
//! writes: the skel carries `~/.config/quickshell/end4-pC` as a **symlink**
//! into the package's `/usr/share/quickshell/end4-pC` — link and payload are
//! both spelled the config's way, so the QML has one source and `qs -c
//! end4-pC` finds it where Quickshell looks for configurations. The venv
//! bootstrap the fork's colour scripts need rides in
//! `custom/scripts/end4pc-venv-setup.sh`, called from the skel's
//! `custom/execs.lua`: idempotent, so every login after the first is a no-op.
//! `custom/scripts/end4pc-appearance-defaults.sh`, called the same way, seeds
//! the system appearance the fork expects — icons `Colloid-Dark`, cursor
//! `Bibata-Modern-Classic` at size 24, `Inter` 11 and `Maple Mono NF` 11 —
//! only where nothing is set yet, and then applies the cursor that is on
//! record; it replaces the hardcoded `hyprctl setcursor` line upstream keeps
//! in `hyprland/execs.lua`, which would undo a choice made in Settings on
//! every login.

use crate::shell::{Port, Shell, StartMechanism};
use crate::shells::end4pc::IDENTITY;

const PORT: Port = Port {
    config_dir: Some(".config/quickshell/end4-pC"),
    state_dir: None,
    systemd_user_unit: None,
    systemd_user_unit_wanted_by: None,
    mechanism: StartMechanism::StartEvent,
    // Matugen reads its configuration and templates from here; the skel is the
    // only thing that would ever install them, and `backup_extra_paths` is
    // what keeps the user's copy: "being installed" and "being kept" are not
    // the same question, and `plan` deduplicates the path so it is not staged
    // twice.
    skel_extra_paths: &[
        ".config/matugen",
        ".config/ghostty/config",
        ".local/share/applications/dev.noctalia.Noctalia.desktop",
    ],
    backup_extra_paths: &[
        ".config/matugen",
        ".config/ghostty/config",
        ".local/share/applications/dev.noctalia.Noctalia.desktop",
    ],
};

pub static SHELL: Shell = Shell::join(IDENTITY, PORT);
