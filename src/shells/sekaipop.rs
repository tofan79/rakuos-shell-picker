//! SekaiPop — the `Blapples` Quickshell desktop, packaged for RakuOS as
//! `sekaipop-shell`.
//!
//! What is the same wherever it runs (today that is Hyprland only):
//!
//! - **package**: `sekaipop-shell`, from the RakuOS repository. It ships the QML
//!   to `/usr/share/quickshell/sekaipop` and its Hyprland configuration to
//!   `/usr/share/sekaipop/hypr`. The skel carries the compositor half and points
//!   `~/.config/quickshell/sekaipop` at the package, so `qs -c sekaipop` finds
//!   both without anything being copied into `$HOME` at package install time.
//! - **binary**: `sekaipop` is the quickshell config identity, not a program that
//!   ships in `PATH`. The live process is the shared `qs`, which is why the
//!   program name cannot be the probe.
//! - **probe**: `qs -c sekaipop`, the command its own `hypr/hyprland/execs.lua`
//!   runs at login, and the two argv words tell it apart from every other
//!   Quickshell shell.
//! - **install**: two privileged stages, the runtime first. The rest is what the
//!   shell's own configuration calls: `matugen` for the palette, `cliphist` and
//!   `wl-clipboard` for the clipboard, `grim`/`slurp`/`swappy` for the region
//!   capture, `hyprsunset` for the Night Light toggle and `wf-recorder` for the
//!   region recorder, `zenity` for the file dialogs its scripts fall back to,
//!   `ImageMagick` and `ffmpeg` for wallpaper and colour work, `hypridle` with
//!   `hyprlock` as the fallback lock, `easyeffects` for the equalizer panel its
//!   `execs.lua` starts, `adw-gtk3-theme` because its GTK3 recolouring copies
//!   that theme and does nothing without it, the two sliders' tools, `python3`
//!   for its helper scripts — and `python3-numpy`, which `sekai_analyzer.py`
//!   imports outright for the Sekai Pad — and the fonts its interface is drawn
//!   with. `cava`, `jq`, `wtype` and `hyprpicker` are not named: the image has
//!   them, and a switch can only remove what a shell's own plan installed.
//! - **config**: the shell keeps its settings in `~/.config/sekaipop` and its
//!   cache — thumbnails, and `hypr-errors.log`, which is where its own Hyprland
//!   config writes a file that failed to load — in `~/.cache/sekaipop`.
//!   `~/.local/share/sekaipop` holds what it renders and keeps.
//! - **updater**: `UpdateService.qml` can download a new copy of the shell over
//!   the installed one. Against an RPM that must never happen, and the package
//!   neutralises it in the QML; the skel does its part by pointing the config at
//!   a symlink, which the updater's own script refuses to replace.
//!
//! Its lock screen answers with the system `login` PAM service, which includes
//! `system-auth` — so `pam_faillock` applies. `docs/SAFETY.md` says what that
//! means and how to get back in.

use crate::shell::{Identity, InstallPlan, UninstallPlan};

pub const IDENTITY: Identity = Identity {
    id: "sekaipop",
    display_name: "SekaiPop",
    upstream: "Blapples/SekaiPop",
    package: Some("sekaipop-shell"),
    binary: "sekaipop",
    start_argv: &["qs", "-c", "sekaipop"],
    process_match: &["-c", "sekaipop"],
    install_plan: Some(InstallPlan {
        package: "sekaipop-shell",
        stages: &[
            &["quickshell-git"],
            &[
                "sekaipop-shell",
                "matugen",
                "cliphist",
                "wl-clipboard",
                "grim",
                "slurp",
                "swappy",
                "hyprsunset",
                "wf-recorder",
                "zenity",
                "ImageMagick",
                "ffmpeg",
                "hypridle",
                "hyprlock",
                // `MpvpaperWallpaper.qml` is the video-wallpaper backend the
                // Settings offer beside the Qt multimedia one.
                "mpvpaper",
                "easyeffects",
                "adw-gtk3-theme",
                "brightnessctl",
                "playerctl",
                "python3",
                "python3-numpy",
                "material-symbols-fonts",
                "google-rubik-fonts",
                "cascadia-code-nf-fonts",
            ],
        ],
        program: "pkexec",
        args: &["rum", "install", "-y", "--refresh"],
    }),
    uninstall_plan: Some(UninstallPlan {
        package: "sekaipop-shell",
        // Everything the install put there, the shared runtime included; the
        // switch drops whatever the incoming shell declares in
        // `runtime_packages`, which is what keeps `quickshell-git` and the fonts
        // for a shell that needs them too.
        stages: &[&[
            "sekaipop-shell",
            "quickshell-git",
            "matugen",
            "cliphist",
            "wl-clipboard",
            "grim",
            "slurp",
            "swappy",
            "hyprsunset",
            "wf-recorder",
            "zenity",
            "ImageMagick",
            "ffmpeg",
            "hypridle",
            "hyprlock",
            "mpvpaper",
            "easyeffects",
            "adw-gtk3-theme",
            "brightnessctl",
            "playerctl",
            "python3-numpy",
            "material-symbols-fonts",
            "google-rubik-fonts",
            "cascadia-code-nf-fonts",
        ]],
        program: "pkexec",
        args: &["rum", "remove", "-y"],
    }),
    companion_systemd_user_units: &[],
    autostart_systemd_user_units: &[],
    // Settings, scale and plugins live in `~/.config/sekaipop`; the shell writes
    // them and reads them back, so the directory is the user's and travels with
    // them. `~/.local/share/sekaipop` holds the copies of the pictures the user
    // picked and the presets exported from them, which cannot be rebuilt from
    // the package either.
    companion_paths: &[".config/sekaipop", ".local/share/sekaipop"],
    // Matugen renders the two shared GTK sheets on every colour change. Both
    // directories are in `switch::SHARED_PATHS`, so a backup holds them before
    // `clean` may take either away.
    //
    // The app-theme switches reach further than this, into other programs' own
    // configs: kitty, foot, fuzzel, rofi, fish, `kdeglobals`, the Vencord and
    // Vesktop theme folders, `~/.local/share/themes/sekaipop-adw-gtk3*`, the
    // cursor theme, the KDE colour scheme and the EasyEffects presets. None of
    // them is declared here on purpose — they belong to those programs, a switch
    // overwrites none of them, and the shell rewrites them from its own settings
    // the next time one of those switches is turned on. Deleting them on the way
    // out would take a person's terminal prompt and colour scheme with it.
    generated_paths: &[
        ".config/gtk-3.0/gtk.css",
        ".config/gtk-4.0/gtk.css",
        // The sheet `gtk.css` imports, written beside it by the same theme
        // manager. Kept off the plans until now, so a switch away left it
        // behind and the next shell's `gtk.css` inherited SekaiPop's palette
        // through the dangling import.
        ".config/gtk-3.0/sekaipop-colors.css",
        ".config/gtk-4.0/sekaipop-colors.css",
        // A copy of adw-gtk3 with the palette added to the theme itself, so GTK3
        // apps recolour live when `gtk-theme` changes. The shell rebuilds it on
        // every run, nothing else reads a theme named after this shell, and the
        // copy is 1.2 MB per variant sitting in the home directory forever
        // otherwise. `gtk-theme` is left pointing at it by the shell, so a switch
        // away falls back to the base theme rather than to a half-deleted one.
        ".local/share/themes/sekaipop-adw-gtk3",
        ".local/share/themes/sekaipop-adw-gtk3-dark",
    ],
    // Thumbnails and the Hyprland error log: rebuilt on the next start, and the
    // log describes a session that is over by the time a switch runs.
    discard_paths: &[".cache/sekaipop"],
    runtime_packages: &[
        "quickshell",
        "quickshell-git",
        "matugen",
        "material-symbols-fonts",
        "google-rubik-fonts",
        "cascadia-code-nf-fonts",
        "easyeffects",
    ],
    // Its `execs.lua` says the shell is the session's polkit agent and that no
    // other one may run. See `Identity::polkit_agent`.
    polkit_agent: &[],
    logo: Some("sekaipop.svg"),
};
