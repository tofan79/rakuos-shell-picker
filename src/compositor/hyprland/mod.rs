//! Hyprland backend.
//!
//! Compositor-specific tooling for Hyprland (hyprctl wrappers, window rules,
//! autostart handling) belongs here, next to the shells it applies to.

pub mod shells;

use crate::compositor::{Compositor, EnvSource};
use crate::shell::Shell;

pub struct Hyprland;

impl Compositor for Hyprland {
    fn id(&self) -> &'static str {
        "hyprland"
    }

    fn label(&self) -> &'static str {
        "hyprland"
    }

    fn shells(&self) -> &'static [Shell] {
        shells::ALL
    }

    fn config_dir(&self) -> &'static str {
        ".config/hypr"
    }

    fn marker_env(&self) -> &'static str {
        "HYPRLAND_INSTANCE_SIGNATURE"
    }

    fn is_running_with(&self, env: &dyn EnvSource) -> bool {
        let Some(signature) = env.var(self.marker_env()) else {
            return false;
        };
        // The signature is set for every child of Hyprland, so it is the signal
        // we trust. The socket is only a soft confirmation: its exact path has
        // moved between Hyprland releases, and a wrong path would turn a
        // running Hyprland into "not running". When the path cannot be worked
        // out at all, the marker stands on its own.
        match self.socket_path(env, &signature) {
            Some(path) => path.exists(),
            None => true,
        }
    }

    fn reload(&self) -> Result<(), String> {
        let out = std::process::Command::new("hyprctl")
            .arg("reload")
            .output()
            .map_err(|e| format!("cannot run hyprctl reload: {e}"))?;
        if out.status.success() {
            return Ok(());
        }
        let detail = String::from_utf8_lossy(&out.stderr);
        let detail = detail.trim();
        Err(if detail.is_empty() {
            format!("hyprctl reload exited with {}", out.status)
        } else {
            format!("hyprctl reload: {detail}")
        })
    }

    fn exit(&self) -> Result<(), String> {
        // Hyprland 0.56 replaced the string dispatchers with a Lua API:
        // `hyprctl dispatch exit` is read as the Lua identifier `exit` and
        // rejected with "expected a dispatcher" (exit status 7), so the
        // session never ends. `hl.dsp.exit()` (`hl.meta.lua`'s
        // `HL.DspNamespace`) is the current spelling; the string form stays
        // as the fallback for compositors that predate the change.
        let lua = std::process::Command::new("hyprctl")
            .args(["dispatch", "hl.dsp.exit()"])
            .output()
            .map_err(|e| format!("cannot run hyprctl dispatch hl.dsp.exit(): {e}"))?;
        if lua.status.success() {
            return Ok(());
        }
        let out = std::process::Command::new("hyprctl")
            .args(["dispatch", "exit"])
            .output()
            .map_err(|e| format!("cannot run hyprctl dispatch exit: {e}"))?;
        if out.status.success() {
            return Ok(());
        }
        let detail = String::from_utf8_lossy(&out.stderr);
        let detail = detail.trim();
        Err(if detail.is_empty() {
            format!(
                "hyprctl dispatch hl.dsp.exit() exited with {}; hyprctl dispatch exit with {}",
                lua.status, out.status
            )
        } else {
            format!("hyprctl dispatch exit: {detail}")
        })
    }
}

impl Hyprland {
    /// `<runtime dir>/hypr/<signature>/.socket.sock`
    fn socket_path(&self, env: &dyn EnvSource, signature: &str) -> Option<std::path::PathBuf> {
        let runtime_dir = env.var("XDG_RUNTIME_DIR")?;
        let base = std::path::Path::new(&runtime_dir)
            .join("hypr")
            .join(signature);
        for candidate in [".socket.sock", ".socket2.sock"] {
            let path = base.join(candidate);
            if path.exists() {
                return Some(path);
            }
        }
        // Signature directory exists but neither socket name does: let the
        // caller see the directory so we do not fall back to trusting the
        // marker over contradicting evidence.
        base.exists().then_some(base.join(".socket.sock"))
    }
}
