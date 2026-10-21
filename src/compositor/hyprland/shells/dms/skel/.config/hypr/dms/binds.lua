-- Key bindings. DMS rewrites this file from its keybind editor, so a change
-- made there wins over the same key defined here.
--
-- Lid switch and GloView live in rakuos.lua instead: they must survive DMS
-- rewriting this file. Default app variables come from rakuos.lua too, which is
-- required first.

local M = "SUPER"
local launchPrefix = "uwsm app -- "

-- Core
hl.bind(M .. " + SHIFT + R", hl.dsp.exec_cmd("hyprctl reload && notify-send -u low 'Hyprland reloaded'"), { description = "Reload Hyprland config" })
hl.bind(M .. " + Q", hl.dsp.window.close(), { description = "Close active window" })
hl.bind(M .. " + Escape", hl.dsp.exec_cmd("dms ipc call powermenu toggle"), { description = "Session menu" })
hl.bind(M .. " + CTRL + L", hl.dsp.exec_cmd("dms ipc call lock lock"), { description = "Lock screen" })
hl.bind("CTRL + SHIFT + Escape", hl.dsp.exec_cmd(launchPrefix .. TERMINAL .. " --title=btop-monitor -e btop"), { description = "System monitor (btop, floating)" })

-- DMS shell
hl.bind(M .. " + Space", hl.dsp.exec_cmd("dms ipc call spotlight toggle"), { description = "App launcher" })
hl.bind(M .. " + ALT + Space", hl.dsp.exec_cmd("dms ipc call control-center toggle"), { description = "Control center" })
hl.bind(M .. " + CTRL + Space", hl.dsp.exec_cmd("dms ipc call settings toggle"), { description = "Settings toggle" })
hl.bind(M .. " + CTRL + W", hl.dsp.exec_cmd("dms ipc call dash toggle wallpaper"), { description = "Wallpaper picker" })
hl.bind(M .. " + CTRL + period", hl.dsp.exec_cmd("dms ipc call notifications clearAll"), { description = "Clear notifications" })
hl.bind(M .. " + CTRL + comma", hl.dsp.exec_cmd("dms clipboard clear"), { description = "Clear clipboard" })
hl.bind(M .. " + CTRL + C", hl.dsp.exec_cmd("dms ipc call inhibit toggle"), { description = "Toggle caffeine" })
hl.bind(M .. " + ALT + P", hl.dsp.exec_cmd("hyprpicker -a -n"), { description = "Color picker (hyprpicker)" })

-- DMS surfaces
hl.bind(M .. " + V", hl.dsp.exec_cmd("dms ipc call clipboard toggle"), { description = "Clipboard history" })
hl.bind(M .. " + M", hl.dsp.exec_cmd("dms ipc call dash toggle media"), { description = "Media dash" })
hl.bind(M .. " + comma", hl.dsp.exec_cmd("dms ipc call settings focusOrToggle"), { description = "DMS settings" })
hl.bind(M .. " + SHIFT + N", hl.dsp.exec_cmd("dms ipc call notepad toggleExpand"), { description = "Notepad" })
hl.bind(M .. " + CTRL + N", hl.dsp.exec_cmd("dms ipc call notifications toggle"), { description = "Notification center" })
hl.bind(M .. " + D", hl.dsp.exec_cmd([[dms ipc call dash toggle ""]]), { description = "Dash" })
hl.bind(M .. " + Y", hl.dsp.exec_cmd("dms ipc call dash toggle wallpaper"), { description = "Wallpaper picker (DMS)" })
hl.bind(M .. " + A", hl.dsp.exec_cmd("dms ipc call control-center toggle"), { description = "Control center (DMS)" })
hl.bind(M .. " + O", hl.dsp.exec_cmd("dms ipc call hypr toggleOverview"), { description = "DMS overview" })
hl.bind(M .. " + SHIFT + C", hl.dsp.exec_cmd("dms ipc call color-picker toggle"), { description = "Color picker (DMS)" })
hl.bind(M .. " + ALT + N", hl.dsp.exec_cmd("dms ipc call night toggle"), { description = "Night light" })
hl.bind(M .. " + ALT + I", hl.dsp.exec_cmd("dms ipc call inhibit toggle"), { description = "Toggle caffeine" })
hl.bind(M .. " + SHIFT + Slash", hl.dsp.exec_cmd("dms ipc call keybinds toggle hyprland"), { description = "Keybind cheat sheet" })
hl.bind("CTRL + ALT + Delete", hl.dsp.exec_cmd("dms ipc call processlist focusOrToggle"), { description = "Process list" })
hl.bind("CTRL + ALT + M", hl.dsp.exec_cmd("dms ipc call processlist focusOrToggle"), { description = "Process list" })

-- Window focus and movement (scrolling layout)
hl.bind(M .. " + left",  hl.dsp.focus({ direction = "l" }), { description = "Focus left" })
hl.bind(M .. " + right", hl.dsp.focus({ direction = "r" }), { description = "Focus right" })
hl.bind(M .. " + up",    hl.dsp.focus({ direction = "u" }), { description = "Focus up" })
hl.bind(M .. " + down",  hl.dsp.focus({ direction = "d" }), { description = "Focus down" })
hl.bind(M .. " + H", hl.dsp.focus({ direction = "l" }), { description = "Focus left (HJKL)" })
hl.bind(M .. " + J", hl.dsp.focus({ direction = "d" }), { description = "Focus down (HJKL)" })
hl.bind(M .. " + K", hl.dsp.focus({ direction = "u" }), { description = "Focus up (HJKL)" })
hl.bind(M .. " + L", hl.dsp.focus({ direction = "r" }), { description = "Focus right (HJKL)" })

hl.bind(M .. " + SHIFT + left",  hl.dsp.window.move({ direction = "l" }), { description = "Move window left" })
hl.bind(M .. " + SHIFT + right", hl.dsp.window.move({ direction = "r" }), { description = "Move window right" })
hl.bind(M .. " + SHIFT + up",    hl.dsp.window.move({ direction = "u" }), { description = "Move window up" })
hl.bind(M .. " + SHIFT + down",  hl.dsp.window.move({ direction = "d" }), { description = "Move window down" })
hl.bind(M .. " + SHIFT + H", hl.dsp.window.move({ direction = "l" }), { description = "Move window left (HJKL)" })
hl.bind(M .. " + SHIFT + J", hl.dsp.window.move({ direction = "d" }), { description = "Move window down (HJKL)" })
hl.bind(M .. " + SHIFT + K", hl.dsp.window.move({ direction = "u" }), { description = "Move window up (HJKL)" })
hl.bind(M .. " + SHIFT + L", hl.dsp.window.move({ direction = "r" }), { description = "Move window right (HJKL)" })

hl.bind(M .. " + CTRL + up",   hl.dsp.focus({ workspace = "e-1" }), { description = "Previous workspace" })
hl.bind(M .. " + CTRL + down", hl.dsp.focus({ workspace = "e+1" }), { description = "Next workspace" })

-- Window states
hl.bind(M .. " + F", hl.dsp.window.fullscreen({ mode = "fullscreen", action = "toggle" }), { description = "Toggle fullscreen" })
hl.bind(M .. " + SHIFT + F", hl.dsp.window.fullscreen({ mode = "maximized", action = "toggle" }), { description = "Toggle maximize window" })
hl.bind(M .. " + SHIFT + T", hl.dsp.window.float({ action = "toggle" }), { description = "Toggle floating" })
hl.bind(M .. " + CTRL + T", function ()
    hl.dispatch(hl.dsp.window.float({ action = "toggle" }))
    hl.dispatch(hl.dsp.window.pin())
end, { description = "Toggle floating + pinned" })

-- Layout controls
hl.bind("ALT + Tab", hl.dsp.window.cycle_next({ tiled = true }), { description = "Cycle windows" })
hl.bind(M .. " + CTRL + K", hl.dsp.layout("swapsplit"), { description = "(Dwindle) Swap split" })
hl.bind(M .. " + CTRL + J", hl.dsp.layout("togglesplit"), { description = "(Dwindle) Toggle split" })
hl.bind(M .. " + CTRL + M", hl.dsp.layout("orientationnext"), { description = "(Master) Cycle master orientation" })
hl.bind(M .. " + R", hl.dsp.layout("togglesplit"), { description = "Toggle split" })
hl.bind(M .. " + W", hl.dsp.group.toggle(), { description = "Toggle window group" })
hl.bind(M .. " + SHIFT + W", hl.dsp.exec_cmd("dms ipc call window-rules toggle"), { description = "Window rules" })

-- Window grouping
hl.bind(M .. " + SHIFT + G", hl.dsp.group.toggle(), { description = "Toggle window group" })
hl.bind(M .. " + CTRL + G", hl.dsp.window.move({ out_of_group = true }), { description = "Out of group" })
hl.bind(M .. " + CTRL + Bracketleft", hl.dsp.window.move({ into_group = "l" }), { description = "Into group left" })
hl.bind(M .. " + CTRL + Bracketright", hl.dsp.window.move({ into_group = "r" }), { description = "Into group right" })
hl.bind(M .. " + ALT + Bracketright", hl.dsp.window.move({ into_group = "u" }), { description = "Into group up" })
hl.bind(M .. " + ALT + Bracketleft", hl.dsp.window.move({ into_group = "d" }), { description = "Into group down" })
hl.bind(M .. " + Tab", hl.dsp.group.next(), { description = "Group next" })
hl.bind(M .. " + SHIFT + Tab", hl.dsp.group.prev(), { description = "Group prev" })
for i = 1, 9 do
    hl.bind(M .. " + CTRL + " .. i, hl.dsp.group.active({ index = i }), { description = "Group index " .. i })
end

-- Scratchpad
hl.bind(M .. " + S", hl.dsp.workspace.toggle_special("magic"), { description = "Toggle special workspace magic" })
hl.bind(M .. " + SHIFT + S", hl.dsp.window.move({ workspace = "special:magic" }), { description = "Send window to special workspace" })
hl.bind(M .. " + SHIFT + CTRL + S", hl.dsp.window.move({ workspace = "previous" }), { description = "Move window out of special workspace" })

-- Media keys
hl.bind("XF86AudioRaiseVolume", hl.dsp.exec_cmd("dms ipc call audio increment 3"), { locked = true, repeating = true, description = "Volume up" })
hl.bind("XF86AudioLowerVolume", hl.dsp.exec_cmd("dms ipc call audio decrement 3"), { locked = true, repeating = true, description = "Volume down" })
hl.bind("XF86AudioMute", hl.dsp.exec_cmd("dms ipc call audio mute"), { locked = true, description = "Mute audio" })
hl.bind("XF86AudioMicMute", hl.dsp.exec_cmd("dms ipc call audio micmute"), { locked = true, description = "Mute mic" })
hl.bind("XF86AudioPlay", hl.dsp.exec_cmd("dms ipc call mpris playPause"), { locked = true, description = "Play/pause" })
hl.bind("XF86AudioNext", hl.dsp.exec_cmd("dms ipc call mpris next"), { locked = true, description = "Next track" })
hl.bind("XF86AudioPrev", hl.dsp.exec_cmd("dms ipc call mpris previous"), { locked = true, description = "Previous track" })
hl.bind("XF86AudioPause", hl.dsp.exec_cmd("dms ipc call mpris playPause"), { locked = true, description = "Play/pause" })
hl.bind("CTRL + XF86AudioRaiseVolume", hl.dsp.exec_cmd("dms ipc call mpris increment 3"), { locked = true, repeating = true, description = "Next track volume" })
hl.bind("CTRL + XF86AudioLowerVolume", hl.dsp.exec_cmd("dms ipc call mpris decrement 3"), { locked = true, repeating = true, description = "Previous track volume" })
hl.bind("XF86MonBrightnessUp", hl.dsp.exec_cmd([[dms ipc call brightness increment 5 ""]]), { locked = true, repeating = true, description = "Brightness up" })
hl.bind("XF86MonBrightnessDown", hl.dsp.exec_cmd([[dms ipc call brightness decrement 5 ""]]), { locked = true, repeating = true, description = "Brightness down" })
hl.bind("XF86Sleep", hl.dsp.exec_cmd("dms ipc call lock lockAndOutputsOff; systemctl suspend"), { description = "Lock then suspend" })
hl.bind("XF86Calculator", hl.dsp.exec_cmd(launchPrefix .. CALCULATOR), { locked = true, description = "Calculator" })

-- Workspaces
for i = 1, 9 do
    hl.bind(M .. " + " .. i, hl.dsp.focus({ workspace = i }), { description = "Workspace " .. i })
    hl.bind(M .. " + SHIFT + " .. i, hl.dsp.window.move({ workspace = i }), { description = "Move to workspace " .. i })
end
hl.bind(M .. " + 0", hl.dsp.focus({ workspace = "name:gaming" }), { description = "Workspace gaming" })
hl.bind(M .. " + SHIFT + 0", hl.dsp.window.move({ workspace = "name:gaming" }), { description = "Move window to gaming workspace" })
hl.bind(M .. " + U", hl.dsp.focus({ workspace = "e+1" }), { description = "Next workspace (U)" })
hl.bind(M .. " + I", hl.dsp.focus({ workspace = "e-1" }), { description = "Previous workspace (I)" })
hl.bind(M .. " + CTRL + U", hl.dsp.window.move({ workspace = "e+1" }), { description = "Move window to next workspace" })
hl.bind(M .. " + CTRL + I", hl.dsp.window.move({ workspace = "e-1" }), { description = "Move window to previous workspace" })
hl.bind(M .. " + ALT + R", hl.dsp.exec_cmd("dms ipc call workspace-rename open"), { description = "Rename workspace" })

-- Multi-monitor
hl.bind(M .. " + CTRL + ALT + Tab", hl.dsp.window.cycle_next(), { description = "Cycle windows" })
hl.bind(M .. " + CTRL + ALT + left", hl.dsp.focus({ monitor = "l" }), { description = "Focus left monitor" })
hl.bind(M .. " + CTRL + ALT + right", hl.dsp.focus({ monitor = "r" }), { description = "Focus right monitor" })
hl.bind(M .. " + CTRL + ALT + up", hl.dsp.focus({ monitor = "u" }), { description = "Focus top monitor" })
hl.bind(M .. " + CTRL + ALT + down", hl.dsp.focus({ monitor = "d" }), { description = "Focus bottom monitor" })
hl.bind(M .. " + CTRL + SHIFT + left", hl.dsp.window.move({ monitor = "l" }), { description = "Move window left monitor" })
hl.bind(M .. " + CTRL + SHIFT + right", hl.dsp.window.move({ monitor = "r" }), { description = "Move window right monitor" })
hl.bind(M .. " + CTRL + SHIFT + up", hl.dsp.window.move({ monitor = "u" }), { description = "Move window top monitor" })
hl.bind(M .. " + CTRL + SHIFT + down", hl.dsp.window.move({ monitor = "d" }), { description = "Move window bottom monitor" })

-- Move floating windows
hl.bind("CTRL + SHIFT + up", hl.dsp.window.move({ x = 0, y = -50, relative = true }), { repeating = true, description = "Move window up" })
hl.bind("CTRL + SHIFT + down", hl.dsp.window.move({ x = 0, y = 50, relative = true }), { repeating = true, description = "Move window down" })
hl.bind("CTRL + SHIFT + left", hl.dsp.window.move({ x = -50, y = 0, relative = true }), { repeating = true, description = "Move window left" })
hl.bind("CTRL + SHIFT + right", hl.dsp.window.move({ x = 50, y = 0, relative = true }), { repeating = true, description = "Move window right" })

-- App launchers
hl.bind(M .. " + Return", hl.dsp.exec_cmd(launchPrefix .. TERMINAL), { description = "Terminal" })
hl.bind(M .. " + T", hl.dsp.exec_cmd(launchPrefix .. TERMINAL), { description = "Terminal" })
hl.bind(M .. " + E", hl.dsp.exec_cmd(launchPrefix .. FILE_MANAGER), { description = "File manager" })
hl.bind(M .. " + B", hl.dsp.exec_cmd(launchPrefix .. BROWSER), { description = "Browser" })
hl.bind(M .. " + N", hl.dsp.exec_cmd(launchPrefix .. EDITOR), { description = "Editor" })

-- Screenshots (DMS owns the screenshot UI now)
hl.bind("Print", hl.dsp.exec_cmd("dms screenshot"), { description = "Screenshot: region" })
hl.bind("CTRL + Print", hl.dsp.exec_cmd("dms screenshot full"), { description = "Screenshot: fullscreen" })
hl.bind("ALT + Print", hl.dsp.exec_cmd("dms screenshot window"), { description = "Screenshot: active window" })

-- Display
hl.bind(M .. " + P", hl.dsp.exec_cmd("dms ipc outputs cycleProfile"), { description = "Cycle display profile" })
hl.bind(M .. " + SHIFT + P", hl.dsp.dpms({ action = "toggle" }), { description = "Toggle display power" })

-- Mouse bindings
hl.bind(M .. " + mouse:272", hl.dsp.window.drag(), { mouse = true, description = "Move window with mouse" })
hl.bind(M .. " + mouse:273", hl.dsp.window.resize(), { mouse = true, description = "Resize window with mouse" })
hl.bind("mouse:274", hl.dsp.window.fullscreen({ mode = "maximized", action = "toggle" }), { mouse = true, description = "Toggle maximize (middle click)" })

-- Scroll through workspaces
hl.bind(M .. " + mouse_up", hl.dsp.focus({ workspace = "e-1" }), { description = "Previous workspace (scroll)" })
hl.bind(M .. " + mouse_down", hl.dsp.focus({ workspace = "e+1" }), { description = "Next workspace (scroll)" })
hl.bind(M .. " + ALT + mouse_up", hl.dsp.window.move({ workspace = "e-1" }), { description = "Move window to previous workspace (scroll)" })
hl.bind(M .. " + ALT + mouse_down", hl.dsp.window.move({ workspace = "e+1" }), { description = "Move window to next workspace (scroll)" })
hl.bind(M .. " + CTRL + mouse_up", hl.dsp.layout("focus l"), { description = "Focus left (scrolling)" })
hl.bind(M .. " + CTRL + mouse_down", hl.dsp.layout("focus r"), { description = "Focus right (scrolling)" })

-- Zoom cursor (HiDPI assist)
local function zoomfunction(value)
    local zoomvalue = hl.get_config("cursor:zoom_factor")
    if (zoomvalue + value) > 3.0 then
        hl.config({ cursor = { zoom_factor = 3.0 } })
    elseif (zoomvalue + value) < 1.0 then
        hl.config({ cursor = { zoom_factor = 1.0 } })
    else
        hl.config({ cursor = { zoom_factor = zoomvalue + value } })
    end
end
hl.bind(M .. " + Minus", function () zoomfunction(-0.3) end, { repeating = true, description = "Zoom out cursor" })
hl.bind(M .. " + Plus", function () zoomfunction(0.3) end, { repeating = true, description = "Zoom in cursor" })
hl.bind(M .. " + code:82", function () zoomfunction(-0.3) end, { repeating = true, description = "Zoom out cursor (keypad)" })
hl.bind(M .. " + code:86", function () zoomfunction(0.3) end, { repeating = true, description = "Zoom in cursor (keypad)" })
