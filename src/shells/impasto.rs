//! Impasto — the `andreumassanet` Quickshell desktop, packaged for RakuOS as
//! `impasto-shell`.
//!
//! What is the same wherever it runs (today that is Hyprland only):
//!
//! - **package**: `impasto-shell` ships the QML to `/usr/share/quickshell/impasto`,
//!   its Hyprland modules to `/usr/share/impasto/hypr`, its samples and profiles
//!   to `/usr/share/impasto/share`, and the `impasto` CLI to `/usr/bin`. Upstream
//!   installs the shell *at the root* of `~/.config/quickshell`; the package
//!   moves it one level down and the skel symlinks it back under the config
//!   name, so `qs -c impasto` works and end4-pC's subdirectory is not disturbed.
//! - **binary**: `impasto` is the CLI, a real program, so `installed()` follows
//!   the package exactly as it does for the shells with a launcher of their own.
//!   It is *not* the shell: the live process is the shared `qs`.
//! - **probe**: `qs -c impasto`, the command its own `modules/autostart.lua`
//!   runs at login, with the config name as the second argv word.
//! - **install**: two privileged stages, the runtime first. `awww` is the
//!   wallpaper daemon the palette is derived from — without it the shell still
//!   draws, it just has no new palette to draw with. `ImageMagick` and `ffmpeg`
//!   read the wallpapers, `satty` annotates a capture, `grim`/`slurp`/`swappy`
//!   take it, `wl-clipboard` is the clipboard the panels use, `brightnessctl`
//!   the slider, `python3` the CLI, `qt6ct` the Qt theme bridge its palette
//!   drives, and the icon font its interface.
//! - **not installed**: nothing from upstream's `system/`. The SDDM theme, the
//!   two polkit actions, `/etc/pam.d/impasto-face` and the logind drop-in are
//!   login-screen and system policy, not a shell switch — and `./setup` is never
//!   run, so nothing writes outside `$HOME`. Face unlock (`howdy-next`) is not in
//!   the repository and `FaceService` stays quiet without it, as does the hyprpm
//!   plugin pair, which cannot be built per compositor from a noarch package.
//! - **config**: the shell writes its own — `~/.config/impasto` and a settings
//!   file under `~/.local/state/quickshell`, its profiles and camera samples in
//!   `~/.local/share/impasto`, its version
//!   record in `~/.local/state/impasto/version`, and its caches in
//!   `~/.cache/impasto`. The skel seeds that version file with this package's
//!   version and an empty repository, which is what keeps the System page from
//!   offering a `./setup update` that does not exist here.
//!
//! Its lock screen answers with the system `login` PAM service, which includes
//! `system-auth` — so `pam_faillock` applies. `docs/SAFETY.md` says what that
//! means and how to get back in.

use crate::shell::{Identity, InstallPlan, UninstallPlan};

pub const IDENTITY: Identity = Identity {
    id: "impasto",
    display_name: "Impasto",
    upstream: "andreumassanet/impasto",
    package: Some("impasto-shell"),
    binary: "impasto",
    start_argv: &["qs", "-c", "impasto"],
    process_match: &["-c", "impasto"],
    install_plan: Some(InstallPlan {
        package: "impasto-shell",
        stages: &[
            &["quickshell-git"],
            &[
                "impasto-shell",
                "awww",
                "satty",
                "ImageMagick",
                "ffmpeg",
                "grim",
                "slurp",
                "swappy",
                "wl-clipboard",
                "brightnessctl",
                "hypridle",
                "hyprlock",
                // Its night light drives `hyprsunset`, its animated wallpapers
                // play through `mpvpaper`, and `scripts/record.py` records with
                // `wl-screenrec` or `wf-recorder` — it has neither otherwise.
                "hyprsunset",
                "mpvpaper",
                "wf-recorder",
                "qt6ct",
                "python3",
                "material-symbols-fonts",
                "google-rubik-fonts",
            ],
        ],
        program: "pkexec",
        args: &["rum", "install", "-y", "--refresh"],
    }),
    uninstall_plan: Some(UninstallPlan {
        package: "impasto-shell",
        // The runtime and the tools, for the reason spelled out in DMS's plan:
        // `switch::uninstall` keeps whatever the incoming shell declares in
        // `runtime_packages`, so a shared runtime survives a switch to another
        // Quickshell shell and leaves with the last one that wanted it.
        stages: &[&[
            "impasto-shell",
            "quickshell-git",
            "awww",
            "satty",
            "ImageMagick",
            "ffmpeg",
            "grim",
            "slurp",
            "swappy",
            "wl-clipboard",
            "brightnessctl",
            "hypridle",
            "hyprlock",
            "hyprsunset",
            "mpvpaper",
            "wf-recorder",
            "qt6ct",
            "material-symbols-fonts",
            "google-rubik-fonts",
        ]],
        program: "pkexec",
        args: &["rum", "remove", "-y"],
    }),
    companion_systemd_user_units: &[],
    autostart_systemd_user_units: &[],
    // Profiles, the camera samples and the palette board. The shell writes them
    // and reads them back on the next start, so they are the user's. The skel
    // seeds only upstream's defaults — the palette board and its three stock
    // profiles — because `./setup`, which copies them out of
    // `/usr/share/impasto/share`, is deliberately never run; a backup of this
    // shell wins over the seed when the shell comes back.
    companion_paths: &[".local/share/impasto"],
    // The two GTK sheets `scripts/theme_manager.py` derives from the palette.
    // Nothing matugen-shaped: this shell writes them from ImageMagick and its own
    // state, so there is no shared sheet to hand over — but they are still files
    // this shell owns, and a switch away has to clean them unless the incoming
    // shell writes the same paths (SekaiPop does).
    generated_paths: &[".config/gtk-3.0/gtk.css", ".config/gtk-4.0/gtk.css"],
    // Compiled QML and the rendered thumbnails.
    discard_paths: &[".cache/impasto"],
    runtime_packages: &[
        "quickshell",
        "quickshell-git",
        "material-symbols-fonts",
        "google-rubik-fonts",
    ],
    // The shell registers itself as the session's polkit agent and starts the
    // KDE agent itself when it cannot. See `Identity::polkit_agent`.
    polkit_agent: &[],
    logo: Some("impasto.svg"),
};
