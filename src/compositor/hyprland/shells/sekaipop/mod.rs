//! SekaiPop on Hyprland.
//!
//! Only what is specific to this compositor. Everything about SekaiPop itself —
//! its package, its plans, that it is its own polkit agent — is in
//! [`crate::shells::sekaipop`], and is not repeated here.
//!
//! - **config**: `~/.config/quickshell/sekaipop`, the symlink the skel creates
//!   into `/usr/share/quickshell/sekaipop` — the path `qs -c sekaipop` resolves,
//!   so it is what a switch has to write for the shell to start at all. It is
//!   also how the package's updater stays out of `/usr`: its script refuses to
//!   replace a symlinked shell. The shell's settings live in `~/.config/sekaipop`,
//!   which `Identity::companion_paths` declares.
//! - **start**: the compositor's start event. `~/.config/hypr/hyprland/execs.lua`,
//!   which the skel carries, arms `hl.on("hyprland.start")`, so a login and a
//!   switch start the shell the same way. The skel also carries upstream's
//!   `hypridle.conf`, because that file locks through the shell
//!   (`qs -c sekaipop ipc call lock lock`).
//! - **backup**: `config_dir`, `state_dir`, `~/.local/share/sekaipop` (a
//!   companion: the shell writes it, so it is the user's), the symlinked shell
//!   directory, and the hidden Noctalia launcher entry.
//! - **clean**: the two directories once this switch's backup reproduces them,
//!   and `~/.cache/sekaipop` (`Identity::discard_paths`).
//! - **verify**: a process whose command line carries `-c sekaipop` is alive.
//!
//! **`monitors.lua` is deliberately absent from the skel.** Upstream generates it
//! for the machine it is installed on and never ships one, so a shipped copy
//! would be another machine's layout — the black-screen case. A switch replaces
//! `~/.config/hypr` whole, so the user's own `monitors.lua` is kept in the backup
//! of that switch rather than being carried over; the shell's Settings writes a
//! fresh one from the displays it finds.
//!
//! The Hyprland configuration is upstream's `hypr/` tree, copied into the skel
//! whole: `hyprland.lua`, `hyprland/*.lua` with `lib/init.lua` and the two
//! scripts the keybinds call, `custom/*.lua`, `hypridle.conf` and
//! `sekaipop_overrides.lua`. A copy rather than a symlink into
//! `/usr/share/sekaipop/hypr` so that a switch writes one tree it owns — the same
//! one it backs up and restores — and the session never depends on the package's
//! copy still being at that path. Upstream's `pjsekai_overrides.lua` stays out:
//! `hyprland.lua` does not load it any more, the shell writes
//! `sekaipop_overrides.lua` instead.

use crate::shell::{Port, Shell, StartMechanism};
use crate::shells::sekaipop::IDENTITY;

const PORT: Port = Port {
    // The symlink, not `~/.config/sekaipop`: the shell cannot start without it,
    // and it is the one path a switch must write. The settings directory is the
    // shell's own, so it is declared as a companion in `Identity` instead — the
    // same split end4-pC's port uses.
    config_dir: Some(".config/quickshell/sekaipop"),
    state_dir: Some(".local/state/sekaipop"),
    systemd_user_unit: None,
    systemd_user_unit_wanted_by: None,
    mechanism: StartMechanism::StartEvent,
    skel_extra_paths: &[".local/share/applications/dev.noctalia.Noctalia.desktop"],
    // A switch must not leave Noctalia advertised in the app menu while SekaiPop
    // is the shell, and the two theme copies are the only place its palette is
    // left behind on disk: `generated_paths` removes them, so the backup has to
    // walk them (`SHARED_PATHS` does not, and it should not — every switch would
    // then copy every theme a person owns).
    backup_extra_paths: &[
        ".local/share/applications/dev.noctalia.Noctalia.desktop",
        ".local/share/themes/sekaipop-adw-gtk3",
        ".local/share/themes/sekaipop-adw-gtk3-dark",
    ],
};

pub static SHELL: Shell = Shell::join(IDENTITY, PORT);
