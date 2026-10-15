//! Noctalia — a native C++ shell, and the one in the Hyprland base image.
//!
//! What is the same wherever it runs:
//!
//! - **install/remove**: the `noctalia` package from the `rakuos` repository. On
//!   the Hyprland base image it is already there, so the install never runs and
//!   the removal never reaches the package manager — [`crate::switch::uninstall`]
//!   drops a package whose origin is `Base` before asking for a password. On a
//!   compositor whose image does not carry it, Niri, the same plan installs and
//!   removes it like any other shell. Both plans exist for that reason: a plan
//!   that a given compositor's image happens to make a no-op is not a wrong plan,
//!   and the alternative — no plan at all — is a shell that can never be removed
//!   where it *was* installed by a switch.
//! - **binary**: `/usr/bin/noctalia`, a native binary rather than a quickshell
//!   process, so the process name is unambiguous.
//! - **logo**: `noctalia.svg`.
//!
//! What differs per compositor is the [`Port`]: Hyprland starts it from
//! `config/autostart.lua`, Niri from `spawn-at-startup`, and the Hyprland page also
//! carries the launcher entry and the theme files its skel replaces.
//!
//! Backup is worth spelling out, because it is easy to get wrong: most of a real
//! Noctalia setup lives in `~/.local/state/noctalia` — 389 of the 445 files a real
//! backup takes. Backing up only the config directory would throw most of it away.

use crate::shell::{Identity, InstallPlan, UninstallPlan};

pub const IDENTITY: Identity = Identity {
    id: "noctalia",
    display_name: "Noctalia",
    upstream: "noctalia-shell",
    package: Some("noctalia"),
    binary: "noctalia",
    start_argv: &["noctalia"],
    process_match: &[],
    // One package, one stage. Noctalia is a native C++ binary that names
    // everything it needs at the package level, so nothing else has to be listed
    // beside it the way DMS's runtime and Caelestia's fonts are.
    install_plan: Some(InstallPlan {
        package: "noctalia",
        stages: &[&["noctalia"]],
        program: "pkexec",
        args: &["rum", "install", "-y", "--refresh"],
    }),
    // Its own package and nothing else. On the image this is filtered out before
    // `rum` sees it — see the module note — and where a switch installed the
    // package this takes it back off.
    uninstall_plan: Some(UninstallPlan {
        package: "noctalia",
        stages: &[&["noctalia"]],
        program: "pkexec",
        args: &["rum", "remove", "-y"],
    }),
    companion_systemd_user_units: &[],
    // No daemon of its own, so nothing to keep in step with systemd.
    autostart_systemd_user_units: &[],
    // No companions, so nothing of theirs to save or bring back.
    companion_paths: &[],
    // Rendered by Noctalia itself from its own templates, not installed by the
    // switch — `[theme.templates] builtin_ids = ["gtk3", "gtk4", "hyprland",
    // "qt"]` writes all four on every start. DMS uses none of them, so they are
    // what is left over after a switch to DMS, and `clean` takes them away again.
    //
    // The other files Noctalia can render are deliberately absent: they go to
    // `btop`, `cava`, `foot`, `ghostty` and the rest, and those templates are not
    // in `builtin_ids` on this machine. What is listed here is what actually
    // lands in a directory the backup already walks.
    generated_paths: &[
        ".config/gtk-3.0/noctalia.css",
        ".config/gtk-4.0/noctalia.css",
        ".config/qt5ct/colors/noctalia.conf",
        ".config/qt6ct/colors/noctalia.conf",
    ],
    // Only the cache: every theme Noctalia renders lands in one of the four
    // directories above, which are in `SHARED_PATHS` and therefore backed up
    // before `clean` touches them. What is left is a megabyte of log and
    // regenerated state under `.cache/noctalia`, which no backup should carry
    // and no switch has a use for afterwards.
    discard_paths: &[".cache/noctalia"],
    // Nothing. Noctalia 5 is a native C++ binary and needs no runtime the rest
    // of the desktop does not already have, so a switch to it keeps nothing back
    // — which is exactly why leaving DMS takes quickshell with it.
    runtime_packages: &[],
    // Noctalia answers polkit itself (`polkit_agent = true`), so it needs no
    // external agent. See `Identity::polkit_agent`.
    polkit_agent: &[],
    logo: Some("noctalia.svg"),
};
