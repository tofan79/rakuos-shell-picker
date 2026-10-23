local home_dir = os.getenv("HOME")

-- Wayland
hl.env("ELECTRON_OZONE_PLATFORM_HINT", "auto")

-- Applications
local xdg_data_dirs_old = os.getenv("XDG_DATA_DIRS") or ""
hl.env("XDG_DATA_DIRS", home_dir .. "/.local/share/flatpak/exports/share:/var/lib/flatpak/exports/share:/usr/local/share:/usr/share:" .. xdg_data_dirs_old)

-- Qt theming. `kde` is what Settings > Desktop > System colours needs for Dolphin and other KDE
-- apps to follow the shell's colours (needs plasma-integration). qt5ct / qt6ct also work for
-- plain Qt apps; the shell's `system-theme.sh check` says which one you have.
hl.env("QT_QPA_PLATFORM", "wayland;xcb")
hl.env("QT_QPA_PLATFORMTHEME", "kde")
hl.env("XDG_MENU_PREFIX", "plasma-")

-- The shell runs $TERMINAL before its own list of terminals.
if terminalName and terminalName ~= "" and not os.getenv("TERMINAL") then
    hl.env("TERMINAL", terminalName)
end

-- File chooser for the avatar / banner picker: kdialog, zenity or yad. Unset = the shell picks.
-- hl.env("SEKAIPOP_CHOOSER", "kdialog")
