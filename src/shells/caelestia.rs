//! Caelestia Shell — a Quickshell shell published by the same project as its
//! Hyprland dots, and packaged for RakuOS as `caelestia-shell`.
//!
//! What is the same wherever it runs:
//!
//! - **install**: two privileged stages, in this order:
//!
//!   ```text
//!   1. quickshell-git
//!   2. caelestia-shell google-rubik-fonts cascadia-code-nf-fonts material-symbols-fonts mate-polkit
//!   ```
//!
//!   `quickshell-git` is the runtime, named first for the reason DMS names it
//!   first: it is the piece most likely to fail on its own, and if it does there
//!   is then nothing of Caelestia on the machine. Caelestia runs on the same
//!   shared `quickshell` as DMS and Ambxst, which is why it declares that in
//!   [`Self::runtime_packages`] rather than owning it.
//!
//!   `caelestia-cli` is not named: `caelestia-shell` requires it, so the
//!   dependency solver brings it in. Naming it explicitly would ask `rum` for a
//!   package whose repository also publishes a source RPM named the same, which
//!   is the trap the `.noarch` spelling exists to dodge — better to let the
//!   requirement pick the binary subpackage.
//!
//!   The three fonts are what makes the shell look right rather than merely
//!   run: the bar and launcher are drawn with Material Symbols Rounded, and the
//!   icons and interface text come from Rubik and Cascadia Code Nerd Font.
//!   Nothing requires them at the package level, so they are named here or the
//!   shell starts with blank glyphs.
//!
//! - **uninstall**: one stage, `pkexec rum remove -y`, naming
//!   `caelestia-shell`, `quickshell-git`, the three fonts and `mate-polkit` —
//!   everything Caelestia's install brought in, the runtime included, and for
//!   DMS's reason: a removal plan is the declaration of what this shell put on
//!   the machine, and *which* of it may actually go is decided at the moment of
//!   the switch. `switch::uninstall` drops any package the shell being switched
//!   **to** declares in [`Self::runtime_packages`]: DMS and Ambxst both declare
//!   `quickshell-git`, so leaving for either keeps the runtime, and Noctalia
//!   declares nothing, so leaving for it takes the whole of Caelestia — never an
//!   orphan left for `rum autoremove`. The fonts are in the plan too: they were
//!   installed for this shell's look, and the guard keeps any the target still
//!   declares.
//!
//!   After the fonts, `mate-polkit` is the other package that leaves with the
//!   shell: Caelestia's config starts the GTK polkit agent from
//!   `hypr/hyprland/execs.lua`. The agent is the package-level dependency that
//!   makes that `exec_cmd` able to run, so it is installed with the shell and
//!   removed when a switch leaves it behind.
//!
//! - **start**: the compositor's start event. Caelestia's own Hyprland config
//!   runs `caelestia shell -d` from `hypr/hyprland/execs.lua`, and the live
//!   switch runs the same command directly. Both end as `qs -c caelestia …`,
//!   which is why [`Self::process_match`] is set: the process is the shared
//!   runtime, not a binary called `caelestia`.
//!
//! - **no companions**: it has none, so there is no companion data to save.
//!
//! The config directory is `~/.config/caelestia` and the state directory is
//! `~/.local/state/caelestia`; both belong to the [`Port`]. The QML itself lives
//! in the package under `/etc/xdg/quickshell/caelestia` and is not user data.

use crate::shell::{Identity, InstallPlan, UninstallPlan};

pub const IDENTITY: Identity = Identity {
    id: "caelestia",
    display_name: "Caelestia Shell",
    upstream: "caelestia-dots/shell",
    package: Some("caelestia-shell"),
    binary: "caelestia",
    start_argv: &["caelestia", "shell", "-d"],
    // `caelestia shell -d` execs the shared `quickshell` runtime, so the live
    // process is `qs -c caelestia …` and its `comm` is `qs`. These two
    // arguments, in order, are what tell that process apart from DMS's or a
    // legacy `noctalia-qs` one on the same runtime.
    process_match: &["-c", "caelestia"],
    install_plan: Some(InstallPlan {
        package: "caelestia-shell",
        stages: &[
            // The runtime first and on its own, mirroring DMS: if it fails there
            // is then nothing of Caelestia on the machine.
            &["quickshell-git"],
            &[
                "caelestia-shell",
                "google-rubik-fonts",
                "cascadia-code-nf-fonts",
                "material-symbols-fonts",
                "mate-polkit",
            ],
        ],
        program: "pkexec",
        args: &["rum", "install", "-y", "--refresh"],
    }),
    uninstall_plan: Some(UninstallPlan {
        package: "caelestia-shell",
        // Everything the install put there, the runtime included. Mirroring
        // DMS's phase-three note: `quickshell-git` is a shared runtime and the
        // switch itself is where the guard lives — `switch::uninstall` drops any
        // package the shell being switched to declares in `runtime_packages`,
        // so DMS and Ambxst keep it while Noctalia does not and the whole of
        // Caelestia goes. Nothing else on the machine asks for the runtime
        // once `caelestia-shell` is in the same transaction, so leaving it out
        // only ever produced an orphan that `rum autoremove` was left to find.
        // The fonts were installed for this shell's look, so they go with it;
        // `mate-polkit` is the agent its `execs.lua` starts. The switch's guard
        // keeps whichever of them the target shell still declares.
        stages: &[&[
            "caelestia-shell",
            "quickshell-git",
            "mate-polkit",
            "google-rubik-fonts",
            "cascadia-code-nf-fonts",
            "material-symbols-fonts",
        ]],
        program: "pkexec",
        args: &["rum", "remove", "-y"],
    }),
    companion_systemd_user_units: &[],
    // No companions, so nothing of theirs to save or bring back.
    companion_paths: &[],
    // Caelestia renders these four into `gtk-3.0` and `gtk-4.0` on every start;
    // `gtk.css` is the entry point other GTK programs read and `thunar.css` is
    // what the shell writes for the file manager. Both directories are in
    // `switch::SHARED_PATHS`, so each file is backed up before `clean` may take
    // it away. Caelestia's other renders land in program-owned directories that
    // no backup walks; they are in `discard_paths` instead.
    generated_paths: &[
        ".config/gtk-3.0/gtk.css",
        ".config/gtk-4.0/gtk.css",
        ".config/gtk-3.0/thunar.css",
        ".config/gtk-4.0/thunar.css",
    ],
    // Regenerated output and cached state, none of it written by the user.
    //
    // Everything here is a render Caelestia overwrites on the next start, or a
    // cache it can rebuild, and every directory it lives in belongs to another
    // program rather than to the session — so none of it is walked by a backup,
    // which is the property `every_discard_path_is_never_preserved` checks.
    discard_paths: &[
        ".cache/caelestia",
        ".config/btop/themes/caelestia.theme",
        ".config/cava/config",
        ".config/fuzzel/fuzzel.ini",
        ".config/nvtop/nvtop.colors",
        ".config/qtengine/caelestia.colors",
        ".config/spicetify/Themes/caelestia",
        ".config/zed/themes/caelestia.json",
        ".local/share/PandoraLauncher/themes/caelestia.json",
        ".local/share/warp-terminal/themes/caelestia.yaml",
    ],
    // The runtime it is built on, shared with DMS and Ambxst, and the polkit
    // agent its own config starts, shared with Ambxst. Listed so a switch to
    // Caelestia or Ambxst keeps them; leaving for a shell that needs neither
    // takes them away.
    runtime_packages: &["quickshell", "quickshell-git", "mate-polkit"],
    // Caelestia's config starts this from `execs.lua`; a live switch has to start
    // it itself. See `Identity::polkit_agent`.
    polkit_agent: &["/usr/libexec/polkit-mate-authentication-agent-1"],
    logo: Some("caelestia.svg"),
};
