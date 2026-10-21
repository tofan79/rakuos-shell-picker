# Hyprland Configuration — Quick Guide

Reload after editing: `SUPER + SHIFT + R`

> **Important for existing users.** This folder is *copied* into your home the
> first time you log in on this desktop environment. From then on Hyprland reads
> the copy in your own home, so edit the files there — changes to `/etc/skel`
> only reach accounts that have not been populated yet, and are copied without
> overwriting anything you already have.

---

## Changing monitors?

**`config/variables.lua`**

```lua
MONITOR1 = "eDP-1"
MONITOR2 = "DP-1"
```

List the outputs available on your machine:

```bash
hyprctl monitors
```

Use the value from the `NAME` column. Leave it empty (`""`) if there is no second
monitor.

These variables are read by:

- `config/monitors.lua` — the output that gets configured
- `config/workspaces.lua` — workspaces 1-5 go to `MONITOR1`, 6-9 to `MONITOR2`
- `PRIMARY_MONITOR` — where launcher, Discord and `.exe` windows are opened

---

## Changing or adding default apps?

**`config/variables.lua`**

```lua
TERMINAL     = "ghostty"
FILE_MANAGER = "nautilus"
BROWSER      = "firefox"
EDITOR       = "zed"
CALCULATOR   = "gnome-calculator"
```

`binds.lua` reads these variables, so this is the only file you need to touch —
no need to edit the binds themselves. Session-aware launching goes through the
`launchPrefix` (`uwsm app -- `) defined in `binds.lua`.

---

## Where are the keyboard binds?

**`config/binds.lua`** — every `SUPER + ...` binding is here. `M` stands for
`SUPER`.

Example:

```lua
hl.bind(M .. " + Return", hl.dsp.exec_cmd(launchPrefix .. TERMINAL), { description = "Terminal" })
```

Shortcuts worth knowing:

| Shortcut | Action |
|---|---|
| `SUPER + Return` | Terminal |
| `SUPER + E` / `B` / `N` | File manager / Browser / Editor |
| `SUPER + Space` | App launcher |
| `SUPER + 1..9` | Switch workspace |
| `SUPER + SHIFT + 1..9` | Send window to workspace |
| `SUPER + 0` | `gaming` workspace |
| `SUPER + S` | Scratchpad |
| `SUPER + F` / `SHIFT + F` | Fullscreen / maximize |
| `SUPER + SHIFT + S` | Send window to scratchpad |
| `Print` / `SHIFT + Print` | Screenshot region / monitor (Noctalia) |
| `SUPER + ALT` + click-drag | Move window (mouse:272) |
| `SUPER + SHIFT + R` | Reload config |
| `SUPER + SHIFT + O` / `ALT + O` / `CTRL + O` | GloView: overview / desktop mode / all workspaces |

The scripts called by these binds live in `scripts/`:

| Script | Action |
|---|---|
| `focus-dir.sh` / `swap-dir.sh` | Directional focus/swap, adapting to the active layout |
| `gamemode.sh` | Toggle decorative effects (max FPS / restore) |

Screenshot and lock/suspend now use Noctalia directly: `noctalia msg screenshot-region`,
`screenshot-fullscreen`, and `session lock-and-suspend`. OCR, QR scan, and Google Lens
were removed from the default setup.

---

## Gaps, borders, blur, opacity

**`config/decorations.lua`**

```lua
gaps_in  = 5     -- spacing between windows
gaps_out = 10    -- spacing from window to screen edge
border_size = 2 -- border thickness
rounding = 12    -- corner rounding
active_opacity   = 0.9
inactive_opacity = 0.7
```

The `blur` group (`size`, `passes`) and `shadow` are in the same file. **`scale`
is not here** — that lives in `monitors.lua`.

---

## Layouts and animations

- **Layout** — `config/misc.lua`. The active layout is forced to `scrolling` on
  the last line. The `dwindle` / `master` / `swapsplit` values can be changed
  there.
- **Layout presets** — `layouts/fair.lua` (automatic grid), `layouts/deck.lua`.
  Reachable via `SUPER + ALT + W` (the Noctalia panel).
- **Animations** — `config/animations.lua`. Switch preset by changing one line:

  ```lua
  ANIMATION_PRESET = "bounce"   -- air, minimal, rage, smooth, ...
  ```

  The full list is in `config/animations_presets/`.

---

## Optional plugins (Noctalia)

Some settings are easier through a Noctalia plugin than by editing Lua. All of
them are optional — nothing here is required for a working session.

Source: <https://github.com/tofan79/mindset-noctalia-plugins>

These two are the recommended ones: between them they cover the fiddly parts of
this config — switching tiling layouts (including your own `deck.lua` and
`fair.lua`) and picking an animation preset — without hand-editing Lua.

| Plugin | API | What it does |
|---|---|---|
| `mindset/hypr-layouts` | 13 | Per-workspace tiling layout picker. Auto-detects your custom `layouts/*.lua` on top of the built-ins |
| `mindset/hypr-animations` | 13 | Animation preset picker, reads `config/animations_presets/` |

### Install

```bash
# register the source (Noctalia fetches it at startup)
noctalia msg plugins source add mindset git https://github.com/tofan79/mindset-noctalia-plugins.git

# enable what you want
noctalia msg plugins enable mindset/hypr-layouts
noctalia msg plugins enable mindset/hypr-animations
```

Verify with `noctalia msg plugins list | grep mindset`.

Edits inside a `.luau` plugin file hot-reload. Changes to the manifest
(`plugin.toml`) need a full config reload.

`hypr-layouts` is already wired to `SUPER + ALT + W` in `config/binds.lua` via
`mindset/hypr-layouts:panel`, so enabling that plugin is all it takes for that
shortcut to work. `hypr-animations` has no bind — open it from the Noctalia
plugin list.

### GloView (Mission Control-style overview)

GloView is installed from the `rakuos` repository (`gloview-git`, 0.3.0) and is
loaded at startup in `hyprland.lua`:

```lua
hl.plugin.load("/usr/lib64/gloview.so")
```

The binds live at the end of `config/binds.lua`, inside a guard so they stay
harmless if the package is ever removed:

| Shortcut | Action |
|---|---|
| `SUPER + SHIFT + O` | Toggle overview |
| `SUPER + ALT + O` | Desktop mode |
| `SUPER + CTRL + O` | All workspaces |

```lua
if hl.plugin.gloview then
    hl.bind(M .. " + SHIFT + O", hl.plugin.gloview.toggle, { description = "Toggle overview" })
    hl.bind(M .. " + ALT + O", hl.plugin.gloview.desktop, { description = "Desktop mode" })
    hl.bind(M .. " + CTRL + O", hl.plugin.gloview.allworkspaces, { description = "All workspaces" })
end
```

Those two files are the only places that reference it. To disable GloView,
comment the `hl.plugin.load` line; the guarded binds then register nothing and
raise no error.


---

## Everything else (if you need it)

| File | Contents |
|---|---|
| `config/inputs.lua` | Mouse sensitivity, touchscreen gestures, natural scroll |
| `config/windowrules.lua` | Per-app rules: float, size, opacity, auto-routing to the `gaming` workspace |
| `config/autostart.lua` | Runs on Hyprland start (`noctalia`, clipboard history) |
| `config/lid.lua` | Close the laptop lid → lock and suspend |
| `config/environment.lua` | NVIDIA environment variables (commented out). If you use UWSM, edit `~/.config/uwsm/env` instead |
| `noctalia.lua` | **Generated** — theme colors. Overwritten whenever the theme changes in Noctalia |

---

## When something breaks

Check first:

```bash
hyprctl configerrors
```

Then reload. `SUPER + SHIFT + R` is safe to use even in a broken state, since
`~/.config/hypr` can be restored from git.
