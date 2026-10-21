-- DMS writes the cursor theme and size here when that setting changes. These
-- are the RakuOS defaults it starts from.
--
-- hyprland.lua requires it, so it has to exist before DMS runs: see the note in
-- layout.lua for why a failing require is not survivable.
hl.env("HYPRCURSOR_THEME", "Bibata-Modern-Ice")
hl.env("XCURSOR_THEME", "Bibata-Modern-Ice")
hl.env("HYPRCURSOR_SIZE", "24")
hl.env("XCURSOR_SIZE", "24")
