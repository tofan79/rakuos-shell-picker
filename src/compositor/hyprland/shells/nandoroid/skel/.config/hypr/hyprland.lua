-- ─────────────────────────────────────────────────────────────────────────────
--  Nandoroid Shell: Main Hyprland Configuration
-- ─────────────────────────────────────────────────────────────────────────────

-- --- Source Modular Configs ---
require("configs/env")
require("configs/execs")
require("configs/general")
require("configs/rules")
require("configs/keybinds")

-- --- Source Nandoroid Shell Keybinds & Rules ---
require("nandoroid/nandoroid")
require("nandoroid/user_persistence")

-- --- Monitors ---
hl.monitor({
    output = "",
    mode = "preferred",
    position = "auto",
    scale = "1"
})