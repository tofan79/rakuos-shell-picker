//! One selectable shell card: logo, name, status.
//!
//! Clicking a card never runs anything. It only asks the safety gate what the
//! click means; see `super::dialog`.

use gtk4::pango;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, Orientation, ToggleButton};

use crate::compositor::Compositor;
use crate::shell::Shell;
use crate::ui::{logo, Shared};

/// Card width is pinned here (not CSS) so every column is the same width and the
/// flow in `grid.rs` can wrap on a clean boundary at any window size.
pub const CARD_WIDTH: i32 = 160;

pub fn build(
    shell: &'static Shell,
    compositor: &'static dyn Compositor,
    shared: &Shared,
) -> ToggleButton {
    let btn = ToggleButton::new();
    btn.set_size_request(CARD_WIDTH, -1);
    // Centred in its slot: the slot grows with the window, the card does not.
    btn.set_halign(Align::Center);
    btn.add_css_class("card");

    let status = shell.status_label();
    if !shell.installed() {
        btn.add_css_class("unavailable");
    }
    if shell.running() {
        btn.add_css_class("running");
    }

    let col = GtkBox::new(Orientation::Vertical, 0);
    col.set_halign(Align::Center);

    let name = Label::new(Some(shell.display_name));
    name.add_css_class("card-label");
    name.set_halign(Align::Center);
    // Bounds the label so a long name can never widen its column.
    name.set_width_chars(12);
    name.set_max_width_chars(12);
    name.set_ellipsize(pango::EllipsizeMode::End);

    let sub = Label::new(Some(status));
    sub.add_css_class("card-sub");
    sub.set_halign(Align::Center);
    if shell.running() {
        // Same class as the button, so CSS can colour the text too.
        sub.add_css_class("running");
    }

    col.append(&logo::frame(&logo::build(shell)));
    col.append(&name);
    col.append(&sub);
    btn.set_child(Some(&col));

    shared.borrow_mut().register(shell.id, &btn, &name);

    let selected = shared.clone();
    btn.connect_toggled(move |btn| {
        if btn.is_active() {
            selected.borrow_mut().activate(&selected, compositor, shell);
        }
    });

    btn
}
