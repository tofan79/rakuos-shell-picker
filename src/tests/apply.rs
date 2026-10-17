use super::*;
use crate::compositor::hyprland::shells::{caelestia, dms, end4pc, noctalia};
use crate::compositor::hyprland::Hyprland;

struct Temp(PathBuf);

impl Temp {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "rsp-apply-{tag}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Temp(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn write(path: &Path, body: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}

/// A backup directory with a file at each of `rels`.
///
/// `exists` is what the lookup uses, so a backup may hold a single file as
/// well as a whole directory; the marker sits at the path itself either way.
fn make_backup(root: &Path, name: &str, rels: &[&str]) {
    for rel in rels {
        write(
            &root.join(name).join("home").join(rel).join("marker"),
            &format!("{name}:{rel}"),
        );
    }
    write(&root.join(name).join("manifest.toml"), "");
}

/// The marker `make_backup` left inside a backed-up directory.
fn marker(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("marker")).unwrap()
}

#[test]
fn the_compositor_config_comes_from_the_skel_even_when_a_backup_has_one() {
    // This is the whole reason `plan` exists. `~/.config/hypr` is in every
    // backup, because every switch saves it, so "newest backup first" would
    // hand back the file that launches the shell being replaced. The backup
    // below deliberately holds a config that starts the old shell.
    let t = Temp::new("comp-from-skel");
    let backups = t.path().join("backups");
    let skel = t.path().join("skel");

    make_backup(
        &backups,
        "20260101-000000-001-hyprland-noctalia",
        &[".config/hypr"],
    );
    write(
        &backups
            .join("20260101-000000-001-hyprland-noctalia/home/.config/hypr/config/autostart.lua"),
        "hl.exec_cmd(\"noctalia\")",
    );
    write(
        &skel.join(".config/hypr/config/autostart.lua"),
        "systemctl --user start dms.service",
    );

    let mut said = Vec::new();
    let entries = plan(&dms::SHELL, &Hyprland, &backups, &skel, &mut |s| {
        said.push(s.to_string())
    })
    .unwrap();

    let hypr = entries.iter().find(|e| e.rel == ".config/hypr").unwrap();
    assert_eq!(
        hypr.origin,
        Origin::Skel,
        "the compositor config must not come from a backup"
    );
    let body = std::fs::read_to_string(hypr.src.join("config/autostart.lua")).unwrap();
    assert!(
        body.contains("dms.service"),
        "the compositor config the switch would install still launches the old shell: {body}"
    );
    assert!(
        said.iter()
            .any(|l| l.contains("comes from the skel, not a backup")),
        "the log has to say where it took it from: {said:?}"
    );
}

#[test]
fn a_shells_own_config_comes_from_the_newest_backup_kept_for_it() {
    // The rule `plan` runs on. A backup only ever describes the desktop as it
    // was with the shell named at the end of it in charge, so among that
    // shell's own backups the newest is the newest arrangement the user had.
    let t = Temp::new("own-from-backup");
    let backups = t.path().join("backups");
    let skel = t.path().join("skel");

    make_backup(
        &backups,
        "20260101-hyprland-dms",
        &[".config/DankMaterialShell"],
    );
    make_backup(
        &backups,
        "20260202-hyprland-dms",
        &[".config/DankMaterialShell"],
    );
    write(
        &skel.join(".config/DankMaterialShell/settings.json"),
        "from skel",
    );

    write(&skel.join(".config/hypr/config/autostart.lua"), "dms");

    let entries = plan(&dms::SHELL, &Hyprland, &backups, &skel, &mut |_| {}).unwrap();
    let own = entries
        .iter()
        .find(|e| e.rel == ".config/DankMaterialShell")
        .unwrap();

    match &own.origin {
        Origin::Backup(root) => {
            assert_eq!(
                root.file_name().unwrap().to_string_lossy(),
                "20260202-hyprland-dms",
                "the newest backup kept for this shell must win"
            );
            assert_eq!(
                marker(&own.src),
                "20260202-hyprland-dms:.config/DankMaterialShell"
            );
        }
        Origin::Skel => panic!("a backup holding it exists, so the skel must not be used"),
    }
}

#[test]
fn a_backup_of_another_shell_is_never_read_for_this_shells_paths() {
    // The reason the rule is "this shell's backups" and not "any backup holding
    // it". After a switch away from DMS the newest directory on disk is DMS's,
    // and it holds DMS's version of every shared file: a `gtk.css` importing
    // `dank-colors.css`, and a launcher entry with `NoDisplay=true`. Reading it
    // on the way back to Noctalia hides Noctalia behind the launcher and points
    // GTK at a sheet no shell on the machine writes any more.
    let t = Temp::new("foreign-backup");
    let backups = t.path().join("backups");
    let skel = t.path().join("skel");

    // Newer, and it holds both Noctalia's config and a shared file — but it is
    // a backup of DMS, so it is not an answer to either question.
    make_backup(
        &backups,
        "20261008-hyprland-dms",
        &[".config/noctalia", ".config/gtk-3.0/gtk.css"],
    );
    make_backup(
        &backups,
        "20261001-hyprland-noctalia",
        &[".config/noctalia"],
    );

    write(
        &skel.join(".config/gtk-3.0/gtk.css"),
        "@import url(\"noctalia.css\")",
    );
    write(&skel.join(".config/hypr/config/autostart.lua"), "noctalia");

    let mut said = Vec::new();
    let entries = plan(&noctalia::SHELL, &Hyprland, &backups, &skel, &mut |line| {
        said.push(line.to_string())
    })
    .unwrap();

    let own = entries
        .iter()
        .find(|e| e.rel == ".config/noctalia")
        .unwrap();
    assert_eq!(
        own.origin,
        Origin::Backup(backups.join("20261001-hyprland-noctalia")),
        "the backup of the shell coming back, not the newest one on disk"
    );

    let gtk = entries
        .iter()
        .find(|e| e.rel == ".config/gtk-3.0/gtk.css")
        .unwrap();
    assert_eq!(
        gtk.origin,
        Origin::Skel,
        "only Noctalia's own skel may write the import Noctalia runs on: {said:?}"
    );
}

#[test]
fn a_shell_falls_back_to_the_skel_when_no_backup_holds_its_config() {
    let t = Temp::new("fallback-skel");
    let backups = t.path().join("backups");
    let skel = t.path().join("skel");

    // A backup exists, but only of the shell being replaced.
    make_backup(&backups, "20260101-000000-001", &[".config/noctalia"]);
    write(
        &skel.join(".config/DankMaterialShell/settings.json"),
        "from skel",
    );
    write(&skel.join(".config/hypr/config/autostart.lua"), "dms");

    let entries = plan(&dms::SHELL, &Hyprland, &backups, &skel, &mut |_| {}).unwrap();
    let own = entries
        .iter()
        .find(|e| e.rel == ".config/DankMaterialShell")
        .unwrap();
    assert_eq!(own.origin, Origin::Skel);
    assert_eq!(
        std::fs::read_to_string(own.src.join("settings.json")).unwrap(),
        "from skel"
    );
}

#[test]
fn a_shell_with_no_config_anywhere_still_switches_on_its_defaults() {
    // The counterpart to the compositor rule: a shell that has never been
    // configured is not a broken switch, it is a shell on its defaults. That
    // must not block the switch, but it has to be said out loud.
    let t = Temp::new("defaults-ok");
    let backups = t.path().join("backups");
    let skel = t.path().join("skel");
    std::fs::create_dir_all(&backups).unwrap();
    write(&skel.join(".config/hypr/config/autostart.lua"), "dms");

    let mut said = Vec::new();
    let entries = plan(&dms::SHELL, &Hyprland, &backups, &skel, &mut |s| {
        said.push(s.to_string())
    })
    .expect("a missing shell config must not block the switch");

    assert!(!entries.iter().any(|e| e.rel == ".config/DankMaterialShell"));
    assert!(
        said.iter().any(|l| l.contains("on its own defaults")),
        "silently dropping a config would hide what the user is getting: {said:?}"
    );
}

#[test]
fn plan_refuses_when_the_skel_has_no_compositor_config() {
    let t = Temp::new("nothing");
    let backups = t.path().join("backups");
    let skel = t.path().join("skel");
    std::fs::create_dir_all(&backups).unwrap();
    std::fs::create_dir_all(&skel).unwrap();

    let err = plan(&dms::SHELL, &Hyprland, &backups, &skel, &mut |_| {}).unwrap_err();
    assert!(
        err.contains("nothing to switch to"),
        "the error has to say why refusing is the safe answer: {err}"
    );
}

#[test]
fn staging_counts_what_it_copied_and_the_swap_leaves_the_old_copy_aside() {
    let t = Temp::new("swap");
    let home = t.path().join("home");
    let backups = t.path().join("backups");
    let skel = t.path().join("skel");
    std::fs::create_dir_all(&home).unwrap();

    write(
        &skel.join(".config/hypr/config/autostart.lua"),
        "new skel config",
    );
    write(
        &skel.join(".config/hypr/scripts/x.sh"),
        "#!/bin/sh\necho hi\n",
    );
    // Give the source the mode a real skel script has (they are 755 in git),
    // so this asserts the mode is carried across, not that one is invented.
    std::fs::set_permissions(
        skel.join(".config/hypr/scripts/x.sh"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    write(
        &home.join(".config/hypr/config/autostart.lua"),
        "old live config",
    );
    // A second path that does not exist yet, so one swap covers both cases:
    // something replaced and something created.
    write(&skel.join(".config/DankMaterialShell/settings.json"), "{}");

    let entries = plan(&dms::SHELL, &Hyprland, &backups, &skel, &mut |_| {}).unwrap();
    let staging = stage(&entries, &home, "dms", "20260101-000000-000").unwrap();

    let staged_files = measure(&staging.join(".config/hypr")).0;
    assert!(staged_files > 0, "staging copied nothing");

    let asides = swap(
        &entries,
        &staging,
        &home,
        "dms",
        "20260101-000000-000",
        &mut |_| {},
    )
    .expect("swap");

    let live = std::fs::read_to_string(home.join(".config/hypr/config/autostart.lua")).unwrap();
    assert_eq!(
        live, "new skel config",
        "the skel did not reach the live path"
    );

    assert_eq!(
        asides.len(),
        2,
        "both the replaced and the created path have to be recorded"
    );
    // Looked up by name rather than by position: `plan` walks the shell's own
    // config before the compositor's, and a test that depends on that ordering
    // breaks the day the order stops mattering.
    let kept = asides
        .iter()
        .find(|a| a.rel == ".config/hypr")
        .and_then(|a| a.kept.as_ref())
        .expect("hypr was displaced, so it has an aside");
    let kept = std::fs::read_to_string(kept.join("config/autostart.lua")).unwrap();
    assert_eq!(kept, "old live config", "the aside is not the old config");

    // An executable script must still be executable, or the staged copy is
    // subtly broken in a way a file count cannot see.
    let mode = std::fs::metadata(home.join(".config/hypr/scripts/x.sh"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(
        mode & 0o777,
        0o755,
        "the staged script did not keep the mode it had in the skel"
    );

    assert!(restore(&asides, &home));
    let back = std::fs::read_to_string(home.join(".config/hypr/config/autostart.lua")).unwrap();
    assert_eq!(
        back, "old live config",
        "rollback did not put the original back"
    );
    assert!(
        !home.join(".config/DankMaterialShell").exists(),
        "rollback left the path it created in place"
    );
}

#[test]
fn a_path_that_did_not_exist_before_is_removed_again_on_rollback() {
    // Before the switch there is no DMS config at all. Rolling back therefore
    // cannot "put the old one back", because there was no old one: it has to
    // remove what the switch created, or the next Noctalia login reads a DMS
    // configuration and behaves oddly with no obvious cause.
    let t = Temp::new("no-aside");
    let home = t.path().join("home");
    let skel = t.path().join("skel");
    std::fs::create_dir_all(&home).unwrap();
    write(&skel.join(".config/DankMaterialShell/settings.json"), "{}");

    let entries = vec![Entry {
        rel: ".config/DankMaterialShell".to_string(),
        src: skel.join(".config/DankMaterialShell"),
        origin: Origin::Skel,
    }];
    let staging = stage(&entries, &home, "dms", "20260101-000000-000").unwrap();
    let asides = swap(
        &entries,
        &staging,
        &home,
        "dms",
        "20260101-000000-000",
        &mut |_| {},
    )
    .expect("swap");

    assert_eq!(
        asides.len(),
        1,
        "the path was touched, so it must be recorded"
    );
    assert!(
        asides[0].kept.is_none(),
        "nothing was displaced, so there is nothing to put back"
    );
    assert!(home
        .join(".config/DankMaterialShell/settings.json")
        .is_file());

    assert!(restore(&asides, &home));
    assert!(
        !home.join(".config/DankMaterialShell").exists(),
        "rollback left the created configuration in place: {:?}",
        std::fs::read_dir(home.join(".config")).map(|d| d
            .filter_map(|e| e.ok())
            .map(|e| e.file_name())
            .collect::<Vec<_>>())
    );
}

#[test]
fn staging_short_copied_files_are_caught_before_anything_is_swapped() {
    // The check compares the two counts after copying. Make them disagree by
    // pointing at a source that changes underneath the measurement.
    let t = Temp::new("short");
    let home = t.path().join("home");
    let src = t.path().join("src");
    std::fs::create_dir_all(&home).unwrap();
    write(&src.join("one.txt"), "1");
    write(&src.join("two.txt"), "2");

    let entries = vec![Entry {
        rel: ".config/thing".to_string(),
        src: src.clone(),
        origin: Origin::Skel,
    }];

    // Copy succeeds and the counts match, so this staging is accepted.
    let staging = stage(&entries, &home, "dms", "20260101-000000-000").unwrap();
    assert_eq!(measure(&staging.join(".config/thing")).0, 2);

    // Now a source that is not there at all: copy fails rather than writing
    // an empty tree, which is what keeps a later swap from wiping a path.
    let entries = vec![Entry {
        rel: ".config/missing".to_string(),
        src: t.path().join("nope"),
        origin: Origin::Skel,
    }];
    assert!(stage(&entries, &home, "dms", "20260101-000000-001").is_err());
}

#[test]
fn pids_of_finds_this_process_and_ignores_a_name_nobody_has() {
    let me = std::process::id();
    let comm = std::fs::read_to_string(format!("/proc/{me}/comm")).unwrap();
    let comm = comm.trim().to_string();
    assert!(
        pids_of(&comm, &[]).contains(&me),
        "should find pid {me} by comm {comm}"
    );
    assert!(
        pids_of("definitely-not-a-running-process-xyzzy", &[]).is_empty(),
        "a name nobody has must come back empty, not a guess"
    );
}

#[test]
fn the_backup_check_rejects_a_directory_with_no_manifest() {
    // apply() refuses to proceed on a backup root with no manifest, because
    // that backup is the only way back from the steps that follow.
    let t = Temp::new("no-manifest");
    let root = t.path().join("20260101-000000-001");
    std::fs::create_dir_all(&root).unwrap();
    assert!(!root.join("manifest.toml").is_file());
}

#[test]
fn both_shell_definitions_the_plan_is_tested_against_are_the_ones_registered() {
    // Guards against the tests drifting onto a different shell than the one
    // a switch actually uses.
    assert_eq!(dms::SHELL.config_dir, Some(".config/DankMaterialShell"));
    assert_eq!(dms::SHELL.systemd_user_unit, Some("dms"));
    assert_eq!(noctalia::SHELL.systemd_user_unit, None);
    assert_eq!(Hyprland.config_dir(), ".config/hypr");
}

#[test]
fn the_skel_paths_resolve_to_real_directories() {
    for (comp, shell) in [("hyprland", "dms"), ("hyprland", "noctalia")] {
        let dir = skel_for(comp, shell);
        assert!(dir.is_dir(), "{} should exist", dir.display());
    }
}

#[test]
fn dms_declares_the_calendar_service_it_draws_widgets_from() {
    // Without this the live switch starts DMS with nothing behind it and the
    // login that follows looks merely incomplete rather than broken.
    assert_eq!(
        dms::SHELL.companion_systemd_user_units,
        &["dcal"],
        "DMS draws calendar widgets that dcal.service provides"
    );
    assert!(
        noctalia::SHELL.companion_systemd_user_units.is_empty(),
        "Noctalia draws no widgets from a helper service"
    );
}

#[test]
fn dms_declares_the_search_unit_that_autostarts_with_it() {
    // `danksearch` ships dsearch.service and DMS searches through it. A switch
    // takes it up and down with DMS itself — unmasked, enabled and started on
    // the way in, stopped, disabled and masked on the way out — and it is named
    // on DMS rather than on every shell that is *not* DMS, so there is one copy
    // of the fact and a fifth shell cannot forget it. The verbs live in
    // `apply::start` and `apply::stop_unit`.
    assert_eq!(
        dms::SHELL.autostart_systemd_user_units, &["dsearch"],
        "DMS searches through dsearch.service, which must not outlive it"
    );
    assert!(
        noctalia::SHELL.autostart_systemd_user_units.is_empty(),
        "Noctalia owns no daemon of its own"
    );
}

#[test]
fn a_shell_is_stopped_the_same_way_it_is_started() {
    // The whole reason this is a function and not a branch inside `stop`:
    // `stop` cannot be exercised against a live desktop from the suite, and the
    // cost of getting the branch wrong is a supervised shell being signalled
    // like a bare process — left running beside the shell that replaced it,
    // both after the same notification bus.
    assert_eq!(stop_plan(&dms::SHELL), Ok(StopPlan::Unit("dms")));
    assert_eq!(stop_plan(&noctalia::SHELL), Ok(StopPlan::Signals));
    assert_eq!(dms::SHELL.systemd_user_unit, Some("dms"));
}

#[test]
fn the_notification_bus_is_read_from_the_live_system_or_reports_nothing() {
    // There is no session bus in a plain test run, so the honest expectation
    // is `None` — the point is that it must not invent an answer, and must not
    // panic when the name is unowned.
    let owner = notification_owner();
    println!("notification owner on this bus: {owner:?}");
    if let Some(pid) = owner {
        assert!(pid > 0, "a parsed pid should be positive, got {pid}");
    }
}

const LAUNCHER: &str = ".local/share/applications/dev.noctalia.Noctalia.desktop";

#[test]
fn switching_to_dms_would_hide_the_noctalia_launcher() {
    // The whole point of listing one launcher path per shell: DMS is installed
    // and visible, Noctalia is installed but hidden, so the menu cannot be used
    // to launch the shell that is no longer running.
    let mut said = Vec::new();
    let entries = plan(
        &dms::SHELL,
        &Hyprland,
        Path::new("/nonexistent-backups"),
        &skel_for("hyprland", "dms"),
        &mut |s| said.push(s.to_string()),
    )
    .expect("the real dms skel should plan cleanly");

    let launcher = entries
        .iter()
        .find(|e| e.rel == LAUNCHER)
        .expect("dms must carry the launcher path so it can be hidden");
    let body = std::fs::read_to_string(&launcher.src).unwrap();
    assert!(
        body.contains("NoDisplay=true"),
        "the staged override would not hide the launcher: {body}"
    );

    // And the rest of `~/.local/share/applications` must be untouched, which is
    // why a single file is listed rather than the directory.
    assert!(
        !entries.iter().any(|e| e.rel == ".local/share/applications"),
        "listing the whole directory would replace every launcher the user has"
    );
}

#[test]
fn switching_back_to_noctalia_would_bring_its_launcher_back() {
    let mut said = Vec::new();
    let entries = plan(
        &noctalia::SHELL,
        &Hyprland,
        Path::new("/nonexistent-backups"),
        &skel_for("hyprland", "noctalia"),
        &mut |s| said.push(s.to_string()),
    )
    .expect("the real noctalia skel should plan cleanly");

    let launcher = entries
        .iter()
        .find(|e| e.rel == LAUNCHER)
        .expect("noctalia must carry the launcher path to undo the hiding");
    let body = std::fs::read_to_string(&launcher.src).unwrap();
    assert!(
        !body.contains("NoDisplay"),
        "the skel entry still hides the launcher, so switching back would leave \
         Noctalia installed and invisible"
    );
}

#[test]
fn switching_to_caelestia_would_hide_the_noctalia_launcher() {
    // The same contract DMS holds, for the shell that joined it: whatever is
    // running when the switch lands, the launcher of the shell that just left
    // must not still be on offer in the menu.
    let mut said = Vec::new();
    let entries = plan(
        &caelestia::SHELL,
        &Hyprland,
        Path::new("/nonexistent-backups"),
        &skel_for("hyprland", "caelestia"),
        &mut |s| said.push(s.to_string()),
    )
    .expect("the real caelestia skel should plan cleanly");

    let launcher = entries
        .iter()
        .find(|e| e.rel == LAUNCHER)
        .expect("caelestia must carry the launcher path so it can be hidden");
    let body = std::fs::read_to_string(&launcher.src).unwrap();
    assert!(
        body.contains("NoDisplay=true"),
        "the staged override would not hide the launcher: {body}"
    );

    assert!(
        !entries.iter().any(|e| e.rel == ".local/share/applications"),
        "listing the whole directory would replace every launcher the user has"
    );
}

#[test]
fn caelestia_skel_carries_the_config_dir_its_hyprland_bootstrap_needs() {
    // The dir `hyprland.lua` writes `hypr-vars.lua` and `hypr-user.lua` into.
    // Without the skel holding it, `plan` has no source for `.config/caelestia`
    // and the switch logs "will start on its own defaults" — leaving the
    // directory uncreated, so upstream's `maybe_create` (an `io.open` that
    // fails *silently* when the parent is missing) never writes the files and
    // `require("hypr-vars")` dies at startup.
    let mut said = Vec::new();
    let entries = plan(
        &caelestia::SHELL,
        &Hyprland,
        Path::new("/nonexistent-backups"),
        &skel_for("hyprland", "caelestia"),
        &mut |s| said.push(s.to_string()),
    )
    .expect("the real caelestia skel should plan cleanly");

    let config = entries
        .iter()
        .find(|e| e.rel == ".config/caelestia")
        .expect("the skel must carry .config/caelestia so the shell can bootstrap");
    assert!(
        config.src.join("hypr-vars.lua").is_file(),
        "hyprland.lua requires hypr-vars, so the skel has to provide it"
    );
}

#[test]
fn end4pc_skel_ships_the_symlink_the_shell_starts_with() {
    // `qs -c end4-pC` resolves `<dir>/quickshell/<name>/shell.qml` by exact
    // name under the XDG config directories — `/usr/share/quickshell` is not
    // on that list. So the skel ships `~/.config/quickshell/end4-pC` as a
    // symlink into the package, and both ends of the link must be spelled the
    // way the package installs them. Without the link a switch starts `qs`
    // directly, which finds no config and exits at once; the switch then waits
    // out the whole notification-bus grace against a shell that is dead.
    let shell = &end4pc::SHELL;
    let entries = plan(
        shell,
        &Hyprland,
        Path::new("/nonexistent-backups"),
        &skel_for("hyprland", "end4pc"),
        &mut |_| {},
    )
    .expect("the real end4pc skel should plan cleanly");

    let config = entries
        .iter()
        .find(|e| e.rel == ".config/quickshell/end4-pC")
        .expect("the skel must carry the config dir the shell starts with");
    let meta = std::fs::symlink_metadata(&config.src).expect("the link is on disk");
    assert!(
        meta.file_type().is_symlink(),
        "the config dir has to be a symlink into the package"
    );
    let target = std::fs::read_link(&config.src).expect("the link points somewhere");
    assert_eq!(
        target.as_path(),
        Path::new("/usr/share/quickshell/end4-pC"),
        "the link must point at the payload directory the package installs"
    );

    // And the name it is delivered as is the name `-c` asks for: a symlink
    // named anything else — `end4-PC` was tried — is invisible to the shell.
    let name = Path::new(shell.config_dir.expect("end4pc names its config dir"))
        .file_name()
        .expect("a config dir has a name")
        .to_str()
        .expect("config names are utf-8");
    assert_eq!(shell.start_argv, &["qs", "-c", name][..]);
}

#[test]
fn a_shared_xdg_directory_is_never_replaced_as_a_whole() {
    // `swap` renames the live path aside and puts the staged one in place; it
    // does not merge. Any directory shared with the rest of the desktop listed
    // as a skel extra would take the user's other files with it.
    for shell in [&dms::SHELL, &noctalia::SHELL, &caelestia::SHELL] {
        for rel in shell.skel_extra_paths {
            assert!(
                !matches!(
                    *rel,
                    ".config"
                        | ".local"
                        | ".local/share"
                        | ".local/state"
                        | ".local/share/applications"
                        | ".config/gtk-3.0"
                        | ".config/gtk-4.0"
                ),
                "{} lists {rel}, which would replace a directory the user shares \
                 with the rest of their desktop",
                shell.display_name
            );
        }
    }
}

#[test]
fn dms_state_and_theme_paths_reach_the_plan_from_the_real_skel() {
    let entries = plan(
        &dms::SHELL,
        &Hyprland,
        Path::new("/nonexistent-backups"),
        &skel_for("hyprland", "dms"),
        &mut |_| {},
    )
    .unwrap();
    // Individual files, not the gtk directories: the plan is what decides how
    // much of the user's desktop gets replaced, and this list is the contract.
    for rel in [
        ".config/DankMaterialShell",
        ".config/hypr",
        ".config/gtk-3.0/settings.ini",
        ".config/gtk-4.0/settings.ini",
        ".config/gtk-4.0/gtk.css",
        ".local/state/DankMaterialShell",
        LAUNCHER,
    ] {
        assert!(
            entries.iter().any(|e| e.rel == rel),
            "{rel} should be part of the switch to dms; planned: {:?}",
            entries.iter().map(|e| &e.rel).collect::<Vec<_>>()
        );
    }
}

#[test]
fn an_aside_is_discarded_only_when_a_backup_reproduces_it_byte_for_byte() {
    // This runs after `start` and `verify` succeeded, so the aside can no
    // longer be rolled back to. It is still only thrown away if the backup
    // holds the same number of files and bytes at the same path: dropping
    // one no backup holds would destroy the last copy of what the user had.
    let t = Temp::new("discard-asides");
    let home = t.path().join("home");
    let backups = t.path().join("backups");

    write(
        &backups.join("20261006-hyprland-dms/home/.config/gtk-3.0/settings.ini"),
        "[Settings]",
    );
    write(
        &backups.join("20261006-hyprland-dms/home/.config/gtk-4.0/gtk.css"),
        "the skel's css",
    );
    write(&backups.join("20261006-hyprland-dms/manifest.toml"), "");

    // A newer backup of the other shell, holding a different bytes at the same
    // path. It is the one a lookup for "the newest backup holding this" would
    // return, and it is the reason that lookup was wrong: the bytes of a copy
    // have nothing to do with which shell the directory is named after, so
    // insisting on the newest left a verified `.pre-*` file on disk that
    // nothing would ever read again.
    write(
        &backups.join("20261007-hyprland-noctalia/home/.config/gtk-3.0/settings.ini"),
        "[Settings]\nnoctalia had other ideas",
    );
    write(
        &backups.join("20261007-hyprland-noctalia/manifest.toml"),
        "",
    );

    let identical = home.join(".config/gtk-3.0/.settings.ini.pre-dms-20261006-102219");
    write(&identical, "[Settings]");
    let different = home.join(".config/gtk-4.0/.gtk.css.pre-dms-20261006-102219");
    write(&different, "the user's own css");
    let unbacked = home.join(".config/noctalia/.settings.json.pre-dms-20261006-102219");
    write(&unbacked, "no backup has this");

    let asides = vec![
        Aside {
            rel: ".config/gtk-3.0/settings.ini".to_string(),
            kept: Some(identical.clone()),
        },
        Aside {
            rel: ".config/gtk-4.0/gtk.css".to_string(),
            kept: Some(different.clone()),
        },
        Aside {
            rel: ".config/noctalia/settings.json".to_string(),
            kept: Some(unbacked.clone()),
        },
        Aside {
            rel: ".config/DankMaterialShell".to_string(),
            kept: None,
        },
    ];

    assert_eq!(
        discard_asides(&asides, &backups),
        (1, 2),
        "only the one the backup already holds may go"
    );
    assert!(
        !identical.exists(),
        "the copy in the backup must have replaced it, not been deleted with it"
    );
    assert!(
        different.exists(),
        "a backup holding other bytes is not a match"
    );
    assert!(
        unbacked.exists(),
        "an aside no backup holds is the last copy"
    );
}

#[test]
fn an_aside_is_kept_when_no_backup_holds_its_path_at_all() {
    // The case a switch that lost its backup directory lands in: every
    // aside is then the only surviving copy of the config it came from.
    let t = Temp::new("discard-without-backups");
    let home = t.path().join("home");
    let aside = home.join(".config/hypr/.config.toml.pre-dms-20261006-102219");
    write(&aside, "the only copy");

    let asides = vec![Aside {
        rel: ".config/hypr/config.toml".to_string(),
        kept: Some(aside.clone()),
    }];

    assert_eq!(
        discard_asides(&asides, &t.path().join("backups-that-do-not-exist")),
        (0, 1)
    );
    assert!(aside.exists());
}

#[test]
fn a_shells_state_comes_back_from_the_backup_that_holds_it() {
    // Noctalia keeps 389 of the 445 files a real backup takes under its state
    // directory rather than its config, so a switch that cleans that directory
    // away has to be able to hand it back. Nothing planned it before: it was
    // copied into the backup and never read again, which only stayed safe
    // because nothing ever deleted it.
    let t = Temp::new("state-from-backup");
    let backups = t.path().join("backups");

    make_backup(
        &backups,
        "20261006-hyprland-noctalia",
        &[".local/state/noctalia"],
    );

    let entries = plan(
        &noctalia::SHELL,
        &Hyprland,
        &backups,
        &skel_for("hyprland", "noctalia"),
        &mut |_| {},
    )
    .unwrap();

    let state = entries
        .iter()
        .find(|e| e.rel == ".local/state/noctalia")
        .expect("a backup that holds the state must be planned, or cleaning it loses it");
    assert_eq!(
        state.origin,
        Origin::Backup(backups.join("20261006-hyprland-noctalia"))
    );
}

#[test]
fn state_that_no_backup_holds_is_left_to_start_empty() {
    // The other half of the same rule. Refusing to switch would be wrong — a
    // shell with no state is a shell running on its defaults — but saying so is
    // not optional, because silence here reads as "it was restored".
    let t = Temp::new("state-empty");
    let mut said = Vec::new();

    let entries = plan(
        &noctalia::SHELL,
        &Hyprland,
        &t.path().join("backups-that-do-not-exist"),
        &skel_for("hyprland", "noctalia"),
        &mut |line| said.push(line.to_string()),
    )
    .unwrap();

    assert!(!entries.iter().any(|e| e.rel == ".local/state/noctalia"));
    let line = said
        .iter()
        .find(|l| l.contains("starts with no state carried over"))
        .unwrap_or_else(|| panic!("the panel has to say the state was not brought back: {said:?}"));
    // `pull` writes the prefix itself; the hint is only what comes after it. The
    // two were briefly written as if each owned the whole sentence, and the panel
    // printed the first half twice before the second.
    assert_eq!(
        line.matches("no backup or skel holds").count(),
        1,
        "the prefix is said once: {line}"
    );
}

#[test]
fn the_outgoing_shells_own_files_are_cleaned_once_the_backup_reproduces_them() {
    // The point of `clean`: leaving a shell behind must not leave it installed
    // in the user's home as well. Both paths go, and the backup taken for this
    // very switch is untouched, because it is the only way back.
    let t = Temp::new("clean");
    let home = t.path().join("home");
    let backup = t.path().join("backups/20261006-hyprland-noctalia");

    write(
        &home.join(".config/DankMaterialShell/settings.json"),
        "the setup",
    );
    write(
        &home.join(".local/state/DankMaterialShell/session.json"),
        "the session",
    );
    write(
        &backup.join("home/.config/DankMaterialShell/settings.json"),
        "the setup",
    );
    write(
        &backup.join("home/.local/state/DankMaterialShell/session.json"),
        "the session",
    );

    let mut said = Vec::new();
    assert_eq!(
        clean(
            &dms::SHELL,
            &noctalia::SHELL,
            &home,
            backup.parent().unwrap(),
            &backup,
            &mut |line| { said.push(line.to_string()) }
        ),
        (2, 0),
        "both paths are in the backup byte for byte, so both may go"
    );

    assert!(
        !home.join(".config/DankMaterialShell").exists(),
        "the config of the shell just left must not survive it"
    );
    assert!(
        !home.join(".local/state/DankMaterialShell").exists(),
        "nor its state"
    );
    assert!(
        backup
            .join("home/.config/DankMaterialShell/settings.json")
            .exists(),
        "cleaning must not touch the backup it is relying on"
    );
    assert!(said
        .iter()
        .any(|l| l.contains("removed .config/DankMaterialShell")));
}

#[test]
fn something_with_no_data_in_it_is_discarded_without_a_backup() {
    // The rule for the other half of the list: what carries data is backed up,
    // what does not is simply cleaned. A cache a shell regenerates on its next
    // start and a theme file nobody edits are worth no bytes of a backup and no
    // byte-for-byte comparison, so they go on the strength of the declaration
    // alone.
    let t = Temp::new("discard");
    let home = t.path().join("home");
    let backup = t.path().join("backups/20261007-hyprland-dms");

    write(&home.join(".cache/DankMaterialShell/dms-colors.json"), "{}");
    write(&home.join(".config/ghostty/themes/dankcolors"), "palette");
    write(
        &home.join(".local/share/color-schemes/DankMatugenDark.colors"),
        "ini",
    );

    let mut said = Vec::new();
    assert_eq!(
        clean(
            &dms::SHELL,
            &noctalia::SHELL,
            &home,
            backup.parent().unwrap(),
            &backup,
            &mut |line| { said.push(line.to_string()) }
        ),
        (3, 0),
        "no copy of any of it exists and none is needed"
    );

    for gone in [
        ".cache/DankMaterialShell",
        ".config/ghostty/themes/dankcolors",
        ".local/share/color-schemes/DankMatugenDark.colors",
    ] {
        assert!(
            !home.join(gone).exists(),
            "{gone} holds no data, so leaving a shell must take it away"
        );
    }
    assert!(
        !backup.exists(),
        "cleaning a discard path must not invent a backup for it"
    );
    assert!(said
        .iter()
        .any(|l| l.contains("discarded .cache/DankMaterialShell")));
}

#[test]
fn a_path_the_backup_does_not_reproduce_is_kept_when_the_shell_leaves() {
    // The shell was running when the backup was taken and was stopped a moment
    // later, so it can easily have written something in between. Deleting on a
    // backup that has gone stale would throw away the only copy of exactly the
    // change that mattered.
    let t = Temp::new("clean-stale");
    let home = t.path().join("home");
    let backup = t.path().join("backups/20261006-hyprland-noctalia");

    write(
        &home.join(".config/DankMaterialShell/settings.json"),
        "the setup",
    );
    write(
        &home.join(".local/state/DankMaterialShell/session.json"),
        "written after the backup",
    );
    write(
        &backup.join("home/.config/DankMaterialShell/settings.json"),
        "the setup",
    );
    write(
        &backup.join("home/.local/state/DankMaterialShell/session.json"),
        "written before the backup",
    );

    let mut said = Vec::new();
    assert_eq!(
        clean(
            &dms::SHELL,
            &noctalia::SHELL,
            &home,
            backup.parent().unwrap(),
            &backup,
            &mut |line| { said.push(line.to_string()) }
        ),
        (1, 1),
        "only the path the backup still describes may go"
    );
    assert!(!home.join(".config/DankMaterialShell").exists());
    assert!(
        home.join(".local/state/DankMaterialShell/session.json")
            .exists(),
        "a state the backup no longer matches is the last copy"
    );
    assert!(said
        .iter()
        .any(|l| l.contains("kept .local/state/DankMaterialShell")));
}

#[test]
fn a_shell_never_cleans_a_path_the_shell_being_switched_to_owns() {
    // Two shells sharing a path is not possible in the registry today, and
    // `plan` would write the target's copy over the outgoing one anyway. The
    // guard is here so that a future registry which does share a path loses the
    // clean step rather than the target's live configuration.
    let t = Temp::new("clean-overlap");
    let home = t.path().join("home");
    let backup = t.path().join("backups/20261006-hyprland-noctalia");

    write(
        &home.join(".config/DankMaterialShell/settings.json"),
        "shared",
    );
    write(
        &backup.join("home/.config/DankMaterialShell/settings.json"),
        "shared",
    );

    let thief = Shell {
        config_dir: Some(".config/DankMaterialShell"),
        state_dir: None,
        ..dms::SHELL
    };

    assert_eq!(
        clean(
            &dms::SHELL,
            &thief,
            &home,
            backup.parent().unwrap(),
            &backup,
            &mut |_| {},
        ),
        (0, 1)
    );
    assert!(
        home.join(".config/DankMaterialShell/settings.json")
            .exists(),
        "the path the target owns must be left for the target to write"
    );
}

#[test]
fn clean_refuses_any_path_that_reaches_the_backups() {
    // The registry test says no declared path sits anywhere near the backups;
    // this is the other half — the refusal itself, exercised. It matters that
    // the refusal fires *before* the byte-for-byte check, because that check
    // cannot be relied on here: a path above the backups would have to be
    // compared against a copy the backup never takes, and the day it did compare
    // equal, the deletion would take the recovery copies with the config. The
    // refusal is what makes the answer independent of the comparison.
    let t = Temp::new("clean-reaches-backups");
    let home = t.path().join("home");
    let backups = home.join(".local/share/backups");
    let backup = backups.join("20261006-hyprland-noctalia");

    write(&home.join(".local/share/dankcal/dankcal.db"), "database");
    write(
        &backup.join("home/.local/share/dankcal/dankcal.db"),
        "database",
    );

    // A shell wrongly declaring one directory above the backups as its config.
    let sinner = Shell {
        config_dir: Some(".local/share"),
        state_dir: None,
        ..noctalia::SHELL
    };

    let mut said = Vec::new();
    assert_eq!(
        clean(
            &sinner,
            &dms::SHELL,
            &home,
            &backups,
            &backup,
            &mut |line| said.push(line.to_string()),
        ),
        (0, 1)
    );
    assert!(
        said.iter().any(|l| l.contains("reaches the backups")),
        "the refusal must say why it refused, not fall through to the byte check"
    );
    assert!(
        home.join(".local/share/backups/20261006-hyprland-noctalia")
            .exists(),
        "the backups themselves are the one thing a switch never cleans"
    );
}

#[test]
fn the_backup_is_taken_again_after_the_shell_stops_so_its_state_can_go() {
    // The window this closes: the backup is written before anything is
    // installed, which is the order that makes the install reversible, and the
    // shell being left runs through the whole install. Noctalia wrote to its own
    // state in that window on the machine this was found on, `clean` then found
    // a backup that no longer described the file and quite correctly kept the
    // file — and the switch finished with a state directory it had just said it
    // removed. Stopping the shell ends the window, and taking the same sources
    // into the same directory once more makes the backup describe the file as it
    // was left, so the path goes like every other one.
    let t = Temp::new("refresh");
    let home = t.path().join("home");
    let backups = t.path().join("backups");
    let name = "20261008-hyprland-noctalia";

    write(
        &home.join(".local/state/noctalia/clipboard/index.enc"),
        "before",
    );
    let sources =
        crate::switch::backup_sources_for(&home, &Hyprland, Some(&noctalia::SHELL), &dms::SHELL);
    crate::backup::Backup::create(&backups, name, &sources, &home).unwrap();
    let backup_root = backups.join(name);

    // What the shell wrote while the install was running.
    write(
        &home.join(".local/state/noctalia/clipboard/index.enc"),
        "after",
    );

    let mut said = Vec::new();
    refresh_backup(
        &noctalia::SHELL,
        &dms::SHELL,
        &home,
        &backups,
        &backup_root,
        &Hyprland,
        &mut |line| said.push(line.to_string()),
    );

    assert_eq!(
        std::fs::read_to_string(backup_root.join("home/.local/state/noctalia/clipboard/index.enc"))
            .unwrap(),
        "after",
        "the copy has to be what the shell last wrote, not what it wrote before the install"
    );
    assert!(said
        .iter()
        .any(|l| l.contains("took the backup again now that Noctalia has stopped")));

    assert_eq!(
        clean(
            &noctalia::SHELL,
            &dms::SHELL,
            &home,
            backup_root.parent().unwrap(),
            &backup_root,
            &mut |line| said.push(line.to_string())
        ),
        (1, 0),
        "the backup now describes the file, so the state goes"
    );
    assert!(
        !home.join(".local/state/noctalia").exists(),
        "leaving a shell must not leave its state behind in the home"
    );
}

#[test]
fn a_companions_data_comes_back_from_the_backup_when_the_shell_returns() {
    // Nothing installs a calendar database, so no skel carries it: the backup
    // kept for DMS is the only source there is. A backup of the other shell
    // that happens to hold the same path is not a source at all — it holds
    // whatever was on disk with Noctalia in charge, which for this path means
    // nothing DMS would recognise.
    let t = Temp::new("companion-from-backup");
    let backups = t.path().join("backups");
    let skel = t.path().join("skel");

    make_backup(&backups, "20260101-hyprland-dms", &[".local/share/dankcal"]);
    make_backup(
        &backups,
        "20261008-hyprland-noctalia",
        &[".local/share/dankcal"],
    );
    write(&skel.join(".config/hypr/config/autostart.lua"), "dms");

    let mut said = Vec::new();
    let entries = plan(&dms::SHELL, &Hyprland, &backups, &skel, &mut |line| {
        said.push(line.to_string())
    })
    .unwrap();

    let data = entries
        .iter()
        .find(|e| e.rel == ".local/share/dankcal")
        .expect("DMS's own backup holds the calendar, so it must be planned");
    assert_eq!(
        data.origin,
        Origin::Backup(backups.join("20260101-hyprland-dms")),
        "the backup kept for DMS, not the newest one on disk"
    );

    // Nothing holds this one and no skel carries it. Not fatal, and not
    // silent: a calendar that did not come back is worth saying out loud.
    assert!(!entries.iter().any(|e| e.rel == ".config/dankcal"));
    assert!(
        said.iter()
            .any(|l| l.contains("DankMaterialShell starts without it")),
        "the panel has to say the companion data was not brought back: {said:?}"
    );
}

#[test]
fn a_companions_data_goes_only_when_this_switchs_backup_reproduces_it() {
    // The same byte-for-byte rule `config_dir` is held to, because that is what
    // these paths are: data. The empty directory is in here deliberately — a
    // directory with nothing in it has nothing to lose either, and the backup
    // records it as present, so it has to be matched rather than left behind as
    // the one piece of the shell that never goes.
    let t = Temp::new("companion-clean");
    let home = t.path().join("home");
    let backup = t.path().join("backups/20261006-hyprland-dms");

    write(&home.join(".local/share/dankcal/dankcal.db"), "database");
    write(
        &backup.join("home/.local/share/dankcal/dankcal.db"),
        "database",
    );
    std::fs::create_dir_all(home.join(".config/dankcal")).unwrap();
    std::fs::create_dir_all(backup.join("home/.config/dankcal")).unwrap();
    write(&home.join(".config/dgop/colors.json"), "the live palette");
    write(
        &backup.join("home/.config/dgop/colors.json"),
        "what the backup took",
    );

    assert_eq!(
        clean(
            &dms::SHELL,
            &noctalia::SHELL,
            &home,
            backup.parent().unwrap(),
            &backup,
            &mut |_| {}
        ),
        (2, 1),
        "the database and the empty directory go, the palette that moved on stays"
    );
    assert!(
        !home.join(".local/share/dankcal").exists(),
        "leaving DMS must not leave its companions' data in the home"
    );
    assert!(
        !home.join(".config/dankcal").exists(),
        "an empty companion directory is still the shell's to remove"
    );
    assert!(
        home.join(".config/dgop/colors.json").exists(),
        "a file the backup no longer describes must be kept, not deleted"
    );
}
