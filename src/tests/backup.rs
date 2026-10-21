use super::*;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("rakuos-shell-picker-tests")
        .join("backup")
        .join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

#[test]
fn a_backup_of_a_single_file_round_trips() {
    // Noctalia's launcher is a file, not a directory, and a switch replaces it.
    // Backup, verify and restore all have to cope with a tree that holds one
    // file under an empty key. This was found by running the backup against a
    // real setup: every unit test used a directory, so `read_dir` on the launcher
    // failed with "Not a directory" and no test had caught it.
    let root = sandbox("single-file");
    let home = root.join("home");
    let launcher = home.join(".local/share/applications/dev.noctalia.Noctalia.desktop");
    write(&launcher, "[Desktop Entry]\nName=Noctalia\nExec=noctalia\n");

    let backup = Backup::create(
        &root.join("backups"),
        "launcher",
        std::slice::from_ref(&launcher),
        &home,
    )
    .expect("a single file is a valid backup source");
    backup.verify().expect("verify must read a file back");

    let key = ".local/share/applications/dev.noctalia.Noctalia.desktop";
    let tree = &backup.manifest.trees[key];
    assert_eq!(tree.files, 1);
    assert_eq!(tree.bytes, 44, "the byte count is the file's own");
    assert!(
        tree.entries.contains_key(""),
        "recorded under the empty key"
    );

    // And it comes back: replace it with something else, restore, compare.
    write(&launcher, "clobbered");
    let files = backup
        .restore(&home)
        .expect("restore must put a single file back");
    assert_eq!(files, 1);
    assert_eq!(
        std::fs::read_to_string(&launcher).unwrap(),
        "[Desktop Entry]\nName=Noctalia\nExec=noctalia\n"
    );
}

#[test]
fn a_symlinked_file_is_recorded_as_a_link() {
    // The launcher can be a symlink to something in /usr/share, which is what a
    // package install leaves behind. Restoring has to reproduce the link rather
    // than a copy of its target, or the user silently loses the link.
    let root = sandbox("symlinked-file");
    let home = root.join("home");
    let target = root.join("elsewhere.desktop");
    write(&target, "[Desktop Entry]\n");
    let launcher = home.join(".local/share/applications/dev.noctalia.Noctalia.desktop");
    fs::create_dir_all(launcher.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&target, &launcher).unwrap();

    let backup = Backup::create(
        &root.join("backups"),
        "linked",
        std::slice::from_ref(&launcher),
        &home,
    )
    .expect("a symlinked file is a valid source");
    backup
        .verify()
        .expect("verify must follow the recorded link");

    let key = ".local/share/applications/dev.noctalia.Noctalia.desktop";
    match backup.manifest.trees[key].entries.get("") {
        Some(Entry::Symlink { target: recorded }) => {
            assert_eq!(recorded, &target, "recorded as a link, not a copy");
        }
        other => panic!("expected a symlink entry, got {other:?}"),
    }
}

#[test]
fn a_second_backup_under_the_same_name_replaces_the_first() {
    // One backup per day is the point of dating them by day, so a second
    // switch on that day has to take the place of the first rather than sit
    // beside it as `20261006-2`. What must come out the other end is the
    // *new* copy, complete and verified, with nothing of the old one left
    // to be mistaken for it.
    let dir = sandbox("replace");
    let home = dir.join("home");
    write(
        &home.join(".config/noctalia/settings.json"),
        "{\"old\":true}",
    );
    let backups = dir.join("backups");

    let first = Backup::create(
        &backups,
        "20261006-hyprland-dms",
        &[home.join(".config/noctalia")],
        &home,
    )
    .expect("the first backup of the day should be written");
    let first_files = first.file_count();
    assert_eq!(first_files, 1);

    write(
        &home.join(".config/noctalia/settings.json"),
        "{\"new\":true}",
    );
    write(&home.join(".config/noctalia/extra.json"), "{}");
    let second = Backup::create(
        &backups,
        "20261006-hyprland-dms",
        &[home.join(".config/noctalia")],
        &home,
    )
    .expect("the second backup of the day should replace the first");

    assert_eq!(
        second.file_count(),
        2,
        "the manifest must describe what is actually there now"
    );
    second.verify().expect("the replacement must verify");
    assert_eq!(
        fs::read_to_string(second.root.join("home/.config/noctalia/settings.json")).unwrap(),
        "{\"new\":true}",
        "the old copy must be gone, not still on disk under the new manifest"
    );

    let names: Vec<String> = fs::read_dir(&backups)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        names,
        vec!["20261006-hyprland-dms".to_string()],
        "a day of switching must leave one directory, not one per click: {names:?}"
    );
}

#[test]
fn a_backup_that_cannot_be_built_leaves_the_previous_one_alone() {
    // Replacing is only safe because the new copy is finished and verified
    // first. If building it fails — here, a source outside `$HOME`, which
    // `relative_to_home` refuses — then nothing has been moved yet and the
    // backup the switch is relying on must still be readable.
    let dir = sandbox("keep-previous");
    let home = dir.join("home");
    write(
        &home.join(".config/noctalia/settings.json"),
        "{\"keep\":true}",
    );
    let backups = dir.join("backups");

    Backup::create(
        &backups,
        "20261006-hyprland-dms",
        &[home.join(".config/noctalia")],
        &home,
    )
    .expect("the first backup should be written");

    let outside = dir.join("not-in-home.json");
    write(&outside, "system file");
    let err = Backup::create(
        &backups,
        "20261006-hyprland-dms",
        &[home.join(".config/noctalia"), outside],
        &home,
    )
    .expect_err("a source outside home must be refused");
    assert!(
        format!("{err:#}").contains("outside"),
        "the refusal has to say why: {err:#}"
    );

    let previous = Backup {
        root: backups.join("20261006-hyprland-dms"),
        manifest: read_manifest(&backups.join("20261006-hyprland-dms")),
    };
    previous
        .verify()
        .expect("the previous backup must be untouched after a failed build");

    let leftovers: Vec<String> = fs::read_dir(&backups)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains(".new") || n.contains(".old"))
        .collect();
    assert!(
        leftovers.is_empty(),
        "a failed build must not leave scratch directories behind: {leftovers:?}"
    );
}

#[test]
fn rollback_copies_are_not_carried_into_the_backup() {
    // `swap` renames what it is replacing to `.{name}.pre-...` inside the
    // very directory being backed up. Left in, each generation is copied
    // into the next backup and they compound: this is how 214 of them
    // ended up inside four directories.
    let dir = sandbox("no-asides");
    let home = dir.join("home");
    write(&home.join(".config/gtk-3.0/settings.ini"), "[Settings]\n");
    write(
        &home.join(".config/gtk-3.0/.settings.ini.pre-dms-20261006-102219"),
        "litter",
    );
    write(
        &home.join(".config/gtk-3.0/.not.an aside"),
        "a name that only looks like one",
    );

    let backup = Backup::create(
        &dir.join("backups"),
        "20261006-hyprland-dms",
        &[home.join(".config/gtk-3.0")],
        &home,
    )
    .expect("the tree should copy");

    assert_eq!(
        backup.file_count(),
        2,
        "only the user's own files belong in the backup"
    );
    backup.verify().expect("the filtered copy must verify");
    assert!(!backup
        .root
        .join("home/.config/gtk-3.0/.settings.ini.pre-dms-20261006-102219")
        .exists());
}

/// Read a manifest back off disk, for checking a backup this test did not build.
fn read_manifest(root: &Path) -> Manifest {
    let text = fs::read_to_string(root.join("manifest.toml")).expect("manifest");
    toml::from_str(&text).expect("manifest should parse")
}

#[test]
fn restoring_a_backup_taken_before_the_filter_writes_its_asides_back_too() {
    // Backups written before rollback copies were filtered record them next
    // to the user's own files. Restoring has to reproduce every one of them,
    // because `restore` compares what it wrote against the manifest: an
    // aside quietly skipped would abort the restore partway through, which
    // is the worst moment to stop.
    let dir = sandbox("restore-old-style");
    let source = dir.join("source/.config/gtk-3.0");
    write(&source.join("settings.ini"), "[Settings]\n");
    write(
        &source.join(".settings.ini.pre-dms-20261006-102219"),
        "litter",
    );

    let root = dir.join("backups/20261005-hyprland-dms");
    let stored = root.join("home/.config/gtk-3.0");
    let entries = copy_tree(&source, &stored, false).expect("the whole tree copies");
    assert_eq!(entries.files, 2, "the aside belongs to this old backup");

    let mut manifest = Manifest::default();
    manifest
        .trees
        .insert(".config/gtk-3.0".to_string(), entries);
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("manifest.toml"),
        toml::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();

    let backup = Backup { root, manifest };
    backup.verify().expect("the old backup must verify");

    let home = dir.join("restored");
    backup
        .restore(&home)
        .expect("an old backup must restore whole");
    assert!(
        home.join(".config/gtk-3.0/.settings.ini.pre-dms-20261006-102219")
            .exists(),
        "what the manifest recorded has to come back"
    );
}

#[test]
fn only_the_exact_rollback_name_is_treated_as_an_aside() {
    // The filter decides what the user's configuration is allowed to lose,
    // so it has to be exact. Anything short of the shape `swap` writes —
    // leading dot, `.pre-`, then a target, an eight digit date and a six
    // digit time — must be copied like any other file.
    for name in [
        ".settings.ini.pre-dms-20261006-102219",
        ".config.toml.pre-noctalia-19991231-235959",
        ".hyprland.conf.pre-hyprland-20260101-000000",
    ] {
        assert!(is_aside(name), "{name} is an aside and must be skipped");
    }

    for name in [
        "settings.ini",
        ".settings.ini",
        ".settings.ini.pre-",
        ".settings.ini.pre-dms-20261006",
        ".settings.ini.pre-dms-20261006-10221",
        ".settings.ini.pre-dms-2026-1006-102219",
        ".settings.ini.pre-dms-20261006-102219-extra",
        "notes about previews-20261006",
        "pre-20261006",
    ] {
        assert!(!is_aside(name), "{name} is the user's own file, keep it");
    }
}

#[test]
fn a_backup_copies_the_whole_tree() {
    let root = sandbox("whole-tree");
    let home = root.join("home");
    write(&home.join(".config/noctalia/settings.json"), "{\"a\":1}");
    write(&home.join(".config/noctalia/themes/dark/gtk.css"), "dark");

    let backup = Backup::create(
        &root.join("backups"),
        "1234-hyprland-noctalia",
        &[home.join(".config/noctalia")],
        &home,
    )
    .unwrap();

    assert_eq!(backup.file_count(), 2);
    assert_eq!(backup.manifest.absent, Vec::<String>::new());
    assert!(backup.manifest.trees.contains_key(".config/noctalia"));
}

#[test]
fn a_backup_never_writes_into_a_directory_that_is_already_there() {
    // SAFETY.md rule 2. When every backup had a unique name this was
    // settled by refusing a second use of the same name. Dated by day the
    // name is *meant* to be used twice now, so the invariant has to be held
    // another way: the new copy is built and verified in a directory of its
    // own first, and only then moved in as a whole. What must never come
    // out is a directory holding part of the first switch and part of the
    // second — and equally, no half-written copy beside either of them.
    let root = sandbox("reuse");
    let home = root.join("home");
    write(&home.join(".config/noctalia/a"), "first");
    let sources = [home.join(".config/noctalia")];
    let backups = root.join("backups");

    let first = Backup::create(&backups, "same-name", &sources, &home).unwrap();

    write(&home.join(".config/noctalia/a"), "second");
    write(&home.join(".config/noctalia/b"), "second");
    let second = Backup::create(&backups, "same-name", &sources, &home)
        .expect("a second switch on the same day replaces rather than refusing");

    assert_eq!(
        first.root, second.root,
        "one name, one directory — the point of dating them by day"
    );
    assert_eq!(
        std::fs::read_to_string(second.root.join("home/.config/noctalia/a")).unwrap(),
        "second",
        "the old copy must be replaced whole, not written over in place"
    );
    assert_eq!(
        second.file_count(),
        2,
        "the manifest must describe the new copy only"
    );
    second
        .verify()
        .expect("what replaced the first backup must verify");
    first.verify().expect_err(
        "the first handle now names a directory holding the second switch, \
         and must not pass as if it still held the first",
    );

    let stray: Vec<String> = fs::read_dir(&backups)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n != "same-name")
        .collect();
    assert!(
        stray.is_empty(),
        "neither switch may leave a scratch or a moved-aside copy behind: {stray:?}"
    );
}

#[test]
fn a_missing_source_is_recorded_rather_than_failing() {
    let root = sandbox("missing");
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();

    // A shell that was never opened has no config; that is not a failure.
    let backup = Backup::create(
        &root.join("backups"),
        "1-hyprland-dms",
        &[home.join(".config/dms")],
        &home,
    )
    .unwrap();

    assert_eq!(backup.file_count(), 0);
    assert_eq!(backup.manifest.absent, [".config/dms"]);
}

#[test]
fn verification_catches_a_damaged_copy() {
    let root = sandbox("damaged");
    let home = root.join("home");
    write(&home.join(".config/noctalia/settings.json"), "{\"a\":1}");
    let backup = Backup::create(
        &root.join("backups"),
        "1-hyprland-noctalia",
        &[home.join(".config/noctalia")],
        &home,
    )
    .unwrap();

    // Something eats the copy after it was taken.
    fs::write(
        backup.root.join("home/.config/noctalia/settings.json"),
        "{\"a\":",
    )
    .unwrap();

    let err = backup.verify().expect_err("a truncated copy must not pass");
    let text = format!("{err:#}");
    assert!(text.contains("does not match its manifest"), "{text}");
    // The complaint has to point at the file that went wrong.
    assert!(text.contains("settings.json"), "{text}");
}

#[test]
fn verification_catches_a_file_that_vanished() {
    let root = sandbox("vanished");
    let home = root.join("home");
    write(&home.join(".config/noctalia/keep.json"), "{}");
    write(&home.join(".config/noctalia/gone.json"), "{}");
    let config = home.join(".config/noctalia");
    let backup = Backup::create(
        &root.join("backups"),
        "1-hyprland-noctalia",
        std::slice::from_ref(&config),
        &home,
    )
    .unwrap();

    fs::remove_file(backup.root.join("home/.config/noctalia/gone.json")).unwrap();

    let err = backup.verify().expect_err("a missing copy must not pass");
    let text = format!("{err:#}");
    assert!(text.contains("gone.json"), "{text}");
}

#[test]
fn restore_puts_the_original_back() {
    let root = sandbox("restore");
    let home = root.join("home");
    let config = home.join(".config/noctalia");
    write(&config.join("settings.json"), "{\"a\":1}");
    let backup = Backup::create(
        &root.join("backups"),
        "1-hyprland-noctalia",
        std::slice::from_ref(&config),
        &home,
    )
    .unwrap();

    // The switch replaces the config, then fails and has to undo it.
    fs::remove_dir_all(&config).unwrap();
    write(&config.join("settings.json"), "{\"a\":999}");

    let restored = backup.restore(&home).unwrap();
    assert_eq!(restored, 1);
    assert_eq!(
        fs::read_to_string(config.join("settings.json")).unwrap(),
        "{\"a\":1}"
    );
}

#[test]
fn a_path_outside_home_is_refused() {
    let root = sandbox("outside");
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();

    let err = Backup::create(
        &root.join("backups"),
        "1-hyprland-noctalia",
        &[PathBuf::from("/etc")],
        &home,
    )
    .expect_err("system paths must be refused");
    assert!(format!("{err:#}").contains("outside"), "{err:#}");
}

#[test]
fn a_relative_path_is_refused() {
    let root = sandbox("relative");
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();

    let err = Backup::create(
        &root.join("backups"),
        "1-x-y",
        &[PathBuf::from(".config")],
        &home,
    )
    .expect_err("relative paths must be refused");
    assert!(format!("{err:#}").contains("not an absolute"), "{err:#}");
}

#[test]
fn a_directory_name_with_a_path_is_refused() {
    let root = sandbox("sneaky-name");
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();

    let err = Backup::create(&root.join("backups"), "../escape", &[], &home)
        .expect_err("a name must not be able to escape the backup directory");
    assert!(
        format!("{err:#}").contains("refusing a backup directory"),
        "{err:#}"
    );
}

#[test]
fn symlinks_survive_a_round_trip() {
    let root = sandbox("symlinks");
    let home = root.join("home");
    let config = home.join(".config/noctalia");
    write(&config.join("real.json"), "{}");
    symlink("real.json", config.join("link.json")).unwrap();

    let backup = Backup::create(
        &root.join("backups"),
        "1-hyprland-noctalia",
        std::slice::from_ref(&config),
        &home,
    )
    .unwrap();
    assert!(backup.verify().is_ok());

    // Restoring must not turn a symlink into a copy of its target.
    fs::remove_file(config.join("link.json")).unwrap();
    backup.restore(&home).unwrap();
    assert!(config.join("link.json").is_symlink());
    assert_eq!(
        fs::read_link(config.join("link.json")).unwrap(),
        PathBuf::from("real.json")
    );
}

/// Copies the real user config. Ignored so `cargo test` never touches it by
/// accident; run with `cargo test -- --include-ignored`.
///
/// This is the test that found the single-file bug: every unit test here used
/// a directory as its source, and Noctalia's launcher is a file, so nothing
/// covered it until this ran against an actual desktop..
///
/// Restoring is deliberately not part of this: writing over a live desktop
/// is not something a test should do, even into its own backup.
#[test]
#[ignore = "copies the real user config"]
fn the_real_noctalia_setup_backs_up_and_verifies() {
    use crate::compositor::hyprland::shells::noctalia::SHELL as NOCTALIA;

    let home = PathBuf::from(std::env::var_os("HOME").expect("HOME"));
    let dest = std::env::temp_dir().join("rakuos-shell-picker-live-backup");
    let _ = fs::remove_dir_all(&dest);

    let sources =
        crate::switch::backup_sources(&home, &crate::compositor::HYPRLAND, Some(&NOCTALIA));
    let backup = Backup::create(&dest, "live-check", &sources, &home).unwrap();

    println!(
        "backed up {} files across {} trees, {} recorded absent",
        backup.file_count(),
        backup.manifest.trees.len(),
        backup.manifest.absent.len()
    );
    for key in backup.manifest.trees.keys() {
        let tree = &backup.manifest.trees[key];
        println!("  {key}: {} files, {} bytes", tree.files, tree.bytes);
    }
    for key in &backup.manifest.absent {
        println!("  {key}: absent");
    }

    // A real setup is hundreds of files; a handful means the walk stopped
    // early on something it should not have.
    //
    // That setup is legitimately gone on a machine that has been switched to
    // another shell: `clean` takes the outgoing shell's config and state away,
    // which is its job. So the size is conditioned on the config having been
    // found rather than on anything about the walk. The walk itself still has
    // to hold up either way — the launcher entry is deliberately left behind
    // and is a file, which is the case every unit test above misses.
    let setup_present = !backup
        .manifest
        .absent
        .iter()
        .any(|p| p.as_str() == ".config/noctalia");
    assert!(
        backup.file_count() > if setup_present { 100 } else { 0 },
        "only {} files copied, expected the whole setup",
        backup.file_count()
    );
    backup.verify().expect("the copy must read back clean");

    let _ = fs::remove_dir_all(&dest);
}

#[test]
fn the_manifest_is_written_next_to_the_copy() {
    let root = sandbox("manifest");
    let home = root.join("home");
    write(&home.join(".config/noctalia/a"), "a");
    let backup = Backup::create(
        &root.join("backups"),
        "1-hyprland-noctalia",
        &[home.join(".config/noctalia")],
        &home,
    )
    .unwrap();

    let raw = fs::read_to_string(backup.root.join("manifest.toml")).unwrap();
    let parsed: Manifest = toml::from_str(&raw).unwrap();
    assert_eq!(parsed, backup.manifest);
}
