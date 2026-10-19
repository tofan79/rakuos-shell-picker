//! iNiR — a Quickshell desktop shell for Niri, packaged for RakuOS as
//! `inir-shell`.
//!
//! What is the same wherever it runs (today that is Niri only):
//!
//! - **install**: two privileged stages, in this order:
//!
//!   ```text
//!   1. quickshell-git
//!   2. inir-shell qt6-qt5compat qt6-qtmultimedia qt6-qtimageformats qt6ct
//!      darkly kf6-kservice cliphist wl-clipboard grim slurp wf-recorder
//!      ImageMagick tesseract ddcutil brightnessctl swayidle swaylock
//!      wlsunset golang material-symbols-fonts jetbrainsmono-nerd-fonts
//!   ```
//!
//!   `quickshell-git` is named explicitly rather than left to the solver, for
//!   the reason DMS's plan states: the repository also carries `noctalia-qs`,
//!   which provides `quickshell` as well, and that is the legacy Noctalia build.
//!   It is a stage of its own because that ordering is a safety property — if the
//!   runtime fails there is then nothing of iNiR on the machine at all.
//!
//!   Everything after `inir-shell` is a package nothing requires and the shell
//!   would be visibly broken or silently incomplete without:
//!
//!   - `qt6-qt5compat` and `qt6-qtmultimedia`: the QML imports behind the bar's
//!     blur and the video wallpaper. A missing QML module is a component that
//!     fails to load, so this is the first pair that has to be named.
//!   - `qt6-qtimageformats`: the Qt image plugins that make webp, avif and gif
//!     wallpapers load at all.
//!   - `qt6ct` and `darkly`: the platform theme and style the session's own
//!     `QT_QPA_PLATFORMTHEME=qt6ct` / `QT_STYLE_OVERRIDE=Darkly` name. The
//!     Niri skel sets both, and without the packages every Qt app falls back to
//!     a default look while the shell writes colour schemes for it to read.
//!   - `kf6-kservice`: `kbuildsycoca6`, which the skel's `spawn-at-startup` runs
//!     so that `XDG_MENU_PREFIX=plasma-` means something and the launchers and
//!     "Open With" menus are populated.
//!   - `cliphist` and `wl-clipboard`: clipboard history, which is the watcher the
//!     skel starts and the overlay `Mod+V` opens. `cliphist` alone is a binary
//!     with nothing feeding it; `wl-clipboard` is what watches.
//!   - `grim`, `slurp`: the region selector's capture. `wf-recorder`: screen
//!     recording, which the recorder scripts and the pill both drive.
//!   - `ImageMagick`: `/usr/bin/magick identify`, which the background reads to
//!     size every wallpaper before it shows it.
//!   - `tesseract`: OCR of a selected region. Its language models are the one
//!     thing here that is not a package: `scripts/ocr-runner.sh` symlinks the
//!     distro's data and downloads what is missing into
//!     `~/.local/share/inir/tessdata`, so the English model inside `tesseract`
//!     is enough to start with.
//!   - `ddcutil` and `brightnessctl`: brightness on an external monitor and on
//!     the panel, both of which `Brightness.qml` drives directly.
//!   - `swayidle`: the idle service does not reimplement idle handling, it runs
//!     this. `swaylock`: the fallback the lock screen falls back to when the
//!     shell's own session lock cannot take the seat — a session that cannot be
//!     locked is worth a second path. `wlsunset`: the night-light toggle on
//!     Niri, spawned by path.
//!   - `golang`: `go build`. Three theming modules compile their own generator
//!     from the payload at the moment a theme is applied — editor themes, the
//!     system24 palette, washi — instead of shipping a binary per architecture.
//!   - `material-symbols-fonts` and `jetbrainsmono-nerd-fonts`: the icon font the
//!     whole interface is drawn with and the monospace font in its config.
//!     Nothing requires them at the package level, so the shell would start with
//!     blank glyphs.
//!
//!   Deliberately **not** named: `matugen` (iNiR generates its own palettes; the
//!   tool only appears in its distribution-specific extras), `khal` (that is
//!   DMS's calendar), `kdialog`, `swappy`, `gum` and `playerctl` (the shell owns
//!   the annotation editor, the menu and MPRIS, and the file chooser is the
//!   session's portal). This is the list of what would be missing, not a copy of
//!   upstream's installer, which also installs things only its own setup flow
//!   reads.
//!
//! - **uninstall**: one stage, `pkexec rum remove -y` over every package the
//!   install plan put there, and nothing else. It runs at the very end of a
//!   switch away, once the new shell has proved itself alive, and leaves whatever
//!   the incoming shell declares in its own `runtime_packages` — see
//!   `switch::uninstall`. That is why iNiR names the runtime, its two fonts and
//!   `wf-recorder` there: DMS, Caelestia, Ambxst and end4-pC all remove
//!   `quickshell-git` on their way out, and the last two also remove
//!   `material-symbols-fonts` and `wf-recorder`, all of which iNiR runs on.
//!
//! - **binary**: `inir` is the package's own launcher. It is a shell script, so
//!   no live process is ever named `inir` — see `process_match` — and a `which`
//!   hit is still the right test for "this shell is on the machine".
//!   `inir run --session` is the argv the package's own unit runs, and is what a
//!   switch uses on a compositor with no unit of its own to start.
//!
//! - **probe**: the launcher `exec`s the shared runtime, so the process that
//!   matters is `qs -n -p <config dir>`. `process_match` names the argument pair
//!   of the **system** payload, `/usr/share/quickshell/inir`, because that is
//!   what a switch installs and therefore what the process it starts looks like.
//!   A user who copies the payload to `~/.config/quickshell/inir` gets a shell
//!   this picker does not recognise as running — the same copy that already wins
//!   over the package's payload at launch, and which no switch here manages.
//!
//! - **config**: `~/.config/illogical-impulse`. That name is not a mistake and
//!   not legacy here: the shell's `Directories.shellConfig` is
//!   `${config}/illogical-impulse`, it writes `config.json`, `themes/`,
//!   `plugins/` and `ai/` there, and it creates the directory on startup. The
//!   package's own scripts call `~/.config/inir` the canonical path and migrate
//!   the old one onto it; nothing runs that migration on a machine the picker
//!   set up, so the directory the running shell actually uses is the one a
//!   switch has to carry. `~/.config/inir` is still iNiR's, though — custom
//!   widgets live in `~/.config/inir/widgets` — which is why it is declared
//!   under `companion_paths` rather than left out.
//!
//! - **state**: `~/.local/state/inir`, where the recorder status and the warp
//!   provider live. The shell's *interface* state is somewhere else entirely:
//!   `~/.local/state/quickshell/user` (`events.json`, `todo.json`, `notes.txt`,
//!   `screentime/`, `generated/`, the AI chats) and
//!   `~/.local/state/quickshell/states.json`, under Quickshell's own application
//!   state rather than iNiR's. Those are declared as `companion_paths` and not as
//!   `state_dir` for a reason that matters: the Python venv sits beside them in
//!   the same directory, and a `state_dir` of `~/.local/state/quickshell` would
//!   put a whole virtualenv into every backup a switch takes.
//!
//! - **generated**: the six entry points iNiR renders from the wallpaper's
//!   palette — GTK's `gtk.css` and `settings.ini` for GTK 3 and GTK 4, and the
//!   `qt5ct`/`qt6ct` configuration. Every one of them lands inside a directory
//!   `switch::SHARED_PATHS` already walks, so leaving iNiR takes its colours away
//!   and needs no `backup_extra_paths` entry to do it. `kdeglobals` is rendered
//!   by the same script and is deliberately left out: it is a file directly under
//!   `~/.config`, which nothing in the registry covers, and no other shell here
//!   touches it either.
//!
//! - **discard**: the caches and the regenerated files. `~/.cache/quickshell` is
//!   the compiled QML and the media caches the runtime rebuilds on the next
//!   start; `~/.cache/inir` is window previews and downloaded dictionaries;
//!   `~/.local/share/inir/tessdata` is the OCR models the OCR runner downloads
//!   again when it needs them; `~/.local/state/quickshell/.venv` is the Python
//!   environment, tens of megabytes that upstream's own installer builds from
//!   nothing; and `~/.local/share/color-schemes/Darkly.colors` is the Qt style
//!   colour scheme iNiR renders, which `qt6ct` reads and which is rebuilt the
//!   next time a colour is applied. None of them holds anything a person wrote,
//!   and none of them sits inside a path a backup walks — which is the test that
//!   keeps this list honest.
//!
//!   The venv is worth stating plainly as a limit: this picker does not build it.
//!   A switch to iNiR gives a machine that runs the shell, and the parts of iNiR
//!   that go through the venv — colour generation, translations, the AI features
//!   — stay degraded until the environment is built by hand or by upstream's own
//!   setup.
//!
//! - **polkit**: iNiR answers polkit from its own agent, like DMS and the
//!   `illogical-impulse` fork it grew from, so no external agent is started here.
//!
//! - **logo**: `inir.svg`, the project's mascot.
//!
//! What differs per compositor is the [`Port`](crate::shell::Port); today that is
//! Niri and nothing else, including how its unit gets enabled.

use crate::shell::{Identity, InstallPlan, UninstallPlan};

/// The packages a switch installs, in the order the stages run and in the order
/// `rum` is handed them: the shell, the QML and Qt pieces its interface needs,
/// the session tools, the theming toolchain, and the two fonts last.
///
/// One list rather than two, because the uninstall plan is exactly this list —
/// see the module comment for why that is safe.
const PACKAGES: &[&str] = &[
    "inir-shell",
    "qt6-qt5compat",
    "qt6-qtmultimedia",
    "qt6-qtimageformats",
    "qt6ct",
    "darkly",
    "kf6-kservice",
    "cliphist",
    "wl-clipboard",
    "grim",
    "slurp",
    "wf-recorder",
    "ImageMagick",
    "tesseract",
    "ddcutil",
    "brightnessctl",
    "swayidle",
    "swaylock",
    "wlsunset",
    "golang",
    "material-symbols-fonts",
    "jetbrainsmono-nerd-fonts",
];

pub const IDENTITY: Identity = Identity {
    id: "inir",
    display_name: "iNiR",
    upstream: "snowarch/iNiR",
    package: Some("inir-shell"),
    binary: "inir",
    start_argv: &["inir", "run", "--session"],
    process_match: &["-p", "/usr/share/quickshell/inir"],
    install_plan: Some(InstallPlan {
        package: "inir-shell",
        stages: &[&["quickshell-git"], PACKAGES],
        program: "pkexec",
        // `--refresh` because a cached copy of the metadata is only known to be
        // young, not known to be current. See `switch::install`.
        args: &["rum", "install", "-y", "--refresh"],
    }),
    uninstall_plan: Some(UninstallPlan {
        package: "inir-shell",
        // Everything iNiR installed behind it, the runtime included. `rum remove`
        // also refuses any package another package still requires, and
        // `switch::uninstall` drops whatever the incoming shell declares in its
        // own `runtime_packages`, so the solver and the guard between them cover
        // the two ways this list could reach too far.
        stages: &[PACKAGES],
        program: "pkexec",
        args: &["rum", "remove", "-y"],
    }),
    // The agent Supaste and the notification centre talk to is iNiR's own, so
    // there is no companion unit to start beside it.
    companion_systemd_user_units: &[],
    // Nothing autostarts with iNiR: the clipboard watchers and `kbuildsycoca6`
    // are `spawn-at-startup` lines in the compositor's skel, which the compositor
    // itself runs at login and a switch does not need to re-run.
    autostart_systemd_user_units: &[],
    // Data that is iNiR's and that nothing would install: the custom widgets the
    // user drops into `~/.config/inir/widgets`, and the state the shell keeps
    // under Quickshell's own application directory — the events it recorded, the
    // todo lists and notes the user wrote, the AI chats, the generated material
    // palettes, and the persistent states that hold what the sidebar had pinned.
    // None of it is a configuration a switch hands out, and all of it is somebody's
    // data, so it is backed up with iNiR and read back the next time iNiR is the
    // shell.
    companion_paths: &[
        ".config/inir",
        ".local/state/quickshell/user",
        ".local/state/quickshell/states.json",
    ],
    // See the module comment: the theme entry points `apply-gtk-theme.sh` renders
    // from the wallpaper's palette. They live in the four session-wide
    // directories `switch::SHARED_PATHS` walks, so the backup that makes their
    // removal safe is the one already taken for every switch.
    generated_paths: &[
        ".config/gtk-3.0/gtk.css",
        ".config/gtk-4.0/gtk.css",
        ".config/gtk-3.0/settings.ini",
        ".config/gtk-4.0/settings.ini",
        ".config/qt5ct/qt5ct.conf",
        ".config/qt6ct/qt6ct.conf",
    ],
    // Regenerated output and nothing else. Each entry holds no data, none of it
    // is walked by the backup, and `every_discard_path_is_never_preserved` fails
    // the suite if that ever stops being true.
    discard_paths: &[
        ".cache/inir",
        ".cache/quickshell",
        ".local/share/color-schemes/Darkly.colors",
        ".local/share/inir/tessdata",
        ".local/state/quickshell/.venv",
    ],
    // What iNiR runs on rather than what it is: the shared Quickshell runtime,
    // the two fonts the whole interface is drawn with, and the recorder behind
    // its screen-record controls. Named so a switch *to* iNiR keeps them —
    // Caelestia's and end4-pC's removal plans name all four, and DMS's and
    // Ambxst's name the runtime.
    runtime_packages: &[
        "quickshell",
        "quickshell-git",
        "material-symbols-fonts",
        "jetbrainsmono-nerd-fonts",
        "wf-recorder",
    ],
    // iNiR is its own polkit agent. See `Identity::polkit_agent`.
    polkit_agent: &[],
    logo: Some("inir.svg"),
};
