//! Channel 2 of docs/LOGGING.md: the log panel inside the window.
//!
//! The panel is not a console. It shows the steps of an operation while one is
//! running, and nothing else: no version banner, no inventory, no record of the
//! user browsing around. Those go to the application log instead, so that a
//! panel that is visible always means "something is happening".

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation, Revealer, TextView};

use crate::logging;

/// Fixed height for the log panel at the bottom of the window.
pub const LOG_HEIGHT: i32 = 100;

pub fn build() -> TextView {
    let view = TextView::new();
    view.set_editable(false);
    view.set_cursor_visible(false);
    view.set_monospace(true);
    view.add_css_class("log-view");
    view
}

/// The panel: a header with a title and a close button, over the text view.
///
/// Returned as a `Revealer` so the whole thing can be hidden while idle. An
/// earlier version returned the inner scroller, which silently dropped all of
/// the styling, and the panel was always visible.
pub fn panel(log: &TextView) -> Revealer {
    let title = Label::new(Some("Progress"));
    title.add_css_class("log-title");
    title.set_hexpand(true);
    title.set_halign(Align::Start);

    let close = Button::from_icon_name("window-close-symbolic");
    close.add_css_class("log-close");
    close.set_tooltip_text(Some("Hide this panel"));

    let header = GtkBox::new(Orientation::Horizontal, 8);
    header.append(&title);
    header.append(&close);

    let scroller = gtk4::ScrolledWindow::builder()
        .child(log)
        .hexpand(true)
        .vexpand(true)
        .build();
    // Long lines (package lists) scroll sideways instead of being cut off.
    scroller.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    scroller.set_propagate_natural_width(false);
    scroller.set_propagate_natural_height(false);

    let frame = GtkBox::new(Orientation::Vertical, 0);
    frame.add_css_class("log-frame");
    frame.set_size_request(-1, LOG_HEIGHT);
    frame.append(&header);
    frame.append(&scroller);

    let revealer = Revealer::new();
    revealer.set_child(Some(&frame));
    // The log scroller inside is vexpand, and expand propagates upwards, so left
    // alone this hidden panel still claims its share of the free height and the
    // card area stops short, leaving a blank band at the bottom. Pin it to its
    // natural height instead.
    revealer.set_vexpand(false);
    // Idle means hidden. A panel that is always on screen is a second console,
    // which is what this was before.
    revealer.set_reveal_child(false);

    // Only for the failure case: a reason must not vanish on its own.
    let dismissable = revealer.clone();
    close.connect_clicked(move |_| {
        dismissable.set_reveal_child(false);
    });

    revealer
}

fn append(log: &TextView, line: &str) {
    let buffer = log.buffer();
    let mut end = buffer.end_iter();
    buffer.insert(&mut end, line);
    buffer.insert(&mut end, "\n");
}

/// One step of a running operation: application log **and** panel.
///
/// Anything that is not a step of an operation must use `crate::logging`
/// directly, so it stays out of the panel.
pub fn step(log: &TextView, revealer: &Revealer, line: &str) {
    logging::info(line);
    append(log, line);
    revealer.set_reveal_child(true);
}

/// The operation finished. The final line goes to the application log and the
/// panel goes away again, because there is nothing left to report.
pub fn succeeded(revealer: &Revealer, line: &str) {
    logging::info(line);
    revealer.set_reveal_child(false);
}

/// The operation failed. The panel stays until the user closes it, because a
/// failure that disappears on its own is a failure the user never learns about.
pub fn failed(log: &TextView, revealer: &Revealer, line: &str) {
    logging::error(line);
    append(log, line);
    revealer.set_reveal_child(true);
}
