//! The shell grid for one compositor, and the placeholder for an empty one.
//!
//! A wrapping flow of cards, at most [`MAX_PER_ROW`] to a row. The window is
//! free-sized (a tiling compositor decides it), so the number of columns follows
//! the width instead of being fixed: wide window, one row of up to five; narrow
//! window, the cards wrap. Whatever still does not fit scrolls, vertically or
//! horizontally, so the window never has to grow and nothing is clipped.

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation};

use crate::compositor::Compositor;
use crate::ui::{card, logo, Shared};

/// Most cards in one row; fewer when the window is narrower.
const MAX_PER_ROW: u32 = 5;
const COLUMN_SPACING: u32 = 20;
const ROW_SPACING: u32 = 28;
/// Widest the card area gets. Past this the cards stay together in the middle
/// instead of drifting apart on a very wide window.
const MAX_WIDTH: i32 = 960;
/// Breathing room between the cards and the window edge.
const SIDE_MARGIN: i32 = 24;
/// Width used by the "coming soon" placeholder, which spans the grid.
const CARD_WIDE: i32 = 320;

pub fn build(compositor: &'static dyn Compositor, shared: &Shared) -> gtk4::ScrolledWindow {
    let shells = compositor.shells();
    // The warning is about the compositor, not about how full the page is. It
    // is shown up front either way, so the reason is never something the user
    // only discovers after clicking.
    let guarded = shared.borrow().guard_reason(compositor);

    if shells.is_empty() {
        return scroller(placeholder(compositor, shared).upcast());
    }

    let column = GtkBox::new(Orientation::Vertical, 0);
    column.set_halign(Align::Fill);
    column.set_hexpand(true);
    column.set_valign(Align::Start);
    column.set_spacing(14);
    column.set_margin_top(14);
    column.set_margin_bottom(14);

    if let Some(reason) = &guarded {
        column.append(&banner(reason));
    }

    let flow = gtk4::FlowBox::new();
    flow.set_selection_mode(gtk4::SelectionMode::None);
    flow.set_homogeneous(true);
    flow.set_min_children_per_line(1);
    // Never more columns than there are cards: a homogeneous FlowBox spreads its
    // slots over the full width, so four cards in five slots would leave a gap
    // on the right instead of sitting centred.
    flow.set_max_children_per_line((shells.len() as u32).clamp(1, MAX_PER_ROW));
    flow.set_column_spacing(COLUMN_SPACING);
    flow.set_row_spacing(ROW_SPACING);
    flow.set_halign(Align::Fill);
    flow.set_hexpand(true);
    flow.set_valign(Align::Start);
    flow.set_margin_start(SIDE_MARGIN);
    flow.set_margin_end(SIDE_MARGIN);

    for shell in shells.iter() {
        flow.insert(&card::build(shell, compositor, shared), -1);
    }
    let clamp = libadwaita::Clamp::builder()
        .maximum_size(MAX_WIDTH)
        .tightening_threshold(MAX_WIDTH)
        .child(&flow)
        .build();
    column.append(&clamp);

    scroller(column.upcast())
}

/// The up-front reason a page cannot be used. Always the first thing on the
/// page, so it reads as a banner and not as an afterthought.
fn banner(reason: &str) -> Label {
    let label = Label::new(Some(reason));
    label.set_justify(gtk4::Justification::Center);
    label.set_max_width_chars(46);
    label.set_wrap(true);
    label.set_halign(Align::Center);
    label.add_css_class("guard-banner");
    label
}

/// Shown when a compositor has no shells registered yet.
///
/// When that compositor is also not the running one, this doubles as the guard
/// surface: the card stays clickable so the refusal can actually be seen, and
/// a banner above it says why up front.
fn placeholder(compositor: &'static dyn Compositor, shared: &Shared) -> GtkBox {
    let column = GtkBox::new(Orientation::Vertical, 0);
    column.set_halign(Align::Center);
    column.set_valign(Align::Center);
    column.set_spacing(16);

    let guarded = shared.borrow().guard_reason(compositor);

    if let Some(reason) = &guarded {
        column.append(&banner(reason));
    }

    let card = Button::new();
    card.set_halign(Align::Center);
    card.set_size_request(CARD_WIDE, -1);
    card.add_css_class("card");
    card.add_css_class("card-wide");
    if guarded.is_some() {
        card.add_css_class("card-guard");
    }

    let inner = GtkBox::new(Orientation::Vertical, 0);
    inner.set_halign(Align::Center);
    inner.set_valign(Align::Center);

    let mark = Label::new(Some(if guarded.is_some() { "!" } else { "?" }));
    mark.add_css_class("logo-fallback");
    if guarded.is_none() {
        mark.add_css_class("dim-fallback");
    }
    mark.set_halign(Align::Center);
    mark.set_valign(Align::Center);
    inner.append(&logo::frame(&mark.upcast()));

    let title = Label::new(Some(if guarded.is_some() {
        "Blocked"
    } else {
        "Coming soon"
    }));
    title.add_css_class("card-label");
    title.set_halign(Align::Center);
    inner.append(&title);

    let sub = Label::new(Some(&format!(
        "no {} shells registered yet",
        compositor.id()
    )));
    sub.add_css_class("card-sub");
    sub.set_halign(Align::Center);
    inner.append(&sub);

    card.set_child(Some(&inner));

    let selected = shared.clone();
    card.connect_clicked(move |_| {
        selected.borrow_mut().guard_click(compositor);
    });

    column.append(&card);
    column
}

fn scroller(child: gtk4::Widget) -> gtk4::ScrolledWindow {
    let scroller = gtk4::ScrolledWindow::builder()
        .child(&child)
        .vexpand(true)
        .build();
    // Both directions scroll when needed. The flow wraps first, so sideways
    // scrolling only appears once the window is narrower than a single card.
    scroller.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    scroller.set_propagate_natural_width(false);
    scroller.set_propagate_natural_height(false);
    scroller.set_hexpand(true);
    scroller
}
