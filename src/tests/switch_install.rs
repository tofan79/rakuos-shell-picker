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
