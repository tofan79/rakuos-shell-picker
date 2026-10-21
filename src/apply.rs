//! Putting a shell's configuration in place, starting it, and retiring the old one.
//!
//! The order here is the whole point, and every step exists to make the one after
//! it safe:
//!
//! 1. work out where each path comes from, and say so out loud;
//! 2. build the replacement in a staging directory and count it;
//! 3. confirm the backup that makes all of this reversible is still on disk;
//! 4. stop the old shell, and take its backup again now that it can no longer
//!    write to what the first copy recorded;
//! 5. swap, which is a pair of renames inside one filesystem;
//! 6. start the new shell;
//! 7. check it actually came up;
//! 8. only then end the session, so the next login starts from the new config
//!    rather than from a half-applied mix.
//!
//! Nothing is deleted. Whatever was in the way is renamed aside, so the previous
//! state is still on disk after a successful switch and is what a rollback puts
//! back after a failed one.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use crate::compositor::Compositor;
use crate::shell::Shell;

/// Where one path's replacement copy comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    /// Copied out of a backup taken during an earlier switch.
    Backup(PathBuf),
    /// Copied out of the skel shipped in the source tree.
    Skel,
}

impl Origin {
    fn label(&self) -> String {
        match self {
            Origin::Skel => "skel".to_string(),
            Origin::Backup(root) => format!(
                "backup {}",
                root.file_name()
                    .unwrap_or(root.as_os_str())
                    .to_string_lossy()
            ),
        }
    }
}

/// How a switch ended.
#[derive(Debug)]
pub enum Apply {
    /// The new shell came up and the session was ended.
    Switched { logged_out: bool },
    /// A step failed. `rolled_back` says whether the old configuration is back.
    Failed {
        step: &'static str,
        detail: String,
        rolled_back: bool,
    },
}

/// One path that will exist under `$HOME` after the switch.
#[derive(Debug)]
struct Entry {
    /// Path relative to `$HOME`, e.g. `.config/hypr`.
    rel: String,
    src: PathBuf,
    origin: Origin,
}

/// A path that was renamed aside, so a rollback knows where to find it.
#[derive(Debug)]
struct Aside {
    /// Where it used to live, relative to `$HOME`.
    rel: String,
    /// Where the previous version was moved to, absolute.
    ///
    /// `None` means there was no previous version: the path did not exist before
    /// the switch. Rolling that back is not a rename, it is a removal, and getting
    /// it wrong leaves the new shell's configuration sitting on disk after a
    /// rollback that claimed to have undone it.
    kept: Option<PathBuf>,
}

/// How long the old shell gets to exit on its own before being killed.
const STOP_GRACE: Duration = Duration::from_secs(10);

/// How long the new shell gets to come up before the switch is called a failure.
///
/// dms.service declares TimeoutStartSec=90s, so this is not generous by accident.
const START_GRACE: Duration = Duration::from_secs(90);

/// How long the new shell gets to answer polkit before a privileged stage runs.
///
/// The shell that was running is stopped before this, and the one replacing it
/// registers its authentication agent only after it is up: DMS and end4-pC
/// bring up a PolkitService and Noctalia sets `polkit_agent = true`, both a
/// beat after start. Ask for a password while no agent exists and polkit falls
/// back to pkexec's textual prompt, which needs a controlling terminal — a
/// window does not have one, and the stage comes back as dismissed.
const AGENT_GRACE: Duration = Duration::from_secs(5);

/// How often a wait that can hold for a whole grace period says it is still
/// there.
///
/// These waits sit on `START_GRACE`, and a silent minute and a half reads as a
/// hung switch — it has been closed as one, which leaves the machine halfway
/// through: the old shell gone, and the rollback that never ran never bringing
/// it back. One line every ten seconds costs nothing and says the switch is
/// alive.
const HEARTBEAT: Duration = Duration::from_secs(10);

/// The three roots a switch reads and writes inside.
///
/// They travel together because they have to: staging has to sit on the same
/// filesystem as the destination for the swap to be a rename rather than a copy,
/// and the skel has to be findable relative to wherever the binary was run from.
pub struct Context {
    pub home: PathBuf,
    pub backups_root: PathBuf,
    pub skel: PathBuf,
}

/// Where a shell's skel lives.
///
/// In the source tree it is `src/compositor/<compositor>/shells/<shell>/skel`:
/// the templates sit inside the shell's own folder, so everything about one
/// shell on one compositor is in one place — the `PORT` that says where its
/// files go, and the files themselves. A package installs the same tree under
/// `<prefix>/share/rakuos-shell-picker/<compositor>/shells/<shell>/skel`. The
/// path is derived rather than registered, so a skel cannot end up somewhere
/// other than where the shell's code expects it.
pub fn skel_for(compositor_id: &str, shell_id: &str) -> PathBuf {
    skel_root()
        .join(compositor_id)
        .join("shells")
        .join(shell_id)
        .join("skel")
}

/// The directory that holds `<compositor>/shells/<shell>/skel`, resolved.
///
/// `env!("CARGO_MANIFEST_DIR")` is a build-time constant: it points at the tree
/// the binary was compiled in, which does not exist on the machine the package
/// was installed on. So the installed location — derived from the running
/// binary's own path, `<prefix>/share/rakuos-shell-picker` — is tried first, and
/// the build tree is only the fallback for dev and for tests. An explicit
/// override wins over both.
fn skel_root() -> PathBuf {
    if let Some(dir) = std::env::var_os("RAKUOS_SHELL_PICKER_SKEL_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(candidate) = exe
            .parent()
            .map(|bin| bin.join("..").join("share").join("rakuos-shell-picker"))
        {
            // Installed from a package: /usr/bin/rakuos-shell-picker resolves
            // to /usr/share/rakuos-shell-picker. Plain `cargo run` lands in
            // target/debug, where the sibling does not exist, so it falls
            // through to the source tree.
            if candidate.is_dir() {
                return candidate;
            }
        }
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/compositor")
}

/// Work out what a switch would do, and say it, without doing any of it.
///
/// A switch replaces whole directories and ends the session, so being able to see
/// the plan first is worth the twenty lines. Every path it would touch, where that
/// path comes from, how much of it there is, and whether anything is already there
/// to roll back to.
pub fn dry_run(
    target: &'static Shell,
    compositor: &'static dyn Compositor,
    ctx: &Context,
    progress: &mut dyn FnMut(&str),
) -> Result<(), String> {
    let Context {
        home,
        backups_root,
        skel,
    } = ctx;
    let (home, backups_root, skel) = (home.as_path(), backups_root.as_path(), skel.as_path());

    progress(&format!(
        "would switch {} to {}",
        compositor.label(),
        target.display_name
    ));
    progress(&format!("  skel: {}", skel.display()));
    progress(&format!("  backups: {}", backups_root.display()));

    let entries = plan(target, compositor, backups_root, skel, progress)?;
    if entries.is_empty() {
        return Err("nothing to install at all".to_string());
    }

    let stamp = crate::clock::stamp();
    let staging = stage(&entries, home, target.id, &stamp)?;

    progress("would install:");
    for entry in &entries {
        let (files, bytes) = measure(&staging.join(&entry.rel));
        let live = home.join(&entry.rel);
        let current = if live.exists() {
            let (f, b) = measure(&live);
            format!("yours now: {f} files, {b} bytes (kept aside, not deleted)")
        } else {
            "yours now: absent (removed again on rollback)".to_string()
        };
        progress(&format!(
            "  {} — {} from {}, {files} files, {bytes} bytes",
            entry.rel,
            if entry.origin == Origin::Skel {
                "skel"
            } else {
                "backup"
            },
            entry.origin.label(),
        ));
        progress(&format!("      {current}"));
    }
    progress(&format!(
        "staged everything to {} to check it arrived whole; removing it now",
        staging.display()
    ));
    let _ = std::fs::remove_dir_all(&staging);
    progress("nothing was stopped, replaced, started or logged out");
    Ok(())
}

/// The bus name the desktop notification service lives on.
///
/// Both Noctalia and DMS claim it, which is the whole reason a switch has to stop
/// the old shell before starting the new one, and the reason it has to be able to
/// prove the name changed hands rather than assume it.
pub const NOTIFICATION_BUS: &str = "org.freedesktop.Notifications";

/// The pid that currently owns [`NOTIFICATION_BUS`], if anyone does.
///
/// `None` covers both "nobody owns it" and "I could not tell", which are treated
/// the same on purpose: a probe that cannot answer must not be read as a pass.
fn notification_owner() -> Option<u32> {
    let out = run("busctl", &["--user", "status", NOTIFICATION_BUS]).ok()?;
    if !out.status.success() {
        return None;
    }
    // busctl prints its status report to stdout, as `PID=<n>` on its own line.
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .find_map(|line| line.strip_prefix("PID="))
        .and_then(|pid| pid.trim().parse().ok())
}

/// Put `target` in place, start it, and end the session.
///
/// `backup_root` is the backup that was just taken and verified. It is checked
/// again here rather than trusted, because the steps after it are the ones that
/// can be undone only by that directory still being there.
pub fn apply(
    target: &'static Shell,
    current: Option<&'static Shell>,
    compositor: &'static dyn Compositor,
    backup_root: &Path,
    ctx: &Context,
    progress: &mut dyn FnMut(&str),
) -> Apply {
    apply_with(
        target,
        current,
        compositor,
        backup_root,
        ctx,
        true,
        progress,
    )
}

/// [`apply`], with the last step left to the caller.
///
/// Ending the session is separated out because it is the one step that cannot be
/// undone from inside the process: once the session is gone, so is the panel that
/// would have reported the result. A caller that wants to check the new shell
/// first — a test, or a command-line run someone is watching — asks for it not to
/// log out, and says so rather than quietly leaving the session alone.
pub fn apply_with(
    target: &'static Shell,
    current: Option<&'static Shell>,
    compositor: &'static dyn Compositor,
    backup_root: &Path,
    ctx: &Context,
    log_out: bool,
    progress: &mut dyn FnMut(&str),
) -> Apply {
    let Context {
        home,
        backups_root,
        skel,
    } = ctx;
    let (home, backups_root, skel) = (home.as_path(), backups_root.as_path(), skel.as_path());
    let entries = match plan(target, compositor, backups_root, skel, progress) {
        Ok(entries) => entries,
        Err(detail) => {
            return Apply::Failed {
                step: "plan",
                detail,
                rolled_back: false,
            }
        }
    };

    let stamp = crate::clock::stamp();
    let staging = match stage(&entries, home, target.id, &stamp) {
        Ok(dir) => dir,
        Err(detail) => {
            let _ = std::fs::remove_dir_all(
                home.join(".local/share/rakuos-shell-picker")
                    .join(format!("staging-{}-{}", target.id, stamp)),
            );
            return Apply::Failed {
                step: "stage",
                detail,
                rolled_back: false,
            };
        }
    };
    progress(&format!(
        "staged {} path(s) in {}",
        entries.len(),
        staging.display()
    ));

    // The whole reason the earlier steps are safe is this directory. Re-check it
    // here so a backup that vanished between being written and being relied on
    // stops the switch instead of quietly removing the way back.
    if !backup_root.join("manifest.toml").is_file() {
        let _ = std::fs::remove_dir_all(&staging);
        return Apply::Failed {
            step: "backup",
            detail: format!(
                "the backup at {} is gone, so there would be no way back. Nothing was changed.",
                backup_root.display()
            ),
            rolled_back: false,
        };
    }
    progress(&format!(
        "backup still in place at {}",
        backup_root.display()
    ));

    // Read before the old shell is stopped. Once it exits the name is unowned, and
    // an unowned name cannot be compared against anything.
    let bus_owner_before = notification_owner();
    progress(&format!(
        "{NOTIFICATION_BUS} is owned by {}",
        bus_owner_before.map_or_else(|| "nobody".to_string(), |p| p.to_string())
    ));

    if let Some(shell) = current {
        if let Err(detail) = stop(shell, home, progress) {
            let _ = std::fs::remove_dir_all(&staging);
            return Apply::Failed {
                step: "stop",
                detail,
                rolled_back: false,
            };
        }
        refresh_backup(
            shell,
            target,
            home,
            backups_root,
            backup_root,
            compositor,
            progress,
        );
    }

    let asides = match swap(&entries, &staging, home, target.id, &stamp, progress) {
        Ok(asides) => asides,
        Err((detail, asides)) => {
            // The old shell was stopped before the swap, so putting the files back
            // is only half of the undo. Leaving the session with no shell at all is
            // worse than the failure that caused it.
            progress("putting the previous configuration back");
            let configs_back = restore(&asides, home);
            let old_up = resume(current, home, progress);
            let _ = std::fs::remove_dir_all(&staging);
            return Apply::Failed {
                step: "swap",
                detail,
                rolled_back: configs_back && old_up,
            };
        }
    };
    let _ = std::fs::remove_dir_all(&staging);

    if let Err((step, detail)) =
        start(target, home, progress).and_then(|()| verify(target, bus_owner_before, progress))
    {
        progress(&format!("{step} did not work: {detail}"));
        // Stop it before touching any files. It may already own the notification
        // bus, and starting the old shell back up alongside it means one of the
        // two fails to own it — so the rollback would produce two half-started
        // shells instead of the one working shell the user had before.
        let stopped = stop(target, home, progress);
        progress("putting the previous configuration back");
        let configs_back = restore(&asides, home);
        let old_up = resume(current, home, progress);
        return Apply::Failed {
            step,
            detail,
            rolled_back: stopped.is_ok() && configs_back && old_up,
        };
    }
    if let Some(shell) = current {
        progress(&format!(
            "{} is up; {} is stopped",
            target.display_name, shell.display_name
        ));
    } else {
        progress(&format!("{} is up", target.display_name));
    }

    // The new shell is up, so nothing can roll back to the copies this switch
    // renamed aside: `restore` only runs from the failure paths above. They are
    // dropped here rather than left behind, which is what a day of switching
    // used to do — one generation per attempt, and then the next backup copied
    // every one of them along with the user's config.
    let (dropped, held) = discard_asides(&asides, backups_root);
    if dropped > 0 {
        progress(&format!(
            "removed {dropped} of the copies this switch set aside; each is still in {}",
            backup_root.display()
        ));
    }
    if held > 0 {
        progress(&format!(
            "kept {held} of the copies set aside, which no backup holds"
        ));
    }

    // The shell just left goes, now that nothing can roll back to it: the new
    // one is up, and the backup written before the switch is the way home. Both
    // steps are reported rather than fatal — a switch that has reached here has
    // succeeded, and a file or a package that would not go away leaves the
    // machine dirtier than asked, not broken.
    if let Some(outgoing) = current {
        let (cleaned, held) = clean(outgoing, target, home, backups_root, backup_root, progress);
        if cleaned > 0 {
            progress(&format!(
                "cleaned {cleaned} path(s) belonging to {}",
                outgoing.display_name
            ));
        }
        if held > 0 {
            progress(&format!("kept {held} path(s) of {}", outgoing.display_name));
        }

        if let Some(plan) = outgoing.uninstall_plan {
            // Everything that has to happen before a password can be asked for.
            // The compositor is still running the configuration this switch
            // replaced, so it is reloaded first and the new shell's rules are
            // the ones in effect. Then the new shell must have a polkit agent:
            // shells that answer polkit themselves (DMS, Noctalia, end4-pC)
            // register one a beat after start, and the ones that lean on
            // `mate-polkit` (Caelestia, Ambxst) never started it, because a
            // reload does not run their `exec-once` line. So start that agent if
            // the shell names one, then give whatever is up time to register —
            // without an agent, `pkexec` is answered by nobody and falls back to
            // a textual prompt a window cannot show.
            match compositor.reload() {
                Ok(()) => progress(&format!("reloaded {}'s configuration", compositor.label())),
                Err(detail) => progress(&format!(
                    "could not reload {}'s configuration: {detail}",
                    compositor.label()
                )),
            }
            ensure_polkit_agent(target, progress);
            progress(&format!(
                "waiting {} seconds for {} to be ready to answer polkit",
                AGENT_GRACE.as_secs(),
                target.display_name
            ));
            std::thread::sleep(AGENT_GRACE);
            progress(&format!("removing the {} packages", outgoing.display_name));
            match crate::switch::uninstall(plan, target.runtime_packages, progress) {
                crate::switch::Removed::Finished { stages, .. } => progress(&format!(
                    "{}'s packages are off the machine ({stages} stage(s))",
                    outgoing.display_name
                )),
                crate::switch::Removed::AlreadyGone { .. } => {
                    progress("its packages were already gone");
                }
                crate::switch::Removed::Kept { .. } => {
                    progress(&format!(
                        "{} has nothing this switch may remove: what it was \
                         installed with is either shared with {} or part of the \
                         base image, so nothing was removed",
                        outgoing.display_name, target.display_name
                    ));
                }
                crate::switch::Removed::Dismissed { stage } => progress(&format!(
                    "stage {stage} was not authorized, so {} stays installed",
                    outgoing.display_name
                )),
                crate::switch::Removed::Failed { stage, status, .. } => {
                    let line = format!(
                        "stage {stage} could not remove {}: {status}",
                        outgoing.display_name
                    );
                    progress(&line);
                }
                crate::switch::Removed::Malformed { package } => {
                    let line = format!(
                        "the removal plan for {package} names no packages, so nothing was removed"
                    );
                    progress(&line);
                }
            }
        }
    }

    if !log_out {
        progress(&format!(
            "leaving the session alone as asked; log out yourself to start {} for real",
            target.display_name
        ));
        return Apply::Switched { logged_out: false };
    }

    match logout(compositor, progress) {
        Ok(()) => Apply::Switched { logged_out: true },
        Err(detail) => {
            // The switch itself worked. Only the clean restart did not, which is
            // not a reason to tear the new configuration back out.
            progress(&format!(
                "{detail} — {} is running now; log out yourself when convenient.",
                target.display_name
            ));
            Apply::Switched { logged_out: false }
        }
    }
}

/// Work out where each path comes from.
///
/// The rule is "the backup kept for this shell, skel as the fallback", and it is
/// right for a shell's own configuration for two reasons at once: if this machine
/// has run DMS before, that backup holds how the user had it arranged — and it
/// holds *DMS's* era of the shared files too, which is what makes it safe to
/// restore `gtk.css` from it. The newest backup overall would be the one taken
/// when the shell currently in charge left, and its `gtk.css` still imports the
/// stylesheet of a shell that is not running any more.
///
/// Hence `backups_for` rather than a search for any backup holding the path: the
/// directory name says which shell was in charge, and only that shell's era of a
/// file belongs on the way back to it. A shell that has never been on this
/// machine has no backup of its own, and then the skel is the answer — a fresh
/// install on defaults, not a file borrowed from somebody else's era.
///
/// It is the wrong rule for the compositor's configuration, and applying it there
/// would quietly undo the switch. `~/.config/hypr` is in every backup, because
/// every switch saves it — so even a backup of the right shell holds a config
/// from whenever it was taken, while that file is where the shell is started
/// from. It has to come from the target's own skel.
fn plan(
    target: &Shell,
    compositor: &dyn Compositor,
    backups_root: &Path,
    skel: &Path,
    progress: &mut dyn FnMut(&str),
) -> Result<Vec<Entry>, String> {
    let mut entries = Vec::new();
    let own = backups_for(backups_root, compositor.id(), target.id);

    if let Some(rel) = target.config_dir {
        pull_into(
            &mut entries,
            rel,
            &own,
            skel,
            false,
            &format!("{} will start on its own defaults", target.display_name),
            progress,
        );
    }

    // State is the same question as configuration, and it has to be asked the
    // same way: a switch that removes the outgoing shell's state directory on
    // its way out can only do that because the backup is about to be the way
    // back. Without this the state would be saved and never read — which is
    // exactly what happened while nothing deleted it, since the files simply
    // stayed on disk.
    if let Some(rel) = target.state_dir {
        pull_into(
            &mut entries,
            rel,
            &own,
            skel,
            true,
            &format!("{} starts with no state carried over", target.display_name),
            progress,
        );
    }

    // What the companions left behind when this shell was last in charge. No
    // skel carries it — nothing installs a calendar database — so the backup is
    // the only source there is, and "the skel does not have it" is the ordinary
    // answer on a machine that has never run this shell. Said out loud, because
    // a missing calendar is not something the user should have to discover.
    for rel in target.companion_paths {
        pull_into(
            &mut entries,
            rel,
            &own,
            skel,
            true,
            &format!("{} starts without it", target.display_name),
            progress,
        );
    }

    // The rest of what the skel carries: themes, runtime state, and the launcher
    // entries. A path listed here may be a single file, which is what lets one
    // shell replace the other's `.desktop` without either of them owning the
    // directory it lives in.
    //
    // The backup of this shell still wins over the skel, and that is now the
    // point rather than an accident: these are the files two shells share and
    // both install, so the backup of the right shell is the only copy that
    // matches the shell coming back. A backup that does not hold one — the path
    // was added later, or the directory was empty that day — falls through to the
    // skel, which is what "skel as the fallback" means.
    for rel in target.skel_extra_paths {
        pull_into(
            &mut entries,
            rel,
            &own,
            skel,
            true,
            "nothing to install, and the switch goes on",
            progress,
        );
    }

    // Paths a switch writes that nothing else would have backed up. Usually the
    // same launcher entry as above, which the dedup above already took care of;
    // listed separately because "being installed" and "needing to be kept" are
    // not the same question.
    for rel in target.backup_extra_paths {
        pull_into(
            &mut entries,
            rel,
            &own,
            skel,
            true,
            "nothing to install, and the switch goes on",
            progress,
        );
    }

    let comp_rel = compositor.config_dir().trim_start_matches("./").to_string();
    let comp_src = skel.join(&comp_rel);
    if !comp_src.is_dir() {
        // Fatal, and deliberately so. This file is what starts the shell. If it
        // is missing then "switched" would mean nothing happened, the old config
        // would still launch the old shell, and the logout would land the user
        // right back where they started with a success message on screen.
        return Err(format!(
            "the skel has no {comp_rel} to install, so there is nothing to switch to; \
             refusing to leave your current configuration in place and call it a switch"
        ));
    }
    progress(&format!("{comp_rel} comes from the skel, not a backup"));
    entries.push(Entry {
        rel: comp_rel,
        src: comp_src,
        origin: Origin::Skel,
    });

    Ok(entries)
}

/// Pull one path through [`pull`] and add it, unless the plan already has it.
///
/// Deduplicated by path because the same file is legitimately listed twice — the
/// launcher entry is in `skel_extra_paths` because a switch installs it and in
/// `backup_extra_paths` because nothing else would save it — and the swap would
/// otherwise stage and replace it twice from two sources that may disagree.
fn pull_into(
    entries: &mut Vec<Entry>,
    rel: &str,
    own: &[PathBuf],
    skel: &Path,
    allow_file: bool,
    empty: &str,
    progress: &mut dyn FnMut(&str),
) {
    if entries.iter().any(|e| e.rel == rel) {
        return;
    }
    if let Some(entry) = pull(rel, own, skel, allow_file, empty, progress) {
        entries.push(entry);
    }
}

/// Where one of a shell's own paths comes from.
///
/// The backups kept for this shell first, and the skel as the fallback, which is
/// the right answer for anything belonging to a shell rather than to the session:
/// if this machine has run that shell before, its backup holds how the user had
/// it arranged, and that is what should come back. `own` is filtered by
/// [`backups_for`] before it gets here, so a path this shell never had falls
/// through to the skel instead of picking up another shell's version of it — the
/// difference between `@import url("noctalia.css")` and a stylesheet no shell on
/// the machine writes any more. The compositor's configuration is the one
/// exception and is handled by name in [`plan`] instead.
///
/// `allow_file` is for paths whose skel copy may be a single file rather than a
/// directory; a state path is usually a directory and a configuration path
/// always is, so the strict `is_dir` check stands for the latter. `empty` is
/// what to say when neither holds the path — a legitimate outcome, never fatal
/// here, because the fatal case is the compositor's own configuration below.
fn pull(
    rel: &str,
    own: &[PathBuf],
    skel: &Path,
    allow_file: bool,
    empty: &str,
    progress: &mut dyn FnMut(&str),
) -> Option<Entry> {
    let rel = rel.trim_start_matches("./").to_string();
    match own.iter().find(|d| d.join("home").join(&rel).exists()) {
        Some(root) => {
            progress(&format!(
                "{rel} comes from {}, the backup kept for this shell",
                root.file_name().unwrap_or_default().to_string_lossy()
            ));
            let src = root.join("home").join(&rel);
            Some(Entry {
                rel,
                src,
                origin: Origin::Backup(root.clone()),
            })
        }
        // A symlink counts even when its target does not exist yet: `copy`
        // puts the link itself in place, and the package it points at comes
        // from the install stages that run before this plan reads the skel —
        // end4-pC's config dir is exactly that link.
        None if skel.join(&rel).is_dir()
            || skel.join(&rel).is_symlink()
            || (allow_file && skel.join(&rel).is_file()) =>
        {
            progress(&format!(
                "{rel} comes from the skel, not a backup; no backup kept for this \
                 shell holds it"
            ));
            let src = skel.join(&rel);
            Some(Entry {
                rel,
                src,
                origin: Origin::Skel,
            })
        }
        None => {
            progress(&format!("no backup or skel holds {rel}; {empty}"));
            None
        }
    }
}

/// Every backup that was taken while `shell_id` was the shell on this machine,
/// newest first.
///
/// Backup directories are named `<yyyymmdd>-<compositor>-<shell being left
/// behind>`, and a backup only ever describes the desktop as it was with that
/// shell in charge — a switch saves the shell it is leaving, and the shared paths
/// inside it are that shell's version of them. Filtering by name is therefore how
/// a path belonging to one shell gets the copy of *that* shell rather than the
/// copy of whichever shell happened to leave most recently: the newest backup
/// overall for `gtk.css` after a switch away from DMS is DMS's `@import`, which
/// is the one import Noctalia does not want.
///
/// `compositor` is part of the name for the same reason as `shell_id`: two
/// compositors would each have their own copy of the same shell's desktop.
///
/// The order is by name descending, and the name starts with the day it was
/// taken, so the newest is first. Two backups cannot share a name — a second one
/// replaces the first — so there is nothing else to break the tie.
///
/// A backup taken with nothing running is named `…-<comp>-none` and is excluded
/// by construction: it holds the previous shell's era only by accident, and it
/// has no era of its own to give back.
fn backups_for(backups_root: &Path, compositor_id: &str, shell_id: &str) -> Vec<PathBuf> {
    let suffix = format!("-{compositor_id}-{shell_id}");
    let mut dirs: Vec<PathBuf> = list_backups(backups_root)
        .into_iter()
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(&suffix))
        })
        .collect();
    dirs.sort();
    dirs.reverse();
    dirs
}

/// Every backup holding `rel`, newest first, whichever shell it belongs to.
///
/// Only [`discard_asides`] uses this: a copy a switch renamed aside has to be
/// reproduced byte for byte before it may be dropped, and *some* backup doing so
/// is the whole test — which backup it is says nothing about the copy, and
/// insisting on the newest one was leaving `.pre-*` files behind on disk that the
/// switch had already verified against an older directory.
fn backups_holding(backups_root: &Path, rel: &str) -> Vec<PathBuf> {
    let mut dirs = list_backups(backups_root);
    dirs.sort();
    dirs.reverse();
    dirs.into_iter()
        .filter(|d| d.join("home").join(rel).exists())
        .collect()
}

/// The backup directories under `backups_root`, skipping hidden ones.
///
/// Hidden because a copy still being built, or a previous one left aside by an
/// interrupted replace, is not a backup anything should be sourcing files from.
fn list_backups(backups_root: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(backups_root)
        .ok()
        .map(|entries| {
            entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .filter(|p| {
                    !p.file_name()
                        .is_some_and(|n| n.to_string_lossy().starts_with('.'))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Copy every entry into a staging directory and count what landed.
///
/// The staging directory is under `$HOME` rather than `/tmp` on purpose: the swap
/// that follows is a rename, and a rename cannot cross a filesystem. A `/tmp`
/// that happens to be tmpfs would quietly turn the atomic step into a copy.
fn stage(entries: &[Entry], home: &Path, target_id: &str, stamp: &str) -> Result<PathBuf, String> {
    let root = home
        .join(".local/share/rakuos-shell-picker")
        .join(format!("staging-{target_id}-{stamp}"));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).map_err(|e| format!("cannot create {}: {e}", root.display()))?;

    for entry in entries {
        let to = root.join(&entry.rel);
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
        }
        let (src_files, src_bytes) = measure(&entry.src);
        copy(&entry.src, &to).map_err(|e| format!("cannot stage {}: {e}", entry.rel))?;
        let (dst_files, dst_bytes) = measure(&to);
        if (src_files, src_bytes) != (dst_files, dst_bytes) {
            return Err(format!(
                "{} staged short: {dst_files}/{} files, {dst_bytes}/{} bytes",
                entry.rel, src_files, src_bytes
            ));
        }
    }
    Ok(root)
}

/// How [`stop`] takes a shell down.
///
/// The split mirrors [`StartMechanism`](crate::shell::StartMechanism), because
/// a shell is stopped the same way it was started. The two `start` branches
/// exist for a reason — a shell systemd supervises is not a bare process — and
/// that reason does not stop applying once the shell is on its way out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StopPlan {
    /// The shell runs under a systemd user unit, named as in
    /// [`Shell::systemd_user_unit`](crate::shell::Shell::systemd_user_unit).
    Unit(&'static str),
    /// The shell is a plain process and is taken down through its pids.
    Signals,
}

/// Which instruction [`stop`] sends, decided from the registry alone.
///
/// Pure and separate so the answer can be asserted without a shell to stop:
/// signalling a running desktop from a test is not something the suite can do,
/// and the cost of getting this wrong is a shell that comes back.
fn stop_plan(shell: &Shell) -> Result<StopPlan, String> {
    if shell.mechanism.uses_systemd_unit() {
        let unit = shell
            .systemd_user_unit
            .ok_or_else(|| format!("{} claims a systemd unit but names none", shell.id))?;
        Ok(StopPlan::Unit(unit))
    } else {
        Ok(StopPlan::Signals)
    }
}

/// Stop a shell, politely and then not.
///
/// The branch comes from [`stop_plan`]. A shell systemd owns is stopped through
/// its unit, for the mirror of the reason [`start`] talks to systemd: signalling
/// the process alone leaves the unit itself untouched and outside anything this
/// program can see, so what reports "stopped" and what the session believes
/// would be two different questions. A shell that is a plain process is signalled
/// instead — SIGTERM first because a shell that exits cleanly gets to write its
/// state, with the kill as a fallback rather than the plan, and a loop that waits
/// to see whether the first signal was enough rather than assuming either way.
///
/// The shell's companion units come down after it rather than before. They exist
/// for the shell's benefit, so they are of no use once it is gone, and taking
/// them first would visibly break the widgets they supply while the shell that
/// draws them was still running. They are reported rather than fatal: the shell
/// itself is down by then, and failing a switch over a helper that would not
/// stop would leave the session with no shell at all — the one outcome worse
/// than a helper still sitting there.
fn stop(shell: &Shell, home: &Path, progress: &mut dyn FnMut(&str)) -> Result<(), String> {
    match stop_plan(shell)? {
        StopPlan::Unit(unit) => stop_unit(shell, home, unit, progress)?,
        StopPlan::Signals => stop_signals(shell, progress)?,
    }
    stop_companions(shell, progress);
    Ok(())
}

/// Stop the shell by asking systemd, then confirm the process really went.
///
/// `systemctl stop` is synchronous and waits for the unit's cgroup to empty, so
/// the pid check after it is not paranoia but the answer to the question this
/// branch exists for: the unit saying it stopped and the shell being gone are
/// the same claim, and only one of them is observed rather than reported.
fn stop_unit(
    shell: &Shell,
    home: &Path,
    unit: &'static str,
    progress: &mut dyn FnMut(&str),
) -> Result<(), String> {
    let name = format!("{unit}.service");
    let before = pids_of(shell.binary, shell.process_match);
    if before.is_empty() {
        progress(&format!("{} is not running", shell.display_name));
    } else {
        progress(&format!(
            "stopping {} ({name}, pid {})",
            shell.display_name,
            before
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    let out = run("systemctl", &["--user", "stop", &name])
        .map_err(|e| format!("cannot run systemctl: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "{name} refused to stop: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }

    let left = pids_of(shell.binary, shell.process_match);
    if !left.is_empty() {
        return Err(format!(
            "{} is still running (pid {}) after {name} reported stopped",
            shell.display_name,
            left.iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !before.is_empty() {
        progress(&format!("{} stopped", shell.display_name));
    }

    // Stopping is not the whole job. `systemctl stop` ends the unit for this
    // session but leaves behind whatever enablement would start it again at the
    // next login: the image's own global enable on Niri, and on Hyprland the
    // enable a switch wrote when DMS came in. Disabling and masking closes both
    // doors, and the daemons that autostart with the shell — `dsearch` for DMS,
    // see [`Shell::autostart_systemd_user_units`] — are stopped and held down
    // the same way. `unmask_and_enable` in [`start`] is the exact undo.
    //
    // A unit that was enabled by a wants link instead — see
    // [`Port::systemd_user_unit_wanted_by`] — has no `[Install]` for `disable`
    // to undo, and takes the link away instead. Each branch is the exact mirror
    // of the one `start` takes for the same shell.
    match shell.systemd_user_unit_wanted_by {
        Some(_) => disable_via_wants(home, unit, progress),
        None => disable_and_mask(unit, progress),
    }
    for companion in shell.autostart_systemd_user_units {
        stop_and_mask(companion, progress);
    }
    Ok(())
}

/// Disable and mask a unit the image enables, so a login cannot start it.
///
/// `disable` undoes as much of the enablement as this user may undo — the
/// image's own symlink under `/etc/systemd/user` is root-owned and stays — and
/// `mask`, which shadows the unit per user, is what actually holds it down. Both
/// are reported as lines rather than returned: the shell is stopped by the time
/// this runs, so a failure here is dirt to mention, not a reason to abandon a
/// switch that has otherwise worked.
fn disable_and_mask(unit: &str, progress: &mut dyn FnMut(&str)) {
    let name = format!("{unit}.service");
    for (verb, done) in [("disable", "disabled"), ("mask", "masked")] {
        match run("systemctl", &["--user", verb, &name]) {
            Ok(out) if out.status.success() => progress(&format!("{done} {name}")),
            Ok(out) => progress(&format!(
                "cannot {verb} {name}: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )),
            Err(err) => progress(&format!("cannot run systemctl to {verb} {name}: {err}")),
        }
    }
}

/// Stop a unit that autostarts with a shell on its way out, then hold it down.
///
/// The shell's own unit is stopped by [`stop_unit`], which confirms the process
/// really went; a daemon the shell declares has no process this program looks
/// for, so it is stopped through systemd and the answer taken at face value.
/// Every step is a line rather than an error, for the reason in [`stop`]: the
/// shell is already down, and stranding the session over a helper that would
/// not stop is worse than saying so.
fn stop_and_mask(unit: &str, progress: &mut dyn FnMut(&str)) {
    let name = format!("{unit}.service");
    match run("systemctl", &["--user", "stop", &name]) {
        Ok(out) if out.status.success() => progress(&format!("stopped {name}")),
        Ok(out) => progress(&format!(
            "cannot stop {name}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )),
        Err(err) => progress(&format!("cannot run systemctl to stop {name}: {err}")),
    }
    disable_and_mask(unit, progress);
}

/// Undo [`disable_and_mask`] so a unit the image enables can start again.
///
/// `unmask` has to come first: while a unit is masked, `enable` still records the
/// enablement but a `start` of it would fail. The pair is deliberately the mirror
/// of [`disable_and_mask`], so a switch to the shell is a switch away from it run
/// backwards, with nothing one-way.
fn unmask_and_enable(unit: &str, progress: &mut dyn FnMut(&str)) {
    let name = format!("{unit}.service");
    for (verb, done) in [("unmask", "unmasked"), ("enable", "enabled")] {
        match run("systemctl", &["--user", verb, &name]) {
            Ok(out) if out.status.success() => progress(&format!("{done} {name}")),
            Ok(out) => progress(&format!(
                "cannot {verb} {name}: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )),
            Err(err) => progress(&format!("cannot run systemctl to {verb} {name}: {err}")),
        }
    }
}

/// Enable a unit that ships no `[Install]` section, by writing the wants link
/// `systemctl --user enable` would have written itself.
///
/// [`Port::systemd_user_unit_wanted_by`] names the unit whose `.wants/`
/// directory is the right one — the compositor's own service — and upstream iNiR
/// does exactly this by hand, which is why its unit carries no `[Install]` to
/// begin with. `enable` refuses such a unit, so the link has to be written here:
/// without it the switch's work is invisible at the next login, and the machine
/// boots with no shell at all.
///
/// Every step is reported as a line rather than returned, the same shape as
/// [`disable_and_mask`]: the shell is started by the caller right after this, and
/// a link that could not be written is worth saying out loud, not worth refusing
/// a switch over.
fn enable_via_wants(
    home: &Path,
    target: &str,
    unit: &str,
    progress: &mut dyn FnMut(&str),
) {
    let name = format!("{unit}.service");

    // `unmask` first, for the reason in [`unmask_and_enable`]: a mask shadows
    // everything under it, including a link written a moment later. Nothing here
    // masks this unit, but an older picker did and a user may have done it by
    // hand.
    match run("systemctl", &["--user", "unmask", &name]) {
        Ok(out) if out.status.success() => progress(&format!("unmasked {name}")),
        Ok(out) => progress(&format!(
            "cannot unmask {name}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )),
        Err(err) => progress(&format!("cannot run systemctl to unmask {name}: {err}")),
    }

    let user_dir = systemd_user_dir(home);
    let wants_dir = user_dir.join(format!("{target}.wants"));

    // One enablement at a time. A link under another target would start the unit
    // a second time at login — or under a compositor that is not the one running
    // — and a stale one is what upstream's `cleanup_stale_wants_links` sweeps up.
    drop_wants_links(&user_dir, &name, Some(&format!("{target}.wants")), progress);

    if let Err(err) = std::fs::create_dir_all(&wants_dir) {
        progress(&format!("cannot create {}: {err}", wants_dir.display()));
        return;
    }

    // What the link points at, asked rather than guessed: the unit lives under
    // `/usr/lib/systemd/user` today, and the same command is what upstream's
    // launcher uses for this symlink, so the answer follows the package.
    let link_target = unit_fragment_path(&name).unwrap_or_else(|| user_dir.join(&name));
    let link = wants_dir.join(&name);
    // The link is ours to replace, so it goes before the new one is made rather
    // than through `-f`: `symlink` refuses an existing path, and a stale link to
    // a path that no longer exists is exactly the case this has to survive.
    let _ = std::fs::remove_file(&link);
    match std::os::unix::fs::symlink(&link_target, &link) {
        Ok(()) => progress(&format!(
            "enabled {name} into {target} ({})",
            link_target.display()
        )),
        Err(err) => progress(&format!(
            "cannot link {} to {}: {err}",
            link.display(),
            link_target.display()
        )),
    }

    daemon_reload(progress);
}

/// Undo [`enable_via_wants`] by taking the link away, and nothing else.
///
/// Deliberately not [`disable_and_mask`]. `disable` is the verb that cannot work
/// here — there is no `[Install]` for it to undo, which is the whole reason the
/// link exists — and a mask would hold the unit down after upstream's own
/// `inir service enable` wrote the link back, leaving a shell that never starts
/// and no error to explain it. The link is the entire enablement, so removing
/// every one of them is the entire undo.
fn disable_via_wants(home: &Path, unit: &str, progress: &mut dyn FnMut(&str)) {
    let name = format!("{unit}.service");
    // No `keep`: a link under any target is one that would start this unit
    // without the picker having chosen it.
    drop_wants_links(&systemd_user_dir(home), &name, None, progress);
    daemon_reload(progress);
}

/// The `~/.config/systemd/user` directory a wants link belongs in.
fn systemd_user_dir(home: &Path) -> PathBuf {
    home.join(".config/systemd/user")
}

/// Remove every `*.wants/<name>` link under `user_dir` except, when one is named,
/// the link under `keep`.
///
/// The `*.wants` directories are created by `enable` and by whoever wrote a link
/// by hand, so they are read rather than listed. A broken link counts: it is a
/// link, and it is what `systemctl` would try to start.
fn drop_wants_links(
    user_dir: &Path,
    name: &str,
    keep: Option<&str>,
    progress: &mut dyn FnMut(&str),
) {
    let Ok(entries) = std::fs::read_dir(user_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let dir = entry.path();
        let dir_name = entry.file_name();
        let Some(dir_name) = dir_name.to_str() else {
            continue;
        };
        if !dir_name.ends_with(".wants") || Some(dir_name) == keep || !dir.is_dir() {
            continue;
        }
        let link = dir.join(name);
        if std::fs::symlink_metadata(&link).is_err() {
            continue;
        }
        match std::fs::remove_file(&link) {
            Ok(()) => progress(&format!("removed {}", link.display())),
            Err(err) => progress(&format!("cannot remove {}: {err}", link.display())),
        }
    }
}

/// Where systemd would run a user unit from, or `None` when it cannot say.
fn unit_fragment_path(name: &str) -> Option<PathBuf> {
    let out = run(
        "systemctl",
        &["--user", "show", "-p", "FragmentPath", "--value", name],
    )
    .ok()?;
    if !out.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if path.is_empty() {
        None
    } else {
        Some(PathBuf::from(path))
    }
}

/// Ask systemd to re-read the user unit directory, after a link changed.
///
/// A link written or removed under `~/.config/systemd/user` is invisible until
/// this runs, which is why upstream's launcher does the same after its own
/// symlink. A failure is a line, not an error: it costs the link its effect until
/// the next reload, and the login that follows reloads anyway.
fn daemon_reload(progress: &mut dyn FnMut(&str)) {
    match run("systemctl", &["--user", "daemon-reload"]) {
        Ok(out) if out.status.success() => {}
        Ok(out) => progress(&format!(
            "cannot reload systemd: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )),
        Err(err) => progress(&format!("cannot run systemctl to reload: {err}")),
    }
}

/// Take the helpers the shell declared down with it.
///
/// Every failure is a line rather than an error, for the reason in [`stop`]: by
/// the time this runs the shell is already gone, so the switch cannot retreat
/// to a working session by refusing here. What it can do is say so, because a
/// helper left running after its shell has been uninstalled is dirt the user
/// should be told about rather than discover later.
fn stop_companions(shell: &Shell, progress: &mut dyn FnMut(&str)) {
    for unit in shell.companion_systemd_user_units {
        let name = format!("{unit}.service");
        let line = match run("systemctl", &["--user", "stop", &name]) {
            Ok(out) if out.status.success() => {
                progress(&format!("stopped {name}"));
                continue;
            }
            Ok(out) => format!(
                "cannot stop {name}: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ),
            Err(err) => format!("cannot run systemctl to stop {name}: {err}"),
        };
        progress(&line);
    }
}

/// Stop a shell that systemd does not supervise, politely and then not.
///
/// SIGTERM first because a shell that exits cleanly gets to write its state. The
/// kill is a fallback, not the plan, and the loop waits to see whether the first
/// signal was enough rather than assuming either way.
fn stop_signals(shell: &Shell, progress: &mut dyn FnMut(&str)) -> Result<(), String> {
    let pids = pids_of(shell.binary, shell.process_match);
    if pids.is_empty() {
        progress(&format!("{} is not running", shell.display_name));
        return Ok(());
    }
    progress(&format!(
        "stopping {} (pid {})",
        shell.display_name,
        pids.iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ));

    signal(&pids, "-TERM").map_err(|e| format!("cannot signal {}: {e}", shell.binary))?;

    let deadline = Instant::now() + STOP_GRACE;
    loop {
        let left = pids_of(shell.binary, shell.process_match);
        if left.is_empty() {
            progress(&format!("{} stopped", shell.display_name));
            return Ok(());
        }
        if Instant::now() >= deadline {
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }

    progress(&format!(
        "{} did not exit in {}s, killing it",
        shell.display_name,
        STOP_GRACE.as_secs()
    ));
    let left = pids_of(shell.binary, shell.process_match);
    signal(&left, "-KILL").map_err(|e| format!("cannot kill {}: {e}", shell.binary))?;

    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if pids_of(shell.binary, shell.process_match).is_empty() {
            progress(&format!("{} killed", shell.display_name));
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    Err(format!(
        "{} is still running (pid {}) and could not be stopped",
        shell.display_name,
        pids_of(shell.binary, shell.process_match)
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

/// Move the staged paths into place, renaming whatever was there aside.
///
/// Both halves are renames within one filesystem, so neither can be interrupted
/// halfway. If the second one fails the first is undone before returning, which
/// is why the error carries the asides it managed to create.
fn swap(
    entries: &[Entry],
    staging: &Path,
    home: &Path,
    target_id: &str,
    stamp: &str,
    progress: &mut dyn FnMut(&str),
) -> Result<Vec<Aside>, (String, Vec<Aside>)> {
    let mut asides = Vec::new();

    for entry in entries {
        let from = staging.join(&entry.rel);
        let to = home.join(&entry.rel);
        let parent = match to.parent() {
            Some(p) => p,
            None => return Err((format!("{} has no parent directory", to.display()), asides)),
        };
        if let Err(e) = std::fs::create_dir_all(parent) {
            return Err((format!("cannot create {}: {e}", parent.display()), asides));
        }

        let kept = if to.exists() {
            let kept = parent.join(format!(
                ".{}.pre-{target_id}-{stamp}",
                entry.rel.rsplit('/').next().unwrap_or(&entry.rel)
            ));
            if let Err(e) = std::fs::rename(&to, &kept) {
                return Err((format!("cannot move your {} aside: {e}", entry.rel), asides));
            }
            Some(kept)
        } else {
            None
        };

        // Recorded before the second rename, so a failure there is undone by the
        // same restore path as everything else rather than by a special case.
        let aside = Aside {
            rel: entry.rel.clone(),
            kept: kept.clone(),
        };

        if let Err(e) = std::fs::rename(&from, &to) {
            asides.push(aside);
            return Err((format!("cannot put {} in place: {e}", entry.rel), asides));
        }

        match &kept {
            Some(k) => progress(&format!(
                "{} replaced from the {}; the old one is at {}",
                entry.rel,
                entry.origin.label(),
                k.display()
            )),
            None => progress(&format!(
                "{} created from the {} (there was no previous one)",
                entry.rel,
                entry.origin.label()
            )),
        }
        asides.push(aside);
    }

    Ok(asides)
}

/// Put the renamed-aside paths back.
fn restore(asides: &[Aside], home: &Path) -> bool {
    let mut ok = true;
    for aside in asides {
        let to = home.join(&aside.rel);
        match &aside.kept {
            Some(kept) => {
                if to.exists() {
                    let _ = std::fs::remove_dir_all(&to);
                }
                match std::fs::rename(kept, &to) {
                    Ok(()) => crate::logging::info(&format!("restored {}", aside.rel)),
                    Err(e) => {
                        crate::logging::error(&format!("cannot restore {}: {e}", aside.rel));
                        ok = false;
                    }
                }
            }
            None => {
                // Nothing was here before, so the switch put it here. Leaving it
                // behind means the old shell comes back reading a configuration
                // that was written for the new one.
                if to.exists() {
                    match std::fs::remove_dir_all(&to) {
                        Ok(()) => crate::logging::info(&format!("removed {}", aside.rel)),
                        Err(e) => {
                            crate::logging::error(&format!(
                                "cannot remove the {} this switch created: {e}",
                                aside.rel
                            ));
                            ok = false;
                        }
                    }
                }
            }
        }
    }
    ok
}

/// Throw away the rollback copies a finished switch renamed aside.
///
/// They exist for exactly one thing: the rollback that a failed `start` or
/// `verify` would have triggered. Both have succeeded by the time this runs, so
/// no path in the code can read them again — undoing the switch later reads the
/// backup instead. Leaving them on disk costs a file per path per switch, and
/// because most of them live inside a directory that is itself backed up, each
/// generation was being copied into the next backup too.
///
/// "The backup already holds this" is checked, never assumed: the aside is only
/// dropped when **some** backup contains that exact path with the same number of
/// files and the same number of bytes, and which backup that is does not matter —
/// the question is whether the bytes are safe somewhere, not where. Checking only
/// the newest one was the bug this replaced: after a switch the newest directory
/// belongs to the shell just left, and an aside taken from the shell before it
/// matched an older one instead, so six verified copies were left on disk as
/// `.pre-*` clutter that nothing would ever read again.
///
/// An aside no backup holds is kept, because after the swap it may be the only
/// copy left of what was there. [`crate::switch::backup_sources_for`] makes that
/// the rare case rather than the rule — the target's own config and state are
/// copied with the outgoing shell's, so an aside taken from the shell being
/// switched *to* is reproduced too — but a backup that could not be re-taken
/// leaves an outgoing shell's last write unheld, and that copy is then kept.
///
/// Returns how many were removed and how many were kept.
fn discard_asides(asides: &[Aside], backups_root: &Path) -> (usize, usize) {
    let mut dropped = 0;
    let mut held = 0;
    for aside in asides {
        let Some(from) = &aside.kept else {
            continue;
        };
        if !from.exists() {
            continue;
        }

        let identical = backups_holding(backups_root, &aside.rel)
            .iter()
            .any(|root| measure(from) == measure(&root.join("home").join(&aside.rel)));
        if !identical {
            crate::logging::info(&format!(
                "kept {}, which no backup reproduces byte for byte",
                from.display()
            ));
            held += 1;
            continue;
        }

        let removed = if from.is_dir() && !from.is_symlink() {
            std::fs::remove_dir_all(from)
        } else {
            std::fs::remove_file(from)
        };
        match removed {
            Ok(()) => dropped += 1,
            Err(err) => {
                crate::logging::error(&format!(
                    "cannot remove {} after a successful switch: {err}",
                    from.display()
                ));
                held += 1;
            }
        }
    }
    (dropped, held)
}

/// Take the backup again, now that the outgoing shell has stopped.
///
/// The copy is written before anything is installed, which is the order that
/// makes the install reversible. It also means the shell being left was alive
/// while it was taken, and it goes on running through the whole install: Noctalia
/// wrote two clipboard entries and a notification history between its backup and
/// its stop, and [`clean`] then found a backup that no longer described the file
/// on disk and quite correctly kept the file rather than delete a version it had
/// no copy of. The machine was left with a state directory the switch had just
/// said it removed.
///
/// Stopping the shell closes that window. Nothing can write to its own state
/// after it exits, so taking the same sources into the same directory once more
/// records the last thing it wrote, and the byte-for-byte test [`clean`] runs
/// then passes for the right reason: the backup describes the file because the
/// file cannot have changed since.
///
/// A failure here is reported rather than fatal. The copy taken before the
/// install is still on disk and still verified, and every path whose live copy
/// has moved on is kept by `clean` — the behaviour before this ran, with none of
/// the worse cases.
fn refresh_backup(
    outgoing: &'static Shell,
    target: &'static Shell,
    home: &Path,
    backups_root: &Path,
    backup_root: &Path,
    compositor: &dyn Compositor,
    progress: &mut dyn FnMut(&str),
) {
    let Some(name) = backup_root.file_name().and_then(|n| n.to_str()) else {
        return;
    };
    let sources = crate::switch::backup_sources_for(home, compositor, Some(outgoing), target);
    match crate::backup::Backup::create(backups_root, name, &sources, home) {
        Ok(fresh) => {
            let line = format!(
                "took the backup again now that {} has stopped: {} files, holding what it last wrote",
                outgoing.display_name,
                fresh.file_count(),
            );
            progress(&line);
        }
        Err(err) => {
            let line = format!(
                "could not take the backup again after {} stopped, keeping the one taken before: {err:#}",
                outgoing.display_name,
            );
            progress(&line);
        }
    }
}

/// Take the outgoing shell's own files off the machine.
///
/// Runs only at the tail of a switch: the backup is written, read back and
/// re-checked, and the new shell has already proved itself alive. Nothing here
/// is allowed to delete on trust. A path goes only when the backup taken for
/// this very switch reproduces it byte for byte — the same test
/// [`discard_asides`] applies to the copies it drops, for the same reason: a
/// backup that no longer describes what is on disk is not a way back, it is a
/// reason to leave the file alone.
///
/// The paths are the shell's `config_dir`, its `state_dir`, its
/// `generated_paths` — the themes it renders into shared directories and would
/// render again on its next start — and its `companion_paths`, where the
/// companion programs kept their data while this shell was in charge. See
/// [`crate::shell::Identity::generated_paths`] and
/// [`crate::shell::Identity::companion_paths`].
///
/// Then its `discard_paths`, which are outside that rule on purpose: they are
/// the files and caches the shell wrote that hold no data, so there is nothing
/// for a backup to reproduce and nothing for a check to protect. They are
/// deleted on their own terms, with only the ownership guard left. See
/// [`crate::shell::Identity::discard_paths`].
///
/// A mismatch is now the exception it reads like. [`refresh_backup`] has taken
/// the copy again since the shell stopped, so a file that has moved on since is
/// one the shell somehow wrote after it exited, or a backup that could not be
/// re-taken. The file is then kept and said out loud, which is the honest
/// outcome: cleaner than it was, and not a delete on a backup that has gone
/// stale.
fn clean(
    outgoing: &Shell,
    target: &Shell,
    home: &Path,
    backups_root: &Path,
    backup_root: &Path,
    progress: &mut dyn FnMut(&str),
) -> (usize, usize) {
    let mut removed = 0;
    let mut kept = 0;

    // The backups are the reason any of this can be undone, which makes them the
    // one thing on disk this function must never remove: neither a path inside
    // them nor a path that contains them. Nothing in the registry is anywhere
    // near `~/.local/share/backups`, but a path declared one directory too high
    // would take the recovery copies away with the config — and the byte-for-byte
    // check below would not stop it, because the backup does contain itself.
    // `no_declared_path_can_reach_the_backups` is the registry-side half of this;
    // this is the half that refuses even so.
    let reaches_backups =
        |live: &Path| live.starts_with(backups_root) || backups_root.starts_with(live);

    // The guard applies to both lists below and is asked for by the same
    // reasoning in both: the swap wrote every path in `target.skel_extra_paths`
    // moments ago, so removing a path the incoming shell owns would take away
    // the shell that is starting. A future registry that gave two shells the
    // same `config_dir` is caught here too — and two shells declaring the same
    // companion data, which would be the same mistake one level out.
    //
    // Asked after the existence check rather than before it, because `kept`
    // counts what was kept: a path nobody has written anything to is not a file
    // the target rescued, and `generated_paths` makes that common — most of the
    // entries a shell declares have never landed on this machine.
    let target_owns = |rel: &str| {
        target.config_dir == Some(rel)
            || target.state_dir == Some(rel)
            || target.skel_extra_paths.contains(&rel)
            || target.generated_paths.contains(&rel)
            || target.companion_paths.contains(&rel)
    };

    for rel in [outgoing.config_dir, outgoing.state_dir]
        .into_iter()
        .flatten()
        .chain(outgoing.generated_paths.iter().copied())
        .chain(outgoing.companion_paths.iter().copied())
    {
        let live = home.join(rel);
        // `present`, not `exists`: DMS 2.0 spells a companion path twice, and the
        // older spelling is a link back to the directory this same loop removes
        // first, so the link points at nothing by the time its turn comes. The
        // link is still the path the registry names. See `present`.
        if !present(&live) {
            continue;
        }

        if reaches_backups(&live) {
            progress(&format!(
                "refusing to remove {rel}; it reaches the backups, which a switch never cleans"
            ));
            kept += 1;
            continue;
        }

        if target_owns(rel) {
            progress(&format!(
                "not cleaning {rel}: {} owns the same path",
                target.display_name
            ));
            kept += 1;
            continue;
        }

        let copy = backup_root.join("home").join(rel);
        // `present` here too: the copy of a symlinked companion is a link, and it
        // dangles inside the backup for the same reason it dangles in the home.
        if !present(&copy) || measure(&live) != measure(&copy) {
            let line = format!(
                "kept {rel}: the backup taken for this switch does not hold it byte for byte"
            );
            progress(&line);
            kept += 1;
            continue;
        }

        let result = if live.is_dir() && !live.is_symlink() {
            std::fs::remove_dir_all(&live)
        } else {
            std::fs::remove_file(&live)
        };
        match result {
            Ok(()) => {
                progress(&format!(
                    "removed {rel}; it is still in {}",
                    backup_root.display()
                ));
                removed += 1;
            }
            Err(err) => {
                let line = format!("cannot remove {rel} after a successful switch: {err}");
                progress(&line);
                kept += 1;
            }
        }
    }

    // The list with no data in it. No byte-for-byte comparison, because the
    // check this function is built on exists to stop a delete from losing
    // something a person wrote — and `every_discard_path_is_never_preserved`
    // fails the suite if any of these is reachable from a path a switch backs
    // up. There is nothing here to lose, which is the whole reason the list
    // exists.
    for rel in outgoing.discard_paths {
        let live = home.join(rel);
        // `present`, not `exists`: DMS 2.0 leaves `.cache/DankMaterialShell`
        // pointing at `.cache/dms`, the entry discarded just above it, so by the
        // time the loop reaches the link its target is gone. A link that points at
        // nothing is still the entry this list names, and leaving it behind is
        // exactly what naming it here is meant to prevent. See `present`.
        if !present(&live) {
            continue;
        }

        if reaches_backups(&live) {
            progress(&format!(
                "refusing to discard {rel}; it reaches the backups, which a switch never cleans"
            ));
            kept += 1;
            continue;
        }

        if target_owns(rel) {
            progress(&format!(
                "not discarding {rel}: {} owns the same path",
                target.display_name
            ));
            kept += 1;
            continue;
        }

        let result = if live.is_dir() && !live.is_symlink() {
            std::fs::remove_dir_all(&live)
        } else {
            std::fs::remove_file(&live)
        };
        match result {
            Ok(()) => {
                progress(&format!(
                    "discarded {rel}; it holds no data, so no copy was taken"
                ));
                removed += 1;
            }
            Err(err) => {
                let line = format!("cannot discard {rel} after a successful switch: {err}");
                progress(&line);
                kept += 1;
            }
        }
    }

    (removed, kept)
}

/// Start one systemd user unit, treating a refusal as this switch's failure.
///
/// Shared by the three starts [`start`] performs — the shell's own unit, the
/// companions it needs, and the daemons that autostart with it — so a refusal
/// reads the same wherever it comes from and there is one place to fix if the
/// wording or the error type changes.
fn start_systemd_unit(
    name: &str,
    progress: &mut dyn FnMut(&str),
) -> Result<(), (&'static str, String)> {
    let out = run("systemctl", &["--user", "start", name])
        .map_err(|e| ("start", format!("cannot run systemctl: {e}")))?;
    if !out.status.success() {
        return Err((
            "start",
            format!(
                "{name} refused to start: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ),
        ));
    }
    progress(&format!("{name} started"));
    Ok(())
}

/// Start a shell the way it says it starts.
fn start(
    shell: &Shell,
    home: &Path,
    progress: &mut dyn FnMut(&str),
) -> Result<(), (&'static str, String)> {
    // Companions first, in the order the shell declares them. A shell's own unit
    // does not depend on them, so systemd starts it either way and the result is
    // a running shell quietly missing part of its interface.
    for unit in shell.companion_systemd_user_units {
        let name = format!("{unit}.service");
        progress(&format!(
            "starting {name}, which {} needs",
            shell.display_name
        ));
        start_systemd_unit(&name, progress)?;
    }

    // Which branch to take comes from `mechanism`, not from the presence of a unit
    // name. They agree today, and a_mechanism_agrees_with_its_unit_name keeps
    // them honest, but the enum is the one that also knows about the compositor
    // launchers — where there is no unit because the compositor's own config is
    // what runs the shell. Reading the unit instead would make those shells look
    // like direct processes, which is the same answer for the wrong reason.
    if shell.mechanism.uses_systemd_unit() {
        let unit = shell.systemd_user_unit.ok_or_else(|| {
            (
                "start",
                format!("{} claims a systemd unit but names none", shell.id),
            )
        })?;

        // The daemons that autostart with the shell come up before it, enabled
        // like the shell itself: the mirror of the stop+disable+mask `stop_unit`
        // leaves behind, so switching back to DMS is leaving it run backwards.
        // On the image that globally enables these units it is a mask being
        // undone; anywhere else this enable is the only thing that will start
        // them at the next login.
        for companion in shell.autostart_systemd_user_units {
            unmask_and_enable(companion, progress);
            let name = format!("{companion}.service");
            progress(&format!(
                "starting {name}, which {} needs",
                shell.display_name
            ));
            start_systemd_unit(&name, progress)?;
        }

        // The same move for the shell's own unit, after its companions: the
        // enable verb for a unit that carries its own `[Install]`, and a wants
        // link for one that does not — see
        // [`Port::systemd_user_unit_wanted_by`], whose own shell states which of
        // the two applies. `stop_unit` takes the mirror of whichever branch runs
        // here.
        match shell.systemd_user_unit_wanted_by {
            Some(target) => enable_via_wants(home, target, unit, progress),
            None => unmask_and_enable(unit, progress),
        }
        let name = format!("{unit}.service");
        progress(&format!("starting {name}"));
        start_systemd_unit(&name, progress)?;
        return Ok(());
    }

    // Niri waits for a `spawn-at-startup` command to return before it continues,
    // so a shell started that way has to detach or the session stalls behind it.
    // Hyprland's `exec_on` does not wait, and leaving the flag off there keeps the
    // process a child this program can account for.
    let mut argv: Vec<&str> = shell.start_argv.to_vec();
    if shell.mechanism.needs_detached_start() {
        argv.push("--daemon");
    }

    progress(&format!(
        "starting {} ({})",
        shell.display_name,
        argv.join(" ")
    ));
    Command::new(argv[0])
        .args(&argv[1..])
        .spawn()
        .map_err(|e| ("start", format!("cannot run {}: {e}", argv.join(" "))))?;
    Ok(())
}

/// Start the incoming shell's polkit agent, if it needs one and none is running.
///
/// A shell whose config runs an external agent gets it for free on a normal
/// login, but a live switch reloads the config instead of starting a session and
/// `exec-once` does not run again — so by the time the privileged uninstall
/// stage asks for a password, nobody is there to answer it. Starting the agent
/// named by the shell here closes that gap. A shell that answers polkit from its
/// own process names none and is left alone; one whose agent is already up (a
/// switch between two shells that share it) is left alone too.
fn ensure_polkit_agent(shell: &Shell, progress: &mut dyn FnMut(&str)) {
    let Some((program, args)) = shell.polkit_agent.split_first() else {
        return;
    };
    let name = std::path::Path::new(program)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(program);
    if !pids_of(name, &[]).is_empty() {
        return;
    }
    progress(&format!(
        "starting {} to answer the password prompt",
        shell.polkit_agent.join(" ")
    ));
    match Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(_) => {}
        Err(error) => progress(&format!(
            "could not start {}: {error}",
            shell.polkit_agent.join(" ")
        )),
    }
}

/// The elapsed seconds a wait has been running, once a heartbeat is due;
/// `None` while the wait is still young.
fn heartbeat(start: Instant, next: &mut Duration) -> Option<u64> {
    let elapsed = start.elapsed();
    if elapsed < *next {
        return None;
    }
    while *next <= elapsed {
        *next += HEARTBEAT;
    }
    Some(elapsed.as_secs())
}

/// Check the new shell is really up, not merely launched.
fn verify(
    shell: &Shell,
    bus_owner_before: Option<u32>,
    progress: &mut dyn FnMut(&str),
) -> Result<(), (&'static str, String)> {
    let start = Instant::now();
    let deadline = start + START_GRACE;

    if shell.mechanism.uses_systemd_unit() {
        // Asking systemd whether the unit is active is a stronger answer than
        // looking for a process: a unit can be active while the shell inside it is
        // still failing to draw, and a process can be alive while the unit that
        // should supervise it is not. See `a_mechanism_agrees_with_its_unit_name`
        // for why the unit name is guaranteed to be there here.
        let unit = shell.systemd_user_unit.ok_or_else(|| {
            (
                "verify",
                format!("{} claims a systemd unit but names none", shell.id),
            )
        })?;
        let name = format!("{unit}.service");
        progress(&format!("waiting for {name} to report active"));
        let mut next_tick = HEARTBEAT;
        loop {
            if let Some(elapsed) = heartbeat(start, &mut next_tick) {
                progress(&format!(
                    "still waiting for {name} to report active ({elapsed}s)"
                ));
            }
            let out = match run("systemctl", &["--user", "is-active", &name]) {
                Ok(o) => o,
                Err(e) => return Err(("verify", format!("cannot query {name}: {e}"))),
            };
            if out.status.success() {
                break;
            }
            if Instant::now() >= deadline {
                let state = String::from_utf8_lossy(&out.stdout).trim().to_string();
                return Err((
                    "verify",
                    format!(
                        "{name} did not become active in {}s (last said {state:?})",
                        START_GRACE.as_secs()
                    ),
                ));
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        progress(&format!("{name} is active"));
    }

    let mut next_tick = HEARTBEAT;
    loop {
        if let Some(elapsed) = heartbeat(start, &mut next_tick) {
            progress(&format!(
                "still waiting for {} to appear ({elapsed}s)",
                shell.binary
            ));
        }
        if !pids_of(shell.binary, shell.process_match).is_empty() {
            progress(&format!("{} is running", shell.binary));
            break;
        }
        if Instant::now() >= deadline {
            return Err((
                "verify",
                format!("{} never appeared as a process", shell.binary),
            ));
        }

        std::thread::sleep(Duration::from_millis(500));
    }

    // The last check, and the one that actually says the switch worked: the
    // notification bus has to belong to the new shell. A live process and an active
    // unit are both compatible with the old shell still holding the bus, which is
    // the failure that would otherwise only surface as a desktop with no
    // notifications after the logout.
    //
    // Directly spawned shells (like noctalia) take a moment to claim the bus, so
    // "nobody owns it yet" is treated as transient while we are still inside the
    // grace period, rather than an immediate failure.
    let mut next_tick = HEARTBEAT;
    loop {
        if let Some(elapsed) = heartbeat(start, &mut next_tick) {
            progress(&format!(
                "still waiting for {NOTIFICATION_BUS} ({elapsed}s)"
            ));
        }
        match (bus_owner_before, notification_owner()) {
            (_, None) => {
                if Instant::now() >= deadline {
                    return Err((
                        "verify",
                        format!(
                            "nobody owns {NOTIFICATION_BUS} by the end of the grace period; the new shell never claimed the notification bus"
                        ),
                    ));
                }
                std::thread::sleep(Duration::from_millis(500));
                continue;
            }
            (Some(before), Some(after)) if before == after => {
                if Instant::now() >= deadline {
                    return Err((
                        "verify",
                        format!(
                            "{NOTIFICATION_BUS} is still owned by pid {after}, the process that owned it before, so the old shell never released the notification bus"
                        ),
                    ));
                }
                std::thread::sleep(Duration::from_millis(500));
                continue;
            }
            (_, Some(after)) => {
                progress(&format!(
                    "{NOTIFICATION_BUS} now belongs to pid {after}, not to the one it had before"
                ));
                break;
            }
        }
    }
    Ok(())
}

/// Bring the old shell back after a failed switch.
fn resume(shell: Option<&'static Shell>, home: &Path, progress: &mut dyn FnMut(&str)) -> bool {
    let Some(shell) = shell else { return true };
    progress(&format!("starting {} back", shell.display_name));
    match start(shell, home, progress) {
        Ok(()) => true,
        Err((_, detail)) => {
            crate::logging::error(&format!("cannot restart {}: {detail}", shell.id));
            false
        }
    }
}

/// End the session so the next login starts from the new configuration.
///
/// Deliberately last. Everything before this point has been checked, so the only
/// thing logout adds is that the next session begins clean instead of inheriting
/// a compositor that was configured for the other shell.
///
/// Three ways out, tried in order, because the obvious one does not work from
/// where this runs. A desktop app launched as a systemd user service lives in
/// `user@UID.service/app.slice`, not in the graphical session's own scope, so
/// logind treats `terminate-session` on that session as a foreign request and
/// refuses it (needs `org.freedesktop.login1.manage`). So:
///
/// 1. the compositor's own quit, which talks to it over its socket and has no
///    scope question at all;
/// 2. `uwsm stop`, when UWSM owns the session and can stop it cleanly;
/// 3. `loginctl terminate-session`, which is right when this process *is* part
///    of the session (a run from a terminal inside it).
fn logout(
    compositor: &'static dyn Compositor,
    progress: &mut dyn FnMut(&str),
) -> Result<(), String> {
    progress("ending the session; log back in to start the new shell");

    match compositor.exit() {
        Ok(()) => {
            progress(&format!("asked {} to quit", compositor.label()));
            return Ok(());
        }
        Err(detail) => progress(&format!("could not quit the compositor: {detail}")),
    }

    // UWSM, when it manages the session, stops it as a unit rather than as a
    // logind session, so it is not subject to the scope rule above. Absent, the
    // call fails with "not found" and the next fallback runs.
    if let Ok(out) = run("uwsm", &["stop"]) {
        if out.status.success() {
            progress("uwsm stopped the session");
            return Ok(());
        }
        progress(&format!(
            "uwsm stop did not work: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }

    let session = std::env::var("XDG_SESSION_ID").ok().or_else(|| {
        run("loginctl", &["show-session", "self", "-p", "Id", "--value"])
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| {
                let id = String::from_utf8_lossy(&o.stdout).trim().to_string();
                (!id.is_empty()).then_some(id)
            })
    });

    let Some(session) = session else {
        return Err(
            "the shell is switched but the session was not ended, and I could not find which \
             session this is. Log out yourself when convenient."
                .to_string(),
        );
    };

    progress(&format!("ending session {session} with loginctl"));
    let out = run("loginctl", &["terminate-session", &session])
        .map_err(|e| format!("cannot run loginctl: {e}"))?;
    if out.status.success() {
        progress("loginctl ended the session");
        Ok(())
    } else {
        Err(format!(
            "the shell is switched but loginctl could not end session \
             {session}; log out yourself when convenient. loginctl said: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

fn run(program: &str, args: &[&str]) -> std::io::Result<std::process::Output> {
    std::process::Command::new(program).args(args).output()
}

/// Every pid that is this shell, read from `/proc`.
///
/// Delegates to [`crate::shell::pids_matching`] so detection, stopping and
/// verification all agree: a shell whose process name differs from its binary —
/// Caelestia on the shared `quickshell` runtime — has to be found by the same
/// rule everywhere, and `Shell::running` uses that same function.
fn pids_of(binary: &str, argv_match: &[&str]) -> Vec<u32> {
    crate::shell::pids_matching(binary, argv_match)
}

fn signal(pids: &[u32], sig: &str) -> std::io::Result<()> {
    for pid in pids {
        // `/bin/kill` rather than libc: this binary has no unsafe dependency and
        // a missing signal is not worth adding one for.
        //
        // The result is checked. Discarding it here would let a switch report
        // having stopped the old shell while it is in fact still running and
        // still holding the notification bus, and the failure would surface later
        // somewhere much harder to read.
        let out = run("kill", &[sig, &pid.to_string()])?;
        if !out.status.success() {
            return Err(std::io::Error::other(format!(
                "kill {sig} {pid} said: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )));
        }
    }
    Ok(())
}

/// Copy a file or directory tree.
fn copy(from: &Path, to: &Path) -> std::io::Result<()> {
    let meta = std::fs::symlink_metadata(from)?;
    if meta.is_symlink() {
        let target = std::fs::read_link(from)?;
        let _ = std::fs::remove_file(to);
        return std::os::unix::fs::symlink(target, to);
    }
    if meta.is_dir() {
        std::fs::create_dir_all(to)?;
        for entry in std::fs::read_dir(from)? {
            let entry = entry?;
            copy(&entry.path(), &to.join(entry.file_name()))?;
        }
        // Keep the mode, so an executable script stays executable in the skel.
        let mode = meta.permissions().mode();
        std::fs::set_permissions(to, std::fs::Permissions::from_mode(mode))?;
        return Ok(());
    }
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(from, to)?;
    Ok(())
}

use std::os::unix::fs::PermissionsExt;

/// Whether the path a registry entry names is there at all — as a file, as a
/// directory, or as a link whose target is already gone.
///
/// `Path::exists()` follows the last of those and reports "not there", which is
/// how a link DMS 2.0 left pointing at a directory a switch had just discarded
/// used to survive every switch away and dangle in the home. Both lists that
/// clean the home mean "the path the registry names", and a link that points at
/// nothing is still that path.
fn present(path: &Path) -> bool {
    path.exists() || path.is_symlink()
}

/// How many files and bytes a tree holds, following the same rules as `copy`.
fn measure(dir: &Path) -> (usize, u64) {
    let mut files = 0;
    let mut bytes = 0;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let Ok(meta) = std::fs::symlink_metadata(&current) else {
            continue;
        };
        if meta.is_symlink() {
            files += 1;
            continue;
        }
        if meta.is_dir() {
            let Ok(entries) = std::fs::read_dir(&current) else {
                continue;
            };
            for entry in entries.flatten() {
                stack.push(entry.path());
            }
        } else {
            files += 1;
            bytes += meta.len();
        }
    }
    (files, bytes)
}

#[cfg(test)]
#[path = "tests/apply.rs"]
mod tests;
