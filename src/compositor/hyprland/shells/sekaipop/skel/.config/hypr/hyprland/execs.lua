-- Startup. (Run once per Hyprland session, not on a config reload.)
--
-- The shell is its own polkit agent AND its own notification daemon, and it starts hyprsunset,
-- the clipboard watcher (cliphist), the video wallpaper, the cursor/theme restore and the
-- lockscreen itself. So none of those are started here, and no other polkit agent
-- (hyprpolkitagent, polkit-gnome, lxqt) or notification daemon (mako, dunst, swaync) may run.
hl.on("hyprland.start", function()

    -- Session environment, so portals, Discord and D-Bus-activated apps see WAYLAND_DISPLAY
    hl.exec_cmd("dbus-update-activation-environment --all")
    hl.exec_cmd("sleep 1 && dbus-update-activation-environment --systemd WAYLAND_DISPLAY XDG_CURRENT_DESKTOP")

    -- Secrets (Wi-Fi passwords, browsers, Discord)
    hl.exec_cmd("gnome-keyring-daemon --start --components=secrets")

    -- Fallback cursor (the shell puts your saved one back right after it starts)
    hl.exec_cmd("hyprctl setcursor " .. cursorTheme .. " " .. cursorSize)

    -- The shell: bar, dock, phone, lockscreen, notifications, polkit, wallpaper
    hl.exec_cmd(QS)

    -- Idle: dim/lock/sleep (the lock itself is the shell's: see hypridle.conf)
    hl.exec_cmd("hypridle")

    -- Audio effects service: the shell's Equalizer loads presets into it (`easyeffects -l`)
    hl.exec_cmd("easyeffects --hide-window --service-mode")
end)
