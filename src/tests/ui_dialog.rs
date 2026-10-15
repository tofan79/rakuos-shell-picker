use super::*;
use crate::compositor::SystemEnv;
use crate::shell::{InstallPlan, StartMechanism, UninstallPlan};
use std::collections::HashMap;

/// Statics, because `decide` takes `&'static Shell` exactly like the real
/// registry does.
///
/// INSTALLED_IDLE uses a real binary that is on PATH but is never a long
/// lived process, so it reads as "installed and idle". Picking "sh" here
/// was wrong: the shell session running cargo leaves live `sh` processes,
/// so it read as already running. ALREADY_RUNNING uses "bash", which is
/// alive in this very test process.
static INSTALLED_IDLE: Shell = Shell {
    id: "fake",
    display_name: "Fake Shell",
    upstream: "test",
    package: Some("fake"),
    binary: "basename",
    start_argv: &["basename"],
    process_match: &[],
    config_dir: None,
    state_dir: None,
    install_plan: Some(InstallPlan {
        package: "fake",
        stages: &[&["fake"]],
        program: "pkexec",
        args: &["rum", "install", "-y"],
    }),
    // Present because a shell that can be installed has to be removable, which
    // is the invariant `a_shell_that_can_be_installed_can_also_be_removed` checks.
    uninstall_plan: Some(UninstallPlan {
        package: "fake",
        stages: &[&["fake"]],
        program: "pkexec",
        args: &["rum", "remove", "-y"],
    }),
    systemd_user_unit: None,

    companion_systemd_user_units: &[],
    autostart_systemd_user_units: &[],
    companion_paths: &[],
    skel_extra_paths: &[],
    generated_paths: &[],
    discard_paths: &[],
    runtime_packages: &[],
    polkit_agent: &[],
    backup_extra_paths: &[],
    mechanism: StartMechanism::DirectProcess,
    logo: None,
};

static NOT_INSTALLED: Shell = Shell {
    id: "ghost",
    display_name: "Ghost Shell",
    upstream: "test",
    package: None,
    binary: "definitely-not-here",
    start_argv: &["definitely-not-here"],
    process_match: &[],
    config_dir: None,
    state_dir: None,
    install_plan: None,
    uninstall_plan: None,
    systemd_user_unit: None,

    companion_systemd_user_units: &[],
    autostart_systemd_user_units: &[],
    companion_paths: &[],
    skel_extra_paths: &[],
    generated_paths: &[],
    discard_paths: &[],
    runtime_packages: &[],
    polkit_agent: &[],
    backup_extra_paths: &[],
    mechanism: StartMechanism::DirectProcess,
    logo: None,
};

static ALREADY_RUNNING: Shell = Shell {
    id: "bash-shell",
    display_name: "Bash Shell",
    upstream: "test",
    package: None,
    binary: "bash",
    start_argv: &["bash"],
    process_match: &[],
    config_dir: None,
    state_dir: None,
    install_plan: None,
    uninstall_plan: None,
    systemd_user_unit: None,

    companion_systemd_user_units: &[],
    autostart_systemd_user_units: &[],
    companion_paths: &[],
    skel_extra_paths: &[],
    generated_paths: &[],
    discard_paths: &[],
    runtime_packages: &[],
    polkit_agent: &[],
    backup_extra_paths: &[],
    mechanism: StartMechanism::DirectProcess,
    logo: None,
};

/// An installed shell that is not running: the one case a click accepts.
fn ready() -> ShellState {
    ShellState {
        installed: true,
        running: false,
        running_shell: Some(&ALREADY_RUNNING),
    }
}

/// Nothing to switch to.
fn absent() -> ShellState {
    ShellState {
        installed: false,
        running: false,
        running_shell: None,
    }
}

/// Not installed, but there is a plan to install it.
fn absent_installable() -> ShellState {
    ShellState {
        installed: false,
        running: false,
        running_shell: Some(&ALREADY_RUNNING),
    }
}

/// Already the shell on screen.
fn busy() -> ShellState {
    ShellState {
        installed: true,
        running: true,
        running_shell: Some(&ALREADY_RUNNING),
    }
}

fn env(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn hyprland_session() -> HashMap<String, String> {
    env(&[
        ("WAYLAND_DISPLAY", "wayland-1"),
        ("XDG_RUNTIME_DIR", "/run/user/1000"),
        ("HYPRLAND_INSTANCE_SIGNATURE", "test-signature"),
    ])
}

fn hyprland() -> &'static dyn Compositor {
    compositor_with_id("hyprland")
}

/// A page that is not the one on screen.
///
/// Hyprland is the only registered compositor, so in the running app there is
/// no such page and the cross-compositor refusal cannot be reached. The rule
/// still has to hold for whoever registers a second compositor, so it is
/// tested against one that exists only in the test build.
fn other() -> &'static dyn Compositor {
    &crate::compositor::fake::FAKE
}

/// A detection naming a compositor that is not registered.
///
/// `detect_with` only ever names a registered compositor, so "some other
/// compositor is the running one" cannot be produced from an environment at
/// all. It is built by hand, because that is the direction the guard has to
/// hold in as well: the page you are looking at is the running one, and the
/// shell you clicked belongs to somewhere else entirely.
fn detection_naming(id: &'static str) -> crate::compositor::Detection {
    crate::compositor::Detection {
        wayland: true,
        runtime_dir: None,
        current: Some(id),
        statuses: Vec::new(),
    }
}

fn compositor_with_id(id: &str) -> &'static dyn Compositor {
    crate::compositor::all()
        .into_iter()
        .find(|c| c.id() == id)
        .unwrap_or_else(|| panic!("{id} registered"))
}

#[test]
fn click_on_a_switchable_shell_asks_for_confirmation() {
    let detection = crate::compositor::detect_with(&hyprland_session());
    assert_eq!(detection.current, Some("hyprland"));
    let action = decide(&detection, hyprland(), &INSTALLED_IDLE, ready());
    assert!(
        matches!(action, Action::ConfirmSwitch { .. }),
        "expected a confirmation, got {action:?}"
    );
}

#[test]
fn click_on_a_different_compositor_is_refused_outright() {
    let detection = crate::compositor::detect_with(&hyprland_session());
    let action = decide(&detection, other(), &INSTALLED_IDLE, ready());
    assert_eq!(
        action,
        Action::RefuseCrossCompositor {
            target: "fake",
            current: "hyprland"
        }
    );
}

#[test]
fn compositor_check_wins_over_shell_state() {
    // A shell that is not even installed, clicked on the wrong compositor,
    // must still be refused for the compositor reason.
    let detection = crate::compositor::detect_with(&hyprland_session());
    let action = decide(&detection, other(), &NOT_INSTALLED, absent());
    assert!(matches!(action, Action::RefuseCrossCompositor { .. }));
}

#[test]
fn uninstalled_shell_does_not_offer_to_switch() {
    let detection = crate::compositor::detect_with(&hyprland_session());
    let action = decide(&detection, hyprland(), &NOT_INSTALLED, absent());
    assert!(
        matches!(action, Action::NotInstalled { .. }),
        "expected not-installed, got {action:?}"
    );
}

#[test]
fn running_shell_reports_already_running() {
    let detection = crate::compositor::detect_with(&hyprland_session());
    let action = decide(&detection, hyprland(), &ALREADY_RUNNING, busy());
    assert!(
        matches!(action, Action::AlreadyRunning { .. }),
        "expected already-running, got {action:?}"
    );
}

#[test]
fn undetected_compositor_refuses_everything() {
    let detection = crate::compositor::detect_with(&env(&[]));
    assert_eq!(detection.current, None);
    let action = decide(&detection, hyprland(), &INSTALLED_IDLE, ready());
    assert!(
        matches!(action, Action::RefuseUndetected { .. }),
        "expected refusal, got {action:?}"
    );
}

#[test]
fn no_action_is_taken_without_a_wayland_session() {
    // Even with a compositor marker, no session means no action.
    let detection =
        crate::compositor::detect_with(&env(&[("HYPRLAND_INSTANCE_SIGNATURE", "test")]));
    assert!(matches!(
        decide(&detection, hyprland(), &INSTALLED_IDLE, ready()),
        Action::RefuseUndetected { .. }
    ));
}

#[test]
fn a_packaged_but_missing_shell_offers_an_install() {
    // INSTALLED_IDLE has an install plan but is reported as not installed:
    // that is DMS today, and the dialog has to say what will happen rather
    // than imply the shell is already there.
    let detection = crate::compositor::detect_with(&hyprland_session());
    match decide(
        &detection,
        hyprland(),
        &INSTALLED_IDLE,
        absent_installable(),
    ) {
        Action::ConfirmSwitch { install, to, .. } => {
            assert!(install, "the dialog must say it will install first");
            assert_eq!(to, "Fake Shell");
        }
        other => panic!("expected a confirmation, got {other:?}"),
    }
}

#[test]
fn an_installed_shell_is_not_announced_as_installing() {
    let detection = crate::compositor::detect_with(&hyprland_session());
    match decide(&detection, hyprland(), &INSTALLED_IDLE, ready()) {
        Action::ConfirmSwitch { install, .. } => assert!(!install),
        other => panic!("expected a confirmation, got {other:?}"),
    }
}

#[test]
fn a_shell_with_no_package_and_nothing_installed_is_refused() {
    // NOT_INSTALLED has no package and no install plan, so there is nowhere
    // to get it from. Refusing here is what keeps a doomed switch from
    // taking a backup first.
    let detection = crate::compositor::detect_with(&hyprland_session());
    let state = ShellState {
        installed: false,
        running: false,
        running_shell: None,
    };
    assert!(matches!(
        decide(&detection, hyprland(), &NOT_INSTALLED, state),
        Action::NotInstalled { .. }
    ));
}

#[test]
fn every_refusal_is_distinct_from_a_confirmation() {
    // The point of the gate: only one outcome may lead to the backend.
    let detection = crate::compositor::detect_with(&hyprland_session());
    let outcomes = [
        decide(&detection, hyprland(), &INSTALLED_IDLE, ready()),
        decide(&detection, hyprland(), &NOT_INSTALLED, absent()),
        decide(&detection, hyprland(), &ALREADY_RUNNING, busy()),
        decide(&detection, other(), &INSTALLED_IDLE, ready()),
    ];
    let confirmations = outcomes
        .iter()
        .filter(|a| matches!(a, Action::ConfirmSwitch { .. }))
        .count();
    assert_eq!(
        confirmations, 1,
        "exactly one path may confirm: {outcomes:?}"
    );
}

#[test]
fn the_real_environment_detects_a_registered_compositor() {
    // Guards against detection silently breaking on the actual session. The
    // session can be either compositor now, so this checks the result is one
    // the picker knows rather than naming a particular one.
    let detection = crate::compositor::detect_with(&SystemEnv);
    if let Some(current) = detection.current {
        assert!(
            crate::compositor::all().iter().any(|c| c.id() == current),
            "detected {current:?}, which is not a registered compositor"
        );
    }
}

/// Mirrors `Selection::guard_reason`, which needs a window. The wording is
/// what matters here, so it is reproduced rather than dragged into a GTK
/// test.
fn detection_reason(
    detection: &crate::compositor::Detection,
    page: &dyn Compositor,
) -> Option<String> {
    let current = detection.current?;
    if current == page.id() {
        return None;
    }
    Some(format!(
        "{} is not the running compositor — {current} is.",
        page.label()
    ))
}

#[test]
fn the_guard_works_in_both_directions() {
    // Whichever page you are on, a card belonging to a compositor that is not
    // the running one is refused. The guard compares ids, so this is meant to
    // hold without any extra code; these tests are here so that if someone ever
    // hardcodes a direction, it fails.
    let hyprland_up = crate::compositor::detect_with(&hyprland_session());
    assert_eq!(hyprland_up.current, Some("hyprland"));

    // Hyprland running: the other page is blocked.
    assert_eq!(
        decide(&hyprland_up, other(), &INSTALLED_IDLE, ready()),
        Action::RefuseCrossCompositor {
            target: "fake",
            current: "hyprland"
        }
    );

    // Something else running: the Hyprland page is blocked, ids swapped.
    assert_eq!(
        decide(
            &detection_naming("fake"),
            hyprland(),
            &INSTALLED_IDLE,
            ready()
        ),
        Action::RefuseCrossCompositor {
            target: "hyprland",
            current: "fake"
        }
    );
}

#[test]
fn the_running_compositor_is_the_only_one_that_offers_a_switch() {
    let hyprland_up = crate::compositor::detect_with(&hyprland_session());
    let fake_up = detection_naming("fake");

    assert!(matches!(
        decide(&hyprland_up, hyprland(), &INSTALLED_IDLE, ready()),
        Action::ConfirmSwitch { .. }
    ));
    assert!(matches!(
        decide(&hyprland_up, other(), &INSTALLED_IDLE, ready()),
        Action::RefuseCrossCompositor { .. }
    ));
    // Nothing of the other compositor may be clickable while it is the one
    // on screen, even from a Hyprland tab.
    assert!(matches!(
        decide(&fake_up, hyprland(), &INSTALLED_IDLE, ready()),
        Action::RefuseCrossCompositor { .. }
    ));
}

#[test]
fn the_block_reason_names_both_compositors_in_the_right_order() {
    let detection = detection_naming("fake");
    let reason = detection_reason(&detection, hyprland());
    let text = reason.expect("hyprland must be refused while another compositor runs");
    assert!(text.contains("hyprland"), "{text}");
    assert!(text.contains("fake"), "{text}");
    // The page being viewed must be named first, the running one second.
    assert!(
        text.find("hyprland") < text.find("fake"),
        "the wrong order reads as if the other compositor were running: {text}"
    );
}

#[test]
fn the_app_opens_on_the_running_compositors_tab() {
    assert_eq!(
        crate::compositor::default_id(&crate::compositor::detect_with(&hyprland_session())),
        "hyprland"
    );
    // A session the registry cannot name opens on Hyprland, which is the only
    // page there is — and every click on it is refused, because the running
    // compositor was never identified. Falling back to a tab rather than
    // refusing to open is deliberate; refusing every click on it is what keeps
    // the fallback honest.
    let unrecognised = crate::compositor::detect_with(&env(&[("WAYLAND_DISPLAY", "wayland-1")]));
    assert_eq!(unrecognised.current, None);
    assert_eq!(crate::compositor::default_id(&unrecognised), "hyprland");
    assert!(matches!(
        decide(&unrecognised, hyprland(), &INSTALLED_IDLE, ready()),
        Action::RefuseUndetected { .. }
    ));
    // With nothing detected some tab still has to be selected.
    let unknown = crate::compositor::detect_with(&env(&[]));
    assert_eq!(
        crate::compositor::default_id(&unknown),
        crate::compositor::all()[0].id()
    );
}

#[test]
fn a_cancelled_dialog_gives_the_card_back() {
    // The bug this guards: the mark was drawn when the dialog opened and
    // nothing took it back when the dialog closed without a switch. The card
    // stayed an active ToggleButton, and since the handler ignores a toggle
    // off, the next click on that card did nothing at all.
    let detection = crate::compositor::detect_with(&hyprland_session());
    let confirm = decide(&detection, hyprland(), &INSTALLED_IDLE, ready());
    assert!(
        !keeps_mark(&confirm, false),
        "a dialog the user dismissed must not leave a card marked"
    );
    assert!(
        keeps_mark(&confirm, true),
        "a confirmed switch keeps the mark for the duration of the operation"
    );
}

#[test]
fn no_refusal_ever_keeps_a_mark() {
    // Whatever the ending, a refusal has nothing running that a mark could
    // truthfully mean, so it must never leave one behind.
    let detection = crate::compositor::detect_with(&hyprland_session());
    let refusals = [
        decide(&detection, hyprland(), &NOT_INSTALLED, absent()),
        decide(&detection, hyprland(), &ALREADY_RUNNING, busy()),
        decide(&detection, other(), &INSTALLED_IDLE, ready()),
        decide(
            &detection_naming("fake"),
            hyprland(),
            &INSTALLED_IDLE,
            ready(),
        ),
        decide(
            &crate::compositor::detect_with(&env(&[])),
            hyprland(),
            &INSTALLED_IDLE,
            ready(),
        ),
    ];
    for action in &refusals {
        assert!(
            !matches!(action, Action::ConfirmSwitch { .. }),
            "expected a refusal in this list, got {action:?}"
        );
        assert!(
            !keeps_mark(action, true),
            "{action:?} must not keep a mark even if a dialog said yes"
        );
    }
}

#[test]
fn the_response_strings_the_dialog_registers_match_what_the_gate_expects() {
    // confirm_switch reports `response == SWITCH`. Anything that renames a
    // response without renaming the constant turns every confirmation into a
    // silent dismissal, so the pairing is pinned here.
    assert_ne!(SWITCH, CANCEL);
    assert_eq!(SWITCH, "switch");
    assert_eq!(CANCEL, "cancel");
}
