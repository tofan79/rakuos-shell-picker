#!/bin/sh
# Tampilan sistem default ii/end4-pC, dipanggil dari custom/execs.lua.
#
# Nilai yang diinginkan fork — ikon Colloid-Dark, kursor ukuran 24, font
# Inter 11 dan Maple Mono NF 11 — semuanya diatur lewat skrip milik fork
# sendiri (scripts/system-appearance.sh di paket), jalur yang sama dengan
# Settings -> General -> System Appearance.
#
# Hanya diisi kalau belum diatur: pilihan pengguna di Settings tidak pernah
# ditimpa, dan tiap login skrip ini tinggal menerapkan kursor yang tersimpan.
# Di mesin yang sudah diatur lengkap (mis. RakuOS bawaan) skrip praktis no-op.

set -u

SAP="/usr/share/quickshell/end4-pC/scripts/system-appearance.sh"
[ -x "$SAP" ] || exit 0
command -v kreadconfig6 >/dev/null 2>&1 || exit 0

kread() {
    kreadconfig6 --file "$1" --group "$2" --key "$3" 2>/dev/null
}

# Ikon: hanya kalau temanya benar-benar ada di mesin, supaya mesin tanpa
# Colloid tidak berakhir dengan set ikon yang tidak resolve.
if [ -z "$(kread kdeglobals Icons Theme)" ] &&
    { [ -d /usr/share/icons/Colloid-Dark ] ||
        [ -d "$HOME/.local/share/icons/Colloid-Dark" ] ||
        [ -d "$HOME/.icons/Colloid-Dark" ]; }; then
    "$SAP" --set-icons Colloid-Dark
fi

# Font: Qt (kdeglobals) dan GTK (gsettings) sekaligus.
[ -z "$(kread kdeglobals General font)" ] && "$SAP" --set-font ui Inter 11
[ -z "$(kread kdeglobals General fixed)" ] && "$SAP" --set-font mono "Maple Mono NF" 11

# Kursor: ukuran default 24; temanya ikut pilihan pengguna kalau sudah ada,
# kalau belum baru Bibata-Modern-Classic (default upstream).
cursor_theme=$(kread kcminputrc Mouse cursorTheme)
cursor_size=$(kread kcminputrc Mouse cursorSize)
if [ -z "$cursor_theme" ]; then
    "$SAP" --set-cursor Bibata-Modern-Classic 24
elif [ -z "$cursor_size" ]; then
    "$SAP" --set-cursor "$cursor_theme" 24
fi

# Terapkan kursor yang tersimpan ke Hyprland — pengganti baris hardcoded
# `hyprctl setcursor` yang dulu ada di hyprland/execs.lua.
"$SAP" --apply-cursor

exit 0
