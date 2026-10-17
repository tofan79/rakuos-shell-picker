//! Shells runnable under Niri.
//!
//! Same shape as `compositor::hyprland::shells`: one file per shell, each stating
//! only what is specific to Niri, with everything true of the shell anywhere left
//! in `crate::shells`. No folder imports another.
//!
//! Niri's four are the ones the `rakuos-niri` image knows: DMS in the base
//! image, Noctalia and Ambxst installed from the same repository the other
//! shells come from, and iNiR — a first-party Niri shell whose card exists
//! ahead of its backend. Caelestia and end4-pC are absent because both are
//! Hyprland shells upstream. A card existing on one compositor and not
//! another is the reason these registries are per compositor rather than one
//! shared list.
//!
//! The DMS and Noctalia skels come from the image's own `/etc/skel`, so they are
//! the files a fresh Niri install starts from rather than a hand-written set.
//! Ambxst's is this repository's, because the image does not ship it.

pub mod ambxst;
pub mod dms;
pub mod inir;
pub mod noctalia;

use crate::shell::Shell;

/// Card order in the grid. DMS is first because it is the shell the image ships
/// running; iNiR last because its port is a card first.
pub static ALL: &[Shell] = &[dms::SHELL, noctalia::SHELL, ambxst::SHELL, inir::SHELL];
