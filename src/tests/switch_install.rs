use super::*;

#[test]
fn a_cancelled_polkit_prompt_is_not_reported_as_a_broken_package_manager() {
    // pkexec's documented exit for a denied or cancelled request.
    assert!(looks_dismissed(Some(126), ""));
    assert!(looks_dismissed(Some(127), ""));
    assert!(looks_dismissed(Some(1), "Error: not authorized"));
    assert!(looks_dismissed(
        Some(1),
        "Authentication is required to run 'rum'"
    ));
}

#[test]
fn a_failed_transaction_is_not_mistaken_for_a_refusal() {
    assert!(!looks_dismissed(Some(1), "No match for argument: dms"));
    assert!(!looks_dismissed(Some(1), "Error: GPG check FAILED"));
    assert!(!looks_dismissed(None, "killed by signal"));
}

#[test]
fn the_logged_command_names_every_package_the_plan_stages() {
    let plan = crate::compositor::hyprland::shells::dms::SHELL
        .install_plan
        .unwrap();
    assert_eq!(
        plan.packages(),
        vec![
            "quickshell-git",
            "dms",
            "dms-cli",
            "dgop",
            "danksearch",
            "dankcalendar-git",
            "matugen",
        ]
    );
    // Stage 1 is the runtime alone, so a failure there leaves nothing of dms
    // on the machine.
    assert_eq!(
        plan.stage_command_line(&["quickshell-git"]),
        "pkexec rum install -y --refresh quickshell-git"
    );
    assert!(!plan.stage_command_line(plan.stages[0]).contains("dms"));
}

#[test]
fn the_stage_that_asks_for_a_password_says_where_to_type_it() {
    // A password prompt is never inside the picker's own window: `pkexec` asks
    // through its textual agent in the terminal the picker was started from, or
    // through the desktop's polkit agent in a dialog over the session. Naming
    // which one is the difference between a user finding the prompt and a user
    // watching the picker wait for an answer they cannot see.
    let line = authorizing(2);
    assert!(line.contains("authorizing stage 2"), "{line}");
    assert!(
        line.contains("terminal") || line.contains("dialog"),
        "the line has to name where the prompt appears: {line}"
    );
}
