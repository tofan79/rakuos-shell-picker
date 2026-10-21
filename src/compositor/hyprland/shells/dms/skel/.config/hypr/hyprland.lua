-- RakuOS Hyprland configuration for DMS
--
-- Only the entry point. Everything else lives under dms/: rakuos.lua holds
-- what DMS must not rewrite, the rest are the modules DMS itself owns.

require("dms.rakuos")
require("dms.colors")
require("dms.outputs")
require("dms.layout")
require("dms.cursor")
require("dms.input")
require("dms.binds")
require("dms.binds-user")
require("dms.windowrules")
