-- RakuOS Hyprland Configuration
--
-- Impasto's modules, loaded from the package: /usr/share/impasto/hypr/modules is
-- symlinked next to this file, so `require("modules.…")` resolves and the
-- modules travel with the package instead of being copied in.
--
-- `modules.autostart` is the one upstream file that is not loaded: it runs
-- `qs -d` (the shell at the root of ~/.config/quickshell), while this package
-- installs it under the config name `impasto`. config/autostart.lua starts it
-- the way the switch does, so a login and a switch agree.
require("modules.monitors")
require("config.autostart")
require("modules.env")
require("modules.look")
require("modules.animations")
require("modules.input")
require("modules.keybinds")
require("modules.windowrules")

-- OVERRIDES
-- Down here you can write or source anything you want to override.
