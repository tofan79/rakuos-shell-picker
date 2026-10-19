//! The preview shown in the confirmation dialog, before the user commits to
//! installing or switching to a shell.
//!
//! Screenshots are compiled in from `assets/previews/`, the same way the logos
//! are (see `logo.rs`): a packaged binary cannot find files next to the source
//! tree, and embedding keeps the preview working wherever it is installed.
//!
//! To add one, drop `assets/previews/<shell id>-1.jpg` (16:9, about 720px wide,
//! JPEG or PNG, keep each under ~150 KB) and add a line to [`image_bytes`]. A
//! shell with no entry gets a short "no preview yet" note instead, so adding a
//! shell never has to wait for its screenshots.

use gtk4::prelude::*;
use gtk4::{gdk, glib};
use gtk4::{
    Align, Box as GtkBox, ContentFit, Label, Orientation, Overflow, Picture, PolicyType,
    ScrolledWindow,
};
use libadwaita::{Carousel, CarouselIndicatorDots};

/// Painted size of one preview, 16:9.
pub const WIDTH: i32 = 420;
pub const HEIGHT: i32 = 236;

/// The preview images of one shell, by shell id, in display order.
///
/// Add a shell like this:
/// `"noctalia" => vec![include_bytes!("../../assets/previews/noctalia-1.jpg").as_slice()],`
///
/// Six shots per shell is the working rule: the bar and the dock, a panel, the
/// launcher or an app grid, a settings window, a widget set, and the desktop
/// bare. Six different desktops sell the shell better than six views of one, and
/// the dots still read at a glance. Caelestia starts with four and gains the
/// rest over time — the carousel does not care how many it gets. The files are
/// the ones from `assets/previews/` re-encoded to 720 px wide; see the module
/// comment for the format rules the tests enforce.
///
/// Nothing else has to change for it to gain one.
pub(crate) fn image_bytes(shell_id: &str) -> Vec<&'static [u8]> {
    match shell_id {
        "dms" => vec![include_bytes!("../../assets/previews/dms-1.jpg").as_slice()],
        "ambxst" => vec![
            include_bytes!("../../assets/previews/ambxst-1.jpg").as_slice(),
            include_bytes!("../../assets/previews/ambxst-2.jpg").as_slice(),
            include_bytes!("../../assets/previews/ambxst-3.jpg").as_slice(),
            include_bytes!("../../assets/previews/ambxst-4.jpg").as_slice(),
            include_bytes!("../../assets/previews/ambxst-5.jpg").as_slice(),
            include_bytes!("../../assets/previews/ambxst-6.jpg").as_slice(),
        ],
        "caelestia" => vec![
            include_bytes!("../../assets/previews/caelestia-1.jpg").as_slice(),
            include_bytes!("../../assets/previews/caelestia-2.jpg").as_slice(),
            include_bytes!("../../assets/previews/caelestia-3.jpg").as_slice(),
            include_bytes!("../../assets/previews/caelestia-4.jpg").as_slice(),
        ],
        "end4pc" => vec![
            include_bytes!("../../assets/previews/end4pc-1.jpg").as_slice(),
            include_bytes!("../../assets/previews/end4pc-2.jpg").as_slice(),
            include_bytes!("../../assets/previews/end4pc-3.jpg").as_slice(),
            include_bytes!("../../assets/previews/end4pc-4.jpg").as_slice(),
            include_bytes!("../../assets/previews/end4pc-5.jpg").as_slice(),
            include_bytes!("../../assets/previews/end4pc-6.jpg").as_slice(),
        ],
        "inir" => vec![
            include_bytes!("../../assets/previews/inir-1.jpg").as_slice(),
            include_bytes!("../../assets/previews/inir-2.jpg").as_slice(),
            include_bytes!("../../assets/previews/inir-3.jpg").as_slice(),
            include_bytes!("../../assets/previews/inir-4.jpg").as_slice(),
            include_bytes!("../../assets/previews/inir-5.jpg").as_slice(),
            include_bytes!("../../assets/previews/inir-6.jpg").as_slice(),
        ],
        "noctalia" => vec![
            include_bytes!("../../assets/previews/noctalia-1.jpg").as_slice(),
            include_bytes!("../../assets/previews/noctalia-2.jpg").as_slice(),
            include_bytes!("../../assets/previews/noctalia-3.jpg").as_slice(),
            include_bytes!("../../assets/previews/noctalia-4.jpg").as_slice(),
            include_bytes!("../../assets/previews/noctalia-5.jpg").as_slice(),
            include_bytes!("../../assets/previews/noctalia-6.jpg").as_slice(),
        ],
        _ => Vec::new(),
    }
}

/// Decode what is registered for `shell_id`. A file that does not decode is
/// logged and skipped, so one bad screenshot cannot take the dialog down.
fn textures(shell_id: &str) -> Vec<gdk::Texture> {
    image_bytes(shell_id)
        .into_iter()
        .filter_map(
            |bytes| match gdk::Texture::from_bytes(&glib::Bytes::from_static(bytes)) {
                Ok(texture) => Some(texture),
                Err(err) => {
                    crate::logging::warn(&format!("{shell_id}: cannot load preview: {err}"));
                    None
                }
            },
        )
        .collect()
}

/// One image in a fixed-size frame.
///
/// A `Picture` reports the image's own size as its natural size, so left alone a
/// large screenshot would widen the whole dialog. A scrolled window with fixed
/// minimum content size does not propagate its child's size, which pins the
/// frame; the picture then scales to fit inside it.
///
/// The policy is `External`, not `Never`: with `Never` GTK ignores
/// `min-content-width/height` and the frame collapses to a couple of pixels.
/// `External` shows no scrollbar and keeps the minimum.
fn framed(texture: &gdk::Texture) -> ScrolledWindow {
    let picture = Picture::for_paintable(texture);
    picture.set_content_fit(ContentFit::Contain);
    picture.set_can_shrink(true);

    let frame = ScrolledWindow::new();
    frame.set_policy(PolicyType::External, PolicyType::External);
    frame.set_min_content_width(WIDTH);
    frame.set_min_content_height(HEIGHT);
    frame.set_halign(Align::Center);
    // The CSS radius rounds the background only. Clipping is what stops the
    // picture from painting square corners over it.
    frame.set_overflow(Overflow::Hidden);
    frame.add_css_class("preview-frame");
    frame.set_child(Some(&picture));
    frame
}

/// The preview for `shell_id`: its screenshot, a swipeable set of them with
/// dots when there are several, or a short note when there are none.
pub fn build(shell_id: &str) -> gtk4::Widget {
    let textures = textures(shell_id);
    if textures.is_empty() {
        let note = Label::new(Some("No preview available yet."));
        note.add_css_class("preview-empty");
        return note.upcast();
    }
    if let [only] = textures.as_slice() {
        return framed(only).upcast();
    }

    let carousel = Carousel::new();
    carousel.set_halign(Align::Center);
    for texture in &textures {
        carousel.append(&framed(texture));
    }
    let dots = CarouselIndicatorDots::new();
    dots.set_carousel(Some(&carousel));
    dots.add_css_class("preview-dots");

    let column = GtkBox::new(Orientation::Vertical, 8);
    column.append(&carousel);
    column.append(&dots);
    column.upcast()
}

#[cfg(test)]
mod tests {
    use super::image_bytes;
    use crate::compositor;

    #[test]
    fn a_shell_with_no_screenshots_has_no_preview() {
        assert!(image_bytes("no-such-shell").is_empty());
    }

    /// `gdk::Texture` can decode PNG and JPEG without extra loaders, which is
    /// why those are the formats the guide asks for. WebP depends on a loader
    /// the target system may not have, so it must not slip in unnoticed.
    #[test]
    fn every_registered_preview_is_a_png_or_a_jpeg() {
        for compositor in compositor::all() {
            for shell in compositor.shells() {
                for bytes in image_bytes(shell.id) {
                    let png = bytes.starts_with(&[0x89, b'P', b'N', b'G']);
                    let jpeg = bytes.starts_with(&[0xFF, 0xD8, 0xFF]);
                    assert!(png || jpeg, "{}: preview is neither PNG nor JPEG", shell.id);
                }
            }
        }
    }

    /// Six is the ceiling, not a quota: the carousel takes whatever it is given
    /// (Caelestia has four), and a shell with none still gets the note.
    #[test]
    fn no_shell_carries_more_than_six_screenshots() {
        for compositor in compositor::all() {
            for shell in compositor.shells() {
                let shots = image_bytes(shell.id).len();
                assert!(
                    shots <= 6,
                    "{}: {shots} screenshots, more than the six the guide allows",
                    shell.id
                );
            }
        }
    }

    /// A screenshot is embedded in the binary, so its size is paid on every
    /// install. The guide says under 150 KB each and 720 px wide; this keeps a
    /// full-resolution desktop capture from slipping in unnoticed.
    #[test]
    fn every_registered_preview_fits_the_budget() {
        const LIMIT: usize = 150 * 1024;
        for compositor in compositor::all() {
            for shell in compositor.shells() {
                for bytes in image_bytes(shell.id) {
                    assert!(
                        bytes.len() <= LIMIT,
                        "{}: preview is {} bytes, over the {LIMIT}-byte budget",
                        shell.id,
                        bytes.len()
                    );
                }
            }
        }
    }

    /// The id in the `match` is the shell id, not the folder the screenshots
    /// came from: `end4pc`, not `end4-pc`. A typo there registers screenshots
    /// nothing ever asks for, which no other test would notice.
    #[test]
    fn every_shell_with_screenshots_is_registered_under_its_own_id() {
        let mut registered: Vec<&str> = compositor::all()
            .iter()
            .flat_map(|compositor| compositor.shells())
            .map(|shell| shell.id)
            .filter(|id| !image_bytes(id).is_empty())
            .collect();
        // A shell is registered once per compositor it runs on, so the same id
        // arrives more than once.
        registered.sort_unstable();
        registered.dedup();

        assert_eq!(
            registered,
            ["ambxst", "caelestia", "dms", "end4pc", "inir", "noctalia"]
        );
    }
}
