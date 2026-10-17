//! Shells runnable under Hyprland.
//!
//! One file per shell, and each one states only what is specific to Hyprland: where
//! the shell's config lives here, how it starts here, which skel paths its Hyprland
//! skel carries. Everything true of the shell on any compositor lives in
//! `crate::shells` instead, and is not repeated here.
//!
//! ```text
//! shells/dms.rs  ←  the package, the install plan, the logo
//! shells/dms.rs  ←  this file's sibling: DMS on Hyprland
//! ```
//!
//! Every compositor has its own registry with its own answers to the same questions,
//! and no folder imports another. Hyprland is the only one registered today; see
//! `compositor::niri` for the shape of the next one.
//!
//! Why the split is per shell rather than per step: the steps are not the same for
//! any two shells. Noctalia is in the base image and started from a Lua autostart,
//! DMS is installed from a repository and started by a systemd user unit, Caelestia
//! is installed from a repository and probed by a process pattern, Ambxst is
//! installed from the same repository and started as a plain process on a runtime
//! it shares with the others, and end4-pC is the same kind of configuration shell —
//! its port is a card today, the backend half landing with its skel. What they have
//! in common, such as the shared config paths a backup always saves, stays in
//! `src/switch.rs` so there is one copy of it rather than one per shell.

pub mod ambxst;
pub mod caelestia;
pub mod dms;
pub mod end4pc;
pub mod noctalia;

use crate::shell::Shell;

/// Card order in the grid.
pub static ALL: &[Shell] = &[
    noctalia::SHELL,
    dms::SHELL,
    caelestia::SHELL,
    ambxst::SHELL,
    end4pc::SHELL,
];
