//! Impasto on Hyprland.
//!
//! Only what is specific to this compositor. Everything about Impasto itself —
//! its package, its plans, that it is its own polkit agent — is in
//! [`crate::shells::impasto`], and is not repeated here.
//!
//! - **config**: `~/.config/quickshell/impasto`, the symlink the skel creates
//!   into `/usr/share/quickshell/impasto`. Upstream installs the shell at the
//!   root of `~/.config/quickshell`; the package moves it one level down so that
//!   end4-pC, which owns a subdirectory there, is not in the way, and the symlink
//!   is what makes `qs -c impasto` resolve.
//! - **state**: `~/.local/state/impasto`, which the skel seeds with a `version`
//!   file (this package's version, an empty repository). Without it the System
//!   page offers a `./setup update` that does not exist in a packaged install;
//!   with an empty repository its own check returns early.
//! - **start**: the compositor's start event, from the skel's own
//!   `config/autostart.lua`: upstream's `modules/autostart.lua` runs `qs -d`,
//!   which would look for the shell at the root of `~/.config/quickshell`. The
//!   rest of `modules/` is symlinked in from the package unchanged.
//! - **backup**: the two directories above, `~/.local/share/impasto` (a
//!   companion: profiles, camera samples and the palette board are the user's),
//!   and the hidden Noctalia launcher entry. The skel seeds that companion with
//!   the palette board and the three stock profiles from upstream's
//!   `home/.local/share/impasto`, since `./setup` — which normally copies them
//!   out of `/usr/share/impasto/share` — is never run; a backup of this shell
//!   wins over the seed, so a user's own board and profiles come back intact.
//! - **clean**: `~/.cache/impasto` through `Identity::discard_paths`; the two
//!   GTK sheets in `Identity::generated_paths`; the data directories once this
//!   switch's backup reproduces them.
//! - **verify**: a process whose command line carries `-c impasto` is alive.
//!
//! Nothing from upstream's `system/` is installed and `./setup` is never run:
//! the SDDM theme, `/etc/pam.d/impasto-face`, the polkit actions and the logind
//! drop-in are login-screen and system policy, which a shell switch has no
//! business changing.

use crate::shell::{Port, Shell, StartMechanism};
use crate::shells::impasto::IDENTITY;

const PORT: Port = Port {
    config_dir: Some(".config/quickshell/impasto"),
    state_dir: Some(".local/state/impasto"),
    systemd_user_unit: None,
    systemd_user_unit_wanted_by: None,
    mechanism: StartMechanism::StartEvent,
    skel_extra_paths: &[".local/share/applications/dev.noctalia.Noctalia.desktop"],
    backup_extra_paths: &[".local/share/applications/dev.noctalia.Noctalia.desktop"],
};

pub static SHELL: Shell = Shell::join(IDENTITY, PORT);
