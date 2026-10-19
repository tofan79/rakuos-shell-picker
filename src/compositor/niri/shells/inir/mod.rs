//! iNiR on Niri.
//!
//! Only what is specific to this compositor. What iNiR is — its packages, its
//! launcher, its plans, its paths — is in [`crate::shells::inir`], and is not
//! repeated here.
//!
//! - **start**: the systemd user unit `inir` the package ships. It is `Type=dbus`
//!   on `org.kde.StatusNotifierWatcher`, and `PartOf`/`Requisite`/`After` on
//!   `niri.service`, because Niri owns the session environment the shell renders
//!   into.
//! - **enable**: that unit carries no `[Install]` section, so
//!   `systemctl --user enable` refuses it — which is the whole reason
//!   [`Port::systemd_user_unit_wanted_by`] exists. Upstream enables the shell by
//!   hand, with a symlink into the running compositor's own `.wants/` directory,
//!   and on Niri that is `niri.service.wants/inir.service`. Declaring it here is
//!   the difference between a switch that lasts a session and one that lasts:
//!   the link is what starts the shell at the next login, and `apply` writes it
//!   on the way in and removes it again on the way out.
//! - **config**: `~/.config/illogical-impulse` and `~/.local/state/inir`. Not the
//!   `/usr/share/quickshell/inir` payload — that is the package's, and the skel
//!   does not touch it.
//! - **probe**: the launcher `exec`s the shared runtime, so the process is
//!   `qs -n -p /usr/share/quickshell/inir` — the system payload a switch
//!   installs. See [`crate::shells::inir::IDENTITY`].
//! - **skel**: `~/.config/niri`, upstream's own `defaults/niri` for this
//!   compositor — `config.kdl` plus `config.d/10`-`90` — and the one launcher
//!   entry that hides Noctalia's while iNiR is the shell. `config.kdl` includes
//!   `/usr/share/niri/global.kdl` first, so the image's session-wide environment
//!   and rules are in force under every shell here, and the two files that adapt
//!   the skel to this machine are `40-environment.kdl` (the Qt platform theme)
//!   and `50-startup.kdl` (the clipboard watchers' paths). Both carry a comment
//!   saying what changed and why.
//! - **clean**: `~/.config/niri` comes from the backup taken before every switch,
//!   and iNiR's rendered themes and caches are declared in the identity. Nothing
//!   here is left to the next shell.
//!
//! A fresh switch seeds no configuration of its own: `~/.config/illogical-impulse`
//! is not in the skel because the shell creates it and writes `config.json` from
//! its own defaults on the first run, exactly as it does on upstream's own
//! install. The path is still declared, so a switch away carries whatever the
//! user had configured there.

use crate::shell::{Port, Shell, StartMechanism};
use crate::shells::inir::IDENTITY;

const PORT: Port = Port {
    config_dir: Some(".config/illogical-impulse"),
    state_dir: Some(".local/state/inir"),
    systemd_user_unit: Some("inir"),
    // Niri's own service, so the link `apply` writes is the one upstream's
    // launcher would have written: `niri.service.wants/inir.service`.
    systemd_user_unit_wanted_by: Some("niri.service"),
    mechanism: StartMechanism::SystemdUserUnit,
    // The launcher entry is the counterpart the other Niri skels carry: a switch
    // must not leave Noctalia advertised in the app menu while iNiR is the shell.
    // This copy ships hidden (`NoDisplay=true`), like DMS's.
    skel_extra_paths: &[".local/share/applications/dev.noctalia.Noctalia.desktop"],
    // Same file, listed for the second question a switch asks: nothing else would
    // back it up. The plan deduplicates the path, so it is not staged twice.
    backup_extra_paths: &[".local/share/applications/dev.noctalia.Noctalia.desktop"],
};

pub static SHELL: Shell = Shell::join(IDENTITY, PORT);
