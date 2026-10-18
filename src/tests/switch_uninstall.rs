use super::*;
use crate::shell::UninstallPlan;

#[test]
fn a_removal_with_nothing_installed_never_reaches_the_package_manager() {
    // The mirror of the install's "already there" answer, and for the same
    // reason: a stage with nothing to do would still ask polkit for a password
    // and report a stage that removed nothing as progress.
    let plan = UninstallPlan {
        package: "rsp-no-such-package-anywhere",
        stages: &[&["rsp-no-such-package-anywhere"]],
        program: "pkexec",
        args: &["rum", "remove", "-y"],
    };

    let mut said = Vec::new();
    let outcome = uninstall(plan, &[], &mut |line| said.push(line.to_string()));

    assert!(
        matches!(outcome, Removed::AlreadyGone { .. }),
        "an absent package is not a failure: {outcome:?}"
    );
    assert!(
        said.is_empty(),
        "nothing to remove is nothing to report, so no password prompt: {said:?}"
    );
}

#[test]
fn a_removal_plan_that_names_nothing_is_refused_rather_than_run() {
    // An empty plan has no packages, and "are all of these removed?" over an
    // empty list is answered yes — so without this the switch would announce
    // the shell was gone. It exists because the check before it would
    // otherwise be vacuous, exactly as for an install.
    let plan = UninstallPlan {
        package: "dms",
        stages: &[],
        program: "pkexec",
        args: &["rum", "remove", "-y"],
    };

    let outcome = uninstall(plan, &[], &mut |_| {});
    assert!(matches!(outcome, Removed::Malformed { package: "dms" }));
}

#[test]
fn the_removal_plan_takes_the_runtime_it_was_the_one_to_install() {
    // The half of the old rule that survives. `quickshell-git` goes in DMS's
    // removal plan because nothing else on the machine asks for it once `dms`
    // and `dankcalendar-git` are in the same transaction — an orphan a switch
    // left for `rum autoremove` to find was not a safety property, it was a
    // mess. The other half of the old rule, "never take a shared runtime out",
    // now lives in `uninstall`, which drops anything the incoming shell declares.
    let plan = crate::shell::find("dms")
        .expect("registered")
        .uninstall_plan
        .expect("dms can be installed, so it can be removed");

    assert!(
        plan.packages().contains(&"quickshell-git"),
        "DMS installed the runtime, so leaving DMS should take it too"
    );
    assert!(
        plan.packages().contains(&"dms"),
        "and it must take out the shell itself"
    );
}

#[test]
fn leaving_caelestia_takes_the_runtime_unless_the_next_shell_runs_on_it() {
    // The same contract DMS holds, spelled out for the shell that joined it.
    // Caelestia's install put `quickshell-git` on the machine, so its removal
    // plan has to be able to take it off again — otherwise leaving Caelestia
    // for Noctalia, which runs on nothing, would strand an orphan for `rum
    // autoremove` to find. The shared-runtime guard is the switch's, applied
    // against the target: leaving for DMS or Ambxst keeps the runtime, exactly
    // as it does for a reversed switch.
    let plan = crate::shell::find("caelestia")
        .expect("registered")
        .uninstall_plan
        .expect("caelestia can be installed, so it can be removed");

    assert_eq!(
        plan.packages(),
        vec![
            "caelestia-shell",
            "quickshell-git",
            "mate-polkit",
            "google-rubik-fonts",
            "cascadia-code-nf-fonts",
            "material-symbols-fonts",
            "swappy",
        ],
        "every package the install put there has to be removable"
    );

    let dms = crate::shell::find("dms").expect("registered");
    let kept: Vec<_> = plan
        .packages()
        .into_iter()
        .filter(|p| dms.runtime_packages.contains(p))
        .collect();
    assert_eq!(
        kept,
        vec!["quickshell-git"],
        "switching Caelestia → DMS keeps the runtime DMS runs on"
    );
}

#[test]
fn a_package_the_shell_being_switched_to_runs_on_is_never_sent_to_the_package_manager() {
    // The guard that replaced the blanket "never remove a shared runtime".
    // `keep` is `target.runtime_packages` read by `apply`, and it is applied
    // before the package manager is asked for anything — so the runtime is not
    // removed and then put back, it is simply never offered for removal.
    let plan = UninstallPlan {
        package: "rsp-gone-anyway",
        stages: &[&["rsp-absent-one", "rsp-absent-two"]],
        program: "pkexec",
        args: &["rum", "remove", "-y"],
    };

    let mut said = Vec::new();
    let outcome = uninstall(plan, &["rsp-absent-two"], &mut |line| {
        said.push(line.to_string())
    });

    match outcome {
        Removed::AlreadyGone { packages } => assert_eq!(
            packages,
            vec!["rsp-absent-one"],
            "the package the next shell runs on must not be counted as removable"
        ),
        other => panic!("the rest of the plan is still absent, so: {other:?}"),
    }
    assert!(
        said.iter().any(|l| l.contains("keeping rsp-absent-two")),
        "what was kept has to be said out loud: {said:?}"
    );
    assert!(
        !said.iter().any(|l| l.contains("authorizing")),
        "nothing was sent anywhere, so no password prompt: {said:?}"
    );
}

#[test]
fn a_plan_whose_every_package_is_needed_by_the_next_shell_runs_nothing() {
    // The corner the guard makes reachable: a plan that could otherwise remove
    // something that turns out to be shared by everyone. Reporting it as
    // "already gone" would send a reader looking for packages that are still
    // there on purpose, which is why `Removed::Kept` exists.
    let plan = UninstallPlan {
        package: "rsp-shared",
        stages: &[&["rsp-shared-runtime"]],
        program: "pkexec",
        args: &["rum", "remove", "-y"],
    };

    let mut said = Vec::new();
    let outcome = uninstall(plan, &["rsp-shared-runtime"], &mut |line| {
        said.push(line.to_string())
    });

    assert!(matches!(
        outcome,
        Removed::Kept { ref packages } if packages == &vec!["rsp-shared-runtime"]
    ));
    assert!(
        !said.iter().any(|l| l.contains("authorizing")),
        "keeping something is not a transaction: {said:?}"
    );
}

#[test]
fn a_base_image_package_is_never_sent_to_the_package_manager() {
    // A base image is read-only, so `rum remove` refuses its packages: a stage
    // that named one could only fail. Filtering by origin is what lets a shell
    // that lives in the image carry a removal plan at all — Noctalia on
    // Hyprland, DMS on Niri — so this is the guard those plans rely on.
    //
    // `bash` stands in for any base package: every RakuOS and Fedora build root
    // has it in the default rpm database, so its origin is `Base` wherever the
    // suite runs, and nothing in a one-package plan can be removed.
    let plan = UninstallPlan {
        package: "bash",
        stages: &[&["bash"]],
        program: "pkexec",
        args: &["rum", "remove", "-y"],
    };

    let mut said = Vec::new();
    let outcome = uninstall(plan, &[], &mut |line| said.push(line.to_string()));

    assert!(
        matches!(outcome, Removed::Kept { ref packages } if packages == &vec!["bash"]),
        "a base-image package is kept rather than sent to rum: {outcome:?}"
    );
    assert!(
        said.iter().any(|l| l.contains("base image")),
        "why it was kept has to be said out loud: {said:?}"
    );
    assert!(
        !said.iter().any(|l| l.contains("authorizing")),
        "nothing was sent anywhere, so no password prompt: {said:?}"
    );
}
