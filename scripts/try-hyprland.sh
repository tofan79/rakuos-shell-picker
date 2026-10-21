#!/usr/bin/env bash
# Jalankan picker dari sesi Hyprland: build, lihat rencana switch tanpa
# mengubah apa pun, lalu buka GUI-nya.
#
#   ./scripts/try-hyprland.sh          # dry-run semua shell, lalu buka GUI
#   ./scripts/try-hyprland.sh --dry    # berhenti setelah dry-run
#
# Dry-run tidak menyentuh apa pun — hanya membaca config di $HOME dan mencetak
# langkah yang *akan* dijalankan. GUI-nya yang benar-benar stop/install/apply.

set -uo pipefail

cd "$(dirname "$0")/.." || exit 2

bin=./target/debug/rakuos-shell-picker
# Setiap shell yang punya port Hyprland. `inir` tidak ada di sini: ia hanya
# punya port niri.
shells=(noctalia dms caelestia ambxst end4pc)

echo "== cargo build =="
cargo build || exit 1

rc=0
for shell in "${shells[@]}"; do
  echo
  echo "== dry-run: hyprland $shell =="
  if ! "$bin" --dry-run hyprland "$shell"; then
    rc=1
  fi
done

if [ "${1:-}" = "--dry" ]; then
  exit "$rc"
fi

echo
if [ -z "${HYPRLAND_INSTANCE_SIGNATURE:-}" ]; then
  echo "PERINGATAN: HYPRLAND_INSTANCE_SIGNATURE kosong — ini bukan sesi Hyprland."
  echo "GUI akan menolak switch sampai dijalankan di dalam Hyprland."
fi

echo "== membuka GUI =="
exec "$bin"
