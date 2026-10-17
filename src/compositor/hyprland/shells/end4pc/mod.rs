//! end4-pC on Hyprland.
//!
//! Only what is specific to this compositor. What end4-pC is — its package,
//! its probe pattern, its logo — is in [`crate::shells::end4pc`], and is not
//! repeated here.
//!
//! This port is a card first: the fields a switch would execute on are still
//! empty on purpose, and each one lands with the backend that needs it.
//!
//! - **start**: the compositor's start event, the same mechanism Caelestia
//!   uses. The Hyprland configuration a switch will install from this shell's
//!   skel runs `qs -c end4-pC`, and the live switch will run that command
//!   directly — both roads end in the same process.
//! - **config**: `None` for now. Which path a switch seeds — the packaged
//!   payload under `/usr/share/quickshell/end4-pC` or a user copy under
//!   `~/.config/quickshell/end4-pC` — is a backend decision, and until
//!   `config_dir` is declared a switch refuses in `Refusal::NoConfig` before
//!   any backup.
//! - **skel**: none yet. Nothing is installed to the machine, so a switch has
//!   nothing to complete with.

use crate::shell::{Port, Shell, StartMechanism};
use crate::shells::end4pc::IDENTITY;

const PORT: Port = Port {
    config_dir: None,
    state_dir: None,
    systemd_user_unit: None,
    mechanism: StartMechanism::StartEvent,
    skel_extra_paths: &[],
    backup_extra_paths: &[],
};

pub static SHELL: Shell = Shell::join(IDENTITY, PORT);
