//! The page, assembled from parts.
//!
//! This is the "index" of the UI: it owns the shared selection state and wires
//! the tab bar, the compositor stack and the log panel together. It knows
//! nothing about individual shells, and nothing about how a compositor is
//! implemented, so backend work never has to touch this file.
//!
//! Every click on a shell card goes through `Selection::activate`, which is the
//! only place the safety gate is invoked from.

pub mod card;
pub mod dialog;
pub mod grid;
pub mod log;
pub mod logo;
pub mod preview;
pub mod tabbar;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Revealer, TextView, ToggleButton};
use libadwaita::ApplicationWindow;

use crate::compositor::{self, Compositor, Detection};

/// One selectable shell card, remembered so selecting one can deselect the rest.
struct Item {
    shell_id: &'static str,
    button: ToggleButton,
    label: Label,
}

/// One switch at a time.
///
/// A switch is not re-entrant. Two workers at once would race on the backup
/// directory, and once an install is involved they would each raise their own
/// polkit prompt, so the user could authorize one transaction while a second one
/// they never saw was already writing.
#[derive(Default)]
struct Busy(std::cell::Cell<bool>);

impl Busy {
    /// Take the switch, or report that it is already taken.
    fn claim(&self) -> bool {
        if self.0.get() {
            return false;
        }
        self.0.set(true);
        true
    }

    fn release(&self) {
        self.0.set(false);
    }
}

/// State shared by the tab bar and every card.
pub struct Selection {
    log: TextView,
    panel: Revealer,
    items: Vec<Item>,
    detection: Detection,
    /// Weak, because the window owns this widget: a strong reference would make
    /// the two keep each other alive.
    window: gtk4::glib::WeakRef<ApplicationWindow>,
    /// Claimed when a worker starts, released by the drain when its channel
    /// closes.
    busy: Busy,
}

impl Selection {
    pub fn new(
        log: TextView,
        panel: Revealer,
        detection: Detection,
        window: &ApplicationWindow,
    ) -> Self {
        Self {
            log,
            panel,
            items: Vec::new(),
            detection,
            window: window.downgrade(),
            busy: Busy::default(),
        }
    }

    pub fn register(&mut self, shell_id: &'static str, button: &ToggleButton, label: &Label) {
        self.items.push(Item {
            shell_id,
            button: button.clone(),
            label: label.clone(),
        });
    }

    /// Lock or unlock every card while a switch runs.
    ///
    /// `Busy` already refuses a second switch, but only after the user has been
    /// walked through a confirmation for it. Dimming the cards up front says
    /// "wait" before the click instead of after it. The card being switched to
    /// keeps its mark and stays lit (see `.card:disabled:checked`).
    fn set_locked(&self, locked: bool) {
        for item in self.items.iter() {
            item.button.set_sensitive(!locked);
        }
    }

    /// Undo any pending selection. Used when a click ends in a refusal or a
    /// cancel: a card left selected suggests a switch is waiting to happen.
    fn clear_highlight(&mut self) {
        for item in self.items.iter() {
            item.button.set_active(false);
            item.label.remove_css_class("selected");
        }
    }

    /// Single-selection: clear every other card, then mark this one.
    fn highlight(&mut self, shell_id: &str) {
        for item in self.items.iter() {
            if item.shell_id != shell_id {
                item.button.set_active(false);
                item.label.remove_css_class("selected");
            }
        }
        if let Some(item) = self.items.iter().find(|i| i.shell_id == shell_id) {
            item.label.add_css_class("selected");
        }
    }

    /// Browsing, inventory, refusals. Application log only: a panel that shows
    /// these is a second console instead of a progress indicator.
    pub fn note(&self, line: &str) {
        crate::logging::info(line);
    }

    /// A new operation starts on an empty panel. The previous operation's
    /// lines are its own: left in place they read as steps of the new one,
    /// already done, under a panel that just lit up.
    fn clear_log(&self) {
        self.log.buffer().set_text("");
    }

    /// One step of a running operation. Both channels, and the panel is shown.
    pub fn progress(&self, line: &str) {
        log::step(&self.log, &self.panel, line);
    }

    /// The operation finished: the final line is recorded, the panel hides.
    pub fn succeeded(&self, line: &str) {
        log::succeeded(&self.panel, line);
    }

    /// Something failed: the panel stays until the user closes it.
    pub fn failed(&self, line: &str) {
        log::failed(&self.log, &self.panel, line);
    }

    pub fn detection(&self) -> &Detection {
        &self.detection
    }

    /// A card was clicked. Runs the safety gate, and starts the switch if the
    /// user says yes.
    pub fn activate(
        &mut self,
        shared: &Shared,
        compositor: &'static dyn Compositor,
        shell: &'static crate::shell::Shell,
    ) {
        // State is gathered here, once per click, so the routing below stays a
        // pure function of it.
        let state = dialog::ShellState {
            installed: shell.installed(),
            running: shell.running(),
            running_shell: dialog::running_shell(),
        };
        let action = dialog::decide(&self.detection, compositor, shell, state);

        // Resolved before anything is marked, so the one path that gives up
        // early cannot strand a mark it has already drawn.
        let Some(window) = self.window.upgrade() else {
            return;
        };

        if let dialog::Action::ConfirmSwitch { from, to, install } = &action {
            self.highlight(shell.id);
            let current = self.detection.current.map(str::to_string);
            let from = from.clone();
            let to = *to;
            let install = *install;
            // The callbacks outlive this call, so they take their own copy of
            // the shared state rather than a borrow of it.
            let owned = shared.clone();
            let marks = shared.clone();
            let decided = action.clone();
            dialog::confirm_switch(
                &window,
                from.as_deref(),
                shell.id,
                to,
                install,
                move |confirmed| {
                    // Every ending arrives here, so this is the only place that
                    // can take the mark back on the ones where no switch starts.
                    // Leaving it to the confirm arm is what used to strand it:
                    // the mark outlived a cancelled dialog, and a card left
                    // active swallows the next click on it.
                    if !dialog::keeps_mark(&decided, confirmed) {
                        marks.borrow_mut().clear_highlight();
                    }
                    if confirmed {
                        begin(&owned, compositor, current.clone(), shell);
                    }
                },
            );
            return;
        }

        self.clear_highlight();
        dialog::present(&window, action);
    }

    /// A compositor that is not the running one: cards are inert, but
    /// they still explain themselves instead of looking broken.
    pub fn guard_reason(&self, compositor: &'static dyn Compositor) -> Option<String> {
        if let Some(current) = self.detection.current {
            if current != compositor.id() {
                return Some(format!(
                    "{} is not the running compositor — {} is. Switching image \
                     is done in \"RakuOS system management\".",
                    compositor.label(),
                    current
                ));
            }
            return None;
        }
        self.detection.refusal()
    }

    /// Clicked the placeholder on a page that the guard is blocking. Shows the
    /// refusal instead of silently doing nothing.
    pub fn guard_click(&mut self, compositor: &'static dyn Compositor) {
        let reason = self
            .guard_reason(compositor)
            .unwrap_or_else(|| "refused: nothing to switch to.".to_string());
        self.note(&format!("refused: {reason}"));

        let Some(window) = self.window.upgrade() else {
            return;
        };
        match self.detection.current {
            Some(current) => dialog::refuse_cross_compositor(&window, compositor.id(), current),
            None => dialog::refuse_undetected(&window, &reason),
        }
    }
}

/// One line of progress from the worker thread.
#[derive(Debug, PartialEq)]
enum Line {
    Step(String),
    Done(String),
    Failed(String),
}

/// Run the switch off the main loop and report it in the panel.
///
/// The work is blocking: copying the config tree and shelling out to rum both
/// freeze a UI for seconds. So it runs on a worker thread and the panel is fed
/// from a channel drained back on the main loop, because GTK widgets may only be
/// touched from there.
fn begin(
    shared: &Shared,
    compositor: &'static dyn Compositor,
    current: Option<String>,
    target: &'static crate::shell::Shell,
) {
    // Claim the switch before anything else. Refusing here costs the user one
    // line in the panel; letting two run costs them a backup race and a second
    // polkit prompt they never saw.
    if !shared.borrow().busy.claim() {
        shared
            .borrow()
            .failed("a switch is already running; wait for it to finish");
        return;
    }
    shared.borrow().set_locked(true);
    shared.borrow().clear_log();

    let (tx, rx) = std::sync::mpsc::channel::<Line>();
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
    let backups = crate::session::State::backups_dir();

    std::thread::spawn(move || {
        let Some(home) = home else {
            let _ = tx.send(Line::Failed(
                "HOME is not set, cannot locate your config".into(),
            ));
            return;
        };

        let _ = tx.send(Line::Step("backing up your configuration".to_string()));
        let running = dialog::running_shell();
        if let Some(shell) = running {
            let _ = tx.send(Line::Step(format!(
                "outgoing shell: {}",
                shell.display_name
            )));
        }

        let roots = crate::switch::Roots {
            home: home.clone(),
            backups_root: backups.clone(),
        };
        let mut say = |text: &str| {
            let _ = tx.send(Line::Step(text.to_string()));
        };
        // The same pipeline the command line runs, so the window cannot drift away
        // from what actually happens.
        match crate::switch::everything(
            compositor,
            current.as_deref(),
            target,
            running,
            &roots,
            true,
            &mut say,
        ) {
            crate::switch::Outcome::Done(line) => {
                let _ = tx.send(Line::Done(line));
            }
            crate::switch::Outcome::Failed(line) => {
                let _ = tx.send(Line::Failed(line));
            }
        }
    });

    let polled = shared.clone();
    gtk4::glib::timeout_add_local(Duration::from_millis(80), move || drain(&polled, &rx));
}

/// Drain everything the worker has sent so far, without blocking.
///
/// Split out of [`drain`] so it can be tested without a display. It is a loop
/// with a deliberate exit on every branch, because the exit on
/// [`TryRecvError::Disconnected`] is the whole point: once the worker thread
/// ends and drops its sender, `try_recv` returns `Disconnected` for ever, so a
/// loop that keeps going after it never terminates. That bug spent one whole
/// core at 100% inside the GTK timeout callback, which starved the main loop,
/// so the window stopped painting and the compositor declared the app hung.
/// Nothing about it looked like a crash, which is why it survived several
/// attempts to find it by reading the code.
///
/// Returns the lines and whether the operation is over.
fn receive(rx: &Receiver<Line>) -> (Vec<Line>, bool) {
    let mut lines = Vec::new();
    let mut finished = false;
    loop {
        match rx.try_recv() {
            Ok(line) => {
                if matches!(line, Line::Done(_) | Line::Failed(_)) {
                    finished = true;
                }
                lines.push(line);
                // A worker sends its steps first and its final line last, so once
                // the final line is in hand there is nothing after it to wait for.
                if finished {
                    break;
                }
            }
            // Every sender is gone: no line can ever arrive again.
            Err(TryRecvError::Disconnected) => {
                finished = true;
                break;
            }
            // Nothing buffered, but the worker may still be busy.
            Err(TryRecvError::Empty) => break,
        }
    }
    (lines, finished)
}

fn drain(shared: &Shared, rx: &Receiver<Line>) -> gtk4::glib::ControlFlow {
    let (lines, finished) = receive(rx);
    let selection = shared.borrow();
    for line in lines {
        match line {
            Line::Step(text) => selection.progress(&text),
            Line::Done(text) => selection.succeeded(&text),
            Line::Failed(text) => selection.failed(&text),
        }
    }
    if finished {
        selection.busy.release();
        selection.set_locked(false);
        gtk4::glib::ControlFlow::Break
    } else {
        gtk4::glib::ControlFlow::Continue
    }
}

pub type Shared = Rc<RefCell<Selection>>;

pub struct Picker {
    root: GtkBox,
}

impl Picker {
    pub fn new(window: &ApplicationWindow) -> Self {
        let log = log::build();
        let panel = log::panel(&log);
        let detection = compositor::detect();

        // Version, compositor and inventory belong to the application log. The
        // panel stays hidden until an operation actually starts.
        crate::logging::startup(&detection);

        let shared: Shared = Rc::new(RefCell::new(Selection::new(
            log.clone(),
            panel.clone(),
            detection.clone(),
            window,
        )));

        // One tab per compositor, one page per compositor, straight from the
        // registry. Registering a shell touches no UI file.
        let stack = gtk4::Stack::new();
        stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
        stack.set_vexpand(true);
        for compositor in compositor::all() {
            stack.add_named(&grid::build(compositor, &shared), Some(compositor.id()));
        }
        stack.set_visible_child_name(compositor::default_id(&detection));

        let root = GtkBox::new(Orientation::Vertical, 0);
        root.add_css_class("picker-root");
        root.append(&tabbar::build(&stack, &shared));
        root.append(&stack);
        root.append(&panel);

        Self { root }
    }

    pub fn widget(&self) -> &gtk4::Widget {
        self.root.upcast_ref()
    }
}

#[cfg(test)]
#[path = "../tests/ui_receive.rs"]
mod receive_tests;

#[cfg(test)]
#[path = "../tests/ui_busy.rs"]
mod busy_tests;
