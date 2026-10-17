//! end4-pC — the `pctrade` fork of end-4's illogical-impulse Quickshell
//! configuration, packaged for RakuOS as `end4-pc`.
//!
//! What is the same wherever it runs (today that is Hyprland only):
//!
//! - **package**: `end4-pc`, from the RakuOS repository. It ships the QML only,
//!   to `/usr/share/quickshell/end4-pC` — the config name `end4-pC` with the
//!   package's lowercase name folded in. The compositor configuration that
//!   starts it, and the symlink in `~/.config/quickshell/end4-pC` that makes
//!   `qs -c end4-pC` find it, comes from the skel (see the Hyprland port).
//! - **binary**: `end4-pC` is the quickshell config identity, not a real
//!   program — no launcher ships with the package, and nothing named this is
//!   meant to appear in `PATH`. That is deliberate: the live process is the
//!   shared `qs`, and if `binary` were `qs` then `installed()` would pass any
//!   machine with Quickshell on it off as this shell. With a name that resolves
//!   nowhere, installedness stays driven by the package origin.
//! - **probe**: `qs -c end4-pC`, the command upstream's README starts it with.
//!   Those two argv words tell it apart from the sibling Quickshell shells on
//!   the same runtime; the base `ii` runs as `qs -c ii`.
//! - **logo**: `end4pc.svg`, the illogical-impulse mark.
//! - **install**: two privileged stages, the runtime first for the reason the
//!   other Quickshell shells name it first:
//!
//!   ```text
//!   1. quickshell-git
//!   2. end4-pc illogical-impulse matugen hypridle hyprlock
//!      google-rubik-fonts cascadia-code-nf-fonts material-symbols-fonts
//!      easyeffects hyprsunset wf-recorder swappy fuzzel uv dbus-devel
//!   ```
//!
//!   `illogical-impulse` is the base `ii` this fork grew from. `end4-pc`
//!   requires it, so the plan naming it changes nothing at install time; what
//!   it buys is the symmetric uninstall — leaving end4-pC takes the base back
//!   out too. `matugen` is the theming tool the colour scripts drive.
//!   `hypridle` and `hyprlock` are the lock screen the skel wires up:
//!   `hyprland/execs.lua` starts `hypridle` at login, and its config answers
//!   `loginctl lock-session` — the lock button, `SUPER + L`, before sleep — by
//!   asking the shell to raise its own lock screen (`quickshell:lock`), and
//!   falls back to `hyprlock` when the shell is not running. Without either
//!   package a lock request goes nowhere. `easyeffects` is the audio effects
//!   daemon `execs.lua` starts for the shell's equalizer, `hyprsunset` the
//!   night light its toggles spawn, `wf-recorder` the recorder behind the
//!   region-capture keybinds, `swappy` the annotate step of a screenshot, and
//!   `fuzzel` the fallback menu those keybinds use when the shell is not
//!   around. The three fonts are what the interface is drawn with. `uv`
//!   bootstraps the Python venv the colour and thumbnail helpers run in, and
//!   `dbus-devel` serves that bootstrap alone: the venv's `dbus-python` has no
//!   wheel and compiles against the `dbus-1` headers. No polkit agent is
//!   installed: the fork answers polkit from its own `PolkitService`, which
//!   registers with polkitd when the shell starts — see `polkit_agent` below.
//!
//! The config directory is `~/.config/quickshell/end4-pC` — the symlink into
//! the package, and the one path a switch must write for the shell to start at
//! all. The state directory is deliberately not declared as one, because the
//! venv lives inside it: what this shell keeps under
//! `~/.local/state/quickshell` is split instead. `user/` — the todos, the
//! notes, the AI chats, the equalizer presets — and the shell's own
//! `states.json` travel as companion data, the same road
//! `~/.config/illogical-impulse` takes: backed up on the way out, taken off
//! the machine when that backup reproduces them, read back by `plan` the next
//! time end4-pC is the shell. The venv and its bootstrap log are the two
//! paths a backup must not carry — hundreds of megabytes, rebuilt by
//! `end4pc-venv-setup.sh` in minutes — so they sit in `discard_paths` and a
//! switch away deletes them outright. The directory itself is never named by
//! any list: other programs keep their own data in the same runtime directory
//! (Ryoku's `stage/`, for one), so no path may take it wholesale.

use crate::shell::{Identity, InstallPlan, UninstallPlan};

pub const IDENTITY: Identity = Identity {
    id: "end4pc",
    display_name: "end4-pC",
    upstream: "pctrade/end4-pC",
    package: Some("end4-pc"),
    binary: "end4-pC",
    start_argv: &["qs", "-c", "end4-pC"],
    process_match: &["-c", "end4-pC"],
    install_plan: Some(InstallPlan {
        package: "end4-pc",
        stages: &[
            // The runtime first and on its own, mirroring DMS: if it fails there
            // is then nothing of end4-pC on the machine.
            &["quickshell-git"],
            &[
                "end4-pc",
                "illogical-impulse",
                "matugen",
                "hypridle",
                "hyprlock",
                "google-rubik-fonts",
                "cascadia-code-nf-fonts",
                "material-symbols-fonts",
                "easyeffects",
                "hyprsunset",
                "wf-recorder",
                "swappy",
                "fuzzel",
                "uv",
                "dbus-devel",
            ],
        ],
        program: "pkexec",
        args: &["rum", "install", "-y", "--refresh"],
    }),
    uninstall_plan: Some(UninstallPlan {
        package: "end4-pc",
        // Everything the install put there, the shared runtime included, for the
        // reason Caelestia's plan spells out: `switch::uninstall` drops any
        // package the shell being switched to declares in `runtime_packages`,
        // so the guard keeps `quickshell-git` and `matugen` for a target that
        // declares them too, and leaving for Noctalia takes the whole of
        // end4-pC away.
        stages: &[&[
            "end4-pc",
            "illogical-impulse",
            "quickshell-git",
            "matugen",
            "hypridle",
            "hyprlock",
            "google-rubik-fonts",
            "cascadia-code-nf-fonts",
            "material-symbols-fonts",
            "easyeffects",
            "hyprsunset",
            "wf-recorder",
            "swappy",
            "fuzzel",
            "uv",
            "dbus-devel",
        ]],
        program: "pkexec",
        args: &["rum", "remove", "-y"],
    }),
    // No daemon of its own, so nothing to keep in step with systemd.
    companion_systemd_user_units: &[],
    autostart_systemd_user_units: &[],
    // Three places hold what this shell would lose if a switch simply ignored
    // them. `config_dir` is the symlink Quickshell resolves rather than a
    // settings store, so what a person changes from the shell itself —
    // `config.json`, the saved actions, the presets — lives in
    // `~/.config/illogical-impulse`, which the shell writes on its first
    // start. Under `~/.local/state/quickshell`, `user/` keeps what a person
    // wrote — todos, notes, AI chats, equalizer presets — and `states.json`
    // the shell's own bookkeeping, the overlay widgets' placement and the AI
    // settings among it. All three are declared rather than left alone: backed
    // up with the shell, removed on the way out only when that backup
    // reproduces them byte for byte, and read back by `plan` the next time
    // end4-pC is the shell.
    companion_paths: &[
        ".config/illogical-impulse",
        ".local/state/quickshell/states.json",
        ".local/state/quickshell/user",
    ],
    // Matugen renders `gtk.css` into both shared GTK directories on every
    // colour change. Both directories are in `switch::SHARED_PATHS`, so each
    // file is backed up before `clean` may take it away. `~/.config/matugen` —
    // the configuration and templates the switch installs — goes the same way,
    // which is what stops a switch away from leaving the theming tool's
    // furniture behind; `backup_extra_paths` already walks it, so the
    // byte-for-byte test holds here too.
    generated_paths: &[
        ".config/gtk-3.0/gtk.css",
        ".config/gtk-4.0/gtk.css",
        ".config/matugen",
    ],
    // Regenerated output and cached state, none of it written by the user.
    //
    // `fuzzel_theme.ini` is a matugen render into another program's directory.
    // The two `IllogicalNamed` KDE schemes are always written together by the
    // named-scheme script and applied by `switchwall.sh`; the four
    // `MaterialYou` names and their `2` copies are what
    // `kde-material-you-colors` writes when the matugen template's wrapper
    // drives it — DMS discards its own `DankMatugen` schemes on the same
    // terms. `.config/easyeffects` is the audio tool's own state (the devices
    // it last picked, its window size), rewritten on its next start; the
    // presets under `~/.local/share/easyeffects` are data and stay.
    // `~/.cache/quickshell` is the runtime's compiled-QML cache, which any
    // Quickshell shell rebuilds, and `~/.cache/matugen` and `~/.cache/uv` are
    // the theming tool's and the venv bootstrap's caches. The venv itself and
    // the log of the bootstrap that built it are the two paths under
    // `~/.local/state/quickshell` that no backup should carry: hundreds of
    // megabytes rebuilt by the script on the next login, in a couple of
    // minutes, holding nothing a person wrote — so they are deleted where they
    // stand, and nothing else in that directory is (see the note at the top of
    // this module). The managed Python under `~/.local/share/uv` is
    // deliberately absent: it is a general tool's data directory, not this
    // shell's to take, and the next bootstrap reuses the interpreter it holds.
    // None of these live where a backup walks.
    discard_paths: &[
        ".cache/matugen",
        ".cache/quickshell",
        ".cache/uv",
        ".config/easyeffects",
        ".config/fuzzel/fuzzel_theme.ini",
        ".local/share/color-schemes/IllogicalNamed.colors",
        ".local/share/color-schemes/IllogicalNamed2.colors",
        ".local/share/color-schemes/MaterialYouDark.colors",
        ".local/share/color-schemes/MaterialYouDark2.colors",
        ".local/share/color-schemes/MaterialYouDark_darker_titlebar.colors",
        ".local/share/color-schemes/MaterialYouDark_darker_titlebar2.colors",
        ".local/share/color-schemes/MaterialYouLight.colors",
        ".local/share/color-schemes/MaterialYouLight2.colors",
        ".local/share/color-schemes/MaterialYouLight_darker_titlebar.colors",
        ".local/share/color-schemes/MaterialYouLight_darker_titlebar2.colors",
        ".local/state/quickshell/.venv",
        ".local/state/quickshell/venv-setup.log",
    ],
    // The runtime it is built on, shared with DMS, Caelestia and Ambxst; the
    // theming tool its colour scripts drive, shared with DMS and Ambxst; the
    // audio effects daemon its equalizer drives, shared with Ambxst; and the
    // three fonts its interface is drawn with, shared with Caelestia. All are
    // named so a switch between the shells that use them keeps them, while a
    // switch to a shell that needs none (Noctalia) takes them away.
    runtime_packages: &[
        "quickshell",
        "quickshell-git",
        "matugen",
        "easyeffects",
        "google-rubik-fonts",
        "cascadia-code-nf-fonts",
        "material-symbols-fonts",
    ],
    // The fork answers polkit from its own `PolkitService` — a `PolkitAgent`
    // that registers with polkitd when the shell starts — so it needs no
    // external agent, the same way DMS and Noctalia need none.
    // See `Identity::polkit_agent`.
    polkit_agent: &[],
    logo: Some("end4pc.svg"),
};
