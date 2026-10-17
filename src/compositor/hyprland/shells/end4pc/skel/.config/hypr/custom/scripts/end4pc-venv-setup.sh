#!/bin/sh
# Bootstrap venv Python ii/end4-pC, dipanggil dari custom/execs.lua.
#
# Skrip warna dan thumbnail ii menjalankan Python lewat
# $ILLOGICAL_IMPULSE_VIRTUAL_ENV (diatur di hyprland/env.lua). Venv dibuat di
# sini; ia tinggal selama end4-pC terpasang dan dihapus (discard, tanpa backup)
# saat pindah keluar — bootstrap inilah yang membuatnya lagi pada instalasi
# berikutnya. Idempoten: aman dipanggil pada setiap login — login setelah yang
# pertama hanya keluar.
#
# Tanpa venv shell tetap tampil; hanya fitur warna/thumbnail yang redup.
# Semua keluhan masuk ke $LOG supaya tidak menghilang tanpa jejak.

set -u

VENV="$HOME/.local/state/quickshell/.venv"
REQ="/usr/share/illogical-impulse/sdata/uv/requirements.txt"
LOG="$HOME/.local/state/quickshell/venv-setup.log"

# Selesai = python ada DAN tahap pip sudah pernah tuntas (penanda ditulis
# hanya saat sukses, jadi kegagalan di tengah dicoba lagi login berikutnya).
[ -x "$VENV/bin/python" ] && [ -e "$VENV/.requirements-installed" ] && exit 0

command -v uv >/dev/null 2>&1 || exit 0
[ -f "$REQ" ] || exit 0

mkdir -p "$(dirname "$VENV")"

{
    echo "== $(date -Is) — bootstrap venv =="
    # Fedora's uv forbids interpreter downloads by default and prefers system
    # Pythons (/etc/uv/uv.toml) — while Fedora 44 ships Python 3.14, and ii's
    # pinned requirements were compiled against 3.12. The one managed download
    # is allowed for machines with no system 3.12; where one exists, uv still
    # prefers it. `--clear` because a failed earlier attempt leaves the venv
    # half-made, and this script runs again; a finished one exits above.
    if UV_PYTHON_DOWNLOADS=automatic uv venv --clear --prompt .venv "$VENV" -p 3.12 &&
       uv pip install --python "$VENV/bin/python" -r "$REQ"; then
        touch "$VENV/.requirements-installed"
        echo "== selesai =="
    else
        echo "== gagal; akan dicoba lagi saat login berikutnya =="
        exit 1
    fi
} >>"$LOG" 2>&1
