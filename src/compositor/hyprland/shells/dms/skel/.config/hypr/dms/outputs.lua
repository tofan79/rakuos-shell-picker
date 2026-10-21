-- Monitor rule. DMS writes the real display layout here once a monitor is
-- rearranged in its Settings; until then this is the baseline: output empty so
-- Hyprland picks whatever is connected, preferred mode, automatic position and
-- scale 1.
hl.monitor({
    output    = MONITOR1,
    mode      = "preferred",
    position  = "auto",
    scale     = "1",
})
