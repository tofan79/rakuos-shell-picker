-- SekaiPop: Hyprland configuration (Lua config, Hyprland 0.55+).
--
-- Built on end-4's dots-hyprland layout (GPL-3.0), rewritten around the sekaipop shell.
--
--   hyprland/*.lua          the defaults (variables, env, execs, general, rules, keybinds)
--   custom/*.lua            YOUR changes. Loaded after the defaults, never overwritten.
--   workspaces.lua          nwg-displays / the shell's Settings write this one
--   monitors.lua            made for THIS machine by the installer, then written by the shell's
--                           Settings > Hyprland > Displays or nwg-displays. Never shipped.
--   sekaipop_overrides.lua   written by the SHELL (Settings > Hyprland, Game Mode, Anti-Flashbang,
--                           window-border colours, the region picker's layer rules). Keep it last.
--
-- Every part is loaded on its own. If one file has a mistake, the error is written to
-- ~/.cache/sekaipop/hypr-errors.log and the rest still loads, so a typo (or a bad monitors.lua)
-- never leaves you without keybinds or without the shell.

local HOME_DIR = os.getenv("HOME") or ""

local function log_error(name, err)
    os.execute('mkdir -p "' .. HOME_DIR .. '/.cache/sekaipop"')
    local f = io.open(HOME_DIR .. "/.cache/sekaipop/hypr-errors.log", "a")
    if f then
        f:write(os.date("%Y-%m-%d %H:%M:%S") .. "  " .. name .. ": " .. tostring(err) .. "\n")
        f:close()
    end
end

local function load(name, optional)
    local ok, err = pcall(require, name)
    if not ok then
        -- an optional file that does not exist is not an error
        local missing = tostring(err):find("module '" .. name .. "' not found", 1, true)
        if not (optional and missing) then
            log_error(name, err)
        end
    end
end

-- Helpers (HOME, is_file_exists, first_available) that the files below use --
load("hyprland.lib")

-- Variables first: env, execs and keybinds all read them.
load("hyprland.variables")
load("custom.variables", true)

-- Environment --
load("hyprland.env")
load("custom.env", true)

-- Defaults --
load("hyprland.execs")
load("hyprland.general")
load("hyprland.rules")
load("hyprland.keybinds")

-- Yours --
load("custom.execs", true)
load("custom.general", true)
load("custom.rules", true)
load("custom.keybinds", true)

-- Workspaces and monitors (monitors.lua is generated on install, see install.sh) --
load("workspaces", true)
load("monitors", true)

-- The shell's own overrides. LAST, so what you set in the shell's Settings wins.
load("sekaipop_overrides", true)
