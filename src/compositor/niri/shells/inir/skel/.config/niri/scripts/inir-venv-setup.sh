#!/bin/sh
# Bootstrap venv Python iNiR, dipanggil dari config.d/50-startup.kdl.
#
# Skrip warna, thumbnail, dan AI iNiR menjalankan Python lewat $INIR_VENV
# (diatur di config.d/40-environment.kdl, di lingkungan Niri). Venv dibuat di
# sini; ia tinggal selama iNiR terpasang dan dihapus (discard, tanpa backup)
# saat pindah keluar — bootstrap inilah yang membuatnya lagi pada instalasi
# berikutnya. Idempoten: aman dipanggil pada setiap login — login setelah yang
# pertama hanya keluar.
#
# Tanpa venv shell tetap tampil; yang redup hanya fitur yang lewat Python:
# palet warna dari wallpaper, thumbnail, dan AI. Semua keluhan masuk ke $LOG
# supaya tidak menghilang tanpa jejak.
#
# Tempatnya di pohon config Niri, bukan di ~/.config/illogical-impulse, karena
# config compositor selalu datang dari skel pada setiap switch, sedangkan isi
# config_dir shell datang dari backup kalau ada backup — dan skrip yang tidak
# pernah terpasang berarti venv yang tidak pernah dibangun. Baris 50-startup.kdl
# yang memanggilnya ikut di pohon yang sama, jadi keduanya tak bisa berpisah.
#
# uv ada di plan paket picker, begitu juga cairo-devel, cairo-gobject-devel,
# gobject-introspection-devel, dan dbus-devel. Yang cairo dan
# gobject-introspection dipakai membangun pycairo dan pygobject dari sumber
# (keduanya tanpa wheel Linux di PyPI), dan cairo-gobject.h yang dipakai
# pygobject datang dari paketnya sendiri, cairo-gobject-devel; dbus-devel
# menyediakan dbus-1.pc untuk membangun dbus-python, yang ditarik
# kde-material-you-colors. Lihat src/shells/inir.rs.

set -u

VENV="$HOME/.local/state/quickshell/.venv"
REQ="/usr/share/quickshell/inir/sdata/uv/requirements.txt"
LOG="$HOME/.local/state/quickshell/venv-setup.log"

# Selesai = python ada DAN tahap pip sudah pernah tuntas (penanda ditulis
# hanya saat sukses, jadi kegagalan di tengah dicoba lagi login berikutnya).
[ -x "$VENV/bin/python" ] && [ -e "$VENV/.requirements-installed" ] && exit 0

command -v uv >/dev/null 2>&1 || exit 0
[ -f "$REQ" ] || exit 0

mkdir -p "$(dirname "$VENV")"

{
    echo "== $(date -Is) — bootstrap venv iNiR =="
    # Fedora's uv forbids interpreter downloads by default and prefers system
    # Pythons (/etc/uv/uv.toml) — while Fedora 44 ships Python 3.14, and iNiR's
    # requirements were compiled against 3.12. The one managed download is
    # allowed for machines with no system 3.12; where one exists, uv still
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
