//! Shell logos, embedded from `assets/` as SVG.
//!
//! GTK 4.22 renders SVG through GdkTexture, so there is no raster step and no
//! PNG fallback to maintain. The file name comes from `Shell::logo`, which is
//! declared next to the shell itself.
//!
//! The SVGs are compiled into the binary with `include_bytes!` rather than read
//! from `assets/` at runtime. `env!("CARGO_MANIFEST_DIR")` points at the build
//! tree, so a packaged binary could never find them there; embedding keeps the
//! logos working wherever the binary is installed, with no data files to ship.

use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Image, Label, Orientation};

use crate::shell::Shell;

/// Painted edge length of the logo artwork.
pub const LOGO_SIZE: i32 = 72;
/// Size of the bordered box that holds it.
pub const LOGO_BOX: i32 = 100;

/// The bordered, centred container. Shared by shell cards and the placeholder.
pub fn frame(child: &gtk4::Widget) -> GtkBox {
    let frame = GtkBox::new(Orientation::Horizontal, 0);
    frame.set_halign(Align::Center);
    frame.set_valign(Align::Center);
    frame.set_size_request(LOGO_BOX, LOGO_BOX);
    frame.add_css_class("logo-frame");
    // A horizontal box packs its children from the start, so `halign(Center)` on
    // the child alone does nothing sideways: it is centred inside a slot exactly
    // as wide as itself. Letting the child expand makes the slot as wide as the
    // box, and the alignment then centres the logo in it.
    child.set_hexpand(true);
    child.set_vexpand(true);
    frame.append(child);
    frame
}

pub fn build(shell: &Shell) -> gtk4::Widget {
    let widget: gtk4::Widget = match load(shell) {
        Some(texture) => {
            let image = Image::new();
            image.set_pixel_size(LOGO_SIZE);
            // pixel_size only sets a minimum, so pin the request too; otherwise
            // the image can allocate at the SVG's natural size and overflow.
            image.set_size_request(LOGO_SIZE, LOGO_SIZE);
            image.set_paintable(Some(&texture));
            image.upcast()
        }
        None => letter_fallback(shell),
    };

    widget.set_halign(Align::Center);
    widget.set_valign(Align::Center);
    widget
}

/// The logo files, compiled in. `Shell::logo` names one of these; a name that
/// is not here (or a shell with no logo) has no texture and gets the letter.
fn logo_bytes(name: &str) -> Option<&'static [u8]> {
    Some(match name {
        "ambxst-logo-color.svg" => include_bytes!("../../assets/ambxst-logo-color.svg").as_slice(),
        "caelestia.svg" => include_bytes!("../../assets/caelestia.svg").as_slice(),
        "danklogo2.svg" => include_bytes!("../../assets/danklogo2.svg").as_slice(),
        "noctalia.svg" => include_bytes!("../../assets/noctalia.svg").as_slice(),
        _ => return None,
    })
}

fn load(shell: &Shell) -> Option<gdk::Texture> {
    let name = shell.logo?;
    let Some(bytes) = logo_bytes(name) else {
        crate::logging::warn(&format!("{}: unknown logo {name}", shell.id));
        return None;
    };
    match gdk::Texture::from_bytes(&glib::Bytes::from_static(bytes)) {
        Ok(texture) => Some(texture),
        Err(err) => {
            crate::logging::warn(&format!("{}: cannot load logo: {err}", shell.id));
            None
        }
    }
}

fn letter_fallback(shell: &Shell) -> gtk4::Widget {
    let initial = shell
        .display_name
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".into());
    let label = Label::new(Some(&initial));
    label.add_css_class("logo-fallback");
    label.upcast()
}
