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
//!   and `50-startup.kdl` (the clipboard watchers' paths, the icon theme GTK
//!   apps read, and the Python environment iNiR's own scripts run in). Both
//!   carry a comment saying what changed and why.
//! - **clean**: `~/.config/niri` comes from the backup taken before every switch,
//!   and iNiR's rendered themes and caches are declared in the identity. Nothing
//!   here is left to the next shell.
//!
//! The skel also seeds two files of iNiR's own: the shell's configuration,
//! `~/.config/illogical-impulse/config.json` — the path its own
//! `Directories.shellConfig` resolves to, old name and all — with the icon theme
//! and the three font families, and `scripts/inir-venv-setup.sh`, the bootstrap
//! of the Python environment iNiR's scripts run in.
//!
//! The configuration file holds names iNiR would otherwise take from defaults
//! compiled into its `modules/common/Config.qml`, and those defaults are the
//! reason it is seeded at all. `appearance.iconTheme` starts as `WhiteSur-dark`,
//! a family no repository RakuOS enables packages. iNiR renders whatever name it
//! holds into gsettings, `gtk-3.0`/`gtk-4.0/settings.ini`, `kdeglobals` and
//! `qt6ct`, and takes the icons of the session with it: a GTK app asked for a
//! theme that is not installed draws its fallback, which reads as missing icons
//! rather than a different look. Seeding `Colloid-Dark` — the theme
//! `rakuos-branding` installs, and the name DMS's and Caelestia's skels already
//! render — is what holds the shell, and every session after it, on an icon theme
//! this machine has. It matters outside the session too: the name is one dconf
//! key, and no switch replaces dconf, so a switch away carries it to the next
//! shell.
//!
//! The three families are the quieter half of the same story. `Roboto Flex`,
//! `Gabarito` and `JetBrainsMono Nerd Font` are no more available here than
//! `WhiteSur-dark` is, and iNiR writes the names it holds into those same
//! settings files (`syncWithSystem` is on), so what the shell holds is what the
//! rest of the desktop reads: `Inter` and `Maple Mono NF`, the sans and the
//! monospace the image already sets system-wide, are what a session gets instead
//! of each toolkit quietly falling back on its own.
//!
//! A handful of keys is enough because the shell reads the file as a set of
//! properties ([`JsonAdapter`](https://quickshell.org/docs/types/Quickshell.Io/JsonAdapter/)):
//! every key the file does not carry keeps the value the QML declares, so seeding
//! these neither freezes upstream's defaults at the version this skel was written
//! against nor survives past what the user chooses in the shell's own settings,
//! which grep the file back into shape. Everything else in the path is still the
//! shell's own, and the path is declared, so a switch away carries whatever is in
//! it.
//!
//! The script is the one step a switch does not take. Upstream builds the venv
//! from its own installer — `./setup install`, or `./setup doctor` to repair it —
//! and nobody runs that on a machine this picker set up; the package ships the
//! requirements as data and the venv as something to make. So `50-startup.kdl`
//! runs the script at login instead: idempotent, everything it says in a log
//! beside the venv, and declared in the identity's `discard_paths` so it leaves
//! with the venv the moment iNiR is left and returns at the next login that has
//! iNiR again. Nothing about it is required for the shell to appear — the parts
//! that need Python (the palette from a wallpaper, the thumbnails, the AI
//! features) are simply degraded until the first build finishes.
//!
//! It lives in this tree rather than under the shell's own config directory, and
//! the distinction is the one [`apply::plan`] draws: this directory is installed
//! from the skel on every switch, while a shell's configuration comes from that
//! shell's own backup when there is one, skel only as the fallback. A script
//! under `~/.config/illogical-impulse` would therefore never arrive on a machine
//! that has run iNiR before — the backup would be the whole of that directory,
//! minus a file added after it was taken — while the startup line that calls it
//! comes from this tree either way. Both halves on the same side of the rule is
//! what keeps a login from calling a file that is not there.

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
    //
    // The autostart file masks KDE's Xwayland video bridge, which the image
    // starts from `/etc/xdg/autostart`. Its only job is to put Wayland windows
    // into the screen-share picker of X11 applications, but it keeps an
    // undeletable "Wayland to X Recording bridge" window in the compositor's
    // window list all session. `Hidden=true` in a user autostart file makes
    // systemd-xdg-autostart-generator skip the entry, so the helper never runs;
    // Wayland applications share through the portal as before. Deleting the
    // file brings the bridge back for somebody who does need it.
    skel_extra_paths: &[
        ".local/share/applications/dev.noctalia.Noctalia.desktop",
        ".config/autostart/org.kde.xwaylandvideobridge.desktop",
        ".config/ghostty/config",
    ],
    // Same files, listed for the second question a switch asks: nothing else
    // would back them up. The plan deduplicates the paths, so none is staged
    // twice — and because they are backed up, switching away takes the
    // autostart override with it instead of leaving it for the next shell.
    backup_extra_paths: &[
        ".local/share/applications/dev.noctalia.Noctalia.desktop",
        ".config/autostart/org.kde.xwaylandvideobridge.desktop",
        ".config/ghostty/config",
    ],
};

pub static SHELL: Shell = Shell::join(IDENTITY, PORT);
