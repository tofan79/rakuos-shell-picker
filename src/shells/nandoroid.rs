//! NAnDoroid — the `na-ive` Quickshell desktop, packaged for RakuOS as
//! `nandoroid-shell`.
//!
//! What is the same wherever it runs (today that is Hyprland only):
//!
//! - **package**: `nandoroid-shell`, from the RakuOS repository. It ships the
//!   QML to `/usr/share/quickshell/nandoroid`, the matugen templates to
//!   `/usr/share/nandoroid/matugen`, and its Hyprland glue to
//!   `/usr/share/nandoroid/hypr` — the last two because the shell expects them
//!   under `~/.config`, where the skel points at them (see the Hyprland port).
//! - **binary**: `nandoroid` is the quickshell config identity, not a program
//!   that ships in `PATH`; the live process is the shared `qs`. The same reason
//!   end4-pC spells its identity out: a `binary` of `qs` would let any machine
//!   with Quickshell on it pass for this shell.
//! - **probe**: `qs -c nandoroid`, which is how the package's own
//!   `hypr/nandoroid.lua` starts it. The two argv words tell it apart from the
//!   sibling Quickshell shells on the same runtime.
//! - **install**: two privileged stages, the runtime first, so a failure there
//!   leaves nothing of the shell on the machine. The rest is the tool set the
//!   panels, the Settings page and the keybinds call out to: `matugen` renders
//!   the palette the shell reads, `hypridle`/`hyprlock`/`hyprsunset` its idle,
//!   lock and blue-light paths, `cliphist` + `wl-clipboard` the clipboard
//!   history, `grim`/`slurp`/`swappy` the region screenshot and annotate steps,
//!   `wf-recorder` and `mpvpaper` the recorder and the video wallpaper,
//!   `brightnessctl` and `playerctl` the two sliders in the quick settings,
//!   `zenity` the file dialogs its scripts use, `translate-shell` the
//!   translation panel, `socat` what its brightness and media helpers bridge
//!   with, `easyeffects` its equalizer, `qt5ct`/`qt6ct`/`nwg-look` the three
//!   theme editors its Settings opens, `python3` its lyrics and theming helpers,
//!   and the three fonts what the interface is drawn with. `cloudflare-warp` is
//!   deliberately *not* here: its WARP quick-settings tile asks
//!   `Network.warpCLIInstalled` before drawing itself, so without the package a
//!   machine gets one button fewer instead of a broken one, and nobody gets a
//!   VPN client they did not ask for.
//! - **python**: the shell's lyrics and KDE theming helpers run in a venv at
//!   `~/.local/share/nandoroid/venv`, which upstream's `install.sh` builds with
//!   `python3 -m venv` plus four pip packages. Nothing builds it here — not the
//!   install plan and not the skel, which carries no script — and the shell runs
//!   without it: only lyrics and the KDE colour export go quiet, which is why a
//!   machine with no network at login still gets a desktop. A venv somebody
//!   builds by hand is thrown away with the shell (`discard_paths`).
//! - **settings**: `~/.config/nandoroid/config.json`, written by the shell as
//!   the person rearranges it. It sits outside the QML directory, so the
//!   package-backed `config_dir` symlink does not cover it; see
//!   `companion_paths`.
//!
//! Nothing here is written to `~/.config/hypr` except through the file the
//! shell persists its own monitor and game-mode choices in
//! (`~/.config/hypr/nandoroid/user_persistence.lua`), which the skel seeds
//! empty and the Hyprland config loads last. The compositor configuration
//! itself belongs to the skel; upstream's `install.sh` is never run, so the
//! dotfiles it would spread over kitty, fish and starship stay where they are.

use crate::shell::{Identity, InstallPlan, UninstallPlan};

pub const IDENTITY: Identity = Identity {
    id: "nandoroid",
    display_name: "NAnDoroid",
    upstream: "na-ive/nandoroid-shell",
    package: Some("nandoroid-shell"),
    binary: "nandoroid",
    start_argv: &["qs", "-c", "nandoroid"],
    process_match: &["-c", "nandoroid"],
    install_plan: Some(InstallPlan {
        package: "nandoroid-shell",
        stages: &[
            &["quickshell-git"],
            &[
                "nandoroid-shell",
                // The Qt side the QML needs on a fresh machine, in Fedora's
                // names: the declarative runtime, the Wayland platform plugin,
                // SVG (the icons are `image://icon` lookups, so without
                // `qt6-qtsvg` they come out blank), and `qt6-qt5compat` for the
                // `Qt5Compat.GraphicalEffects` imports the dock and dashboard
                // still carry. `qt5-qtsvg` is *not* one of them: the effects are
                // Qt6's rewrite and need no Qt5 runtime — the picker asked for it
                // once and the machine has never had the package installed while
                // the shell ran fine. Then the three config editors its Settings
                // page launches (`qt5ct`, `qt6ct`, `nwg-look`).
                "qt6-qtdeclarative",
                "qt6-qtwayland",
                "qt6-qtsvg",
                "qt6-qt5compat",
                "qt5ct",
                "qt6ct",
                "nwg-look",
                // Portals: the shell's screenshot, screencast and file choosers
                // go through them. In the image already; named here so a machine
                // that somehow lost one gets it back with the shell.
                "xdg-desktop-portal-hyprland",
                "xdg-desktop-portal-gtk",
                "matugen",
                "hypridle",
                "hyprlock",
                "hyprsunset",
                "cliphist",
                "wl-clipboard",
                "grim",
                "slurp",
                "swappy",
                // The annotation editor behind the `satty` switch in Settings
                // (the default is `swappy`), and the calculator the launcher
                // runs as `qalc -t`; without them both do nothing.
                "satty",
                "qalc",
                // `SystemInfo.qml` reads CPU deltas from `dgop` and only
                // falls back to `ps` when it is missing.
                "dgop",
                "wf-recorder",
                "mpvpaper",
                "socat",
                "brightnessctl",
                "playerctl",
                "zenity",
                "translate-shell",
                "easyeffects",
                "python3",
                "material-symbols-fonts",
                "google-rubik-fonts",
                "cascadia-code-nf-fonts",
            ],
        ],
        program: "pkexec",
        args: &["rum", "install", "-y", "--refresh"],
    }),
    uninstall_plan: Some(UninstallPlan {
        package: "nandoroid-shell",
        // The packages the shell itself brought: its own tools, the fonts, and
        // the runtime it dragged in. The Qt libraries and the two portals are
        // deliberately absent — they are the image's, every Qt application and
        // every desktop on the machine uses them, and a switch away must not
        // take the screenshot backend out from under the next shell. A package
        // that something else still requires is refused by `rum remove` anyway.
        //
        // The half of this list that is *also* another shell's business is safe
        // for the same reason: `cliphist`, `wl-clipboard`, `grim`, `slurp`,
        // `brightnessctl`, `playerctl`, `qt6ct` and `python3` are in the
        // read-only image (`rpm -q` answers for each of them), so a switch away
        // keeps them whatever this plan says — `switch::uninstall` reports them
        // as `image_owned` and the next shell still finds its clipboard watcher,
        // its screenshot key and its Qt platform theme. What this plan actually
        // takes off the machine is the overlay half: the shell, its idle/lock/
        // blue-light helpers, the annotator, the recorder, the video wallpaper,
        // `socat`, the translation panel, the equaliser, the three theme editors,
        // `zenity` and the fonts.
        stages: &[&[
            "nandoroid-shell",
            "quickshell-git",
            "matugen",
            "hypridle",
            "hyprlock",
            "hyprsunset",
            "cliphist",
            "wl-clipboard",
            "grim",
            "slurp",
            "swappy",
            "satty",
            "qalc",
            "dgop",
            "wf-recorder",
            "mpvpaper",
            "socat",
            "brightnessctl",
            "playerctl",
            "zenity",
            "translate-shell",
            "easyeffects",
            "qt5ct",
            "qt6ct",
            "nwg-look",
            "material-symbols-fonts",
            "google-rubik-fonts",
            "cascadia-code-nf-fonts",
        ]],
        program: "pkexec",
        args: &["rum", "remove", "-y"],
    }),
    companion_systemd_user_units: &[],
    autostart_systemd_user_units: &[],
    // The settings the shell writes when a person moves a widget, picks a
    // wallpaper, pins an app or changes a panel: `~/.config/nandoroid/config.json`
    // and its `presets/`. Upstream keeps them outside the QML directory on
    // purpose, which is why `config_dir` (the symlink into the package) does not
    // cover them — a switch away that ignored this path would hand back a shell
    // at its defaults. Real data, so it is a companion and not a discard: backed
    // up with the shell, removed only when this switch's copy reproduces it byte
    // for byte, and read back when NAnDoroid comes home.
    //
    // `~/.local/state/quickshell/user/generated` (the matugen colour JSON) is
    // not here: end4-pC declares that directory as a companion, both shells
    // render the same file from the same wallpaper, and two shells claiming one
    // directory would have each other's backup argue about it.
    companion_paths: &[".config/nandoroid"],
    // Matugen renders both shared GTK sheets on every colour change; both
    // directories are in `switch::SHARED_PATHS`, so a backup holds them before
    // `clean` may take either away. `.config/matugen` is the shell-specific
    // configuration installed by NAnDoroid's skel. Keep it in the same
    // byte-for-byte cleanup path so a switch to DMS cannot leave NAnDoroid's
    // user templates enabled (the backup is still used if NAnDoroid returns).
    generated_paths: &[
        ".config/gtk-3.0/gtk.css",
        ".config/gtk-4.0/gtk.css",
        ".config/matugen",
    ],
    // Regenerated output and cached state, none of it written by the user:
    // `~/.cache/nandoroid` is wallpaper/weather thumbnails plus the alarm,
    // reminder and favourites lists upstream itself keeps under `cache`;
    // `~/.cache/quickshell/nandoroid` is the todo list, the reminder list and
    // the cover art the shell re-fetches — Quickshell's own `_qt_*` and
    // `qtpipelinecache-*` siblings stay, they are shared with every Quickshell
    // shell and with Qt itself; `~/.local/share/nandoroid` is the Python venv
    // for the lyrics and KDE colour export, which the install plan does not
    // build and which is worthless without the shell; and
    // `~/.local/state/quickshell/quickshell/nandoroid_states.json` is
    // Quickshell's own record of where a popup was left, rewritten on the next
    // move.
    discard_paths: &[
        ".cache/nandoroid",
        ".cache/quickshell/nandoroid",
        ".local/share/nandoroid",
        ".local/state/quickshell/quickshell/nandoroid_states.json",
    ],
    // The runtime and the theming tool it shares with the other Quickshell
    // shells, the Qt libraries and portals every desktop needs, and the
    // icon/font trio its interface is drawn with.
    runtime_packages: &[
        "quickshell",
        "quickshell-git",
        "qt6-qtdeclarative",
        "qt6-qtwayland",
        "qt6-qtsvg",
        "qt6-qt5compat",
        "xdg-desktop-portal-hyprland",
        "xdg-desktop-portal-gtk",
        "matugen",
        // The video-wallpaper helper. Its QML spawns `mpvpaper` per monitor when
        // the wallpaper is a video. Named here as well as in the install plan
        // because the two lists answer different questions: the plan says what a
        // switch brings, this says what a switch *into* NAnDoroid must not take
        // away.
        "mpvpaper",
        // The equaliser its Settings page opens. Named here as well as in the
        // install plan because the two lists answer different questions: the
        // plan says what a switch brings, this says what a switch *into* this
        // shell must not take away — and `easyeffects` is also Ambxst's and
        // end4-pC's, so a switch between them used to remove it from under the
        // other shell.
        "easyeffects",
        "material-symbols-fonts",
        "google-rubik-fonts",
        "cascadia-code-nf-fonts",
    ],
    // The package ships `services/PolkitService.qml`, a `PolkitAgent` that
    // registers with polkitd when the shell starts, so the privilege stage has
    // somebody to ask without an external agent. See `Identity::polkit_agent`.
    polkit_agent: &[],
    logo: Some("nandoroid.svg"),
};
