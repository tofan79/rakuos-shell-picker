//! Ambxst, a packaged shell started as a plain process.
//!
//! What is the same wherever it runs:
//!
//! - **package**: `ambxst`, from the same `rakuos` overlay the other non-base
//!   shells live in. `rum origin ambxst` reports Overlay and the binary is owned
//!   by the package, so the earlier reading of it as unpackaged was wrong: it is
//!   a real package, installable and removable like DMS.
//!
//! - **install**: two privileged stages, runtime first:
//!
//!   ```text
//!   1. quickshell-git
//!   2. ambxst axctl matugen phosphor-icons-fonts mate-polkit
//!   ```
//!
//!   `quickshell-git` is named explicitly because nothing requires it: Ambxst
//!   starts `qs -p /usr/share/ambxst/shell.qml`, but its own `Requires` names
//!   only libc, libpam and the rest — no runtime. Left to the solver the shell
//!   would be installed and unable to draw anything, which is the same reason
//!   DMS and Caelestia name it. The runtime is a stage of its own so that if it
//!   fails there is then nothing of Ambxst on the machine at all.
//!
//!   `axctl` is the companion daemon (`AxctlService.qml`), `matugen` renders the
//!   theme from the wallpaper, `phosphor-icons-fonts` supplies the icon glyphs
//!   the shell draws, `mate-polkit` answers its `pkexec` and `easyeffects` runs
//!   its audio processing. No package requires any of them; nothing but this
//!   list would put them on the machine.
//!
//! - **uninstall**: one stage over Ambxst's own packages, `quickshell-git`,
//!   `matugen` and `mate-polkit` included because a switch to a shell that
//!   does not use them should not leave them orphaned. All three are shared —
//!   DMS runs on the same runtime and renders with the same matugen, and
//!   Caelestia starts the same polkit agent — so `switch::uninstall` drops any
//!   of them the shell being switched to declares in `runtime_packages`. The
//!   icon font and `easyeffects` are in the plan for the same reason: both were
//!   installed for this shell, and the guard keeps whichever the target still
//!   declares.
//!
//! - **start**: a direct process, `ambxst`. Ambxst arms its own
//!   `hyprland.start` hook from the `~/.local/share/ambxst/hyprland.lua` it
//!   regenerates, which this shell's Hyprland skel loads — so the live switch
//!   starts it here, and the next login starts it from that hook.
//!
//! Ambxst keeps data of its own outside `config_dir`, and it is declared as
//! [`crate::shell::Identity::companion_paths`] so a switch saves it, cleans it
//! and reads it back: `~/.local/share/ambxst` holds the chat history, the pinned
//! clipboard databases and the compositor configs Ambxst regenerates, and
//! `~/.local/share/ambxst-notes` holds its notes. `axctl`'s brightness record,
//! in `~/.config/axctl`, travels the same way. Nothing installs any of it, so
//! the backup is the only thing that can hand it back after Ambxst has been
//! away — which is why none of it can be left to sit through a switch.
//!
//! Upstream note, for whoever adds a Niri port: Ambxst generates its Niri config
//! into `~/.local/share/ambxst/niri.kdl` and pulls it in with an include block,
//! so there may be no hand-written config directory to point at. It would not be
//! the Hyprland port with a different path.

use crate::shell::{Identity, InstallPlan, UninstallPlan};

pub const IDENTITY: Identity = Identity {
    id: "ambxst",
    display_name: "Ambxst",
    upstream: "Axenide/Ambxst",
    package: Some("ambxst"),
    binary: "ambxst",
    start_argv: &["ambxst"],
    process_match: &[],
    install_plan: Some(InstallPlan {
        package: "ambxst",
        stages: &[
            // The runtime first and on its own, mirroring DMS and Caelestia: it
            // is the piece most likely to fail, and nothing in the `ambxst`
            // package's own requirements would ever pull it in.
            &["quickshell-git"],
            &[
                "ambxst",
                "axctl",
                "matugen",
                "phosphor-icons-fonts",
                "mate-polkit",
                "easyeffects",
            ],
        ],
        program: "pkexec",
        // `--refresh` for the same reason as DMS: a cached copy of the metadata
        // is only known to be young, not known to be current. See
        // `switch::install`.
        args: &["rum", "install", "-y", "--refresh"],
    }),
    uninstall_plan: Some(UninstallPlan {
        package: "ambxst",
        // Ambxst's own package, its companion daemon, and the pieces it
        // installed. `switch::uninstall` drops any of the shared ones the shell
        // being switched to declares in `runtime_packages`, so a switch to DMS
        // keeps what DMS shares and a switch to Noctalia takes them all. The
        // icon font and `easyeffects` were installed with Ambxst, so they travel
        // with it too.
        stages: &[&[
            "ambxst",
            "axctl",
            "matugen",
            "phosphor-icons-fonts",
            "quickshell-git",
            "mate-polkit",
            "easyeffects",
        ]],
        program: "pkexec",
        args: &["rum", "remove", "-y"],
    }),
    companion_systemd_user_units: &[],
    // No daemon of its own, so nothing to keep in step with systemd.
    autostart_systemd_user_units: &[],
    // Where the companion keeps its own data, and where the shell keeps data
    // that is not configuration. `axctl` records the brightness it has applied
    // in `~/.config/axctl/brightness.tsv`; Ambxst keeps the chat history and the
    // pinned clipboard in `~/.local/share/ambxst`, and its notes in
    // `~/.local/share/ambxst-notes`. Nothing installs this data and no skel
    // carries it, so the backup is the only thing that can hand it back after
    // Ambxst has been away — the skel seeds only the compositor config Ambxst
    // regenerates there. Each sits outside `config_dir`, so each has to be
    // declared or a switch away would take it with no copy anywhere.
    companion_paths: &[
        ".config/axctl",
        ".local/share/ambxst",
        ".local/share/ambxst-notes",
    ],
    // Rendered by Ambxst itself, not installed by the switch. Ambxst's own Qt
    // template (`modules/theme/QtCtGenerator.qml`) writes the `[ColorScheme]`
    // palette into both of these on every start, so the skel does not carry
    // them: shipping a copy the switch never installs would be a seed only in
    // name. Nothing else on the desktop writes either filename, so they are what
    // is left over after a switch to another shell, and `clean` takes them away
    // once this switch's backup reproduces them. Both sit under `~/.config/qt5ct`
    // and `~/.config/qt6ct`, which `switch::SHARED_PATHS` backs up.
    generated_paths: &[
        ".config/qt5ct/colors/ambxst.colors",
        ".config/qt6ct/colors/ambxst.colors",
    ],
    // Rendered output and cached state, none of it written by the user: colour
    // schemes, wallpapers, usage counters and icon caches Ambxst rebuilds on the
    // next start. `~/.cache/quickshell` is the same kind of thing one level out:
    // the QML and pipeline caches written by the Quickshell runtime Ambxst runs
    // on, which DMS declares too and neither shell needs after a switch. They
    // live under `~/.cache`, which no backup walks —
    // `every_discard_path_is_never_preserved` fails the suite if that ever
    // changes.
    discard_paths: &[".cache/ambxst", ".cache/quickshell"],
    // The runtime and theming tool Ambxst shares with DMS, and the polkit agent
    // it shares with Caelestia. All have to be named so a switch between the
    // shells that use them keeps them, while a switch to a shell that needs none
    // (Noctalia) takes them away.
    runtime_packages: &["quickshell", "quickshell-git", "matugen", "mate-polkit"],
    // Each compositor's skel starts this — from `config/autostart.lua` on
    // Hyprland, from `spawn-at-startup` in the Niri skel's `config.kdl` — and a
    // live switch has to start it itself. See `Identity::polkit_agent`.
    polkit_agent: &["/usr/libexec/polkit-mate-authentication-agent-1"],
    logo: Some("ambxst-logo-color.svg"),
};
