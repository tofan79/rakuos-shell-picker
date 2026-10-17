
-- end4-pC: bootstrap venv Python milik ii/end4-pC (dipakai skrip warna dan
-- thumbnail lewat $ILLOGICAL_IMPULSE_VIRTUAL_ENV, lihat hyprland/env.lua).
-- Idempoten: skrip keluar sendiri kalau venv sudah siap, aman tiap login.
hl.on("hyprland.start", function()
    hl.exec_cmd("$HOME/.config/hypr/custom/scripts/end4pc-venv-setup.sh")
end)

-- end4-pC: default tampilan sistem (ikon Colloid-Dark, kursor 24, Inter 11,
-- Maple Mono NF 11) lewat Settings -> General -> System Appearance. Hanya
-- diisi kalau belum diatur, lalu kursor yang tersimpan diterapkan — jadi
-- pilihan pengguna tidak ditimpa tiap login.
hl.on("hyprland.start", function()
    hl.exec_cmd("$HOME/.config/hypr/custom/scripts/end4pc-appearance-defaults.sh")
end)
