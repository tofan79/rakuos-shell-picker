-- Auto-start config
-- if you dont use UWSM add your auto start programs here, otherwise use XDG autostart https://wiki.archlinux.org/title/XDG_Autostart

hl.on("hyprland.start", function ()
    -- Noctalia is not started from here any more, and neither is DMS.
    --
    -- DMS and its search daemon come from their systemd units: the picker
    -- unmasks, enables and starts `dms.service` and `dsearch.service` when DMS
    -- is the chosen shell, so every login after that starts them on its own,
    -- and switching away stops, disables and masks them again. They must go
    -- through systemd either way — `dms.service` declares
    -- BusName=org.freedesktop.Notifications with Type=dbus and Restart=on-failure,
    -- so started outside systemd it would own no bus name and nothing would
    -- restart it.
    --
    -- dcal stays here: the calendar widget connects to it on startup, and it is
    -- started for the session rather than enabled.
    hl.exec_cmd("systemctl --user start dcal.service")

    hl.exec_cmd("xhost +SI:localuser:root")

    -- Clipboard history
    hl.exec_cmd("wl-paste --watch cliphist store")
end)
