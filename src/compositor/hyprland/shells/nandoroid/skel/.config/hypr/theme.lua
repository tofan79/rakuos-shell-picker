-- ─────────────────────────────────────────────────────────────────────────────
--  Nandoroid Shell: Theme & Environment Configuration
--  This file provides the necessary environment variables for Qt/GTK apps
--  to follow the shell's dynamic colors.
-- ─────────────────────────────────────────────────────────────────────────────

-- --- Theme Engines ---
-- `kde` is what upstream's copy asks for, but the KDE platform theme is a
-- plugin from `plasma-integration` and it is not installed here (the only
-- plugins in /usr/lib64/qt6/plugins/platformthemes are qt6ct, gtk3 and the
-- portal). When the plugin named by this variable is missing, Qt quietly falls
-- back to the default theme, which carries no icon theme — the Colloid icons
-- every other shell on this machine uses then come out as hicolor. `qt6ct` is
-- the engine that is installed, and its `icon_theme` is already pinned to
-- Colloid-Dark, so it is also what `~/.config/uwsm/env` sets.
hl.env("QT_QPA_PLATFORMTHEME", "qt6ct")
hl.env("QT_QPA_PLATFORM", "wayland;xcb")
-- No `XDG_MENU_PREFIX = "plasma-"`: it makes the menu specification look for
-- `plasma-applications.menu`, which only exists with Plasma installed.

-- --- Toolkit Backends ---
hl.env("GDK_BACKEND", "wayland,x11,*")
hl.env("SDL_VIDEODRIVER", "wayland")
hl.env("CLUTTER_BACKEND", "wayland")

-- --- XDG Desktop Portal ---
hl.env("XDG_CURRENT_DESKTOP", "Hyprland")
hl.env("XDG_SESSION_TYPE", "wayland")
hl.env("XDG_SESSION_DESKTOP", "Hyprland")

-- --- Initial Theming Hook ---
hl.on("hyprland.start", function()
    hl.exec_cmd("gsettings set org.gnome.desktop.interface gtk-theme 'adw-gtk3'")
    hl.exec_cmd("gsettings set org.gnome.desktop.interface color-scheme 'prefer-dark'")
    
    -- --- Dbus Activation Sync ---
    hl.exec_cmd("dbus-update-activation-environment --all")
    hl.exec_cmd("sleep 1 && dbus-update-activation-environment --systemd WAYLAND_DISPLAY XDG_CURRENT_DESKTOP")
end)