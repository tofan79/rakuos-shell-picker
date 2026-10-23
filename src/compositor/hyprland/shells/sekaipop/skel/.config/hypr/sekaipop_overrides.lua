hl.layer_rule({ name = "sekaipop-region-no-anim", match = { namespace = "^sekaipop-region$" }, no_anim = true }) -- sekaipop:region-no-anim
hl.layer_rule({ name = "sekaipop-region-preview-no-anim", match = { namespace = "^sekaipop-region-preview$" }, no_anim = true }) -- sekaipop:region-preview-no-anim
hl.config({ ["general"] = { ["gaps_out"] = 14 } }) -- sekaipop:hypr:general.gaps_out
hl.config({ ["general"] = { ["gaps_in"] = 6 } }) -- sekaipop:hypr:general.gaps_in
hl.config({ ["decoration"] = { ["inactive_opacity"] = 1.00 } }) -- sekaipop:hypr:decoration.inactive_opacity
hl.config({ ["general"] = { ["border_size"] = 2 } }) -- sekaipop:hypr:general.border_size
hl.config({ ["misc"] = { ["animate_manual_resizes"] = true } }) -- sekaipop:hypr:misc.animate_manual_resizes
hl.animation({ leaf = "workspaces", enabled = true, speed = 7.00, bezier = "menu_decel", style = "slidefadevert" }) -- sekaipop:hypr:animation.workspaces
pcall(hl.config, { ["general"] = { ["col"] = { ["active_border"] = "rgba(00feeeee)", ["inactive_border"] = "rgba(00757bcc)" } } }) -- sekaipop:apptheme:hyprland
