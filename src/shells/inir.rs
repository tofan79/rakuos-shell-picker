//! iNiR — a Quickshell desktop shell for Niri, packaged for RakuOS as
//! `inir-shell`.
//!
//! What is the same wherever it runs (today that is Niri only):
//!
//! - **install**: one privileged stage:
//!
//!   ```text
//!   inir-shell qt6-qt5compat qt6-qtmultimedia qt6-qtimageformats darkly
//!   kf6-kservice cliphist grim slurp wf-recorder ImageMagick tesseract
//!   brightnessctl swayidle wlsunset golang material-symbols-fonts
//!   ```
//!
//!   Unlike the other quickshell shells here, there is no stage of its own for
//!   the runtime: the RakuOS niri image installs `quickshell-git` *and* lists it
//!   in `protected-packages.txt`, so it cannot be missing and cannot be removed.
//!   Naming it would be stating what the image already guarantees. The niri
//!   build files are the reference for the rest of the list too — what the image
//!   carries is not repeated here, because the same list is the uninstall plan
//!   and a package the base image owns is one a switch neither installed nor may
//!   take away. So `qt6ct` (the image installs `qt6ct-kde`, which provides it),
//!   `wl-clipboard`, `ddcutil` and `matugen` are left out, and `swaylock` is
//!   left out for the opposite reason: the image removes it on purpose.
//!   `runtime_packages` still names the runtime — that is a different statement,
//!   about what a switch *to* iNiR must not let another shell's removal take.
//!
//!   Everything listed is a package nothing requires and the shell would be
//!   visibly broken or silently incomplete without:
//!
//!   - `qt6-qt5compat` and `qt6-qtmultimedia`: the QML imports behind the bar's
//!     blur and the video wallpaper. A missing QML module is a component that
//!     fails to load, so this is the first pair that has to be named. Neither is
//!     in the niri image's own list, and only the first is in the Hyprland
//!     image's.
//!   - `qt6-qtimageformats`: the Qt image plugins that make webp, avif and gif
//!     wallpapers load at all.
//!   - `darkly`: the Qt style the skel's `QT_STYLE_OVERRIDE` names, which iNiR
//!     points at the colour scheme it renders. (The platform theme beside it,
//!     `qt6ct`, is the image's.)
//!   - `kf6-kservice`: `kbuildsycoca6`, which the skel's `spawn-at-startup` runs
//!     so that `XDG_MENU_PREFIX=plasma-` means something and the launchers and
//!     "Open With" menus are populated.
//!   - `cliphist`: clipboard history, which the overlay `Mod+V` shows and the
//!     image's `spawn-at-startup "wl-paste --watch cliphist store"` fills. The
//!     watcher is the image's and so is `wl-clipboard` behind it, but `cliphist`
//!     itself is only in the Hyprland image; the Niri image has neither the
//!     store nor anything that writes to it. The skel starts one watcher of its
//!     own, for images, and explains there why the text one is off.
//!   - `grim`, `slurp`: the region selector's capture. `wf-recorder`: screen
//!     recording, which the recorder scripts and the pill both drive. All three
//!     are Hyprland-image packages; none is in the Niri image.
//!   - `ImageMagick`: `/usr/bin/magick identify`, which the background reads to
//!     size every wallpaper before it shows it. The running image has it as a
//!     dependency of the base image's `braille-printer-app`, which is not a
//!     promise the niri build files make.
//!   - `tesseract`: OCR of a selected region. Its language models are the one
//!     thing here that is not a package: `scripts/ocr-runner.sh` symlinks the
//!     distro's data and downloads what is missing into
//!     `~/.local/share/inir/tessdata`, so the English model inside `tesseract`
//!     is enough to start with.
//!   - `brightnessctl`: brightness on the panel, which `Brightness.qml` drives
//!     directly. (`ddcutil` beside it, for external monitors, is the image's.)
//!   - `swayidle`: the idle service does not reimplement idle handling, it runs
//!     this. `wlsunset`: the night-light toggle on Niri, spawned by path.
//!   - `golang`: `go build`. Three theming modules compile their own generator
//!     from the payload at the moment a theme is applied — editor themes, the
//!     system24 palette, washi — instead of shipping a binary per architecture.
//!   - `material-symbols-fonts`: the icon font the whole interface is drawn
//!     with. Nothing requires it at the package level, and an icon font has no
//!     fallback worth the name — its glyphs are private-use codepoints, so a
//!     missing family is a grid of empty boxes rather than a different
//!     typeface. It is in neither image, and the only copy on this machine is an
//!     unowned file an earlier setup left in `~/.local/share/fonts`, which a
//!     fresh install will not have.
//!
//!     The mono font in iNiR's config is deliberately **not** named:
//!     `JetBrainsMono Nerd Font` is in no repository RakuOS enables, and the
//!     fallback is the family the image already sets as monospace, Maple Mono NF
//!     — a Nerd Font as well, so the glyphs the interface asks it for are there.
//!     `mainFont`/`titleFont` (`Roboto Flex`, `Gabarito`) are missing for the
//!     same reason and resolve to Inter, the system sans. All three names live in
//!     the package's own `defaults/config.json`, which a switch does not write.
//!
//!   Deliberately **not** named, and not because of the image: `khal` (that is
//!   DMS's calendar), `kdialog`, `swappy`, `gum` and `playerctl` (the shell owns
//!   the annotation editor and the menu, its MPRIS is native Quickshell, and the
//!   file chooser belongs to the session's portal). This is the list of what
//!   would be missing, not a copy of upstream's installer, which also installs
//!   things only its own setup flow reads.
//!
//! - **uninstall**: one stage, `pkexec rum remove -y` over every package the
//!   install plan put there, and nothing else. It runs at the very end of a
//!   switch away, once the new shell has proved itself alive, and leaves whatever
//!   the incoming shell declares in its own `runtime_packages` — see
//!   `switch::uninstall`. That is why iNiR names the runtime, its icon font and
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

/// The packages a switch installs, in the order `rum` is handed them: the shell,
/// the QML and Qt pieces its interface needs, the session tools, the theming
/// toolchain, and the icon font last.
///
/// Only what the image does not already bring, because the same list is the
/// uninstall plan: naming a package the base image owns buys nothing (the
/// install is a no-op and `switch::uninstall` drops base-image packages before
/// the package manager sees them) and says the wrong thing — that a switch put
/// it there.
///
/// Left out because the RakuOS niri image already carries them, with the
/// `rakuos-niri` build files as the reference:
///
/// - `quickshell-git` — installed *and* listed in `protected-packages.txt`, so
///   it cannot be taken off the machine at all. The other quickshell shells
///   here name it in a stage of its own; iNiR, which is only offered on niri,
///   would be stating a fact the image already guarantees. It is still in
///   [`Identity::runtime_packages`], which is a different statement: what a
///   switch *to* iNiR must not let another shell's removal take away.
/// - `qt6ct` — the image installs `qt6ct-kde`, which provides it.
/// - `wl-clipboard`, `ddcutil` — in the image's own install list.
/// - `matugen` — the image has it, and iNiR does not call it: it makes its own
///   palettes.
/// - `swaylock` — the image takes it off on purpose. iNiR has its own session
///   lock and only reaches for `swaylock` as a fallback; putting it back would
///   be arguing with the image about which shell owns locking.
///
/// Left out for a different reason — no repository RakuOS enables carries it:
///
/// - `jetbrainsmono-nerd-fonts` — the monospace family iNiR's config names. The
///   family the image sets as monospace, Maple Mono NF, is a Nerd Font too, so
///   the fallback already has the glyphs the interface asks for, and the package
///   would only add a second copy of a font nobody picked.
///
/// `ImageMagick` and `qt6-qtmultimedia` stay in the list even though the
/// running image happens to have them: the first arrives as a dependency of the
/// base image's `braille-printer-app` and the second is required by nothing at
/// all, so neither is something the niri build files promise. iNiR reads
/// `/usr/bin/magick` directly to size a wallpaper and imports `QtMultimedia`
/// for the video wallpaper, and a shell that installs cleanly on a machine
/// where an incidental dependency changed is worth two entries that are
/// usually no-ops.
const PACKAGES: &[&str] = &[
    "inir-shell",
    "qt6-qt5compat",
    "qt6-qtmultimedia",
    "qt6-qtimageformats",
    "darkly",
    "kf6-kservice",
    "cliphist",
    "grim",
    "slurp",
    "wf-recorder",
    "ImageMagick",
    "tesseract",
    "brightnessctl",
    "swayidle",
    "wlsunset",
    "golang",
    "material-symbols-fonts",
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
        // One stage: see the module comment for why the runtime is not a stage
        // of its own here as it is for the other quickshell shells.
        stages: &[PACKAGES],
        program: "pkexec",
        // `--refresh` because a cached copy of the metadata is only known to be
        // young, not known to be current. See `switch::install`.
        args: &["rum", "install", "-y", "--refresh"],
    }),
    uninstall_plan: Some(UninstallPlan {
        package: "inir-shell",
        // Exactly what the install plan put there. `rum remove` also refuses any
        // package another package still requires, and `switch::uninstall` drops
        // whatever the incoming shell declares in its own `runtime_packages`, so
        // the solver and the guard between them cover the two ways this list
        // could reach too far.
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
    // the icon font the whole interface is drawn with, and the recorder behind
    // its screen-record controls. Named so a switch *to* iNiR keeps them —
    // Caelestia's and end4-pC's removal plans name all three, and DMS's and
    // Ambxst's name the runtime. The mono font is not here: iNiR does not
    // install it and the system's own monospace stands in for it.
    runtime_packages: &[
        "quickshell",
        "quickshell-git",
        "material-symbols-fonts",
        "wf-recorder",
    ],
    // iNiR is its own polkit agent. See `Identity::polkit_agent`.
    polkit_agent: &[],
    logo: Some("inir.svg"),
};
