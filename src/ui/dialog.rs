//! The safety gate: nothing reaches the backend without passing through here.
//!
//! `decide()` is deliberately pure — it maps a situation to an `Action` with no
//! GTK involved, so every branch is unit testable. The `AdwAlertDialog`
//! builders below only render what `decide()` already decided.
//!
//! Rules implemented here come from docs/SAFETY.md.

use libadwaita::prelude::*;
use libadwaita::{AlertDialog, ApplicationWindow, ResponseAppearance};

use crate::compositor::{Compositor, Detection};
use crate::shell::Shell;

/// What a click on a shell card means.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// The card belongs to a compositor that is not the running one.
    RefuseCrossCompositor {
        target: &'static str,
        current: &'static str,
    },
    /// This shell is already the one running.
    AlreadyRunning { shell: String },
    /// Not installed, and installing is out of scope.
    NotInstalled { shell: String },
    /// Real switch candidate: needs an explicit yes.
    ConfirmSwitch {
        from: Option<String>,
        to: &'static str,
        /// The target is not installed yet and will be installed by the switch.
        install: bool,
    },
    /// Compositor could not be identified at all.
    RefuseUndetected { reason: String },
}

/// What the caller knows about a shell at the moment of the click.
///
/// Passing this in is what keeps `decide` pure. Asking the shell directly would
/// mean a package query and a scan of /proc inside a routing decision, which is
/// both slow to do twice and impossible to test without depending on whatever
/// happens to be running on the machine.
#[derive(Clone, Copy)]
pub struct ShellState {
    /// The package is installed, or an unpackaged shell is fully set up.
    pub installed: bool,
    /// A process for this shell is alive right now.
    pub running: bool,
    /// Any registered shell that is alive, named in the confirmation.
    pub running_shell: Option<&'static Shell>,
}

/// Decide what a click means. No side effects.
pub fn decide(
    detection: &Detection,
    compositor: &'static dyn Compositor,
    shell: &'static Shell,
    state: ShellState,
) -> Action {
    // Compositor checks come first: a card on the wrong compositor must never
    // be reachable, whatever state the shell itself is in.
    match detection.current {
        None => {
            return Action::RefuseUndetected {
                reason: detection.refusal().unwrap_or_else(|| {
                    "the running compositor could not be identified.".to_string()
                }),
            }
        }
        Some(current) if current != compositor.id() => {
            return Action::RefuseCrossCompositor {
                target: compositor.id(),
                current,
            }
        }
        _ => {}
    }

    if state.running {
        return Action::AlreadyRunning {
            shell: shell.display_name.to_string(),
        };
    }

    // Not being installed only refuses a shell there is no way to obtain. The
    // test is the install plan rather than the package name, because the plan is
    // the per-shell declaration that an install is actually possible: DMS,
    // Caelestia and Ambxst all have one, while Noctalia is in the base image and
    // needs none. A shell with a plan is worth switching to, because the backend
    // installs it only after the backup.
    let install = !state.installed && shell.install_plan.is_some();
    if !state.installed && !install {
        return Action::NotInstalled {
            shell: shell.display_name.to_string(),
        };
    }

    let from = state.running_shell.map(|s| s.display_name.to_string());

    Action::ConfirmSwitch {
        from,
        to: shell.display_name,
        install,
    }
}

/// The shell that is alive right now, if any of the registered ones is.
pub fn running_shell() -> Option<&'static Shell> {
    // Look across every compositor so a running shell is still recognised when
    // the tab being viewed is a different one.
    crate::compositor::all()
        .iter()
        .flat_map(|c| c.shells())
        .find(|s| s.running())
}

/// Whether the clicked card may stay marked once the dialog is gone.
///
/// Only a confirmed switch keeps the mark, because the mark means "a switch to
/// this is under way". Every other ending takes it back, including a cancelled
/// dialog.
///
/// This is not cosmetic. The mark is a `ToggleButton` left active, and the card
/// handler deliberately ignores a toggle *off* — that is what stops one click
/// from running two switches. So a card left active swallows the next click:
/// clicking it turns it off and nothing happens, and only the click after that
/// reaches the dialog again. A user who cancelled once has to click twice for
/// every later attempt, with nothing on screen to explain why.
pub fn keeps_mark(action: &Action, confirmed: bool) -> bool {
    matches!(action, Action::ConfirmSwitch { .. }) && confirmed
}

// ---------------------------------------------------------------------------
// GTK rendering
// ---------------------------------------------------------------------------

fn markup_label(text: &str) -> gtk4::Label {
    let label = gtk4::Label::new(Some(text));
    label.set_use_markup(true);
    label.set_justify(gtk4::Justification::Center);
    label.set_wrap(true);
    label.set_max_width_chars(38);
    label
}

/// A short alert with an optional emphasised line underneath.
///
/// The body is plain text, so bold cannot be smuggled in through markup tags.
/// The emphasised part therefore goes in `extra` as its own markup label.
///
/// Responses are added before presenting: a dialog that appears before it has
/// any response is a dialog with no visible button.
fn alert(
    parent: &ApplicationWindow,
    heading: &str,
    body: &str,
    extra: Option<&str>,
) -> AlertDialog {
    let dialog = AlertDialog::new(Some(heading), Some(body));

    if let Some(text) = extra {
        dialog.set_extra_child(Some(&markup_label(text)));
    }

    dialog.set_default_response(Some("close"));
    dialog.set_close_response("close");
    dialog.add_response("close", "Close");
    dialog.present(Some(parent));
    dialog
}

/// Compositor mismatch. Not a yes/no: there is nothing to agree to.
///
/// `current` is named because being told what is blocked without being told what
/// is running leaves the user with nothing to act on: the card they wanted is
/// unavailable and the answer does not say what to use instead. It was passed in
/// and dropped, while the equivalent message on the blocked page names both.
pub fn refuse_cross_compositor(parent: &ApplicationWindow, target: &str, current: &str) {
    alert(
        parent,
        "Blocked",
        &format!("{target} is not the running compositor."),
        // Markup, so the name has to come from somewhere we control. It does:
        // a detection id is always one of ours, never anything read from the
        // environment. The body above stays plain text.
        Some(&format!(
            "<b>{current}</b> is running. Switch image in \"RakuOS system management\"."
        )),
    );
}

/// Detection failed: we refuse rather than guess.
pub fn refuse_undetected(parent: &ApplicationWindow, reason: &str) {
    alert(
        parent,
        "Blocked",
        // Reasons already end in a full stop; adding another printed "..".
        &format!("{}.", reason.trim_end_matches('.')),
        Some("Nothing was changed."),
    );
}

/// Already running: nothing to switch to.
pub fn already_running(parent: &ApplicationWindow, shell: &str) {
    alert(
        parent,
        "Already running",
        &format!("{shell} is already running."),
        Some("Nothing to switch to."),
    );
}

/// Not installed. Installing belongs to the image, not to this picker.
pub fn not_installed(parent: &ApplicationWindow, shell: &str) {
    alert(
        parent,
        "Not installed",
        &format!("{shell} is not installed."),
        Some("Packages come from the <b>RakuOS</b> image."),
    );
}

/// The response that starts a switch. Every other response is a dismissal.
pub const SWITCH: &str = "switch";

/// The response for Cancel, for Escape and for the window's close button.
///
/// Named in all three places on purpose. `set_close_response` is what makes
/// Escape and the title-bar close arrive here as a dismissal rather than as
/// silence, so a dismissed dialog takes the card's mark back exactly like an
/// explicit Cancel does.
pub const CANCEL: &str = "cancel";

/// The real switch: an explicit yes, and only a yes.
///
/// Cancel is the default response, so pressing Enter does the safe thing.
///
/// `on_response` is told whether the user said yes. It fires for every ending,
/// including the ones where nothing happens, because the caller has already
/// marked the clicked card by then and has to be able to take that mark back.
/// A callback that could only report success would leave that to nobody.
pub fn confirm_switch(
    parent: &ApplicationWindow,
    from: Option<&str>,
    shell_id: &str,
    to: &str,
    install: bool,
    on_response: impl Fn(bool) + 'static,
) {
    let body = match from {
        Some(name) => format!("{name} is running now."),
        None => "No registered shell is running.".to_string(),
    };

    // A shell that has to be installed says so in the title and on the button,
    // so the person who sees a preview and presses the button knows what it does.
    let heading = if install {
        format!("Install {to}?")
    } else {
        format!("Switch to {to}?")
    };
    let dialog = AlertDialog::new(Some(&heading), Some(&body));
    // Say so before the click, not after: installing is the only step here that
    // changes the system, and it happens after the backup, not before.
    let extra = if install {
        format!("{to} is not installed yet. Your configuration is backed up first, then it is installed.")
    } else {
        "Configuration is backed up and never deleted.".to_string()
    };

    // The preview comes first, so it is the first thing seen before deciding.
    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    content.append(&super::preview::build(shell_id));
    content.append(&markup_label(&extra));
    dialog.set_extra_child(Some(&content));

    dialog.add_response(CANCEL, "Cancel");
    let action = if install {
        format!("Install {to}")
    } else {
        format!("Switch to {to}")
    };
    dialog.add_response(SWITCH, &action);
    dialog.set_response_appearance(SWITCH, ResponseAppearance::Suggested);
    dialog.set_default_response(Some(CANCEL));
    dialog.set_close_response(CANCEL);
    dialog.connect_response(None, move |dialog, response| {
        dialog.close();
        on_response(response == SWITCH);
    });
    dialog.present(Some(parent));
}

/// Show `action` to the user.
///
/// Refusals are not operations, so they go to the application log only: the
/// panel must not light up because the user clicked something the app was never
/// going to do. Only the confirmed switch opens the panel, because from there
/// on steps are actually being taken.
pub fn present(parent: &ApplicationWindow, action: Action) {
    match action {
        Action::RefuseCrossCompositor { target, current } => {
            crate::logging::info(&format!(
                "refused: {target} is not the running compositor ({current} is)"
            ));
            refuse_cross_compositor(parent, target, current);
        }
        Action::RefuseUndetected { reason } => {
            crate::logging::info(&format!("refused: {reason}"));
            refuse_undetected(parent, &reason);
        }
        Action::AlreadyRunning { shell } => {
            crate::logging::info(&format!("no-op: {shell} is already running"));
            already_running(parent, &shell);
        }
        Action::NotInstalled { shell } => {
            crate::logging::info(&format!("no-op: {shell} is not installed"));
            not_installed(parent, &shell);
        }
        Action::ConfirmSwitch { .. } => {
            // Handled by the caller, which owns the shared selection state and
            // can therefore start the operation. Reaching this arm means the
            // caller sent the action back without acting on it, which would be a
            // bug rather than a state, so it is recorded instead of ignored.
            crate::logging::info("switch confirmed but not started: caller did not handle it");
        }
    }
}

#[cfg(test)]
#[path = "../tests/ui_dialog.rs"]
mod tests;
