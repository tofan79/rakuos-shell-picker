-- Auto-start config
--
-- Impasto upstream starts `qs -d` from modules/autostart.lua, which expects the
-- shell at the root of ~/.config/quickshell. This package installs it one level
-- down (the skel symlinks ~/.config/quickshell/impasto at it), so the shell is
-- started by config name here — the same argv a switch uses.
--
-- The plugins block upstream's autostart carries is left out on purpose: the two
-- hyprpm plugins are not in the package (they are built per compositor), so the
-- block would only offer a `./setup plugins` that does not exist here.
hl.on("hyprland.start", function()
    hl.exec_cmd("qs -c impasto")
    -- Wallpaper daemon: the shell derives the desk palette from it. Without it
    -- the shell still draws, it just has no new palette to draw with.
    hl.exec_cmd("awww-daemon")
end)
