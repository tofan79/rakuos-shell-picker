-- Monitor wiki https://wiki.hypr.land/Configuring/Basics/Monitors/
-- output can be found with hyprctl monitors. Edit variables.lua for the monitor
-- outputs instead of here directly.
--
-- Nothing here is pinned to a particular machine. output comes from
-- variables.lua (empty, so Hyprland picks whatever is connected), mode is
-- "preferred" and position "auto" so any panel is placed by the compositor.
--
-- scale is set to 1.25 as a default comfortable for typical laptop/desktop
-- panels. HiDPI users should raise it for their own output, e.g.
--
--     hl.monitor({ output = "eDP-1", scale = "2" })
--
-- Note that the skel is copied into a home the first time you log in on this
-- desktop environment. Your own copy in $HOME is what Hyprland reads from then
-- on, so editing it is what changes your session -- edits to /etc/skel only
-- reach accounts (or images) that have not been populated yet. See
-- ~/.config/hypr/docs.md.
hl.monitor({
    output    = MONITOR1,
    mode      = "preferred",
    position = "auto",
    scale     = "1.25",
})
