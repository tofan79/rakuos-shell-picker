#!/usr/bin/env bash
# Jalankan picker dari sesi Niri: build, lihat rencana switch tanpa mengubah
# apa pun, lalu buka GUI-nya.
#
#   ./scripts/try-niri.sh          # dry-run DMS + Noctalia + iNiR, lalu buka GUI
#   ./scripts/try-niri.sh --dry    # berhenti setelah dry-run
#
# Dry-run tidak menyentuh apa pun — hanya membaca config di $HOME dan mencetak
# langkah yang *akan* dijalankan. GUI-nya yang benar-benar stop/install/apply.
# Snapshot VM dulu sebelum mode GUI.

set -uo pipefail

cd "$(dirname "$0")/.." || exit 2

bin=./target/debug/rakuos-shell-picker

echo "== cargo build =="
cargo build || exit 1

rc=0
for shell in dms noctalia inir; do
  echo
  echo "== dry-run: niri $shell =="
  if ! "$bin" --dry-run niri "$shell"; then
    rc=1
  fi
done

if [ "${1:-}" = "--dry" ]; then
  exit "$rc"
fi

echo
if [ -z "${NIRI_SOCKET:-}" ]; then
  echo "PERINGATAN: NIRI_SOCKET kosong — ini bukan sesi Niri."
  echo "GUI akan menolak switch sampai dijalankan di dalam Niri."
fi

echo "== membuka GUI =="
exec "$bin"
