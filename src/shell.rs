//! The shell type and everything needed to inspect one.
//!
//! A shell here is the panel/widgets/status layer, not the compositor and not
//! the desktop environment. Hyprland stays running across every entry here, so
//! switching shells never restarts the compositor.
//!
//! # One shell, two files
//!
//! A shell's record splits in two, because most of it is the same on every
//! compositor and a little of it is not:
//!
//! - [`Identity`] — what the shell *is*: its upstream, its package, its install
//!   plan, its logo. One file per shell, under `crate::shells`. Written once.
//! - [`Port`] — what the shell is *on this compositor*: where its config lives,
//!   how it starts, which skel paths it carries. One file per shell **per
//!   compositor**, under `compositor::<id>::shells`.
//!
//! [`Shell::join`] puts the two together into the flat record the rest of the
//! program reads.
//!
//! The split is not tidiness for its own sake. DMS's install plan is two stages
//! and seven packages, the same wherever DMS runs; Noctalia's package and binary
//! are the same everywhere too. Written per compositor, each of those was a
//! second copy to keep in step, and two compositors disagreeing about which
//! package provides a shell is exactly the kind of drift that only shows up as a
//! failed switch. Because [`Port`] has no `id`, no `upstream` and no
//! `install_plan` field, that disagreement is not expressible.
//!
//! Which shells exist for a given compositor is still not decided here: those
//! registries live under `compositor::<id>::shells`, because the same shell may
//! not exist for every compositor. Hyprland and Niri are registered today.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StartMechanism {
    /// Launched from the compositor's autostart file.
    Autostart,
    /// Launched from a compositor start-event hook.
    StartEvent,
    /// Niri's `spawn-at-startup` in `config.kdl`. Kept apart from `Autostart`
    /// because the backend has to write a different syntax into a different
    /// file, and Niri waits for the command to return before continuing.
    ///
    /// Noctalia on Niri is the one port that uses it — see
    /// `compositor::niri::shells::noctalia`.
    SpawnAtStartup,
    /// Launched by a systemd user unit.
    SystemdUserUnit,
    /// Launched directly, with no supervisor.
    DirectProcess,
}

impl StartMechanism {
    /// Whether this shell is started by a systemd user unit.
    ///
    /// The two answers here have to agree with
    /// [`Shell::systemd_user_unit`](crate::shell::Shell::systemd_user_unit), and
    /// `a_mechanism_agrees_with_its_unit_name` fails the build if they ever stop
    /// doing so. `start` and `verify` both branch on this method rather than on
    /// the unit being present, so that a shell launched by the compositor's own
    /// autostart is not silently treated as a direct process — the answer would
    /// come out the same but for the wrong reason, and the day it stopped coming
    /// out the same there would be nothing to notice the change.
    pub fn uses_systemd_unit(self) -> bool {
        matches!(self, StartMechanism::SystemdUserUnit)
    }

    /// Whether a shell started this way has to be told to detach.
    ///
    /// Niri runs a `spawn-at-startup` command and waits for it to return before
    /// continuing with the rest of its startup, so a shell that stays in the
    /// foreground stalls the session behind it. Hyprland's `exec_on` and a bare
    /// spawn do not wait, so nothing there needs the flag.
    pub fn needs_detached_start(self) -> bool {
        matches!(self, StartMechanism::SpawnAtStartup)
    }
}

/// Where a package lives on a RakuOS system.
///
/// RakuOS keeps the read-only base image and the writable overlay in two
/// separate rpm databases, and `rpm -q` reads only the base one. A package that
/// `rum install` placed in the overlay is therefore reported as not installed by
/// `rpm -q`, which is why this is an enum instead of the bool the old code had.
///
/// Verified equivalent to `rum origin` on all three cases, without rum touching
/// the network.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Origin {
    /// Part of the read-only base image.
    Base,
    /// Installed into the overlay after the image was built.
    Overlay,
    NotInstalled,
}

impl Origin {
    pub fn installed(self) -> bool {
        !matches!(self, Origin::NotInstalled)
    }

    pub fn label(self) -> &'static str {
        match self {
            Origin::Base => "base image",
            Origin::Overlay => "overlay",
            Origin::NotInstalled => "not installed",
        }
    }

    /// The verdict from two independent database lookups.
    ///
    /// Base wins over overlay, matching what `rum origin` reports, so a package
    /// that somehow exists in both is not described twice.
    fn from_flags(base: bool, overlay: bool) -> Self {
        match (base, overlay) {
            (true, _) => Origin::Base,
            (false, true) => Origin::Overlay,
            (false, false) => Origin::NotInstalled,
        }
    }
}

/// What a shell is, everywhere it runs.
///
/// One of these per shell, in `crate::shells::<id>`. Every field is a fact about
/// the shell itself and would be written identically for every compositor that
/// supports it — which is the point: there is only ever one copy.
#[derive(Clone, Copy, Debug)]
pub struct Identity {
    pub id: &'static str,
    pub display_name: &'static str,
    pub upstream: &'static str,
    /// The package that identifies this shell, and what availability is checked
    /// against. `None` for a shell that is not packaged at all, where a binary in
    /// `PATH` is the only evidence there is.
    pub package: Option<&'static str>,
    pub binary: &'static str,
    pub start_argv: &'static [&'static str],
    /// Extra arguments that identify this shell's live process when its `comm`
    /// is not `binary`.
    ///
    /// Most shells here are their own process, so matching `/proc/<pid>/comm`
    /// against `binary` is enough. Caelestia is not: `caelestia shell -d` hands
    /// off to the shared `quickshell` runtime, and the process that ends up on
    /// screen is `qs -c caelestia …` whose `comm` is `qs`, never `caelestia`.
    /// Name matching alone would then read a running Caelestia as "not running",
    /// so `stop` would report a clean stop while doing nothing and `verify`
    /// would time out and roll the switch back.
    ///
    /// Each entry is one whole argument, and the list must appear **consecutively
    /// and in order** in the process's `cmdline`. A Caelestia process is matched
    /// by `["-c", "caelestia"]`, which is specific enough that the sibling
    /// quickshell shells (`noctalia-qs`, DMS) do not accidentally match it.
    ///
    /// Empty is the usual case, and means `comm` alone decides.
    pub process_match: &'static [&'static str],
    /// How to install this shell if it is missing, or `None` when there is
    /// nothing to run. See [`InstallPlan`].
    pub install_plan: Option<InstallPlan>,
    /// How to take this shell's packages off again, or `None` when that is not
    /// possible or not needed. See [`UninstallPlan`].
    pub uninstall_plan: Option<UninstallPlan>,
    /// User services this shell needs before it is worth starting, in order. A
    /// shell's own unit does not depend on these, so systemd will happily start
    /// it with them still down. DMS uses this for `dcal`, which supplies the
    /// calendar widgets it draws: the service comes up fine without it and looks
    /// subtly wrong, which is the kind of fault nobody notices until later. Empty
    /// means the shell needs nothing else, which is the common case.
    pub companion_systemd_user_units: &'static [&'static str],
    /// Data paths the shell itself or its companion programs keep, relative to
    /// `$HOME`, that a person would want back.
    ///
    /// Not the companions themselves — those are
    /// [`Self::companion_systemd_user_units`] — but what they, or the shell,
    /// store that is neither configuration nor state. DMS's calendar is
    /// `dankcalendar-git`, and it owns `~/.local/share/dankcal` (its database and
    /// keyring) and `~/.local/state/dankcal`; `dgop` owns `~/.config/dgop`.
    /// Ambxst keeps its chat history and pinned clipboard in
    /// `~/.local/share/ambxst` and its notes in `~/.local/share/ambxst-notes`.
    /// The paths already carry the shell's own data as well as a companion's, so
    /// this is not the only sense the word is used in. Nothing installs any of
    /// it: the programs write it the first time they run and no skel carries it,
    /// so a switch that ignored it would drop the calendar, the chats or the
    /// notes the moment the shell left.
    ///
    /// Handled like [`Self::generated_paths`] and not like
    /// [`Self::discard_paths`]: backed up with the shell, taken off the machine on
    /// the way out only when this switch's backup reproduces them byte for byte,
    /// and read back by `plan` when the shell comes home. The data is real — a
    /// calendar database is not something a restart rebuilds — which is exactly
    /// the property [`Self::generated_paths`] is checked for, and the reason the
    /// no-data shortcut [`Self::discard_paths`] allows must not be taken here.
    ///
    /// On [`Identity`] rather than [`Port`] because it is a fact about the shell:
    /// the same programs keep the same paths on every compositor.
    ///
    /// Empty is the usual answer: a shell that keeps nothing outside its own
    /// config and state has nothing to declare, and two of the four registered
    /// shells are exactly that.
    pub companion_paths: &'static [&'static str],
    /// Files this shell renders into shared directories while it runs, relative
    /// to `$HOME`.
    ///
    /// These are not installed by the switch. The shell produces them from its
    /// own templates on every start — Noctalia renders `noctalia.css` for GTK and
    /// a colour scheme for qt5ct/qt6ct — so they sit in directories shared with
    /// everything else and outlive the shell that made them. `clean` removes them
    /// on the way out for the same reason it removes `config_dir`: leaving the
    /// previous shell's furniture in place is untidy, and the next shell has no
    /// use for it.
    ///
    /// The safety is the same, not weaker: an entry is only deleted when this
    /// switch's backup holds it byte for byte. That makes every path here
    /// dependent on `backup_sources` walking it, which is why the four directories
    /// Noctalia writes into — `gtk-3.0`, `gtk-4.0`, `qt5ct`, `qt6ct` — are
    /// already in [`crate::switch::SHARED_PATHS`], and why
    /// `every_generated_path_is_backed_up` in `src/tests/lib_registry.rs`
    /// fails the suite rather than the switch.
    ///
    /// On [`Identity`] rather than [`Port`] because it is a fact about the
    /// shell: the same templates write the same paths on every compositor.
    ///
    /// Empty is the usual answer. A shell that keeps itself inside its own
    /// `config_dir` writes nothing into directories shared with anything else,
    /// and three of the four registered shells are exactly that.
    pub generated_paths: &'static [&'static str],
    /// Paths this shell leaves behind that hold **no user data**, relative to
    /// `$HOME`, and are therefore removed without the byte-for-byte check
    /// [`Self::generated_paths`] is subject to.
    ///
    /// The distinction is what the rule this field encodes says: *something that
    /// carries data is backed up; something that carries none is cleaned.* A
    /// theme file a shell rendered, or a cache it wrote, can be regenerated by
    /// the shell that made them and edited by nobody — there is nothing in them
    /// worth preserving, and nothing a backup could restore that a restart would
    /// not rebuild. Backing them up would only make every switch larger and
    /// slower to no end.
    ///
    /// What it must never contain is anything a person has written to. That is
    /// enforced rather than trusted: `every_discard_path_is_never_preserved`
    /// fails the suite if an entry here is reachable from any path a switch
    /// backs up or installs — `SHARED_PATHS`, `config_dir`, `state_dir`,
    /// `skel_extra_paths`, `generated_paths`, `backup_extra_paths`, or the skel
    /// itself. A path that is reachable from any of those belongs in
    /// [`Self::generated_paths`] instead, where deletion requires the backup to
    /// hold a byte-for-byte copy first.
    ///
    /// `clean` still refuses to take a path the incoming shell owns, so a
    /// discarded path can never be one the next shell needs. That is the only
    /// guard left, and it is the right amount: there is nothing here to lose.
    ///
    /// On [`Identity`] rather than [`Port`] because it is a fact about the
    /// shell: the same renders write the same paths on every compositor.
    ///
    /// Empty is the usual answer, and is correct for a shell that renders
    /// nothing outside its own `config_dir`.
    pub discard_paths: &'static [&'static str],
    /// Packages this shell runs **on** rather than packages that are this
    /// shell, relative to a removal plan.
    ///
    /// The guard that lets a removal plan take more than its own packages out.
    /// `quickshell` is the case the field exists for: every packaged non-base
    /// shell (DMS, Caelestia, Ambxst) installs it and runs on it, and `matugen`
    /// is shared by DMS and Ambxst — so a switch between them that removed the
    /// runtime would leave the shell it handed off to dead. The outgoing shell's
    /// uninstall therefore skips any package the incoming shell declares here,
    /// which turns "never remove a shared runtime" from a rule somebody has to
    /// remember into a check the code performs.
    ///
    /// Listing a package here can only ever cause it to be **kept**, so the
    /// failure mode of a wrong entry is a leftover rather than a broken shell —
    /// which is why it is declared even where upstream's own `Requires` is vague
    /// about the runtime: these are all Quickshell shells, and keeping a package
    /// nobody needed is the harmless direction to be wrong in.
    ///
    /// On [`Identity`] rather than [`Port`] because it is a fact about the
    /// shell: what a shell runs on does not change from compositor to
    /// compositor.
    pub runtime_packages: &'static [&'static str],
    /// The polkit authentication agent this shell starts, as an argv to run, or
    /// empty when the shell answers polkit itself.
    ///
    /// DMS brings up its own agent and Noctalia sets `polkit_agent = true`, so
    /// both answer `pkexec` from the shell process. Caelestia and Ambxst do not:
    /// their Hyprland config runs
    /// `/usr/libexec/polkit-mate-authentication-agent-1` from an
    /// `exec-once`/autostart line, which a live switch never reaches — a switch
    /// reloads the config, and `exec-once` does not run again on reload. So a
    /// switch that stops such a shell and starts one of these finds nobody to
    /// answer `pkexec` when it tries to remove the old shell's packages, and the
    /// stage comes back dismissed. Running the agent here, before that stage, is
    /// what lets the removal happen.
    ///
    /// On [`Identity`] rather than [`Port`] because it is a fact about the
    /// shell: the agent it starts does not change from compositor to compositor.
    pub polkit_agent: &'static [&'static str],
    /// File name inside the project `assets/` directory.
    pub logo: Option<&'static str>,
}

/// How one shell lives on one compositor.
///
/// One of these per shell **per compositor**, in `compositor::<id>::shells::<shell>`.
/// Only the facts that can legitimately differ between compositors belong here;
/// anything that would be the same on both is in [`Identity`] instead.
#[derive(Clone, Copy, Debug)]
pub struct Port {
    /// Where this shell's configuration lives, relative to `$HOME`. `None` means
    /// it is not known, and the shell must then be refused rather than guessed at
    /// — see `docs/SAFETY.md`, rule 4.
    pub config_dir: Option<&'static str>,
    /// Where this shell keeps state that is not configuration, relative to
    /// `$HOME`. Backed up alongside the config: Noctalia keeps most of itself
    /// here, so a backup that skipped it would lose the user's actual setup.
    /// `None` means the shell has none, or that it is not known, and less is
    /// then carried out of a switch.
    pub state_dir: Option<&'static str>,
    /// The systemd user unit that starts this shell, when it has one.
    pub systemd_user_unit: Option<&'static str>,
    pub mechanism: StartMechanism,
    /// Whether the image enables this unit globally, so the session starts it on
    /// its own at login even when another shell was chosen.
    ///
    /// This differs per compositor even though the unit is the same one, because
    /// it is the image that decides it: `rakuos-niri` runs
    /// `systemctl --global enable dms`, and `dms.service` is
    /// `WantedBy=graphical-session.target`, so every login starts DMS whether or
    /// not it is the shell the user picked. Stopping the unit is then only half a
    /// switch — it has to be masked while another shell holds the session, or it
    /// comes back at the next login beside the chosen shell. The Hyprland image
    /// ships the same unit but enables nothing globally, which is why this is
    /// `false` there, and why the flag cannot live on [`Identity`].
    pub unit_enabled_by_image: bool,
    /// Other paths, relative to `$HOME`, that this shell's skel also carries.
    ///
    /// The compositor config and `config_dir` are handled by name, because where
    /// they come from is a decision worth making explicitly. These are the rest:
    /// the theme files, the runtime state directory, and the one launcher entry a
    /// switch has to overwrite. They exist in the skel, so the switch installs
    /// them, and a path listed here may be a single file rather than a directory,
    /// which is what makes it possible for one shell to hide another's launcher
    /// without having to own the whole directory it lives in.
    ///
    /// Listing a path both shells would install whole would be a mistake: the swap
    /// replaces the live path outright rather than merging into it, so a shared
    /// directory such as `~/.local/share/applications` must never appear here.
    pub skel_extra_paths: &'static [&'static str],
    /// Paths this shell's skel installs that are neither its `config_dir` nor its
    /// `state_dir`, and that nothing else on the desktop would have backed up.
    ///
    /// A switch writes every path in `skel_extra_paths`, so each of them has to be
    /// in the backup or the switch destroys the user's copy of a file it is about
    /// to replace, with no way back. `config_dir` and `state_dir` are already
    /// included by name, and `~/.config/gtk-3.0` and `~/.config/gtk-4.0` are
    /// session-wide, so the theme files both shells install need no mention here.
    /// What is left is genuinely per shell: Noctalia's launcher entry, which DMS
    /// overwrites to hide it, and DMS's runtime state.
    ///
    /// Empty means this shell needs nothing beyond its config and state. The
    /// invariant is enforced by `every_installed_path_is_backed_up` in
    /// `src/tests/lib_registry.rs`, so adding a path to `skel_extra_paths`
    /// without adding it here fails the test suite rather than silently
    /// shipping an unrecoverable switch.
    pub backup_extra_paths: &'static [&'static str],
}

#[derive(Clone, Copy, Debug)]
pub struct Shell {
    pub id: &'static str,
    pub display_name: &'static str,
    pub upstream: &'static str,
    pub package: Option<&'static str>,
    pub binary: &'static str,
    pub start_argv: &'static [&'static str],
    /// See [`Identity::process_match`]. Carried here for the same reason as the
    /// rest: `stop` and `verify` read the shell being switched through `Shell`,
    /// so which process counts as this shell has to travel with it.
    pub process_match: &'static [&'static str],
    pub config_dir: Option<&'static str>,
    /// Where this shell keeps state that is not configuration, relative to
    /// `$HOME`. Backed up alongside the config: Noctalia keeps most of itself
    /// here, so a backup that skipped it would lose the user's actual setup.
    /// `None` means the shell has none, or that it is not known, and less is
    /// then carried out of a switch.
    pub state_dir: Option<&'static str>,
    pub systemd_user_unit: Option<&'static str>,
    /// User services this shell needs before it is worth starting, in order.
    ///
    /// A shell's own unit does not depend on these, so systemd will happily start
    /// it with them still down. DMS uses this for `dcal`, which supplies the
    /// calendar widgets it draws: the service comes up fine without it and looks
    /// subtly wrong, which is the kind of fault nobody notices until later. Empty
    /// means the shell needs nothing else, which is the common case.
    pub companion_systemd_user_units: &'static [&'static str],
    /// See [`Identity::companion_paths`]. Kept on `Shell` for the same reason as
    /// [`Self::generated_paths`]: both `clean` and `plan` read the shell they are
    /// working on through `Shell`, so the paths a companion program owns have to
    /// travel with the shell rather than be looked up again from the registry.
    pub companion_paths: &'static [&'static str],
    /// Other paths, relative to `$HOME`, that this shell's skel also carries.
    ///
    /// The compositor config and `config_dir` are handled by name, because where
    /// they come from is a decision worth making explicitly. These are the rest:
    /// the theme files, the runtime state directory, and the one launcher entry a
    /// switch has to overwrite. They exist in the skel, so the switch installs them,
    /// and a path listed here may be a single file rather than a directory, which
    /// is what makes it possible for one shell to hide another's launcher without
    /// having to own the whole directory it lives in.
    ///
    /// Listing a path both shells would install whole would be a mistake: the swap
    /// replaces the live path outright rather than merging into it, so a shared
    /// directory such as `~/.local/share/applications` must never appear here.
    pub skel_extra_paths: &'static [&'static str],
    /// See [`Identity::generated_paths`]. Kept on `Shell` for the same reason as
    /// [`Self::backup_extra_paths`]: `clean` reads the shell being left behind
    /// through `Shell`, so the paths it is allowed to remove have to travel with
    /// it rather than be looked up again from the registry.
    pub generated_paths: &'static [&'static str],
    /// See [`Identity::discard_paths`]. Kept on `Shell` for the same reason as
    /// [`Self::generated_paths`]: `clean` reads the shell being left behind
    /// through `Shell`, so the paths it may throw away have to travel with it.
    pub discard_paths: &'static [&'static str],
    /// See [`Identity::runtime_packages`]. Kept on `Shell` because the guard
    /// belongs where the removal runs: `apply` reads both shells through
    /// `Shell`, and it has the incoming one in hand at that moment.
    pub runtime_packages: &'static [&'static str],
    /// See [`Identity::polkit_agent`]. Kept on `Shell` for the same reason as
    /// [`Self::runtime_packages`]: the uninstall stage reads the incoming shell
    /// through `Shell` and starts its agent itself before asking for a password.
    pub polkit_agent: &'static [&'static str],
    /// See [`Port::backup_extra_paths`]. Kept here as well because a backup is
    /// taken from the live system and read back through `Shell`, so the paths a
    /// switch could destroy have to travel with the shell they belong to.
    pub backup_extra_paths: &'static [&'static str],
    /// How to install this shell if it is missing. `None` means there is nothing
    /// to run: either the base image already has it, or no repository carries it.
    pub install_plan: Option<InstallPlan>,
    /// How to take this shell's packages off again after switching away from it,
    /// so the shell just left does not accumulate as leftover. `None` means the
    /// operation does not exist, because the shell has no package a switch could
    /// have installed. A package the base image owns is not a reason for `None`:
    /// `switch::uninstall` filters those out by origin at the moment of removal,
    /// so a shell that is in one image and installable on another — Noctalia —
    /// declares the plan and lets the origin check decide.
    pub uninstall_plan: Option<UninstallPlan>,
    /// How this shell gets started on this compositor.
    ///
    /// Not decoration, and not something the backend re-derives: `apply::start`
    /// branches on it to decide between a unit and a spawn, and `apply::verify`
    /// reads it to decide what "up" means for this shell. Every value is
    /// consistent with `systemd_user_unit` above, and
    /// `a_mechanism_agrees_with_its_unit_name` enforces that.
    ///
    /// `Autostart`, `SpawnAtStartup` and `StartEvent` have no unit because the
    /// compositor's own configuration is the launcher. `SpawnAtStartup` stays
    /// distinct from `Autostart` even though neither has a unit, because Niri waits
    /// for a startup command to return before continuing — a foreground shell
    /// there would stall the session — so that mechanism is what decides the
    /// command has to carry `--daemon`.
    pub mechanism: StartMechanism,
    /// See [`Port::unit_enabled_by_image`]. Carried here for the same reason as
    /// the rest of the port: `stop` and `start` read the shell being switched
    /// through [`Self`], so a fact that decides what they do to the unit has to
    /// travel with the shell rather than be looked up from the registry again.
    pub unit_enabled_by_image: bool,
    /// File name inside the project `assets/` directory.
    pub logo: Option<&'static str>,
}

impl Shell {
    /// The record the rest of the program reads: one shell, on one compositor.
    ///
    /// `const`, so a per-shell file can state its result as a plain declaration
    /// and every consumer can take a `&'static Shell`:
    ///
    /// ```text
    /// const PORT: Port = Port { /* only what differs on this compositor */ };
    /// pub static SHELL: Shell = Shell::join(IDENTITY, PORT);
    /// ```
    ///
    /// A doc comment rather than a doctest, because a real snippet needs an
    /// `IDENTITY` and a complete `Port` to mean anything, and a partial one would
    /// have to be `ignore`d to compile — which is a comment with extra steps. The
    /// declarations this stands for are the seven files under
    /// `compositor/*/shells/*/`, and those are compiled by the build.
    ///
    /// Takes both halves by value. They are `Copy`, so this is a field-by-field
    /// assembly with nothing borrowed and nothing shared between the two
    /// compositors' copies beyond the `&'static str`s they were built from.
    pub const fn join(identity: Identity, port: Port) -> Self {
        Self {
            id: identity.id,
            display_name: identity.display_name,
            upstream: identity.upstream,
            package: identity.package,
            binary: identity.binary,
            start_argv: identity.start_argv,
            process_match: identity.process_match,
            companion_systemd_user_units: identity.companion_systemd_user_units,
            companion_paths: identity.companion_paths,
            install_plan: identity.install_plan,
            uninstall_plan: identity.uninstall_plan,
            generated_paths: identity.generated_paths,
            discard_paths: identity.discard_paths,
            runtime_packages: identity.runtime_packages,
            polkit_agent: identity.polkit_agent,
            logo: identity.logo,

            config_dir: port.config_dir,
            state_dir: port.state_dir,
            systemd_user_unit: port.systemd_user_unit,
            mechanism: port.mechanism,
            unit_enabled_by_image: port.unit_enabled_by_image,
            skel_extra_paths: port.skel_extra_paths,
            backup_extra_paths: port.backup_extra_paths,
        }
    }
}

/// This project's own directory name, used to keep its files from colliding with
/// anything else under a shared XDG directory.
pub const PICKER_DIR: &str = "rakuos-shell-picker";

/// How this shell gets installed when it is not there yet.
///
/// Declared next to the shell rather than in a dispatch table, because whether a
/// shell can be installed is a fact about that shell. Every registered shell
/// ships in the `rakuos` repository; Noctalia also happens to be in the Hyprland
/// base image, where its install never runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstallPlan {
    /// The package that identifies this shell, and what availability is checked
    /// against.
    pub package: &'static str,
    /// The install stages, in the order they must run. Each stage is one
    /// privileged transaction with its own prompt.
    ///
    /// Ordering is a safety property, not a convenience. A shell's runtime goes
    /// in its own stage first: it is small, it is the piece most likely to fail
    /// for its own reasons, and if it fails there is then nothing of the shell
    /// itself installed. One transaction would install everything or roll back
    /// together, so a failure in the runtime would take the shell's own packages
    /// with it for no benefit.
    ///
    /// Naming the packages inside a stage also takes a choice away from the
    /// dependency solver, which is stronger than inspecting the choice
    /// beforehand. `dms` requires `(quickshell or quickshell-git)` and the
    /// repository also carries `noctalia-qs`, which provides the same capability;
    /// a check reads metadata that can already be stale by the time the install
    /// runs, while the command line is read at the moment it matters.
    pub stages: &'static [&'static [&'static str]],
    /// The program to run, and its arguments.
    ///
    /// Package names do not belong here: they come from [`Self::stages`], so that
    /// the logged command and the command that actually runs cannot drift apart
    /// by someone forgetting to repeat a name in two places.
    pub program: &'static str,
    pub args: &'static [&'static str],
}

/// String equality in a `const fn`.
///
/// `==` on `&str` is not const yet, so the bytes are compared by hand. Only used
/// to check that a package name appears in a stage, where both sides are short
/// literals and the length check settles most comparisons.
const fn same_bytes(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

impl InstallPlan {
    /// Whether this plan describes an install that can actually do something.
    ///
    /// Three ways to get it wrong, all of which look fine in a source listing:
    ///
    /// - no stages at all, so there is nothing to run;
    /// - an empty stage, which [`Self::packages`] silently drops;
    /// - a `package` that appears in no stage, so the shell the plan is
    ///   declared on is not among the packages it installs.
    ///
    /// The first two are not hypothetical tidiness. `packages()` on an empty plan
    /// returns an empty list, and `install()` asks whether every package in that
    /// list is already present — a question with no packages in it is answered
    /// yes. The switch would then report the shell as already installed, after
    /// taking a backup and getting a polkit authorization out of the user, and
    /// go on to start a shell that was never installed.
    ///
    /// `const` so a registry guard can run it while iterating the statics.
    pub const fn is_well_formed(&self) -> bool {
        if self.stages.is_empty() {
            return false;
        }
        // Tracked rather than returned on: finding the package must not let an
        // empty stage further along go unchecked.
        let mut named = false;
        let mut index = 0;
        while index < self.stages.len() {
            let stage = self.stages[index];
            if stage.is_empty() {
                return false;
            }
            let mut item = 0;
            while item < stage.len() {
                if same_bytes(self.package, stage[item]) {
                    named = true;
                }
                item += 1;
            }
            index += 1;
        }
        named
    }

    /// The whole plan as one command line, for the log and the progress panel.
    ///
    /// This is what the plan would amount to, shown as a single line so the user
    /// can read the full set before authorizing anything. The runs are per stage.
    ///
    /// `pkexec` and not a password prompt: the password goes to the polkit
    /// agent, so it never passes through this process or its log.
    pub fn command_line(&self) -> String {
        self.line_for(&self.packages())
    }

    /// One stage as a command line.
    pub fn stage_command_line(&self, stage: &[&str]) -> String {
        self.line_for(stage)
    }

    fn line_for(&self, packages: &[&str]) -> String {
        let mut line = self.program.to_string();
        for arg in self.args {
            line.push(' ');
            line.push_str(arg);
        }
        for package in packages {
            line.push(' ');
            line.push_str(package);
        }
        line
    }

    /// Every package the plan installs, in stage order and without duplicates.
    pub fn packages(&self) -> Vec<&'static str> {
        let mut packages: Vec<&'static str> = Vec::new();
        for stage in self.stages {
            for package in *stage {
                if !packages.contains(package) {
                    packages.push(package);
                }
            }
        }
        packages
    }
}

/// What it takes to take a shell's packages off the machine again.
///
/// Declared beside [`InstallPlan`] for the same reason it exists: whether a
/// shell can be removed is a fact about that shell, not a decision for the
/// switch to invent. Every packaged shell declares one; a package the base image
/// owns is kept out of the transaction by origin, in `crate::switch::uninstall`,
/// rather than by leaving the plan off — which is what lets Noctalia carry a plan
/// that is a no-op on Hyprland and real on Niri.
///
/// The stages run in the order they are written, and there is usually only one:
/// a plan names the shell's own packages and stops there. Shared runtimes are
/// left on the machine on purpose — another shell may run on the same one, and
/// taking it out from under the shell just switched to is the exact failure this
/// list exists to prevent. An orphaned runtime is a candidate for
/// `rum autoremove`, which `docs/SAFETY.md` keeps as a separate explicit choice
/// rather than something a switch decides on its own. `rum remove` refuses any
/// package another package still requires, so the dependency solver is a second
/// line of defence behind this declaration rather than a replacement for it.
#[derive(Clone, Copy, Debug)]
pub struct UninstallPlan {
    /// The package that identifies this shell, so a guard can check the shell
    /// being uninstalled is actually among the packages being removed.
    pub package: &'static str,
    /// The removal stages, in the order they must run. Each stage is one
    /// privileged transaction with its own prompt, exactly as for an install.
    pub stages: &'static [&'static [&'static str]],
    pub program: &'static str,
    pub args: &'static [&'static str],
}

impl UninstallPlan {
    /// Whether this plan could actually remove something.
    ///
    /// The two ways to get it wrong are the same two as [`InstallPlan`] minus
    /// the package check's mirror image: no stages at all, or an empty stage.
    /// An empty stage matters because a run over it removes nothing and reports
    /// success, which is how a shell would end up half-removed and still
    /// counted as installed.
    ///
    /// `const` so a registry guard can run it while iterating the statics.
    pub const fn is_well_formed(&self) -> bool {
        if self.stages.is_empty() {
            return false;
        }
        let mut named = false;
        let mut index = 0;
        while index < self.stages.len() {
            let stage = self.stages[index];
            if stage.is_empty() {
                return false;
            }
            let mut item = 0;
            while item < stage.len() {
                if same_bytes(self.package, stage[item]) {
                    named = true;
                }
                item += 1;
            }
            index += 1;
        }
        named
    }

    /// One stage as a command line, for the log and the progress panel.
    ///
    /// `pkexec` rather than a password prompt, so the password goes to the
    /// polkit agent and never passes through this process or its log.
    pub fn stage_command_line(&self, stage: &[&str]) -> String {
        let mut line = self.program.to_string();
        for arg in self.args {
            line.push(' ');
            line.push_str(arg);
        }
        for package in stage {
            line.push(' ');
            line.push_str(package);
        }
        line
    }

    /// Every package the plan removes, in stage order and without duplicates.
    pub fn packages(&self) -> Vec<&'static str> {
        let mut packages: Vec<&'static str> = Vec::new();
        for stage in self.stages {
            for package in *stage {
                if !packages.contains(package) {
                    packages.push(package);
                }
            }
        }
        packages
    }
}

/// Look a shell up across every registered compositor. Registries stay
/// per-compositor; this is only for resolving an id that came from elsewhere,
/// such as the saved session state.
pub fn find(id: &str) -> Option<&'static Shell> {
    crate::compositor::all()
        .iter()
        .flat_map(|c| c.shells())
        .find(|s| s.id == id)
}

impl Shell {
    pub fn binary_present(&self) -> bool {
        which(self.binary).is_some()
    }

    /// Where this shell's package lives, if it has one.
    ///
    /// `None` for shells that are not packaged at all, where a binary in `PATH`
    /// is the only evidence there is.
    pub fn origin(&self) -> Option<Origin> {
        self.package.map(package_origin)
    }

    pub fn installed(&self) -> bool {
        installed_from(self.origin(), self.binary_present(), self.config_present())
    }

    /// Has this shell actually been set up on this machine?
    pub fn config_present(&self) -> bool {
        self.config_path().is_some_and(|dir| dir.is_dir())
    }

    pub fn config_path(&self) -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        self.config_dir.map(|d| PathBuf::from(home).join(d))
    }

    pub fn state_path(&self) -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        self.state_dir.map(|d| PathBuf::from(home).join(d))
    }

    pub fn mechanism_label(&self) -> &'static str {
        match self.mechanism {
            StartMechanism::Autostart => "compositor autostart",
            StartMechanism::StartEvent => "compositor start event",
            StartMechanism::SpawnAtStartup => "niri spawn-at-startup",
            StartMechanism::SystemdUserUnit => "systemd user unit",
            StartMechanism::DirectProcess => "direct process",
        }
    }

    pub fn start_command(&self) -> String {
        self.start_argv.join(" ")
    }

    pub fn running(&self) -> bool {
        !pids_matching(self.binary, self.process_match).is_empty()
    }

    /// The line under the shell name on its card.
    ///
    /// Says what clicking will do, per docs/UI.md: an installed, idle shell is
    /// a candidate to switch to, while an uninstalled one has nothing to switch
    /// to at all. `running` is the one case where clicking changes nothing,
    /// which the dialog then explains.
    pub fn status_label(&self) -> &'static str {
        if self.running() {
            "running"
        } else if self.installed() {
            "switch"
        } else {
            "not installed"
        }
    }
}

fn which(bin: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(bin))
        .find(|p| p.is_file())
}

/// Whether a shell counts as installed, from the three facts we can observe.
///
/// Split out from the subprocess calls so the rules can be tested directly.
fn installed_from(origin: Option<Origin>, binary: bool, config: bool) -> bool {
    match origin {
        Some(origin) => origin.installed() || binary,
        // Nothing to ask about, so the binary has to carry the whole answer, and
        // it cannot be trusted alone: installing DMS pulls in noctalia-qs, which
        // puts quickshell in PATH, so any quickshell-based shell with no overlay
        // entry would look installed while its config directory does not exist.
        // Requiring the config too keeps a shared runtime from standing in for a
        // shell that was never set up.
        None => binary && config,
    }
}

/// The overlay rpm database path, as reported by `rum paths`.
///
/// Read from the system rather than hardcoded, because that path is an
/// implementation detail of the running image. Resolved once per process.
fn overlay_dbpath() -> Option<&'static Path> {
    static DB: OnceLock<Option<PathBuf>> = OnceLock::new();
    DB.get_or_init(|| {
        let out = Command::new("rum").args(["paths"]).output().ok()?;
        parse_overlay_dbpath(&String::from_utf8_lossy(&out.stdout))
    })
    .as_deref()
}

/// Pull the overlay database path out of `rum paths` output.
fn parse_overlay_dbpath(text: &str) -> Option<PathBuf> {
    text.lines()
        .find_map(|line| line.trim().strip_prefix("overlay rpmdb:"))
        .map(|path| PathBuf::from(path.trim()))
        .filter(|path| path.is_absolute())
}

/// Is `pkg` present in the rpm database at `dbpath`?
///
/// `None` means the default database, which on RakuOS is the base image.
fn in_rpmdb(dbpath: Option<&Path>, pkg: &str) -> bool {
    let mut cmd = Command::new("rpm");
    if let Some(db) = dbpath {
        cmd.arg("--dbpath").arg(db);
    }
    cmd.args(["-q", pkg])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// Whether the package is installed, and where it came from if it is.
///
/// Gives the same answer as `rum origin` without rum touching the network: that
/// command spends several seconds refreshing repository metadata, and this runs
/// once per shell every time the card list is built.
pub fn package_origin(pkg: &str) -> Origin {
    Origin::from_flags(in_rpmdb(None, pkg), in_rpmdb(overlay_dbpath(), pkg))
}

/// Every pid that is this shell, sorted.
///
/// A process matches when its `comm` is exactly `binary`, or — when
/// `argv_match` is not empty — when its `cmdline` contains those arguments
/// consecutively and in order. The second rule exists for shells that run on a
/// shared runtime and therefore have a process name that says nothing about
/// which shell it is; see [`Identity::process_match`]. Read from `/proc` rather
/// than shelling out, and shared with `apply` so detection, stopping and
/// verification cannot disagree about what "running" means.
pub fn pids_matching(binary: &str, argv_match: &[&str]) -> Vec<u32> {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    let mut pids: Vec<u32> = entries
        .flatten()
        .filter_map(|entry| {
            let pid = entry.file_name().to_str()?.parse::<u32>().ok()?;
            if std::fs::read_to_string(format!("/proc/{pid}/comm"))
                .map(|comm| comm.trim() == binary)
                .unwrap_or(false)
            {
                return Some(pid);
            }
            if argv_match.is_empty() {
                return None;
            }
            let raw = std::fs::read(format!("/proc/{pid}/cmdline")).ok()?;
            let args: Vec<&[u8]> = raw.split(|b| *b == 0).filter(|s| !s.is_empty()).collect();
            argv_contains(&args, argv_match).then_some(pid)
        })
        .collect();
    pids.sort_unstable();
    pids
}

/// Whether `needle` appears as a run of whole arguments inside `args`.
fn argv_contains(args: &[&[u8]], needle: &[&str]) -> bool {
    needle.len() <= args.len()
        && args.windows(needle.len()).any(|window| {
            window
                .iter()
                .zip(needle)
                .all(|(arg, want)| *arg == want.as_bytes())
        })
}

#[cfg(test)]
#[path = "tests/shell.rs"]
mod tests;
