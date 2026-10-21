-- RakuOS settings that DMS must not rewrite.
--
-- DMS generates layout.lua, input.lua, outputs.lua and windowrules.lua from its
-- own Settings. Anything the desktop needs regardless of what DMS writes lives
-- here instead, and hyprland.lua requires this file first.

-- ───────────────────────────────────────────
-- Startup
-- ───────────────────────────────────────────
hl.on("hyprland.start", function ()
    hl.exec_cmd("dbus-update-activation-environment --systemd --all")

    -- DMS and dsearch come from their systemd units: the picker unmasks,
    -- enables and starts both when DMS is the chosen shell. dcal is different:
    -- the calendar widget connects to it on startup, and it is started for the
    -- session rather than enabled.
    hl.exec_cmd("systemctl --user start dcal.service")

    hl.exec_cmd("xhost +SI:localuser:root")

    -- Clipboard history that DMS's clipboard panel reads.
    hl.exec_cmd("wl-paste --watch cliphist store")
end)

hl.plugin.load("/usr/lib64/gloview.so")

-- ───────────────────────────────────────────
-- Default apps (used by dms/binds.lua)
-- ───────────────────────────────────────────
TERMINAL      = "ghostty"
FILE_MANAGER  = "nautilus"
BROWSER       = "firefox"
EDITOR        = "zed"
CALCULATOR    = "gnome-calculator"

MUSIC_PLAYER  = "mpv"
VIDEO_PLAYER  = "mpv"
IMAGE_VIEWER  = "nomacs"

-- ───────────────────────────────────────────
-- Monitors (used by dms/outputs.lua and the rules below)
-- ───────────────────────────────────────────
MONITOR1 = ""
MONITOR2 = ""
MONITOR3 = ""
PRIMARY_MONITOR = MONITOR1

-- ───────────────────────────────────────────
-- Behaviour
-- ───────────────────────────────────────────
hl.config({
    dwindle = {
        preserve_split = true,
        force_split = 2,
        smart_split = false,
        smart_resizing = true,
    },
    master = {
        mfact = 0.60,
        new_status = "master",
        smart_resizing = true,
    },
    scrolling = {
        fullscreen_on_one_column = false,
    },
    ecosystem = {
        no_update_news = true,
        no_donation_nag = true,
    },
    misc = {
        disable_splash_rendering = true,
        middle_click_paste = false,
        enable_swallow = true,
        swallow_regex = "(ghostty|[Kk]onsole|Alacritty|gnome-terminal|xfce[0-9]?-terminal)",
        vrr = -1,
        focus_on_activate = true,
        allow_session_lock_restore = true,
        disable_scale_notification = true,
        disable_hyprland_logo = true,
        force_default_wallpaper = 0,
    },
    render = {
        direct_scanout = 2,
    },
    xwayland = {
        force_zero_scaling = true,
    },
})

-- DMS has no setting for the layout type, so it stays here rather than in
-- dms/layout.lua, which DMS rewrites.
hl.config({ general = { layout = "scrolling" } })

-- ───────────────────────────────────────────
-- Animations
-- ───────────────────────────────────────────
hl.curve("my_linear_curve", { type = "bezier", points = { { 0, 0 }, { 1, 1 } } })
hl.animation({ leaf = "border", enabled = true, speed = 10, bezier = "default" })
hl.animation({ leaf = "borderangle", enabled = false })
hl.animation({ leaf = "borderangle", enabled = true, speed = 20, bezier = "my_linear_curve", style = "loop" })

-- ───────────────────────────────────────────
-- Workspaces
-- ───────────────────────────────────────────
-- Named gaming workspace. No `default = true` here: with it, Hyprland treats the
-- named workspace as a low-id default and lands the session there on login instead
-- of workspace 1. Window rules still route matching apps here.
hl.workspace_rule({ workspace = "name:gaming", monitor = PRIMARY_MONITOR })
hl.workspace_rule({ workspace = "1", monitor = MONITOR1, default = true, persistent = true })
hl.workspace_rule({ workspace = "2", monitor = MONITOR1, default = true, persistent = true })
hl.workspace_rule({ workspace = "3", monitor = MONITOR1, default = true, persistent = true })
hl.workspace_rule({ workspace = "4", monitor = MONITOR1, default = true, persistent = true })
hl.workspace_rule({ workspace = "5", monitor = MONITOR1, default = true, persistent = true })
hl.workspace_rule({ workspace = "6", monitor = MONITOR2, default = true, persistent = true })
hl.workspace_rule({ workspace = "7", monitor = MONITOR2, default = true, persistent = true })
hl.workspace_rule({ workspace = "8", monitor = MONITOR2, default = true, persistent = true })
hl.workspace_rule({ workspace = "9", monitor = MONITOR2, default = true, persistent = true })

-- ───────────────────────────────────────────
-- Window rules
-- ───────────────────────────────────────────
hl.window_rule({ match = { float = true }, center = true, persistent_size = true })

hl.window_rule({
    match = { title = "^btop-monitor$" },
    float = true,
    center = true,
    size = { 1000, 700 },
})

hl.window_rule({
    match             = { title = "^([Pp]icture[-\\s]?[Ii]n[-\\s]?[Pp]icture)(.*)$" },
    float             = true,
    keep_aspect_ratio = true,
    size              = { "max(monitor_w, monitor_h)*0.25", "min(monitor_w, monitor_h)*0.25" },
    pin               = true,
})

local gamingApps = "^(steam_app.*|gamescope)$"
local gamingWorkspace = "name:gaming"

hl.window_rule({ match = { content = "game" }, workspace = gamingWorkspace })
hl.window_rule({ match = { xdg_tag = "^(.*game.*)$" }, workspace = gamingWorkspace, fullscreen_state = 2, content = "game", sync_fullscreen = true })
hl.window_rule({ match = { class = gamingApps }, workspace = gamingWorkspace })
hl.window_rule({ match = { class = "^(steam)$", title = "^(Friends List)$" }, float = true })
hl.window_rule({ match = { class = "^(steam)$", title = "^(Launching\\.{3})$" }, float = true, center = true, workspace = gamingWorkspace })
hl.window_rule({
    match = {
        class         = gamingApps,
        title         = "^(.+)$",
        initial_title = "negative:^(.*\\\\home\\\\.*)$",
    },
    content          = "game",
    decorate         = false,
    fullscreen_state = 2,
    size             = { "monitor_w", "monitor_h" },
    sync_fullscreen  = true,
})
hl.window_rule({
    match = {
        class         = "^(steam_app.*)$",
        initial_title = "^$",
    },
    center           = true,
    float            = true,
    fullscreen       = false,
    fullscreen_state = 0,
    workspace        = gamingWorkspace,
})

hl.window_rule({ match = { class = "^(.*\\.exe)$", float = true }, monitor = PRIMARY_MONITOR, center = true, fullscreen_state = 0 })
hl.window_rule({ match = { class = "^(.*[Ll]auncher.*)$" }, float = true, monitor = PRIMARY_MONITOR })
hl.window_rule({ match = { class = "^(vesktop|discord)$" }, monitor = PRIMARY_MONITOR })
hl.window_rule({ match = { class = "^(.*[Cc]alc.*)$" }, float = true, size = { "max(monitor_w, monitor_h)*0.17", "min(monitor_w, monitor_h)*0.43" } })
hl.window_rule({ match = { class = "^(org\\.kde\\.keditfiletype)$" }, float = true })
hl.window_rule({ match = { class = "^(org\\.kde\\.ark)$" }, size = { "max(monitor_w, monitor_h)*0.40", "min(monitor_w, monitor_h)*0.40" } })
hl.window_rule({ match = { class = "^(.*satty.*)$", title = "^(Satty)$" }, min_size = { "max(monitor_w, monitor_h)*0.35", "min(monitor_w, monitor_h)*0.35" }, float = true })

-- Shell picker: fixed square window (GUI_LAYOUT.md ratio ~1:1, ~800x800).
-- Must float, because a tiled WM would force a column size and break the ratio.
hl.window_rule({ match = { class = "^(org\\.rakuos\\.ShellPicker)$" }, float = true, size = { "806", "806" } })

-- Full opacity for video players and image viewers so the content looks right.
hl.window_rule({ match = { class = "^(mpv|.*plex.*|.*vlc.*|loupe|org.gnome.Loupe)$" }, opacity = "1.0 override" })

local floatApps = {
    { class = "^(kvantummanager|qt[56]ct|nwg-look)$" },
    { class = "^(org.pulseaudio.pavucontrol|blueman-manager|nm-applet|nm-connection-editor)$" },
    { title = "^(Winetricks.*|Protontricks.*)$" },
}
for _, m in ipairs(floatApps) do hl.window_rule({ match = m, float = true }) end

local modalMatches = {
    { title = "^(Open|Authentication Required|Add Folder to Workspace|Choose Files|Save As|Confirm to replace files|File Operation Progress)$" },
    { initial_title = "^(Open File)$" },
    { class = "^([Xx]dg-desktop-portal-gtk)$" },
    { title = "^(File Upload|Choose wallpaper|Library)(.*)$" },
    { class = "^(.*dialog.*)$" },
    { title = "^(.*dialog.*)$" },
    { class = "^(hyprland-share-picker)$"},
}
for _, m in ipairs(modalMatches) do hl.window_rule({ match = m, float = true }) end

hl.window_rule({
    name  = "suppress-maximize-events",
    match = { class = ".*" },
    suppress_event = "maximize",
})

hl.window_rule({
    name  = "fix-xwayland-drags",
    match = {
        class      = "^$",
        title      = "^$",
        xwayland   = true,
        float      = true,
        fullscreen = false,
        pin        = false,
    },
    no_focus = true,
})

hl.layer_rule({ match = { namespace = "hyprpicker" }, animation = "fade" })
hl.layer_rule({ match = { namespace = "selection" }, animation = "fade" })

-- DMS surfaces: bar, panels, notification, dock and the rest of the shell.
hl.layer_rule({
    name = "dms",
    match = {
        namespace = "^dms:(bar|osd|dankisland|dock|control-center|dash|spotlight|power-menu|app-launcher|notification-popup|notification-center-popout|notification-center-modal)(:.+)?$",
    },
    ignore_alpha = 0.5,
    blur = true,
    blur_popups = true,
})

hl.window_rule({
    name  = "btop-float",
    match = { title = "^btop$" },
    float = true,
    size  = { 1200, 700 },
})

hl.window_rule({
    name  = "localsend-float",
    match = { class = "^localsend$" },
    float = true,
    size  = { 800, 600 },
})
hl.window_rule({
    name  = "localsend-org-float",
    match = { class = "^org\\.localsend\\.localsend_app$" },
    float = true,
    size  = { 800, 600 },
})

hl.window_rule({
    name  = "calc-gnome-float",
    match = { class = "^org\\.gnome\\.Calculator$" },
    float = true,
    size  = { 400, 500 },
})

hl.window_rule({
    name   = "loupe-float",
    match  = { class = "^org\\.gnome\\.Loupe$" },
    float  = true,
    size   = { 900, 700 },
    opaque = true,
})
hl.window_rule({
    name   = "zoom",
    match  = { class = "^zoom$" },
    float  = true,
    opaque = true,
})

-- Port aturan DMS dari skel niri (windowrules.kdl). Aturan ini tinggal di
-- rakuos.lua, bukan dms/windowrules.lua, karena DMS menulis ulang file itu.
-- Catatan: Spotify di sini di-maximize (mengikuti DMS/niri), jadi aturan
-- float-spotify yang lama dihapus supaya tidak saling menimpa.
hl.window_rule({
    name    = "dms-maximized",
    match   = { class = "^(Spotify|spotify|vesktop|GDevelop 5|discord|Godot|Pixelorama)$" },
    maximize = true,
})

hl.window_rule({
    name  = "dms-floating",
    match = { class = "^(dev\\.duxlabs\\.shell|com\\.danklinux\\.dms|org\\.quickshell|gsr-ui|org\\.rakuos\\.Installer\\.Gtk)$" },
    float = true,
})

hl.window_rule({
    name  = "dms-gnome-weather",
    match = { class = "^org\\.gnome\\.Weather$" },
    float = true,
    size  = { "838", "498" },
    move  = "100%-838-32 32",
})

hl.window_rule({
    name    = "dms-window-opaque",
    match   = { class = "^(dev\\.duxlabs\\.shell|com\\.danklinux\\.dms)$" },
    opacity = "1.0 override",
})

hl.window_rule({
    name   = "media-editors-opaque",
    match  = { class = "^(krita|gimp|inkscape|darktable|resolve|kdenlive|shotcut|blender|godot)$" },
    opaque = true,
})

hl.window_rule({
    name  = "file-roller-float",
    match = { class = "org\\.gnome\\.FileRoller|file-roller" },
    float = true,
})

hl.window_rule({
    name  = "yad-zenity-float",
    match = { class = "^yad|zenity$" },
    float = true,
})

hl.window_rule({
    name  = "gnome-settings-float",
    match = { class = "^org\\.gnome\\.Settings$" },
    float = true,
    size  = { "(monitor_w*0.7)", "(monitor_h*0.8)" },
})

hl.window_rule({
    name  = "printer-float",
    match = { class = "^system-config-printer$" },
    float = true,
    size  = { "(monitor_w*0.5)", "(monitor_h*0.6)" },
})

-- ───────────────────────────────────────────
-- Lid switch
-- ───────────────────────────────────────────
-- Lock and blank, then suspend. The suspend is separated with ";", so a failing
-- lock cannot keep the machine awake with the lid shut.
hl.bind("switch:on:Lid Switch", hl.dsp.exec_cmd("dms ipc call lock lockAndOutputsOff; systemctl suspend"), { locked = true })
hl.bind("switch:off:Lid Switch", hl.dsp.exec_cmd(""), { locked = true })

-- ───────────────────────────────────────────
-- GloView (Mission Control overview)
-- ───────────────────────────────────────────
-- Opt-in: GloView is not installed by default. Run "rum install gloview-git"
-- (the package is in the rakuos repository), then uncomment the matching
-- hl.plugin.load("/usr/lib64/gloview.so") line below. The guard registers
-- nothing until the plugin is loaded.
if hl.plugin.gloview then
    hl.bind("SUPER + SHIFT + O", hl.plugin.gloview.toggle, { description = "Toggle overview" })
    hl.bind("SUPER + ALT + O", hl.plugin.gloview.desktop, { description = "Desktop mode" })
    hl.bind("SUPER + CTRL + O", hl.plugin.gloview.allworkspaces, { description = "All workspaces" })
end
-- hl.plugin.load("/usr/lib64/gloview.so")
