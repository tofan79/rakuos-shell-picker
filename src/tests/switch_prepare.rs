use super::*;
use crate::compositor::hyprland::shells::{dms::SHELL as DMS, noctalia::SHELL as NOCTALIA};
use crate::shell::StartMechanism;

/// A shell that is on no machine: no package, no install plan, a binary nobody
/// has, and a config directory nobody created.
///
/// Used instead of a real registry shell so the refusal tests say the same thing
/// wherever they run. Ambxst used to stand here, and once it gained a skel the
/// test passed only on machines where Ambxst happened not to be installed — the
/// opposite of what a test is for.
static GHOST: Shell = Shell {
    id: "ghost",
    display_name: "Ghost Shell",
    upstream: "test",
    package: None,
    binary: "rakuos-shell-picker-no-such-binary",
    start_argv: &["rakuos-shell-picker-no-such-binary"],
    process_match: &[],
    config_dir: Some(".config/rakuos-shell-picker-no-such-shell"),
    state_dir: None,
    install_plan: None,
    uninstall_plan: None,
    companion_systemd_user_units: &[],
    autostart_systemd_user_units: &[],
    companion_paths: &[],
    skel_extra_paths: &[],
    generated_paths: &[],
    discard_paths: &[],
    runtime_packages: &[],
    polkit_agent: &[],
    backup_extra_paths: &[],
    systemd_user_unit: None,
    systemd_user_unit_wanted_by: None,
    mechanism: StartMechanism::DirectProcess,
    logo: None,
};

/// As [`GHOST`], but with nothing recorded for where its config lives: the shape
/// `docs/SAFETY.md` rule 4 refuses before the backup is taken.
static NO_CONFIG: Shell = Shell {
    config_dir: None,
    ..GHOST
};

/// Which shell is alive right now, if any.
fn running_shell() -> Option<&'static Shell> {
    crate::ui::dialog::running_shell()
}

/// The backup list for one compositor, as paths relative to a fake home.
fn list_for(compositor: &'static dyn Compositor, shell: Option<&'static Shell>) -> Vec<String> {
    backup_sources(&PathBuf::from("/home/tester"), compositor, shell)
        .iter()
        .map(|p| {
            p.strip_prefix("/home/tester")
                .expect("every backup path is inside home")
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

fn scratch(name: &str) -> (PathBuf, PathBuf) {
    let root = std::env::temp_dir()
        .join("rakuos-shell-picker-tests")
        .join("switch")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("home")).unwrap();
    let home = root.join("home");
    (root, home)
}

#[test]
fn each_compositor_backs_up_its_own_config_and_not_the_other_one() {
    // The compositor's config directory is the one path in the list that
    // follows the compositor. Backing up Hyprland's while running something
    // else would be a backup of a file nothing touched, and missing the
    // running one's would leave the config the switch really edits
    // unprotected.
    let hypr = list_for(&crate::compositor::HYPRLAND, Some(&DMS));
    assert!(
        hypr.contains(&".config/hypr".to_string()),
        "Hyprland list must include its own config: {hypr:?}"
    );
    assert!(
        !hypr.contains(&".config/fake".to_string()),
        "Hyprland list must not mention another compositor: {hypr:?}"
    );

    // The second compositor is the test-only one, so this test does not depend
    // on which real compositors are registered. The rule is the same either
    // way: the list is the compositor's own, not a fixed set.
    let other = crate::switch::backup_sources(
        &PathBuf::from("/home/tester"),
        &crate::compositor::fake::FAKE,
        Some(&DMS),
    )
    .iter()
    .map(|p| {
        p.strip_prefix("/home/tester")
            .unwrap()
            .display()
            .to_string()
    })
    .collect::<Vec<_>>();
    assert!(
        other.contains(&".config/fake".to_string()),
        "the second compositor's list must include its own config: {other:?}"
    );
    assert!(
        !other.contains(&".config/hypr".to_string()),
        "the second compositor's list must not mention Hyprland: {other:?}"
    );
}

#[test]
fn the_compositor_config_is_saved_even_with_no_shell_running() {
    // A switch writes fragments into that file, so it has to be recoverable
    // even when there is no outgoing shell config to save alongside it.
    let only = list_for(&crate::compositor::HYPRLAND, None);
    assert!(only.contains(&".config/hypr".to_string()), "got {only:?}");
}

#[test]
fn the_wrong_compositor_is_refused_before_anything_is_copied() {
    let (root, home) = scratch("wrong-compositor");
    let err = prepare(
        &crate::compositor::fake::FAKE,
        Some("hyprland"),
        &DMS,
        Some(&DMS),
        false,
        &home,
        &root.join("backups"),
    )
    .expect_err("a cross-compositor switch must be refused");

    assert_eq!(
        err,
        Refusal::Compositor {
            target: "fake".to_string(),
            current: Some("hyprland".to_string())
        }
    );
    assert!(
        !root.join("backups").exists(),
        "no backup directory may be created for a refused switch"
    );
}

#[test]
fn an_undetected_compositor_is_refused() {
    let (root, home) = scratch("undetected");
    let err = prepare(
        &crate::compositor::HYPRLAND,
        None,
        &DMS,
        None,
        false,
        &home,
        &root.join("backups"),
    )
    .expect_err("an unidentified compositor must be refused");
    assert_eq!(
        err.message(),
        "the running compositor could not be identified"
    );
    assert!(!root.join("backups").exists());
}

#[test]
fn a_shell_with_nothing_to_install_from_is_refused() {
    let (root, home) = scratch("unavailable");
    let err = prepare(
        &crate::compositor::HYPRLAND,
        Some("hyprland"),
        &GHOST,
        None,
        false,
        &home,
        &root.join("backups"),
    )
    .expect_err("a shell with no package, no plan and no binary has nothing to switch to");
    assert!(matches!(err, Refusal::Unavailable { .. }), "{err:?}");
    assert!(!root.join("backups").exists());
}

#[test]
fn a_shell_that_does_not_say_where_its_config_lives_is_refused() {
    // The backend half of SAFETY.md rule 4. Without a config location there is
    // nothing to back up and nothing to seed, so the switch must stop here —
    // before the backup directory exists, not after.
    let (root, home) = scratch("no-config");
    let err = prepare(
        &crate::compositor::HYPRLAND,
        Some("hyprland"),
        &NO_CONFIG,
        None,
        false,
        &home,
        &root.join("backups"),
    )
    .expect_err("a shell with no config_dir must be refused");
    assert!(matches!(err, Refusal::NoConfig { .. }), "{err:?}");
    assert!(!root.join("backups").exists());
}

#[test]
fn a_packaged_shell_is_backed_up_even_though_it_is_not_installed_yet() {
    // DMS is the case that matters: not installed, so nothing exists at
    // ~/.config/dms, and the backup still has to be taken.
    let (root, home) = scratch("dms");
    std::fs::create_dir_all(home.join(".config/noctalia")).unwrap();
    std::fs::write(home.join(".config/noctalia/a"), "a").unwrap();

    let prepared = prepare(
        &crate::compositor::HYPRLAND,
        Some("hyprland"),
        &DMS,
        Some(&NOCTALIA),
        false,
        &home,
        &root.join("backups"),
    )
    .expect("a packaged target is worth switching to");

    assert_eq!(prepared.to.id, "dms");
    // The outgoing shell's config is in there, the target's is not.
    assert!(prepared
        .backup
        .manifest
        .trees
        .contains_key(".config/noctalia"));
    assert!(!prepared
        .backup
        .manifest
        .trees
        .contains_key(".config/dms"));
    // Everything shared is accounted for: copied if it exists, recorded
    // absent if it does not. Neither may be silently missing, because that
    // is the difference between "was never there" and "was forgotten".
    for shared in SHARED_PATHS {
        let key = (*shared).to_string();
        assert!(
            prepared.backup.manifest.trees.contains_key(&key)
                || prepared.backup.manifest.absent.contains(&key),
            "{shared} was neither copied nor recorded absent"
        );
    }
    assert!(prepared
        .backup
        .manifest
        .absent
        .contains(&".config/hypr".to_string()));
    assert!(prepared.backup.verify().is_ok());
}

#[test]
fn the_backup_directory_is_named_for_the_switch() {
    let (root, home) = scratch("naming");
    let prepared = prepare(
        &crate::compositor::HYPRLAND,
        Some("hyprland"),
        &DMS,
        Some(&NOCTALIA),
        false,
        &home,
        &root.join("backups"),
    )
    .unwrap();

    let name = prepared
        .backup
        .root
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    assert!(name.starts_with("202"), "{name} should start with a date");
    assert!(name.ends_with("-hyprland-noctalia"), "{name}");
    assert!(prepared.backup.root.join("manifest.toml").is_file());
}

#[test]
fn two_switches_in_a_row_do_not_collide() {
    // Two clicks in quick succession used to land on the same directory name,
    // and the second was refused because the name was taken. That is the
    // worst possible outcome: the backup the user asked for is the one that
    // fails, and the reason on screen ("cannot create backup directory")
    // points at the disk rather than at the clock.
    //
    // Backups are dated by day now, so on the same day the two switches
    // *do* share a name — one backup per day is the whole point. What has
    // to hold instead is that neither switch is refused, that each one
    // verifies what it wrote, and that the day ends with one complete
    // directory rather than two or a half-written one.
    let (root, home) = scratch("two-switches");
    let backups = root.join("backups");

    let mut roots = Vec::new();
    for round in 0..2 {
        let prepared = prepare(
            &crate::compositor::HYPRLAND,
            Some("hyprland"),
            &DMS,
            Some(&NOCTALIA),
            false,
            &home,
            &backups,
        )
        .unwrap_or_else(|e| panic!("switch {round} must not be refused: {}", e.message()));
        prepared
            .backup
            .verify()
            .unwrap_or_else(|e| panic!("switch {round} must verify what it wrote: {e:#}"));
        roots.push(prepared.backup.root.clone());
    }

    assert_eq!(
        roots[0], roots[1],
        "one day means one directory, and neither switch may be turned away"
    );
    let names: Vec<String> = std::fs::read_dir(&backups)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        names,
        vec![roots[0].file_name().unwrap().to_string_lossy().into_owned()],
        "the day must end with one directory, not one per click: {names:?}"
    );
}

/// Runs the real prepare() against the real config and the real backup
/// directory. Ignored so `cargo test` never writes there by accident; run
/// with `cargo test -- --include-ignored`.
///
/// It only copies. Nothing is installed, stopped or started, which is the
/// whole point of where the order currently stops.
#[test]
#[ignore = "writes to the real backup directory"]
fn a_real_switch_to_dms_stops_after_the_backup() {
    let home = PathBuf::from(std::env::var_os("HOME").expect("HOME"));
    let backups = crate::session::State::backups_dir();

    let prepared = prepare(
        &crate::compositor::HYPRLAND,
        Some("hyprland"),
        &DMS,
        running_shell(),
        false,
        &home,
        &backups,
    )
    .expect("prepare should reach the backup");

    println!("from: {:?}", prepared.from.map(|s| s.id));
    println!("to:   {}", prepared.to.id);
    println!("backup directory: {}", prepared.backup.root.display());
    println!("files: {}", prepared.backup.file_count());
    for (key, tree) in &prepared.backup.manifest.trees {
        println!(
            "  copied   {key}: {} files, {} bytes",
            tree.files, tree.bytes
        );
    }
    for key in &prepared.backup.manifest.absent {
        println!("  absent   {key}");
    }
    assert!(prepared.backup.verify().is_ok());
}

#[test]
fn installer_output_arrives_before_the_command_finishes() {
    // The whole point: a user must see the installer working while it works.
    // With `Command::output` this assertion fails, because nothing is reported
    // until the process has already exited.
    let mut command = Command::new("sh");
    command.args(["-c", "echo first; sleep 0.4; echo second"]);
    let mut seen: Vec<String> = Vec::new();
    let (status, text) = run_streaming(
        &mut command,
        &mut |line| seen.push(line.to_string()),
        "installing",
    )
    .unwrap();

    assert!(status.success(), "sh exited {status}");
    assert_eq!(seen, vec!["first".to_string(), "second".to_string()]);
    assert!(text.contains("first") && text.contains("second"));
}

#[test]
fn a_failing_installer_is_reported_with_its_own_words() {
    let mut command = Command::new("sh");
    command.args(["-c", "echo 'No match for argument: nope' >&2; exit 1"]);
    let mut seen: Vec<String> = Vec::new();
    let (status, text) = run_streaming(
        &mut command,
        &mut |line| seen.push(line.to_string()),
        "installing",
    )
    .unwrap();

    assert!(!status.success());
    assert!(
        seen.iter().any(|l| l.contains("No match")),
        "stderr must reach the panel, got {seen:?}"
    );
    assert!(text.contains("No match"));
}

#[test]
fn an_installer_that_says_nothing_still_finishes() {
    let mut command = Command::new("sh");
    command.args(["-c", "exit 0"]);
    let mut seen: Vec<String> = Vec::new();
    let (status, _) = run_streaming(
        &mut command,
        &mut |line| seen.push(line.to_string()),
        "installing",
    )
    .unwrap();
    assert!(status.success());
    assert!(seen.is_empty(), "nothing to report is not an error");
}
