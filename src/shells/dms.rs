//! Dux Shell (DMS; formerly DankMaterialShell), and the only shell here that is
//! not already in the base image.
//!
//! What is the same wherever it runs:
//!
//! - **install**: two privileged stages, in this order:
//!
//!   ```text
//!   1. quickshell-git
//!   2. dms dms-cli dgop danksearch dankcalendar-git matugen
//!   ```
//!
//!   Both from the `rakuos` repository, as are Caelestia's and Ambxst's
//!   packages.
//!
//!   `quickshell-git` is named explicitly rather than left to the solver: `dms`
//!   requires `(quickshell or quickshell-git)` and the same repository also carries
//!   `noctalia-qs`, which provides `quickshell` as well. That is the legacy
//!   Noctalia build, required only by the obsolete `noctalia-legacy` package;
//!   current Noctalia 5 is C++ and does not use quickshell at all.
//!
//!   `dankcalendar-git` is named for a different reason: nothing requires it. DMS
//!   calls it from `quickshell/Modules/DankDash`, so the calendar widget is simply
//!   broken without it, and no rpm dependency would ever have said so.
//!
//!   `khal` is deliberately absent. DMS's calendar is DankCalendar, which
//!   `dankcalendar-git` supplies; the terminal calendar was a second answer to a
//!   question the shell had already answered, and installing both put two
//!   calendars on the machine where the widget reads one.
//!
//!   The runtime is a stage of its own because that ordering is a safety property,
//!   not a convenience: if the runtime fails there is then nothing of DMS on the
//!   machine at all. One transaction would take the shell's own packages down with
//!   it for no benefit.
//!
//! - **uninstall**: one stage, `pkexec rum remove -y` over DMS's own packages and
//!   nothing else. It runs at the very end of a switch to something else, once
//!   the new shell has proved itself alive, and it deliberately leaves
//!   `quickshell-git` behind: that is a runtime rather than DMS, another shell
//!   can run on it, and taking it out from under the shell just switched to is
//!   the exact failure this list is here to prevent. An orphaned runtime stays a
//!   candidate for `rum autoremove`, which `docs/SAFETY.md` keeps as a separate
//!   explicit choice.
//!
//! - **start**: a systemd user unit. Confirmed present in the package at
//!   `/usr/lib/systemd/user/dms.service`, so starting means
//!   `systemctl --user start dms` and not a bare `dms run`.
//! - **`dcal` first**: it supplies the calendar widgets DMS draws. Nothing makes
//!   DMS wait for it, and DMS looks merely incomplete without it, so it is declared
//!   rather than left to the skel autostart, which only runs on the next login. The
//!   live switch starts it in the current session too.
//! - **companion data**: `dankcalendar-git`, `dgop` and `danksearch` own
//!   `companion_paths`, and DMS is the only shell here with any. Nothing installs those paths — the
//!   programs write them the first time they run — so the switch neither replaces
//!   them on the way in nor leaves them behind on the way out: they are backed up
//!   with DMS, removed only when that backup reproduces them byte for byte, and
//!   read back the next time DMS comes home. See
//!   [`crate::shell::Identity::companion_paths`].
//!
//! What differs per compositor is the [`Port`]. DMS's own config directory is the
//! same on both; the Hyprland page additionally carries the launcher entry that
//! hides Noctalia while DMS is the shell.
//!
//! Its state directory is `~/.local/state/dms`, declared as
//! [`Port::state_dir`] rather than as one more extra path: it is DMS's own state
//! rather than something the skel hands out, so it belongs to the same question as
//! the config — saved before a switch, cleaned after one, and read back from the
//! newest backup on the way home.

use crate::shell::{Identity, InstallPlan, UninstallPlan};

pub const IDENTITY: Identity = Identity {
    id: "dms",
    display_name: "Dux Shell",
    upstream: "duxlabs-dev/dux-shell",
    package: Some("dms"),
    binary: "dms",
    start_argv: &["dms", "run"],
    process_match: &[],
    install_plan: Some(InstallPlan {
        package: "dms",
        stages: &[
            // The runtime first and on its own: it is small, it is the piece most
            // likely to fail on its own, and if it does there is then nothing of
            // dms installed.
            &["quickshell-git"],
            &[
                "dms",
                "dms-cli",
                "dgop",
                "danksearch",
                "dankcalendar-git",
                "matugen",
            ],
        ],
        program: "pkexec",
        // `--refresh` because a cached copy of the metadata is only known to be
        // young, not known to be current. See `switch::install`.
        args: &["rum", "install", "-y", "--refresh"],
    }),
    uninstall_plan: Some(UninstallPlan {
        package: "dms",
        // Everything DMS installed, including the runtime it dragged in. The
        // runtime is the interesting one: it used to be left out on the grounds
        // that another shell might be running on it, which produced an orphan
        // `rum autoremove` was left to find instead of a clean machine. The
        // concern was right and the placement was wrong — the package that must
        // survive is the one the shell being switched **to** runs on, and that
        // is known at the moment of the switch, not when this list is written.
        // `switch::uninstall` therefore filters against
        // `target.runtime_packages`, which is where the guard lives now.
        //
        // `rum remove` also refuses any package another package still requires,
        // so `matugen` in particular cannot be pulled out from under something
        // that uses it — the solver is the second line of defence, not the first.
        stages: &[&[
            "dms",
            "dms-cli",
            "dgop",
            "danksearch",
            "dankcalendar-git",
            "matugen",
            // With the rest rather than in a stage of its own: nothing else on
            // the machine asks for it once `dms` and `dankcalendar-git` are in
            // the same transaction, so leaving it behind only ever produced an
            // orphan that `rum autoremove` was left to find.
            //
            // It is still a shared runtime, and that is now handled where it
            // belongs — `switch::uninstall` drops any package the shell being
            // switched to declares in `runtime_packages`. Caelestia and Ambxst
            // both do, so switching DMS → either of them keeps quickshell; DMS →
            // Noctalia does not, and takes it.
            "quickshell-git",
        ]],
        program: "pkexec",
        args: &["rum", "remove", "-y"],
    }),
    companion_systemd_user_units: &["dcal"],
    // `danksearch` is declared here rather than beside `dcal`, because its life
    // is different: `dcal` is started and stopped with the shell and never
    // disabled, while `dsearch` shares the shell's enablement — unmasked,
    // enabled and started when DMS comes in, stopped, disabled and masked when
    // it goes, on every compositor. The Niri image enables it globally
    // (`dsearch.service` is `WantedBy=default.target`), so leaving DMS without
    // masking it would leave the search daemon running for a shell that is
    // gone; on Hyprland nothing enables it, so the picker's own enable is what
    // starts it at the next login.
    //
    // The list lives on DMS and not on the shells that are *not* DMS: leaving
    // DMS is the moment these go down, and every shell a user might switch to
    // is already covered by that one event. Writing "mask dsearch" on Noctalia,
    // Caelestia and Ambxst would be the same fact in four places, and the fifth
    // shell would have to remember it.
    autostart_systemd_user_units: &["dsearch"],
    // Where the companions themselves keep their data. `dankcalendar-git` owns
    // the calendar database and its keyring, `dgop` owns the palette it writes,
    // and `danksearch` owns the settings its service reads; nothing installs any
    // of it, so the skel carries none of it and the backup is the only thing
    // that can hand it back after DMS has been away.
    //
    // Declared rather than left to `discard_paths`: a calendar database is data.
    // It is backed up with DMS, removed on the way out only when that backup
    // reproduces it byte for byte, and read back by `plan` the next time DMS is
    // the shell. `.config/dankcal` is empty on this machine today — it is still
    // declared, because the day dankcalendar writes a setting there the difference
    // between "backed up" and "thrown away without asking" is somebody's data.
    companion_paths: &[
        ".config/dankcal",
        ".config/danksearch",
        ".config/dgop",
        ".local/share/dankcal",
        ".local/state/dankcal",
    ],
    // Matugen renders these from the wallpaper, not the switch — DMS's
    // `matugen/configs/gtk3-*.toml` and `qt5ct.toml`/`qt6ct.toml` name them as
    // their output paths, and every one lands in a directory SHARED_PATHS
    // already walks. They are the exact counterpart of Noctalia's four: the same
    // four directories, the same reason, so that leaving DMS takes its theme
    // with it instead of leaving `dank-colors.css` beside a gtk.css that no
    // longer imports it.
    //
    // Only those four. DMS also writes into `.config/ghostty`, `.config/kitty`,
    // `.config/foot`, `.config/alacritty` and the rest of matugen's templates,
    // and those directories belong to the programs themselves rather than to
    // the session, so a switch neither backs them up nor cleans them. Listing
    // them here would fail `every_generated_path_is_backed_up`, which is the
    // test saying so out loud.
    generated_paths: &[
        ".config/gtk-3.0/dank-colors.css",
        ".config/gtk-4.0/dank-colors.css",
        ".config/qt5ct/colors/matugen.conf",
        ".config/qt6ct/colors/matugen.conf",
    ],
    // The rest of matugen's output, which is DMS's own work and nobody's data.
    //
    // `ghostty`, `zed`, `vesktop` and `equibop` get one file each: the directory
    // belongs to the program and a switch leaves it alone, but the theme inside
    // it is DMS's render and there is nothing in it worth keeping, so only that
    // file is named rather than the whole directory. The three colour schemes are the
    // same in KDE's own format.
    //
    // The caches go too. `.cache/dms` is several megabytes of
    // regenerated output (`matugen.lock`, `dms-colors.json`), and the two
    // `dankcal` spellings belong to `dankcalendar-git`, which an uninstall
    // takes away with DMS — its shader cache outlives the package only as
    // rubbish. `.cache/danksearch` is the index the search service builds —
    // close to 200 MiB on this machine — and once that service is masked and
    // its package gone it is the same kind of rubbish. `.cache/quickshell` is a
    // compiled-QML cache of a few hundred
    // bytes: the runtime may or may not still be on the machine after this
    // switch (see `runtime_packages`), but whoever runs it next recompiles
    // whatever it needs on the first start, so there is nothing here worth
    // carrying across either way.
    //
    // Nothing here is walked by the backup, and `every_discard_path_is_
    // never_preserved` fails the suite if any of it ever is.
    discard_paths: &[
        ".cache/dms",
        // The pre-2.0 spelling. DMS leaves it behind as a symlink to
        // `.cache/dms`; discarding both keeps the old link from dangling.
        ".cache/DankMaterialShell",
        ".cache/dankcal",
        ".cache/dank-calendar",
        ".cache/danksearch",
        ".cache/quickshell",
        ".config/equibop/themes/dank-discord.css",
        ".config/ghostty/themes/dankcolors",
        ".config/vesktop/themes/dank-discord.css",
        ".config/zed/themes/dank-zed-theme.json",
        ".local/share/color-schemes/DankMatugen.colors",
        ".local/share/color-schemes/DankMatugenDark.colors",
        ".local/share/color-schemes/DankMatugenLight.colors",
    ],
    // The runtime DMS is built on, which Caelestia and Ambxst are built on too,
    // and the theming tool it shares with Ambxst. Named so a switch to either
    // keeps them: DMS → Noctalia takes the runtime away, DMS → Ambxst keeps both.
    runtime_packages: &["quickshell", "quickshell-git", "matugen"],
    // DMS answers polkit from its own PolkitService, so it needs no external
    // agent. See `Identity::polkit_agent`.
    polkit_agent: &[],
    logo: Some("danklogo2.svg"),
};
